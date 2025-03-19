use proc_macro2::TokenStream;
use syn::Result;

use crate::asn1_type::Asn1Type;
use crate::container::*;
use crate::sequence::*;

pub fn derive_set(s: synstructure::Structure) -> TokenStream {
    match DeriveSequence::new(&s, ContainerType::Set) {
        Ok(s) => s.to_tokens(),
        Err(e) => e.to_compile_error().into(),
    }
}

//--- old-style derive

pub fn derive_ber_set(s: synstructure::Structure) -> TokenStream {
    derive_ber_container(s, ContainerType::Set)
}

pub fn derive_der_set(s: synstructure::Structure) -> TokenStream {
    derive_der_container(s, ContainerType::Set)
}

pub fn derive_berparser_set(s: synstructure::Structure) -> Result<TokenStream> {
    derive_berparser_container(s, ContainerType::Set, Asn1Type::Ber)
}

pub fn derive_derparser_set(s: synstructure::Structure) -> Result<TokenStream> {
    derive_berparser_container(s, ContainerType::Set, Asn1Type::Der)
}

pub fn derive_tober_set(s: synstructure::Structure) -> Result<TokenStream> {
    derive_tober_container(s, ContainerType::Set, Asn1Type::Ber)
}

pub fn derive_toder_set(s: synstructure::Structure) -> Result<TokenStream> {
    derive_tober_container(s, ContainerType::Set, Asn1Type::Der)
}
