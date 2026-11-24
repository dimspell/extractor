//! Single source of truth for the binary `#[extractor]` record schema and the
//! crate paths used by every derive macro in this crate.
//!
//! Before this module, `FieldInfo`, `parse_extractor_attr`, the encoding table
//! and the `crate::references::* / crate::modding::* / byteorder::*` paths were
//! duplicated across the individual codegen files. Three independent codegen
//! paths (parse/write, patch arms, byte-offset layout) consumed the same
//! hand-rolled attribute grammar, and each of them spelled out the emitted
//! paths itself. Everything below is shared and directly unit-tested so that
//! a rename in `dispel-core` only has to be applied once.

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, LitInt, LitStr, Type};

// ---------------------------------------------------------------------------
// Codegen configuration — every emitted path/type name flows through here.
// ---------------------------------------------------------------------------

/// Names of the crates and modules that generated code refers to.
///
/// `Default` reproduces the historical token streams exactly (so generated
/// code — and therefore on-disk bytes — is unchanged). Overriding a field
/// renames that path everywhere in one step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodegenConfig {
    pub crate_root: String,
    pub references_mod: String,
    pub enums_mod: String,
    pub modding_mod: String,
    pub localization_mod: String,
    pub inventory_item_ty: String,
    pub byteorder_crate: String,
    pub encoding_rs_crate: String,
    pub encoding_rs_io_crate: String,
}

impl Default for CodegenConfig {
    fn default() -> Self {
        Self {
            crate_root: "crate".into(),
            references_mod: "references".into(),
            enums_mod: "enums".into(),
            modding_mod: "modding".into(),
            localization_mod: "localization".into(),
            inventory_item_ty: "InventoryItem".into(),
            byteorder_crate: "byteorder".into(),
            encoding_rs_crate: "encoding_rs".into(),
            encoding_rs_io_crate: "encoding_rs_io".into(),
        }
    }
}

impl CodegenConfig {
    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    /// `crate_root::module` followed by `::segment` for each remaining part.
    fn crate_path(&self, module: &str, rest: &[&str]) -> TokenStream2 {
        let root = Self::ident(&self.crate_root);
        let module = Self::ident(module);
        let rest = rest.iter().map(|part| Self::ident(part));
        quote! { #root::#module #(:: #rest)* }
    }

    /// `crate::references::extractor::Extractor`
    pub(crate) fn extractor_trait(&self) -> TokenStream2 {
        self.crate_path(&self.references_mod, &["extractor", "Extractor"])
    }
    /// `crate::references::extractor::read_mapper`
    pub(crate) fn read_mapper(&self) -> TokenStream2 {
        self.crate_path(&self.references_mod, &["extractor", "read_mapper"])
    }
    /// `crate::references::extractor::parse_null`
    pub(crate) fn parse_null(&self) -> TokenStream2 {
        self.crate_path(&self.references_mod, &["extractor", "parse_null"])
    }
    /// `crate::references::layout::*`
    pub(crate) fn layout(&self, item: &str) -> TokenStream2 {
        self.crate_path(&self.references_mod, &["layout", item])
    }
    /// `crate::references::enums::InventoryItem`
    pub(crate) fn inventory_item_type(&self) -> TokenStream2 {
        self.crate_path(
            &self.references_mod,
            &[&self.enums_mod, &self.inventory_item_ty],
        )
    }
    /// `crate::modding::*`
    pub(crate) fn modding(&self, parts: &[&str]) -> TokenStream2 {
        self.crate_path(&self.modding_mod, parts)
    }
    /// `crate::localization::*`
    pub(crate) fn localization(&self, parts: &[&str]) -> TokenStream2 {
        self.crate_path(&self.localization_mod, parts)
    }
    /// `crate::references::enums::WireValue` — the fallback policy shared by
    /// every derive that reads an enum discriminant.
    pub(crate) fn wire_value_trait(&self) -> TokenStream2 {
        self.crate_path(&self.references_mod, &[&self.enums_mod, "WireValue"])
    }
    /// `byteorder` path prefix (used as `#bo::ReadBytesExt` etc.).
    pub(crate) fn byteorder(&self) -> Ident {
        Self::ident(&self.byteorder_crate)
    }
    /// `encoding_rs` path prefix.
    pub(crate) fn encoding_rs(&self) -> Ident {
        Self::ident(&self.encoding_rs_crate)
    }
    /// `encoding_rs_io` path prefix.
    pub(crate) fn encoding_rs_io(&self) -> Ident {
        Self::ident(&self.encoding_rs_io_crate)
    }

    /// `encoding_rs::WINDOWS_1250` / `EUC_KR` / `UTF_8` for a textual encoding
    /// name. Spellings are resolved by the shared `dispel-encodings` registry
    /// (via `crate::encoding`); only the emitted crate path comes from this
    /// config, so there is exactly one encoding table and one path config.
    pub(crate) fn encoding_constant(&self, encoding: &str) -> TokenStream2 {
        let enc = self.encoding_rs();
        crate::encoding::encoding_rs_tokens(encoding, &enc)
    }

    /// `crate::localization::TextEncoding::<variant>` for a textual encoding.
    /// The variant name also comes from the shared encoding registry.
    pub(crate) fn text_encoding(&self, encoding: &str) -> TokenStream2 {
        let path = self.localization(&["TextEncoding"]);
        crate::encoding::text_encoding_tokens(encoding, &path)
    }
}

// ---------------------------------------------------------------------------
// Shared `#[extractor]` field grammar (the AST all three codegen paths use).
// ---------------------------------------------------------------------------

pub(crate) enum FieldInfo<'a> {
    Id {
        ident: &'a Ident,
    },
    Index {
        ident: &'a Ident,
    },
    String {
        ident: &'a Ident,
        encoding: String,
        size: usize,
    },
    Primitive {
        ident: &'a Ident,
        ty: String,
    },
    InventoryItem {
        ident: &'a Ident,
        wire_type: String,
    },
    EnumFromU8 {
        ident: &'a Ident,
        enum_ty: String,
    },
    EnumFromU32 {
        ident: &'a Ident,
        enum_ty: String,
    },
    EnumFromI16 {
        ident: &'a Ident,
        enum_ty: String,
    },
    EnumFromI32 {
        ident: &'a Ident,
        enum_ty: String,
    },
    EnumFromI32FromU8 {
        ident: &'a Ident,
        enum_ty: String,
    },
    Padding {
        ident: &'a Ident,
        count: usize,
        ty: String,
        default_value: Option<String>,
    },
    Array {
        ident: &'a Ident,
        size: usize,
        ty: String,
    },
    VecU8 {
        ident: &'a Ident,
        size: usize,
    },
    Skip,
}

