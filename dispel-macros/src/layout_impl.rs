use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::schema::{CodegenConfig, RecordSchema};

/// Generate `RecordLayout` metadata from the shared record schema.
pub(crate) fn derive_record_layout(input: &DeriveInput) -> TokenStream {
    derive_record_layout_with_config(input, &CodegenConfig::default())
}

/// Same codegen with an explicit path configuration (used by the tests).
pub(crate) fn derive_record_layout_with_config(
    input: &DeriveInput,
    config: &CodegenConfig,
) -> TokenStream {
    let name = &input.ident;
    let schema = RecordSchema::parse(input);
    schema.assert_layout_consistent(&name.to_string());

    let header_size = schema.counter_size as u32;
    let record_size = schema.property_item_size as u32;
    let field_def = config.layout("FieldDef");
    let defs = schema
        .layout_fields()
        .into_iter()
        .map(|(field_name, offset, size, ty)| {
            quote! {
                #field_def {
                    name: #field_name, offset: #offset, size: #size, ty: #ty,
                }
            }
        });

    let record_layout_trait = config.layout("RecordLayout");
    let fixed_layout = config.layout("FixedRecordLayout");

    quote! {
        impl #record_layout_trait for #name {
            const LAYOUT: #fixed_layout = #fixed_layout {
                type_name: stringify!(#name), header_size: #header_size, record_size: #record_size,
                fields: &[#(#defs),*],
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generated_layout_matches_schema_offsets() {
        let input: DeriveInput = parse_quote! {
            #[extractor(property_item_size = 26, counter_size = 0)]
            struct Sample {
                #[extractor(index)]
                id: i32,
                #[extractor(string(encoding = "EUC-KR", size = 24))]
                name: String,
                #[extractor(primitive(type = "i16"))]
                value: i16,
            }
        };
        let tokens = derive_record_layout(&input).to_string();
        assert!(tokens.contains("crate :: references :: layout :: RecordLayout for Sample"));
        assert!(tokens.contains("header_size : 0u32 , record_size : 26u32"));
        assert!(tokens.contains("name : \"name\" , offset : 0u32 , size : 24u32"));
        assert!(tokens.contains("name : \"value\" , offset : 24u32 , size : 2u32"));
    }

    #[test]
    fn test_generated_layout_paths_follow_config() {
        let input: DeriveInput = parse_quote! {
            #[extractor(property_item_size = 4)]
            struct Sample {
                #[extractor(primitive(type = "i32"))]
                value: i32,
            }
        };
        let config = CodegenConfig {
            references_mod: "renamed_refs".into(),
            ..CodegenConfig::default()
        };
        let tokens = derive_record_layout_with_config(&input, &config).to_string();
        assert!(tokens.contains("crate :: renamed_refs :: layout :: RecordLayout for Sample"));
        assert!(tokens.contains("crate :: renamed_refs :: layout :: FieldDef"));
        assert!(!tokens.contains("crate :: references ::"));
    }
}
