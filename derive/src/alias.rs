use crate::asn1_type::Asn1Type;
use crate::check_derive::check_lastderive_fromber;
use crate::container::*;
use crate::options::Options;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, Attribute, Data, DataStruct, Error, Fields, Ident, Lifetime, Result,
    WherePredicate,
};
use synstructure::BindingInfo;

pub fn derive_alias(s: synstructure::Structure) -> TokenStream {
    match DeriveAlias::new(&s) {
        Ok(s) => s.to_tokens(),
        Err(e) => e.to_compile_error().into(),
    }
}

pub struct DeriveAlias<'s> {
    options: Options,

    ident: Ident,
    synstruct: &'s synstructure::Structure<'s>,
    target: &'s BindingInfo<'s>,
    error: Option<Attribute>,
    where_predicates: Vec<WherePredicate>,
}

impl<'s> DeriveAlias<'s> {
    pub fn new(s: &'s synstructure::Structure<'s>) -> Result<Self> {
        let err_msg = "'Alias' can only be derived on anonymous `struct` type with one field";

        let ast = s.ast();
        if !matches!(
            &ast.data,
            Data::Struct(DataStruct {
                fields: Fields::Unnamed(_),
                ..
            })
        ) {
            return Err(Error::new_spanned(&ast.ident, err_msg));
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

        // get lifetimes from generics
        let lfts: Vec<_> = s.ast().generics.lifetimes().collect();
        let mut where_predicates = Vec::new();
        if !lfts.is_empty() {
            // input slice must outlive all lifetimes from Self
            let lft = Lifetime::new("'ber", Span::call_site());
            let pred: WherePredicate = parse_quote! { #lft: #(#lfts)+* };
            where_predicates.push(pred);
        };

        let ident = ast.ident.clone();
        let options = Options::from_struct(s)?;
        // check that we have a tuple struct with exactly one field
        let variants = s.variants();
        if variants.len() != 1 {
            return Err(Error::new_spanned(&ast.ident, err_msg));
        }
        let vi = &variants[0];
        if vi.bindings().len() != 1 {
            return Err(Error::new_spanned(&vi.ast().ident, err_msg));
        }
        let target = &vi.bindings()[0];

        let s = Self {
            options,
            ident,
            synstruct: s,
            target,
            error,
            where_predicates,
        };
        Ok(s)
    }

    fn to_tokens(&self) -> TokenStream {
        let dyntagged = self.derive_alias_dyntagged();
        let berparser = self.derive_alias_parser(Asn1Type::Ber);
        let derparser = self.derive_alias_parser(Asn1Type::Der);
        let berencode = self.derive_alias_encode(Asn1Type::Ber);
        let derencode = self.derive_alias_encode(Asn1Type::Der);

        let ts = quote! {
            #dyntagged
            #berparser
            #derparser
            #berencode
            #derencode
        };

        if self.options.debug {
            eprintln!("// ALIAS for {}", self.ident);
            eprintln!("{}", ts);
        }
        ts
    }

    fn derive_alias_dyntagged(&self) -> TokenStream {
        let ty = &self.target.ast().ty;

        self.synstruct.gen_impl(quote! {
            use asn1_rs::DynTagged;

            gen impl asn1_rs::DynTagged for @Self {
                fn accept_tag(tag: asn1_rs::Tag) -> bool { <#ty as asn1_rs::DynTagged>::accept_tag(tag) }

                fn class(&self) -> asn1_rs::Class { self.0.class() }

                fn constructed(&self) -> bool { self.0.constructed() }

                fn tag(&self) -> asn1_rs::Tag { self.0.tag() }
            }
        })
    }

    fn derive_alias_parser(&self, asn1_type: Asn1Type) -> TokenStream {
        if !self.options.parsers.contains(&asn1_type) {
            if self.options.debug {
                eprintln!("// Parsers: skipping asn1_type {:?}", asn1_type);
            }
            return quote! {};
        }

        let parser = asn1_type.parser();
        let from_ber_content = asn1_type.from_ber_content();
        let lft = Lifetime::new("'ber", Span::call_site());

        let f_ty = &self.target.ast().ty;

        // error type
        let error = if let Some(attr) = &self.error {
            get_attribute_meta(attr).expect("Invalid error attribute format")
        } else {
            quote! { <#f_ty as #parser<#lft>>::Error }
        };

        let mut where_predicates = self.where_predicates.clone();

        // Note: if Self has lifetime bounds, then a new bound must be added to the implementation
        // For ex: `pub struct AA<'a>` will require a bound `impl[..] DerParser[..] where 'i: 'a`
        // Container::from_datastruct takes care of this.

        // NOTE: this can fail it #ty contains lifetimes
        // for ex: <&'a [u8] as BerParser<'ber>>  <-- would require 'a: 'ber

        // workaround 1:
        // let f_ty = {
        //     let mut f_ty = f_ty.clone();
        //     // replace all lifetimes with #lft
        //     // this can be complex, there are many possible types
        //     if let Type::Reference(ref mut r) = f_ty {
        //         r.lifetime = Some(lft.clone());
        //     }
        //     f_ty
        // };

        // workaround 2:
        // add all reverse lifetime bounds ('a: 'ber)
        // this makes 'ber equal to all lifetimes (which seems right here)
        for l in self.synstruct.ast().generics.lifetimes() {
            if l.lifetime != lft {
                let pred: WherePredicate = parse_quote! { #l: #lft };
                where_predicates.push(pred);
            }
        }

        // if using custom error, we need to map errors before return
        let map_err = self
            .error
            .as_ref()
            .map(|_| quote! { .map_err(asn1_rs::nom::Err::convert) });

        let fn_content = quote! {
                let (rem, obj) = #parser::#from_ber_content(header, input)#map_err?;
                Ok((rem, Self(obj)))

        };

        // note: other lifetimes will automatically be added by gen_impl
        let tokens = self.synstruct.gen_impl(quote! {
            use asn1_rs::#parser;

            gen impl<#lft> #parser<#lft> for @Self where #(#where_predicates),* {
                type Error = #error;

                fn #from_ber_content(header: &'_ asn1_rs::Header<#lft>, input: asn1_rs::Input<#lft>) -> asn1_rs::nom::IResult<asn1_rs::Input<#lft>, Self, Self::Error> {
                    #fn_content
                }
            }
        });

        // let s = tokens.clone();
        // eprintln!("{}", quote! {#s});

        tokens
    }

    fn derive_alias_encode(&self, asn1_type: Asn1Type) -> TokenStream {
        if !self.options.encoders.contains(&asn1_type) {
            if self.options.debug {
                eprintln!("// Encoders: skipping asn1_type {:?}", asn1_type);
            }
            return quote! {};
        }

        let wh = &self.where_predicates;
        // we must filter out the 'ber lifetime (added for parsers, but not used here)
        let wh = wh.iter().filter(|predicate| match predicate {
            WherePredicate::Lifetime(lft) => lft.lifetime.ident != "ber",
            _ => true,
        });

        let tober = asn1_type.tober();
        let ber_content_len = asn1_type.compose("_content_len");
        let ber_tag_info = asn1_type.compose("_tag_info");
        let ber_write_content = asn1_type.compose("_write_content");

        // note: `gen impl` in synstructure takes care of appending extra where clauses if any, and removing
        // the `where` statement if there are none.
        let ts = self.synstruct.gen_impl(quote! {
            extern crate asn1_rs;

            #[cfg(feature = "std")]
            gen impl asn1_rs::#tober for @Self where #(#wh)+* {
                type Encoder = asn1_rs::BerGenericEncoder;

                fn #ber_content_len(&self) -> asn1_rs::Length {
                    use asn1_rs::#tober;
                    self.0.#ber_content_len()
                }

                fn #ber_tag_info(&self) -> (asn1_rs::Class, bool, asn1_rs::Tag) {
                    use asn1_rs::#tober;
                    self.0.#ber_tag_info()
                }

                fn #ber_write_content<W: std::io::Write>(&self, target: &mut W) -> asn1_rs::SerializeResult<usize> {
                    use asn1_rs::#tober;
                    self.0.#ber_write_content(target)
                }
            }
        });
        ts
    }
}

pub fn derive_ber_alias(s: synstructure::Structure) -> TokenStream {
    let ast = s.ast();

    let container = match &ast.data {
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Alias).unwrap(),
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
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Alias)?,
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
    let impl_berparser = container.gen_berparser(Asn1Type::Ber, &options, &s);
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
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Alias).unwrap(),
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
        Data::Struct(ds) => Container::from_datastruct(ds, ast, ContainerType::Alias)?,
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
    let impl_derparser = container.gen_berparser(Asn1Type::Der, &options, &s);
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
