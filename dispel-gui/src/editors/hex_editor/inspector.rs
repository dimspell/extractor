//! Data inspector decoders and encoders.
//!
//! Pure functions consumed by the inspector view. Each entry knows how many
//! bytes it needs, how to render the slice as a string, and (optionally)
//! how to encode a user-typed string back into bytes.
//!
//! The codebase is little-endian-only (every parser uses `from_le_bytes`),
//! so we don't expose an endianness toggle.

/// Encode a user-typed value back into bytes, or report a human-readable error.
pub type EncodeFn = fn(&str) -> Result<Vec<u8>, String>;

/// One inspector row.
pub struct InspectorEntry {
    pub name: &'static str,
    pub min_size: usize,
    pub decode: fn(&[u8]) -> String,
    /// `Some` if this row is editable through the inspector modal.
    pub encode: Option<EncodeFn>,
}

/// All built-in inspector rows, in display order.
pub const ENTRIES: &[InspectorEntry] = &[
    InspectorEntry {
        name: "u8",
        min_size: 1,
        decode: dec_u8,
        encode: Some(enc_u8),
    },
    InspectorEntry {
        name: "i8",
        min_size: 1,
        decode: dec_i8,
        encode: Some(enc_i8),
    },
    InspectorEntry {
        name: "u16",
        min_size: 2,
        decode: dec_u16,
        encode: Some(enc_u16),
    },
    InspectorEntry {
        name: "i16",
        min_size: 2,
        decode: dec_i16,
        encode: Some(enc_i16),
    },
    InspectorEntry {
        name: "u32",
        min_size: 4,
        decode: dec_u32,
        encode: Some(enc_u32),
    },
    InspectorEntry {
        name: "i32",
        min_size: 4,
        decode: dec_i32,
        encode: Some(enc_i32),
    },
    InspectorEntry {
        name: "u64",
        min_size: 8,
        decode: dec_u64,
        encode: Some(enc_u64),
    },
    InspectorEntry {
        name: "i64",
        min_size: 8,
        decode: dec_i64,
        encode: Some(enc_i64),
    },
    InspectorEntry {
        name: "f32",
        min_size: 4,
        decode: dec_f32,
        encode: Some(enc_f32),
    },
    InspectorEntry {
        name: "f64",
        min_size: 8,
        decode: dec_f64,
        encode: Some(enc_f64),
    },
    InspectorEntry {
        name: "ascii",
        min_size: 1,
        decode: dec_ascii,
        encode: None,
    },
    InspectorEntry {
        name: "utf8",
        min_size: 1,
        decode: dec_utf8,
        encode: None,
    },
    InspectorEntry {
        name: "rgb565",
        min_size: 2,
        decode: dec_rgb565,
        encode: None,
    },
    InspectorEntry {
        name: "cstr",
        min_size: 1,
        decode: dec_cstr,
        encode: None,
    },
    InspectorEntry {
        name: "hex",
        min_size: 1,
        decode: dec_hex,
        encode: None,
    },
];

const MAX_CSTR_LEN: usize = 64;
const MAX_HEX_LEN: usize = 16;

fn dec_u8(b: &[u8]) -> String {
    format!("{} (0x{:02X})", b[0], b[0])
}

fn dec_i8(b: &[u8]) -> String {
    format!("{}", b[0] as i8)
}

fn dec_u16(b: &[u8]) -> String {
    let v = u16::from_le_bytes([b[0], b[1]]);
    format!("{} (0x{:04X})", v, v)
}

fn dec_i16(b: &[u8]) -> String {
    format!("{}", i16::from_le_bytes([b[0], b[1]]))
}

fn dec_u32(b: &[u8]) -> String {
    let v = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
    format!("{} (0x{:08X})", v, v)
}

