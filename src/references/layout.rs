//! Static metadata for fixed-size binary record layouts.

/// One on-disk field within a fixed-size record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldDef {
    pub name: &'static str,
    pub offset: u32,
    pub size: u32,
    pub ty: &'static str,
}

/// Complete byte layout for a file made from fixed-size records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedRecordLayout {
    pub type_name: &'static str,
    pub header_size: u32,
    pub record_size: u32,
    pub fields: &'static [FieldDef],
}

/// Static layout metadata generated from an `Extractor` record definition.
pub trait RecordLayout {
    const LAYOUT: FixedRecordLayout;
}
