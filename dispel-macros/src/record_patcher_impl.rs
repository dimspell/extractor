use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Ident};

use crate::schema::{CodegenConfig, FieldInfo, PatcherKey, RecordSchema};

pub fn expand(input: DeriveInput) -> TokenStream2 {
    codegen(&input, &CodegenConfig::default())
}

/// Generate the `RecordPatcher` impl from the shared record schema.
pub(crate) fn codegen(input: &DeriveInput, config: &CodegenConfig) -> TokenStream2 {
    let name = &input.ident;
    let patcher_ident = Ident::new(&format!("{name}Patcher"), Span::call_site());
    let name_str = name.to_string();

    let key_consts = match PatcherKey::parse(input, "RecordPatcher") {
        PatcherKey::Filename(filename) => quote! {
            /// The relative filename this patcher targets (exact match).
            pub const FILENAME: &'static str = #filename;
        },
        PatcherKey::Pattern {
            extension,
            stem_prefix,
        } => quote! {
            /// Extension this patcher matches (case-insensitive, no leading dot).
            pub const EXTENSION: &'static str = #extension;
            /// Filename-stem prefix this patcher matches (case-insensitive).
            pub const STEM_PREFIX: &'static str = #stem_prefix;
        },
    };

    let malformed = config.modding(&["error", "ModdingError", "Malformed"]);
    let value = config.modding(&["value", "Value"]);
    let wrong_type = config.modding(&["patcher", "wrong_type"]);
    let result = config.modding(&["error", "Result"]);
    let record_patcher_trait = config.modding(&["patcher", "RecordPatcher"]);
    let out_of_range = config.modding(&["patcher", "out_of_range"]);
    let unknown_field = config.modding(&["patcher", "unknown_field"]);
    let inventory_item = config.inventory_item_type();
    let extractor_trait = config.extractor_trait();

    // The field grammar comes from the one shared schema parser; the patch
    // arms below are just another codegen path over that AST.
    let schema = RecordSchema::parse(input);
    let mut field_arms: Vec<TokenStream2> = Vec::new();

    for info in &schema.fields {
        let Some(ident) = info.ident() else { continue };
        let field_name = ident.to_string();
        let arm = match info {
            FieldInfo::Id { .. } | FieldInfo::Index { .. } => quote! {
                #field_name => {
                    return Err(#malformed(format!(
                        "{}.{} is positional and cannot be patched",
                        Self::RECORD_NAME, #field_name
                    )));
                }
            },
            FieldInfo::String { ident, .. } => quote! {
                #field_name => match new {
                    #value::String(s) => rec.#ident = s.clone(),
                    _ => return Err(#wrong_type(
                        Self::RECORD_NAME, #field_name, "string", new,
                    )),
                },
            },
            FieldInfo::Primitive { ident, ty } => primitive_arm(&field_name, ident, ty, config),
            FieldInfo::InventoryItem { ident, .. } => quote! {
                #field_name => match new {
                    #value::I64(v) => {
                        rec.#ident = #inventory_item::from(*v as i32);
                    }
                    #value::String(s) => match s.trim().parse::<i32>() {
                        Ok(v) => rec.#ident = #inventory_item::from(v),
                        Err(_) => return Err(#wrong_type(
                            Self::RECORD_NAME, #field_name, "i32", new,
                        )),
                    },
                    _ => return Err(#wrong_type(
                        Self::RECORD_NAME, #field_name, "i32", new,
                    )),
                },
            },
            FieldInfo::EnumFromU8 { ident, enum_ty } => {
                enum_arm(&field_name, ident, enum_ty, "u8", config)
            }
            FieldInfo::EnumFromU32 { ident, enum_ty } => {
                enum_arm(&field_name, ident, enum_ty, "u32", config)
            }
            FieldInfo::EnumFromI16 { ident, enum_ty } => {
                enum_arm(&field_name, ident, enum_ty, "i16", config)
            }
            FieldInfo::EnumFromI32 { ident, enum_ty } => {
                enum_arm(&field_name, ident, enum_ty, "i32", config)
            }
            FieldInfo::EnumFromI32FromU8 { ident, enum_ty } => {
                enum_arm(&field_name, ident, enum_ty, "u8", config)
            }
            FieldInfo::Array { ident, size, ty } => {
                if ty != "u8" {
                    panic!("RecordPatcher only supports array(type = \"u8\")");
                }
                quote! {
                    #field_name => match new {
                        #value::Bytes(b) if b.len() == #size => {
                            let mut arr = [0u8; #size];
                            arr.copy_from_slice(b);
                            rec.#ident = arr;
                        }
                        #value::Bytes(_) => {
                            return Err(#malformed(format!(
                                "{}.{}: expected {} bytes, got {}",
                                Self::RECORD_NAME, #field_name, #size,
                                if let #value::Bytes(b) = new { b.len() } else { 0 }
                            )));
                        }
                        _ => return Err(#wrong_type(
                            Self::RECORD_NAME, #field_name,
                            concat!("bytes(", stringify!(#size), ")"), new,
                        )),
                    },
                }
            }
            FieldInfo::VecU8 { ident, size } => quote! {
                #field_name => match new {
                    #value::Bytes(b) if b.len() == #size => {
                        rec.#ident = b.clone();
                    }
                    #value::Bytes(_) => {
                        return Err(#malformed(format!(
                            "{}.{}: expected {} bytes, got {}",
                            Self::RECORD_NAME, #field_name, #size,
                            if let #value::Bytes(b) = new { b.len() } else { 0 }
                        )));
                    }
                    _ => return Err(#wrong_type(
                        Self::RECORD_NAME, #field_name,
                        concat!("bytes(", stringify!(#size), ")"), new,
                    )),
                },
            },
            FieldInfo::Padding { .. } => quote! {
                #field_name => {
                    return Err(#malformed(format!(
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
        /// Auto-generated patcher for the surrounding struct. Implements the
        /// `RecordPatcher` trait (path supplied by codegen configuration).
        pub struct #patcher_ident;

        impl #patcher_ident {
            #key_consts
            /// Human-readable record name, used in error messages.
            pub const RECORD_NAME: &'static str = #name_str;
        }

        impl #record_patcher_trait for #patcher_ident {
            fn name(&self) -> &'static str {
                Self::RECORD_NAME
            }

            fn apply_field(
                &self,
                bytes: &[u8],
                record_id: u32,
                field: &str,
                new: &#value,
            ) -> #result<Vec<u8>> {
                use #extractor_trait as _;

                let mut cursor = std::io::Cursor::new(bytes);
                let mut records = #name::parse(&mut cursor, bytes.len() as u64)?;

                let idx = record_id as usize;
                if idx >= records.len() {
                    return Err(#out_of_range(
                        Self::RECORD_NAME, record_id, records.len(),
                    ));
                }
                let rec = &mut records[idx];

                match field {
                    #(#field_arms)*
                    other => return Err(#unknown_field(
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
fn primitive_arm(
    field_name: &str,
    ident: &Ident,
    ty: &str,
    config: &CodegenConfig,
) -> TokenStream2 {
    let ty_ident = Ident::new(ty, Span::call_site());
    let expected = ty.to_string();
    let value = config.modding(&["value", "Value"]);
    let wrong_type = config.modding(&["patcher", "wrong_type"]);
    quote! {
        #field_name => match new {
            #value::I64(v) => {
                rec.#ident = (*v) as #ty_ident;
            }
            #value::String(s) => match s.trim().parse::<#ty_ident>() {
                Ok(v) => rec.#ident = v,
                Err(_) => return Err(#wrong_type(
                    Self::RECORD_NAME, #field_name, #expected, new,
                )),
            },
            _ => return Err(#wrong_type(
                Self::RECORD_NAME, #field_name, #expected, new,
            )),
        },
    }
}