impl FieldInfo<'_> {
    /// The field identifier, if this variant corresponds to an actual struct
    /// field (`Skip` is synthetic).
    pub(crate) fn ident(&self) -> Option<&Ident> {
        match self {
            FieldInfo::Id { ident }
            | FieldInfo::Index { ident }
            | FieldInfo::String { ident, .. }
            | FieldInfo::Primitive { ident, .. }
            | FieldInfo::InventoryItem { ident, .. }
            | FieldInfo::EnumFromU8 { ident, .. }
            | FieldInfo::EnumFromU32 { ident, .. }
            | FieldInfo::EnumFromI16 { ident, .. }
            | FieldInfo::EnumFromI32 { ident, .. }
            | FieldInfo::EnumFromI32FromU8 { ident, .. }
            | FieldInfo::Padding { ident, .. }
            | FieldInfo::Array { ident, .. }
            | FieldInfo::VecU8 { ident, .. } => Some(ident),
            FieldInfo::Skip => None,
        }
    }

    /// Byte size and human-readable type for the generated byte-offset layout.
    /// Returns `None` for positional/synthetic fields, matching the historic
    /// layout codegen.
    pub(crate) fn layout_size_and_type(&self) -> Option<(u32, String)> {
        let scalar = |ty: &str| match ty {
            "u8" | "i8" => 1,
            "u16" | "i16" => 2,
            "u32" | "i32" => 4,
            _ => panic!("unsupported layout scalar type: {ty}"),
        };
        match self {
            FieldInfo::Id { .. } | FieldInfo::Index { .. } | FieldInfo::Skip => None,
            FieldInfo::String { encoding, size, .. } => {
                Some((*size as u32, format!("string({encoding})")))
            }
            FieldInfo::Primitive { ty, .. } => Some((scalar(ty), ty.clone())),
            FieldInfo::EnumFromU8 { enum_ty, .. }
            | FieldInfo::EnumFromI32FromU8 { enum_ty, .. } => Some((1, enum_ty.clone())),
            FieldInfo::EnumFromU32 { enum_ty, .. } | FieldInfo::EnumFromI32 { enum_ty, .. } => {
                Some((4, enum_ty.clone()))
            }
            FieldInfo::EnumFromI16 { enum_ty, .. } => Some((2, enum_ty.clone())),
            FieldInfo::VecU8 { size, .. } => Some((*size as u32, "bytes".into())),
            FieldInfo::InventoryItem { wire_type, .. } => {
                Some((scalar(wire_type), format!("InventoryItem({wire_type})")))
            }
            FieldInfo::Padding { count, ty, .. } => {
                Some(((*count as u32) * scalar(ty), format!("padding({ty})")))
            }
            FieldInfo::Array { size, ty, .. } => {
                Some(((*size as u32) * scalar(ty), format!("[{ty}; {size}]")))
            }
        }
    }

    /// Byte size contributed by a field in a `BinaryRecord`.
    ///
    /// This is the single source of truth for both the generated
    /// `<NAME>_RECORD_SIZE` constant and the data-length check in `parse`.
    pub(crate) fn binary_record_size(&self) -> usize {
        match self {
            FieldInfo::String { size, .. } => *size,
            FieldInfo::InventoryItem { wire_type, .. } => match wire_type.as_str() {
                "i32" => 4,
                "i16" => 2,
                other => panic!("Unsupported wire_type for InventoryItem: {other}"),
            },
            FieldInfo::Primitive { ty, .. } => match ty.as_str() {
                "i16" | "u16" => 2,
                "i32" | "u32" => 4,
                "u8" | "i8" => 1,
                other => panic!("Unsupported primitive type: {other}"),
            },
            FieldInfo::VecU8 { size, .. } => *size,
            FieldInfo::Array { size, .. } => *size,
            FieldInfo::Padding { count, ty, .. } => {
                let per = match ty.as_str() {
                    "i16" => 2,
                    "i32" => 4,
                    "u8" => 1,
                    other => panic!("Unsupported padding type: {other}"),
                };
                per * count
            }
            FieldInfo::Skip => 0,
            FieldInfo::Id { .. }
            | FieldInfo::Index { .. }
            | FieldInfo::EnumFromU8 { .. }
            | FieldInfo::EnumFromU32 { .. }
            | FieldInfo::EnumFromI16 { .. }
            | FieldInfo::EnumFromI32 { .. }
            | FieldInfo::EnumFromI32FromU8 { .. } => {
                panic!("BinaryRecord does not support this field kind")
            }
        }
    }
}

