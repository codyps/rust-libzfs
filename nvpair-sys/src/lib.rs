#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


pub enum __va_list_tag {}

pub const NV_ENCODE_NATIVE: ::std::os::raw::c_int = 0;
pub const NV_ENCODE_XDR: ::std::os::raw::c_int = 0;

include!("bindings.rs");
