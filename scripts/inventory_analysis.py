#!/usr/bin/env python3
"""
Analyze save-file inventory format.

Correct spec:
- Quest items (no header): null-terminated name only, at start of inventory
- Standard items: 12B header [type(u32)][id(u32)][qty(u16)][pad(u16)] +
  variable name\0 + variable desc\0 + 4B price(i32) + padding

ItemTypeId: Weapon=1, Healing=2, Edit=3, Event=4, Misc=5
"""

import struct
import sys
import os

ITEM_TYPES = {
    1: "Weapon",
    2: "Healing",
    3: "Edit",
    4: "Event",
    5: "Misc",
}

HEADER_12 = 12
NAME_MAX = 30
DESC_MAX = 232
PRICE_SIZE = 4

def read_save(path):
    with open(path, "rb") as f:
        return f.read()

def find_inventory_bounds(data):
    TAIL = 2251 * 284 + 114 + 3 * 100 * 37
    events_start = len(data) - TAIL
    
    sprite_marker = b"inter\\"
    pos = events_start
    while pos > 0:
        idx = data.rfind(sprite_marker, 0, pos)
        if idx == -1:
            break
        candidate = idx
        while candidate >= 60 and data[candidate-60:candidate-54] == sprite_marker:
            candidate -= 60
        hdr_start = candidate - 8
        if hdr_start >= 0:
            u1, u2 = struct.unpack_from('<II', data, hdr_start)
            if u1 == 7 and u2 == 7:
                cd_start = candidate + 240
                if cd_start + 118 <= events_start:
                    inv_start = cd_start + 114
                    inv_end = find_96_block(data, inv_start)
                    return inv_start, inv_end
        pos = idx
    return None, None

def find_96_block(data, start):
    pos = start
    while pos + 96 + 24 <= len(data):
        zero_count = sum(1 for i in range(96) if data[pos + i] == 0)
        if zero_count >= 72:
            after = data[pos + 96:]
            name_raw = after[:11]
            name_len = next((i for i, b in enumerate(name_raw) if b == 0), 11)
            if 3 <= name_len <= 10 and 65 <= name_raw[0] <= 90:
                cid = struct.unpack_from('<h', after, 11)[0]
                if 1 <= cid <= 12:
                    return pos
        pos += 1
    return None

def try_parse_standard(data, offset):
    """Try to parse a standard item with 12B header at offset."""
    if offset + HEADER_12 > len(data):
        return None
    
    type_val = struct.unpack_from('<I', data, offset)[0]
    item_type = type_val & 0xFF
    if item_type not in ITEM_TYPES:
        return None
    
    item_id = struct.unpack_from('<I', data, offset + 4)[0]
    qty = struct.unpack_from('<H', data, offset + 8)[0]
    
    # Read null-terminated name after header
    name_start = offset + HEADER_12
    name_end = name_start
    while name_end < len(data) and data[name_end] != 0:
        name_end += 1
    if name_end >= len(data) or name_end == name_start:
        return None  # empty name = not valid
    
    # Read null-terminated description
    desc_start = name_end + 1
    desc_end = desc_start
    while desc_end < len(data) and data[desc_end] != 0:
        desc_end += 1
    if desc_end > len(data):
        return None
    
    # Read price (4 bytes after desc)
    price_start = desc_end + 1
    price = struct.unpack_from('<i', data, price_start)[0] if price_start + 4 <= len(data) else None
    
    try:
        name = data[name_start:name_end].decode('cp1250')
    except:
        name = repr(data[name_start:name_end])
    try:
        desc = data[desc_start:desc_end].decode('cp1250')
    except:
        desc = repr(data[desc_start:desc_end])
    
    # Record ends after price
    record_end = price_start + PRICE_SIZE
    
    return {
        'offset': offset,
        'type_val': type_val,
        'item_type': item_type,
        'type_name': ITEM_TYPES[item_type],
        'item_id': item_id,
        'qty': qty,
        'name': name,
        'name_len': name_end - name_start,
        'desc': desc,
        'desc_len': desc_end - desc_start,
        'price': price,
        'record_end': record_end,
    }

