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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_encoding_rs_tokens_cover_every_registry_spelling() {
        let enc: Ident = parse_quote!(renamed_encoding_rs);
        for spelling in TextEncoding::ALL.into_iter().flat_map(|e| e.spellings()) {
            let expected = TextEncoding::from_spelling(spelling)
                .unwrap()
                .encoding_rs_name();
            let tokens = encoding_rs_tokens(spelling, &enc).to_string();
            assert_eq!(
                tokens,
                format!("renamed_encoding_rs :: {expected}"),
                "spelling {spelling}"
            );
        }
    }

    #[test]
    fn test_text_encoding_tokens_cover_every_registry_spelling() {
        let path: TokenStream2 = quote! { crate::localization::TextEncoding };
        for spelling in TextEncoding::ALL.into_iter().flat_map(|e| e.spellings()) {
            let expected = TextEncoding::from_spelling(spelling)
                .unwrap()
                .variant_name();
            let tokens = text_encoding_tokens(spelling, &path).to_string();
            assert_eq!(
                tokens,
                format!("crate :: localization :: TextEncoding :: {expected}"),
                "spelling {spelling}"
            );
        }
    }

    #[test]
    #[should_panic(expected = "Unknown encoding")]
    fn test_unknown_encoding_is_rejected() {
        let enc: Ident = parse_quote!(encoding_rs);
        encoding_rs_tokens("ISO-8859-2", &enc);
    }
}
