extern crate nvpair_sys as sys;
extern crate cstr_argument;
#[macro_use]
extern crate foreign_types;

use cstr_argument::CStrArgument;
use std::mem;
use std::io;
use std::ptr;
use std::ffi;
use std::os::raw::{c_int, c_uint};
pub use foreign_types::{Opaque, ForeignTypeRef, ForeignType};

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
    Int64(i64),
    Uint64(u64),
    String(ffi::CString),
    NvListRef(NvList),
    // TODO: arrays
    // hrtime
    // double
}

pub trait NvEncode {
    fn insert<S: CStrArgument>(&self, S, &mut NvListRef) -> io::Result<()>;
    //fn read(NvPair &nv) -> io::Result<Self>;
}

impl NvEncode for bool {
    fn insert<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_boolean_value(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                if *self {
                    sys::boolean::B_TRUE
                } else {
                    sys::boolean::B_FALSE
                },
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
    fn insert<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_uint32(nv.as_mut_ptr(), name.as_ref().as_ptr(), *self) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for ffi::CStr {
    fn insert<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_string(nv.as_mut_ptr(), name.as_ref().as_ptr(), self.as_ptr())
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for NvListRef {
    fn insert<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_nvlist(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                self.as_ptr() as *mut _,
            )
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
    Xdr,
}

impl NvEncoding {
    fn as_raw(&self) -> c_int {
        match self {
            &NvEncoding::Native => sys::NV_ENCODE_NATIVE,
            &NvEncoding::Xdr => sys::NV_ENCODE_XDR,
        }
    }
}

foreign_type! {
    type CType = sys::nvlist;
    fn drop = sys::nvlist_free;
    /// An `NvList`
    pub struct NvList;
    /// A borrowed `NvList`
    pub struct NvListRef;
}

impl NvList {
    /// Create a new `NvList` with no options
    pub fn new() -> io::Result<Self> {
        let mut n = ptr::null_mut();
        let v = unsafe {
            // TODO: second arg is a bitfield of NV_UNIQUE_NAME|NV_UNIQUE_NAME_TYPE
            sys::nvlist_alloc(&mut n, 0, 0)
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { Self::from_ptr(n) })
        }
    }

    /// Create a new `NvList` with the `NV_UNIQUE_NAME` constraint
    pub fn new_unqiue_names() -> io::Result<Self> {
        let mut n = ptr::null_mut();
        let v = unsafe { sys::nvlist_alloc(&mut n, sys::NV_UNIQUE_NAME, 0) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { Self::from_ptr(n) })
        }
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        let mut n = ptr::null_mut();
        let v = unsafe { sys::nvlist_dup(self.0, &mut n, 0) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { Self::from_ptr(n) })
        }
    }
}

impl Clone for NvList {
    fn clone(&self) -> Self {
        self.try_clone().unwrap()
    }
}

impl NvListRef {
    pub unsafe fn from_mut_ptr<'a>(v: *mut sys::nvlist) -> &'a mut Self {
        std::mem::transmute::<*mut sys::nvlist, &mut Self>(v)
    }

    pub unsafe fn from_ptr<'a>(v: *const sys::nvlist) -> &'a Self {
        std::mem::transmute::<*const sys::nvlist, &Self>(v)
    }

    pub fn as_mut_ptr(&mut self) -> *mut sys::nvlist {
        unsafe { std::mem::transmute::<&mut NvListRef, *mut sys::nvlist>(self) }
    }

    pub fn as_ptr(&self) -> *const sys::nvlist {
        unsafe { std::mem::transmute::<&NvListRef, *const sys::nvlist>(self) }
    }

    pub fn encoded_size(&self, encoding: NvEncoding) -> io::Result<usize> {
        let mut l = 0usize;
        let v = unsafe { sys::nvlist_size(self.as_ptr() as *mut _, &mut l, encoding.as_raw()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(l)
        }
    }

    pub fn is_empty(&self) -> bool {
        let v = unsafe { sys::nvlist_empty(self.as_ptr() as *mut _) };
        v != sys::boolean::B_FALSE
    }

    pub fn add_boolean<S: CStrArgument>(&mut self, name: S) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_boolean(self.as_mut_ptr(), name.as_ref().as_ptr()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    pub fn first(&self) -> Option<&NvPair> {
        let np = unsafe { sys::nvlist_next_nvpair(self.as_ptr() as *mut _, ptr::null_mut()) };
        if np.is_null() {
            None
        } else {
            Some(unsafe { NvPair::from_ptr(np) })
        }
    }

    pub fn iter(&self) -> NvListIter {
        NvListIter {
            parent: self,
            pos: ptr::null_mut(),
        }
    }

    pub fn exists<S: CStrArgument>(&self, name: S) -> bool {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_exists(self.as_ptr() as *mut _, name.as_ref().as_ptr()) };
        v != sys::boolean::B_FALSE
    }

    /* 
    // not allowed because `pair` is borrowed from `self`. Need to fiddle around so that we can
    // check:
    //  - `pair` is from `self`
    //  - `pair` is the only outstanding reference to this pair (need by-value semantics)
    pub fn remove(&mut self, pair: &NvPair) -> io::Result<()>
    {
        let v = unsafe { sys::nvlist_remove_nvpair(self.as_mut_ptr(), pair.as_ptr())};
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
    */

    pub fn lookup<S: CStrArgument>(&self, name: S) -> io::Result<&NvPair> {
        let name = name.into_cstr();
        let mut n = ptr::null_mut();
        let v = unsafe {
            sys::nvlist_lookup_nvpair(self.as_ptr() as *mut _, name.as_ref().as_ptr(), &mut n)
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { NvPair::from_ptr(n) })
        }
    }

