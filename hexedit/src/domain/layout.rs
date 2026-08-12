//! Query-oriented structure overlays for binary files.

use std::ops::Range;

/// One resolved on-disk field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpan {
    pub range: Range<u64>,
    pub name: &'static str,
    pub ty: &'static str,
    pub record_type: &'static str,
    pub record_index: u64,
    pub color_index: u8,
}

/// A binary layout answers only the addresses Hexedit needs to draw.
pub trait BinaryLayout: Send + Sync {
    fn field_at(&self, address: u64, file_len: u64) -> Option<FieldSpan>;
    fn fields_in(&self, range: Range<u64>, file_len: u64) -> Vec<FieldSpan>;

    fn is_header_at(&self, _address: u64, _file_len: u64) -> bool {
        false
    }

    fn is_truncated_at(&self, _address: u64, _file_len: u64) -> bool {
        false
    }
}

/// A field definition supplied by an embedding application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedRecordField {
    pub name: &'static str,
    pub offset: u32,
    pub size: u32,
    pub ty: &'static str,
}

/// Generic arithmetic for a fixed-record binary format.
#[derive(Debug, Clone)]
pub struct FixedRecordBinaryLayout {
    type_name: &'static str,
    header_size: u64,
    record_size: u64,
    fields: Vec<FixedRecordField>,
}

impl FixedRecordBinaryLayout {
    pub fn new(
        type_name: &'static str,
        header_size: u32,
        record_size: u32,
        fields: impl Into<Vec<FixedRecordField>>,
    ) -> Self {
        assert!(
            record_size > 0,
            "fixed record layout must have a record size"
        );
        Self {
            type_name,
            header_size: u64::from(header_size),
            record_size: u64::from(record_size),
            fields: fields.into(),
        }
    }

    fn record_index_at(&self, address: u64, file_len: u64) -> Option<u64> {
        if address < self.header_size || address >= file_len {
            return None;
        }
        let payload = file_len - self.header_size;
        let complete_end = self.header_size + (payload / self.record_size) * self.record_size;
        (address < complete_end).then_some((address - self.header_size) / self.record_size)
    }

    fn field_for_record(&self, record_index: u64, field: FixedRecordField) -> FieldSpan {
        let start = self.header_size + record_index * self.record_size + u64::from(field.offset);
        FieldSpan {
            range: start..start + u64::from(field.size),
            name: field.name,
            ty: field.ty,
            record_type: self.type_name,
            record_index,
            color_index: (self
                .fields
                .iter()
                .position(|candidate| *candidate == field)
                .unwrap_or(0)
                % 16) as u8,
        }
    }
}

impl BinaryLayout for FixedRecordBinaryLayout {
    fn field_at(&self, address: u64, file_len: u64) -> Option<FieldSpan> {
        let record_index = self.record_index_at(address, file_len)?;
        self.fields
            .iter()
            .copied()
            .map(|field| self.field_for_record(record_index, field))
            .find(|span| span.range.contains(&address))
    }

    fn fields_in(&self, range: Range<u64>, file_len: u64) -> Vec<FieldSpan> {
        if range.start >= range.end || range.start >= file_len {
            return Vec::new();
        }
        let range = range.start..range.end.min(file_len);
        let first = self.record_index_at(range.start.max(self.header_size), file_len);
        let last = self.record_index_at(range.end.saturating_sub(1), file_len);
        match (first, last) {
            (Some(first), Some(last)) => (first..=last)
                .flat_map(|record| {
                    self.fields
                        .iter()
                        .copied()
                        .map(move |field| self.field_for_record(record, field))
                })
                .filter(|span| span.range.start < range.end && range.start < span.range.end)
                .collect(),
            _ => Vec::new(),
        }
    }

    fn is_header_at(&self, address: u64, file_len: u64) -> bool {
        address < self.header_size.min(file_len)
    }

    fn is_truncated_at(&self, address: u64, file_len: u64) -> bool {
        if address < self.header_size || address >= file_len {
            return false;
        }
        let payload = file_len - self.header_size;
        let complete_end = self.header_size + (payload / self.record_size) * self.record_size;
        address >= complete_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> FixedRecordBinaryLayout {
        FixedRecordBinaryLayout::new(
            "Record",
            4,
            8,
            vec![
                FixedRecordField {
                    name: "tag",
                    offset: 0,
                    size: 2,
                    ty: "u16",
                },
                FixedRecordField {
                    name: "value",
                    offset: 2,
                    size: 4,
                    ty: "i32",
                },
            ],
        )
    }

    #[test]
    fn test_fixed_layout_resolves_headered_record_fields() {
        let layout = layout();
        let field = layout.field_at(14, 20).unwrap();
        assert_eq!(field.name, "value");
        assert_eq!(field.record_index, 1);
        assert_eq!(field.range, 14..18);
    }

    #[test]
    fn test_fixed_layout_clips_visible_range_and_hides_partial_record() {
        let layout = layout();
        assert_eq!(layout.fields_in(10..16, 21).len(), 2);
        assert!(layout.field_at(20, 21).is_none());
        assert!(layout.is_truncated_at(20, 21));
    }
}