/// Emit the value assigned to a padding field after its bytes are consumed.
///
/// Shared by the `Extractor` and `BinaryRecord` codegens; both used to carry
/// a byte-for-byte identical copy of this expression.
pub(crate) fn padding_default_value_expr(ty: &str, default_value: Option<&str>) -> TokenStream2 {
    if let Some(dv) = default_value {
        match ty {
            "i16" => {
                let val = dv.parse::<i16>().expect("default_value must be i16");
                quote! { #val }
            }
            "i32" => {
                let val = dv.parse::<i32>().expect("default_value must be i32");
                quote! { #val }
            }
            "u8" => {
                let val = dv.parse::<u8>().expect("default_value must be u8");
                quote! { #val }
            }
            _ => panic!("Unsupported padding type: {ty}"),
        }
    } else {
        match ty {
            "i16" => quote! { 0i16 },
            "i32" => quote! { 0i32 },
            "u8" => quote! { 0u8 },
            _ => panic!("Unsupported padding type: {ty}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Shared nested-argument parsers. Every derive that accepts a
// `string(...)`, `padding(...)` or `inventory_item(...)` argument routes
// through these helpers, so the grammar lives in exactly one place.
// ---------------------------------------------------------------------------

fn parse_string_args(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<(String, usize)> {
    let mut encoding = None;
    let mut size = None;
    meta.parse_nested_meta(|string_meta| {
        if string_meta.path.is_ident("encoding") {
            let value = string_meta.value()?;
            let lit: LitStr = value.parse()?;
            encoding = Some(lit.value());
        } else if string_meta.path.is_ident("size") {
            let value = string_meta.value()?;
            let lit: LitInt = value.parse()?;
            size = Some(lit.base10_parse::<usize>().expect("size must be usize"));
        }
        Ok(())
    })?;
    Ok((
        encoding.expect("string requires encoding"),
        size.expect("string requires size"),
    ))
}

fn parse_padding_args(
    meta: &syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<(usize, String, Option<String>)> {
    let mut count = None;
    let mut padding_ty = None;
    let mut default_value = None;
    meta.parse_nested_meta(|pad_meta| {
        if pad_meta.path.is_ident("count") {
            let value = pad_meta.value()?;
            let lit: LitInt = value.parse()?;
            count = Some(lit.base10_parse::<usize>().expect("count must be usize"));
        } else if pad_meta.path.is_ident("type") {
            let value = pad_meta.value()?;
            let lit: LitStr = value.parse()?;
            padding_ty = Some(lit.value());
        } else if pad_meta.path.is_ident("default_value") {
            let value = pad_meta.value()?;
            let lit: LitStr = value.parse()?;
            default_value = Some(lit.value());
        }
        Ok(())
    })?;
    Ok((
        count.expect("padding requires count"),
        padding_ty.expect("padding requires type"),
        default_value,
    ))
}

/// Returns `None` when `wire_type` was omitted (the `Extract` grammar requires
/// it; `BinaryRecord` falls back to `i32`).
fn parse_inventory_wire_type(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<Option<String>> {
    let mut wire_type = None;
    meta.parse_nested_meta(|inv_meta| {
        if inv_meta.path.is_ident("wire_type") {
            let value = inv_meta.value()?;
            let lit: LitStr = value.parse()?;
            wire_type = Some(lit.value());
        }
        Ok(())
    })?;
    Ok(wire_type)
}

/// Parse a single field-level `#[extractor(...)]` attribute.
///
/// Returns the field metadata plus any struct-wide size overrides found in the
/// attribute (`counter_size` / `property_item_size` may appear on a field).
pub(crate) fn parse_extractor_attr<'a>(
    attr: &'a syn::Attribute,
    ident: &'a Ident,
) -> (Option<FieldInfo<'a>>, Option<u8>, Option<i32>) {
    let mut field_info = None;
    let mut counter_size = None;
    let mut property_item_size = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            field_info = Some(FieldInfo::Id { ident });
        } else if meta.path.is_ident("index") {
            field_info = Some(FieldInfo::Index { ident });
        } else if meta.path.is_ident("string") {
            let (encoding, size) = parse_string_args(&meta)?;
            field_info = Some(FieldInfo::String {
                ident,
                encoding,
                size,
            });
        } else if meta.path.is_ident("primitive") {
            let mut primitive_ty = None;
            meta.parse_nested_meta(|prim_meta| {
                if prim_meta.path.is_ident("type") {
                    let value = prim_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    primitive_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let primitive_ty = primitive_ty.expect("primitive requires type");
            field_info = Some(FieldInfo::Primitive {
                ident,
                ty: primitive_ty,
            });
        } else if meta.path.is_ident("enum_from_u8") {
            let mut enum_ty = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    enum_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let enum_ty = enum_ty.expect("enum_from_u8 requires type");
            field_info = Some(FieldInfo::EnumFromU8 { ident, enum_ty });
        } else if meta.path.is_ident("enum_from_u32") {
            let mut enum_ty = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    enum_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let enum_ty = enum_ty.expect("enum_from_u32 requires type");
            field_info = Some(FieldInfo::EnumFromU32 { ident, enum_ty });
        } else if meta.path.is_ident("enum_from_i16") {
            let mut enum_ty = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    enum_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let enum_ty = enum_ty.expect("enum_from_i16 requires type");
            field_info = Some(FieldInfo::EnumFromI16 { ident, enum_ty });
        } else if meta.path.is_ident("enum_from_i32") {
            let mut enum_ty = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    enum_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let enum_ty = enum_ty.expect("enum_from_i32 requires type");
            field_info = Some(FieldInfo::EnumFromI32 { ident, enum_ty });
        } else if meta.path.is_ident("enum_from_i32_from_u8") {
            let mut enum_ty = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    enum_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let enum_ty = enum_ty.expect("enum_from_i32_from_u8 requires type");
            field_info = Some(FieldInfo::EnumFromI32FromU8 { ident, enum_ty });
        } else if meta.path.is_ident("padding") {
            let (count, ty, default_value) = parse_padding_args(&meta)?;
            field_info = Some(FieldInfo::Padding {
                ident,
                count,
                ty,
                default_value,
            });
        } else if meta.path.is_ident("array") {
            let mut size = None;
            let mut array_ty = None;
            meta.parse_nested_meta(|arr_meta| {
                if arr_meta.path.is_ident("size") {
                    let value = arr_meta.value()?;
                    let lit: LitInt = value.parse()?;
                    size = Some(lit.base10_parse::<usize>().expect("size must be usize"));
                } else if arr_meta.path.is_ident("type") {
                    let value = arr_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    array_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let size = size.expect("array requires size");
            let array_ty = array_ty.expect("array requires type");
            field_info = Some(FieldInfo::Array {
                ident,
                size,
                ty: array_ty,
            });
        } else if meta.path.is_ident("vec_u8") {
            let mut size = None;
            meta.parse_nested_meta(|vec_meta| {
                if vec_meta.path.is_ident("size") {
                    let value = vec_meta.value()?;
                    let lit: LitInt = value.parse()?;
                    size = Some(lit.base10_parse::<usize>().expect("size must be usize"));
                }
                Ok(())
            })?;
            let size = size.expect("vec_u8 requires size");
            field_info = Some(FieldInfo::VecU8 { ident, size });
        } else if meta.path.is_ident("skip") {
            field_info = Some(FieldInfo::Skip);
        } else if meta.path.is_ident("inventory_item") {
            let wire_type =
                parse_inventory_wire_type(&meta)?.expect("inventory_item requires wire_type");
            field_info = Some(FieldInfo::InventoryItem { ident, wire_type });
        } else if meta.path.is_ident("counter_size") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            counter_size = Some(lit.base10_parse::<u8>().expect("counter_size must be u8"));
        } else if meta.path.is_ident("property_item_size") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            property_item_size = Some(
                lit.base10_parse::<i32>()
                    .expect("property_item_size must be i32"),
            );
        }
        Ok(())
    })
    .expect("Failed to parse #[extractor(...)] arguments");

    (field_info, counter_size, property_item_size)
}

// ---------------------------------------------------------------------------
// Whole-record schema — combines struct-level and field-level parsing.
// ---------------------------------------------------------------------------

pub(crate) struct RecordSchema<'a> {
    pub counter_size: u8,
    pub property_item_size: i32,
    pub fields: Vec<FieldInfo<'a>>,
}

/// Named fields of a struct, with a macro-specific diagnostic on bad shapes.
pub(crate) fn named_fields<'a>(
    input: &'a DeriveInput,
    macro_name: &str,
) -> &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma> {
    match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("{macro_name} only supports structs with named fields"),
        },
        _ => panic!("{macro_name} can only be derived for structs"),
    }
}

impl<'a> RecordSchema<'a> {
    /// Parse struct-level sizes and every field's `#[extractor(...)]` grammar
    /// in a single pass shared by all derive macros.
    pub(crate) fn parse(input: &'a DeriveInput) -> Self {
        let mut counter_size = 4u8;
        let mut property_item_size = None;

        for attr in &input.attrs {
            if attr.path().is_ident("extractor") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("counter_size") {
                        let value = meta.value()?;
                        let lit: LitInt = value.parse()?;
                        counter_size = lit.base10_parse::<u8>().expect("counter_size must be u8");
                    } else if meta.path.is_ident("property_item_size") {
                        let value = meta.value()?;
                        let lit: LitInt = value.parse()?;
                        property_item_size = Some(
                            lit.base10_parse::<i32>()
                                .expect("property_item_size must be i32"),
                        );
                    }
                    Ok(())
                })
                .expect("Failed to parse struct-level #[extractor(...)]");
            }
        }

        let mut fields = Vec::new();
        for field in named_fields(input, "Extractor") {
            let ident = field.ident.as_ref().expect("named field");
            let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("extractor")) else {
                continue;
            };
            let (info, field_counter, field_item_size) = parse_extractor_attr(attr, ident);
            if let Some(cs) = field_counter {
                counter_size = cs;
            }
            if let Some(pis) = field_item_size {
                property_item_size = Some(pis);
            }
            if let Some(info) = info {
                fields.push(info);
            }
        }

        let property_item_size = property_item_size.expect(
            "Extractor requires #[extractor(property_item_size = N)] on some field or the struct itself",
        );

        Self {
            counter_size,
            property_item_size,
            fields,
        }
    }

    /// Field layout metadata (name, offset, size, type) in declaration order.
    pub(crate) fn layout_fields(&self) -> Vec<(String, u32, u32, String)> {
        let mut offset = 0u32;
        let mut out = Vec::new();
        for info in &self.fields {
            let Some((size, ty)) = info.layout_size_and_type() else {
                continue;
            };
            let name = info
                .ident()
                .expect("layout fields always have an identifier")
                .to_string();
            out.push((name, offset, size, ty));
            offset += size;
        }
        out
    }

    /// The compile-time invariant previously checked inline in `layout_impl`.
    pub(crate) fn assert_layout_consistent(&self, type_name: &str) {
        let occupied = self
            .layout_fields()
            .iter()
            .map(|(_, _, size, _)| size)
            .sum::<u32>();
        let declared = self.property_item_size as u32;
        assert_eq!(
            occupied, declared,
            "RecordLayout fields for {type_name} occupy {occupied} bytes, but property_item_size is {declared}"
        );
    }
}

// ---------------------------------------------------------------------------
// `BinaryRecord` field grammar.
//
// The derive historically used a different attribute name (`binary_record`),
// accepted a bare `size = N`, defaulted `inventory_item` to an i32 wire type,
// and auto-detected fields from their Rust type when no attribute was present.
// Those are *grammar* differences only: the resulting metadata is the very
// same `FieldInfo` used by every other binary codegen path.
// ---------------------------------------------------------------------------

/// Parse a `#[binary_record(...)]` field attribute into shared metadata.
fn parse_binary_record_attr<'a>(attr: &syn::Attribute, ident: &'a Ident) -> Option<FieldInfo<'a>> {
    let mut field_info = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("string") {
            let (encoding, size) = parse_string_args(&meta)?;
            field_info = Some(FieldInfo::String {
                ident,
                encoding,
                size,
            });
        } else if meta.path.is_ident("size") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            let size = lit.base10_parse::<usize>().expect("size must be usize");
            field_info = Some(FieldInfo::VecU8 { ident, size });
        } else if meta.path.is_ident("padding") {
            let (count, ty, default_value) = parse_padding_args(&meta)?;
            field_info = Some(FieldInfo::Padding {
                ident,
                count,
                ty,
                default_value,
            });
        } else if meta.path.is_ident("inventory_item") {
            let wire_type = parse_inventory_wire_type(&meta)?.unwrap_or_else(|| "i32".to_string());
            field_info = Some(FieldInfo::InventoryItem { ident, wire_type });
        } else if meta.path.is_ident("skip") {
            field_info = Some(FieldInfo::Skip);
        }
        Ok(())
    })
    .expect("Failed to parse #[binary_record(...)] arguments");

    field_info
}

