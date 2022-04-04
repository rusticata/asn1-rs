fn test_tag_errors() {
    use asn1_rs::*;

    /// Should not compile: both implicit and explicit tags
    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct T0 {
        #[tag_explicit(0)]
        #[tag_implicit(0)]
        a: u16,
    }

    /// Should not compile: multiple explicit tags
    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct T1 {
        #[tag_explicit(0)]
        #[tag_explicit(1)]
        a: u16,
    }
}

fn main() {
    test_tag_errors();
}
