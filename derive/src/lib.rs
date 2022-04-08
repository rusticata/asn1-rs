mod sequence;
use sequence::*;

synstructure::decl_derive!([BerSequence, attributes(
    debug_derive,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_ber_sequence);
synstructure::decl_derive!([DerSequence, attributes(
    debug_derive,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_der_sequence);
