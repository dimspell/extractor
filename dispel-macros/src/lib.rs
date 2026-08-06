use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod binary_record_impl;
mod extractor_impl;
mod localizable_impl;
mod record_patcher_impl;
mod text_extractor_impl;
mod text_record_patcher_impl;

/// Derive macro that generates a `Localizable` impl for a struct.
///
/// Fields annotated with `#[translatable(encoding = "...", max_bytes = N)]` are included
/// in extraction and application. Other fields are ignored.
///
/// Supported `encoding` values: `"WINDOWS-1250"`, `"EUC-KR"`, `"UTF-8"`.
/// Both `String` and `Option<String>` field types are supported.
#[proc_macro_derive(Localizable, attributes(translatable))]
pub fn derive_localizable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    localizable_impl::expand(input).into()
}

/// Derive macro that generates inherent `parse`, `write`, and `record_size`
/// methods for a fixed-size binary record struct.
///
/// Fields are auto-detected for primitive types (`u8`, `u16`, `u32`, `i16`, `i32`).
/// `String` fields require `#[binary_record(string(encoding = "...", size = N))]`.
/// `Vec<u8>` fields require `#[binary_record(size = N)]`.
///
/// Additional annotations:
/// - `#[binary_record(padding(count = N, type = "u8|i16|i32"))]` — padding bytes
/// - `#[binary_record(skip)]` — skip field (uses Default)
///
/// Supported encoding values: "WINDOWS-1250", "EUC-KR", "UTF-8"
#[proc_macro_derive(BinaryRecord, attributes(binary_record))]
pub fn derive_binary_record(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    binary_record_impl::expand(input).into()
}

/// Derive macro that generates an `Extractor` impl for a struct.
///
/// Fields can be annotated with `#[extractor(...)]` to specify how they should
/// be parsed and written:
///
/// - `#[extractor(id)]` - Record ID field (auto-incremented during parsing)
/// - `#[extractor(string(encoding = "...", size = N))]` - Fixed-size string field
/// - `#[extractor(primitive(type = "i16|i32|u8|u16|u32"))]` - Primitive numeric field
/// - `#[extractor(enum_from_u8(type = "EnumType"))]` - u8-based enum field
/// - `#[extractor(enum_from_i16(type = "EnumType"))]` - i16-based enum field
/// - `#[extractor(enum_from_i32(type = "EnumType"))]` - i32-based enum field
/// - `#[extractor(padding(count = N, type = "i16|i32|u8"))]` - Padding field(s)
/// - `#[extractor(array(size = N, type = "u8"))]` - Fixed-size array
/// - `#[extractor(skip)]` - Skip field during parsing/writing
/// - `#[extractor(counter_size = N)]` - Set COUNTER_SIZE (default 4)
/// - `#[extractor(property_item_size = N)]` - Set PROPERTY_ITEM_SIZE
///
/// Supported encoding values: "WINDOWS-1250", "EUC-KR", "UTF-8"
#[proc_macro_derive(Extractor, attributes(extractor))]
pub fn derive_extractor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    extractor_impl::expand(input).into()
}

/// Derive macro that generates an `Extractor` impl for text/CSV files.
///
/// Struct-level attributes:
/// - `#[extractor(encoding = "EUC_KR|WINDOWS_1250|UTF_8")]`
/// - `#[extractor(delimiter = ",")]`
/// - `#[extractor(comment_char = ";")]`
///
/// Field-level attributes:
/// - `#[extractor(field = N)]` - CSV field index (0-based)
/// - `#[extractor(parse_null)]` - Field is `Option<String>`, uses parse_null()
/// - `#[extractor(enum_from_i32(type = "EnumType"))]` - Parse as i32-based enum
#[proc_macro_derive(TextExtractor, attributes(extractor))]
pub fn derive_text_extractor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    text_extractor_impl::expand(input).into()
}

/// Derive macro that generates a `RecordPatcher`
/// implementation by *reusing* the `#[extractor(...)]` attributes already on
/// the struct. Emits a unit struct named `<Struct>Patcher;` alongside the
/// trait impl.
///
/// # Struct attribute
///
/// `#[patcher(filename = "MiscItem.db")]` (required) — used as the patcher's
/// `name()` (after stripping the extension) and exposed as
/// `<Struct>Patcher::FILENAME`.
#[proc_macro_derive(RecordPatcher, attributes(extractor, patcher))]
pub fn derive_record_patcher(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    record_patcher_impl::expand(input).into()
}

/// Derive macro that generates a `RecordPatcher`
/// implementation for `TextExtractor`-based catalogs (CSV / pipe-delimited
/// `.ini` / `.scr` files).
#[proc_macro_derive(TextRecordPatcher, attributes(extractor, patcher))]
pub fn derive_text_record_patcher(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    text_record_patcher_impl::expand(input).into()
}
