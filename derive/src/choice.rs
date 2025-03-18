use crate::check_derive::check_lastderive_fromber;
use crate::container::*;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, Ident, Lifetime};
use synstructure::VariantInfo;

struct TagVariant<'a, 'r> {
    tag: u32,
    vi: &'r VariantInfo<'a>,
}

pub fn derive_berparser_choice(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_berparser_choice_container(s, Asn1Type::Ber)
}

pub fn derive_derparser_choice(s: synstructure::Structure) -> proc_macro2::TokenStream {
    derive_berparser_choice_container(s, Asn1Type::Der)
}

pub fn derive_berparser_choice_container(
    s: synstructure::Structure,
    asn1_type: Asn1Type,
) -> proc_macro2::TokenStream {
    let parse_ber = asn1_type.parse_ber();
    let from_ber_content = asn1_type.from_ber_content();
    let parser = asn1_type.parser();
    let lft = Lifetime::new("'ber", Span::call_site());

    let ast = s.ast();

    if !matches!(&ast.data, Data::Enum(_)) {
        panic!("Unsupported type, cannot derive")
    };

    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });
    let tagged_implicit = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("tagged_implicit", Span::call_site()))
    });
    let tag_kind = if tagged_implicit {
        Asn1TagKind::Implicit
    } else {
        Asn1TagKind::Explicit
    };
    // get custom attributes on container
    let error_attr = ast
        .attrs
        .iter()
        .find(|attr| {
            attr.meta
                .path()
                .is_ident(&Ident::new("error", Span::call_site()))
        })
        .cloned();
    let last_berderive = check_lastderive_fromber(ast);

    // counter for auto-assignement of tag values (if not specified)
    let mut current_tag: u32 = 0;
    let variants: Vec<_> = s
        .variants()
        .iter()
        .map(|vi| {
            // eprintln!("variant {current_tag} info: {vi:?}");

            // check that variant is a single type

            let tag_variant = TagVariant {
                tag: current_tag,
                vi,
            };

            current_tag += 1;

            tag_variant
        })
        .collect();

    let tag_branches = variants.iter().map(|v| {
        let pat = v.vi.pat();
        let tag = v.tag;
        quote! { #pat => asn1_rs::Tag(#tag),  }
    });

    let parse_branches = variants.iter().map(|v| {
        let bindings = v.vi.bindings();
        if bindings.len() != 1 {
            panic!("Enum/CHOICE: only variants with one unnamed binding are supported now");
        }
        let tag = v.tag;
        let bi = &bindings[0];
        let construct = v.vi.construct(|_, _i| bi);
        match tag_kind {
            Asn1TagKind::Explicit => quote! {
                #tag => {
                    let (rem, #bi) = #parser::#parse_ber(rem)?;
                    Ok((rem, #construct))
                }
            },
            Asn1TagKind::Implicit => quote! {
                #tag => {
                    let (rem, #bi) = #parser::#from_ber_content(header, rem)?;
                    Ok((rem, #construct))
                }
            },
        }
    });
    let assert_constructed = match tag_kind {
        Asn1TagKind::Explicit => quote! {
            header.assert_constructed_input(&input).map_err(Err::Error)?;
        },
        Asn1TagKind::Implicit => quote! {},
    };

    // error type
    let error = if let Some(attr) = &error_attr {
        get_attribute_meta(attr).expect("Invalid error attribute format")
    } else {
        quote! { asn1_rs::BerError<asn1_rs::Input<#lft>> }
    };

    // generate DynTagged, only if last implementation
    let dyntagged = if last_berderive {
        s.gen_impl(quote! {
            gen impl asn1_rs::DynTagged for @Self {
                fn accept_tag(_: asn1_rs::Tag) -> bool { true }

                fn tag(&self) -> asn1_rs::Tag {
                    match self {
                        #(#tag_branches)*
                    }
                }
            }
        })
    } else {
        quote! {}
    };

    let ber_parser = s.gen_impl(quote! {
        extern crate asn1_rs;

        gen impl<#lft> asn1_rs::#parser<#lft> for @Self {
            type Error = #error;
            fn #from_ber_content(header: &'_ Header<#lft>, input: Input<#lft>) -> IResult<Input<#lft>, Self, Self::Error> {
                #assert_constructed
                let rem = input.clone();
                match header.tag().0 {
                    #(#parse_branches)*
                    _ => {
                        return Err(asn1_rs::nom::Err::Error(
                            asn1_rs::BerError::unexpected_tag(input, None, header.tag()).into()
                        ));
                    }
                }
            }
        }
    });

    let ts = quote! {
        #dyntagged
        #ber_parser
    };
    if debug_derive {
        eprintln!("{}", ts);
    }
    ts
}
