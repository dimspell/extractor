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
                        if !entry.is_encodable() {
                            statuses.push(#truncation_status::Unencodable);
                        } else {
                            let (text, was_truncated) = #truncate_to_fit(
                                &entry.translation, &entry.encoding, entry.max_bytes,
                            );
                            self.#field_ident = Some(text);
                            statuses.push(if was_truncated {
                                #truncation_status::Truncated {
                                    original_bytes: entry.encoded_translation_len(),
                                }
                            } else {
                                #truncation_status::Ok
                            });
                        }
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
                    if !entry.translation.is_empty() {
                        if !entry.is_encodable() {
                            statuses.push(#truncation_status::Unencodable);
                        } else {
                            let (text, was_truncated) = #truncate_to_fit(
                                &entry.translation, &entry.encoding, entry.max_bytes,
                            );
                            self.#field_ident = text;
                            statuses.push(if was_truncated {
                                #truncation_status::Truncated {
                                    original_bytes: entry.encoded_translation_len(),
                                }
                            } else {
                                #truncation_status::Ok
                            });
                        }
                    }
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

    // Token generation stays in `codegen`, where `CodegenConfig` resolves the
    // spelling through the shared encoding registry and supplies the module path.
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

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn sample() -> DeriveInput {
        parse_quote! {
            struct Entry {
                #[translatable(encoding = "WINDOWS-1250", max_bytes = 202)]
                name: String,
                #[translatable(encoding = "EUC_KR", max_bytes = 32)]
                note: Option<String>,
                ignored: i32,
            }
        }
    }

    #[test]
    fn test_localizable_emits_extract_and_apply_arms_for_both_field_shapes() {
        let tokens = codegen(&sample(), &CodegenConfig::default()).to_string();
        // Plain String: direct clone on extract, direct assign on apply.
        assert!(
            tokens.contains("original : self . name . clone ()"),
            "{tokens}"
        );
        assert!(tokens.contains("self . name = text"), "{tokens}");
        // Option<String>: guarded extract and Some(..) assign on apply.
        assert!(
            tokens.contains("if let Some (ref text) = self . note"),
            "{tokens}"
        );
        assert!(tokens.contains("self . note = Some (text)"), "{tokens}");
        // Unannotated field is ignored, and the encoding registry is used.
        assert!(!tokens.contains("self . ignored"), "{tokens}");
        assert!(
            tokens.contains("crate :: localization :: TextEncoding :: Windows1250"),
            "{tokens}"
        );
        assert!(
            tokens.contains("crate :: localization :: TextEncoding :: EucKr"),
            "{tokens}"
        );
        assert!(tokens.contains("202usize"), "{tokens}");
    }

    #[test]
    fn test_localizable_paths_follow_config_without_changing_behavior() {
        let input = sample();
        let default = codegen(&input, &CodegenConfig::default()).to_string();

        let renamed = CodegenConfig {
            localization_mod: "i18n".into(),
            ..CodegenConfig::default()
        };
        let renamed_tokens = codegen(&input, &renamed).to_string();
        assert!(renamed_tokens.contains("crate :: i18n :: Localizable"));
        assert!(renamed_tokens.contains("crate :: i18n :: TextEntry"));
        assert!(renamed_tokens.contains("crate :: i18n :: TruncationStatus"));

        let normalized = renamed_tokens.replace("crate :: i18n ::", "crate :: localization ::");
        assert_eq!(default, normalized);
    }
}
