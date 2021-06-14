use crate::{FromBer, FromDer, Tag, Tagged};

pub trait Choice<'a> {
    /// Is the provided [`Tag`] decodable as a variant of this `CHOICE`?
    fn can_decode(tag: Tag) -> bool;
}

/// This blanket impl allows any [`Tagged`] type to function as a [`Choice`]
/// with a single alternative.
impl<'a, T> Choice<'a> for T
where
    T: Tagged,
{
    fn can_decode(tag: Tag) -> bool {
        T::TAG == tag
    }
}

pub trait BerChoice<'a>: Choice<'a> + FromBer<'a> {}

pub trait DerChoice<'a>: Choice<'a> + FromDer<'a> {}
