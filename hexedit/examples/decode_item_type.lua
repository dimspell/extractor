-- Decodes a Dispel ItemTypeId u8 into its human-readable name.
-- Useful when inspecting item records in WeaponItem.db, HealItem.db, etc.

local ITEM_TYPES = {
    [0] = "Weapon",
    [1] = "Armor",
    [2] = "Heal",
    [3] = "Misc",
    [4] = "Edit",
    [5] = "Event",
}

return {
    name = "ItemTypeId",
    min_size = 1,
    category = "Dispel",
    description = "Dispel item type enum (0=Weapon, 1=Armor, 2=Heal, 3=Misc, 4=Edit, 5=Event)",

    decode = function(bytes)
        local v = string.byte(bytes, 1)
        local name = ITEM_TYPES[v] or ("Unknown (0x%02X)"):format(v)
        return string.format("%s  (%d)", name, v)
    end,

    encode = function(str)
        -- Accept name or numeric value
        local n
        -- try numeric first
        n = tonumber(str)
        if n then
            n = math.floor(n)
            if n < 0 or n > 255 then
                error("ItemTypeId out of range (0–255)")
            end
            return string.char(n)
        end
        -- try name lookup
        for k, v in pairs(ITEM_TYPES) do
            if v:lower() == str:lower() then
                return string.char(k)
            end
        end
        error("unknown item type '" .. str ..
              "'; expected: Weapon, Armor, Heal, Misc, Edit, Event, or a number 0–5")
    end,
}
