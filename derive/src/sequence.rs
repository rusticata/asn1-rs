use crate::asn1_type::Asn1Type;
use crate::check_derive::check_lastderive_fromber;
use crate::container::*;
use crate::options::Options;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, Error, Ident, Result};

pub fn derive_sequence(s: synstructure::Structure) -> TokenStream {
    match DeriveSequence::new(&s, ContainerType::Sequence) {
        Ok(s) => s.to_tokens(),
        Err(e) => e.to_compile_error().into(),
    }
}

pub struct DeriveSequence<'s> {
    ident: Ident,
    options: Options,

    synstruct: &'s synstructure::Structure<'s>,
    container: Container,
}

impl<'s> DeriveSequence<'s> {
    pub fn new(s: &'s synstructure::Structure<'_>, container_type: ContainerType) -> Result<Self> {
        let ast = s.ast();

        let container = match &ast.data {
            Data::Struct(ds) => Container::from_datastruct(ds, ast, container_type)?,
            _ => {
                return Err(Error::new_spanned(
                    &ast.ident,
                    "'Sequence/Set' can only be derived on `struct` type",
                ));
            }
        };

        let ident = ast.ident.clone();
        let options = Options::from_struct(&s)?;

        let s = Self {
            ident,
            options,
            synstruct: s,
            container,
        };
        Ok(s)
    }

    pub fn to_tokens(&self) -> TokenStream {
        let Self {
            options, synstruct, ..
        } = self;

        let impl_tagged = self.container.gen_tagged();

        let impl_berparser = self
            .container
            .gen_berparser(Asn1Type::Ber, options, synstruct);
        let impl_derparser = self
            .container
            .gen_berparser(Asn1Type::Der, options, synstruct);
        let impl_tober = self.container.gen_tober(Asn1Type::Ber, options, synstruct);
        let impl_toder = self.container.gen_tober(Asn1Type::Der, options, synstruct);
        let ts = self.synstruct.gen_impl(quote! {
            extern crate asn1_rs;

            #impl_tagged
            #impl_berparser
            #impl_derparser
            #impl_tober
            #impl_toder
        });
        if self.options.debug {
            eprintln!("// SEQUENCE for {}", self.ident);
            eprintln!("{}", ts);
        }
        ts
    }
}

//--- old-style derive

pub fn derive_ber_sequence(s: synstructure::Structure) -> TokenStream {
    derive_ber_container(s, ContainerType::Sequence)
}

pub fn derive_der_sequence(s: synstructure::Structure) -> TokenStream {
    derive_der_container(s, ContainerType::Sequence)
}

pub fn derive_berparser_sequence(s: synstructure::Structure) -> Result<TokenStream> {
    derive_berparser_container(s, ContainerType::Sequence, Asn1Type::Ber)
}

pub fn derive_derparser_sequence(s: synstructure::Structure) -> Result<TokenStream> {
    derive_berparser_container(s, ContainerType::Sequence, Asn1Type::Der)
}

pub fn derive_tober_sequence(s: synstructure::Structure) -> Result<TokenStream> {
    derive_tober_container(s, ContainerType::Sequence, Asn1Type::Ber)
}

pub fn derive_toder_sequence(s: synstructure::Structure) -> Result<TokenStream> {
    derive_tober_container(s, ContainerType::Sequence, Asn1Type::Der)
}

pub(crate) fn derive_ber_container(
    s: synstructure::Structure,
    container_type: ContainerType,
) -> TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, container_type).unwrap(),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });

    let last_berderive = check_lastderive_fromber(ast);

    let impl_tryfrom = container.gen_tryfrom();
    let impl_tagged = if last_berderive {
        container.gen_tagged()
    } else {
        quote! {}
    };
    let ts = s.gen_impl(quote! {
        extern crate asn1_rs;

        #impl_tryfrom
        #impl_tagged
    });
    if debug_derive {
        eprintln!("{}", ts);
    }
    ts
}

pub fn derive_der_container(
    s: synstructure::Structure,
    container_type: ContainerType,
) -> TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, container_type).unwrap(),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });
    let last_berderive = check_lastderive_fromber(ast);

    let impl_tagged = if last_berderive {
        container.gen_tagged()
    } else {
        quote! {}
    };
    let impl_tryfrom = container.gen_tryfrom();
    let impl_checkconstraints = container.gen_checkconstraints();
    let impl_fromder = container.gen_fromder();
    let ts = s.gen_impl(quote! {
        extern crate asn1_rs;

        #impl_tryfrom
        #impl_tagged
        #impl_checkconstraints
        #impl_fromder
    });
    if debug_derive {
        eprintln!("{}", ts);
    }
    ts
}

pub(crate) fn derive_berparser_container(
    s: synstructure::Structure,
    container_type: ContainerType,
    asn1_type: Asn1Type,
) -> Result<TokenStream> {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, container_type)?,
        _ => {
            return Err(Error::new_spanned(
                &ast.ident,
                "'Sequence/Set' can only be derived on `struct` type",
            ));
        }
    };

    let options = Options::from_struct(&s)?;
    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });

    let last_berderive = check_lastderive_fromber(ast);

    let impl_tagged = if last_berderive {
        container.gen_tagged()
    } else {
        quote! {}
    };
    let impl_berparser = container.gen_berparser(asn1_type, &options, &s);
    let ts = s.gen_impl(quote! {
        extern crate asn1_rs;

        #impl_tagged
        #impl_berparser
    });
    if debug_derive {
        eprintln!("{}", ts);
    }
    Ok(ts)
}

pub(crate) fn derive_tober_container(
    s: synstructure::Structure,
    container_type: ContainerType,
    asn1_type: Asn1Type,
) -> Result<TokenStream> {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, container_type)?,
        _ => panic!("Unsupported type, cannot derive"),
    };

    let options = Options::from_struct(&s)?;
    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });

    let ts = container.gen_tober(asn1_type, &options, &s);
    if debug_derive {
        eprintln!("{}", ts);
    }
    Ok(ts)
}
