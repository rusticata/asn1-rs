use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataStruct, Ident, Lifetime};

pub fn derive_ber_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(s) => Container::from_datastruct(s),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let impl_tryfrom = container.gen_tryfrom();
    let impl_tagged = container.gen_tagged();
    let ts = s.gen_impl(quote! {
        #impl_tryfrom
        #impl_tagged
    });
    eprintln!("{}", ts.to_string());
    ts
}

pub fn derive_der_sequence(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(s) => Container::from_datastruct(s),
        _ => panic!("Unsupported type, cannot derive"),
    };

    let impl_tryfrom = container.gen_tryfrom();
    let impl_tagged = container.gen_tagged();
    let impl_checkconstraints = container.gen_checkconstraints();
    let ts = s.gen_impl(quote! {
        #impl_tryfrom
        #impl_tagged
        #impl_checkconstraints
    });
    eprintln!("{}", ts.to_string());
    ts
}

pub struct Container {
    parse_content: TokenStream,
    field_names: Vec<Ident>,
}

impl Container {
    pub fn from_datastruct(s: &DataStruct) -> Self {
        let parse_content = derive_ber_sequence_content(s);
        let field_names = s.fields.iter().map(|f| f.ident.clone().unwrap()).collect();

        Container {
            parse_content,
            field_names,
        }
    }

    pub fn gen_tryfrom(&self) -> TokenStream {
        let parse_content = &self.parse_content;
        let field_names = &self.field_names;
        let lifetime = Lifetime::new("'ber", Span::call_site());
        quote! {
            extern crate asn1_rs;
            use asn1_rs::{Any, FromBer};
            use core::convert::TryFrom;

            gen impl<#lifetime> TryFrom<Any<#lifetime>> for @Self {
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
            gen impl asn1_rs::Tagged for @Self {
                const TAG: asn1_rs::Tag = asn1_rs::Tag::Sequence;
            }
        }
    }

    pub fn gen_checkconstraints(&self) -> TokenStream {
        quote! {
            gen impl asn1_rs::CheckDerConstraints for @Self {
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