/// Infer shared field metadata from the Rust type when a `BinaryRecord` field
/// has no `#[binary_record(...)]` attribute.
fn auto_detect_binary_record_field<'a>(ident: &'a Ident, ty: &Type) -> Option<FieldInfo<'a>> {
    // `[u8; N]` — fixed byte array.
    if let Type::Array(arr) = ty
        && let Type::Path(elem_path) = arr.elem.as_ref()
        && elem_path.path.is_ident("u8")
        && let syn::Expr::Lit(lit) = &arr.len
        && let syn::Lit::Int(n) = &lit.lit
    {
        let size = n.base10_parse::<usize>().expect("array size must be usize");
        return Some(FieldInfo::Array {
            ident,
            size,
            ty: "u8".to_string(),
        });
    }

    // `Vec<u8>` requires an explicit size annotation.
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(Type::Path(elem))) = args.args.first()
        && elem.path.is_ident("u8")
    {
        panic!("Field `{ident}` is Vec<u8> but has no #[binary_record(size = N)] annotation");
    }

    // Primitives, InventoryItem, and the diagnostic for a bare String.
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let type_name = segment.ident.to_string();
        match type_name.as_str() {
            "i16" | "i32" | "i8" | "u8" | "u16" | "u32" => {
                return Some(FieldInfo::Primitive {
                    ident,
                    ty: type_name,
                });
            }
            "InventoryItem" => {
                return Some(FieldInfo::InventoryItem {
                    ident,
                    wire_type: "i32".to_string(),
                });
            }
            "String" => {
                panic!(
                    "Field `{ident}` is `String` but has no #[binary_record(string(...))] annotation"
                );
            }
            _ => return None,
        }
    }

    None
}

