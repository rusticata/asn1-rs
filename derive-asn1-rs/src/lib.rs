mod sequence;
use sequence::*;

synstructure::decl_derive!([BerSequence] => derive_ber_sequence);
synstructure::decl_derive!([DerSequence] => derive_der_sequence);
