use std::convert::TryFrom;

use proc_macro2::{Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::{
    parse_quote, Attribute, DataStruct, DeriveInput, Expr, Field, Fields, Ident, Lifetime, LitInt,
    LitStr, Meta, Type, WherePredicate,
};

use crate::asn1_type::Asn1Type;
use crate::options::Options;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContainerType {
    Alias,
    Sequence,
    Set,
}

impl ToTokens for ContainerType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = match self {
            ContainerType::Alias => quote! {},
            ContainerType::Sequence => quote! { asn1_rs::Tag::Sequence },
            ContainerType::Set => quote! { asn1_rs::Tag::Set },
        };
        s.to_tokens(tokens)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Asn1TagKind {
    Explicit,
    Implicit,
}

impl Default for Asn1TagKind {
    fn default() -> Self {
        Asn1TagKind::Explicit
    }
}

impl ToTokens for Asn1TagKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = match self {
            Asn1TagKind::Explicit => quote! { asn1_rs::Explicit },
            Asn1TagKind::Implicit => quote! { asn1_rs::Implicit },
        };
        s.to_tokens(tokens)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Asn1TagClass {
    Universal,
    Application,
    ContextSpecific,
    Private,
}

impl Asn1TagClass {
    pub fn class_tokens(&self) -> TokenStream {
        match *self {
            Asn1TagClass::Universal => quote! { asn1_rs::Class::Universal },
            Asn1TagClass::Application => quote! { asn1_rs::Class::Application },
            Asn1TagClass::ContextSpecific => quote! { asn1_rs::Class::ContextSpecific },
            Asn1TagClass::Private => quote! { asn1_rs::Class::Private },
        }
    }
}

impl ToTokens for Asn1TagClass {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = match self {
            Asn1TagClass::Application => quote! { asn1_rs::Class::APPLICATION },
            Asn1TagClass::ContextSpecific => quote! { asn1_rs::Class::CONTEXT_SPECIFIC },
            Asn1TagClass::Private => quote! { asn1_rs::Class::PRIVATE },
            Asn1TagClass::Universal => quote! { asn1_rs::Class::UNIVERSAL },
        };
        s.to_tokens(tokens)
    }
}

pub struct Container {
    pub container_type: ContainerType,
    pub fields: Vec<FieldInfo>,
    pub where_predicates: Vec<WherePredicate>,
    pub error: Option<Attribute>,

    is_any: bool,
}

