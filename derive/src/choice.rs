use crate::asn1_type::Asn1Type;
use crate::check_derive::check_lastderive_fromber;
use crate::container::*;
use crate::options::Options;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Attribute, Data, Error, Ident, Lifetime, LitInt, Result, WherePredicate};
use synstructure::VariantInfo;

pub fn derive_choice(s: synstructure::Structure) -> TokenStream {
    match DeriveChoice::new(&s) {
        Ok(s) => s.to_tokens(),
        Err(e) => e.to_compile_error().into(),
    }
}

pub struct DeriveChoice<'s> {
    options: Options,

    ident: Ident,
    synstruct: &'s synstructure::Structure<'s>,
    variants: Vec<TagVariant<'s, 's>>,
    error: Option<Attribute>,
}

impl<'s> DeriveChoice<'s> {
    fn new(s: &'s synstructure::Structure<'_>) -> Result<Self> {
        let ast = s.ast();
        if !matches!(&ast.data, Data::Enum(_)) {
            return Err(Error::new_spanned(
                &ast.ident,
                "'Choice' can only be derived on `enum` type",
            ));
        };

        // get custom attributes on container
        let error = ast
            .attrs
            .iter()
            .find(|attr| {
                attr.meta
                    .path()
                    .is_ident(&Ident::new("error", Span::call_site()))
            })
            .cloned();

        let ident = ast.ident.clone();
        let options = Options::from_struct(&s)?;
        let variants = parse_tag_variants(&s)?;

        let s = Self {
            options,
            ident,
            synstruct: s,
            variants,
            error,
        };
        Ok(s)
    }

    fn to_tokens(&self) -> TokenStream {
        let dyntagged = self.derive_choice_dyntagged();
        let berparser = self.derive_choice_parser(Asn1Type::Ber);
        let derparser = self.derive_choice_parser(Asn1Type::Der);
        let berencode = self.derive_choice_encode(Asn1Type::Ber);
        let derencode = self.derive_choice_encode(Asn1Type::Der);

        let ts = quote! {
            #dyntagged
            #berparser
            #derparser
            #berencode
            #derencode
        };

        if self.options.debug {
            eprintln!("// CHOICE for {}", self.ident);
            eprintln!("{}", ts);
        }
        ts
    }