fn dec_i32(b: &[u8]) -> String {
    format!("{}", i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn dec_u64(b: &[u8]) -> String {
    let v = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
    format!("{} (0x{:016X})", v, v)
}

fn dec_i64(b: &[u8]) -> String {
    format!(
        "{}",
        i64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
    )
}

fn dec_f32(b: &[u8]) -> String {
    format!("{}", f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn dec_f64(b: &[u8]) -> String {
    format!(
        "{}",
        f64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
    )
}

fn dec_ascii(b: &[u8]) -> String {
    let c = b[0];
    if (0x20..0x7F).contains(&c) {
        format!("'{}'", c as char)
    } else {
        format!("\\x{:02X}", c)
    }
}

fn dec_utf8(b: &[u8]) -> String {
    // Try the longest valid prefix that decodes to one char (1..=4 bytes).
    for n in 1..=b.len().min(4) {
        if let Ok(s) = std::str::from_utf8(&b[..n]) {
            if let Some(c) = s.chars().next() {
                return format!("'{}'  U+{:04X}", c, c as u32);
            }
        }
    }
    format!("\\x{:02X}", b[0])
}

/// Decode 16-bit RGB565 the same way the sprite renderer does
/// ([`crate::dispel_core::sprite::rgb16_565_produce_color`]):
/// 5 R | 6 G | 5 B, expanded to 8 bits per channel.
fn dec_rgb565(b: &[u8]) -> String {
    let pixel = u16::from_le_bytes([b[0], b[1]]);
    let r = ((pixel & 0xF800) >> 11) << 3;
    let g = ((pixel & 0x07E0) >> 5) << 2;
    let b_chan = (pixel & 0x001F) << 3;
    format!(
        "#{:02X}{:02X}{:02X}  ({}, {}, {})",
        r, g, b_chan, r, g, b_chan
    )
}

fn dec_cstr(b: &[u8]) -> String {
    let take = b
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(b.len())
        .min(MAX_CSTR_LEN);
    let lossy = String::from_utf8_lossy(&b[..take]);
    if take == 0 {
        "\"\"".to_string()
    } else {
        format!("\"{}\"", lossy.escape_debug())
    }
}

fn dec_hex(b: &[u8]) -> String {
    let n = b.len().min(MAX_HEX_LEN);
    let mut s = String::with_capacity(n * 3);
    for (i, c) in b[..n].iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        s.push_str(&format!("{:02X}", c));
    }
    if b.len() > n {
        s.push_str(" …");
    }
    s
}

// ── Encoders ──────────────────────────────────────────────────────────────
//
// All accept the user-typed string from the inspector modal. Numbers accept
// either decimal (`42`, `-1`) or `0x`-prefixed hex (`0xFF`). Whitespace
// around the value is trimmed. Any error is surfaced as a String suitable
// for display next to the input.

fn parse_int_unsigned<T>(s: &str, max_str: &str) -> Result<T, String>
where
    T: num_parse::ParseUnsigned,
{
    let s = s.trim();
    if s.is_empty() {
        return Err("value is empty".to_string());
    }
    let (radix, body) = if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        (16u32, rest)
    } else {
        (10u32, s)
    };
    T::parse(body, radix).map_err(|_| format!("not a valid {max_str} (decimal or 0xHEX)"))
}

fn parse_int_signed<T>(s: &str, max_str: &str) -> Result<T, String>
where
    T: num_parse::ParseSigned,
{
    let s = s.trim();
    if s.is_empty() {
        return Err("value is empty".to_string());
    }
    // Signed values: only accept decimal (with optional minus). Hex would be
    // ambiguous for negative numbers; users can fall back to the unsigned row.
    T::parse(s).map_err(|_| format!("not a valid {max_str} (decimal)"))
}

fn enc_u8(s: &str) -> Result<Vec<u8>, String> {
    let v: u8 = parse_int_unsigned(s, "u8")?;
    Ok(vec![v])
}

fn enc_i8(s: &str) -> Result<Vec<u8>, String> {
    let v: i8 = parse_int_signed(s, "i8")?;
    Ok(vec![v as u8])
}

fn enc_u16(s: &str) -> Result<Vec<u8>, String> {
    let v: u16 = parse_int_unsigned(s, "u16")?;
    Ok(v.to_le_bytes().to_vec())
}

fn enc_i16(s: &str) -> Result<Vec<u8>, String> {
    let v: i16 = parse_int_signed(s, "i16")?;
    Ok(v.to_le_bytes().to_vec())
}

fn enc_u32(s: &str) -> Result<Vec<u8>, String> {
    let v: u32 = parse_int_unsigned(s, "u32")?;
    Ok(v.to_le_bytes().to_vec())
}

fn enc_i32(s: &str) -> Result<Vec<u8>, String> {
    let v: i32 = parse_int_signed(s, "i32")?;
    Ok(v.to_le_bytes().to_vec())
}

fn enc_u64(s: &str) -> Result<Vec<u8>, String> {
    let v: u64 = parse_int_unsigned(s, "u64")?;
    Ok(v.to_le_bytes().to_vec())
}

fn enc_i64(s: &str) -> Result<Vec<u8>, String> {
    let v: i64 = parse_int_signed(s, "i64")?;
    Ok(v.to_le_bytes().to_vec())
}

fn enc_f32(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    let v: f32 = s.parse().map_err(|_| "not a valid f32".to_string())?;
    Ok(v.to_le_bytes().to_vec())
}

fn enc_f64(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    let v: f64 = s.parse().map_err(|_| "not a valid f64".to_string())?;
    Ok(v.to_le_bytes().to_vec())
}

/// Tiny shim so the integer parsers can share a body without bringing in a
/// crate. Each numeric type implements one of the two traits below.
mod num_parse {
    pub trait ParseUnsigned: Sized {
        fn parse(s: &str, radix: u32) -> Result<Self, ()>;
    }
    pub trait ParseSigned: Sized {
        fn parse(s: &str) -> Result<Self, ()>;
    }
    macro_rules! impl_unsigned {
        ($($t:ty),*) => {$(
            impl ParseUnsigned for $t {
                fn parse(s: &str, radix: u32) -> Result<Self, ()> {
                    Self::from_str_radix(s, radix).map_err(|_| ())
                }
            }
        )*};
    }
    macro_rules! impl_signed {
        ($($t:ty),*) => {$(
            impl ParseSigned for $t {
                fn parse(s: &str) -> Result<Self, ()> {
                    s.parse::<Self>().map_err(|_| ())
                }
            }
        )*};
    }
    impl_unsigned!(u8, u16, u32, u64);
    impl_signed!(i8, i16, i32, i64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8_displays_decimal_and_hex() {
        assert_eq!(dec_u8(&[0xAB]), "171 (0xAB)");
    }

    #[test]
    fn i8_displays_signed() {
        assert_eq!(dec_i8(&[0xFF]), "-1");
        assert_eq!(dec_i8(&[0x7F]), "127");
    }

    #[test]
    fn u32_le_decodes_correctly() {
        // 0x12345678 in LE: 78 56 34 12
        assert_eq!(dec_u32(&[0x78, 0x56, 0x34, 0x12]), "305419896 (0x12345678)");
    }

    #[test]
    fn i32_le_decodes_negative() {
        assert_eq!(dec_i32(&[0xFF, 0xFF, 0xFF, 0xFF]), "-1");
    }

    #[test]
    fn u64_le_decodes_correctly() {
        let v: u64 = 0x0123_4567_89AB_CDEF;
        let b = v.to_le_bytes();
        assert!(dec_u64(&b).starts_with("81985529216486895"));
    }

    #[test]
    fn f32_le_decodes_one() {
        let b = 1.0f32.to_le_bytes();
        assert_eq!(dec_f32(&b), "1");
    }

    #[test]
    fn ascii_printable_and_control() {
        assert_eq!(dec_ascii(b"A"), "'A'");
        assert_eq!(dec_ascii(&[0x00]), "\\x00");
        assert_eq!(dec_ascii(&[0x7F]), "\\x7F");
    }

    #[test]
    fn rgb565_white_and_black() {
        // White: all bits set in each channel: R=0x1F<<3=0xF8, G=0x3F<<2=0xFC, B=0x1F<<3=0xF8
        // Pixel value = 0xFFFF.
        let s = dec_rgb565(&[0xFF, 0xFF]);
        assert!(s.contains("#F8FCF8"));
        // Black: 0x0000 → #000000
        let s = dec_rgb565(&[0x00, 0x00]);
        assert!(s.contains("#000000"));
    }

    #[test]
    fn cstr_terminates_at_nul() {
        assert_eq!(dec_cstr(b"hello\0world"), "\"hello\"");
    }

    #[test]
    fn cstr_handles_no_terminator_within_max() {
        // No NUL → take all (up to MAX_CSTR_LEN).
        let s = dec_cstr(b"abcdef");
        assert_eq!(s, "\"abcdef\"");
    }

    #[test]
    fn hex_lists_bytes_with_spaces_and_truncates() {
        let s = dec_hex(&[0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(s, "DE AD BE EF");
        let twenty = vec![0x55u8; 20];
        let s = dec_hex(&twenty);
        assert!(s.ends_with(" …"));
    }

    #[test]
    fn entries_have_increasing_or_equal_min_sizes_for_widening_ints() {
        // Sanity: u8 first, then u16/i16, etc.
        let names: Vec<&str> = ENTRIES.iter().map(|e| e.name).collect();
        assert_eq!(names[0], "u8");
        assert_eq!(names[2], "u16");
        assert_eq!(names[4], "u32");
    }

    #[test]
    fn enc_u8_accepts_decimal_and_hex() {
        assert_eq!(enc_u8("42").unwrap(), vec![42]);
        assert_eq!(enc_u8("0xFF").unwrap(), vec![0xFF]);
        assert_eq!(enc_u8(" 7 ").unwrap(), vec![7]);
    }

    #[test]
    fn enc_u8_rejects_overflow_and_garbage() {
        assert!(enc_u8("256").is_err());
        assert!(enc_u8("nope").is_err());
        assert!(enc_u8("").is_err());
    }

    #[test]
    fn enc_i32_signed_decimal_only() {
        assert_eq!(enc_i32("-1").unwrap(), vec![0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(enc_i32("0").unwrap(), vec![0, 0, 0, 0]);
        assert!(enc_i32("0xFF").is_err());
    }

    #[test]
    fn enc_u32_le_round_trip() {
        let bytes = enc_u32("305419896").unwrap();
        assert_eq!(bytes, vec![0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn enc_f64_round_trip() {
        let bytes = enc_f64("1.5").unwrap();
        let v = f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        assert_eq!(v, 1.5);
        assert!(enc_f64("not a number").is_err());
    }

    #[test]
    fn editable_entries_match_numeric_ones() {
        let editable: Vec<&str> = ENTRIES
            .iter()
            .filter(|e| e.encode.is_some())
            .map(|e| e.name)
            .collect();
        assert_eq!(
            editable,
            vec!["u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "f32", "f64"]
        );
    }
}