    pub fn lookup_nv_list<S: CStrArgument>(&self, name: S) -> io::Result<NvList> {
        let name = name.into_cstr();

        let mut n = ptr::null_mut();

        let v = unsafe {
            sys::nvlist_lookup_nvlist(self.as_ptr() as *mut _, name.as_ref().as_ptr(), &mut n)
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            let r = unsafe { NvList::from_ptr(n) };

            Ok(r)
        }
    }

    pub fn lookup_string<S: CStrArgument>(&self, name: S) -> io::Result<ffi::CString> {
        let name = name.into_cstr();
        let mut n;

        let v = unsafe {
            n = mem::uninitialized();

            sys::nvlist_lookup_string(self.as_ptr() as *mut _, name.as_ref().as_ptr(), &mut n)
        };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            let s = unsafe { ffi::CStr::from_ptr(n).to_owned() };
            Ok(s)
        }
    }

    pub fn lookup_uint64<S: CStrArgument>(&self, name: S) -> io::Result<u64> {
        let name = name.into_cstr();
        let mut n: u64;

        let v = unsafe {
            n = mem::uninitialized();

            sys::nvlist_lookup_uint64(self.as_ptr() as *mut _, name.as_ref().as_ptr(), &mut n)
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(n)
        }
    }


    pub fn lookup_nv_list_array<S: CStrArgument>(&self, name: S) -> io::Result<Vec<NvList>> {
        let name = name.into_cstr();

        let mut n = ptr::null_mut();

        let mut len: c_uint;

        let v = unsafe {
            len = mem::uninitialized();
            sys::nvlist_lookup_nvlist_array(
                self.as_ptr() as *mut _,
                name.as_ref().as_ptr(),
                &mut n,
                &mut len,
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            let r = unsafe {
                std::slice::from_raw_parts(n, len as usize)
                    .iter()
                    .map(|x| NvList::from_ptr(*x))
                    .collect()
            };

            Ok(r)
        }
    }

    pub fn lookup_uint64_array<S: CStrArgument>(&self, name: S) -> io::Result<Vec<u64>> {
        let name = name.into_cstr();

        let mut n = ptr::null_mut();

        let mut len: c_uint;

        let v = unsafe {
            len = mem::uninitialized();
            nv_sys::nvlist_lookup_uint64_array(
                self.as_ptr() as *mut _,
                name.as_ref().as_ptr(),
                &mut n,
                &mut len,
            )
        };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            let r = unsafe {
                ::std::slice::from_raw_parts(n, len as usize)
                    .iter()
                    .map(|x| *x)
                    .collect()
            };

            Ok(r)
        }
    }

    pub fn try_to_owned(&self) -> io::Result<NvList> {
        let mut n = NvList(ptr::null_mut());
        let v = unsafe { sys::nvlist_dup(self.as_ptr() as *mut _, &mut n.0, 0) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(n)
        }
    }
}

pub struct NvListIter<'a> {
    parent: &'a NvListRef,
    pos: *mut sys::nvpair,
}

impl<'a> Iterator for NvListIter<'a> {
    type Item = &'a NvPair;

    fn next(&mut self) -> Option<Self::Item> {
        let np = unsafe { sys::nvlist_next_nvpair(self.parent.as_ptr() as *mut _, self.pos) };
        self.pos = np;
        if np.is_null() {
            None
        } else {
            Some(unsafe { NvPair::from_ptr(np) })
        }
    }
}

pub struct NvPair(Opaque);
impl ForeignTypeRef for NvPair {
    type CType = sys::nvpair;
}

impl NvPair {
    pub fn name(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(sys::nvpair_name(self.as_ptr())) }
    }

    pub fn value_nv_list(&self) -> io::Result<NvList> {
        let mut nvl_target = ptr::null_mut();

        unsafe {
            let code = sys::nvpair_value_nvlist(self.as_ptr(), &mut nvl_target);

            if code == 0 {
                Ok(NvList::from_ptr(nvl_target))
            } else {
                Err(io::Error::from_raw_os_error(code))
            }
        }
    }
}
