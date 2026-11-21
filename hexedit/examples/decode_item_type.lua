-- Decodes a Dispel ItemTypeId byte into its human-readable name.
-- Useful when inspecting item records in WeaponItem.db, HealItem.db, etc.
--
-- The enum is signed: 1-5 are the defined types and -1 is the catch-all,
-- stored on disk as 0xFF. SQL and GUI numeric keys keep the byte value 255.

local ITEM_TYPES = {
    [1] = "Weapon",
    [2] = "Healing",
    [3] = "Edit",
    [4] = "Event",
    [5] = "Misc",
    [0xFF] = "Other",
}

local function name_for(v)
    return ITEM_TYPES[v] or ("Unknown (0x%02X)"):format(v)
end

return {
    name = "ItemTypeId",
    min_size = 1,
    category = "Dispel",
    description = "Dispel item type enum (1=Weapon, 2=Healing, 3=Edit, 4=Event, 5=Misc, 0xFF/-1=Other; SQL/GUI keys use 255)",

    decode = function(bytes)
        local v = string.byte(bytes, 1)
        return string.format("%s  (%d)", name_for(v), v)
    end,

    encode = function(str)
        -- Accept name or numeric value (-1 and 255 both encode the 0xFF byte)
        local n = tonumber(str)
        if n then
            n = math.floor(n)
            if n == -1 then
                n = 0xFF
            end
            if n < 0 or n > 255 then
                error("ItemTypeId out of range (-1 or 0-255)")
            end
            return string.char(n)
        end
        for k, v in pairs(ITEM_TYPES) do
            if v:lower() == str:lower() then
                return string.char(k)
            end
        end
        error("unknown item type '" .. str ..
              "'; expected: Weapon, Healing, Edit, Event, Misc, Other, or a number")
    end,
}
