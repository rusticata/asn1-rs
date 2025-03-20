use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Ident, LitStr, Token};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Asn1Type {
    Ber,
    Der,
}

impl Asn1Type {
    pub(crate) fn tober(&self) -> TokenStream {
        match *self {
            Asn1Type::Ber => quote!(ToBer),
            Asn1Type::Der => quote!(ToDer),
        }
    }

    pub(crate) fn compose(&self, suffix: &str) -> TokenStream {
        let prefix = match *self {
            Asn1Type::Ber => "ber",
            Asn1Type::Der => "der",
        };
        let s = format!("{prefix}{suffix}");
        let ident = Ident::new(&s, Span::call_site());
        quote! { #ident }
    }

    pub(crate) fn total_len_tokens(&self) -> TokenStream {
        match *self {
            Asn1Type::Ber => quote!(ber_total_len),
            Asn1Type::Der => quote!(der_total_len),
        }
    }

    pub(crate) fn content_len_tokens(&self) -> TokenStream {
        match *self {
            Asn1Type::Ber => quote!(ber_content_len),
            Asn1Type::Der => quote!(der_content_len),
        }
    }

    pub(crate) fn tag_info_tokens(&self) -> TokenStream {
        match *self {
            Asn1Type::Ber => quote!(ber_tag_info),
            Asn1Type::Der => quote!(der_tag_info),
        }
    }

    pub(crate) fn encode_tokens(&self) -> TokenStream {
        match *self {
            Asn1Type::Ber => quote!(ber_encode),
            Asn1Type::Der => quote!(der_encode),
        }
    }

    pub(crate) fn write_content_tokens(&self) -> TokenStream {
        match *self {
            Asn1Type::Ber => quote!(ber_write_content),
            Asn1Type::Der => quote!(der_write_content),
        }
    }

    pub(crate) fn parser(&self) -> TokenStream {
        match *self {
            Asn1Type::Ber => quote!(BerParser),
            Asn1Type::Der => quote!(DerParser),
        }
    }

    pub(crate) fn parse_ber(&self) -> TokenStream {
        match *self {
            Asn1Type::Ber => quote!(parse_ber),
            Asn1Type::Der => quote!(parse_der),
        }
    }

    pub(crate) fn from_ber_content(&self) -> TokenStream {
        match *self {
            Asn1Type::Ber => quote!(from_ber_content),
            Asn1Type::Der => quote!(from_der_content),
        }
    }

    pub fn parse_multi(input: ParseStream<'_>) -> syn::Result<impl IntoIterator<Item = Self>> {
        let lit_s: LitStr = input.parse()?;
        lit_s.parse_with(Punctuated::<Self, Token![,]>::parse_terminated)
    }
}

impl Parse for Asn1Type {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "BER" {
            Ok(Asn1Type::Ber)
        } else if ident == "DER" {
            Ok(Asn1Type::Der)
        } else {
            Err(Error::new(
                ident.span(),
                "Invalid ASN.1 type (possible values: BER, DER)",
            ))
        }
    }
}
