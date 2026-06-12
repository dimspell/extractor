#!/usr/bin/env python3
"""
Dispel RPG Save File Extractor

Reads .sav files and extracts all known data structures:
  - Player identity (name, class, stats)
  - Surface monsters, NPCs, objects
  - Event scripts (quest states)
  - Journal entries (main/side/trade)
  - Inventory items (best-effort text extraction)

Usage:
  python3 scripts/save_extractor.py <path/to/save.sav>
  python3 scripts/save_extractor.py nuno-0.sav  --json    # JSON output
  python3 scripts/save_extractor.py 0.sav       --brief    # Compact listing
"""

import json
import struct
import sys


# ── Constants ──────────────────────────────────────────────────────────────

WINDOWS_1250 = "cp1250"

EVENT_SIZE = 284
EVENT_COUNT = 2251           # 2250 script events + 1 null header
EVENTS_SIZE = EVENT_COUNT * EVENT_SIZE  # 639,284

UNKNOWN_SIZE = 114
JOURNAL_PAGE_SIZE = 100 * 37  # 3,700
JOURNAL_SIZE = 3 * JOURNAL_PAGE_SIZE  # 11,100

TAIL_SIZE = EVENTS_SIZE + UNKNOWN_SIZE + JOURNAL_SIZE  # 650,498

INVENTORY_RECORD = 4 + 30 + 234 + 4  # 272


# ── Text helpers ──────────────────────────────────────────────────────────────

def decode_cp1250(data: bytes) -> str:
    """Decode WINDOWS-1250 bytes, replacing unrepresentable chars."""
    return data.decode(WINDOWS_1250, errors="replace")


def extract_text(buf: bytes) -> str:
    """
    Find the first readable CP1250 text segment >= 2 chars starting with a
    letter (ASCII or extended Latin).  Skips binary junk / single-character
    false positives (e.g. ``%``, ``2``, ``=``).
    """
    i = 0
    while i < len(buf):
        b = buf[i]
        is_text = (0x21 <= b <= 0x7E) or b in (0x20, 0x09) or b >= 0x80
        if is_text:
            seg_start = i
            while i < len(buf):
                bb = buf[i]
                if bb == 0 or not ((0x21 <= bb <= 0x7E) or bb in (0x20, 0x09) or bb >= 0x80):
                    break
                i += 1
            seg_len = i - seg_start
            first_char = chr(buf[seg_start]) if buf[seg_start] < 0x80 else "?"
            if seg_len >= 2 and (first_char.isalpha() or buf[seg_start] >= 0x80):
                return buf[seg_start:i].decode(WINDOWS_1250, errors="replace").strip()
        else:
            i += 1
    return ""


def extract_name_or_desc(name_buf: bytes, desc_buf: bytes) -> str:
    """Try name buffer first, fall back to description buffer."""
    name = extract_text(name_buf)
    if name:
        return name
    return extract_text(desc_buf)


def read_cstr(data: bytes, offset: int, max_len: int) -> str:
    """Read a null-terminated CP1250 string up to *max_len* bytes."""
    end = offset
    while end < offset + max_len and end < len(data) and data[end] != 0:
        end += 1
    return decode_cp1250(data[offset:end])


def read_u16(data: bytes, offset: int) -> int:
    return struct.unpack_from("<H", data, offset)[0]


def read_u32(data: bytes, offset: int) -> int:
    return struct.unpack_from("<I", data, offset)[0]


def read_i16(data: bytes, offset: int) -> int:
    return struct.unpack_from("<h", data, offset)[0]


# ── Parsers ────────────────────────────────────────────────────────────────────

def parse_header(data: bytes) -> dict:
    """First 12 bytes – game version / identifier."""
    return {
        "raw_hex": data[:12].hex(),
    }


