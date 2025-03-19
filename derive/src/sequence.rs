use crate::asn1_type::Asn1Type;
use crate::check_derive::check_lastderive_fromber;
use crate::container::*;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, Ident, WherePredicate};

pub fn derive_ber_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_ber_container(s, ContainerType::Sequence)
}

pub fn derive_der_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_der_container(s, ContainerType::Sequence)
}

pub fn derive_berparser_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_berparser_container(s, ContainerType::Sequence, Asn1Type::Ber)
}

pub fn derive_derparser_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_berparser_container(s, ContainerType::Sequence, Asn1Type::Der)
}

pub fn derive_tober_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_tober_container(s, ContainerType::Sequence, Asn1Type::Ber)
}

pub fn derive_toder_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_tober_container(s, ContainerType::Sequence, Asn1Type::Der)
}

pub(crate) fn derive_ber_container(
    s: synstructure::Structure,
    container_type: ContainerType,
) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, container_type),
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
) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, container_type),
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
) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, container_type),
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
    let impl_berparser = container.gen_berparser(asn1_type);
    let ts = s.gen_impl(quote! {
        extern crate asn1_rs;

        #impl_tagged
        #impl_berparser
    });
    if debug_derive {
        eprintln!("{}", ts);
    }
    ts
}

pub(crate) fn derive_tober_container(
    s: synstructure::Structure,
    container_type: ContainerType,
    asn1_type: Asn1Type,
) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, container_type),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });

    //let lifetime = Lifetime::new("'ber", Span::call_site());
    let wh = &container.where_predicates;
    // we must filter out the 'ber lifetime (added for parsers, but not used here)
    let wh = wh.iter().filter(|predicate| match predicate {
        WherePredicate::Lifetime(lft) => lft.lifetime.ident != "ber",
        _ => true,
    });

    let impl_tober_content_len = container.gen_tober_content_len(asn1_type);
    let impl_tober_tag_info = container.gen_tober_tag_info(asn1_type);
    let impl_tober_write_content = container.gen_tober_write_content(asn1_type);
    let tober = asn1_type.tober();

    // note: `gen impl` in synstructure takes care of appending extra where clauses if any, and removing
    // the `where` statement if there are none.
    let ts = s.gen_impl(quote! {
        extern crate asn1_rs;

        #[cfg(feature = "std")]
        gen impl asn1_rs::#tober for @Self where #(#wh)+* {
            type Encoder = asn1_rs::BerGenericEncoder;

            #impl_tober_content_len
            #impl_tober_tag_info
            #impl_tober_write_content
        }
    });
    if debug_derive {
        eprintln!("{}", ts);
    }
    ts
}
