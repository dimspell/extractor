# Store.db Documentation

## File Information

### Overview

Binary database file that defines shops and inns with inventories, prices, merchant dialogue, and economic behavior for the game's commerce system.

### File Structure

**Location**: `CharacterInGame/STORE.DB`
**Encoding**: Binary (Little-Endian)
**Text Encoding**: WINDOWS-1250
**Header**: 4-byte record count
**Record Size**: 948 bytes
**Total Records**: Variable (determined by header)

### Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of shop/inn entries)

[Records: 948 bytes each]
- name: 32 bytes (WINDOWS-1250, null-padded)
- inn_night_cost: i32 (determines record type)
- IF inn_night_cost > 0 (Inn):
  - 144 bytes padding (price modifier + 15 product slots, all zero)
- ELSE (Shop):
  - price_modifier: i16 (percentage applied to item prices)
  - products: 15 × (i16, i16) slots (type, item_id), terminated by type = 0
  - 82 bytes padding
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
| price_modifier | 2 | i16 | Percentage applied to item prices (shops only) |
| products | 60 | array | Product list (shops only, up to 15 slots) |
| padding | 82 | bytes | Always zero (shops) |
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
- Product inventory (up to 15 items)
- Sells goods and equipment
- Price modifier affects economy
- Merchant dialogue and haggling

### Product Structure

**ProductType Enum:**
- **1**: Weapon - Weapons and armor
- **2**: Healing - Healing items (potions, etc.)
- **3**: EditItem - Editable/modifiable equipment
- **4**: MiscItem - Miscellaneous items

**Product Format:**
- `(type: i16, item_id: i16)` per slot (4 bytes each)
- Terminated by `type = 0`
- Max 15 products per shop (the game iterates `iVar4 < 0xf`)
- Slot index is the display order

### Price Modifier & Pricing Formulas

The `price_modifier` field is a signed percentage applied to an item's base price:

```
buy_price  = base_price + (price_modifier * base_price) / 100
sell_price = base_price/2 + (price_modifier * (base_price/2)) / 100
```

- **Buy price**: the player pays `base_price * (1 + price_modifier/100)`.
- **Sell price**: the shop pays the player `(base_price/2) * (1 + price_modifier/100)`
  (items sell at half their base price, then the modifier is applied).
- A positive modifier raises prices (e.g. `10` → +10%); a negative modifier lowers them.
- A value of `0` means the shop sells at the item's unmodified base price.

### Data Structure

The codebase defines the store structure as:

```rust
pub struct Store {
    index: i32,                    // Record index
    store_name: String,            // Shop name (32 chars max)
    inn_night_cost: i32,           // >0 = inn, 0 = shop
    price_modifier: i16,           // Price modifier (shops)
    products: Vec<StoreProduct>,   // Product inventory
    invitation: String,            // Greeting text (512 chars)
    haggle_success: String,        // Success text (128 chars)
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
36     | 2    | mod   | price_modifier (i16, shops only)
38     | 60   | prods | 15 × (type, item_id) slots (shops only)
98     | 82   | pad   | Always zero
180    | 512  | invit | Invitation text (WINDOWS-1250)
692    | 128  | succ  | Haggle success text
820    | 128  | fail  | Haggle fail text
```

### Special Values

- **inn_night_cost > 0**: Inn record (no products)
- **inn_night_cost = 0**: Shop record (with products)

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
price_modifier: 10 (i16)
products: [(Weapon, 101), (Healing, 205), ...]
82 bytes padding
invitation: "Welcome to my shop!" (512 bytes)
haggle_success: "Great deal!" (128 bytes)
haggle_fail: "No discounts!" (128 bytes)
```
