use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Ident};

use crate::schema::{CodegenConfig, FieldInfo, RecordSchema, extract_vec_inner};

pub fn expand(input: DeriveInput) -> TokenStream2 {
    codegen(&input, &CodegenConfig::default())
}

/// Generate the `Extractor` impl for `input` using the supplied path config.
/// The proc-macro entry point above always uses the default config; keeping
/// codegen separate lets the generated tokens be tested directly.
pub(crate) fn codegen(input: &DeriveInput, config: &CodegenConfig) -> TokenStream2 {
    let name = &input.ident;
    let schema = RecordSchema::parse(input);
    let counter_size = schema.counter_size;
    let property_item_size = schema.property_item_size;
    let field_infos = &schema.fields;

    let bo = config.byteorder();
    let extractor_trait = config.extractor_trait();
    let read_mapper = config.read_mapper();
    let inventory_item = config.inventory_item_type();

    // Generate parse statements
    let mut parse_stmts: Vec<TokenStream2> = Vec::new();
    let mut struct_field_inits: Vec<TokenStream2> = Vec::new();

    for info in field_infos {
        match info {
            FieldInfo::Id { ident } => {
                struct_field_inits.push(quote! { #ident: i as i32, });
            }
            FieldInfo::Index { ident } => {
                struct_field_inits.push(quote! { #ident: i as i32, });
            }
            FieldInfo::String {
                ident,
                encoding,
                size,
            } => {
                let encoding_tokens = config.encoding_constant(encoding);
                let buf_ident = Ident::new(&format!("{}_buf", ident), Span::call_site());
                let data_len_ident = Ident::new(&format!("{}_data_len", ident), Span::call_site());
                parse_stmts.push(quote! {
                    let mut #buf_ident = [0u8; #size];
                    reader.read_exact(&mut #buf_ident)?;
                    let #data_len_ident = #buf_ident.iter().position(|&b| b == 0).unwrap_or(#size);
                    let (#ident, _, _) = #encoding_tokens.decode(&#buf_ident[..#data_len_ident]);
                    let #ident = #ident.trim().to_string();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::Primitive { ident, ty } => {
                let read_stmt = match ty.as_str() {
                    "i16" => {
                        quote! { #bo::ReadBytesExt::read_i16::<#bo::LittleEndian>(reader)? }
                    }
                    "i32" => {
                        quote! { #bo::ReadBytesExt::read_i32::<#bo::LittleEndian>(reader)? }
                    }
                    "u8" => quote! { #bo::ReadBytesExt::read_u8(reader)? },
                    "u16" => {
                        quote! { #bo::ReadBytesExt::read_u16::<#bo::LittleEndian>(reader)? }
                    }
                    "u32" => {
                        quote! { #bo::ReadBytesExt::read_u32::<#bo::LittleEndian>(reader)? }
                    }
                    _ => panic!("Unsupported primitive type: {}", ty),
                };
                parse_stmts.push(quote! {
                    let #ident = #read_stmt;
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::EnumFromU8 { ident, enum_ty } => {
                let enum_ident = Ident::new(enum_ty, Span::call_site());
                parse_stmts.push(quote! {
                    let #ident = #enum_ident::from_u8(#bo::ReadBytesExt::read_u8(reader)?).unwrap_or_default();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::EnumFromU32 { ident, enum_ty } => {
                let enum_ident = Ident::new(enum_ty, Span::call_site());
                parse_stmts.push(quote! {
                    let #ident = #enum_ident::from_u32(#bo::ReadBytesExt::read_u32::<#bo::LittleEndian>(reader)?).unwrap_or_default();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::EnumFromI16 { ident, enum_ty } => {
                let enum_ident = Ident::new(enum_ty, Span::call_site());
                parse_stmts.push(quote! {
                    let #ident = #enum_ident::from_i16(#bo::ReadBytesExt::read_i16::<#bo::LittleEndian>(reader)?).unwrap_or_default();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::EnumFromI32 { ident, enum_ty } => {
                let enum_ident = Ident::new(enum_ty, Span::call_site());
                parse_stmts.push(quote! {
                    let #ident = #enum_ident::from_i32(#bo::ReadBytesExt::read_i32::<#bo::LittleEndian>(reader)?).unwrap_or_default();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::EnumFromI32FromU8 { ident, enum_ty } => {
                let enum_ident = Ident::new(enum_ty, Span::call_site());
                parse_stmts.push(quote! {
                    let #ident = #enum_ident::from_u8(#bo::ReadBytesExt::read_u8(reader)?).unwrap_or_default();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::VecU8 { ident, size } => {
                parse_stmts.push(quote! {
                    let mut #ident = vec![0u8; #size];
                    reader.read_exact(&mut #ident)?;
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::Nested {
                ident,
                item_ty,
                max_items,
                item_size: _,
            } => {
                let inner_ty = extract_vec_inner(item_ty).unwrap_or_else(|| {
                    panic!(
                        "#[extractor(nested)] requires the field type to be Vec<T>; got `{}`",
                        quote! { #item_ty }
                    )
                });
                let inner_ty_tokens = quote! { #inner_ty };
                parse_stmts.push(quote! {
                    let mut #ident: Vec<#inner_ty_tokens> = Vec::with_capacity(#max_items);
                    for _ in 0..#max_items {
                        match <#inner_ty_tokens as #extractor_trait>::parse_one(reader)? {
                            Some(item) => #ident.push(item),
                            None => break,
                        }
                    }
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::InventoryItem { ident, wire_type } => {
                let read_stmt = match wire_type.as_str() {
                    "u16" => {
                        quote! { #inventory_item::from(#bo::ReadBytesExt::read_u16::<#bo::LittleEndian>(reader)?) }
                    }
                    "i16" => {
                        quote! { #inventory_item::from(#bo::ReadBytesExt::read_i16::<#bo::LittleEndian>(reader)?) }
                    }
                    "i32" => {
                        quote! { #inventory_item::from(#bo::ReadBytesExt::read_i32::<#bo::LittleEndian>(reader)?) }
                    }
                    _ => panic!("Unsupported wire_type for inventory_item: {}", wire_type),
                };
                parse_stmts.push(quote! {
                    let #ident = #read_stmt;
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::Padding {
                ident,
                count,
                ty,
                default_value,
            } => {
                let default_expr = if let Some(dv) = default_value {
                    match ty.as_str() {
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
                        _ => panic!("Unsupported padding type: {}", ty),
                    }
                } else {
                    match ty.as_str() {
                        "i16" => quote! { 0i16 },
                        "i32" => quote! { 0i32 },
                        "u8" => quote! { 0u8 },
                        _ => panic!("Unsupported padding type: {}", ty),
                    }
                };
                for _ in 0..*count {
                    let read_stmt = match ty.as_str() {
                        "i16" => {
                            quote! { #bo::ReadBytesExt::read_i16::<#bo::LittleEndian>(reader)? }
                        }
                        "i32" => {
                            quote! { #bo::ReadBytesExt::read_i32::<#bo::LittleEndian>(reader)? }
                        }
                        "u8" => quote! { #bo::ReadBytesExt::read_u8(reader)? },
                        _ => panic!("Unsupported padding type: {}", ty),
                    };
                    parse_stmts.push(quote! {
                        let _ = #read_stmt;
                    });
                }
                parse_stmts.push(quote! { let #ident = #default_expr; });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::Array { ident, size, ty } => {
                if ty == "u8" {
                    parse_stmts.push(quote! {
                        let mut #ident = [0u8; #size];
                        reader.read_exact(&mut #ident)?;
                    });
                    struct_field_inits.push(quote! { #ident: #ident, });
                }
            }
            FieldInfo::Skip => {}
        }
    }

    // Generate write statements
    let mut write_stmts: Vec<TokenStream2> = Vec::new();

    for info in field_infos {
        match info {
            FieldInfo::Id { ident: _ } => {}
            FieldInfo::Index { ident: _ } => {}
            FieldInfo::String {
                ident,
                encoding,
                size,
            } => {
                let encoding_tokens = config.encoding_constant(encoding);
                let buf_ident = Ident::new(&format!("{}_buf", ident), Span::call_site());
                write_stmts.push(quote! {
                    let mut #buf_ident = vec![0u8; #size];
                    let (cow, _, _) = #encoding_tokens.encode(&record.#ident);
                    let len = std::cmp::min(cow.len(), #size);
                    #buf_ident[..len].copy_from_slice(&cow[..len]);
                    writer.write_all(&#buf_ident)?;
                });
            }
            FieldInfo::Primitive { ident, ty } => {
                let write_stmt = match ty.as_str() {
                    "i16" => {
                        quote! { #bo::WriteBytesExt::write_i16::<#bo::LittleEndian>(writer, record.#ident)?; }
                    }
                    "i32" => {
                        quote! { #bo::WriteBytesExt::write_i32::<#bo::LittleEndian>(writer, record.#ident)?; }
                    }
                    "u8" => quote! { #bo::WriteBytesExt::write_u8(writer, record.#ident)?; },
                    "u16" => {
                        quote! { #bo::WriteBytesExt::write_u16::<#bo::LittleEndian>(writer, record.#ident)?; }
                    }
                    "u32" => {
                        quote! { #bo::WriteBytesExt::write_u32::<#bo::LittleEndian>(writer, record.#ident)?; }
                    }
                    _ => panic!("Unsupported primitive type: {}", ty),
                };
                write_stmts.push(quote! {
                    #write_stmt
                });
            }
            FieldInfo::EnumFromU8 { ident, enum_ty: _ } => {
                write_stmts.push(quote! {
                    #bo::WriteBytesExt::write_u8(writer, u8::from(record.#ident))?;
                });
            }
            FieldInfo::EnumFromU32 { ident, enum_ty: _ } => {
                write_stmts.push(quote! {
                    #bo::WriteBytesExt::write_u32::<#bo::LittleEndian>(writer, u32::from(record.#ident))?;
                });
            }
            FieldInfo::EnumFromI16 { ident, enum_ty: _ } => {
                write_stmts.push(quote! {
                    #bo::WriteBytesExt::write_i16::<#bo::LittleEndian>(writer, i16::from(record.#ident))?;
                });
            }
            FieldInfo::EnumFromI32 { ident, enum_ty: _ } => {
                write_stmts.push(quote! {
                    #bo::WriteBytesExt::write_i32::<#bo::LittleEndian>(writer, i32::from(record.#ident))?;
                });
            }
            FieldInfo::EnumFromI32FromU8 { ident, enum_ty: _ } => {
                write_stmts.push(quote! {
                    #bo::WriteBytesExt::write_u8(writer, u8::from(record.#ident))?;
                });
            }
            FieldInfo::VecU8 { ident, size: _ } => {
                write_stmts.push(quote! {
                    writer.write_all(&record.#ident)?;
                });
            }
            FieldInfo::Nested {
                ident,
                item_ty,
                max_items,
                item_size,
            } => {
                let inner_ty = extract_vec_inner(item_ty).unwrap_or_else(|| {
                    panic!(
                        "#[extractor(nested)] requires the field type to be Vec<T>; got `{}`",
                        quote! { #item_ty }
                    )
                });
                let inner_ty_tokens = quote! { #inner_ty };
                let total_size = *max_items as i32 * *item_size;
                write_stmts.push(quote! {
                    for item in record.#ident.iter() {
                        <#inner_ty_tokens as #extractor_trait>::write_one(item, writer)?;
                    }
                    let written = (record.#ident.len() as i32) * (#item_size as i32);
                    let remaining_slots = ((#total_size as i32 - written) / #item_size as i32) as usize;
                    if remaining_slots > 0 {
                        let pad = vec![0u8; #item_size as usize];
                        for _ in 0..remaining_slots {
                            writer.write_all(&pad)?;
                        }
                    }
                });
            }
            FieldInfo::InventoryItem { ident, wire_type } => {
                let write_stmt = match wire_type.as_str() {
                    "u16" => {
                        quote! { #bo::WriteBytesExt::write_u16::<#bo::LittleEndian>(writer, record.#ident.raw() as u16)?; }
                    }
                    "i16" => {
                        quote! { #bo::WriteBytesExt::write_i16::<#bo::LittleEndian>(writer, record.#ident.raw() as i16)?; }
                    }
                    "i32" => {
                        quote! { #bo::WriteBytesExt::write_i32::<#bo::LittleEndian>(writer, record.#ident.raw())?; }
                    }
                    _ => panic!("Unsupported wire_type for inventory_item: {}", wire_type),
                };
                write_stmts.push(quote! {
                    #write_stmt
                });
            }
            FieldInfo::Padding {
                ident: _,
                count,
                ty,
                default_value: _,
            } => {
                for _ in 0..*count {
                    let write_zero = match ty.as_str() {
                        "i16" => {
                            quote! { #bo::WriteBytesExt::write_i16::<#bo::LittleEndian>(writer, 0)?; }
                        }
                        "i32" => {
                            quote! { #bo::WriteBytesExt::write_i32::<#bo::LittleEndian>(writer, 0)?; }
                        }
                        "u8" => quote! { #bo::WriteBytesExt::write_u8(writer, 0)?; },
                        _ => panic!("Unsupported padding type: {}", ty),
                    };
                    write_stmts.push(quote! {
                        #write_zero
                    });
                }
            }
            FieldInfo::Array {
                ident,
                size: _,
                ty: _,
            } => {
                write_stmts.push(quote! {
                    writer.write_all(&record.#ident)?;
                });
            }
            FieldInfo::Skip => {}
        }
    }

    // Generate the full implementation
    let expanded = quote! {
        impl #extractor_trait for #name {
            const ITEM_SIZE: i32 = #property_item_size;

            fn parse<R: std::io::Read + std::io::Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>> {
                const COUNTER_SIZE: u8 = #counter_size;
                const PROPERTY_ITEM_SIZE: i32 = #property_item_size;

                let elements = #read_mapper(
                    reader, len, COUNTER_SIZE, PROPERTY_ITEM_SIZE,
                )?;

                let mut items: Vec<#name> = Vec::with_capacity(elements as usize);

                for i in 0..elements {
                    #(#parse_stmts)*

                    items.push(#name {
                        #(#struct_field_inits)*
                    });
                }

                Ok(items)
            }

            fn to_writer<W: std::io::Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
                const COUNTER_SIZE: u8 = #counter_size;
                if COUNTER_SIZE > 0 {
                    let elements = records.len() as i32;
                    #bo::WriteBytesExt::write_i32::<#bo::LittleEndian>(writer, elements)?;
                }

                for record in records {
                    #(#write_stmts)*
                }

                Ok(())
            }

            fn parse_one<R: std::io::Read + std::io::Seek>(reader: &mut R) -> std::io::Result<Option<Self>> {
                const ITEM_SIZE: usize = #property_item_size as usize;
                let mut buf = vec![0u8; ITEM_SIZE];
                reader.read_exact(&mut buf)?;
                if buf.iter().all(|&b| b == 0) {
                    return Ok(None);
                }
                let mut cursor = std::io::Cursor::new(buf);
                let mut parsed = Self::parse(&mut cursor, ITEM_SIZE as u64)?;
                Ok(parsed.pop())
            }

            fn write_one<W: std::io::Write>(record: &Self, writer: &mut W) -> std::io::Result<()> {
                Self::to_writer(std::slice::from_ref(record), writer)
            }
        }
    };

    expanded
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn sample_schema() -> DeriveInput {
        parse_quote! {
            #[extractor(property_item_size = 14, counter_size = 0)]
            struct Sample {
                #[extractor(index)]
                id: i32,
                #[extractor(string(encoding = "EUC-KR", size = 4))]
                name: String,
                #[extractor(inventory_item(wire_type = "u16"))]
                item: InventoryItem,
                #[extractor(primitive(type = "i32"))]
                value: i32,
            }
        }
    }

    #[test]
    fn test_generated_extractor_uses_historical_paths_by_default() {
        let tokens = codegen(&sample_schema(), &CodegenConfig::default()).to_string();
        assert!(
            tokens.contains("impl crate :: references :: extractor :: Extractor for Sample"),
            "{tokens}"
        );
        assert!(
            tokens.contains("crate :: references :: extractor :: read_mapper"),
            "{tokens}"
        );
        assert!(
            tokens.contains("crate :: references :: enums :: InventoryItem :: from"),
            "{tokens}"
        );
        assert!(
            tokens.contains(
                "byteorder :: WriteBytesExt :: write_i32 :: < byteorder :: LittleEndian >"
            ),
            "{tokens}"
        );
        assert!(
            tokens.contains("encoding_rs :: EUC_KR . decode"),
            "{tokens}"
        );
    }

    #[test]
    fn test_generated_extractor_paths_follow_config_without_changing_behavior() {
        let input = sample_schema();
        let default_tokens = codegen(&input, &CodegenConfig::default()).to_string();

        let renamed = CodegenConfig {
            references_mod: "renamed_refs".into(),
            inventory_item_ty: "RenamedInventoryItem".into(),
            byteorder_crate: "renamed_bo".into(),
            ..CodegenConfig::default()
        };
        let renamed_tokens = codegen(&input, &renamed).to_string();

        assert!(renamed_tokens.contains("crate :: renamed_refs :: extractor :: Extractor"));
        assert!(renamed_tokens.contains(":: RenamedInventoryItem :: from"));
        assert!(renamed_tokens.contains("renamed_bo :: WriteBytesExt"));

        // Renaming configured paths must be the *only* difference: every
        // byte-affecting parse/write statement stays token-for-token identical.
        let normalized = renamed_tokens
            .replace("crate :: renamed_refs ::", "crate :: references ::")
            .replace(":: RenamedInventoryItem", ":: InventoryItem")
            .replace("renamed_bo ::", "byteorder ::");
        assert_eq!(default_tokens, normalized);
    }
}
