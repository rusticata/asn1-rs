use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Data, DataStruct, DeriveInput, Ident, Lifetime, WherePredicate};

pub fn derive_ber_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.path
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });

    let impl_tryfrom = container.gen_tryfrom();
    let impl_tagged = container.gen_tagged();
    let ts = s.gen_impl(quote! {
        #impl_tryfrom
        #impl_tagged
    });
    if debug_derive {
        eprintln!("{}", ts.to_string());
    }
    ts
}

pub fn derive_der_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.path
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });

    let impl_tryfrom = container.gen_tryfrom();
    let impl_tagged = container.gen_tagged();
    let impl_checkconstraints = container.gen_checkconstraints();
    let ts = s.gen_impl(quote! {
        #impl_tryfrom
        #impl_tagged
        #impl_checkconstraints
    });
    if debug_derive {
        eprintln!("{}", ts.to_string());
    }
    ts
}

pub struct Container {
    parse_content: TokenStream,
    field_names: Vec<Ident>,
    where_predicates: Vec<WherePredicate>,
}

impl Container {
    pub fn from_datastruct(ds: &DataStruct, ast: &DeriveInput) -> Self {
        let parse_content = derive_ber_sequence_content(ds);
        let field_names = ds.fields.iter().map(|f| f.ident.clone().unwrap()).collect();
        // dbg!(s);

        // get lifetimes from generics
        let lfts: Vec<_> = ast.generics.lifetimes().collect();
        let mut where_predicates = Vec::new();
        if !lfts.is_empty() {
            // input slice must outlive all lifetimes from Self
            let lft = Lifetime::new("'ber", Span::call_site());
            let wh: WherePredicate = parse_quote! { #lft: #(#lfts)+* };
            where_predicates.push(wh);
        };

        Container {
            parse_content,
            field_names,
            where_predicates,
        }
    }

    pub fn gen_tryfrom(&self) -> TokenStream {
        let parse_content = &self.parse_content;
        let field_names = &self.field_names;
        let lifetime = Lifetime::new("'ber", Span::call_site());
        let wh = &self.where_predicates;
        // note: `gen impl` in synstructure takes care of appending extra where clauses if any, and removing
        // the `where` statement if there are none.
        quote! {
            extern crate asn1_rs;
            use asn1_rs::{Any, FromBer};
            use core::convert::TryFrom;

            gen impl<#lifetime> TryFrom<Any<#lifetime>> for @Self where #(#wh)+* {
                type Error = asn1_rs::Error;

                fn try_from(any: Any<#lifetime>) -> asn1_rs::Result<Self> {
                    any.tag().assert_eq(Self::TAG)?;

                    // no need to parse sequence, we already have content
                    let i = any.data;
                    //
                    #parse_content
                    //
                    let _ = i; // XXX check if empty?
                    Ok(Self{#(#field_names),*})
                }
            }
        }
    }

    pub fn gen_tagged(&self) -> TokenStream {
        quote! {
            gen impl<'ber> asn1_rs::Tagged for @Self {
                const TAG: asn1_rs::Tag = asn1_rs::Tag::Sequence;
            }
        }
    }

    pub fn gen_checkconstraints(&self) -> TokenStream {
        quote! {
            gen impl<'ber> asn1_rs::CheckDerConstraints for @Self {
                fn check_constraints(_any: &Any) -> Result<()> {
                    Ok(())
                }
            }
        }
    }
}

fn derive_ber_sequence_content(s: &DataStruct) -> TokenStream {
    let field_parsers: Vec<_> = s
        .fields
        .iter()
        .map(|f| {
            let name = &f.ident;
            quote! {
                let (i, #name) = FromBer::from_ber(i)?;
            }
        })
        .collect();

    quote! {
        #(#field_parsers)*
    }
}
