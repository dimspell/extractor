//! Tests for the auto-generated `MiscItemPatcher` (defined alongside
//! `MiscItem` via `#[derive(RecordPatcher)]`). The struct itself is
//! re-exported from [`crate::modding::patchers`] for downstream callers.

#[cfg(test)]
mod tests {
    use crate::modding::patcher::RecordPatcher;
    use crate::modding::patchers::MiscItemPatcher;
    use crate::modding::value::Value;
    use crate::references::extractor::Extractor;
    use crate::references::misc_item_db::MiscItem;
    use std::io::Cursor;

    fn one_item_blob(name: &str, base_price: i32) -> Vec<u8> {
        let mut data = 1i32.to_le_bytes().to_vec();
        let mut name_buf = [0u8; 30];
        let n = name.len().min(29);
        name_buf[..n].copy_from_slice(&name.as_bytes()[..n]);
        data.extend_from_slice(&name_buf);
        data.extend(vec![0u8; 202]);
        data.extend_from_slice(&base_price.to_le_bytes());
        data.extend(vec![0u8; 20]);
        data
    }

    fn parse_back(bytes: &[u8]) -> Vec<MiscItem> {
        let mut c = Cursor::new(bytes);
        MiscItem::parse(&mut c, bytes.len() as u64).unwrap()
    }

    #[test]
    fn rename_field() {
        let p = MiscItemPatcher;
        let original = one_item_blob("Helt", 15);
        let patched = p
            .apply_field(&original, 0, "name", &Value::String("Helmet".into()))
            .unwrap();
        let recs = parse_back(&patched);
        assert_eq!(recs[0].name, "Helmet");
        assert_eq!(recs[0].base_price, 15);
    }

    #[test]
    fn change_price() {
        let p = MiscItemPatcher;
        let original = one_item_blob("Torch", 15);
        let patched = p
            .apply_field(&original, 0, "base_price", &Value::I64(99))
            .unwrap();
        let recs = parse_back(&patched);
        assert_eq!(recs[0].base_price, 99);
        assert_eq!(recs[0].name, "Torch");
    }

    #[test]
    fn unknown_field_errors() {
        let p = MiscItemPatcher;
        let bytes = one_item_blob("X", 1);
        let err = p
            .apply_field(&bytes, 0, "nope", &Value::String("".into()))
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn change_price_via_string() {
        let p = MiscItemPatcher;
        let original = one_item_blob("Torch", 15);
        let patched = p
            .apply_field(&original, 0, "base_price", &Value::String("99".into()))
            .unwrap();
        assert_eq!(parse_back(&patched)[0].base_price, 99);
    }

    #[test]
    fn unparseable_string_for_numeric_errors() {
        let p = MiscItemPatcher;
        let bytes = one_item_blob("X", 1);
        let err = p
            .apply_field(&bytes, 0, "base_price", &Value::String("oops".into()))
            .unwrap_err();
        assert!(err.to_string().contains("expected i32"));
    }

    #[test]
    fn out_of_range_errors() {
        let p = MiscItemPatcher;
        let bytes = one_item_blob("X", 1);
        let err = p
            .apply_field(&bytes, 5, "name", &Value::String("Y".into()))
            .unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn id_field_rejected() {
        let p = MiscItemPatcher;
        let bytes = one_item_blob("X", 1);
        let err = p.apply_field(&bytes, 0, "id", &Value::I64(99)).unwrap_err();
        assert!(err.to_string().contains("positional"));
    }
}
