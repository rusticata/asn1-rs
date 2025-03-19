use crate::asn1_type::Asn1Type;
use crate::check_derive::check_lastderive_fromber;
use crate::container::*;
use crate::options::Options;
use crate::sequence::DeriveSequence;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, Ident, Result};

pub fn derive_alias(s: synstructure::Structure) -> TokenStream {
    match DeriveSequence::new(&s, ContainerType::Alias) {
        Ok(s) => s.to_tokens(),
        Err(e) => e.to_compile_error().into(),
    }
}

pub fn derive_ber_alias(s: synstructure::Structure) -> TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Alias),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });

    let impl_tryfrom = container.gen_tryfrom();
    let impl_tagged = container.gen_tagged();
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

pub fn derive_berparser_alias(s: synstructure::Structure) -> Result<TokenStream> {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Alias),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let options = Options::from_struct(&s)?;
    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });
    let last_berderive = check_lastderive_fromber(ast);

    let impl_tagged = if last_berderive {
        // Note: this impl can (and will) be empty if inner type is Any
        container.gen_tagged()
    } else {
        quote! {}
    };
    let impl_berparser = container.gen_berparser(Asn1Type::Ber, &options);
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

pub fn derive_der_alias(s: synstructure::Structure) -> TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Alias),
        _ => panic!("Unsupported type, cannot derive"),
    };

    // let options = Options::from_struct(&s)?;
    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });
    let impl_tryfrom = container.gen_tryfrom();
    let impl_tagged = container.gen_tagged();
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

pub fn derive_derparser_alias(s: synstructure::Structure) -> Result<TokenStream> {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Alias),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let options = Options::from_struct(&s)?;
    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });
    let last_berderive = check_lastderive_fromber(ast);

    let impl_tagged = if last_berderive {
        // Note: this impl can (and will) be empty if inner type is Any
        container.gen_tagged()
    } else {
        quote! {}
    };
    let impl_derparser = container.gen_berparser(Asn1Type::Der, &options);
    let ts = s.gen_impl(quote! {
        extern crate asn1_rs;

        #impl_tagged
        #impl_derparser
    });
    if debug_derive {
        eprintln!("{}", ts);
    }
    Ok(ts)
}
