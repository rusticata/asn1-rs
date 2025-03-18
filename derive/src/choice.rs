use crate::check_derive::check_lastderive_fromber;
use crate::container::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, Data, Ident, Lifetime};
use synstructure::VariantInfo;

#[derive(Default)]
struct ChoiceOptions {
    debug: bool,
    error: Option<Attribute>,
    tag_kind: Asn1TagKind,
}

impl ChoiceOptions {
    pub fn from_struct(s: &synstructure::Structure) -> Self {
        let mut options = Self::default();
        let ast = s.ast();
        let ident_debug = Ident::new("debug_derive", Span::call_site());
        let ident_explicit = Ident::new("tagged_explicit", Span::call_site());
        let ident_implicit = Ident::new("tagged_implicit", Span::call_site());
        let ident_error = Ident::new("tagged_error", Span::call_site());

        for attr in ast.attrs.iter() {
            let path = attr.meta.path();
            if path.is_ident(&ident_debug) {
                options.debug = true;
            } else if path.is_ident(&ident_explicit) {
                options.tag_kind = Asn1TagKind::Explicit;
            } else if path.is_ident(&ident_implicit) {
                options.tag_kind = Asn1TagKind::Implicit;
            } else if path.is_ident(&ident_error) {
                options.error = Some(attr.clone());
            }
        }

        options
    }
}

struct TagVariant<'a, 'r> {
    tag: u32,
    vi: &'r VariantInfo<'a>,
}

fn parse_tag_variants<'a, 'r>(s: &'r synstructure::Structure<'a>) -> Vec<TagVariant<'a, 'r>> {
    // counter for auto-assignement of tag values (if not specified)
    let mut current_tag: u32 = 0;
    s.variants()
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
        .collect()
}

pub fn derive_choice(s: synstructure::Structure) -> TokenStream {
    let ast = s.ast();
    if !matches!(&ast.data, Data::Enum(_)) {
        panic!("Unsupported type, cannot derive")
    };

    let options = ChoiceOptions::from_struct(&s);

    let variants = parse_tag_variants(&s);

    let dyntagged = derive_choice_dyntagged(&variants, &options, &s);
    let berparser = derive_choice_parser(Asn1Type::Ber, &variants, &options, &s);
    let derparser = derive_choice_parser(Asn1Type::Der, &variants, &options, &s);
    let berencode = derive_choice_encode(Asn1Type::Ber, &variants, &options, &s);
    let derencode = derive_choice_encode(Asn1Type::Der, &variants, &options, &s);

    let ts = quote! {
        #dyntagged
        #berparser
        #derparser
        #berencode
        #derencode
    };
    if options.debug {
        eprintln!("// CHOICE for {}", ast.ident);
        eprintln!("{}", ts);
    }
    ts
}