    fn derive_choice_dyntagged(&self) -> TokenStream {
        let accept_tags = {
            match self.options.tag_kind {
                Some(_) => {
                    let tags = self.variants.iter().map(|v| v.tag);
                    quote! {
                        [#(#tags),*].contains(&tag.0)
                    }
                }
                None => {
                    let accept_tags_if_else = self.variants.iter().map(|v| {
                        let bindings = v.vi.bindings();
                        if bindings.len() != 1 {
                            panic!(
                                "Enum/CHOICE: only variants with one unnamed binding are supported now"
                            );
                        }
                        let bi = &bindings[0];
                        let ty = &bi.ast().ty;
                        quote! {
                            if <#ty>::accept_tag(tag) {
                                true
                            } else
                        }
                    });

                    quote! {
                        #(#accept_tags_if_else)* { false }
                    }
                }
            }
        };
        let class = match self.options.tag_kind {
            Some(_) => quote! { asn1_rs::Class::ContextSpecific },
            None => {
                // more complex answer: depends on variant/binding
                let class_branches = self.variants.iter().map(|v| {
                    let pat = v.vi.pat();
                    let bi = &v.vi.bindings()[0];
                    quote! { #pat => #bi.class(),  }
                });
                quote! {
                    match self {
                        #(#class_branches)*
                    }
                }
            }
        };
        let constructed = match self.options.tag_kind {
            Some(Asn1TagKind::Explicit) => quote! { true },
            Some(Asn1TagKind::Implicit) | None => {
                // more complex answer: depends on variant/binding
                let constructed_branches = self.variants.iter().map(|v| {
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
        let tag = {
            let tag_branches = self.variants.iter().map(|v| {
                let pat = v.vi.pat();
                match self.options.tag_kind {
                    Some(_) => {
                        let tag = v.tag;
                        quote! { #pat => asn1_rs::Tag(#tag),  }
                    }
                    None => {
                        let bi = &v.vi.bindings()[0];
                        quote! { #pat => #bi.tag(),  }
                    }
                }
            });
            quote! {
                match self {
                    #(#tag_branches)*
                }
            }
        };

        self.synstruct.gen_impl(quote! {
            gen impl asn1_rs::DynTagged for @Self {
                fn accept_tag(tag: asn1_rs::Tag) -> bool { #accept_tags }

                fn class(&self) -> asn1_rs::Class { #class }

                fn constructed(&self) -> bool { #constructed }

                fn tag(&self) -> asn1_rs::Tag { #tag }
            }
        })
    }

    fn derive_choice_parser(&self, asn1_type: Asn1Type) -> TokenStream {
        match self.options.tag_kind {
            Some(tag_kind) => self.derive_choice_parser_tagged(tag_kind, asn1_type),
            None => self.derive_choice_parser_untagged(asn1_type),
        }
    }

    fn derive_choice_parser_tagged(
        &self,
        tag_kind: Asn1TagKind,
        asn1_type: Asn1Type,
    ) -> TokenStream {
        if !self.options.parsers.contains(&asn1_type) {
            if self.options.debug {
                eprintln!("// Parsers: skipping asn1_type {:?}", asn1_type);
            }
            return quote! {};
        }

        let parse_ber = asn1_type.parse_ber();
        let from_ber_content = asn1_type.from_ber_content();
        let parser = asn1_type.parser();
        let lft = Lifetime::new("'ber", Span::call_site());

        // if using custom error, we need to map errors before return
        let map_err = self
            .error
            .as_ref()
            .map(|_| quote! { .map_err(asn1_rs::nom::Err::convert) });

        let parse_branches = self.variants.iter().map(|v| {
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
                        let (rem, #bi) = #parser::#parse_ber(rem)#map_err?;
                        Ok((rem, #construct))
                    }
                },
                Asn1TagKind::Implicit => quote! {
                    #tag => {
                        let (rem, #bi) = #parser::#from_ber_content(header, rem)#map_err?;
                        Ok((rem, #construct))
                    }
                },
            }
        });
        let assert_constructed = match tag_kind {
            Asn1TagKind::Explicit => quote! {
                header.assert_constructed_input(&input).map_err(|e| Err::convert(Err::Error(e)))?;
            },
            Asn1TagKind::Implicit => quote! {},
        };

        // error type
        let error = if let Some(attr) = &self.options.error {
            get_attribute_meta(attr).expect("Invalid error attribute format")
        } else {
            quote! { asn1_rs::BerError<asn1_rs::Input<#lft>> }
        };

        // Note: if Self has lifetime bounds, then a new bound must be added to the implementation
        // For ex: `pub struct AA<'a>` will require a bound `impl[..] DerParser[..] where 'i: 'a`
        // get lifetimes from generics
        let lfts: Vec<_> = self.synstruct.ast().generics.lifetimes().collect();
        let mut where_predicates = Vec::new();
        if !lfts.is_empty() {
            // input slice must outlive all lifetimes from Self
            let wh: WherePredicate = parse_quote! { #lft: #(#lfts)+* };
            where_predicates.push(wh);
        };

        self.synstruct.gen_impl(quote! {
                extern crate asn1_rs;

                gen impl<#lft> asn1_rs::#parser<#lft> for @Self where #(#where_predicates)+* {
                    type Error = #error;
                    fn #from_ber_content(header: &'_ asn1_rs::Header<#lft>, input: asn1_rs::Input<#lft>) -> asn1_rs::nom::IResult<asn1_rs::Input<#lft>, Self, Self::Error> {
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

    fn derive_choice_parser_untagged(&self, asn1_type: Asn1Type) -> TokenStream {
        if !self.options.parsers.contains(&asn1_type) {
            if self.options.debug {
                eprintln!("// Parsers: skipping asn1_type {:?}", asn1_type);
            }
            return quote! {};
        }

        let from_ber_content = asn1_type.from_ber_content();
        let parser = asn1_type.parser();
        let lft = Lifetime::new("'ber", Span::call_site());

        // if using custom error, we need to map errors before return
        let map_err = self
            .error
            .as_ref()
            .map(|_| quote! { .map_err(asn1_rs::nom::Err::convert) });

        let parse_branches_if_else = self.variants.iter().map(|v| {
            let bindings = v.vi.bindings();
            if bindings.len() != 1 {
                panic!("Enum/CHOICE: only variants with one unnamed binding are supported now");
            }
            let bi = &bindings[0];
            let construct = v.vi.construct(|_, _i| bi);
            let ty = &bi.ast().ty;
            quote! {
                if <#ty>::accept_tag(header.tag()) {
                    let (rem, #bi) = asn1_rs::#parser::#from_ber_content(header, rem)#map_err?;
                    Ok((rem, #construct))
                } else
            }
        });

        // error type
        let error = if let Some(attr) = &self.error {
            get_attribute_meta(attr).expect("Invalid error attribute format")
        } else {
            quote! { asn1_rs::BerError<asn1_rs::Input<#lft>> }
        };

        // Note: if Self has lifetime bounds, then a new bound must be added to the implementation
        // For ex: `pub struct AA<'a>` will require a bound `impl[..] DerParser[..] where 'i: 'a`
        // get lifetimes from generics
        let lfts: Vec<_> = self.synstruct.ast().generics.lifetimes().collect();
        let mut where_predicates = Vec::new();
        if !lfts.is_empty() {
            // input slice must outlive all lifetimes from Self
            let wh: WherePredicate = parse_quote! { #lft: #(#lfts)+* };
            where_predicates.push(wh);
        };

        self.synstruct.gen_impl(quote! {
                extern crate asn1_rs;

                gen impl<#lft> asn1_rs::#parser<#lft> for @Self where #(#where_predicates)+* {
                    type Error = #error;
                    fn #from_ber_content(header: &'_ asn1_rs::Header<#lft>, input: asn1_rs::Input<#lft>) -> asn1_rs::nom::IResult<asn1_rs::Input<#lft>, Self, Self::Error> {
                        // #assert_constructed
                        use asn1_rs::DynTagged;
                        let rem = input.clone();
                        #(#parse_branches_if_else)*
                        {
                            return Err(asn1_rs::nom::Err::Error(
                                asn1_rs::BerError::unexpected_tag(input, None, header.tag()).into()
                            ));
                        }
                    }
                }
            })
    }

    fn derive_choice_encode(&self, asn1_type: Asn1Type) -> TokenStream {
        if !self.options.encoders.contains(&asn1_type) {
            if self.options.debug {
                eprintln!("// Encoders: skipping asn1_type {:?}", asn1_type);
            }
            return quote! {};
        }

        let tober = asn1_type.tober();

        let impl_tober_content_len = self.choice_gen_tober_content_len(asn1_type);
        let impl_tober_tag_info = self.choice_gen_tober_tag_info(asn1_type);
        let impl_tober_write_content = self.choice_gen_tober_write_content(asn1_type);

        self.synstruct.gen_impl(quote! {
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

    fn choice_gen_tober_content_len(&self, asn1_type: Asn1Type) -> TokenStream {
        let content_len = asn1_type.content_len_tokens();
        let total_len = asn1_type.total_len_tokens();

        // NOTE: fold() only works on variants with bindings, but we know that it is the case
        let content_len_branches = self.synstruct.fold(quote! {}, |acc, bi| {
            let instrs = match self.options.tag_kind {
                Some(Asn1TagKind::Explicit) => quote! { #bi.#total_len() },
                Some(Asn1TagKind::Implicit) | None => quote! { #bi.#content_len() },
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

    fn choice_gen_tober_tag_info(&self, asn1_type: Asn1Type) -> TokenStream {
        let tag_info = asn1_type.tag_info_tokens();

        let impl_tober_tag_info = quote! {
            fn #tag_info(&self) -> (asn1_rs::Class, bool, asn1_rs::Tag) {
                use asn1_rs::DynTagged;
                (self.class(), self.constructed(), self.tag())
            }
        };
        impl_tober_tag_info
    }

    fn choice_gen_tober_write_content(&self, asn1_type: Asn1Type) -> TokenStream {
        let encode = asn1_type.compose("_encode");
        let write_content = asn1_type.compose("_write_content");

        let write_branches = self.variants.iter().map(|v| {
            let bindings = v.vi.bindings();
            if bindings.len() != 1 {
                panic!("Enum/CHOICE: only variants with one unnamed binding are supported now");
            }
            let pat = v.vi.pat();
            let bi = &bindings[0];
            match self.options.tag_kind {
                Some(Asn1TagKind::Explicit) => quote! {
                    #pat => {
                        // encode as tagged explicit (write full object)
                        #bi.#encode(writer)
                    }
                },
                Some(Asn1TagKind::Implicit) | None => quote! {
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
}

//--- old-style derive

pub fn derive_berparser_choice(s: synstructure::Structure) -> Result<TokenStream> {
    derive_berparser_choice_container(s, Asn1Type::Ber)
}

pub fn derive_derparser_choice(s: synstructure::Structure) -> Result<TokenStream> {
    derive_berparser_choice_container(s, Asn1Type::Der)
}

pub fn derive_berparser_choice_container(
    s: synstructure::Structure,
    asn1_type: Asn1Type,
) -> Result<TokenStream> {
    let ast = s.ast();

    let derive = DeriveChoice::new(&s)?;

    let last_berderive = check_lastderive_fromber(ast);

    let dyntagged = if last_berderive {
        derive.derive_choice_dyntagged()
    } else {
        quote! {}
    };
    let berparser = derive.derive_choice_parser(asn1_type);

    let ts = quote! {
        #dyntagged
        #berparser
    };
    if derive.options.debug {
        eprintln!("{}", ts);
    }
    Ok(ts)
}

struct TagVariant<'a, 'r> {
    tag: u32,
    vi: &'r VariantInfo<'a>,
}

fn parse_tag_variants<'a, 'r>(
    s: &'r synstructure::Structure<'a>,
) -> Result<Vec<TagVariant<'a, 'r>>> {
    // check that all variants has a single type (binding)
    for v in s.variants() {
        if v.bindings().len() != 1 {
            return Err(Error::new_spanned(
                &v.ast().ident,
                "'Choice': only variants with one unnamed binding are supported",
            ));
        }
    }

    // counter for auto-assignment of tag values (if not specified)
    let mut current_tag: u32 = 0;
    let v = s
        .variants()
        .iter()
        .try_fold(Vec::new(), |mut acc, vi| -> Result<Vec<_>> {
            // eprintln!("variant {current_tag} info: {vi:?}");

            let tag = match get_variant_tag(vi)? {
                Some(tag) => {
                    // if a tag was provided, update counter to use it from now
                    current_tag = tag;
                    tag
                }
                None => current_tag,
            };
            let tag_variant = TagVariant { tag, vi };
            // before inserting, check for tags uniqueness
            if acc.iter().any(|tv: &TagVariant<'_, '_>| tv.tag == tag) {
                return Err(Error::new_spanned(
                    &vi.ast().ident,
                    "'Choice': duplicate tag found",
                ));
            }
            acc.push(tag_variant);

            current_tag += 1;

            Ok(acc)
        })?;

    Ok(v)
}

/// Check attributes for 'tag' and use tag if provided
fn get_variant_tag(vi: &VariantInfo<'_>) -> Result<Option<u32>> {
    for attr in vi.ast().attrs {
        let path = attr.meta.path();
        if path.is_ident("tag") {
            let lit: LitInt = attr.parse_args()?;
            let tag_number = lit.base10_parse::<u32>()?;
            return Ok(Some(tag_number));
        }
    }

    Ok(None)
}
