use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, LitInt};

use crate::extractor_impl::{FieldInfo, parse_extractor_attr};

pub(crate) fn derive_record_layout(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => panic!("RecordLayout only supports structs with named fields"),
        },
        _ => panic!("RecordLayout can only be derived for structs"),
    };

    let mut header_size = 4u32;
    let mut record_size = None;
    for attr in &input.attrs {
        if attr.path().is_ident("extractor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("counter_size") {
                    let value = meta.value()?;
                    header_size = value.parse::<LitInt>()?.base10_parse()?;
                } else if meta.path.is_ident("property_item_size") {
                    let value = meta.value()?;
                    record_size = Some(value.parse::<LitInt>()?.base10_parse::<u32>()?);
                }
                Ok(())
            })
            .expect("failed to parse #[extractor(...)]");
        }
    }
    let record_size = record_size.expect("RecordLayout requires property_item_size");
    let mut offset = 0u32;
    let mut defs = Vec::new();
    for field in fields {
        let ident = field.ident.as_ref().expect("named field");
        let Some(attr) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("extractor"))
        else {
            continue;
        };
        let (info, _counter, _item_size) = parse_extractor_attr(attr, ident, &field.ty);
        let Some(info) = info else { continue };
        let Some((size, ty)) = field_size_and_type(&info) else {
            continue;
        };
        let field_name = ident.to_string();
        defs.push(quote! {
            crate::references::layout::FieldDef {
                name: #field_name, offset: #offset, size: #size, ty: #ty,
            }
        });
        offset += size;
    }
    if offset != record_size {
        panic!(
            "RecordLayout fields for {} occupy {offset} bytes, but property_item_size is {record_size}",
            name
        );
    }
    quote! {
        impl crate::references::layout::RecordLayout for #name {
            const LAYOUT: crate::references::layout::FixedRecordLayout = crate::references::layout::FixedRecordLayout {
                type_name: stringify!(#name), header_size: #header_size, record_size: #record_size,
                fields: &[#(#defs),*],
            };
        }
    }
}

fn field_size_and_type(info: &FieldInfo<'_>) -> Option<(u32, String)> {
    let scalar = |ty: &str| match ty {
        "u8" | "i8" => 1,
        "u16" | "i16" => 2,
        "u32" | "i32" => 4,
        _ => panic!("unsupported layout scalar type: {ty}"),
    };
    match info {
        FieldInfo::Id { .. } | FieldInfo::Index { .. } | FieldInfo::Skip => None,
        FieldInfo::String { encoding, size, .. } => {
            Some((*size as u32, format!("string({encoding})")))
        }
        FieldInfo::Primitive { ty, .. } => Some((scalar(ty), ty.clone())),
        FieldInfo::EnumFromU8 { enum_ty, .. } | FieldInfo::EnumFromI32FromU8 { enum_ty, .. } => {
            Some((1, enum_ty.clone()))
        }
        FieldInfo::EnumFromU32 { enum_ty, .. } | FieldInfo::EnumFromI32 { enum_ty, .. } => {
            Some((4, enum_ty.clone()))
        }
        FieldInfo::EnumFromI16 { enum_ty, .. } => Some((2, enum_ty.clone())),
        FieldInfo::VecU8 { size, .. } => Some((*size as u32, "bytes".into())),
        FieldInfo::InventoryItem { wire_type, .. } => {
            Some((scalar(wire_type), format!("InventoryItem({wire_type})")))
        }
        FieldInfo::Padding { count, ty, .. } => {
            Some(((*count as u32) * scalar(ty), format!("padding({ty})")))
        }
        FieldInfo::Array { size, ty, .. } => {
            Some(((*size as u32) * scalar(ty), format!("[{ty}; {size}]")))
        }
    }
}
