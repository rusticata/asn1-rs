mod alias;
mod asn1_type;
mod check_derive;
mod choice;
mod container;
mod enumerated;
mod options;
mod sequence;
mod set;
mod tostatic;

use alias::*;
use choice::*;
use enumerated::*;
use sequence::*;
use set::*;
use tostatic::derive_tostatic;

synstructure::decl_derive!([BerAlias, attributes(
    debug_derive,
    default,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_ber_alias);
synstructure::decl_derive!([DerAlias, attributes(
    debug_derive,
    default,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_der_alias);

synstructure::decl_derive!([BerSequence, attributes(
    debug_derive,
    default,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_ber_sequence);
synstructure::decl_derive!([DerSequence, attributes(
    debug_derive,
    default,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_der_sequence);

synstructure::decl_derive!([BerSet, attributes(
    debug_derive,
    default,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_ber_set);
synstructure::decl_derive!([DerSet, attributes(
    debug_derive,
    default,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_der_set);

synstructure::decl_derive!([ToStatic, attributes(
    debug_derive
)] => derive_tostatic);

synstructure::decl_derive!([ToBerSequence, attributes(
    debug_derive,
)] => derive_tober_sequence);

synstructure::decl_derive!([ToDerSequence, attributes(
    debug_derive,
)] => derive_toder_sequence);

synstructure::decl_derive!([ToDerSet, attributes(
    debug_derive,
)] => derive_toder_set);

synstructure::decl_derive!([ToBerSet, attributes(
    debug_derive,
)] => derive_tober_set);

//----------- new BerParser

synstructure::decl_derive!([BerParserSequence, attributes(
    debug_derive,
    default,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_berparser_sequence);

synstructure::decl_derive!([DerParserSequence, attributes(
    debug_derive,
    default,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_derparser_sequence);

synstructure::decl_derive!([BerParserSet, attributes(
    debug_derive,
    default,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_berparser_set);

synstructure::decl_derive!([DerParserSet, attributes(
    debug_derive,
    default,
    optional,
    tag_explicit,
    tag_implicit,
    error,
    map_err
)] => derive_derparser_set);

synstructure::decl_derive!([BerParserChoice, attributes(
    debug_derive,
    tagged_explicit,
    tagged_implicit,
)] => derive_berparser_choice);

synstructure::decl_derive!([DerParserChoice, attributes(
    debug_derive,
    tagged_explicit,
    tagged_implicit,
)] => derive_derparser_choice);

synstructure::decl_derive!([BerParserAlias, attributes(
    debug_derive
)] => derive_berparser_alias);

synstructure::decl_derive!([DerParserAlias, attributes(
    debug_derive
)] => derive_derparser_alias);

//--- new derive

synstructure::decl_derive!([Sequence, attributes(
    debug_derive,
    default,
    optional,
    error,
    map_err,
    tag_explicit,
    tag_implicit,
    asn1,
)] => derive_sequence);

synstructure::decl_derive!([Set, attributes(
    debug_derive,
    default,
    optional,
    error,
    map_err,
    tag_explicit,
    tag_implicit,
    asn1,
)] => derive_set);

synstructure::decl_derive!([Choice, attributes(
    debug_derive,
    tagged_explicit,
    tagged_implicit,
    tag,
    error,
    asn1,
)] => derive_choice);

synstructure::decl_derive!([Enumerated, attributes(
    debug_derive,
    error,
    asn1,
)] => derive_enumerated);

synstructure::decl_derive!([Alias, attributes(
    debug_derive,
    error,
    asn1,
)] => derive_alias);
