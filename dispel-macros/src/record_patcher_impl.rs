use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, LitStr};

use crate::extractor_impl::{parse_extractor_attr, FieldInfo};

pub fn expand(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let patcher_ident = Ident::new(&format!("{name}Patcher"), Span::call_site());
    let name_str = name.to_string();

    // Two forms of `#[patcher(...)]`:
    //   1. `#[patcher(filename = "MiscItem.db")]`            — exact match
    //   2. `#[patcher(extension = "ref", stem_prefix = "ext")]` — pattern,
    //      for files whose name varies per map (`Extdun01.ref`, etc.).
    let mut filename: Option<String> = None;
    let mut extension: Option<String> = None;
    let mut stem_prefix: Option<String> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("patcher") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("filename") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    filename = Some(lit.value());
                } else if meta.path.is_ident("extension") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    extension = Some(lit.value());
                } else if meta.path.is_ident("stem_prefix") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    stem_prefix = Some(lit.value());
                }
                Ok(())
            })
            .expect("Failed to parse #[patcher(...)] arguments");
        }
    }
    let key_consts = match (&filename, &extension, &stem_prefix) {
        (Some(f), None, None) => quote! {
            /// The relative filename this patcher targets (exact match).
            pub const FILENAME: &'static str = #f;
        },
        (None, Some(ext), Some(prefix)) => quote! {
            /// Extension this patcher matches (case-insensitive, no leading dot).
            pub const EXTENSION: &'static str = #ext;
            /// Filename-stem prefix this patcher matches (case-insensitive).
            pub const STEM_PREFIX: &'static str = #prefix;
        },
        _ => panic!(
            "RecordPatcher requires either #[patcher(filename = \"...\")] or \
             #[patcher(extension = \"...\", stem_prefix = \"...\")] on the struct"
        ),
    };

    // Re-parse field-level #[extractor(...)] attributes via the existing helper.
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("RecordPatcher only supports structs with named fields"),
        },
        _ => panic!("RecordPatcher can only be derived for structs"),
    };

    let mut field_arms: Vec<TokenStream2> = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_name = field_ident.to_string();
        let field_ty = &field.ty;
        let attr = match field.attrs.iter().find(|a| a.path().is_ident("extractor")) {
            Some(a) => a,
            None => continue,
        };
        let (info, _, _) = parse_extractor_attr(attr, field_ident, field_ty);
        let Some(info) = info else { continue };

        let arm = match info {
            FieldInfo::Id { .. } | FieldInfo::Index { .. } => quote! {
                #field_name => {
                    return Err(crate::modding::error::ModdingError::Malformed(format!(
                        "{}.{} is positional and cannot be patched",
                        Self::RECORD_NAME, #field_name
                    )));
                }
            },
            FieldInfo::String { ident, .. } => quote! {
                #field_name => match new {
                    crate::modding::value::Value::String(s) => rec.#ident = s.clone(),
                    _ => return Err(crate::modding::patcher::wrong_type(
                        Self::RECORD_NAME, #field_name, "string", new,
                    )),
                },
            },
            FieldInfo::Primitive { ident, ty } => primitive_arm(&field_name, ident, &ty),
            FieldInfo::InventoryItem { ident, .. } => {
                let field_name_str = field_name.clone();
                quote! {
                    #field_name_str => match new {
                        crate::modding::value::Value::I64(v) => {
                            rec.#ident = crate::references::enums::InventoryItem::from(*v as i32);
                        }
                        crate::modding::value::Value::String(s) => match s.trim().parse::<i32>() {
                            Ok(v) => rec.#ident = crate::references::enums::InventoryItem::from(v),
                            Err(_) => return Err(crate::modding::patcher::wrong_type(
                                Self::RECORD_NAME, #field_name_str, "i32", new,
                            )),
                        },
                        _ => return Err(crate::modding::patcher::wrong_type(
                            Self::RECORD_NAME, #field_name_str, "i32", new,
                        )),
                    },
                }
            },
            FieldInfo::EnumFromU8 { ident, enum_ty } => {
                enum_arm(&field_name, ident, &enum_ty, "u8", "from_u8")
            }
            FieldInfo::EnumFromU32 { ident, enum_ty } => {
                enum_arm(&field_name, ident, &enum_ty, "u32", "from_u32")
            }
            FieldInfo::EnumFromI16 { ident, enum_ty } => {
                enum_arm(&field_name, ident, &enum_ty, "i16", "from_i16")
            }
            FieldInfo::EnumFromI32 { ident, enum_ty } => {
                enum_arm(&field_name, ident, &enum_ty, "i32", "from_i32")
            }
            FieldInfo::EnumFromI32FromU8 { ident, enum_ty } => {
                enum_arm(&field_name, ident, &enum_ty, "u8", "from_u8")
            }
            FieldInfo::Array { ident, size, ty } => {
                if ty != "u8" {
                    panic!("RecordPatcher only supports array(type = \"u8\")");
                }
                quote! {
                    #field_name => match new {
                        crate::modding::value::Value::Bytes(b) if b.len() == #size => {
                            let mut arr = [0u8; #size];
                            arr.copy_from_slice(b);
                            rec.#ident = arr;
                        }
                        crate::modding::value::Value::Bytes(_) => {
                            return Err(crate::modding::error::ModdingError::Malformed(format!(
                                "{}.{}: expected {} bytes, got {}",
                                Self::RECORD_NAME, #field_name, #size,
                                if let crate::modding::value::Value::Bytes(b) = new { b.len() } else { 0 }
                            )));
                        }
                        _ => return Err(crate::modding::patcher::wrong_type(
                            Self::RECORD_NAME, #field_name,
                            concat!("bytes(", stringify!(#size), ")"), new,
                        )),
                    },
                }
            }
            FieldInfo::VecU8 { ident, size } => quote! {
                #field_name => match new {
                    crate::modding::value::Value::Bytes(b) if b.len() == #size => {
                        rec.#ident = b.clone();
                    }
                    crate::modding::value::Value::Bytes(_) => {
                        return Err(crate::modding::error::ModdingError::Malformed(format!(
                            "{}.{}: expected {} bytes, got {}",
                            Self::RECORD_NAME, #field_name, #size,
                            if let crate::modding::value::Value::Bytes(b) = new { b.len() } else { 0 }
                        )));
                    }
                    _ => return Err(crate::modding::patcher::wrong_type(
                        Self::RECORD_NAME, #field_name,
                        concat!("bytes(", stringify!(#size), ")"), new,
                    )),
                },
            },
            FieldInfo::Padding { .. } => quote! {
                #field_name => {
                    return Err(crate::modding::error::ModdingError::Malformed(format!(
                        "{}.{} is synthetic padding and cannot be patched",
                        Self::RECORD_NAME, #field_name
                    )));
                }
            },
            FieldInfo::Skip => continue,
        };
        field_arms.push(arm);
    }

    let expanded = quote! {
        /// Auto-generated patcher for the surrounding struct. See
        /// [`crate::modding::patcher::RecordPatcher`].
        pub struct #patcher_ident;

        impl #patcher_ident {
            #key_consts
            /// Human-readable record name, used in error messages.
            pub const RECORD_NAME: &'static str = #name_str;
        }

        impl crate::modding::patcher::RecordPatcher for #patcher_ident {
            fn name(&self) -> &'static str {
                Self::RECORD_NAME
            }

            fn apply_field(
                &self,
                bytes: &[u8],
                record_id: u32,
                field: &str,
                new: &crate::modding::value::Value,
            ) -> crate::modding::error::Result<Vec<u8>> {
                use crate::references::extractor::Extractor as _;

                let mut cursor = std::io::Cursor::new(bytes);
                let mut records = #name::parse(&mut cursor, bytes.len() as u64)?;

                let idx = record_id as usize;
                if idx >= records.len() {
                    return Err(crate::modding::patcher::out_of_range(
                        Self::RECORD_NAME, record_id, records.len(),
                    ));
                }
                let rec = &mut records[idx];

                match field {
                    #(#field_arms)*
                    other => return Err(crate::modding::patcher::unknown_field(
                        Self::RECORD_NAME, other,
                    )),
                }

                let mut out = Vec::with_capacity(bytes.len());
                #name::to_writer(&records, &mut out)?;
                Ok(out)
            }
        }
    };

    expanded
}