/// Generate a `match` arm for an enum field. Accepts `Value::I64` (cast to
/// the wire type, then converted) and `Value::String` (parsed as the wire
/// type). Both routes use the same `WireValue::from_wire` fallback as the
/// parse side in `Extractor`.
fn enum_arm(
    field_name: &str,
    ident: &Ident,
    enum_ty: &str,
    wire_ty: &str,
    config: &CodegenConfig,
) -> TokenStream2 {
    let wire_ident = Ident::new(wire_ty, Span::call_site());
    let wire_value = config.wire_value_trait();
    let from_i64 = crate::wire_enum::from_wire_tokens(
        &wire_value,
        enum_ty,
        wire_ty,
        quote! { (*v) as #wire_ident },
    );
    let from_string =
        crate::wire_enum::from_wire_tokens(&wire_value, enum_ty, wire_ty, quote! { v });
    let expected = format!("i64 (discriminant of {})", enum_ty);
    let value = config.modding(&["value", "Value"]);
    let wrong_type = config.modding(&["patcher", "wrong_type"]);
    quote! {
        #field_name => match new {
            #value::I64(v) => {
                rec.#ident = #from_i64;
            }
            #value::String(s) => match s.trim().parse::<#wire_ident>() {
                Ok(v) => rec.#ident = #from_string,
                Err(_) => return Err(#wrong_type(
                    Self::RECORD_NAME, #field_name, #expected, new,
                )),
            },
            _ => return Err(#wrong_type(
                Self::RECORD_NAME, #field_name, #expected, new,
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generated_patcher_uses_historical_modding_paths() {
        let input: DeriveInput = parse_quote! {
            #[extractor(property_item_size = 8)]
            #[patcher(filename = "Sample.db")]
            struct Sample {
                #[extractor(index)]
                id: i32,
                #[extractor(primitive(type = "i32"))]
                value: i32,
                #[extractor(inventory_item(wire_type = "i32"))]
                item: InventoryItem,
            }
        };
        let tokens = codegen(&input, &CodegenConfig::default()).to_string();
        assert!(
            tokens.contains("impl crate :: modding :: patcher :: RecordPatcher for SamplePatcher")
        );
        assert!(tokens.contains("crate :: modding :: value :: Value :: I64"));
        assert!(
            tokens.contains("crate :: references :: enums :: InventoryItem :: from"),
            "{tokens}"
        );
        assert!(tokens.contains("pub const FILENAME : & 'static str = \"Sample.db\""));
    }

    #[test]
    fn test_generated_patcher_paths_follow_config() {
        let input: DeriveInput = parse_quote! {
            #[extractor(property_item_size = 4)]
            #[patcher(filename = "Sample.db")]
            struct Sample {
                #[extractor(primitive(type = "i32"))]
                value: i32,
            }
        };
        let config = CodegenConfig {
            modding_mod: "renamed_modding".into(),
            references_mod: "renamed_refs".into(),
            ..CodegenConfig::default()
        };
        let tokens = codegen(&input, &config).to_string();
        assert!(tokens.contains("crate :: renamed_modding :: patcher :: RecordPatcher"));
        assert!(tokens.contains("crate :: renamed_modding :: value :: Value :: I64"));
        assert!(tokens.contains("use crate :: renamed_refs :: extractor :: Extractor as _"));
        assert!(!tokens.contains("crate :: modding"));
        assert!(!tokens.contains("crate :: references"));
    }
}
