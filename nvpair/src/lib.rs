extern crate nvpair_sys as sys;
extern crate cstr_argument;

use cstr_argument::CStrArgument;
use std::io;
use std::ptr;
use std::ffi;
use std::os::raw::c_int;

pub enum NvEncoding {
    Native,
    Xdr
}

impl NvEncoding {
    fn as_raw(&self) -> c_int {
        match self {
            &NvEncoding::Native => sys::NV_ENCODE_NATIVE,
            &NvEncoding::Xdr => sys::NV_ENCODE_XDR
        }
    }
}

pub struct NvList {
    raw: *mut sys::nvlist
}

impl NvList {
    pub fn new() -> io::Result<Self> {
        let mut n = Self { raw: ptr::null_mut() };
        let v = unsafe {
            // TODO: second arg is a bitfield of NV_UNIQUE_NAME|NV_UNIQUE_NAME_TYPE
            sys::nvlist_alloc(&mut n.raw, 0, 0)
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(n) 
        }
    }

    pub fn encoded_size(&self, encoding: NvEncoding) -> io::Result<usize>
    {
        let mut l = 0usize;
        let v = unsafe {
            sys::nvlist_size(self.raw, &mut l, encoding.as_raw())
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(l)
        }
    }

    pub fn is_empty(&self) -> bool {
        let v = unsafe { sys::nvlist_empty(self.raw) };
        v != sys::boolean::B_FALSE
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        let mut n = Self { raw: ptr::null_mut() };
        let v = unsafe { sys::nvlist_dup(self.raw, &mut n.raw, 0) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(n)
        }
    }

    pub fn add_boolean<S: CStrArgument>(&mut self, name: S) -> io::Result<()>
    {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_boolean(self.raw, name.as_ref().as_ptr()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    pub fn first(&self) -> Option<NvPair> {
        let np = unsafe { sys::nvlist_next_nvpair(self.raw, ptr::null_mut()) };  
        if np.is_null() {
            None
        } else {
            Some(NvPair { parent: self, raw: np })
        }
    }
}

impl Clone for NvList {
    fn clone(&self) -> Self {
        self.try_clone().unwrap()
    }
}

impl Drop for NvList {
    fn drop(&mut self) {
        unsafe {
            sys::nvlist_free(self.raw);
        }
    }
}

pub struct NvPair<'a> {
    parent: &'a NvList,
    raw: *mut sys::nvpair
}

impl<'a> NvPair<'a> {
    pub fn next(&self) -> Option<NvPair>
    {
        let np = unsafe { sys::nvlist_next_nvpair(self.parent.raw, self.raw) };
        if np.is_null() {
            None
        } else {
            Some(NvPair { parent: self.parent, raw: np })
        }
    }

    pub fn name(&self) -> &ffi::CStr
    {
        unsafe { ffi::CStr::from_ptr(sys::nvpair_name(self.raw)) }
    }
}
