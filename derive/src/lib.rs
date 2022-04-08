mod container;
mod sequence;
mod set;
use sequence::*;
use set::*;

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

synstructure::decl_derive!([BerSet, attributes(
    debug_derive,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_ber_set);
synstructure::decl_derive!([DerSet, attributes(
    debug_derive,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_der_set);