/// Parse every field of a `BinaryRecord` into the shared `FieldInfo` metadata.
pub(crate) fn parse_binary_record_fields(input: &DeriveInput) -> Vec<FieldInfo<'_>> {
    let mut fields = Vec::new();
    for field in named_fields(input, "BinaryRecord") {
        let ident = field.ident.as_ref().expect("named field");
        let info = match field
            .attrs
            .iter()
            .find(|a| a.path().is_ident("binary_record"))
        {
            Some(attr) => parse_binary_record_attr(attr, ident),
            None => auto_detect_binary_record_field(ident, &field.ty),
        };
        if let Some(info) = info {
            fields.push(info);
        }
    }
    fields
}

// ---------------------------------------------------------------------------
// Patcher registration grammar — shared by both patcher derives.
// ---------------------------------------------------------------------------

/// How a generated patcher is registered with the patcher registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PatcherKey {
    Filename {
        filename: String,
        /// Additional filenames that share the primary filename's record format.
        aliases: Vec<String>,
    },
    Pattern {
        extension: String,
        stem_prefix: String,
    },
}

impl PatcherKey {
    pub(crate) fn parse(input: &DeriveInput, macro_name: &str) -> Self {
        let mut filename = None;
        let mut extension = None;
        let mut stem_prefix = None;
        let mut aliases: Vec<String> = Vec::new();

        for attr in &input.attrs {
            if attr.path().is_ident("patcher") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("filename") {
                        let value = meta.value()?;
                        filename = Some(value.parse::<LitStr>()?.value());
                    } else if meta.path.is_ident("alias") {
                        let value = meta.value()?;
                        aliases.push(value.parse::<LitStr>()?.value());
                    } else if meta.path.is_ident("extension") {
                        let value = meta.value()?;
                        extension = Some(value.parse::<LitStr>()?.value());
                    } else if meta.path.is_ident("stem_prefix") {
                        let value = meta.value()?;
                        stem_prefix = Some(value.parse::<LitStr>()?.value());
                    }
                    Ok(())
                })
                .expect("Failed to parse #[patcher(...)] arguments");
            }
        }

        match (filename, extension, stem_prefix) {
            (Some(f), None, None) => Self::Filename {
                filename: f,
                aliases,
            },
            (None, Some(ext), Some(prefix)) => Self::Pattern {
                extension: ext,
                stem_prefix: prefix,
            },
            _ => panic!(
                "{macro_name} requires either #[patcher(filename = \"...\")] or \
                 #[patcher(extension = \"...\", stem_prefix = \"...\")] on the struct"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_struct(src: &str) -> DeriveInput {
        syn::parse_str(src).expect("fixture must parse")
    }

    fn single_field(src: &str) -> (DeriveInput, syn::Field) {
        let input = parse_struct(src);
        let field = named_fields(&input, "Extractor").first().unwrap().clone();
        (input, field)
    }

    fn info_tokens(info: &FieldInfo<'_>) -> String {
        match info {
            FieldInfo::Id { .. } => "id".into(),
            FieldInfo::Index { .. } => "index".into(),
            FieldInfo::String { encoding, size, .. } => format!("string({encoding},{size})"),
            FieldInfo::Primitive { ty, .. } => format!("primitive({ty})"),
            FieldInfo::InventoryItem { wire_type, .. } => format!("inventory({wire_type})"),
            FieldInfo::EnumFromU8 { enum_ty, .. } => format!("u8({enum_ty})"),
            FieldInfo::EnumFromU32 { enum_ty, .. } => format!("u32({enum_ty})"),
            FieldInfo::EnumFromI16 { enum_ty, .. } => format!("i16({enum_ty})"),
            FieldInfo::EnumFromI32 { enum_ty, .. } => format!("i32({enum_ty})"),
            FieldInfo::EnumFromI32FromU8 { enum_ty, .. } => format!("i32fromu8({enum_ty})"),
            FieldInfo::Padding {
                count,
                ty,
                default_value,
                ..
            } => {
                format!("padding({count},{ty},{default_value:?})")
            }
            FieldInfo::Array { size, ty, .. } => format!("array({size},{ty})"),
            FieldInfo::VecU8 { size, .. } => format!("vec_u8({size})"),
            FieldInfo::Skip => "skip".into(),
        }
    }

    #[test]
    fn test_parse_extractor_attr_recognizes_every_field_kind() {
        let cases = [
            ("#[extractor(id)] id: i32,", "id"),
            ("#[extractor(index)] id: i32,", "index"),
            (
                "#[extractor(string(encoding = \"EUC-KR\", size = 24))] name: String,",
                "string(EUC-KR,24)",
            ),
            (
                "#[extractor(primitive(type = \"i32\"))] hp: i32,",
                "primitive(i32)",
            ),
            (
                "#[extractor(inventory_item(wire_type = \"u16\"))] item: InventoryItem,",
                "inventory(u16)",
            ),
            (
                "#[extractor(enum_from_u8(type = \"Flag\"))] f: Flag,",
                "u8(Flag)",
            ),
            (
                "#[extractor(enum_from_u32(type = \"Flag\"))] f: Flag,",
                "u32(Flag)",
            ),
            (
                "#[extractor(enum_from_i16(type = \"Flag\"))] f: Flag,",
                "i16(Flag)",
            ),
            (
                "#[extractor(enum_from_i32(type = \"Flag\"))] f: Flag,",
                "i32(Flag)",
            ),
            (
                "#[extractor(enum_from_i32_from_u8(type = \"Flag\"))] f: Flag,",
                "i32fromu8(Flag)",
            ),
            (
                "#[extractor(padding(count = 2, type = \"i16\", default_value = \"7\"))] pad: i16,",
                "padding(2,i16,Some(\"7\"))",
            ),
            (
                "#[extractor(array(size = 4, type = \"u8\"))] raw: [u8; 4],",
                "array(4,u8)",
            ),
            ("#[extractor(vec_u8(size = 8))] raw: Vec<u8>,", "vec_u8(8)"),
            ("#[extractor(skip)] ignored: u32,", "skip"),
        ];

        for (field_src, expected) in cases {
            let src = format!("struct S {{ {field_src} }}");
            let (_, field) = single_field(&src);
            let attr = field.attrs.first().unwrap();
            let (info, counter, item_size) =
                parse_extractor_attr(attr, field.ident.as_ref().unwrap());
            let info = info.unwrap_or_else(|| panic!("no FieldInfo for {field_src}"));
            assert_eq!(info_tokens(&info), expected, "for `{field_src}`");
            assert_eq!(counter, None);
            assert_eq!(item_size, None);
        }
    }

    #[test]
    fn test_parse_extractor_attr_reads_size_overrides_on_a_field() {
        let (_, field) = single_field(
            "struct S { #[extractor(primitive(type = \"i16\"), counter_size = 2, property_item_size = 84)] x: i16 }",
        );
        let attr = field.attrs.first().unwrap();
        let (info, counter, item_size) = parse_extractor_attr(attr, field.ident.as_ref().unwrap());
        assert_eq!(info_tokens(&info.unwrap()), "primitive(i16)");
        assert_eq!(counter, Some(2));
        assert_eq!(item_size, Some(84));
    }

    #[test]
    fn test_record_schema_parses_struct_level_and_field_level_attributes() {
        let input = parse_struct(
            r#"
            #[extractor(counter_size = 0, property_item_size = 26)]
            struct Rec {
                #[extractor(index)] id: i32,
                #[extractor(string(encoding = "WINDOWS-1250", size = 24))] name: String,
                #[extractor(primitive(type = "i16"))] value: i16,
            }
            "#,
        );
        let schema = RecordSchema::parse(&input);
        assert_eq!(schema.counter_size, 0);
        assert_eq!(schema.property_item_size, 26);
        assert_eq!(schema.fields.len(), 3);
        assert_eq!(
            schema.layout_fields(),
            vec![
                ("name".into(), 0, 24, "string(WINDOWS-1250)".into()),
                ("value".into(), 24, 2, "i16".into()),
            ]
        );
        schema.assert_layout_consistent("Rec");
    }

    #[test]
    fn test_record_schema_field_level_size_overrides_are_honored() {
        let input = parse_struct(
            r#"
            struct Rec {
                #[extractor(primitive(type = "i16"), counter_size = 1, property_item_size = 2)] x: i16,
            }
            "#,
        );
        let schema = RecordSchema::parse(&input);
        assert_eq!(schema.counter_size, 1);
        assert_eq!(schema.property_item_size, 2);
    }

    #[test]
    #[should_panic(expected = "occupy 4 bytes, but property_item_size is 8")]
    fn test_assert_layout_consistent_rejects_mismatched_size() {
        let input = parse_struct(
            r#"
            #[extractor(property_item_size = 8)]
            struct Rec {
                #[extractor(primitive(type = "i32"))] x: i32,
            }
            "#,
        );
        RecordSchema::parse(&input).assert_layout_consistent("Rec");
    }

    #[test]
    fn test_encoding_constant_maps_both_spellings() {
        let config = CodegenConfig::default();
        for name in ["WINDOWS-1250", "WINDOWS_1250"] {
            assert_eq!(
                config.encoding_constant(name).to_string(),
                "encoding_rs :: WINDOWS_1250"
            );
        }
        for name in ["EUC-KR", "EUC_KR"] {
            assert_eq!(
                config.encoding_constant(name).to_string(),
                "encoding_rs :: EUC_KR"
            );
        }
        for name in ["UTF-8", "UTF_8"] {
            assert_eq!(
                config.encoding_constant(name).to_string(),
                "encoding_rs :: UTF_8"
            );
        }
    }

    #[test]
    fn test_codegen_config_default_emits_historical_paths() {
        let config = CodegenConfig::default();
        assert_eq!(
            config.extractor_trait().to_string(),
            "crate :: references :: extractor :: Extractor"
        );
        assert_eq!(
            config.read_mapper().to_string(),
            "crate :: references :: extractor :: read_mapper"
        );
        assert_eq!(
            config.parse_null().to_string(),
            "crate :: references :: extractor :: parse_null"
        );
        assert_eq!(
            config.inventory_item_type().to_string(),
            "crate :: references :: enums :: InventoryItem"
        );
        assert_eq!(
            config.layout("FieldDef").to_string(),
            "crate :: references :: layout :: FieldDef"
        );
        assert_eq!(
            config.modding(&["value", "Value"]).to_string(),
            "crate :: modding :: value :: Value"
        );
        assert_eq!(
            config.localization(&["TextEncoding", "EucKr"]).to_string(),
            "crate :: localization :: TextEncoding :: EucKr"
        );
        assert_eq!(config.byteorder().to_string(), "byteorder");
        assert_eq!(config.encoding_rs().to_string(), "encoding_rs");
        assert_eq!(config.encoding_rs_io().to_string(), "encoding_rs_io");
    }

    #[test]
    fn test_codegen_config_renames_paths_from_one_place() {
        let config = CodegenConfig {
            references_mod: "refs".into(),
            enums_mod: "item_enums".into(),
            modding_mod: "mods".into(),
            byteorder_crate: "bo".into(),
            ..CodegenConfig::default()
        };
        assert_eq!(
            config.extractor_trait().to_string(),
            "crate :: refs :: extractor :: Extractor"
        );
        assert_eq!(
            config.inventory_item_type().to_string(),
            "crate :: refs :: item_enums :: InventoryItem"
        );
        assert_eq!(
            config.modding(&["value", "Value"]).to_string(),
            "crate :: mods :: value :: Value"
        );
        assert_eq!(
            config.encoding_constant("EUC-KR").to_string(),
            "encoding_rs :: EUC_KR"
        );
        assert_eq!(config.byteorder().to_string(), "bo");
    }

    #[test]
    fn test_parse_patcher_key_supports_both_forms() {
        let by_filename =
            parse_struct("#[patcher(filename = \"MiscItem.db\")] struct MiscItem { value: i32 }");
        match PatcherKey::parse(&by_filename, "RecordPatcher") {
            PatcherKey::Filename { filename, aliases } => {
                assert_eq!(filename, "MiscItem.db");
                assert!(aliases.is_empty());
            }
            other => panic!("expected Filename, got {other:?}"),
        }

        let with_alias = parse_struct(
            "#[patcher(filename = \"Magic.db\", alias = \"MulMagic.db\")] struct MagicSpell { value: i32 }",
        );
        match PatcherKey::parse(&with_alias, "RecordPatcher") {
            PatcherKey::Filename { filename, aliases } => {
                assert_eq!(filename, "Magic.db");
                assert_eq!(aliases, vec!["MulMagic.db"]);
            }
            other => panic!("expected Filename, got {other:?}"),
        }

        let by_pattern = parse_struct(
            "#[patcher(extension = \"ref\", stem_prefix = \"ext\")] struct ExtraRef { value: i32 }",
        );
        match PatcherKey::parse(&by_pattern, "RecordPatcher") {
            PatcherKey::Pattern {
                extension,
                stem_prefix,
            } => {
                assert_eq!(extension, "ref");
                assert_eq!(stem_prefix, "ext");
            }
            other => panic!("expected Pattern, got {other:?}"),
        }
    }

    #[test]
    #[should_panic(expected = "RecordPatcher requires either")]
    fn test_parse_patcher_key_rejects_incomplete_pattern() {
        let input = parse_struct("#[patcher(extension = \"ref\")] struct Bad { value: i32 }");
        PatcherKey::parse(&input, "RecordPatcher");
    }

    #[test]
    fn test_text_encoding_maps_every_spelling() {
        let config = CodegenConfig::default();
        assert_eq!(
            config.text_encoding("WINDOWS-1250").to_string(),
            "crate :: localization :: TextEncoding :: Windows1250"
        );
        assert_eq!(
            config.text_encoding("EUC_KR").to_string(),
            "crate :: localization :: TextEncoding :: EucKr"
        );
        assert_eq!(
            config.text_encoding("UTF_8").to_string(),
            "crate :: localization :: TextEncoding :: Utf8"
        );
    }
}
