use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

use crate::schema::{CodegenConfig, PatcherKey};
use crate::text_extractor_impl::{TextFieldType, parse_text_extractor_attr};

pub fn expand(input: DeriveInput) -> TokenStream2 {
    codegen(&input, &CodegenConfig::default())
}

/// Generate the text `RecordPatcher` impl with an explicit path config.
pub(crate) fn codegen(input: &DeriveInput, config: &CodegenConfig) -> TokenStream2 {
    let name = &input.ident;
    let patcher_ident = Ident::new(&format!("{name}Patcher"), Span::call_site());
    let name_str = name.to_string();

    let key_consts = match PatcherKey::parse(input, "TextRecordPatcher") {
        PatcherKey::Filename { filename, .. } => quote! {
            pub const FILENAME: &'static str = #filename;
        },
        PatcherKey::Pattern {
            extension,
            stem_prefix,
        } => quote! {
            pub const EXTENSION: &'static str = #extension;
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
    let extractor_trait = config.extractor_trait();

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("TextRecordPatcher only supports structs with named fields"),
        },
        _ => panic!("TextRecordPatcher can only be derived for structs"),
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
        let info = match parse_text_extractor_attr(attr, field_ident, field_ty) {
            Some(i) => i,
            None => continue,
        };

        // Field 0 is the row id by convention; reject patches.
        if info.index == 0 {
            field_arms.push(quote! {
                #field_name => {
                    return Err(#malformed(format!(
                        "{}.{} is positional and cannot be patched",
                        Self::RECORD_NAME, #field_name
                    )));
                }
            });
            continue;
        }

        let arm = match info.ty {
            TextFieldType::String => quote! {
                #field_name => match new {
                    #value::String(s) => rec.#field_ident = s.clone(),
                    _ => return Err(#wrong_type(
                        Self::RECORD_NAME, #field_name, "string", new,
                    )),
                },
            },
            TextFieldType::OptionString => quote! {
                #field_name => match new {
                    #value::Null => rec.#field_ident = None,
                    #value::String(s) => {
                        rec.#field_ident = if s == "null" { None } else { Some(s.clone()) };
                    }
                    _ => return Err(#wrong_type(
                        Self::RECORD_NAME, #field_name, "string|null", new,
                    )),
                },
            },
            TextFieldType::I32 => quote! {
                #field_name => match new {
                    #value::I64(v) => {
                        rec.#field_ident = (*v) as i32;
                    }
                    #value::String(s) => match s.trim().parse::<i32>() {
                        Ok(v) => rec.#field_ident = v,
                        Err(_) => return Err(#wrong_type(
                            Self::RECORD_NAME, #field_name, "i32", new,
                        )),
                    },
                    _ => return Err(#wrong_type(
                        Self::RECORD_NAME, #field_name, "i32", new,
                    )),
                },
            },
            TextFieldType::EnumFromI32(enum_ty) => {
                let wire_value = config.wire_value_trait();
                let from_i64 = crate::wire_enum::from_wire_tokens(
                    &wire_value,
                    &enum_ty,
                    "i32",
                    quote! { (*v) as i32 },
                );
                let from_string =
                    crate::wire_enum::from_wire_tokens(&wire_value, &enum_ty, "i32", quote! { v });
                let expected = format!("i32 (discriminant of {enum_ty})");
                quote! {
                    #field_name => match new {
                        #value::I64(v) => {
                            rec.#field_ident = #from_i64;
                        }
                        #value::String(s) => match s.trim().parse::<i32>() {
                            Ok(v) => rec.#field_ident = #from_string,
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
        };
        field_arms.push(arm);
    }

    let expanded = quote! {
        pub struct #patcher_ident;

        impl #patcher_ident {
            #key_consts
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

                // For text formats `record_id` is the row index in the parsed
                // list, matching how the GUI's table editor enumerates rows.
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

                let mut out = Vec::new();
                #name::to_writer(&records, &mut out)?;
                Ok(out)
            }
        }
    };

    expanded
}
