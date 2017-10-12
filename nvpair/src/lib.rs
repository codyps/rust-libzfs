extern crate nvpair_sys as sys;
extern crate cstr_argument;

use cstr_argument::CStrArgument;
use std::marker::PhantomData;
use std::io;
use std::ptr;
use std::ffi;
use std::os::raw::c_int;

pub enum NvData {
    Bool,
    BoolV(bool),
    Byte(u8),
    Int8(i8),
    Uint8(i8),
    Int16(i16),
    Uint16(u16),
    Int32(i32),
    Uint32(u32),
    String(ffi::CString),
    NvList(NvList),
    // TODO: arrays
    // hrtime
    // double
}

pub trait NvEncode {
    fn insert<S: CStrArgument>(&self, S, &mut NvList) -> io::Result<()>;
    //fn read(NvPair &nv) -> io::Result<Self>;
}

impl NvEncode for bool {
    fn insert<S: CStrArgument>(&self, name: S, nv: &mut NvList) -> io::Result<()>
    {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_boolean_value(nv.as_ptr(), name.as_ref().as_ptr(),
                if *self {
                    sys::boolean::B_TRUE
                } else {
                    sys::boolean::B_FALSE
                }
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for u32 {
    fn insert<S: CStrArgument>(&self, name: S, nv: &mut NvList) -> io::Result<()>
    {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_uint32(nv.as_ptr(), name.as_ref().as_ptr(), *self)
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for ffi::CStr {
    fn insert<S: CStrArgument>(&self, name: S, nv: &mut NvList) -> io::Result<()>
    {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_string(nv.as_ptr(), name.as_ref().as_ptr(), self.as_ptr())
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for NvList {
    fn insert<S: CStrArgument>(&self, name: S, nv: &mut NvList) -> io::Result<()>
    {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_nvlist(nv.as_ptr(), name.as_ref().as_ptr(), self.as_ptr())
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

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
    pub unsafe fn from_ptr(v: *mut sys::nvlist) -> Self
    {
        NvList { raw: v }
    }

    pub fn as_ptr(&self) -> *mut sys::nvlist
    {
        self.raw
    }

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

    pub fn new_unqiue_names() -> io::Result<Self> {
        let mut n = Self { raw: ptr::null_mut() };
        let v = unsafe {
            sys::nvlist_alloc(&mut n.raw, sys::NV_UNIQUE_NAME, 0)
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
            Some(NvPair { parent: PhantomData, raw: np })
        }
    }

    pub fn iter(&self) -> NvListIter {
        NvListIter {
            parent: self,
            pos: ptr::null_mut(),
        }
    }

    pub fn exists<S: CStrArgument>(&self, name: S) -> bool
    {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_exists(self.raw, name.as_ref().as_ptr()) };
        v != sys::boolean::B_FALSE
    }

    pub fn remove(&self, pair: &NvPair) -> io::Result<()>
    {
        let v = unsafe { sys::nvlist_remove_nvpair(self.as_ptr(), pair.as_ptr())};
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    pub fn lookup<S: CStrArgument>(&self, name: S) -> io::Result<NvPair>
    {
        let name = name.into_cstr();
        let mut n = NvPair { parent: PhantomData, raw: ptr::null_mut() };
        let v = unsafe { sys::nvlist_lookup_nvpair(self.as_ptr(), name.as_ref().as_ptr(), &mut n.raw) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(n)
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

pub struct NvListIter<'a> {
    parent: &'a NvList,
    pos: *mut sys::nvpair, 
}

impl<'a> Iterator for NvListIter<'a> {
    type Item = NvPair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let np = unsafe { sys::nvlist_next_nvpair(self.parent.raw, self.pos) };  
        self.pos = np;
        if np.is_null() {
            None
        } else {
            Some(NvPair { parent: PhantomData, raw: np })
        }
    }
}


// TODO: Consider changing this into a CStr style type
pub struct NvPair<'a> {
    parent: PhantomData<&'a ()>,
    raw: *mut sys::nvpair
}

impl<'a> NvPair<'a> {
    pub fn as_ptr(&self) -> *mut sys::nvpair
    {
        self.raw
    }

    pub fn name(&self) -> &ffi::CStr
    {
        unsafe { ffi::CStr::from_ptr(sys::nvpair_name(self.as_ptr())) }
    }
}
