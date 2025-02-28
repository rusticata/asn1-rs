use crate::*;
use core::convert::TryFrom;

/// EMBEDDED PDV ASN.1 Object
///
/// <div class="warning">Warning: this implementation has not been tested
///
/// Due to lack of other implementations and of testing data, the parsers for this object have
/// **not** been tested with real data. If you have such data, please contact us (for ex, open
/// an issue).
///
/// </div>
#[derive(Debug, PartialEq, Eq)]
pub struct EmbeddedPdv<'a> {
    pub identification: PdvIdentification<'a>,
    pub data_value_descriptor: Option<ObjectDescriptor<'a>>,
    pub data_value: &'a [u8],
}

#[derive(Debug, PartialEq, Eq)]
pub enum PdvIdentification<'a> {
    Syntaxes {
        s_abstract: Oid<'a>,
        s_transfer: Oid<'a>,
    },
    Syntax(Oid<'a>),
    PresentationContextId(Integer<'a>),
    ContextNegotiation {
        presentation_context_id: Integer<'a>,
        presentation_syntax: Oid<'a>,
    },
    TransferSyntax(Oid<'a>),
    Fixed,
}

impl<'a> TryFrom<Any<'a>> for EmbeddedPdv<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        TryFrom::try_from(&any)
    }
}

impl<'a, 'b> TryFrom<&'b Any<'a>> for EmbeddedPdv<'a> {
    type Error = Error;

    fn try_from(any: &'b Any<'a>) -> Result<Self> {
        let data = any.data.clone();
        // AUTOMATIC TAGS means all values will be tagged (IMPLICIT)
        // [0] -> identification
        let (rem, seq0) = TaggedParser::<Explicit, Any>::parse_ber(
            Class::ContextSpecific,
            Tag(0),
            data.as_bytes2(),
        )?;
        let inner = seq0.inner;
        let identification = match inner.tag() {
            Tag(0) => {
                // syntaxes SEQUENCE {
                //     abstract OBJECT IDENTIFIER,
                //     transfer OBJECT IDENTIFIER
                // },
                // AUTOMATIC tags -> implicit! Hopefully, Oid does not check tag value!
                let (rem, s_abstract) = Oid::from_ber(inner.data.as_bytes2())?;
                let (_, s_transfer) = Oid::from_ber(rem)?;
                PdvIdentification::Syntaxes {
                    s_abstract,
                    s_transfer,
                }
            }
            Tag(1) => {
                // syntax OBJECT IDENTIFIER
                let oid = Oid::new(inner.data.as_bytes2().into());
                PdvIdentification::Syntax(oid)
            }
            Tag(2) => {
                // presentation-context-id INTEGER
                let i = Integer::new(inner.data.as_bytes2());
                PdvIdentification::PresentationContextId(i)
            }
            Tag(3) => {
                // context-negotiation SEQUENCE {
                //     presentation-context-id INTEGER,
                //     transfer-syntax OBJECT IDENTIFIER
                // },
                // AUTOMATIC tags -> implicit!
                let (rem, any) = Any::from_ber(inner.data.as_bytes2())?;
                let presentation_context_id = Integer::new(any.data.as_bytes2());
                let (_, presentation_syntax) = Oid::from_ber(rem)?;
                PdvIdentification::ContextNegotiation {
                    presentation_context_id,
                    presentation_syntax,
                }
            }
            Tag(4) => {
                // transfer-syntax OBJECT IDENTIFIER
                let oid = Oid::new(inner.data.as_bytes2().into());
                PdvIdentification::TransferSyntax(oid)
            }
            Tag(5) => {
                // fixed NULL
                PdvIdentification::Fixed
            }
            _ => {
                return Err(inner
                    .tag()
                    .invalid_value("Invalid identification tag in EMBEDDED PDV"))
            }
        };
        // [1] -> data-value-descriptor ObjectDescriptor OPTIONAL
        // *BUT* WITH COMPONENTS data-value-descriptor ABSENT
        // XXX this should be parse_ber?
        // let (rem, data_value_descriptor) =
        //     TaggedOptional::from(1).parse_der(rem, |_, inner| ObjectDescriptor::from_ber(inner))?;
        let (rem, data_value_descriptor) = (rem, None);
        // [2] -> data-value OCTET STRING
        let (_, data_value) =
            TaggedParser::<Implicit, &[u8]>::parse_ber(Class::ContextSpecific, Tag(2), rem)?;
        let data_value = data_value.inner;
        let obj = EmbeddedPdv {
            identification,
            data_value_descriptor,
            data_value,
        };
        Ok(obj)
    }
}