def fmt_hex(data, off, n=16):
    return ' '.join(f'{data[off+i]:02x}' for i in range(min(n, len(data)-off)))

def analyze_inventory(data, label):
    inv_start, inv_end = find_inventory_bounds(data)
    if inv_start is None:
        print(f"{label}: Cannot find inventory")
        return
    
    inv = data[inv_start:inv_end]
    print(f"\n{'='*80}")
    print(f"{label}: {len(data)}B, inv={inv_start:#x}-{inv_end:#x} ({len(inv)}B)")
    print(f"{'='*80}\n")
    
    items = []
    pos = 0
    standard_idx = 0
    
    while pos < len(inv):
        # Skip zero bytes
        while pos < len(inv) and inv[pos] == 0:
            pos += 1
        if pos >= len(inv):
            break
        
        remaining = len(inv) - pos
        
        # Try standard item (12B header)
        item = try_parse_standard(inv, pos) if remaining >= 12 else None
        
        if item:
            type_name = item['type_name']
            rid = item['item_id']
            qty = item['qty']
            name = item['name']
            desc = item['desc']
            price = item['price']
            record_end = item['record_end']
            next_off = pos + (record_end - pos)
            
            # Find actual next item (skip padding)
            actual_next = next_off
            while actual_next < len(inv) and inv[actual_next] == 0:
                actual_next += 1
            
            actual_gap = actual_next - pos
            
            print(f"ITEM #{standard_idx} at +{pos:#05x} [{type_name}] id={rid} qty={qty}")
            print(f"  Name ({item['name_len']}B): '{name}'")
            if desc:
                print(f"  Desc ({item['desc_len']}B): '{desc}'")
            else:
                print(f"  Desc: (empty)")
            print(f"  Price: {price}  Record: {record_end-pos}B  Gap: {actual_gap}B")
            print(f"  Next: +{actual_next:#05x}  (padding: {actual_next - next_off}B)")
            print(f"  Raw header: {fmt_hex(inv, pos, 12)}")
            name_start = pos + 12
            search_end = min(name_start + 48, len(inv))
            nend = name_start
            while nend < search_end and inv[nend] != 0:
                nend += 1
            raw_name_len = min(nend - name_start + 1, 32)
            if raw_name_len > 0:
                raw_hex = ' '.join(f'{inv[name_start+i]:02x}' for i in range(raw_name_len))
                print(f"  Raw name:   {raw_hex}")
            
            items.append({
                'idx': standard_idx, 'offset': pos, 'type': type_name,
                'name': name, 'gap': actual_gap
            })
            standard_idx += 1
            pos = next_off
            continue
        
        # Try as quest item (name without header)
        name_end = pos
        while name_end < len(inv) and inv[name_end] != 0:
            name_end += 1
        if name_end > pos and name_end < len(inv):
            try:
                qname = inv[pos:name_end].decode('cp1250')
            except:
                qname = repr(inv[pos:name_end])
            print(f"QUEST at +{pos:#05x}: '{qname}' ({name_end-pos}B)")
            
            items.append({'idx': 'Q', 'offset': pos, 'type': 'Quest', 'name': qname})
            pos = name_end + 1
            continue
        
        pos += 1
    
    # Summary
    print(f"\n{'─'*60}")
    print(f"Total: {len(items)} items")
    std_items = [i for i in items if i['type'] != 'Quest']
    if std_items:
        print(f"Standard item gaps:")
        for i in range(len(std_items)-1):
            g = std_items[i+1]['offset'] - std_items[i]['offset']
            print(f"  #{std_items[i]['idx']:>3} +{std_items[i]['offset']:#06x} -> #{std_items[i+1]['idx']:>3} "
                  f"+{std_items[i+1]['offset']:#06x}: gap={g}B name='{std_items[i]['name'][:24]}'")

def main():
    for path in sys.argv[1:]:
        if not os.path.exists(path):
            print(f"Not found: {path}")
            continue
        analyze_inventory(read_save(path), os.path.basename(path))

if __name__ == "__main__":
    main()
