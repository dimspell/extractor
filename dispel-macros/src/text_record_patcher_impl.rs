use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, LitStr};

use crate::text_extractor_impl::{parse_text_extractor_attr, TextFieldType};

pub fn expand(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let patcher_ident = Ident::new(&format!("{name}Patcher"), Span::call_site());
    let name_str = name.to_string();

    // Same `#[patcher(...)]` parsing as the binary derive — reused verbatim
    // because the registration shape is identical.
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
            pub const FILENAME: &'static str = #f;
        },
        (None, Some(ext), Some(prefix)) => quote! {
            pub const EXTENSION: &'static str = #ext;
            pub const STEM_PREFIX: &'static str = #prefix;
        },
        _ => panic!(
            "TextRecordPatcher requires either #[patcher(filename = \"...\")] or \
             #[patcher(extension = \"...\", stem_prefix = \"...\")] on the struct"
        ),
    };

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
                    return Err(crate::modding::error::ModdingError::Malformed(format!(
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
                    crate::modding::value::Value::String(s) => rec.#field_ident = s.clone(),
                    _ => return Err(crate::modding::patcher::wrong_type(
                        Self::RECORD_NAME, #field_name, "string", new,
                    )),
                },
            },
            TextFieldType::OptionString => quote! {
                #field_name => match new {
                    crate::modding::value::Value::Null => rec.#field_ident = None,
                    crate::modding::value::Value::String(s) => {
                        rec.#field_ident = if s == "null" { None } else { Some(s.clone()) };
                    }
                    _ => return Err(crate::modding::patcher::wrong_type(
                        Self::RECORD_NAME, #field_name, "string|null", new,
                    )),
                },
            },
            TextFieldType::I32 => quote! {
                #field_name => match new {
                    crate::modding::value::Value::I64(v) => {
                        rec.#field_ident = (*v) as i32;
                    }
                    crate::modding::value::Value::String(s) => match s.trim().parse::<i32>() {
                        Ok(v) => rec.#field_ident = v,
                        Err(_) => return Err(crate::modding::patcher::wrong_type(
                            Self::RECORD_NAME, #field_name, "i32", new,
                        )),
                    },
                    _ => return Err(crate::modding::patcher::wrong_type(
                        Self::RECORD_NAME, #field_name, "i32", new,
                    )),
                },
            },
            TextFieldType::EnumFromI32(enum_ty) => {
                let enum_ident = Ident::new(&enum_ty, Span::call_site());
                let expected = format!("i32 (discriminant of {enum_ty})");
                quote! {
                    #field_name => match new {
                        crate::modding::value::Value::I64(v) => {
                            rec.#field_ident = #enum_ident::from_i32((*v) as i32).unwrap_or_default();
                        }
                        crate::modding::value::Value::String(s) => match s.trim().parse::<i32>() {
                            Ok(v) => rec.#field_ident = #enum_ident::from_i32(v).unwrap_or_default(),
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
        };
        field_arms.push(arm);
    }

    let expanded = quote! {
        pub struct #patcher_ident;

        impl #patcher_ident {
            #key_consts
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

                // For text formats `record_id` is the row index in the parsed
                // list, matching how the GUI's table editor enumerates rows.
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

                let mut out = Vec::new();
                #name::to_writer(&records, &mut out)?;
                Ok(out)
            }
        }
    };

    expanded
}