impl<'i> BerParser<'i> for EmbeddedPdv<'i> {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::EmbeddedPdv
    }

    fn from_any_ber(
        input: Input<'i>,
        _header: Header<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // AUTOMATIC TAGS means all values will be tagged (IMPLICIT)
        // definition taken from <https://www.oss.com/asn1/products/documentation/asn1_java_8.7/api/toed/com/oss/asn1/EmbeddedPDV.html>

        // [0] -> identification
        // this is a [0 IMPLICIT] SEQUENCE, so we can just parse it as Any and take the content
        let (rem, t0) = TaggedImplicit::<Any, Self::Error, 0>::parse_ber(input)?;
        let inner0 = t0.into_inner();
        let identification = match inner0.tag() {
            Tag(0) => {
                // syntaxes SEQUENCE {
                //     abstract OBJECT IDENTIFIER,
                //     transfer OBJECT IDENTIFIER
                // },
                let inner_data = inner0.data;
                let (rem, t_abstract) =
                    TaggedImplicit::<Oid, Self::Error, 0>::parse_ber(inner_data)?;
                let (_, t_transfer) = TaggedImplicit::<Oid, Self::Error, 1>::parse_ber(rem)?;
                PdvIdentification::Syntaxes {
                    s_abstract: t_abstract.into_inner(),
                    s_transfer: t_transfer.into_inner(),
                }
            }
            Tag(1) => {
                // syntax OBJECT IDENTIFIER
                let (_, oid) = Oid::from_any_ber(inner0.data, inner0.header)?;
                PdvIdentification::Syntax(oid)
            }
            Tag(2) => {
                // presentation-context-id INTEGER
                let (_, i) = Integer::from_any_ber(inner0.data, inner0.header)?;
                PdvIdentification::PresentationContextId(i)
            }
            Tag(3) => {
                // context-negotiation SEQUENCE {
                //     presentation-context-id INTEGER,
                //     transfer-syntax OBJECT IDENTIFIER
                // },
                let inner_data = inner0.data;
                let (rem, t_presentation_context_id) =
                    TaggedImplicit::<Integer, Self::Error, 0>::parse_ber(inner_data)?;
                let (_, t_presentation_syntax) =
                    TaggedImplicit::<Oid, Self::Error, 1>::parse_ber(rem)?;

                let presentation_context_id = t_presentation_context_id.into_inner();
                let presentation_syntax = t_presentation_syntax.into_inner();
                PdvIdentification::ContextNegotiation {
                    presentation_context_id,
                    presentation_syntax,
                }
            }
            Tag(4) => {
                // transfer-syntax OBJECT IDENTIFIER
                let (_, oid) = Oid::from_any_ber(inner0.data, inner0.header)?;
                PdvIdentification::TransferSyntax(oid)
            }
            Tag(5) => {
                // fixed NULL
                PdvIdentification::Fixed
            }
            _ => {
                let e = InnerError::invalid_value(
                    inner0.tag(),
                    "Invalid identification tag in EMBEDDED PDV",
                );
                return Err(BerError::nom_err_input(&inner0.data, e));
            }
        };
        // [1] -> data-value-descriptor ObjectDescriptor OPTIONAL
        // *BUT* WITH COMPONENTS data-value-descriptor ABSENT
        let (rem, t1) = <Option<TaggedImplicit<ObjectDescriptor, Self::Error, 1>>>::parse_ber(rem)?;
        let data_value_descriptor = t1.map(|o| o.into_inner());
        // [2] -> data-value OCTET STRING
        let (rem, t2) = TaggedImplicit::<&[u8], Self::Error, 2>::parse_ber(rem)?;
        let data_value = t2.into_inner();

        let obj = EmbeddedPdv {
            identification,
            data_value_descriptor,
            data_value,
        };
        Ok((rem, obj))
    }
}

impl<'i> DerParser<'i> for EmbeddedPdv<'i> {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::EmbeddedPdv
    }

    fn from_any_der(input: Input<'i>, header: Header<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        Self::from_any_ber(input, header)
    }
}

impl CheckDerConstraints for EmbeddedPdv<'_> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length().assert_definite()?;
        any.header.assert_constructed()?;
        Ok(())
    }
}

impl DerAutoDerive for EmbeddedPdv<'_> {}
