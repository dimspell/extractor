# Store.db Documentation

## File Format: Shop & Inn Database

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, shop data, or proprietary assets. All references to commerce systems are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Binary database file that defines shops and inns with inventories, prices, merchant dialogue, and economic behavior for the game's commerce system.

### File Structure

**Location**: `CharacterInGame/STORE.DB`
**Encoding**: Binary (Little-Endian)
**Text Encoding**: WINDOWS-1250
**Header**: 4-byte record count
**Record Size**: 948 bytes (237 × 4-byte fields)
**Total Records**: Variable (determined by header)

### Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of shop/inn entries)

[Records: 948 bytes each]
- name: 32 bytes (WINDOWS-1250, null-padded)
- inn_night_cost: i32 (determines record type)
- IF inn_night_cost > 0 (Inn):
  - 144 bytes padding
- ELSE (Shop):
  - some_unknown_number: i16 (price modifier?)
  - products: 71 × (i16, i16) pairs (order, type, item_id)
- invitation: 512 bytes (WINDOWS-1250, null-padded)
- haggle_success: 128 bytes (WINDOWS-1250, null-padded)
- haggle_fail: 128 bytes (WINDOWS-1250, null-padded)
```

### Field Definitions

| Field | Size | Type | Description |
|-------|------|------|-------------|
| index | N/A | i32 | Record index (assigned during parsing) |
| store_name | 32 | string | Shop/Inn name (WINDOWS-1250 encoded) |
| inn_night_cost | 4 | i32 | Night cost (>0 = inn, 0 = shop) |
| some_unknown_number | 2 | i16 | Price modifier (shops only) |
| products | 142 | array | Product list (shops only) |
| invitation | 512 | string | Merchant greeting (WINDOWS-1250) |
| haggle_success | 128 | string | Successful haggle response |
| haggle_fail | 128 | string | Failed haggle response |

### Store Types

**Inn (inn_night_cost > 0):**
- No products (144 bytes padding)
- Provides rest/healing services
- Nightly accommodation cost
- Dialogue for innkeeper

**Shop (inn_night_cost = 0):**
- Product inventory (up to 71 items)
- Sells goods and equipment
- Price modifier affects economy
- Merchant dialogue and haggling

### Product Structure

**ProductType Enum:**
- **1**: Weapon - Bronze/Weapons
- **2**: Healing - Edibles/Consumables
- **3**: Miscellaneous - Equipment
- **4**: Edit - Magical Items

**Product Format:**
- `(order: i16, type: i16, item_id: i16)`
- Terminated by `type = 0`
- Max 71 products per shop
- Order determines display sequence

### Data Structure

The codebase defines the store structure as:

```rust
pub struct Store {
    index: i32,                    // Record index
    store_name: String,            // Shop name (32 chars max)
    inn_night_cost: i32,           // >0 = inn, 0 = shop
    some_unknown_number: i16,      // Price modifier (shops)
    products: Vec<StoreProduct>,   // Product inventory
    invitation: String,           // Greeting text (512 chars)
    haggle_success: String,       // Success text (128 chars)
    haggle_fail: String,           // Failure text (128 chars)
}

pub type StoreProduct = (i16, ProductType, i16); // order, type, item_id
```

### Binary Record Layout

```
Offset | Size | Field | Description
-------|------|-------|-------------
0      | 32   | name  | Null-padded WINDOWS-1250 string
32     | 4    | cost  | inn_night_cost (i32)
36     | VAR  | data  | Variable data (inn=padding, shop=products)
VAR    | 512  | invit | Invitation text (WINDOWS-1250)
VAR+512| 128  | succ  | Haggle success text
VAR+640| 128  | fail  | Haggle fail text
```

### Special Values

- **inn_night_cost > 0**: Inn record (no products)
- **inn_night_cost = 0**: Shop record (with products)
- **inn_night_cost = -1**: Special/quest shop
- **Product type = 0**: Terminates product list
- **Null-padded strings**: Fixed-size text fields

### Example Structures

**Inn Record:**
```
name: "Tavern Name" (32 bytes)
inn_night_cost: 50 (i32)
144 bytes padding
invitation: "Welcome to our tavern!" (512 bytes)
haggle_success: "Pleasure doing business!" (128 bytes)
haggle_fail: "That's my best price!" (128 bytes)
```

**Shop Record:**
```
name: "Weapon Shop" (32 bytes)
inn_night_cost: 0 (i32)
some_unknown_number: 10 (i16 - price modifier)
products: [(1, Weapon, 101), (2, Healing, 205), ...]
invitation: "Welcome to my shop!" (512 bytes)
haggle_success: "Great deal!" (128 bytes)
haggle_fail: "No discounts!" (128 bytes)
```

### Usage in Game

1. **Commerce System**: Shop inventory and pricing
2. **Inn System**: Rest and healing services
3. **Dialogue System**: Merchant interactions
4. **Economy System**: Price modifiers and haggling
5. **Quest System**: Special shop/inn locations

### File Characteristics

- **Record Size**: 948 bytes (fixed)
- **Header**: 4-byte record count
- **Encoding**: WINDOWS-1250 for all text
- **Structure**: Union type (inn vs shop)
- **Complexity**: Mixed record formats

### Technical Analysis

**Efficiency:**
- Fixed record size enables random access
- Union structure optimizes space
- Large text fields for dialogue
- Complex parsing logic required

**Limitations:**
- Fixed product limit (71 items)
- Mixed record types complicate parsing
- Large padding for inns
- Complex binary format

**Performance:**
- Moderate parsing complexity
- Efficient database storage
- Separate product table
- Dialogue text optimization

### Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any shop data or game content
- Focuses on **technical organization and commerce systems**
- Explains **store mechanics and economic structures**
- Maintains **nominal fair use** for trademark references

### Notes

- File uses complex binary format with union types
- Mixed inn/shop records require conditional parsing
- Fixed record size enables efficient access
- Integrated with commerce and dialogue systems
- **No copyrighted game content** is reproduced or distributed

### Comparison with Other Databases

**Store.db vs HealItem.db:**
- **Store.db**: Commerce system with dialogue
- **HealItem.db**: Consumable items only
- **Store.db**: Complex inventory system
- **HealItem.db**: Simple healing effects

**Store.db vs MiscItem.db:**
- **Store.db**: Shops with inventories
- **MiscItem.db**: Individual utility items
- **Store.db**: Economic system
- **MiscItem.db**: Simple item list

### File Purpose Summary

Store.db serves as a comprehensive database for:
- Shop inventories and pricing
- Inn services and accommodation
- Merchant dialogue and haggling
- Economic system integration
- Quest-related commerce locations

The file provides a sophisticated system for managing all commerce-related entities in the game, supporting both simple inns and complex shops with extensive inventories and interactive dialogue.