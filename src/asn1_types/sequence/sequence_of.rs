use crate::*;
use alloc::vec::Vec;
use core::convert::TryFrom;

/// The `SEQUENCE OF` object is an ordered list of homogeneous types.
#[derive(Debug)]
pub struct SequenceOf<T> {
    pub(crate) items: Vec<T>,
}

impl<T> SequenceOf<T> {
    pub const fn new(items: Vec<T>) -> Self {
        SequenceOf { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<'a, T> AsRef<[T]> for SequenceOf<T> {
    fn as_ref(&self) -> &[T] {
        &self.items
    }
}

impl<'a, T> IntoIterator for &'a SequenceOf<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> core::slice::Iter<'a, T> {
        self.items.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SequenceOf<T> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> core::slice::IterMut<'a, T> {
        self.items.iter_mut()
    }
}

impl<T> From<SequenceOf<T>> for Vec<T> {
    fn from(set: SequenceOf<T>) -> Self {
        set.items
    }
}

impl<'a, T> TryFrom<Any<'a>> for SequenceOf<T>
where
    T: FromBer<'a>,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        any.tag().assert_eq(Self::TAG)?;
        if !any.header.is_constructed() {
            return Err(Error::ConstructExpected);
        }
        let items = SequenceIterator::<T, BerParser>::new(any.data).collect::<Result<Vec<T>>>()?;
        Ok(SequenceOf::new(items))
    }
}

impl<T> CheckDerConstraints for SequenceOf<T>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_constructed()?;
        for item in SequenceIterator::<Any, DerParser>::new(&any.data) {
            let item = item?;
            T::check_constraints(&item)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T> ToDer for SequenceOf<T>
where
    T: ToDer,
{
    fn to_der_len(&self) -> Result<usize> {
        self.items.to_der_len()
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        self.items.write_der_header(writer)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        self.items.write_der_content(writer)
    }
}
