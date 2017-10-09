#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


extern crate libc;

use libc::FILE;
pub enum __va_list_tag {}

include!("bindings.rs");
