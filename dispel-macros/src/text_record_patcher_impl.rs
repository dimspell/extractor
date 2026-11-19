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

    // Keep registration in lockstep with the binary `RecordPatcher`: aliases
    // must be exposed as `ALIASES`/`FILENAMES` here too, otherwise a text
    // catalog annotated with `#[patcher(alias = "...")]` would silently lose
    // its alternate filenames.
    let key_consts = match PatcherKey::parse(input, "TextRecordPatcher") {
        PatcherKey::Filename { filename, aliases } => {
            let all_names: Vec<String> = std::iter::once(filename.clone())
                .chain(aliases.iter().cloned())
                .collect();
            quote! {
                /// The relative filename this patcher targets (exact match).
                pub const FILENAME: &'static str = #filename;
                /// Additional filenames sharing this record format.
                pub const ALIASES: &'static [&'static str] = &[#(#aliases),*];
                /// Every filename routed to this patcher (primary + aliases).
                pub const FILENAMES: &'static [&'static str] = &[#(#all_names),*];
            }
        }
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

        // Format-wide convention for derived text catalogs: field 0 is the
        // positional row id (see e.g. `NpcIni::id`, `Map::id`, `Event::event_id`).
        // It identifies the row rather than describing it, so patches that
        // rewrite it would corrupt row lookup and are rejected here.
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

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn sample() -> DeriveInput {
        parse_quote! {
            #[extractor(encoding = "EUC-KR")]
            #[patcher(filename = "Npc.ini")]
            struct NpcIni {
                #[extractor(field = 0)]
                id: i32,
                #[extractor(field = 1, parse_null)]
                sprite: Option<String>,
                #[extractor(field = 2, enum_from_i32(type = "NpcKind"))]
                kind: NpcKind,
            }
        }
    }

    #[test]
    fn test_text_patcher_emits_alias_constants_like_binary_patcher() {
        let input: DeriveInput = parse_quote! {
            #[patcher(filename = "Foo.ini", alias = "Bar.ini", alias = "Baz.ini")]
            struct Foo {
                #[extractor(field = 1)]
                value: String,
            }
        };
        let tokens = codegen(&input, &CodegenConfig::default()).to_string();
        assert!(tokens.contains("pub const FILENAME"), "{tokens}");
        assert!(tokens.contains("pub const ALIASES"), "{tokens}");
        assert!(tokens.contains("pub const FILENAMES"), "{tokens}");
        assert!(tokens.contains("\"Bar.ini\""), "{tokens}");
        assert!(tokens.contains("\"Baz.ini\""), "{tokens}");
        assert!(tokens.contains("\"Foo.ini\""), "{tokens}");

        // A text patcher with no aliases still exposes the empty slice.
        let no_alias = codegen(&sample(), &CodegenConfig::default()).to_string();
        assert!(no_alias.contains("pub const ALIASES"), "{no_alias}");
        assert!(no_alias.contains("& []"), "{no_alias}");
    }

    #[test]
    fn test_text_patcher_rejects_positional_field_zero() {
        let tokens = codegen(&sample(), &CodegenConfig::default()).to_string();
        assert!(
            tokens.contains("is positional and cannot be patched"),
            "{tokens}"
        );
        assert!(tokens.contains("Self :: RECORD_NAME , \"id\""), "{tokens}");
        // Non-positional fields still get real apply arms.
        assert!(tokens.contains("rec . sprite ="), "{tokens}");
        assert!(tokens.contains("rec . kind ="), "{tokens}");
    }

    #[test]
    fn test_text_patcher_pattern_key_emits_extension_and_stem_prefix() {
        let input: DeriveInput = parse_quote! {
            #[patcher(extension = "ref", stem_prefix = "ext")]
            struct ExtraRef {
                #[extractor(field = 1)]
                value: String,
            }
        };
        let tokens = codegen(&input, &CodegenConfig::default()).to_string();
        assert!(tokens.contains("pub const EXTENSION"), "{tokens}");
        assert!(tokens.contains("pub const STEM_PREFIX"), "{tokens}");
        assert!(tokens.contains("\"ref\""), "{tokens}");
        assert!(tokens.contains("\"ext\""), "{tokens}");
    }

    #[test]
    fn test_text_patcher_paths_follow_config_without_changing_behavior() {
        let input = sample();
        let default = codegen(&input, &CodegenConfig::default()).to_string();

        let renamed = CodegenConfig {
            modding_mod: "mods".into(),
            references_mod: "refs".into(),
            ..CodegenConfig::default()
        };
        let renamed_tokens = codegen(&input, &renamed).to_string();
        assert!(renamed_tokens.contains("crate :: mods :: patcher :: RecordPatcher"));
        assert!(renamed_tokens.contains("crate :: refs :: enums :: WireValue"));

        let normalized = renamed_tokens
            .replace("crate :: mods ::", "crate :: modding ::")
            .replace("crate :: refs ::", "crate :: references ::");
        assert_eq!(default, normalized);
    }
}
