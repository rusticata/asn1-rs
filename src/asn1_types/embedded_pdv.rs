use crate::*;

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

impl Tagged for EmbeddedPdv<'_> {
    const CONSTRUCTED: bool = true;
    const TAG: Tag = Tag::EmbeddedPdv;
}

impl_tryfrom_any!('i @ EmbeddedPdv<'i>);

impl<'i> BerParser<'i> for EmbeddedPdv<'i> {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        _header: &'_ Header<'i>,
        input: Input<'i>,
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
                let (_, oid) = Oid::from_ber_content(&inner0.header, inner0.data)?;
                PdvIdentification::Syntax(oid)
            }
            Tag(2) => {
                // presentation-context-id INTEGER
                let (_, i) = Integer::from_ber_content(&inner0.header, inner0.data)?;
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
                let (_, oid) = Oid::from_ber_content(&inner0.header, inner0.data)?;
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

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        Self::from_ber_content(header, input)
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
