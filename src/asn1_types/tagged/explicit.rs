use crate::*;
use alloc::borrow::Cow;
use core::marker::PhantomData;

impl<'a, T> TaggedValue<'a, Explicit, T> {
    pub const fn new_explicit(class: Class, tag: u32, inner: T) -> Self {
        Self {
            header: Header::new(class, true, Tag(tag), Length::Definite(0)),
            inner,
            tag_kind: PhantomData,
        }
    }
}

impl<'a, T> TaggedValue<'a, Explicit, T> {
    pub fn from_ber_and_then<F>(
        class: Class,
        tag: u32,
        bytes: &'a [u8],
        op: F,
    ) -> ParseResult<'a, T>
    where
        F: FnOnce(&'a [u8]) -> ParseResult<T>,
    {
        let (rem, any) = Any::from_ber(bytes)?;
        any.tag().assert_eq(Tag(tag))?;
        if any.class() != class {
            return Err(any.tag().invalid_value("Invalid class").into());
        }
        let data = any.into_borrowed()?;
        let (_, res) = op(data)?;
        Ok((rem, res))
    }

    pub fn from_der_and_then<F>(
        class: Class,
        tag: u32,
        bytes: &'a [u8],
        op: F,
    ) -> ParseResult<'a, T>
    where
        F: FnOnce(&'a [u8]) -> ParseResult<T>,
    {
        let (rem, any) = Any::from_der(bytes)?;
        any.tag().assert_eq(Tag(tag))?;
        if any.class() != class {
            return Err(any.tag().invalid_value("Invalid class").into());
        }
        let data = any.into_borrowed()?;
        let (_, res) = op(data)?;
        Ok((rem, res))
    }
}

impl<'a, T> FromBer<'a> for TaggedValue<'a, Explicit, T>
where
    T: FromBer<'a>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, any) = Any::from_ber(bytes)?;
        let header = any.header;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            Cow::Owned(_) => unreachable!(),
        };
        let (_, inner) = T::from_ber(data)?;
        let tagged = TaggedValue {
            header,
            inner,
            tag_kind: PhantomData,
        };
        Ok((rem, tagged))
    }
}

impl<'a, T> FromDer<'a> for TaggedValue<'a, Explicit, T>
where
    T: FromDer<'a>,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, any) = Any::from_der(bytes)?;
        let header = any.header;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            Cow::Owned(_) => unreachable!(),
        };
        let (_, inner) = T::from_der(data)?;
        let tagged = TaggedValue {
            header,
            inner,
            tag_kind: PhantomData,
        };
        Ok((rem, tagged))
    }
}

impl<'a, T> CheckDerConstraints for TaggedValue<'a, Explicit, T>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        let (_, inner_any) = Any::from_der(&any.data)?;
        T::check_constraints(&inner_any)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<'a, T> ToDer for TaggedValue<'a, Explicit, T>
where
    T: ToDer,
{
    fn to_der_len(&self) -> Result<usize> {
        let sz = self.inner.to_der_len()?;
        if sz < 127 {
            // 1 (class+tag) + 1 (length) + len
            Ok(2 + sz)
        } else {
            // 1 (class+tag) + n (length) + len
            let n = Length::Definite(sz).to_der_len()?;
            Ok(1 + n + sz)
        }
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let inner_len = self.inner.to_der_len()?;
        let header = Header::new(self.class(), true, self.tag(), Length::Definite(inner_len));
        header.write_der_header(writer).map_err(Into::into)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        self.inner.write_der(writer)
    }
}