def parse_monster_record(data: bytes) -> dict:
    """329-byte monster record."""
    return {
        "signature_a": read_u32(data, 0),
        "record_index": read_u32(data, 4),
        "signature_b": read_u32(data, 8),
        "name": read_cstr(data, 12, 24).strip(),
        "hp_current": read_u16(data, 36),
        "hp_maximum": read_u16(data, 38),
        "state_flags": read_u32(data, 40),
        "tile_x": read_u16(data, 44),
        "tile_y": read_u16(data, 46),
        "pixel_x": read_u16(data, 48),
        "pixel_y": read_u16(data, 50),
        "facing_direction": data[52],
        # skip 3 bytes padding at 53-55
        "experience": read_u32(data, 56),
        "attack": read_u16(data, 60),
        "defense": read_u16(data, 62),
        "magic_attack": read_u16(data, 64),
        "magic_defense": read_u16(data, 66),
        "agility": read_u16(data, 68),
        "luck": read_u16(data, 70),
    }


def parse_npc_record(data: bytes) -> dict:
    """349-byte NPC record."""
    return {
        "counter1": read_u32(data, 0),
        "counter2": read_u32(data, 4),
        "counter3": read_u32(data, 8),
        "counter4": read_u32(data, 12),
        "name": read_cstr(data, 48, 32).strip(),
        "role": read_cstr(data, 120, 40).strip(),
    }


def parse_surface_object(data: bytes) -> dict:
    """200-byte surface extra object."""
    prefix = data[14] if len(data) > 14 else 0
    name = read_cstr(data, 15, 185).strip()
    state = 0
    if "Skrzynia" in name:
        name_end = data.find(b"\x00", 15)
        state = data[name_end + 1] if name_end >= 15 and name_end + 1 < len(data) else 0
    return {
        "prefix": prefix,
        "name": name,
        "state": state,
    }


def parse_event_script(data: bytes) -> dict:
    """284-byte event script record."""
    return {
        "event_id": read_u32(data, 0),
        "state": read_u32(data, 8),  # 0=inactive, 1=active, 2=completed
        "script_name": read_cstr(data, 12, 272).strip(),
    }


def parse_journal_entry(data: bytes) -> dict:
    """37-byte journal entry."""
    return {
        "counter": data[0],
        "name": read_cstr(data, 1, 32).strip(),
        "flags": read_u32(data, 33),
    }


def parse_inventory(inv_data: bytes) -> list:
    """
    Parse inventory items from raw inventory area.

    Layout: [quest_items...][standard items: N×272B]
    Quest items have no header – just null-terminated names.
    Standard items: [type: u32(4B)][name: 30B][desc: 234B][price: i32(4B)]
    """
    items = []
    pos = 0
    while pos < len(inv_data):
        # Skip zero bytes
        while pos < len(inv_data) and inv_data[pos] == 0:
            pos += 1
        if pos >= len(inv_data):
            break

        # Try standard 272B record
        if pos + INVENTORY_RECORD <= len(inv_data):
            type_val = read_u32(inv_data, pos)
            type_byte = type_val & 0xFF
            if 1 <= type_byte <= 5:
                name_buf = inv_data[pos + 4: pos + 4 + 30]
                desc_buf = inv_data[pos + 34: pos + 34 + 234]
                price = struct.unpack_from("<i", inv_data, pos + 4 + 30 + 234)[0]
                name = extract_name_or_desc(name_buf, desc_buf)
                desc = extract_text(desc_buf)

                if name:
                    items.append({
                        "location_raw": list(type_val.to_bytes(4, "little")),
                        "is_quest": False,
                        "name": name,
                        "description": desc,
                        "price": price,
                    })
                pos += INVENTORY_RECORD
                continue

        # Quest item – null-terminated name
        name_end = pos
        while name_end < len(inv_data) and inv_data[name_end] != 0:
            name_end += 1
        if name_end > pos and name_end < len(inv_data):
            items.append({
                "location_raw": [0, 0, 0, 0],
                "is_quest": True,
                "name": decode_cp1250(inv_data[pos:name_end]),
                "description": "",
                "price": 0,
            })
            pos = name_end + 1
            continue

        pos += 1

    return items


# ── Character data extraction ──────────────────────────────────────────────────

def find_character_data_start(data: bytes, events_start: int):
    """
    The sprite-path block is 248 bytes (8 header + 4×60B null-terminated paths).
    Paths 0-1 start with ``inter\\``, paths 2-3 with ``CharacterInGame\\``.
    Returns the offset of character data (cd_start) or None.
    """
    sprite_marker = b"inter\\"
    pos = events_start - 1
    while pos > 0:
        idx = data.rfind(sprite_marker, 0, pos)
        if idx == -1:
            break
        # Walk backward by 60-byte strides to find first sprite path
        first = idx
        while first >= 60 and data[first - 60:first - 54] == sprite_marker:
            first -= 60
        # Block header is 8 bytes before first path; total block = 8 + 240 = 248
        cd_start = first + 240
        if cd_start <= events_start and cd_start + 118 <= len(data):
            return cd_start
        pos = idx
    return None


