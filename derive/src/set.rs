use crate::container::*;
use crate::sequence::*;

pub fn derive_ber_set(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_ber_container(s, ContainerType::Set)
}

pub fn derive_berparser_set(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_berparser_container(s, ContainerType::Set)
}

pub fn derive_der_set(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_der_container(s, ContainerType::Set)
}

pub fn derive_derparser_set(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_derparser_container(s, ContainerType::Set)
}

pub fn derive_tober_set(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_tober_container(s, ContainerType::Set, Asn1Type::Ber)
}

pub fn derive_toder_set(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_tober_container(s, ContainerType::Set, Asn1Type::Der)
}
