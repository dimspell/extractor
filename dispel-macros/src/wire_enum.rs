//! Shared code generation for wire-value → enum conversions.
//!
//! Every derive that reads an enum discriminant (`Extractor`, `TextExtractor`,
//! `RecordPatcher`, `TextRecordPatcher`) routes through
//! `WireValue::from_wire`, so the fallback policy is defined in exactly one
//! place (the trait's default method). The trait path itself is supplied by
//! `CodegenConfig`, keeping a single path configuration.

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;

/// Emit `<Enum as WireValue<Wire>>::from_wire(value)`.
///
/// `wire_value_trait` is the configured path to the `WireValue` trait.
pub(crate) fn from_wire_tokens(
    wire_value_trait: &TokenStream2,
    enum_ty: &str,
    wire_ty: &str,
    value: TokenStream2,
) -> TokenStream2 {
    let enum_ident = Ident::new(enum_ty, Span::call_site());
    let wire_ident = Ident::new(wire_ty, Span::call_site());
    quote! {
        <#enum_ident as #wire_value_trait<#wire_ident>>::from_wire(#value)
    }
}
