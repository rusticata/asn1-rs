use nom::{Err, IResult, Input as _};

use crate::{Any, AnyIterator, BerError, BerMode, BerParser, Header, InnerError, Input, Tag};

use super::ber_get_content;

pub trait Appendable {
    fn append(&mut self, other: &mut Self);
}

/// Parse object content recursively for segmented objects (constructed)
///
/// Notes:
/// - header of recursion entrypoint must be constructed
/// - `T` must be able to parse a primitive object using `BerParser`
pub(crate) fn parse_ber_segmented<'i, T>(
    header: &'_ Header<'i>,
    input: Input<'i>,
    recursion_limit: usize,
) -> IResult<Input<'i>, T, BerError<Input<'i>>>
where
    T: Appendable + Default,
    T: BerParser<'i, Error = BerError<Input<'i>>>,
{
    if recursion_limit == 0 {
        return Err(BerError::nom_err_input(&input, InnerError::BerMaxDepth));
    }

    if input.is_empty() {
        return Err(BerError::nom_err_input(&input, InnerError::InvalidLength));
    }

    if header.constructed() {
        let (rem, data) = if header.length.is_definite() {
            ber_get_content(header, input)?
        } else {
            // FIXME: previous get_length already consumed EndOfContent, so we cannot use it
            input.take_split(input.len())
        };
        let mut v = T::default();
        for res in AnyIterator::<BerMode>::new(data) {
            let (_, obj) = res.map_err(Err::Error)?;
            if obj.header.tag() == Tag::EndOfContent {
                break;
            }
            let Any {
                header: h2,
                data: data2,
            } = obj;
            if !<T as BerParser>::check_tag(h2.tag()) {
                return Err(BerError::nom_err_input(&data2, InnerError::InvalidTag));
            }
            // Empty segments are sometimes allowed (for ex: OctetStrings). Just skip recursion if empty
            if !data2.is_empty() {
                let (_, mut part_v) = parse_ber_segmented(&h2, data2, recursion_limit - 1)?;
                v.append(&mut part_v);
            }
        }
        Ok((rem, v))
    } else {
        let (rem, data) = ber_get_content(header, input)?;
        let (_, t) = T::from_ber_content(header, data)?;
        Ok((rem, t))
    }
}