def find_inventory_end(data: bytes, inv_start: int):
    """
    Scan forward from inv_start for a 96-byte block with >= 72 zeros,
    followed by a valid player name pattern (uppercase start, class 1-12).
    """
    pos = inv_start
    while pos + 96 + 24 <= len(data):
        zero_count = sum(1 for i in range(96) if data[pos + i] == 0)
        if zero_count >= 72:
            after = data[pos + 96:]
            name_raw = after[:11]
            name_len = next((i for i, b in enumerate(name_raw) if b == 0), 11)
            if 3 <= name_len <= 10 and 0x41 <= name_raw[0] <= 0x5A:
                if 1 <= read_i16(after, 11) <= 12:
                    return pos
        pos += 1
    return None


def parse_player_attributes_24(data_24: bytes) -> dict:
    """
    24-byte save-file attribute layout (no MP fields):
      STR/DEX/WIS/CON/LCK (5×u16) + HP_CUR/HP_MAX (2×u16) + XP (u32) + LVL (u16) + GOLD (u32)
    """
    if len(data_24) < 24:
        return {}
    return {
        "strength": read_u16(data_24, 0),
        "dexterity": read_u16(data_24, 2),
        "wisdom": read_u16(data_24, 4),
        "constitution": read_u16(data_24, 6),
        "luck": read_u16(data_24, 8),
        "hp_current": read_u16(data_24, 10),
        "hp_maximum": read_u16(data_24, 12),
        "xp_current": read_u32(data_24, 14),
        "level": read_u16(data_24, 18),
        "gold": read_u32(data_24, 20),
    }


def extract_player_identity(data: bytes) -> dict:
    """
    Scan backward for the 96-byte zero block + player name + class pattern.
    """
    if len(data) < 150:
        return {}
    max_offset = len(data) - 120
    for offset in range(max_offset, -1, -1):
        zero_count = sum(1 for i in range(96) if data[offset + i] == 0)
        if zero_count >= 72:
            after = data[offset + 96:]
            name_raw = after[:11]
            name_len = next((i for i, b in enumerate(name_raw) if b == 0), 11)
            if 3 <= name_len <= 10 and 0x41 <= name_raw[0] <= 0x5A:
                if not all(b >= 0x20 for b in name_raw[:name_len]):
                    continue
                cid = read_i16(after, 11)
                if not (1 <= cid <= 12):
                    continue
                cls_raw = after[13:24]
                cls_len = next((i for i, b in enumerate(cls_raw) if b == 0), 11)
                if cls_len < 3 or cls_len > 10:
                    continue
                if not all(b >= 0x20 for b in cls_raw[:cls_len]):
                    continue
                return {
                    "player_name": decode_cp1250(name_raw[:name_len]),
                    "player_class_id": cid,
                    "player_class_name": decode_cp1250(cls_raw[:cls_len]).strip(),
                }
    return {}


# ── Main parser ────────────────────────────────────────────────────────────────

