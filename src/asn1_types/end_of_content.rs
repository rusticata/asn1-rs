use nom::{Err, IResult};

use crate::{
    impl_tryfrom_any, BerError, BerParser, Header, InnerError, Input, Result, Tag, Tagged,
};

/// End-of-contents octets
///
/// `EndOfContent` is not a BER type, but represents a marked to indicate the end of contents
/// of an object, when the length is `Indefinite` (see X.690 section 8.1.5).
///
/// This type cannot exist in DER, and so provides no `DerParser / FromDer`/`ToDer` implementation.
#[derive(Debug)]
pub struct EndOfContent {}

impl Default for EndOfContent {
    fn default() -> Self {
        Self::new()
    }
}

impl EndOfContent {
    pub const fn new() -> Self {
        EndOfContent {}
    }
}

impl_tryfrom_any!(EndOfContent);

impl<'i> BerParser<'i> for EndOfContent {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        if !header.length.is_null() {
            return Err(Err::Error(BerError::new(input, InnerError::InvalidLength)));
        }
        Ok((input, EndOfContent {}))
    }
}

impl Tagged for EndOfContent {
    const TAG: Tag = Tag::EndOfContent;
}

// impl ToDer for EndOfContent {
//     fn to_der_len(&self) -> Result<usize> {
//         Ok(2)
//     }

//     fn write_der_header(&self, writer: &mut dyn std::io::Write) -> crate::SerializeResult<usize> {
//         writer.write(&[Self::TAG.0 as u8, 0x00]).map_err(Into::into)
//     }

//     fn write_der_content(&self, _writer: &mut dyn std::io::Write) -> crate::SerializeResult<usize> {
//         Ok(0)
//     }
// }
