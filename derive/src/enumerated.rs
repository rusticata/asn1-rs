use crate::asn1_type::Asn1Type;
use crate::container::*;
use crate::options::Options;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, Error, Expr, ExprLit, Ident, Lifetime, Lit, Result};

pub fn derive_enumerated(s: synstructure::Structure) -> TokenStream {
    match DeriveEnumerated::new(&s) {
        Ok(s) => s.to_tokens(),
        Err(e) => e.to_compile_error().into(),
    }
}

pub struct DeriveEnumerated<'s> {
    options: Options,

    ident: Ident,
    synstruct: &'s synstructure::Structure<'s>,
    variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    ident: Ident,
    discriminant: u32,
}

impl<'s> DeriveEnumerated<'s> {
    pub fn new(s: &'s synstructure::Structure<'s>) -> Result<Self> {
        let ast = s.ast();
        if !matches!(&ast.data, Data::Enum(_)) {
            return Err(Error::new_spanned(
                &ast.ident,
                "'Enumerated' can only be derived on `enum` type",
            ));
        };

        // TODO: check that enum has 'repr' attribute for some unsigned integer class

        let ident = ast.ident.clone();
        let options = Options::from_struct(s)?;
        let variants = parse_enum_variants(s)?;

        let s = Self {
            options,
            ident,
            synstruct: s,
            variants,
        };
        Ok(s)
    }

    fn to_tokens(&self) -> TokenStream {
        let Self {
            variants,
            options,
            synstruct,
            ..
        } = self;

        let dyntagged = derive_enumerated_dyntagged(variants, options, synstruct);
        let berparser = derive_enumerated_parser(Asn1Type::Ber, variants, options, synstruct);
        let derparser = derive_enumerated_parser(Asn1Type::Der, variants, options, synstruct);
        let berencode = derive_enumerated_encode(Asn1Type::Ber, variants, options, synstruct);
        let derencode = derive_enumerated_encode(Asn1Type::Der, variants, options, synstruct);

        let ts = quote! {
            #dyntagged
            #berparser
            #derparser
            #berencode
            #derencode
        };

        if self.options.debug {
            eprintln!("// ENUMERATED for {}", self.ident);
            eprintln!("{}", ts);
        }
        ts
    }
}

fn derive_enumerated_dyntagged(
    _variants: &[EnumVariant],
    _options: &Options,
    s: &synstructure::Structure,
) -> TokenStream {
    s.gen_impl(quote! {
        gen impl asn1_rs::DynTagged for @Self {
            fn accept_tag(tag: asn1_rs::Tag) -> bool { tag == asn1_rs::Tag::Enumerated }

            fn class(&self) -> asn1_rs::Class { asn1_rs::Class::Universal }

            fn constructed(&self) -> bool { false }

            fn tag(&self) -> asn1_rs::Tag { asn1_rs::Tag::Enumerated }
        }
    })
}

fn derive_enumerated_parser(
    asn1_type: Asn1Type,
    variants: &[EnumVariant],
    options: &Options,
    s: &synstructure::Structure,
) -> TokenStream {
    if !options.parsers.contains(&asn1_type) {
        if options.debug {
            eprintln!("// Parsers: skipping asn1_type {:?}", asn1_type);
        }
        return quote! {};
    }

    let from_ber_content = asn1_type.from_ber_content();
    let parser = asn1_type.parser();
    let lft = Lifetime::new("'ber", Span::call_site());

    let match_branches = variants.iter().map(|v| {
        //
        let discriminant = v.discriminant;
        let ident = &v.ident;
        quote! { #discriminant => Self::#ident, }
    });

    let assert_primitive = quote! {
        header.assert_primitive_input(&input).map_err(asn1_rs::nom::Err::Error)?;
    };

    // error type
    let error = if let Some(attr) = &options.error {
        get_attribute_meta(attr).expect("Invalid error attribute format")
    } else {
        quote! { asn1_rs::BerError<asn1_rs::Input<#lft>> }
    };

    s.gen_impl(quote! {
        extern crate asn1_rs;

        gen impl<#lft> asn1_rs::#parser<#lft> for @Self {
            type Error = #error;
            fn #from_ber_content(header: &'_ asn1_rs::Header<#lft>, input: asn1_rs::Input<#lft>) -> asn1_rs::nom::IResult<asn1_rs::Input<#lft>, Self, Self::Error> {
                #assert_primitive
                // let rem = input.clone();
                let (rem, enumerated) = asn1_rs::Enumerated::#from_ber_content(header, input.clone())?;
                let v = match enumerated.0 {
                    #(#match_branches)*
                    _ => {
                        return Err(asn1_rs::nom::Err::Error(
                            asn1_rs::BerError::unexpected_tag(input, None, header.tag()).into()
                        ));
                    }
                };
                Ok((rem, v))
            }
        }
    })
}

