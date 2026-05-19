-- Minimal hexedit Lua decoder example.
-- Place in <game_path>/hexedit_scripts/ and it auto-loads.

return {
    name = "u32le",
    min_size = 4,
    category = "Numeric",
    description = "Little-endian unsigned 32-bit integer",

    decode = function(bytes)
        local v = string.byte(bytes, 1) +
                  string.byte(bytes, 2) * 256 +
                  string.byte(bytes, 3) * 65536 +
                  string.byte(bytes, 4) * 16777216
        return string.format("%u  (0x%08X)", v, v)
    end,

    encode = function(str)
        local n = tonumber(str)
        if not n or n < 0 or n > 0xFFFFFFFF then
            error("expected a u32 value (0–4294967295)")
        end
        n = math.floor(n)
        return string.char(n % 256,
                           math.floor(n / 256) % 256,
                           math.floor(n / 65536) % 256,
                           math.floor(n / 16777216))
    end,
}
