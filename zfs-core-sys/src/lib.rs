#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(deref_nullptr)]

extern crate nvpair_sys as nvpair;
use nvpair::*;

include!("bindings.rs");
