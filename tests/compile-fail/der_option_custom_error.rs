use asn1_rs::{Error, FromDer, ParseResult};

pub struct CustomError;

impl From<Error> for CustomError {
    fn from(_value: Error) -> Self {
        CustomError
    }
}

pub type T = Option<u32>;

// NOTE: this fails, because Option::FromDer is only available with default
// error type.
// It cannot be implemented for custom errors, because it would conflict with
// default implementation because of default infaillible T -> Option<T> implementation
fn should_fail_for_now<'a>(input: &'a [u8]) -> ParseResult<'a, T, CustomError> {
    T::from_der(input)
}

fn main() {
    let _t = should_fail_for_now(&[0x00, 0x01, 0x02, 0x03]);
}