impl Container {
    pub fn from_datastruct(
        ds: &DataStruct,
        ast: &DeriveInput,
        container_type: ContainerType,
    ) -> syn::Result<Self> {
        let mut is_any = false;
        match (container_type, &ds.fields) {
            (ContainerType::Alias, Fields::Unnamed(f)) => {
                if f.unnamed.len() != 1 {
                    panic!("Alias: only tuple fields with one element are supported");
                }
                match &f.unnamed[0].ty {
                    Type::Path(type_path)
                        if type_path
                            .clone()
                            .into_token_stream()
                            .to_string()
                            .starts_with("Any") =>
                    {
                        is_any = true;
                    }
                    _ => (),
                }
            }
            (ContainerType::Alias, _) => panic!("BER/DER alias must be used with tuple structs"),
            (_, Fields::Unnamed(_)) => panic!("BER/DER sequence cannot be used on tuple structs"),
            _ => (),
        }

        let fields = ds
            .fields
            .iter()
            .map(FieldInfo::try_from)
            .collect::<Result<Vec<_>, syn::Error>>()?;

        // get lifetimes from generics
        let lfts: Vec<_> = ast.generics.lifetimes().collect();
        let mut where_predicates = Vec::new();
        if !lfts.is_empty() {
            // input slice must outlive all lifetimes from Self
            let lft = Lifetime::new("'ber", Span::call_site());
            let wh: WherePredicate = parse_quote! { #lft: #(#lfts)+* };
            where_predicates.push(wh);
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

        let container = Container {
            container_type,
            fields,
            where_predicates,
            error,
            is_any,
        };
        Ok(container)
    }

    pub fn gen_tryfrom(&self) -> TokenStream {
        let field_names = &self.fields.iter().map(|f| &f.name).collect::<Vec<_>>();
        let parse_content =
            derive_ber_sequence_content(&self.fields, Asn1Type::Ber, self.error.is_some());
        let lifetime = Lifetime::new("'ber", Span::call_site());
        let wh = &self.where_predicates;
        let error = if let Some(attr) = &self.error {
            get_attribute_meta(attr).expect("Invalid error attribute format")
        } else {
            quote! { asn1_rs::Error }
        };

        let fn_content = if self.container_type == ContainerType::Alias {
            // special case: is this an alias for Any
            if self.is_any {
                quote! { Ok(Self(any)) }
            } else {
                quote! {
                    let res = TryFrom::try_from(any)?;
                    Ok(Self(res))
                }
            }
        } else {
            quote! {
                use asn1_rs::nom::*;
                any.tag().assert_eq(Self::TAG)?;

                // no need to parse sequence, we already have content
                let i = any.data.as_bytes2();
                //
                #parse_content
                //
                let _ = i; // XXX check if empty?
                Ok(Self{#(#field_names),*})
            }
        };
        // note: `gen impl` in synstructure takes care of appending extra where clauses if any, and removing
        // the `where` statement if there are none.
        quote! {
            use asn1_rs::{Any, FromBer};
            use core::convert::TryFrom;

            gen impl<#lifetime> TryFrom<Any<#lifetime>> for @Self where #(#wh)+* {
                type Error = #error;

                fn try_from(any: Any<#lifetime>) -> asn1_rs::Result<Self, #error> {
                    #fn_content
                }
            }
        }
    }

    pub fn gen_tagged(&self) -> TokenStream {
        let mut constructed = true;
        let class;
        let tag;
        if self.container_type == ContainerType::Alias {
            constructed = false;
            // special case: is this an alias for Any
            if self.is_any {
                return quote! {
                    gen impl<'ber> asn1_rs::DynTagged for @Self {
                        fn class(&self) -> asn1_rs::Class {
                            self.0.class()
                        }

                        fn constructed(&self) -> bool {
                            self.0.constructed()
                        }

                        fn tag(&self) -> asn1_rs::Tag {
                            self.0.tag()
                        }

                        fn accept_tag(_: asn1_rs::Tag) -> bool {
                            // For ANY, all tags are accepted
                            true
                        }
                    }
                };
            }
            // find type of sub-item
            let ty = &self.fields[0].type_;
            class = quote! { <#ty as asn1_rs::Tagged>::CLASS };
            tag = quote! { <#ty as asn1_rs::Tagged>::TAG };
        } else {
            let container_type = self.container_type;
            class = quote! { asn1_rs::Class::Universal };
            tag = quote! { #container_type };
        }
        quote! {
            gen impl<'ber> asn1_rs::Tagged for @Self {
                const CLASS: asn1_rs::Class = #class;
                const CONSTRUCTED: bool = #constructed;
                const TAG: asn1_rs::Tag = #tag;
            }
        }
    }

    /// Generate blanked implementation of Ber/DerParser
    ///
    /// `Self` must be `Tagged`
    pub fn gen_berparser(
        &self,
        asn1_type: Asn1Type,
        options: &Options,
        s: &synstructure::Structure,
    ) -> TokenStream {
        if !options.parsers.contains(&asn1_type) {
            if options.debug {
                eprintln!("// Parsers: skipping asn1_type {:?}", asn1_type);
            }
            return quote! {};
        }

        let parser = asn1_type.parser();
        let from_ber_content = asn1_type.from_ber_content();
        let lft = Lifetime::new("'ber", Span::call_site());

        // error type
        let mut error = if let Some(attr) = &self.error {
            get_attribute_meta(attr).expect("Invalid error attribute format")
        } else {
            quote! { asn1_rs::BerError<asn1_rs::Input<#lft>> }
        };

        let field_names = &self.fields.iter().map(|f| &f.name).collect::<Vec<_>>();

        let parse_content = derive_berparser_sequence_content(&self.fields, asn1_type);

        // Note: if Self has lifetime bounds, then a new bound must be added to the implementation
        // For ex: `pub struct AA<'a>` will require a bound `impl[..] DerParser[..] where 'i: 'a`
        // Container::from_datastruct takes care of this.
        let mut wh = self.where_predicates.clone();

        let orig_input = if options.orig_input {
            Some(quote! { let orig_input = input.clone(); })
        } else {
            None
        };

        let fn_content = if self.container_type == ContainerType::Alias {
            // special case: is this an alias for Any
            if self.is_any {
                quote! {
                    use asn1_rs::#parser;
                    let (rem, any) = asn1_rs::Any::#from_ber_content(header, input)?;
                    Ok((rem, Self(any)))
                }
            } else {
                // we support only 1 unnamed field
                assert_eq!(self.fields.len(), 1);
                let f_ty = &self
                    .fields
                    .first()
                    .expect("Tuple struct without fields")
                    .type_;
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
                for l in s.ast().generics.lifetimes() {
                    if l.lifetime != lft {
                        let pred: WherePredicate = parse_quote! { #l: #lft };
                        wh.push(pred);
                    }
                }

                error = quote! {
                    <#f_ty as #parser<#lft>>::Error
                };
                quote! {
                    use asn1_rs::#parser;
                    let (rem, any) = #parser::#from_ber_content(header, input)?;
                    Ok((rem, Self(any)))
                }
            }
        } else {
            // assert constructed (only for Sequence/Set)
            let assert_constructed = self.gen_assert_constructed();

            quote! {
                #orig_input
                let rem = input;
                //
                #assert_constructed
                #parse_content
                //
                // XXX check if rem empty?
                Ok((
                    rem,
                    Self{#(#field_names),*}
                ))
            }
        };

        // note: other lifetimes will automatically be added by gen_impl
        let tokens = quote! {
            use asn1_rs::#parser;

            gen impl<#lft> #parser<#lft> for @Self where #(#wh),* {
                type Error = #error;

                fn #from_ber_content(header: &'_ asn1_rs::Header<#lft>, input: asn1_rs::Input<#lft>) -> asn1_rs::nom::IResult<asn1_rs::Input<#lft>, Self, Self::Error> {
                    #fn_content
                }
            }
        };

        // let s = tokens.clone();
        // eprintln!("{}", quote! {#s});

        tokens
    }

    fn gen_assert_constructed(&self) -> TokenStream {
        if self.container_type == ContainerType::Alias {
            // do nothing - this should be handled by inner type parser
            quote! {}
        } else {
            quote!(
                // Tagged Explicit must be constructed (X.690 8.14.2)
                if !header.constructed() {
                    return Err(nom::Err::Error(
                        asn1_rs::BerError::new(rem, asn1_rs::InnerError::ConstructExpected).into(),
                    ));
                }
            )
        }
    }

    pub fn gen_checkconstraints(&self) -> TokenStream {
        let lifetime = Lifetime::new("'ber", Span::call_site());
        let wh = &self.where_predicates;
        // let parse_content = derive_ber_sequence_content(&field_names, Asn1Type::Der);

        let fn_content = if self.container_type == ContainerType::Alias {
            // special case: is this an alias for Any
            if self.is_any {
                return quote! {};
            }
            let ty = &self.fields[0].type_;
            quote! {
                any.tag().assert_eq(Self::TAG)?;
                <#ty>::check_constraints(any)
            }
        } else {
            let check_fields: Vec<_> = self
                .fields
                .iter()
                .map(|field| {
                    let ty = &field.type_;
                    quote! {
                        let (rem, any) = Any::from_der(rem)?;
                        <#ty as CheckDerConstraints>::check_constraints(&any)?;
                    }
                })
                .collect();
            quote! {
                any.tag().assert_eq(Self::TAG)?;
                let rem = any.data.as_bytes2();
                #(#check_fields)*
                Ok(())
            }
        };

        // note: `gen impl` in synstructure takes care of appending extra where clauses if any, and removing
        // the `where` statement if there are none.
        quote! {
            use asn1_rs::{CheckDerConstraints, Tagged};
            gen impl<#lifetime> CheckDerConstraints for @Self where #(#wh)+* {
                fn check_constraints(any: &Any) -> asn1_rs::Result<()> {
                    #fn_content
                }
            }
        }
    }

    pub fn gen_fromder(&self) -> TokenStream {
        let lifetime = Lifetime::new("'ber", Span::call_site());
        let wh = &self.where_predicates;
        let field_names = &self.fields.iter().map(|f| &f.name).collect::<Vec<_>>();
        let parse_content =
            derive_ber_sequence_content(&self.fields, Asn1Type::Der, self.error.is_some());
        let error = if let Some(attr) = &self.error {
            get_attribute_meta(attr).expect("Invalid error attribute format")
        } else {
            quote! { asn1_rs::Error }
        };

        let fn_content = if self.container_type == ContainerType::Alias {
            // special case: is this an alias for Any
            if self.is_any {
                quote! {
                    let (rem, any) = asn1_rs::Any::from_der(bytes).map_err(asn1_rs::nom::Err::convert)?;
                    Ok((rem,Self(any)))
                }
            } else {
                quote! {
                    let (rem, any) = asn1_rs::Any::from_der(bytes).map_err(asn1_rs::nom::Err::convert)?;
                    any.header.assert_tag(Self::TAG).map_err(|e| asn1_rs::nom::Err::Error(e.into()))?;
                    let res = TryFrom::try_from(any)?;
                    Ok((rem,Self(res)))
                }
            }
        } else {
            quote! {
                let (rem, any) = asn1_rs::Any::from_der(bytes).map_err(asn1_rs::nom::Err::convert)?;
                any.header.assert_tag(Self::TAG).map_err(|e| asn1_rs::nom::Err::Error(e.into()))?;
                let i = any.data.as_bytes2();
                //
                #parse_content
                //
                // let _ = i; // XXX check if empty?
                Ok((rem,Self{#(#field_names),*}))
            }
        };
        // note: `gen impl` in synstructure takes care of appending extra where clauses if any, and removing
        // the `where` statement if there are none.
        quote! {
            use asn1_rs::FromDer;

            gen impl<#lifetime> asn1_rs::FromDer<#lifetime, #error> for @Self where #(#wh)+* {
                fn from_der(bytes: &#lifetime [u8]) -> asn1_rs::ParseResult<#lifetime, Self, #error> {
                    #fn_content
                }
            }
        }
    }

    pub fn gen_tober_content_len(
        &self,
        asn1_type: Asn1Type,
        s: &synstructure::Structure,
    ) -> TokenStream {
        let total_len = if self.container_type == ContainerType::Alias {
            // alias: content length only
            asn1_type.content_len_tokens()
        } else {
            // this is a structured type, full object length
            asn1_type.total_len_tokens()
        };
        let content_len = asn1_type.content_len_tokens();

        let body = s.fold(quote! {asn1_rs::Length::Definite(0)}, |acc, bi| {
            // check if binding has a 'tag_explicit' or 'tag_implicit' attribute
            let tag_kind = get_field(&self.fields, bi.ast().ident.as_ref())
                .map(|f| f.tag)
                .flatten();

            match tag_kind {
                Some((Asn1TagKind::Explicit, _class, tag)) => {
                    // TAGGED EXPLICIT: add length required to encode tag header
                    let tag = u32::from(tag);
                    quote! {
                        #acc
                            + asn1_rs::ber_header_length(asn1_rs::Tag(#tag), #bi.#total_len()).unwrap_or_default()
                            + #bi.#total_len()
                    }
                }
                Some((Asn1TagKind::Implicit, _class, tag)) => {
                    // TAGGED IMPLICIT: add length required to encode tag header
                    // This could be different from `#bi.#total_len()` in the specific case one of
                    // (implicit tag, object tag) is long and the other is not
                    let tag = u32::from(tag);
                    quote! {
                        #acc
                            + asn1_rs::ber_header_length(asn1_rs::Tag(#tag), #bi.#total_len()).unwrap_or_default()
                            + #bi.#content_len()
                    }
                }
                None => quote! {
                    #acc + #bi.#total_len()
                },
            }
        });

        quote! {
            fn #content_len(&self) -> asn1_rs::Length {
                match *self {
                    #body
                }
            }
        }
    }

    pub fn gen_tober_tag_info(&self, asn1_type: Asn1Type) -> TokenStream {
        let tag_info = asn1_type.tag_info_tokens();
        let body = match self.container_type {
            ContainerType::Alias => {
                if self.is_any {
                    quote! {
                        use asn1_rs::DynTagged;
                        (self.0.class(), self.0.constructed(), self.0.tag())
                    }
                } else {
                    // find type of sub-item
                    let ty = &self.fields[0].type_;
                    quote! {
                        (<#ty as asn1_rs::Tagged>::CLASS, <#ty as asn1_rs::Tagged>::CONSTRUCTED, <#ty as asn1_rs::Tagged>::TAG)
                    }
                }
            }
            ContainerType::Sequence => {
                quote!((asn1_rs::Class::Universal, true, asn1_rs::Tag::Sequence))
            }
            ContainerType::Set => {
                quote!((asn1_rs::Class::Universal, true, asn1_rs::Tag::Set))
            }
        };
        quote! {
            fn #tag_info(&self) -> (asn1_rs::Class, bool, asn1_rs::Tag) {
                #body
            }
        }
    }

    pub fn gen_tober_write_content(
        &self,
        asn1_type: Asn1Type,
        s: &synstructure::Structure,
    ) -> TokenStream {
        let encode = if self.container_type == ContainerType::Alias {
            // alias: only write content
            asn1_type.write_content_tokens()
        } else {
            // this is a structured type, encode full object
            asn1_type.encode_tokens()
        };
        let write_content = asn1_type.write_content_tokens();
        let encode_explicit = asn1_type.compose("_encode_tagged_explicit");
        let encode_implicit = asn1_type.compose("_encode_tagged_implicit");

        // we can't just use `s.fold()` because we need to add a footer `Ok(num_bytes)`
        let body = s.variants().iter().map(|vi| {
            let encode = vi.bindings().iter().map(|bi| {
                // check if binding has a 'tag_explicit' or 'tag_implicit' attribute
                let tag_kind = get_field(&self.fields, bi.ast().ident.as_ref())
                    .map(|f| f.tag)
                    .flatten();

                match tag_kind {
                    Some((Asn1TagKind::Explicit, class, tag)) => {
                        let tk_class = class.class_tokens();
                        let tag = u32::from(tag);
                        quote! {
                            num_bytes += #bi.#encode_explicit(#tk_class, #tag, writer)?;
                        }
                    }
                    Some((Asn1TagKind::Implicit, class, tag)) => {
                        let tk_class = class.class_tokens();
                        let tag = u32::from(tag);
                        quote! {
                            num_bytes += #bi.#encode_implicit(#tk_class, #tag, writer)?;
                        }
                    }
                    None => quote! { num_bytes += #bi.#encode(writer)?; },
                }
            });
            let pat = vi.pat();
            quote! {
                #pat => {
                    let mut num_bytes = 0;
                    #(#encode)*
                    Ok(num_bytes)
                }
            }
        });

        quote! {
            fn #write_content<W: std::io::Write>(&self, writer: &mut W) -> asn1_rs::SerializeResult<usize> {
                match *self {
                    #(#body)*
                }
            }
        }
    }

    pub fn gen_tober(
        &self,
        asn1_type: Asn1Type,
        options: &Options,
        s: &synstructure::Structure,
    ) -> TokenStream {
        if !options.encoders.contains(&asn1_type) {
            if options.debug {
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

        let impl_tober_content_len = self.gen_tober_content_len(asn1_type, &s);
        let impl_tober_tag_info = self.gen_tober_tag_info(asn1_type);
        let impl_tober_write_content = self.gen_tober_write_content(asn1_type, &s);
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
        ts
    }
}

#[derive(Debug)]
pub struct FieldInfo {
    pub name: Ident,
    pub type_: Type,
    pub default: Option<TokenStream>,
    pub optional: bool,
    pub tag: Option<(Asn1TagKind, Asn1TagClass, u16)>,
    pub map_err: Option<TokenStream>,
    pub parse: Option<Expr>,
    // TODO: implement this
    #[allow(unused)]
    pub encode: Option<Expr>,
}

impl TryFrom<&Field> for FieldInfo {
    type Error = syn::Error;

    fn try_from(field: &Field) -> Result<Self, Self::Error> {
        // parse attributes and keep supported ones
        let mut optional = false;
        let mut tag = None;
        let mut map_err = None;
        let mut default = None;
        let name = field
            .ident
            .as_ref()
            .map_or_else(|| Ident::new("_", Span::call_site()), |s| s.clone());
        let mut parse = None;
        let mut encode = None;
        for attr in &field.attrs {
            let ident = match attr.meta.path().get_ident() {
                Some(ident) => ident.to_string(),
                None => continue,
            };
            match ident.as_str() {
                "map_err" => {
                    let expr: syn::Expr = attr.parse_args()?;
                    map_err = Some(quote! { #expr });
                }
                "default" => {
                    let expr: syn::Expr = attr.parse_args()?;
                    default = Some(quote! { #expr });
                    optional = true;
                }
                "optional" => optional = true,
                "tag_explicit" => {
                    if tag.is_some() {
                        panic!("tag cannot be set twice!");
                    }
                    let (class, value) = attr.parse_args_with(parse_tag_args).unwrap();
                    tag = Some((Asn1TagKind::Explicit, class, value));
                }
                "tag_implicit" => {
                    if tag.is_some() {
                        panic!("tag cannot be set twice!");
                    }
                    let (class, value) = attr.parse_args_with(parse_tag_args).unwrap();
                    tag = Some((Asn1TagKind::Implicit, class, value));
                }
                "asn1" => {
                    // see example at <https://docs.rs/syn/latest/syn/meta/struct.ParseNestedMeta.html>
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("parse") {
                            let value = meta.value()?;
                            let lit: LitStr = value.parse()?;
                            let e: Expr = lit.parse()?;
                            parse = Some(e);
                        } else if meta.path.is_ident("encode") {
                            let value = meta.value()?;
                            let lit: LitStr = value.parse()?;
                            let e: Expr = lit.parse()?;
                            encode = Some(e);
                            return Err(meta.error("Attribute 'encode' is not yet supported"));
                        } else {
                            return Err(meta.error("Invalid or unknown attribute"));
                        }
                        return Ok(());
                    })?;
                }
                // ignore unknown attributes
                _ => (),
            }
        }

        let f = FieldInfo {
            name,
            type_: field.ty.clone(),
            default,
            optional,
            tag,
            map_err,
            parse,
            encode,
        };
        Ok(f)
    }
}

fn parse_tag_args(stream: ParseStream) -> Result<(Asn1TagClass, u16), syn::Error> {
    let tag_class: Option<Ident> = stream.parse()?;
    let tag_class = if let Some(ident) = tag_class {
        let s = ident.to_string().to_uppercase();
        match s.as_str() {
            "UNIVERSAL" => Asn1TagClass::Universal,
            "CONTEXT-SPECIFIC" => Asn1TagClass::ContextSpecific,
            "APPLICATION" => Asn1TagClass::Application,
            "PRIVATE" => Asn1TagClass::Private,
            _ => {
                return Err(syn::Error::new(stream.span(), "Invalid tag class"));
            }
        }
    } else {
        Asn1TagClass::ContextSpecific
    };
    let lit: LitInt = stream.parse()?;
    let value = lit.base10_parse::<u16>()?;
    Ok((tag_class, value))
}

fn derive_ber_sequence_content(
    fields: &[FieldInfo],
    asn1_type: Asn1Type,
    custom_errors: bool,
) -> TokenStream {
    let field_parsers: Vec<_> = fields
        .iter()
        .map(|f| get_field_parser(f, asn1_type, custom_errors))
        .collect();

    quote! {
        #(#field_parsers)*
    }
}

fn get_field_parser(f: &FieldInfo, asn1_type: Asn1Type, custom_errors: bool) -> TokenStream {
    let name = &f.name;

    // if a 'parse' attribute was specified, use it
    if let Some(e) = &f.parse {
        return quote! {
            let parse = #e;
            let (rem, #name) = parse(rem)?;
        };
    }

    // else, derive parser
    let from = match asn1_type {
        Asn1Type::Ber => quote! {FromBer::from_ber},
        Asn1Type::Der => quote! {FromDer::from_der},
    };
    let default = f
        .default
        .as_ref()
        // use a type hint, otherwise compiler will not know what type provides .unwrap_or
        .map(|x| quote! {let #name: Option<_> = #name; let #name = #name.unwrap_or(#x);});
    let map_err = if let Some(tt) = f.map_err.as_ref() {
        if asn1_type == Asn1Type::Ber {
            Some(quote! {
                .map_err(|err| err.map(#tt))
                .map_err(asn1_rs::from_nom_error::<_, Self::Error>)
            })
        } else {
            // Some(quote! { .map_err(|err| nom::Err::convert(#tt)) })
            Some(quote! { .map_err(|err| err.map(#tt)) })
        }
    } else {
        // add mapping functions only if custom errors are used
        if custom_errors {
            if asn1_type == Asn1Type::Ber {
                Some(quote! { .map_err(asn1_rs::from_nom_error::<_, Self::Error>) })
            } else {
                Some(quote! { .map_err(nom::Err::convert) })
            }
        } else {
            None
        }
    };
    if let Some((tag_kind, class, n)) = f.tag {
        let tag = Literal::u16_unsuffixed(n);
        // test if tagged + optional
        if f.optional {
            return quote! {
                let (i, #name) = {
                    if i.is_empty() {
                        (i, None)
                    } else {
                        let (_, header): (_, asn1_rs::Header) = #from(i)#map_err?;
                        if header.tag().0 == #tag {
                            let (i, t): (_, asn1_rs::TaggedValue::<_, _, #tag_kind, {#class}, #tag>) = #from(i)#map_err?;
                            (i, Some(t.into_inner()))
                        } else {
                            (i, None)
                        }
                    }
                };
                #default
            };
        } else {
            // tagged, but not OPTIONAL
            return quote! {
                let (i, #name) = {
                    let (i, t): (_, asn1_rs::TaggedValue::<_, _, #tag_kind, {#class}, #tag>) = #from(i)#map_err?;
                    (i, t.into_inner())
                };
                #default
            };
        }
    } else {
        // neither tagged nor optional
        quote! {
            let (i, #name) = #from(i)#map_err?;
            #default
        }
    }
}

fn derive_berparser_sequence_content(fields: &[FieldInfo], asn1_type: Asn1Type) -> TokenStream {
    let field_parsers: Vec<_> = fields
        .iter()
        .map(|f| get_field_berparser(f, asn1_type))
        .collect();

    quote! {
        #(#field_parsers)*
    }
}

fn get_field<'a>(fields: &'a [FieldInfo], ident: Option<&Ident>) -> Option<&'a FieldInfo> {
    let ident = if let Some(ident) = ident {
        ident
    } else {
        return None;
    };
    // eprintln!("Looking for field '{ident}");
    fields.iter().find(|&f| f.name == *ident)
}

// This is an adapted version of `get_field_parser` to use types related to `BerParser`
fn get_field_berparser(f: &FieldInfo, asn1_type: Asn1Type) -> TokenStream {
    let name = &f.name;

    // if a 'parse' attribute was specified, use it
    if let Some(e) = &f.parse {
        return quote! {
            let parse = #e;
            let (rem, #name) = parse(rem)?;
        };
    }

    // else, derive parser
    let parser = asn1_type.parser();
    let from = match asn1_type {
        Asn1Type::Ber => quote! {BerParser::parse_ber},
        Asn1Type::Der => quote! {DerParser::parse_der},
    };

    let default = f
        .default
        .as_ref()
        // use a type hint, otherwise compiler will not know what type provides .unwrap_or
        .map(|x| quote! {let #name: Option<_> = #name; let #name = #name.unwrap_or(#x);});

    // no need to check for custom errors, this should be transparent using `.into()`
    let map_err = if let Some(tt) = f.map_err.as_ref() {
        quote! { .map_err(|err| err.map(#tt)) }
    } else {
        quote! { .map_err(nom::Err::convert) }
    };

    if let Some((tag_kind, class, n)) = f.tag {
        let tag = Literal::u16_unsuffixed(n);

        // test if tagged + optional
        if f.optional {
            // Tagged + optional
            let f_ty = &f.type_;
            quote! {
                let (rem, #name) = {
                    if rem.is_empty() {
                        (rem, None)
                    } else {
                        // clone rem, #map_err may consume it
                        let rem_copy = rem.clone();
                        let (_, obj_header): (_, asn1_rs::Header) = #from(rem_copy)#map_err?;
                        if obj_header.tag().0 == #tag {
                            let (rem, t): (_, asn1_rs::TaggedValue::<
                                _,
                                <#f_ty as asn1_rs::#parser>::Error,
                                #tag_kind,
                                {#class},
                                #tag>
                            ) = #from(rem)#map_err?;
                            (rem, Some(t.into_inner()))
                        } else {
                            (rem, None)
                        }
                    }
                };
                #default
            }
        } else {
            // tagged, but not Optional
            let f_ty = &f.type_;
            quote! {
                let (rem, #name) = {
                    let (rem, t): (_, asn1_rs::TaggedValue::<
                        _,
                        <#f_ty as asn1_rs::#parser>::Error,
                        #tag_kind,
                        {#class},
                        #tag>
                    ) = #from(rem)#map_err?;
                    (rem, t.into_inner())
                };
                #default
            }
        }
    } else {
        // not tagged
        quote! {
            let (rem, #name) = #from(rem)#map_err?;
            #default
        }
    }
}

pub(crate) fn get_attribute_meta(attr: &Attribute) -> Result<TokenStream, syn::Error> {
    if let Meta::List(meta) = &attr.meta {
        let content = &meta.tokens;
        Ok(quote! { #content })
    } else {
        Err(syn::Error::new(
            attr.span(),
            "Invalid error attribute format",
        ))
    }
}