fn derive_enumerated_encode(
    asn1_type: Asn1Type,
    variants: &[EnumVariant],
    options: &Options,
    s: &synstructure::Structure,
) -> TokenStream {
    if !options.encoders.contains(&asn1_type) {
        if options.debug {
            eprintln!("// Encoders: skipping asn1_type {:?}", asn1_type);
        }
        return quote! {};
    }

    let tober = asn1_type.tober();

    let impl_tober_content_len = enumerated_gen_tober_content_len(asn1_type, options, s);
    let impl_tober_tag_info = enumerated_gen_tober_tag_info(asn1_type, variants, options, s);
    let impl_tober_write_content =
        enumerated_gen_tober_write_content(asn1_type, variants, options, s);

    s.gen_impl(quote! {
        extern crate asn1_rs;

        #[cfg(feature = "std")]
        gen impl asn1_rs::#tober for @Self {
            type Encoder = asn1_rs::BerGenericEncoder;

            #impl_tober_content_len
            #impl_tober_tag_info
            #impl_tober_write_content
        }
    })
}

fn enumerated_gen_tober_content_len(
    asn1_type: Asn1Type,
    _options: &Options,
    _s: &synstructure::Structure<'_>,
) -> TokenStream {
    let content_len = asn1_type.content_len_tokens();

    let impl_tober_content_len = quote! {
        fn #content_len(&self) -> asn1_rs::Length {
            let e = asn1_rs::Enumerated::new(*self as u32);
            e.#content_len()
        }
    };
    impl_tober_content_len
}

fn enumerated_gen_tober_tag_info(
    asn1_type: Asn1Type,
    _variants: &[EnumVariant],
    _options: &Options,
    _s: &synstructure::Structure<'_>,
) -> TokenStream {
    let tag_info = asn1_type.tag_info_tokens();

    let impl_tober_tag_info = quote! {
        fn #tag_info(&self) -> (asn1_rs::Class, bool, asn1_rs::Tag) {
            use asn1_rs::DynTagged;
            (self.class(), self.constructed(), self.tag())
        }
    };
    impl_tober_tag_info
}

fn enumerated_gen_tober_write_content(
    asn1_type: Asn1Type,
    _variants: &[EnumVariant],
    _options: &Options,
    _s: &synstructure::Structure<'_>,
) -> TokenStream {
    let write_content = asn1_type.compose("_write_content");

    let impl_tober_write_content = quote! {
        fn #write_content<W: std::io::Write>(&self, writer: &mut W) -> asn1_rs::SerializeResult<usize> {
            let e = asn1_rs::Enumerated::new(*self as u32);
            e.#write_content(writer)
        }
    };
    impl_tober_write_content
}

fn parse_enum_variants(s: &synstructure::Structure<'_>) -> Result<Vec<EnumVariant>> {
    let mut current_value = 0u32;
    let v = s
        .variants()
        .iter()
        .try_fold(Vec::new(), |mut acc, vi| -> Result<_> {
            //
            let ident = vi.ast().ident.clone();
            let discriminant = vi.ast().discriminant;

            if let Some((_eq, expr)) = discriminant {
                match expr {
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(d), ..
                    }) => {
                        let discriminant = d.base10_parse::<u32>()?;
                        current_value = discriminant;
                    }
                    _ => {
                        return Err(Error::new_spanned(
                            &expr,
                            "'Enumerated': only integer literals are supported",
                        ))
                    }
                }
            };
            let discriminant = current_value;
            let e = EnumVariant {
                ident,
                discriminant,
            };
            acc.push(e);

            current_value += 1;
            Ok(acc)
        })?;
    Ok(v)
}