fn derive_choice_dyntagged(
    variants: &[TagVariant],
    options: &ChoiceOptions,
    s: &synstructure::Structure,
) -> TokenStream {
    let constructed = match options.tag_kind {
        Asn1TagKind::Explicit => quote! { true },
        Asn1TagKind::Implicit => {
            // more complex answer: depends on variant/binding
            let constructed_branches = variants.iter().map(|v| {
                let pat = v.vi.pat();
                let bi = &v.vi.bindings()[0];
                quote! { #pat => #bi.constructed(),  }
            });
            quote! {
                match self {
                    #(#constructed_branches)*
                }
            }
        }
    };
    let tag_branches = variants.iter().map(|v| {
        let pat = v.vi.pat();
        let tag = v.tag;
        quote! { #pat => asn1_rs::Tag(#tag),  }
    });

    s.gen_impl(quote! {
        gen impl asn1_rs::DynTagged for @Self {
            fn accept_tag(_: asn1_rs::Tag) -> bool { true }

            fn class(&self) -> asn1_rs::Class { Class::ContextSpecific }

            fn constructed(&self) -> bool {
                #constructed
            }

            fn tag(&self) -> asn1_rs::Tag {
                match self {
                    #(#tag_branches)*
                }
            }
        }
    })
}

fn derive_choice_parser(
    asn1_type: Asn1Type,
    variants: &[TagVariant],
    options: &ChoiceOptions,
    s: &synstructure::Structure,
) -> TokenStream {
    let parse_ber = asn1_type.parse_ber();
    let from_ber_content = asn1_type.from_ber_content();
    let parser = asn1_type.parser();
    let lft = Lifetime::new("'ber", Span::call_site());

    let parse_branches = variants.iter().map(|v| {
        let bindings = v.vi.bindings();
        if bindings.len() != 1 {
            panic!("Enum/CHOICE: only variants with one unnamed binding are supported now");
        }
        let tag = v.tag;
        let bi = &bindings[0];
        let construct = v.vi.construct(|_, _i| bi);
        match options.tag_kind {
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
    let assert_constructed = match options.tag_kind {
        Asn1TagKind::Explicit => quote! {
            header.assert_constructed_input(&input).map_err(Err::Error)?;
        },
        Asn1TagKind::Implicit => quote! {},
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
    })
}

fn derive_choice_encode(
    asn1_type: Asn1Type,
    variants: &[TagVariant],
    options: &ChoiceOptions,
    s: &synstructure::Structure,
) -> TokenStream {
    let tober = asn1_type.tober();

    let impl_tober_content_len = choice_gen_tober_content_len(asn1_type, options, s);
    let impl_tober_tag_info = choice_gen_tober_tag_info(asn1_type, variants, options, s);
    let impl_tober_write_content = choice_gen_tober_write_content(asn1_type, variants, options, s);

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

fn choice_gen_tober_content_len(
    asn1_type: Asn1Type,
    options: &ChoiceOptions,
    s: &synstructure::Structure<'_>,
) -> TokenStream {
    let content_len = asn1_type.content_len_tokens();
    let total_len = asn1_type.total_len_tokens();

    // NOTE: fold() only works on variants with bindings, but we know that it is the case
    let content_len_branches = s.fold(quote! {}, |acc, bi| {
        let instrs = match options.tag_kind {
            Asn1TagKind::Explicit => quote! { #bi.#total_len() },
            Asn1TagKind::Implicit => quote! { #bi.#content_len() },
        };
        quote! { #acc #instrs }
    });
    let impl_tober_content_len = quote! {
        fn #content_len(&self) -> asn1_rs::Length {
            match self {
                #content_len_branches
            }
        }
    };
    impl_tober_content_len
}

fn choice_gen_tober_tag_info(
    asn1_type: Asn1Type,
    _variants: &[TagVariant],
    _options: &ChoiceOptions,
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

fn choice_gen_tober_write_content(
    asn1_type: Asn1Type,
    variants: &[TagVariant],
    options: &ChoiceOptions,
    _s: &synstructure::Structure<'_>,
) -> TokenStream {
    let encode = asn1_type.compose("_encode");
    let write_content = asn1_type.compose("_write_content");

    let write_branches = variants.iter().map(|v| {
        let bindings = v.vi.bindings();
        if bindings.len() != 1 {
            panic!("Enum/CHOICE: only variants with one unnamed binding are supported now");
        }
        let pat = v.vi.pat();
        let bi = &bindings[0];
        match options.tag_kind {
            Asn1TagKind::Explicit => quote! {
                #pat => {
                    // encode as tagged explicit (write full object)
                    #bi.#encode(writer)
                }
            },
            Asn1TagKind::Implicit => quote! {
                #pat => {
                    // encode as tagged implicit (write only content)
                    #bi.#write_content(writer)
                }
            },
        }
    });

    let impl_tober_write_content = quote! {
        fn #write_content<W: std::io::Write>(&self, writer: &mut W) -> asn1_rs::SerializeResult<usize> {
                match self {
                    #(#write_branches)*
                }
            }
    };
    impl_tober_write_content
}

//--- old-style derive

pub fn derive_berparser_choice(s: synstructure::Structure) -> TokenStream {
    derive_berparser_choice_container(s, Asn1Type::Ber)
}

pub fn derive_derparser_choice(s: synstructure::Structure) -> TokenStream {
    derive_berparser_choice_container(s, Asn1Type::Der)
}

pub fn derive_berparser_choice_container(
    s: synstructure::Structure,
    asn1_type: Asn1Type,
) -> TokenStream {
    let ast = s.ast();

    if !matches!(&ast.data, Data::Enum(_)) {
        panic!("Unsupported type, cannot derive")
    };

    let options = ChoiceOptions::from_struct(&s);

    let variants = parse_tag_variants(&s);

    let last_berderive = check_lastderive_fromber(ast);

    let dyntagged = if last_berderive {
        derive_choice_dyntagged(&variants, &options, &s)
    } else {
        quote! {}
    };
    let berparser = derive_choice_parser(asn1_type, &variants, &options, &s);

    let ts = quote! {
        #dyntagged
        #berparser
    };
    if options.debug {
        eprintln!("{}", ts);
    }
    ts
}
