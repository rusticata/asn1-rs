use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, FieldsNamed, FieldsUnnamed, Ident, Lifetime, LitInt};

pub fn derive_tostatic(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let ast = s.ast();

    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });

    let ds: &syn::DataStruct = match &ast.data {
        Data::Struct(ds) => ds,
        Data::Enum(_) => {
            return derive_tostatic_enum(&s);
        }
        _ => panic!("Unsupported type, cannot derive"),
    };

    let ts = match &ds.fields {
        syn::Fields::Unit => derive_unit_struct(ast),
        syn::Fields::Named(fields) => derive_named_struct(fields, ast),
        syn::Fields::Unnamed(fields) => derive_unnamed_struct(fields, ast),
    };

    let ts = s.gen_impl(ts);

    if debug_derive {
        eprintln!("{ts}");
    }
    ts
}

fn derive_tostatic_enum(s: &synstructure::Structure) -> proc_macro2::TokenStream {
    let ast = s.ast();
    let name = &ast.ident;

    let debug_derive = ast.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .is_ident(&Ident::new("debug_derive", Span::call_site()))
    });
    //eprintln!("XXX {:?} XXX", s.variants());

    let body = s.each_variant(|vi| {
        //eprintln!("*** {vi:?}");
        let prefix = vi.prefix;
        let vname = &vi.ast().ident;
        let bindings = vi.bindings();
        // bindings can be empty for unit variants
        let (idents, instrs): (Vec<_>, Vec<_>) = bindings
            .iter()
            .map(|b| {
                let ident = b.ast().ident.as_ref().unwrap_or(&b.binding);
                (ident, quote! {let #ident = #b.to_static(); })
            })
            .collect();
        match vi.ast().fields {
            syn::Fields::Named(_) => {
                quote! {
                    #(#instrs)*
                    #prefix :: #vname { #(#idents),* }
                }
            }
            syn::Fields::Unnamed(_) => {
                quote! {
                    #(#instrs)*
                    #prefix :: #vname ( #(#bindings),* )
                }
            }
            syn::Fields::Unit => {
                quote! { #prefix :: #vname }
            }
        }
    });

    let lft_static = Lifetime::new("'static", Span::call_site());
    let lfts = ast.generics.lifetimes().map(|_| lft_static.clone());

    let ts = s.gen_impl(quote! {
        extern crate asn1_rs;

        gen impl asn1_rs::ToStatic for @Self {
            type Owned = #name<#(#lfts),*>; // with lifetimes!

            fn to_static(&self) -> Self::Owned {
                match *self { #body }
            }
        }
    });
    if debug_derive {
        eprintln!("{ts}");
    }
    ts
}

fn derive_unit_struct(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let struct_ident = &ast.ident;

    quote! {
        gen impl asn1_rs::ToStatic for @Self {
            type Owned = #struct_ident;

            fn to_static(&self) -> Self::Owned {
                #struct_ident
            }
        }
    }
}

fn derive_named_fields(fields: &FieldsNamed) -> (Vec<Ident>, Vec<proc_macro2::TokenStream>) {
    let fields: Vec<_> = fields.named.iter().collect();

    let field_idents: Vec<_> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();

    let field_instrs = field_idents
        .iter()
        .map(|ident| {
            quote! { let #ident = self.#ident.to_static(); }
        })
        .collect();

    (field_idents, field_instrs)
}

fn derive_unnamed_fields(fields: &FieldsUnnamed) -> (Vec<Ident>, Vec<proc_macro2::TokenStream>) {
    let fields: Vec<_> = fields.unnamed.iter().collect();

    let field_idents: Vec<_> = (0..fields.len())
        .map(|idx| Ident::new(&format!("_{idx}"), Span::call_site()))
        .collect();

    let field_instrs = fields
        .iter()
        .zip(field_idents.iter())
        .enumerate()
        .map(|(idx, (_f, ident))| {
            let idx = LitInt::new(&format!("{idx}"), Span::call_site());
            quote! { let #ident = self.#idx.to_static(); }
        })
        .collect();

    (field_idents, field_instrs)
}

fn derive_named_struct(fields: &FieldsNamed, ast: &DeriveInput) -> proc_macro2::TokenStream {
    let (field_idents, field_instrs) = derive_named_fields(fields);

    let struct_ident = &ast.ident;

    // check if struct has lifetimes
    let static_token = match ast.generics.lifetimes().count() {
        0 => None,
        1 => {
            let lt = Lifetime::new("'static", Span::call_site());
            Some(quote! {<#lt>})
        }
        _ => {
            let lt_static = Lifetime::new("'static", Span::call_site());
            let lts = ast.generics.lifetimes().map(|_| lt_static.clone());
            Some(quote! {<#(#lts),*>})
        }
    };

    quote! {
        gen impl asn1_rs::ToStatic for @Self {
            type Owned = #struct_ident #static_token;

            fn to_static(&self) -> Self::Owned {
                #(#field_instrs)*
                #struct_ident{
                    #(#field_idents,)*
                }
            }
        }
    }
}

fn derive_unnamed_struct(fields: &FieldsUnnamed, ast: &DeriveInput) -> proc_macro2::TokenStream {
    let (field_idents, field_instrs) = derive_unnamed_fields(fields);

    let struct_ident = &ast.ident;

    // check if struct has lifetimes
    let static_token = match ast.generics.lifetimes().count() {
        0 => None,
        1 => {
            let lt = Lifetime::new("'static", Span::call_site());
            Some(quote! {<#lt>})
        }
        _ => {
            let lt_static = Lifetime::new("'static", Span::call_site());
            let lts = ast.generics.lifetimes().map(|_| lt_static.clone());
            Some(quote! {<#(#lts),*>})
        }
    };

    quote! {
        gen impl asn1_rs::ToStatic for @Self {
            type Owned = #struct_ident #static_token;

            fn to_static(&self) -> Self::Owned {
                #(#field_instrs)*
                #struct_ident(
                    #(#field_idents,)*
                )
            }
        }
    }
}
