//! Compile-time encoding resolution shared by all derive macros.
//!
//! All accepted spellings, canonical labels and constant names come from the
//! `dispel-encodings` registry — this module only translates a resolved
//! [`TextEncoding`] into the tokens each derive needs to emit. The crate/module
//! paths are supplied by the caller (`CodegenConfig`) so that there is still
//! exactly one path configuration, in `schema.rs`.

use dispel_encodings::TextEncoding;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

/// Resolve an `encoding = "..."` attribute value or panic with a single,
/// consistent error message.
fn resolve(spelling: &str) -> TextEncoding {
    TextEncoding::from_spelling(spelling).unwrap_or_else(|| {
        panic!(
            "Unknown encoding '{spelling}'; expected WINDOWS-1250, EUC-KR, or UTF-8 \
             (hyphen or underscore spelling, case-insensitive)"
        )
    })
}

/// Tokens for an `encoding_rs` constant, e.g. `encoding_rs::WINDOWS_1250`.
///
/// `enc` is the configured `encoding_rs` crate path (from `CodegenConfig`).
pub(crate) fn encoding_rs_tokens(spelling: &str, enc: &Ident) -> TokenStream2 {
    let constant = Ident::new(resolve(spelling).encoding_rs_name(), enc.span());
    quote! { #enc::#constant }
}

/// Tokens for a `TextEncoding` variant, e.g.
/// `crate::localization::TextEncoding::Windows1250`.
///
/// `text_encoding_path` is the configured path to the `TextEncoding` enum
/// (from `CodegenConfig`), e.g. `crate::localization::TextEncoding`.
pub(crate) fn text_encoding_tokens(
    spelling: &str,
    text_encoding_path: &TokenStream2,
) -> TokenStream2 {
    let variant = Ident::new(
        resolve(spelling).variant_name(),
        proc_macro2::Span::call_site(),
    );
    quote! { #text_encoding_path::#variant }
}
