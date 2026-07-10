use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, LitInt, LitStr, Type};

pub fn expand(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("BinaryRecord only supports structs with named fields"),
        },
        _ => panic!("BinaryRecord can only be derived for structs"),
    };

    // Parse all field attributes
    let mut field_infos: Vec<BinaryFieldInfo> = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let binary_attr = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident("binary_record"));

        let info = match binary_attr {
            Some(attr) => parse_binary_record_attr(attr, field_ident, field_ty),
            None => auto_detect_field(field_ident, field_ty),
        };

        if let Some(info) = info {
            field_infos.push(info);
        }
    }

    // Compute total record size
    let mut total_size = 0usize;
    let mut size_contributors: Vec<TokenStream2> = Vec::new();
    for info in &field_infos {
        let (size, token) = info.size_token();
        total_size += size;
        if let Some(t) = token {
            size_contributors.push(t);
        }
    }

    // Generate parse statements
    let mut parse_stmts: Vec<TokenStream2> = Vec::new();
    let mut struct_field_inits: Vec<TokenStream2> = Vec::new();

    for info in &field_infos {
        match info {
            BinaryFieldInfo::String {
                ident,
                encoding,
                size,
            } => {
                let encoding_tokens = get_encoding_tokens(encoding);
                let buf_ident = Ident::new(&format!("{}_buf", ident), proc_macro2::Span::call_site());
                let data_len_ident =
                    Ident::new(&format!("{}_data_len", ident), proc_macro2::Span::call_site());
                parse_stmts.push(quote! {
                    let mut #buf_ident = [0u8; #size];
                    reader.read_exact(&mut #buf_ident)?;
                    let #data_len_ident = #buf_ident.iter().position(|&b| b == 0).unwrap_or(#size);
                    let (#ident, _, _) = #encoding_tokens.decode(&#buf_ident[..#data_len_ident]);
                    let #ident = #ident.trim().to_string();
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            BinaryFieldInfo::Primitive { ident, ty } => {
                let read_stmt = match ty.as_str() {
                    "i16" => {
                        quote! { reader.read_i16::<byteorder::LittleEndian>()? }
                    }
                    "i32" => {
                        quote! { reader.read_i32::<byteorder::LittleEndian>()? }
                    }
                    "u8" => quote! { reader.read_u8()? },
                    "u16" => {
                        quote! { reader.read_u16::<byteorder::LittleEndian>()? }
                    }
                    "u32" => {
                        quote! { reader.read_u32::<byteorder::LittleEndian>()? }
                    }
                    _ => panic!("Unsupported primitive type: {}", ty),
                };
                parse_stmts.push(quote! {
                    let #ident = #read_stmt;
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            BinaryFieldInfo::VecU8 { ident, size } => {
                parse_stmts.push(quote! {
                    let mut #ident = vec![0u8; #size];
                    reader.read_exact(&mut #ident)?;
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            BinaryFieldInfo::FixedArray { ident, size } => {
                parse_stmts.push(quote! {
                    let mut #ident = [0u8; #size];
                    reader.read_exact(&mut #ident)?;
                });
                struct_field_inits.push(quote! { #ident: #ident, });
            }
            BinaryFieldInfo::Padding {
                ident,
                count,
                ty,
                default_value,
            } => {
                let default_expr = default_value_expr(ty, default_value.as_deref());
                for _ in 0..*count {
                    let read_stmt = match ty.as_str() {
                        "i16" => {
                            quote! { reader.read_i16::<byteorder::LittleEndian>()? }
                        }
                        "i32" => {
                            quote! { reader.read_i32::<byteorder::LittleEndian>()? }
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
            BinaryFieldInfo::Skip => {}
        }
    }

    // Generate write statements
    let mut write_stmts: Vec<TokenStream2> = Vec::new();
    for info in &field_infos {
        match info {
            BinaryFieldInfo::String {
                ident,
                encoding,
                size,
            } => {
                let encoding_tokens = get_encoding_tokens(encoding);
                let buf_ident = Ident::new(&format!("{}_buf", ident), proc_macro2::Span::call_site());
                write_stmts.push(quote! {
                    let mut #buf_ident = vec![0u8; #size];
                    let (cow, _, _) = #encoding_tokens.encode(&self.#ident);
                    let len = std::cmp::min(cow.len(), #size);
                    #buf_ident[..len].copy_from_slice(&cow[..len]);
                    writer.write_all(&#buf_ident)?;
                });
            }
            BinaryFieldInfo::Primitive { ident, ty } => {
                let write_stmt = match ty.as_str() {
                    "i16" => {
                        quote! { writer.write_i16::<byteorder::LittleEndian>(self.#ident)?; }
                    }
                    "i32" => {
                        quote! { writer.write_i32::<byteorder::LittleEndian>(self.#ident)?; }
                    }
                    "u8" => quote! { writer.write_u8(self.#ident)?; },
                    "u16" => {
                        quote! { writer.write_u16::<byteorder::LittleEndian>(self.#ident)?; }
                    }
                    "u32" => {
                        quote! { writer.write_u32::<byteorder::LittleEndian>(self.#ident)?; }
                    }
                    _ => panic!("Unsupported primitive type: {}", ty),
                };
                write_stmts.push(quote! {
                    #write_stmt
                });
            }
            BinaryFieldInfo::VecU8 { ident, size: _ } => {
                write_stmts.push(quote! {
                    writer.write_all(&self.#ident)?;
                });
            }
            BinaryFieldInfo::FixedArray { ident, size: _ } => {
                write_stmts.push(quote! {
                    writer.write_all(&self.#ident)?;
                });
            }
            BinaryFieldInfo::Padding {
                ident: _,
                count,
                ty,
                default_value: _,
            } => {
                for _ in 0..*count {
                    let write_zero = match ty.as_str() {
                        "i16" => {
                            quote! { writer.write_i16::<byteorder::LittleEndian>(0)?; }
                        }
                        "i32" => {
                            quote! { writer.write_i32::<byteorder::LittleEndian>(0)?; }
                        }
                        "u8" => quote! { writer.write_u8(0)?; },
                        _ => panic!("Unsupported padding type: {}", ty),
                    };
                    write_stmts.push(quote! {
                        #write_zero
                    });
                }
            }
            BinaryFieldInfo::Skip => {}
        }
    }

    let record_size_name = Ident::new(
        &format!("{}_RECORD_SIZE", name.to_string().to_ascii_uppercase()),
        proc_macro2::Span::call_site(),
    );

    let expanded = quote! {
        const #record_size_name: usize = #total_size;

        impl #name {
            /// Parse a single record from raw bytes.
            pub fn parse(data: &[u8]) -> std::io::Result<Self> {
                use byteorder::ReadBytesExt;
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
                use byteorder::WriteBytesExt;
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

fn default_value_expr(ty: &str, default_value: Option<&str>) -> TokenStream2 {
    if let Some(dv) = default_value {
        match ty {
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
        match ty {
            "i16" => quote! { 0i16 },
            "i32" => quote! { 0i32 },
            "u8" => quote! { 0u8 },
            _ => panic!("Unsupported padding type: {}", ty),
        }
    }
}

enum BinaryFieldInfo {
    String {
        ident: Ident,
        encoding: String,
        size: usize,
    },
    Primitive {
        ident: Ident,
        ty: String,
    },
    VecU8 {
        ident: Ident,
        size: usize,
    },
    FixedArray {
        ident: Ident,
        size: usize,
    },
    Padding {
        ident: Ident,
        count: usize,
        ty: String,
        default_value: Option<String>,
    },
    Skip,
}

impl BinaryFieldInfo {
    /// Returns (byte_size, optional compile-time size token).
    fn size_token(&self) -> (usize, Option<TokenStream2>) {
        match self {
            BinaryFieldInfo::String { size, .. } => (*size, None),
            BinaryFieldInfo::Primitive { ty, .. } => match ty.as_str() {
                "i16" => (2, None),
                "i32" => (4, None),
                "u8" => (1, None),
                "u16" => (2, None),
                "u32" => (4, None),
                _ => panic!("Unsupported primitive type: {}", ty),
            },
            BinaryFieldInfo::VecU8 { size, .. } => (*size, None),
            BinaryFieldInfo::FixedArray { size, .. } => (*size, None),
            BinaryFieldInfo::Padding { count, ty, .. } => {
                let per = match ty.as_str() {
                    "i16" => 2,
                    "i32" => 4,
                    "u8" => 1,
                    _ => panic!("Unsupported padding type: {}", ty),
                };
                (per * count, None)
            }
            BinaryFieldInfo::Skip => (0, None),
        }
    }
}

/// Parse a `#[binary_record(...)]` attribute on a field.
fn parse_binary_record_attr(
    attr: &syn::Attribute,
    ident: &Ident,
    _ty: &Type,
) -> Option<BinaryFieldInfo> {
    let mut field_info: Option<BinaryFieldInfo> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("string") {
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
            field_info = Some(BinaryFieldInfo::String {
                ident: ident.clone(),
                encoding,
                size,
            });
        } else if meta.path.is_ident("size") {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            let size = lit.base10_parse::<usize>().expect("size must be usize");
            // Vec<u8> or [u8; N] — we infer which from the type in auto_detect
            field_info = Some(BinaryFieldInfo::VecU8 {
                ident: ident.clone(),
                size,
            });
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
            field_info = Some(BinaryFieldInfo::Padding {
                ident: ident.clone(),
                count,
                ty: padding_ty,
                default_value,
            });
        } else if meta.path.is_ident("skip") {
            field_info = Some(BinaryFieldInfo::Skip);
        }
        Ok(())
    })
    .expect("Failed to parse #[binary_record(...)] arguments");

    field_info
}

/// Auto-detect field type when no #[binary_record] attribute is present.
/// Supports: i16, i32, u8, u16, u32, Vec<u8>, [u8; N]
fn auto_detect_field(ident: &Ident, ty: &Type) -> Option<BinaryFieldInfo> {
    // Try to detect Vec<u8> — requires size annotation, so we can't auto-detect without it
    // Try to detect [u8; N]
    if let Type::Array(arr) = ty {
        if let Type::Path(elem_path) = arr.elem.as_ref() {
            if elem_path.path.is_ident("u8") {
                if let syn::Expr::Lit(lit) = &arr.len {
                    if let syn::Lit::Int(n) = &lit.lit {
                        let size = n.base10_parse::<usize>().expect("array size must be usize");
                        return Some(BinaryFieldInfo::FixedArray {
                            ident: ident.clone(),
                            size,
                        });
                    }
                }
            }
        }
    }

    // Try to detect Vec<u8>
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(Type::Path(elem))) = args.args.first() {
                        if elem.path.is_ident("u8") {
                            panic!(
                                "Field `{}` is Vec<u8> but has no #[binary_record(size = N)] annotation",
                                ident
                            );
                        }
                    }
                }
            }
        }
    }

    // Try to detect primitive types (i16, i32, u8, u16, u32)
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();
            match type_name.as_str() {
                "i16" | "i32" | "u8" | "u16" | "u32" => {
                    return Some(BinaryFieldInfo::Primitive {
                        ident: ident.clone(),
                        ty: type_name,
                    });
                }
                "String" => {
                    panic!(
                        "Field `{}` is `String` but has no #[binary_record(string(...))] annotation",
                        ident
                    );
                }
                _ => {
                    // Unknown type — skip it (could be a computed field or unit struct)
                    return None;
                }
            }
        }
    }

    // Unknown type — skip
    None
}

fn get_encoding_tokens(encoding: &str) -> TokenStream2 {
    match encoding {
        "WINDOWS-1250" | "WINDOWS_1250" => quote! { encoding_rs::WINDOWS_1250 },
        "EUC-KR" | "EUC_KR" => quote! { encoding_rs::EUC_KR },
        "UTF-8" | "UTF_8" => quote! { encoding_rs::UTF_8 },
        other => panic!(
            "Unknown encoding '{}' in #[binary_record]; expected WINDOWS-1250, EUC-KR, or UTF-8",
            other
        ),
    }
}
