use proc_macro2::{Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::ParseStream, parse_quote, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput,
    Field, Ident, Lifetime, LitInt, Meta, Type, WherePredicate,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Asn1Type {
    Ber,
    Der,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Asn1TagKind {
    Explicit,
    Implicit,
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
        extern crate asn1_rs;

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
    let impl_fromder = container.gen_fromder();
    let ts = s.gen_impl(quote! {
        extern crate asn1_rs;

        #impl_tryfrom
        #impl_tagged
        #impl_checkconstraints
        #impl_fromder
    });
    if debug_derive {
        eprintln!("{}", ts.to_string());
    }
    ts
}

pub struct Container {
    pub fields: Vec<FieldInfo>,
    pub where_predicates: Vec<WherePredicate>,
    pub error: Option<Attribute>,
}

impl Container {
    pub fn from_datastruct(ds: &DataStruct, ast: &DeriveInput) -> Self {
        if let syn::Fields::Unnamed(_) = ds.fields {
            panic!("Unit struct not supported");
        }

        let fields = ds.fields.iter().map(FieldInfo::from).collect();

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
            .find(|attr| attr.path.is_ident(&Ident::new("error", Span::call_site())))
            .map(|attr| attr.clone());

        Container {
            fields,
            where_predicates,
            error,
        }
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
        // note: `gen impl` in synstructure takes care of appending extra where clauses if any, and removing
        // the `where` statement if there are none.
        quote! {
            use asn1_rs::{Any, FromBer};
            use core::convert::TryFrom;

            gen impl<#lifetime> TryFrom<Any<#lifetime>> for @Self where #(#wh)+* {
                type Error = #error;

                fn try_from(any: Any<#lifetime>) -> asn1_rs::Result<Self, #error> {
                    use asn1_rs::nom::*;
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
        let lifetime = Lifetime::new("'ber", Span::call_site());
        let wh = &self.where_predicates;
        // let parse_content = derive_ber_sequence_content(&field_names, Asn1Type::Der);
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
        // note: `gen impl` in synstructure takes care of appending extra where clauses if any, and removing
        // the `where` statement if there are none.
        quote! {
            use asn1_rs::{CheckDerConstraints, Tagged};
            gen impl<#lifetime> CheckDerConstraints for @Self where #(#wh)+* {
                fn check_constraints(any: &Any) -> asn1_rs::Result<()> {
                    any.tag().assert_eq(Self::TAG)?;
                    let rem = &any.data;
                    #(#check_fields)*
                    Ok(())
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
        // note: `gen impl` in synstructure takes care of appending extra where clauses if any, and removing
        // the `where` statement if there are none.
        quote! {
            use asn1_rs::FromDer;

            gen impl<#lifetime> asn1_rs::FromDer<#lifetime, #error> for @Self where #(#wh)+* {
                fn from_der(bytes: &#lifetime [u8]) -> asn1_rs::ParseResult<#lifetime, Self, #error> {
                    let (rem, any) = asn1_rs::Any::from_der(bytes).map_err(asn1_rs::nom::Err::convert)?;
                    any.header.assert_tag(Self::TAG).map_err(|e| asn1_rs::nom::Err::Error(e.into()))?;
                    let i = any.data;
                    //
                    #parse_content
                    //
                    // let _ = i; // XXX check if empty?
                    Ok((rem,Self{#(#field_names),*}))
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct FieldInfo {
    pub name: Ident,
    pub type_: Type,
    pub optional: bool,
    pub tag: Option<(Asn1TagKind, Asn1TagClass, u16)>,
    pub map_err: Option<TokenStream>,
}

impl From<&Field> for FieldInfo {
    fn from(field: &Field) -> Self {
        // parse attributes and keep supported ones
        let mut optional = false;
        let mut tag = None;
        let mut map_err = None;
        for attr in &field.attrs {
            let ident = match attr.path.get_ident() {
                Some(ident) => ident.to_string(),
                None => continue,
            };
            match ident.as_str() {
                "map_err" => {
                    let expr: syn::Expr = attr.parse_args().expect("could not parse map_err");
                    map_err = Some(quote! { #expr });
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
                // ignore unknown attributes
                _ => (),
            }
        }
        FieldInfo {
            name: field.ident.clone().unwrap(),
            type_: field.ty.clone(),
            optional,
            tag,
            map_err,
        }
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
    let from = match asn1_type {
        Asn1Type::Ber => quote! {FromBer::from_ber},
        Asn1Type::Der => quote! {FromDer::from_der},
    };
    let name = &f.name;
    let map_err = if let Some(tt) = f.map_err.as_ref() {
        if asn1_type == Asn1Type::Ber {
            Some(quote! { .finish().map_err(#tt) })
        } else {
            // Some(quote! { .map_err(|err| nom::Err::convert(#tt)) })
            Some(quote! { .map_err(|err| err.map(#tt)) })
        }
    } else {
        // add mapping functions only if custom errors are used
        if custom_errors {
            if asn1_type == Asn1Type::Ber {
                Some(quote! { .finish() })
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
            };
        } else {
            // tagged, but not OPTIONAL
            return quote! {
                let (i, #name) = {
                    let (i, t): (_, asn1_rs::TaggedValue::<_, _, #tag_kind, {#class}, #tag>) = #from(i)#map_err?;
                    (i, t.into_inner())
                };
            };
        }
    } else {
        // neither tagged nor optional
        quote! {
            let (i, #name) = #from(i)#map_err?;
        }
    }
}

fn get_attribute_meta(attr: &Attribute) -> Result<TokenStream, syn::Error> {
    if let Ok(Meta::List(meta)) = attr.parse_meta() {
        let content = &meta.nested;
        Ok(quote! { #content })
    } else {
        Err(syn::Error::new(
            attr.span(),
            "Invalid error attribute format",
        ))
    }
}
