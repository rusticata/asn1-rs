use crate::{asn1_type::Asn1Type, container::*};
use syn::{Attribute, Result};

#[derive(Debug, Default)]
pub struct Options {
    pub debug: bool,
    pub error: Option<Attribute>,
    pub tag_kind: Option<Asn1TagKind>,

    pub parsers: Vec<Asn1Type>,
    pub encoders: Vec<Asn1Type>,
}

impl Options {
    pub fn from_struct(s: &synstructure::Structure) -> Result<Self> {
        let mut options = Self {
            parsers: vec![Asn1Type::Ber, Asn1Type::Der],
            encoders: vec![Asn1Type::Ber, Asn1Type::Der],
            ..Self::default()
        };
        let ast = s.ast();

        for attr in ast.attrs.iter() {
            let path = attr.meta.path();
            if path.is_ident("debug_derive") {
                options.debug = true;
            } else if path.is_ident("tagged_explicit") {
                options.tag_kind = Some(Asn1TagKind::Explicit);
            } else if path.is_ident("tagged_implicit") {
                options.tag_kind = Some(Asn1TagKind::Implicit);
            } else if path.is_ident("error") {
                options.error = Some(attr.clone());
            } else if path.is_ident("asn1") {
                // see example at <https://docs.rs/syn/latest/syn/meta/struct.ParseNestedMeta.html>
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("parse") {
                        let value = meta.value()?;
                        let asn1_types = Asn1Type::parse_multi(value)?;
                        options.parsers = asn1_types.into_iter().collect();
                    } else if meta.path.is_ident("encode") {
                        let value = meta.value()?;
                        let asn1_types = Asn1Type::parse_multi(value)?;
                        options.encoders = asn1_types.into_iter().collect();
                    } else {
                        return Err(meta.error("Invalid or unknown attribute"));
                    }
                    return Ok(());
                })?;
            }
        }

        Ok(options)
    }
}
