-- Dispel game file decoder example for the hexedit Lua inspector.
--
-- Place this file in <game_path>/hexedit_scripts/ (or pass --script-dir
-- pointing to it in the standalone hexedit). Each script must return
-- a table with at minimum:
--   name     – display name in the inspector panel
--   min_size – minimum number of bytes required
--   decode   – function(bytes) → display string
--
-- Optional fields:
--   category    – group heading ("Custom" by default)
--   description – tooltip / help text
--   encode      – function(user_string) → raw bytes (enables editing)

local function to_hex(byte)
    return string.format("%02X", byte)
end

return {
    name = "dispel_tile_coord",
    min_size = 4,
    category = "Dispel",
    description = "Extract tile coordinates (x, y) from a 4-byte map struct: u16 x, u16 y",

    decode = function(bytes)
        -- bytes is a Lua string of raw bytes (min_size guarantees ≥4)
        local x = string.byte(bytes, 1) +
                  string.byte(bytes, 2) * 256
        local y = string.byte(bytes, 3) +
                  string.byte(bytes, 4) * 256
        return string.format("(%d, %d)  0x%04X, 0x%04X", x, y, x, y)
    end,

    encode = function(str)
        -- Accept "(x, y)" or "x y" or "x,y"
        local x_str, y_str = str:match("%s*(%d+)%s*[,%s]+%s*(%d+)%s*")
        if not x_str then
            -- try parenthesized form
            x_str, y_str = str:match("%((%d+)%s*,%s*(%d+)%)")
        end
        if not x_str then
            error("expected format: (x, y) or x y")
        end
        local x = tonumber(x_str)
        local y = tonumber(y_str)
        -- little-endian u16
        return string.char(x % 256, math.floor(x / 256),
                           y % 256, math.floor(y / 256))
    end,
}
