#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub enum __va_list_tag {}

// TODO: get bindgen to emit these defines
pub const NV_VERSION: ::std::os::raw::c_int = 0;

pub const NV_ENCODE_NATIVE: ::std::os::raw::c_int = 0;
pub const NV_ENCODE_XDR: ::std::os::raw::c_int = 0;

pub const NV_UNIQUE_NAME: ::std::os::raw::c_uint = 1;
pub const NV_UNIQUE_NAME_TYPE: ::std::os::raw::c_uint = 2;

pub const NV_FLAG_NOENTOK: ::std::os::raw::c_int = 1;

include!("bindings.rs");
