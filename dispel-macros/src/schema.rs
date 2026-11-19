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
    /// name. Both the hyphenated (`EUC-KR`) and underscore (`EUC_KR`) spellings
    /// are accepted, as used by the different derive macros.
    pub(crate) fn encoding_constant(&self, encoding: &str) -> TokenStream2 {
        let constant = match encoding {
            "WINDOWS-1250" | "WINDOWS_1250" => "WINDOWS_1250",
            "EUC-KR" | "EUC_KR" => "EUC_KR",
            "UTF-8" | "UTF_8" => "UTF_8",
            other => panic!("Unknown encoding '{other}'; expected WINDOWS-1250, EUC-KR, or UTF-8"),
        };
        let enc = self.encoding_rs();
        let constant = Self::ident(constant);
        quote! { #enc::#constant }
    }

    /// `crate::localization::TextEncoding::<variant>` for a textual encoding.
    pub(crate) fn text_encoding(&self, encoding: &str) -> TokenStream2 {
        let variant = match encoding {
            "WINDOWS-1250" | "WINDOWS_1250" => "Windows1250",
            "EUC-KR" | "EUC_KR" => "EucKr",
            "UTF-8" | "UTF_8" => "Utf8",
            other => panic!(
                "Unknown encoding '{other}' in #[translatable]; expected WINDOWS-1250, EUC-KR, or UTF-8"
            ),
        };
        self.localization(&["TextEncoding", variant])
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
    Nested {
        ident: &'a Ident,
        item_ty: &'a Type,
        max_items: usize,
        item_size: i32,
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
            | FieldInfo::VecU8 { ident, .. }
            | FieldInfo::Nested { ident, .. } => Some(ident),
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
            FieldInfo::Nested {
                item_size,
                max_items,
                ..
            } => Some((
                (*max_items as u32) * (*item_size as u32),
                format!("Nested(max={max_items}, item_size={item_size})"),
            )),
        }
    }
}

/// Parse a single field-level `#[extractor(...)]` attribute.
///
/// Returns the field metadata plus any struct-wide size overrides found in the
/// attribute (`counter_size` / `property_item_size` may appear on a field).
pub(crate) fn parse_extractor_attr<'a>(
    attr: &'a syn::Attribute,
    ident: &'a Ident,
    ty: &'a Type,
) -> (Option<FieldInfo<'a>>, Option<u8>, Option<i32>) {
    let mut field_info = None;
    let mut counter_size = None;
    let mut property_item_size = None;
    let mut nested_max_items: Option<usize> = None;
    let mut nested_item_size: Option<i32> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            field_info = Some(FieldInfo::Id { ident });
        } else if meta.path.is_ident("index") {
            field_info = Some(FieldInfo::Index { ident });
        } else if meta.path.is_ident("string") {
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
            let encoding = encoding.expect("string requires encoding");
            let size = size.expect("string requires size");
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
            let count = count.expect("padding requires count");
            let padding_ty = padding_ty.expect("padding requires type");
            field_info = Some(FieldInfo::Padding {
                ident,
                count,
                ty: padding_ty,
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
        } else if meta.path.is_ident("nested") {
            // `nested` is a flag (no parens). `max_items` / `item_size` are
            // top-level metas on the same attribute (handled below).
        } else if meta.path.is_ident("max_items") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            nested_max_items = Some(
                lit.base10_parse::<usize>()
                    .expect("max_items must be usize"),
            );
        } else if meta.path.is_ident("item_size") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            nested_item_size = Some(lit.base10_parse::<i32>().expect("item_size must be i32"));
        } else if meta.path.is_ident("skip") {
            field_info = Some(FieldInfo::Skip);
        } else if meta.path.is_ident("inventory_item") {
            let mut wire_type = None;
            meta.parse_nested_meta(|inv_meta| {
                if inv_meta.path.is_ident("wire_type") {
                    let value = inv_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    wire_type = Some(lit.value());
                }
                Ok(())
            })?;
            let wire_type = wire_type.expect("inventory_item requires wire_type");
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

    if let (Some(max_items), Some(item_size)) = (nested_max_items, nested_item_size) {
        field_info = Some(FieldInfo::Nested {
            ident,
            item_ty: ty,
            max_items,
            item_size,
        });
    }

    (field_info, counter_size, property_item_size)
}

/// Extract the inner `T` from a field declared as `Vec<T>`.
pub(crate) fn extract_vec_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let last = type_path.path.segments.last()?;
    if last.ident != "Vec" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    let inner = args.args.first()?;
    if let syn::GenericArgument::Type(inner_ty) = inner {
        Some(inner_ty)
    } else {
        None
    }
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
            let (info, field_counter, field_item_size) =
                parse_extractor_attr(attr, ident, &field.ty);
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
// Patcher registration grammar — shared by both patcher derives.
// ---------------------------------------------------------------------------

/// How a generated patcher is registered with the patcher registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PatcherKey {
    Filename(String),
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

        for attr in &input.attrs {
            if attr.path().is_ident("patcher") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("filename") {
                        let value = meta.value()?;
                        filename = Some(value.parse::<LitStr>()?.value());
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
            (Some(f), None, None) => Self::Filename(f),
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
    use syn::parse_quote;

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
            FieldInfo::Nested {
                max_items,
                item_size,
                ..
            } => {
                format!("nested({max_items},{item_size})")
            }
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
            (
                "#[extractor(nested, max_items = 3, item_size = 12)] items: Vec<Inner>,",
                "nested(3,12)",
            ),
        ];

        for (field_src, expected) in cases {
            let src = format!("struct S {{ {field_src} }}");
            let (_, field) = single_field(&src);
            let attr = field.attrs.first().unwrap();
            let (info, counter, item_size) =
                parse_extractor_attr(attr, field.ident.as_ref().unwrap(), &field.ty);
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
        let (info, counter, item_size) =
            parse_extractor_attr(attr, field.ident.as_ref().unwrap(), &field.ty);
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
    fn test_extract_vec_inner_only_accepts_vec_of_one_type() {
        let ty: Type = parse_quote! { Vec<Inner> };
        let inner = extract_vec_inner(&ty).expect("Vec<Inner>");
        assert_eq!(quote! { #inner }.to_string(), "Inner");
        let not_vec: Type = parse_quote! { Option<u8> };
        assert!(extract_vec_inner(&not_vec).is_none());
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
            PatcherKey::Filename(name) => assert_eq!(name, "MiscItem.db"),
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
