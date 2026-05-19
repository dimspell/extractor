use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, LitInt, LitStr, Type};

pub fn expand(input: DeriveInput) -> TokenStream2 {
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

    expanded
}

pub(crate) enum TextFieldType {
    I32,
    String,
    OptionString,
    EnumFromI32(String),
}

pub(crate) struct TextFieldInfo {
    pub ident: Ident,
    pub index: usize,
    pub ty: TextFieldType,
}

pub(crate) fn parse_text_extractor_attr(
    attr: &Attribute,
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
