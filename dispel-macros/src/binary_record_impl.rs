use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Ident};

use crate::schema::{
    CodegenConfig, FieldInfo, padding_default_value_expr, parse_binary_record_fields,
};

pub fn expand(input: DeriveInput) -> TokenStream2 {
    codegen(&input, &CodegenConfig::default())
}

/// Generate the fixed-size record helpers with an explicit path config.
///
/// Field metadata comes from the one shared schema parser in `schema.rs`: the
/// codegen below is just another consumer of `FieldInfo`, using the historic
/// method-call byteorder idiom to keep emitted tokens byte-for-byte identical.
pub(crate) fn codegen(input: &DeriveInput, config: &CodegenConfig) -> TokenStream2 {
    let name = &input.ident;
    let bo = config.byteorder();
    let inventory_item = config.inventory_item_type();

    let field_infos = parse_binary_record_fields(input);

    // Compute total record size from the shared per-field size metadata.
    let total_size: usize = field_infos.iter().map(FieldInfo::binary_record_size).sum();

    // Generate parse statements
    let mut parse_stmts: Vec<TokenStream2> = Vec::new();
    let mut struct_field_inits: Vec<TokenStream2> = Vec::new();

    for info in &field_infos {
        match info {
            FieldInfo::String {
                ident,
                encoding,
                size,
            } => {
                let encoding_tokens = config.encoding_constant(encoding);
                let buf_ident = Ident::new(&format!("{ident}_buf"), Span::call_site());
                let data_len_ident = Ident::new(&format!("{ident}_data_len"), Span::call_site());
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
                        quote! { reader.read_i16::<#bo::LittleEndian>()? }
                    }
                    "i32" => {
                        quote! { reader.read_i32::<#bo::LittleEndian>()? }
                    }
                    "i8" => quote! { reader.read_i8()? },
                    "u8" => quote! { reader.read_u8()? },
                    "u16" => {
                        quote! { reader.read_u16::<#bo::LittleEndian>()? }
                    }
                    "u32" => {
                        quote! { reader.read_u32::<#bo::LittleEndian>()? }
                    }
                    _ => panic!("Unsupported primitive type: {}", ty),
                };
                parse_stmts.push(quote! {
                    let #ident = #read_stmt;
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::InventoryItem { ident, wire_type } => {
                let read_stmt = match wire_type.as_str() {
                    "i32" => quote! { reader.read_i32::<#bo::LittleEndian>()? },
                    "i16" => quote! { reader.read_i16::<#bo::LittleEndian>()? as i32 },
                    _ => panic!("Unsupported wire_type for InventoryItem: {}", wire_type),
                };
                parse_stmts.push(quote! {
                    let #ident = #inventory_item::from(#read_stmt);
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
            FieldInfo::Array { ident, size, .. } => {
                parse_stmts.push(quote! {
                    let mut #ident = [0u8; #size];
                    reader.read_exact(&mut #ident)?;
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::Padding {
                ident,
                count,
                ty,
                default_value,
            } => {
                let default_expr = padding_default_value_expr(ty, default_value.as_deref());
                for _ in 0..*count {
                    let read_stmt = match ty.as_str() {
                        "i16" => {
                            quote! { reader.read_i16::<#bo::LittleEndian>()? }
                        }
                        "i32" => {
                            quote! { reader.read_i32::<#bo::LittleEndian>()? }
                        }
                        "u8" => quote! { reader.read_u8()? },
                        _ => panic!("Unsupported padding type: {}", ty),
                    };
                    parse_stmts.push(quote! {
                        let _ = #read_stmt;
                    });
                }
                parse_stmts.push(quote! { let #ident = #default_expr; });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::Skip => {}
            FieldInfo::Id { .. }
            | FieldInfo::Index { .. }
            | FieldInfo::EnumFromU8 { .. }
            | FieldInfo::EnumFromU32 { .. }
            | FieldInfo::EnumFromI16 { .. }
            | FieldInfo::EnumFromI32 { .. }
            | FieldInfo::EnumFromI32FromU8 { .. } => {
                panic!("BinaryRecord does not support this field kind")
            }
        }
    }

    // Generate write statements
    let mut write_stmts: Vec<TokenStream2> = Vec::new();
    for info in &field_infos {
        match info {
            FieldInfo::String {
                ident,
                encoding,
                size,
            } => {
                let encoding_tokens = config.encoding_constant(encoding);
                let buf_ident = Ident::new(&format!("{ident}_buf"), Span::call_site());
                write_stmts.push(quote! {
                    let mut #buf_ident = vec![0u8; #size];
                    let (cow, _, _) = #encoding_tokens.encode(&self.#ident);
                    let len = std::cmp::min(cow.len(), #size);
                    #buf_ident[..len].copy_from_slice(&cow[..len]);
                    writer.write_all(&#buf_ident)?;
                });
            }
            FieldInfo::Primitive { ident, ty } => {
                let write_stmt = match ty.as_str() {
                    "i16" => {
                        quote! { writer.write_i16::<#bo::LittleEndian>(self.#ident)?; }
                    }
                    "i32" => {
                        quote! { writer.write_i32::<#bo::LittleEndian>(self.#ident)?; }
                    }
                    "u8" => quote! { writer.write_u8(self.#ident)?; },
                    "i8" => quote! { writer.write_i8(self.#ident)?; },
                    "u16" => {
                        quote! { writer.write_u16::<#bo::LittleEndian>(self.#ident)?; }
                    }
                    "u32" => {
                        quote! { writer.write_u32::<#bo::LittleEndian>(self.#ident)?; }
                    }
                    _ => panic!("Unsupported primitive type: {}", ty),
                };
                write_stmts.push(quote! {
                    #write_stmt
                });
            }
            FieldInfo::InventoryItem { ident, wire_type } => {
                let write_stmt = match wire_type.as_str() {
                    "i32" => quote! {
                        writer.write_i32::<#bo::LittleEndian>(self.#ident.raw())?;
                    },
                    "i16" => quote! {
                        writer.write_i16::<#bo::LittleEndian>(self.#ident.raw() as i16)?;
                    },
                    _ => panic!("Unsupported wire_type for InventoryItem: {}", wire_type),
                };
                write_stmts.push(quote! { #write_stmt });
            }
            FieldInfo::VecU8 { ident, size: _ } => {
                write_stmts.push(quote! {
                    writer.write_all(&self.#ident)?;
                });
            }
            FieldInfo::Array {
                ident,
                size: _,
                ty: _,
            } => {
                write_stmts.push(quote! {
                    writer.write_all(&self.#ident)?;
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
                            quote! { writer.write_i16::<#bo::LittleEndian>(0)?; }
                        }
                        "i32" => {
                            quote! { writer.write_i32::<#bo::LittleEndian>(0)?; }
                        }
                        "u8" => quote! { writer.write_u8(0)?; },
                        _ => panic!("Unsupported padding type: {}", ty),
                    };
                    write_stmts.push(quote! {
                        #write_zero
                    });
                }
            }
            FieldInfo::Skip => {}
            FieldInfo::Id { .. }
            | FieldInfo::Index { .. }
            | FieldInfo::EnumFromU8 { .. }
            | FieldInfo::EnumFromU32 { .. }
            | FieldInfo::EnumFromI16 { .. }
            | FieldInfo::EnumFromI32 { .. }
            | FieldInfo::EnumFromI32FromU8 { .. } => {
                panic!("BinaryRecord does not support this field kind")
            }
        }
    }

    let record_size_name = Ident::new(
        &format!("{}_RECORD_SIZE", name.to_string().to_ascii_uppercase()),
        Span::call_site(),
    );

    let expanded = quote! {
        const #record_size_name: usize = #total_size;

        impl #name {
            /// Parse a single record from raw bytes.
            pub fn parse(data: &[u8]) -> std::io::Result<Self> {
                use #bo::ReadBytesExt;
                use std::io::Read;

                if data.len() != #total_size {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        concat!(stringify!(#name), " requires ", stringify!(#total_size), " bytes"),
                    ));
                }

                let mut reader = std::io::Cursor::new(data);

                #(#parse_stmts)*

                Ok(#name {
                    #(#struct_field_inits)*
                })
            }

            /// Write this record to a writer.
            pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                use #bo::WriteBytesExt;
                use std::io::Write;

                #(#write_stmts)*

                Ok(())
            }

            /// The fixed byte size of this record.
            pub const fn record_size() -> usize {
                #total_size
            }
        }
    };

    expanded
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    /// A fixture exercising every `BinaryRecord` grammar form at once:
    /// annotated string / inventory / vec / padding / skip and auto-detected
    /// primitives, arrays and default-wire-type inventory items.
    fn golden_input() -> DeriveInput {
        parse_quote! {
            struct Golden {
                #[binary_record(string(encoding = "WINDOWS-1250", size = 8))]
                name: String,
                hp: i16,
                level: u16,
                flag: i8,
                #[binary_record(inventory_item(wire_type = "i16"))]
                weapon: InventoryItem,
                auto_item: InventoryItem,
                #[binary_record(size = 4)]
                raw: Vec<u8>,
                fixed: [u8; 6],
                #[binary_record(padding(count = 2, type = "i16", default_value = "7"))]
                pad: i16,
                #[binary_record(padding(count = 3, type = "u8"))]
                zero: u8,
                #[binary_record(skip)]
                ignored: u32,
            }
        }
    }

    const GOLDEN_TOKENS: &str = r##"const GOLDEN_RECORD_SIZE : usize = 36usize ; impl Golden { # [doc = r" Parse a single record from raw bytes."] pub fn parse (data : & [u8]) -> std :: io :: Result < Self > { use byteorder :: ReadBytesExt ; use std :: io :: Read ; if data . len () != 36usize { return Err (std :: io :: Error :: new (std :: io :: ErrorKind :: InvalidData , concat ! (stringify ! (Golden) , " requires " , stringify ! (36usize) , " bytes") ,)) ; } let mut reader = std :: io :: Cursor :: new (data) ; let mut name_buf = [0u8 ; 8usize] ; reader . read_exact (& mut name_buf) ? ; let name_data_len = name_buf . iter () . position (| & b | b == 0) . unwrap_or (8usize) ; let (name , _ , _) = encoding_rs :: WINDOWS_1250 . decode (& name_buf [.. name_data_len]) ; let name = name . trim () . to_string () ; let hp = reader . read_i16 :: < byteorder :: LittleEndian > () ? ; let level = reader . read_u16 :: < byteorder :: LittleEndian > () ? ; let flag = reader . read_i8 () ? ; let weapon = crate :: references :: enums :: InventoryItem :: from (reader . read_i16 :: < byteorder :: LittleEndian > () ? as i32) ; let auto_item = crate :: references :: enums :: InventoryItem :: from (reader . read_i32 :: < byteorder :: LittleEndian > () ?) ; let mut raw = vec ! [0u8 ; 4usize] ; reader . read_exact (& mut raw) ? ; let mut fixed = [0u8 ; 6usize] ; reader . read_exact (& mut fixed) ? ; let _ = reader . read_i16 :: < byteorder :: LittleEndian > () ? ; let _ = reader . read_i16 :: < byteorder :: LittleEndian > () ? ; let pad = 7i16 ; let _ = reader . read_u8 () ? ; let _ = reader . read_u8 () ? ; let _ = reader . read_u8 () ? ; let zero = 0u8 ; Ok (Golden { name : name , hp : hp , level : level , flag : flag , weapon : weapon , auto_item : auto_item , raw : raw , fixed : fixed , pad : pad , zero : zero , }) } # [doc = r" Write this record to a writer."] pub fn write < W : std :: io :: Write > (& self , writer : & mut W) -> std :: io :: Result < () > { use byteorder :: WriteBytesExt ; use std :: io :: Write ; let mut name_buf = vec ! [0u8 ; 8usize] ; let (cow , _ , _) = encoding_rs :: WINDOWS_1250 . encode (& self . name) ; let len = std :: cmp :: min (cow . len () , 8usize) ; name_buf [.. len] . copy_from_slice (& cow [.. len]) ; writer . write_all (& name_buf) ? ; writer . write_i16 :: < byteorder :: LittleEndian > (self . hp) ? ; writer . write_u16 :: < byteorder :: LittleEndian > (self . level) ? ; writer . write_i8 (self . flag) ? ; writer . write_i16 :: < byteorder :: LittleEndian > (self . weapon . raw () as i16) ? ; writer . write_i32 :: < byteorder :: LittleEndian > (self . auto_item . raw ()) ? ; writer . write_all (& self . raw) ? ; writer . write_all (& self . fixed) ? ; writer . write_i16 :: < byteorder :: LittleEndian > (0) ? ; writer . write_i16 :: < byteorder :: LittleEndian > (0) ? ; writer . write_u8 (0) ? ; writer . write_u8 (0) ? ; writer . write_u8 (0) ? ; Ok (()) } # [doc = r" The fixed byte size of this record."] pub const fn record_size () -> usize { 36usize } }"##;

    #[test]
    fn test_binary_record_generated_tokens_are_unchanged() {
        let tokens = codegen(&golden_input(), &CodegenConfig::default()).to_string();
        assert_eq!(tokens, GOLDEN_TOKENS);
    }

    #[test]
    fn test_binary_record_paths_follow_config_without_changing_behavior() {
        let input = golden_input();
        let default_tokens = codegen(&input, &CodegenConfig::default()).to_string();

        let renamed = CodegenConfig {
            inventory_item_ty: "RenamedInventoryItem".into(),
            byteorder_crate: "renamed_bo".into(),
            encoding_rs_crate: "renamed_encoding_rs".into(),
            ..CodegenConfig::default()
        };
        let renamed_tokens = codegen(&input, &renamed).to_string();

        assert!(renamed_tokens.contains("renamed_bo :: ReadBytesExt"));
        assert!(renamed_tokens.contains(":: RenamedInventoryItem :: from"));
        assert!(renamed_tokens.contains("renamed_encoding_rs :: WINDOWS_1250 . decode"));

        // Renaming configured paths must be the *only* difference.
        let normalized = renamed_tokens
            .replace(":: RenamedInventoryItem", ":: InventoryItem")
            .replace("renamed_bo ::", "byteorder ::")
            .replace("renamed_encoding_rs ::", "encoding_rs ::");
        assert_eq!(default_tokens, normalized);
    }

    #[test]
    fn test_binary_record_auto_detected_inventory_uses_i32_wire_type() {
        // The i32 wire type is the real default: it is what auto-detection
        // supplies for an unannotated `InventoryItem` field.
        let input: DeriveInput = parse_quote! {
            struct Defaults {
                item: InventoryItem,
            }
        };
        let tokens = codegen(&input, &CodegenConfig::default()).to_string();
        assert!(tokens.contains("const DEFAULTS_RECORD_SIZE : usize = 4usize"));
        assert!(tokens.contains(
            "InventoryItem :: from (reader . read_i32 :: < byteorder :: LittleEndian > () ?)"
        ));
    }

    #[test]
    fn test_binary_record_encoding_spellings_share_one_registry() {
        for spelling in ["WINDOWS-1250", "WINDOWS_1250", "windows_1250"] {
            let input: DeriveInput = parse_quote! {
                struct Enc {
                    #[binary_record(string(encoding = #spelling, size = 4))]
                    text: String,
                }
            };
            let tokens = codegen(&input, &CodegenConfig::default()).to_string();
            assert!(
                tokens.contains("encoding_rs :: WINDOWS_1250 . decode"),
                "{tokens}"
            );
        }
    }

    #[test]
    fn test_binary_record_auto_detects_fixed_array_and_primitives() {
        let input: DeriveInput = parse_quote! {
            struct Auto {
                bytes: [u8; 12],
                value: u32,
            }
        };
        let tokens = codegen(&input, &CodegenConfig::default()).to_string();
        assert!(
            tokens.contains("const AUTO_RECORD_SIZE : usize = 16usize"),
            "{tokens}"
        );
        assert!(
            tokens.contains("let mut bytes = [0u8 ; 12usize]"),
            "{tokens}"
        );
        assert!(
            tokens.contains("let value = reader . read_u32 :: < byteorder :: LittleEndian > () ?"),
            "{tokens}"
        );
    }

    #[test]
    #[should_panic(expected = "has no #[binary_record(string(...))] annotation")]
    fn test_binary_record_rejects_unannotated_string() {
        let input: DeriveInput = parse_quote! {
            struct Bad {
                text: String,
            }
        };
        codegen(&input, &CodegenConfig::default());
    }

    #[test]
    #[should_panic(expected = "has no #[binary_record(size = N)] annotation")]
    fn test_binary_record_rejects_unannotated_vec_u8() {
        let input: DeriveInput = parse_quote! {
            struct Bad {
                data: Vec<u8>,
            }
        };
        codegen(&input, &CodegenConfig::default());
    }
}