/// Generate a `match` arm for a primitive numeric field. Accepts
/// `Value::I64` (range-checked), `Value::F64` for `f` types (none yet),
/// and `Value::String` (parsed) so recording-mode stringly deltas work.
fn primitive_arm(field_name: &str, ident: &Ident, ty: &str) -> TokenStream2 {
    let ty_ident = Ident::new(ty, Span::call_site());
    let expected = ty.to_string();
    quote! {
        #field_name => match new {
            crate::modding::value::Value::I64(v) => {
                rec.#ident = (*v) as #ty_ident;
            }
            crate::modding::value::Value::String(s) => match s.trim().parse::<#ty_ident>() {
                Ok(v) => rec.#ident = v,
                Err(_) => return Err(crate::modding::patcher::wrong_type(
                    Self::RECORD_NAME, #field_name, #expected, new,
                )),
            },
            _ => return Err(crate::modding::patcher::wrong_type(
                Self::RECORD_NAME, #field_name, #expected, new,
            )),
        },
    }
}

/// Generate a `match` arm for an enum field. Accepts `Value::I64` (cast to
/// the wire type, then `from_X`) and `Value::String` (parsed as the wire
/// type). Falls back to `Default::default()` if the discriminant is invalid,
/// matching the parse-side behavior in `Extractor`.
fn enum_arm(
    field_name: &str,
    ident: &Ident,
    enum_ty: &str,
    wire_ty: &str,
    from_fn: &str,
) -> TokenStream2 {
    let enum_ident = Ident::new(enum_ty, Span::call_site());
    let wire_ident = Ident::new(wire_ty, Span::call_site());
    let from_ident = Ident::new(from_fn, Span::call_site());
    let expected = format!("i64 (discriminant of {})", enum_ty);
    quote! {
        #field_name => match new {
            crate::modding::value::Value::I64(v) => {
                rec.#ident = #enum_ident::#from_ident((*v) as #wire_ident).unwrap_or_default();
            }
            crate::modding::value::Value::String(s) => match s.trim().parse::<#wire_ident>() {
                Ok(v) => rec.#ident = #enum_ident::#from_ident(v).unwrap_or_default(),
                Err(_) => return Err(crate::modding::patcher::wrong_type(
                    Self::RECORD_NAME, #field_name, #expected, new,
                )),
            },
            _ => return Err(crate::modding::patcher::wrong_type(
                Self::RECORD_NAME, #field_name, #expected, new,
            )),
        },
    }
}