def parse_save_file(data: bytes) -> dict:
    """Parse a complete Dispel .sav file into a structured dict."""
    result = {}

    # 1. Header
    result["header"] = parse_header(data)
    pos = 12

    # 2. Surface monsters
    monster_count = read_u32(data, pos)
    pos += 4
    MONSTER_SIZE = 329
    monsters_raw = data[pos: pos + monster_count * MONSTER_SIZE]
    result["surface_monsters"] = [parse_monster_record(monsters_raw[i:i + MONSTER_SIZE])
                                  for i in range(0, len(monsters_raw), MONSTER_SIZE)]
    result["surface_monster_count"] = monster_count
    pos += monster_count * MONSTER_SIZE

    # 3. NPCs
    npc_count = read_u32(data, pos)
    pos += 4
    NPC_SIZE = 349
    npcs_raw = data[pos: pos + npc_count * NPC_SIZE]
    result["npcs"] = [parse_npc_record(npcs_raw[i:i + NPC_SIZE])
                      for i in range(0, len(npcs_raw), NPC_SIZE)]
    result["npc_count"] = npc_count
    pos += npc_count * NPC_SIZE

    # 4. Surface objects
    separator = read_u32(data, pos)           # always 0
    pos += 4
    obj_count = read_u32(data, pos)
    pos += 4
    OBJ_SIZE = 200
    objs_raw = data[pos: pos + obj_count * OBJ_SIZE]
    result["surface_objects"] = [parse_surface_object(objs_raw[i:i + OBJ_SIZE])
                                  for i in range(0, len(objs_raw), OBJ_SIZE)]
    result["surface_object_count"] = obj_count
    result["surface_object_separator"] = separator
    pos += obj_count * OBJ_SIZE

    # 5. Remaining data (everything after surface objects to EOF)
    remaining = data[pos:]
    result["remaining_size"] = len(remaining)

    # 6. Tail sections: events + journal (fixed position from EOF)
    result["events"] = []
    result["journal_main"] = []
    result["journal_side"] = []
    result["journal_trade"] = []
    result["player_attributes"] = {}
    result["character_details"] = []
    result["extra_character_data"] = []
    result["inventory_items"] = []

    if len(remaining) >= TAIL_SIZE:
        events_start_offset = len(data) - TAIL_SIZE  # absolute offset in data

        # Validate first event (null event at index 0 should have event_id=0)
        if data[events_start_offset:events_start_offset + 4] == b"\x00\x00\x00\x00":
            # Parse events
            result["events"] = [
                parse_event_script(data[events_start_offset + i * EVENT_SIZE:
                                        events_start_offset + (i + 1) * EVENT_SIZE])
                for i in range(EVENT_COUNT)
            ]

            # Parse journal (last 11100 bytes)
            journal_start = len(data) - JOURNAL_SIZE
            journal_raw = data[journal_start:]
            result["journal_main"] = [
                parse_journal_entry(journal_raw[i * 37:(i + 1) * 37])
                for i in range(100)
            ]
            result["journal_side"] = [
                parse_journal_entry(journal_raw[3700 + i * 37:3700 + (i + 1) * 37])
                for i in range(100)
            ]
            result["journal_trade"] = [
                parse_journal_entry(journal_raw[7400 + i * 37:7400 + (i + 1) * 37])
                for i in range(100)
            ]

            # Character data (best-effort)
            cd_start = find_character_data_start(data, events_start_offset)
            if cd_start is not None:
                result["cd_start"] = cd_start
                # 4 bytes padding, then 40 bytes character details
                result["character_details"] = list(data[cd_start + 4:cd_start + 44])
                # 24 bytes save attributes
                result["player_attributes"] = parse_player_attributes_24(
                    data[cd_start + 44:cd_start + 68]
                )
                # 46 bytes extra character data
                result["extra_character_data"] = list(data[cd_start + 68:cd_start + 114])

                # Inventory area
                inv_start = cd_start + 114
                inv_end = find_inventory_end(data, inv_start)
                if inv_end is not None and inv_end > inv_start:
                    inv_data = data[inv_start:inv_end]
                    result["inventory_items"] = parse_inventory(inv_data)

    # 7. Player identity (best-effort from remaining data)
    identity = extract_player_identity(remaining)
    result.update(identity)

    return result


# ── Pretty printing ────────────────────────────────────────────────────────────

def format_flags(flags: int) -> list:
    parts = []
    if flags == 0:
        return ["inactive"]
    if flags & 1:
        parts.append("active")
    if flags & 2:
        parts.append("completed")
    if flags & ~3:
        parts.append(f"extra:0x{flags & ~3:x}")
    return parts


def format_state(flags: int) -> str:
    label = {0: "inactive", 1: "active", 2: "completed"}
    return label.get(flags, f"unknown({flags})")


