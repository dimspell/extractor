use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, LitInt, LitStr, Type};

// ============================================================================
// Localizable Derive Macro
// ============================================================================

/// Derive macro that generates a `Localizable` impl for a struct.
///
/// Fields annotated with `#[translatable(encoding = "...", max_bytes = N)]` are included
/// in extraction and application. Other fields are ignored.
///
/// Supported `encoding` values: `"WINDOWS-1250"`, `"EUC-KR"`, `"UTF-8"`.
/// Both `String` and `Option<String>` field types are supported.
#[proc_macro_derive(Localizable, attributes(translatable))]
pub fn derive_localizable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

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

        let (encoding_tokens, max_bytes) = parse_translatable_args(attr);
        let option_field = is_option_string(&field.ty);

        if option_field {
            extract_arms.push(quote! {
                if let Some(ref text) = self.#field_ident {
                    entries.push(crate::localization::TextEntry {
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
                        let (text, was_truncated) = crate::localization::truncate_to_fit(
                            &entry.translation, &entry.encoding, entry.max_bytes,
                        );
                        self.#field_ident = Some(text);
                        statuses.push(if was_truncated {
                            crate::localization::TruncationStatus::Truncated {
                                original_bytes: entry.translation.len(),
                            }
                        } else {
                            crate::localization::TruncationStatus::Ok
                        });
                    }
                }
            });
        } else {
            extract_arms.push(quote! {
                entries.push(crate::localization::TextEntry {
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
                    let (text, was_truncated) = crate::localization::truncate_to_fit(
                        &entry.translation, &entry.encoding, entry.max_bytes,
                    );
                    self.#field_ident = text;
                    statuses.push(if was_truncated {
                        crate::localization::TruncationStatus::Truncated {
                            original_bytes: entry.translation.len(),
                        }
                    } else {
                        crate::localization::TruncationStatus::Ok
                    });
                }
            });
        }
    }

    quote! {
        impl crate::localization::Localizable for #name {
            fn extract_texts(&self, record_id: usize, file_path: &str) -> Vec<crate::localization::TextEntry> {
                let mut entries = Vec::new();
                #(#extract_arms)*
                entries
            }

            fn apply_texts(&mut self, entries: &[crate::localization::TextEntry]) -> Vec<crate::localization::TruncationStatus> {
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
    .into()
}

fn parse_translatable_args(attr: &syn::Attribute) -> (TokenStream2, usize) {
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

    let encoding_tokens = match encoding.as_str() {
        "WINDOWS-1250" | "WINDOWS_1250" => {
            quote! { crate::localization::TextEncoding::Windows1250 }
        }
        "EUC-KR" | "EUC_KR" => quote! { crate::localization::TextEncoding::EucKr },
        "UTF-8" | "UTF_8" => quote! { crate::localization::TextEncoding::Utf8 },
        other => panic!(
            "Unknown encoding '{}' in #[translatable]; expected WINDOWS-1250, EUC-KR, or UTF-8",
            other
        ),
    };

    (encoding_tokens, max_bytes)
}

fn is_option_string(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

// ============================================================================
// Extractor Derive Macro
// ============================================================================

/// Derive macro that generates an `Extractor` impl for a struct.
///
/// Fields can be annotated with `#[extractor(...)]` to specify how they should
/// be parsed and written:
///
/// - `#[extractor(id)]` - Record ID field (auto-incremented during parsing)
/// - `#[extractor(string(encoding = "...", size = N))]` - Fixed-size string field
/// - `#[extractor(primitive(type = "i16|i32|u8|u16|u32"))]` - Primitive numeric field
/// - `#[extractor(enum_from_u8(type = "EnumType"))]` - u8-based enum field
/// - `#[extractor(enum_from_i16(type = "EnumType"))]` - i16-based enum field
/// - `#[extractor(enum_from_i32(type = "EnumType"))]` - i32-based enum field
/// - `#[extractor(padding(count = N, type = "i16|i32|u8"))]` - Padding field(s)
/// - `#[extractor(array(size = N, type = "u8"))]` - Fixed-size array
/// - `#[extractor(skip)]` - Skip field during parsing/writing
/// - `#[extractor(counter_size = N)]` - Set COUNTER_SIZE (default 4)
/// - `#[extractor(property_item_size = N)]` - Set PROPERTY_ITEM_SIZE
///
/// Supported encoding values: "WINDOWS-1250", "EUC-KR", "UTF-8"
#[proc_macro_derive(Extractor, attributes(extractor))]
pub fn derive_extractor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("Extractor only supports structs with named fields"),
        },
        _ => panic!("Extractor can only be derived for structs"),
    };

    // First, check struct-level attributes for counter_size and property_item_size
    let mut counter_size = quote! { 4u8 };
    let mut property_item_size = None;

    for attr in &input.attrs {
        if attr.path().is_ident("extractor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("counter_size") {
                    let value = meta.value()?;
                    let lit: LitInt = value.parse()?;
                    let val = lit.base10_parse::<u8>().expect("counter_size must be u8");
                    counter_size = quote! { #val };
                } else if meta.path.is_ident("property_item_size") {
                    let value = meta.value()?;
                    let lit: LitInt = value.parse()?;
                    let val = lit
                        .base10_parse::<i32>()
                        .expect("property_item_size must be i32");
                    property_item_size = Some(quote! { #val });
                }
                Ok(())
            })
            .expect("Failed to parse struct-level #[extractor(...)]");
        }
    }

    // Parse all field attributes
    let mut field_infos: Vec<FieldInfo> = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        // Find #[extractor(...)] attribute
        let extractor_attr = field.attrs.iter().find(|a| a.path().is_ident("extractor"));

        let Some(attr) = extractor_attr else {
            // No attribute - skip this field
            continue;
        };

        let (field_info, new_counter_size, new_property_item_size) =
            parse_extractor_attr(attr, field_ident, field_ty);

        if let Some(cs) = new_counter_size {
            counter_size = cs;
        }
        if let Some(pis) = new_property_item_size {
            property_item_size = Some(pis);
        }

        if let Some(info) = field_info {
            field_infos.push(info);
        }
    }

    let property_item_size = property_item_size
        .expect("Extractor requires #[extractor(property_item_size = N)] on some field or the struct itself");

    // Generate parse statements
    let mut parse_stmts: Vec<TokenStream2> = Vec::new();
    let mut struct_field_inits: Vec<TokenStream2> = Vec::new();

    for info in &field_infos {
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
                let encoding_tokens = get_encoding_tokens(encoding);
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
                        quote! { byteorder::ReadBytesExt::read_i16::<byteorder::LittleEndian>(reader)? }
                    }
                    "i32" => {
                        quote! { byteorder::ReadBytesExt::read_i32::<byteorder::LittleEndian>(reader)? }
                    }
                    "u8" => quote! { byteorder::ReadBytesExt::read_u8(reader)? },
                    "u16" => {
                        quote! { byteorder::ReadBytesExt::read_u16::<byteorder::LittleEndian>(reader)? }
                    }
                    "u32" => {
                        quote! { byteorder::ReadBytesExt::read_u32::<byteorder::LittleEndian>(reader)? }
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
                    let #ident = #enum_ident::from_u8(byteorder::ReadBytesExt::read_u8(reader)?).unwrap_or_default();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::EnumFromU32 { ident, enum_ty } => {
                let enum_ident = Ident::new(enum_ty, Span::call_site());
                parse_stmts.push(quote! {
                    let #ident = #enum_ident::from_u32(byteorder::ReadBytesExt::read_u32::<byteorder::LittleEndian>(reader)?).unwrap_or_default();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::EnumFromI16 { ident, enum_ty } => {
                let enum_ident = Ident::new(enum_ty, Span::call_site());
                parse_stmts.push(quote! {
                    let #ident = #enum_ident::from_i16(byteorder::ReadBytesExt::read_i16::<byteorder::LittleEndian>(reader)?).unwrap_or_default();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::EnumFromI32 { ident, enum_ty } => {
                let enum_ident = Ident::new(enum_ty, Span::call_site());
                parse_stmts.push(quote! {
                    let #ident = #enum_ident::from_i32(byteorder::ReadBytesExt::read_i32::<byteorder::LittleEndian>(reader)?).unwrap_or_default();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            FieldInfo::EnumFromI32FromU8 { ident, enum_ty } => {
                let enum_ident = Ident::new(enum_ty, Span::call_site());
                parse_stmts.push(quote! {
                    let #ident = #enum_ident::from_u8(byteorder::ReadBytesExt::read_u8(reader)?).unwrap_or_default();
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
            FieldInfo::Padding {
                ident,
                count,
                ty,
                default_value,
            } => {
                let default_expr = if let Some(ref dv) = default_value {
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
                            quote! { byteorder::ReadBytesExt::read_i16::<byteorder::LittleEndian>(reader)? }
                        }
                        "i32" => {
                            quote! { byteorder::ReadBytesExt::read_i32::<byteorder::LittleEndian>(reader)? }
                        }
                        "u8" => quote! { byteorder::ReadBytesExt::read_u8(reader)? },
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

    for info in &field_infos {
        match info {
            FieldInfo::Id { ident: _ } => {
                // ID is not written separately
            }
            FieldInfo::Index { ident: _ } => {
                // Index is not written separately
            }
            FieldInfo::String {
                ident,
                encoding,
                size,
            } => {
                let encoding_tokens = get_encoding_tokens(encoding);
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
                        quote! { byteorder::WriteBytesExt::write_i16::<byteorder::LittleEndian>(writer, record.#ident)?; }
                    }
                    "i32" => {
                        quote! { byteorder::WriteBytesExt::write_i32::<byteorder::LittleEndian>(writer, record.#ident)?; }
                    }
                    "u8" => quote! { byteorder::WriteBytesExt::write_u8(writer, record.#ident)?; },
                    "u16" => {
                        quote! { byteorder::WriteBytesExt::write_u16::<byteorder::LittleEndian>(writer, record.#ident)?; }
                    }
                    "u32" => {
                        quote! { byteorder::WriteBytesExt::write_u32::<byteorder::LittleEndian>(writer, record.#ident)?; }
                    }
                    _ => panic!("Unsupported primitive type: {}", ty),
                };
                write_stmts.push(quote! {
                    #write_stmt
                });
            }
            FieldInfo::EnumFromU8 { ident, enum_ty: _ } => {
                write_stmts.push(quote! {
                    byteorder::WriteBytesExt::write_u8(writer, u8::from(record.#ident))?;
                });
            }
            FieldInfo::EnumFromU32 { ident, enum_ty: _ } => {
                write_stmts.push(quote! {
                    byteorder::WriteBytesExt::write_u32::<byteorder::LittleEndian>(writer, u32::from(record.#ident))?;
                });
            }
            FieldInfo::EnumFromI16 { ident, enum_ty: _ } => {
                write_stmts.push(quote! {
                    byteorder::WriteBytesExt::write_i16::<byteorder::LittleEndian>(writer, i16::from(record.#ident))?;
                });
            }
            FieldInfo::EnumFromI32 { ident, enum_ty: _ } => {
                write_stmts.push(quote! {
                    byteorder::WriteBytesExt::write_i32::<byteorder::LittleEndian>(writer, i32::from(record.#ident))?;
                });
            }
            FieldInfo::EnumFromI32FromU8 { ident, enum_ty: _ } => {
                write_stmts.push(quote! {
                    byteorder::WriteBytesExt::write_u8(writer, u8::from(record.#ident))?;
                });
            }
            FieldInfo::VecU8 { ident, size: _ } => {
                write_stmts.push(quote! {
                    writer.write_all(&record.#ident)?;
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
                            quote! { byteorder::WriteBytesExt::write_i16::<byteorder::LittleEndian>(writer, 0)?; }
                        }
                        "i32" => {
                            quote! { byteorder::WriteBytesExt::write_i32::<byteorder::LittleEndian>(writer, 0)?; }
                        }
                        "u8" => quote! { byteorder::WriteBytesExt::write_u8(writer, 0)?; },
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
        impl crate::references::extractor::Extractor for #name {
            fn parse<R: std::io::Read + std::io::Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>> {
                const COUNTER_SIZE: u8 = #counter_size;
                const PROPERTY_ITEM_SIZE: i32 = #property_item_size;

                let elements = crate::references::extractor::read_mapper(
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
                    byteorder::WriteBytesExt::write_i32::<byteorder::LittleEndian>(writer, elements)?;
                }

                for record in records {
                    #(#write_stmts)*
                }

                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================================
// Field Info and Parsing
// ============================================================================

enum FieldInfo<'a> {
    Id {
        ident: &'a Ident,
    },
    Index {
        ident: &'a Ident,
    },
    String {
        ident: &'a Ident,
        encoding: String,
        size: usize,
    },
    Primitive {
        ident: &'a Ident,
        ty: String,
    },
    EnumFromU8 {
        ident: &'a Ident,
        enum_ty: String,
    },
    EnumFromU32 {
        ident: &'a Ident,
        enum_ty: String,
    },
    EnumFromI16 {
        ident: &'a Ident,
        enum_ty: String,
    },
    EnumFromI32 {
        ident: &'a Ident,
        enum_ty: String,
    },
    EnumFromI32FromU8 {
        ident: &'a Ident,
        enum_ty: String,
    },
    Padding {
        ident: &'a Ident,
        count: usize,
        ty: String,
        default_value: Option<String>,
    },
    Array {
        ident: &'a Ident,
        size: usize,
        ty: String,
    },
    VecU8 {
        ident: &'a Ident,
        size: usize,
    },
    Skip,
}

fn parse_extractor_attr<'a>(
    attr: &'a syn::Attribute,
    ident: &'a Ident,
    _ty: &'a Type,
) -> (
    Option<FieldInfo<'a>>,
    Option<TokenStream2>,
    Option<TokenStream2>,
) {
    let mut field_info = None;
    let mut counter_size = None;
    let mut property_item_size = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            field_info = Some(FieldInfo::Id { ident });
        } else if meta.path.is_ident("index") {
            field_info = Some(FieldInfo::Index { ident });
        } else if meta.path.is_ident("string") {
            let mut encoding = None;
            let mut size = None;
            meta.parse_nested_meta(|string_meta| {
                if string_meta.path.is_ident("encoding") {
                    let value = string_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    encoding = Some(lit.value());
                } else if string_meta.path.is_ident("size") {
                    let value = string_meta.value()?;
                    let lit: LitInt = value.parse()?;
                    size = Some(lit.base10_parse::<usize>().expect("size must be usize"));
                }
                Ok(())
            })?;
            let encoding = encoding.expect("string requires encoding");
            let size = size.expect("string requires size");
            field_info = Some(FieldInfo::String {
                ident,
                encoding,
                size,
            });
        } else if meta.path.is_ident("primitive") {
            let mut primitive_ty = None;
            meta.parse_nested_meta(|prim_meta| {
                if prim_meta.path.is_ident("type") {
                    let value = prim_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    primitive_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let primitive_ty = primitive_ty.expect("primitive requires type");
            field_info = Some(FieldInfo::Primitive {
                ident,
                ty: primitive_ty,
            });
        } else if meta.path.is_ident("enum_from_u8") {
            let mut enum_ty = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    enum_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let enum_ty = enum_ty.expect("enum_from_u8 requires type");
            field_info = Some(FieldInfo::EnumFromU8 { ident, enum_ty });
        } else if meta.path.is_ident("enum_from_u32") {
            let mut enum_ty = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    enum_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let enum_ty = enum_ty.expect("enum_from_u32 requires type");
            field_info = Some(FieldInfo::EnumFromU32 { ident, enum_ty });
        } else if meta.path.is_ident("enum_from_i16") {
            let mut enum_ty = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    enum_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let enum_ty = enum_ty.expect("enum_from_i16 requires type");
            field_info = Some(FieldInfo::EnumFromI16 { ident, enum_ty });
        } else if meta.path.is_ident("enum_from_i32") {
            let mut enum_ty = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    enum_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let enum_ty = enum_ty.expect("enum_from_i32 requires type");
            field_info = Some(FieldInfo::EnumFromI32 { ident, enum_ty });
        } else if meta.path.is_ident("enum_from_i32_from_u8") {
            let mut enum_ty = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    enum_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let enum_ty = enum_ty.expect("enum_from_i32_from_u8 requires type");
            field_info = Some(FieldInfo::EnumFromI32FromU8 { ident, enum_ty });
        } else if meta.path.is_ident("padding") {
            let mut count = None;
            let mut padding_ty = None;
            let mut default_value = None;
            meta.parse_nested_meta(|pad_meta| {
                if pad_meta.path.is_ident("count") {
                    let value = pad_meta.value()?;
                    let lit: LitInt = value.parse()?;
                    count = Some(lit.base10_parse::<usize>().expect("count must be usize"));
                } else if pad_meta.path.is_ident("type") {
                    let value = pad_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    padding_ty = Some(lit.value());
                } else if pad_meta.path.is_ident("default_value") {
                    let value = pad_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    default_value = Some(lit.value());
                }
                Ok(())
            })?;
            let count = count.expect("padding requires count");
            let padding_ty = padding_ty.expect("padding requires type");
            field_info = Some(FieldInfo::Padding {
                ident,
                count,
                ty: padding_ty,
                default_value,
            });
        } else if meta.path.is_ident("array") {
            let mut size = None;
            let mut array_ty = None;
            meta.parse_nested_meta(|arr_meta| {
                if arr_meta.path.is_ident("size") {
                    let value = arr_meta.value()?;
                    let lit: LitInt = value.parse()?;
                    size = Some(lit.base10_parse::<usize>().expect("size must be usize"));
                } else if arr_meta.path.is_ident("type") {
                    let value = arr_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    array_ty = Some(lit.value());
                }
                Ok(())
            })?;
            let size = size.expect("array requires size");
            let array_ty = array_ty.expect("array requires type");
            field_info = Some(FieldInfo::Array {
                ident,
                size,
                ty: array_ty,
            });
        } else if meta.path.is_ident("vec_u8") {
            let mut size = None;
            meta.parse_nested_meta(|vec_meta| {
                if vec_meta.path.is_ident("size") {
                    let value = vec_meta.value()?;
                    let lit: LitInt = value.parse()?;
                    size = Some(lit.base10_parse::<usize>().expect("size must be usize"));
                }
                Ok(())
            })?;
            let size = size.expect("vec_u8 requires size");
            field_info = Some(FieldInfo::VecU8 { ident, size });
        } else if meta.path.is_ident("skip") {
            field_info = Some(FieldInfo::Skip);
        } else if meta.path.is_ident("counter_size") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            let val = lit.base10_parse::<u8>().expect("counter_size must be u8");
            counter_size = Some(quote! { #val });
        } else if meta.path.is_ident("property_item_size") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            let val = lit
                .base10_parse::<i32>()
                .expect("property_item_size must be i32");
            property_item_size = Some(quote! { #val });
        }
        Ok(())
    })
    .expect("Failed to parse #[extractor(...)] arguments");

    (field_info, counter_size, property_item_size)
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_encoding_tokens(encoding: &str) -> TokenStream2 {
    match encoding {
        "WINDOWS-1250" | "WINDOWS_1250" => quote! { encoding_rs::WINDOWS_1250 },
        "EUC-KR" | "EUC_KR" => quote! { encoding_rs::EUC_KR },
        "UTF-8" | "UTF_8" => quote! { encoding_rs::UTF_8 },
        other => panic!(
            "Unknown encoding '{}' in #[extractor]; expected WINDOWS-1250, EUC-KR, or UTF-8",
            other
        ),
    }
}

// ============================================================================
// TextExtractor Derive Macro
// ============================================================================

/// Derive macro that generates an `Extractor` impl for text/CSV files.
///
/// Struct-level attributes:
/// - `#[extractor(encoding = "EUC_KR|WINDOWS_1250|UTF_8")]`
/// - `#[extractor(delimiter = ",")]`
/// - `#[extractor(comment_char = ";")]`
///
/// Field-level attributes:
/// - `#[extractor(field = N)]` - CSV field index (0-based)
/// - `#[extractor(parse_null)]` - Field is Option<String>, uses parse_null()
/// - `#[extractor(enum_from_i32(type = "EnumType"))]` - Parse as i32-based enum
///
/// Example:
/// ```ignore
/// #[derive(TextExtractor)]
/// #[extractor(encoding = "EUC_KR", delimiter = ",")]
/// pub struct MapIni {
///     #[extractor(field = 0)]
///     pub id: i32,
///     #[extractor(field = 1, parse_null)]
///     pub filename: Option<String>,
/// }
/// ```
#[proc_macro_derive(TextExtractor, attributes(extractor))]
pub fn derive_text_extractor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("TextExtractor only supports structs with named fields"),
        },
        _ => panic!("TextExtractor can only be derived for structs"),
    };

    // Parse struct-level attributes
    let mut encoding = quote! { encoding_rs::EUC_KR };
    let mut delimiter = quote! { "," };
    let mut comment_char = quote! { ";" };

    for attr in &input.attrs {
        if attr.path().is_ident("extractor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("encoding") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    let enc = match lit.value().as_str() {
                        "EUC_KR" => quote! { encoding_rs::EUC_KR },
                        "WINDOWS_1250" => quote! { encoding_rs::WINDOWS_1250 },
                        "UTF_8" => quote! { encoding_rs::UTF_8 },
                        other => panic!(
                            "Unknown encoding '{}'; expected EUC_KR, WINDOWS_1250, or UTF_8",
                            other
                        ),
                    };
                    encoding = enc;
                } else if meta.path.is_ident("delimiter") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    delimiter = quote! { #lit };
                } else if meta.path.is_ident("comment_char") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    comment_char = quote! { #lit };
                }
                Ok(())
            })
            .expect("Failed to parse struct-level #[extractor(...)]");
        }
    }

    // Parse field attributes
    let mut field_infos: Vec<TextFieldInfo> = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let extractor_attr = field.attrs.iter().find(|a| a.path().is_ident("extractor"));

        let Some(attr) = extractor_attr else {
            continue;
        };

        if let Some(info) = parse_text_extractor_attr(attr, field_ident, field_ty) {
            field_infos.push(info);
        }
    }

    // Sort by field index for consistent parsing
    field_infos.sort_by_key(|f| f.index);

    // Generate parse statements
    let parse_arms: Vec<TokenStream2> = field_infos.iter().map(|info| {
        let field_ident = &info.ident;
        let index = info.index;
        match &info.ty {
            TextFieldType::I32 => {
                quote! {
                    #field_ident: parts[#index].trim().parse::<i32>().unwrap_or_default(),
                }
            }
            TextFieldType::String => {
                quote! {
                    #field_ident: parts[#index].trim().to_string(),
                }
            }
            TextFieldType::OptionString => {
                quote! {
                    #field_ident: crate::references::extractor::parse_null(parts[#index].trim()),
                }
            }
            TextFieldType::EnumFromI32(enum_ty) => {
                let enum_ident = Ident::new(enum_ty, Span::call_site());
                quote! {
                    #field_ident: #enum_ident::from_i32(parts[#index].trim().parse::<i32>().unwrap_or_default()).unwrap_or_default(),
                }
            }
        }
    }).collect();

    // Generate write format string
    let field_count = field_infos.len();
    let mut format_parts: Vec<String> = Vec::new();
    let mut write_field_exprs: Vec<TokenStream2> = Vec::new();

    for info in &field_infos {
        let field_ident = &info.ident;
        format_parts.push("{}".to_string());

        match &info.ty {
            TextFieldType::I32 => {
                write_field_exprs.push(quote! { record.#field_ident.to_string() });
            }
            TextFieldType::String => {
                write_field_exprs.push(quote! { record.#field_ident.clone() });
            }
            TextFieldType::OptionString => {
                write_field_exprs
                    .push(quote! { record.#field_ident.as_deref().unwrap_or("null").to_string() });
            }
            TextFieldType::EnumFromI32(_) => {
                write_field_exprs.push(quote! { i32::from(record.#field_ident).to_string() });
            }
        }
    }

    let write_exprs = quote! {
        let mut fields = vec![
            #(#write_field_exprs),*
        ];
        let line = fields.join(#delimiter);
        let line = std::format!("{}\r\n", line);
        let (cow, _, _) = #encoding.encode(&line);
        writer.write_all(&cow)?;
    };

    let expanded = quote! {
        impl crate::references::extractor::Extractor for #name {
            fn parse<R: std::io::Read + std::io::Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
                use std::io::{BufRead, BufReader};
                let decoded = encoding_rs_io::DecodeReaderBytesBuilder::new()
                    .encoding(Some(#encoding))
                    .build(reader.by_ref());
                let buf_reader = BufReader::new(decoded);
                let delim = #delimiter;
                let mut items: Vec<#name> = Vec::new();

                for line in buf_reader.lines().map_while(std::io::Result::ok) {
                    let trimmed = line.trim();
                    if trimmed.starts_with(#comment_char) || trimmed.is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> = trimmed.split(delim).collect();
                    if parts.len() < #field_count {
                        continue;
                    }

                    items.push(#name {
                        #(#parse_arms)*
                    });
                }

                Ok(items)
            }

            fn to_writer<W: std::io::Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
                for record in records {
                    #write_exprs
                }
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

enum TextFieldType {
    I32,
    String,
    OptionString,
    EnumFromI32(String),
}

struct TextFieldInfo {
    ident: Ident,
    index: usize,
    ty: TextFieldType,
}

fn parse_text_extractor_attr(
    attr: &syn::Attribute,
    ident: &Ident,
    ty: &Type,
) -> Option<TextFieldInfo> {
    let mut field_index: Option<usize> = None;
    let mut parse_null = false;
    let mut enum_ty: Option<String> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("field") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            field_index = Some(lit.base10_parse::<usize>().expect("field must be usize"));
        } else if meta.path.is_ident("parse_null") {
            parse_null = true;
        } else if meta.path.is_ident("enum_from_i32") {
            let mut et = None;
            meta.parse_nested_meta(|enum_meta| {
                if enum_meta.path.is_ident("type") {
                    let value = enum_meta.value()?;
                    let lit: LitStr = value.parse()?;
                    et = Some(lit.value());
                }
                Ok(())
            })?;
            enum_ty = et;
        }
        Ok(())
    })
    .expect("Failed to parse field-level #[extractor(...)]");

    let index = field_index.expect("#[extractor(field = N)] is required");

    let field_ty = if parse_null {
        TextFieldType::OptionString
    } else if let Some(ref et) = enum_ty {
        TextFieldType::EnumFromI32(et.clone())
    } else {
        // Infer from Rust type
        let ty_str = quote! { #ty }.to_string();
        if ty_str.contains("String") && !ty_str.contains("Option") {
            TextFieldType::String
        } else if ty_str.contains("i32") {
            TextFieldType::I32
        } else {
            TextFieldType::String // default to string for unknown types
        }
    };

    Some(TextFieldInfo {
        ident: ident.clone(),
        index,
        ty: field_ty,
    })
}
