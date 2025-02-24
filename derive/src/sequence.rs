use crate::{check_derive::check_lastderive_fromber, container::*};
use proc_macro2::Span;
use quote::quote;
use syn::{Data, Ident, WherePredicate};

pub fn derive_ber_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Sequence),
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

pub fn derive_berparser_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Sequence),
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
    let impl_berparser = container.gen_berparser();
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

pub fn derive_der_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Sequence),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });
    let impl_tryfrom = container.gen_tryfrom();
    let impl_tagged = container.gen_tagged();
    let impl_checkconstraints = container.gen_checkconstraints();
    let impl_fromder = container.gen_fromder();
    let impl_derive_derparser = container.gen_derive_derparser();
    let ts = s.gen_impl(quote! {
        extern crate asn1_rs;

        #impl_tryfrom
        #impl_tagged
        #impl_checkconstraints
        #impl_fromder
        #impl_derive_derparser
    });
    if debug_derive {
        eprintln!("{}", ts);
    }
    ts
}

pub fn derive_toder_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Sequence),
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

    let impl_to_der_len = container.gen_to_der_len();
    let impl_write_der_header = container.gen_write_der_header();
    let impl_write_der_content = container.gen_write_der_content();

    // note: `gen impl` in synstructure takes care of appending extra where clauses if any, and removing
    // the `where` statement if there are none.
    let ts = s.gen_impl(quote! {
        extern crate asn1_rs;

        #[cfg(feature = "std")]
        gen impl asn1_rs::ToDer for @Self where #(#wh)+* {
            #impl_to_der_len
            #impl_write_der_header
            #impl_write_der_content
        }
    });
    if debug_derive {
        eprintln!("{}", ts);
    }
    ts
}
