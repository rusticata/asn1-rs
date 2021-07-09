use crate::*;
use alloc::borrow::Cow;
use core::convert::TryFrom;
use core::marker::PhantomData;

impl<'a, T> TaggedValue<'a, Implicit, T> {
    pub const fn new_implicit(class: Class, constructed: bool, tag: u32, inner: T) -> Self {
        Self {
            header: Header::new(class, constructed, Tag(tag), Length::Definite(0)),
            inner,
            tag_kind: PhantomData,
        }
    }
}

impl<'a, T> FromBer<'a> for TaggedValue<'a, Implicit, T>
where
    T: TryFrom<Any<'a>, Error = Error>,
    T: Tagged,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, any) = Any::from_ber(bytes)?;
        let Any { header, data } = any;
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..header.clone()
            },
            data,
        };
        match T::try_from(any) {
            Ok(t) => {
                let tagged_value = TaggedValue {
                    header,
                    inner: t,
                    tag_kind: PhantomData,
                };
                Ok((rem, tagged_value))
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl<'a, T> FromDer<'a> for TaggedValue<'a, Implicit, T>
where
    T: TryFrom<Any<'a>, Error = Error>,
    T: CheckDerConstraints,
    T: Tagged,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, any) = Any::from_der(bytes)?;
        let Any { header, data } = any;
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..header.clone()
            },
            data,
        };
        T::check_constraints(&any)?;
        match T::try_from(any) {
            Ok(t) => {
                let tagged_value = TaggedValue {
                    header,
                    inner: t,
                    tag_kind: PhantomData,
                };
                Ok((rem, tagged_value))
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl<'a, T> CheckDerConstraints for TaggedValue<'a, Implicit, T>
where
    T: CheckDerConstraints,
    T: Tagged,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..any.header.clone()
            },
            data: Cow::Borrowed(&any.data),
        };
        T::check_constraints(&any)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<'a, T> ToDer for TaggedValue<'a, Implicit, T>
where
    T: ToDer,
{
    fn to_der_len(&self) -> Result<usize> {
        self.inner.to_der_len()
    }

    fn write_der(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let mut v = Vec::new();
        let inner_len = self.inner.write_der_content(&mut v)?;
        // XXX X.690 section 8.14.3: if implicing tagging was used [...]:
        // XXX a) the encoding shall be constructed if the base encoding is constructed, and shall be primitive otherwise
        let header = Header::new(self.class(), false, self.tag(), Length::Definite(inner_len));
        let sz = header.write_der_header(writer)?;
        let sz = sz + writer.write(&v)?;
        Ok(sz)
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let mut sink = std::io::sink();
        let inner_len = self.inner.write_der_content(&mut sink)?;
        // XXX X.690 section 8.14.3: if implicing tagging was used [...]:
        // XXX a) the encoding shall be constructed if the base encoding is constructed, and shall be primitive otherwise
        let header = Header::new(self.class(), false, self.tag(), Length::Definite(inner_len));
        header.write_der_header(writer).map_err(Into::into)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        self.inner.write_der_content(writer)
    }
}