def print_result(result: dict, brief: bool = False, json_output: bool = False):
    if json_output:
        # Serialize with clean defaults
        print(json.dumps(result, indent=2, ensure_ascii=False, default=str))
        return

    # ── Player identity ──
    print(f"Player: {result.get('player_name', '?')}")
    print(f"Class:  {result.get('player_class_name', '?')} (id={result.get('player_class_id', '?')})")

    pa = result.get("player_attributes", {})
    if pa:
        print(f"Stats:  STR={pa.get('strength','?')} DEX={pa.get('dexterity','?')} "
              f"WIS={pa.get('wisdom','?')} CON={pa.get('constitution','?')} "
              f"LCK={pa.get('luck','?')}")
        print(f"HP:     {pa.get('hp_current','?')}/{pa.get('hp_maximum','?')}")
        print(f"XP:     {pa.get('xp_current','?')}  "
              f"LVL={pa.get('level','?')}  "
              f"Gold={pa.get('gold','?')}")

    # ── Surface section counts ──
    mon_c = result.get("surface_monster_count", 0)
    npc_c = result.get("npc_count", 0)
    obj_c = result.get("surface_object_count", 0)
    print(f"\nSurface: {mon_c} monsters, {npc_c} NPCs, {obj_c} objects")

    if not brief:
        for i, mon in enumerate(result.get("surface_monsters", [])):
            print(f"  Monster {i:3d}: {mon['name']:20s} HP={mon['hp_current']:>3}/{mon['hp_maximum']:<3} "
                  f"at ({mon['tile_x']:>2},{mon['tile_y']:>2}) flags={mon['state_flags']:#x}")
        for i, npc in enumerate(result.get("npcs", [])):
            print(f"  NPC {i:3d}:     {npc['name']:20s} \"{npc['role']}\"")
        for i, obj in enumerate(result.get("surface_objects", [])):
            state = f" state={obj['state']}" if obj['state'] else ""
            print(f"  Object {i:2d}:   {obj['name']:20s} prefix={obj['prefix']:#04x}{state}")

    # ── Events ──
    events = result.get("events", [])
    active_events = [e for e in events if e.get("state", 0) > 0]
    print(f"\nEvents: {len(events)} total, {len(active_events)} active/completed")
    if not brief and active_events:
        for e in active_events[:20]:     # cap at 20
            print(f"  [{format_state(e['state'])}] {e['script_name']}")
        if len(active_events) > 20:
            print(f"  ... and {len(active_events) - 20} more")

    # ── Journal ──
    for label, key in [("Main", "journal_main"), ("Side", "journal_side"), ("Trade", "journal_trade")]:
        entries = [e for e in result.get(key, []) if e.get("name")]
        if entries:
            print(f"\nJournal ({label}): {len(entries)} entries")
            if not brief:
                for e in entries[:10]:
                    print(f"  [{e['counter']}] {e['name']}  flags={e['flags']:#x}")
                if len(entries) > 10:
                    print(f"  ... and {len(entries) - 10} more")

    # ── Inventory ──
    items = result.get("inventory_items", [])
    if items:
        quest = [i for i in items if i.get("is_quest")]
        standard = [i for i in items if not i.get("is_quest")]
        print(f"\nInventory: {len(standard)} items + {len(quest)} quest items")
        if not brief:
            for i, item in enumerate(standard):
                loc = " ".join(f"{b:02x}" for b in item.get("location_raw", []))
                desc = f" | {item['description']}" if item.get("description") else ""
                price = f" price={item['price']}" if item.get("price") else ""
                print(f"  [{i:2d}] loc=[{loc}]{price} \"{item['name']}\"{desc}")
            for i, item in enumerate(quest):
                print(f"  [Q{i:2d}] QUEST \"{item['name']}\"")
    else:
        print("\nInventory: (none extracted)")

    # ── Raw sizes ──
    print(f"\nRaw: header=12 surface_monsters={result.get('surface_monster_count',0)*329}B "
          f"npcs={result.get('npc_count',0)*349}B "
          f"objects={result.get('surface_object_count',0)*200}B "
          f"remaining={result.get('remaining_size',0)}B")


# ── CLI ────────────────────────────────────────────────────────────────────────

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Extract data from Dispel RPG .sav files")
    parser.add_argument("path", help="Path to .sav file")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    parser.add_argument("--brief", action="store_true", help="Compact listing (no per-item detail)")
    args = parser.parse_args()

    with open(args.path, "rb") as f:
        data = f.read()

    result = parse_save_file(data)
    print_result(result, brief=args.brief, json_output=args.json)


if __name__ == "__main__":
    main()
