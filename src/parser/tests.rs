// These contain testdata
mod games;
mod packages;
mod proofs;

// these contain tests:

/// The complete module contains tests whether things that are supposed to parse actually parse
/// into what is expected.
mod complete;

/// The sound module contains tests whether things that are not supposed to parse actually produce
/// errors - and the right ones.
mod sound;
