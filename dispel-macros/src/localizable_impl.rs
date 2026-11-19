use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitInt, LitStr, Type};

use crate::schema::CodegenConfig;

pub fn expand(input: DeriveInput) -> TokenStream2 {
    codegen(&input, &CodegenConfig::default())
}

/// Generate the `Localizable` impl with an explicit path config.
pub(crate) fn codegen(input: &DeriveInput, config: &CodegenConfig) -> TokenStream2 {
    let name = &input.ident;
    let localizable_trait = config.localization(&["Localizable"]);
    let text_entry = config.localization(&["TextEntry"]);
    let truncate_to_fit = config.localization(&["truncate_to_fit"]);
    let truncation_status = config.localization(&["TruncationStatus"]);

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("Localizable only supports structs with named fields"),
        },
        _ => panic!("Localizable can only be derived for structs"),
    };

    let mut extract_arms: Vec<TokenStream2> = Vec::new();
    let mut apply_arms: Vec<TokenStream2> = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_name_str = field_ident.to_string();

        let translatable_attr = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident("translatable"));

        let Some(attr) = translatable_attr else {
            continue;
        };

        let (encoding, max_bytes) = parse_translatable_args(attr);
        let encoding_tokens = config.text_encoding(&encoding);
        let option_field = is_option_string(&field.ty);

        if option_field {
            extract_arms.push(quote! {
                if let Some(ref text) = self.#field_ident {
                    entries.push(#text_entry {
                        file_path: file_path.to_owned(),
                        record_id,
                        field_name: #field_name_str,
                        original: text.clone(),
                        translation: String::new(),
                        encoding: #encoding_tokens,
                        max_bytes: #max_bytes,
                    });
                }
            });
            apply_arms.push(quote! {
                #field_name_str => {
                    if !entry.translation.is_empty() {
                        let (text, was_truncated) = #truncate_to_fit(
                            &entry.translation, &entry.encoding, entry.max_bytes,
                        );
                        self.#field_ident = Some(text);
                        statuses.push(if was_truncated {
                            #truncation_status::Truncated {
                                original_bytes: entry.translation.len(),
                            }
                        } else {
                            #truncation_status::Ok
                        });
                    }
                }
            });
        } else {
            extract_arms.push(quote! {
                entries.push(#text_entry {
                    file_path: file_path.to_owned(),
                    record_id,
                    field_name: #field_name_str,
                    original: self.#field_ident.clone(),
                    translation: String::new(),
                    encoding: #encoding_tokens,
                    max_bytes: #max_bytes,
                });
            });
            apply_arms.push(quote! {
                #field_name_str => {
                    let (text, was_truncated) = #truncate_to_fit(
                        &entry.translation, &entry.encoding, entry.max_bytes,
                    );
                    self.#field_ident = text;
                    statuses.push(if was_truncated {
                        #truncation_status::Truncated {
                            original_bytes: entry.translation.len(),
                        }
                    } else {
                        #truncation_status::Ok
                    });
                }
            });
        }
    }

    quote! {
        impl #localizable_trait for #name {
            fn extract_texts(&self, record_id: usize, file_path: &str) -> Vec<#text_entry> {
                let mut entries = Vec::new();
                #(#extract_arms)*
                entries
            }

            fn apply_texts(&mut self, entries: &[#text_entry]) -> Vec<#truncation_status> {
                let mut statuses = Vec::new();
                for entry in entries {
                    match entry.field_name {
                        #(#apply_arms)*
                        _ => {}
                    }
                }
                statuses
            }
        }
    }
}

fn parse_translatable_args(attr: &syn::Attribute) -> (String, usize) {
    let mut encoding: Option<String> = None;
    let mut max_bytes: Option<usize> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("encoding") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            encoding = Some(lit.value());
        } else if meta.path.is_ident("max_bytes") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            max_bytes = Some(
                lit.base10_parse::<usize>()
                    .expect("max_bytes must be usize"),
            );
        }
        Ok(())
    })
    .expect("Failed to parse #[translatable(...)] arguments");

    let encoding = encoding.expect("#[translatable] requires encoding = \"...\"");
    let max_bytes = max_bytes.expect("#[translatable] requires max_bytes = N");

    (encoding, max_bytes)
}

fn is_option_string(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}
