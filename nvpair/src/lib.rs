#![warn(missing_debug_implementations, rust_2018_idioms)]

use cstr_argument::CStrArgument;
use foreign_types::{foreign_type, ForeignType, ForeignTypeRef, Opaque};
use nvpair_sys as sys;
use std::mem::MaybeUninit;
use std::os::raw::c_int;
use std::{ffi, fmt, io, ptr};

#[derive(Debug)]
pub enum NvData<'a> {
    Unknown,
    Bool,
    BoolV(bool),
    Byte(u8),
    Int8(i8),
    Uint8(u8),
    Int16(i16),
    Uint16(u16),
    Int32(i32),
    Uint32(u32),
    Int64(i64),
    Uint64(u64),
    Str(&'a ffi::CStr),
    NvListRef(&'a NvListRef),
    ByteArray(&'a [u8]),
    Int8Array(&'a [i8]),
    Uint8Array(&'a [u8]),
    Int16Array(&'a [i16]),
    Uint16Array(&'a [u16]),
    Int32Array(&'a [i32]),
    Uint32Array(&'a [u32]),
    Int64Array(&'a [i64]),
    Uint64Array(&'a [u64]),
    NvListRefArray(Vec<&'a NvListRef>),
    /* TODO:
    pub const DATA_TYPE_STRING_ARRAY: Type = 17;
    pub const DATA_TYPE_HRTIME: Type = 18;
    pub const DATA_TYPE_BOOLEAN_ARRAY: Type = 24;
    pub const DATA_TYPE_DOUBLE: Type = 27;
    */
}

pub trait NvEncode {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()>;
    //fn read(NvPair &nv) -> io::Result<Self>;
}

impl NvEncode for bool {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_boolean_value(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                if *self {
                    sys::boolean_t::B_TRUE
                } else {
                    sys::boolean_t::B_FALSE
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

impl NvEncode for i8 {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_int8(nv.as_mut_ptr(), name.as_ref().as_ptr(), *self) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for u8 {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_uint8(nv.as_mut_ptr(), name.as_ref().as_ptr(), *self) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for i16 {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_int16(nv.as_mut_ptr(), name.as_ref().as_ptr(), *self) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for u16 {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_uint16(nv.as_mut_ptr(), name.as_ref().as_ptr(), *self) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for i32 {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_int32(nv.as_mut_ptr(), name.as_ref().as_ptr(), *self) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for u32 {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_uint32(nv.as_mut_ptr(), name.as_ref().as_ptr(), *self) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for i64 {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_int64(nv.as_mut_ptr(), name.as_ref().as_ptr(), *self) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for u64 {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_uint64(nv.as_mut_ptr(), name.as_ref().as_ptr(), *self) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for [i8] {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_int8_array(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                self.as_ptr() as *mut i8,
                self.len() as u32,
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for [u8] {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_uint8_array(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                self.as_ptr() as *mut u8,
                self.len() as u32,
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for [i16] {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_int16_array(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                self.as_ptr() as *mut i16,
                self.len() as u32,
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for [u16] {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_uint16_array(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                self.as_ptr() as *mut u16,
                self.len() as u32,
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for [i32] {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_int32_array(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                self.as_ptr() as *mut i32,
                self.len() as u32,
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for [u32] {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_uint32_array(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                self.as_ptr() as *mut u32,
                self.len() as u32,
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for [i64] {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_int64_array(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                self.as_ptr() as *mut i64,
                self.len() as u32,
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for [u64] {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::nvlist_add_uint64_array(
                nv.as_mut_ptr(),
                name.as_ref().as_ptr(),
                self.as_ptr() as *mut u64,
                self.len() as u32,
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for ffi::CStr {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
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

impl NvEncode for &str {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        std::ffi::CString::new(*self).unwrap().insert_into(name, nv)
    }
}

impl NvEncode for NvListRef {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
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

impl NvEncode for () {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_add_boolean(nv.as_mut_ptr(), name.as_ref().as_ptr()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl NvEncode for str {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        ffi::CString::new(self).unwrap().insert_into(name, nv)
    }
}

#[derive(Debug)]
pub enum NvEncoding {
    Native,
    Xdr,
}

impl NvEncoding {
    fn as_raw(&self) -> c_int {
        match self {
            NvEncoding::Native => sys::NV_ENCODE_NATIVE,
            NvEncoding::Xdr => sys::NV_ENCODE_XDR,
        }
    }
}

foreign_type! {
    /// An `NvList`
    pub unsafe type NvList {
        type CType = sys::nvlist;
        fn drop = sys::nvlist_free;
    }
}

impl NvList {
    /// Try to create a new `NvList` with no options.
    ///
    /// Returns an error if memory allocation fails
    #[doc(alias = "nvlist_alloc")]
    pub fn try_new() -> io::Result<Self> {
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

    /// Try to create a new `NvList` with the `NV_UNIQUE_NAME` constraint
    ///
    /// Returns an error if memory allocation fails
    #[doc(alias = "nvlist_alloc")]
    pub fn try_new_unique_names() -> io::Result<Self> {
        let mut n = ptr::null_mut();
        let v = unsafe { sys::nvlist_alloc(&mut n, sys::NV_UNIQUE_NAME, 0) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { Self::from_ptr(n) })
        }
    }

    /// Create a new `NvList` with no options
    ///
    /// # Panics
    ///
    ///  - if [`try_new()`] returns an error
    #[doc(alias = "nvlist_alloc")]
    pub fn new() -> Self {
        Self::try_new().unwrap()
    }

    /// Create a new `NvList` with the `NV_UNIQUE_NAME` constraint
    ///
    /// # Panics
    ///
    ///  - if [`try_new_unique_names()`] returns an error
    #[doc(alias = "nvlist_alloc")]
    pub fn new_unique_names() -> Self {
        Self::try_new_unique_names().unwrap()
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        let mut n = ptr::null_mut();
        let v = unsafe { sys::nvlist_dup(self.as_ptr(), &mut n, 0) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { Self::from_ptr(n) })
        }
    }
}

impl Default for NvList {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for NvList {
    fn clone(&self) -> Self {
        self.try_clone().unwrap()
    }
}

impl NvListRef {
    /// # Safety
    ///
    /// Must be passed a valid, non-null nvlist pointer that is mutable, with a lifetime of at
    /// least `'a`
    pub unsafe fn from_mut_ptr<'a>(v: *mut sys::nvlist) -> &'a mut Self {
        &mut *(v as *mut Self)
    }

    /// # Safety
    ///
    /// Must be passed a valid, non-null nvlist pointer with a lifetime of at least `'a`.
    pub unsafe fn from_ptr<'a>(v: *const sys::nvlist) -> &'a Self {
        &*(v as *const Self)
    }

    pub fn as_mut_ptr(&mut self) -> *mut sys::nvlist {
        unsafe { std::mem::transmute::<&mut NvListRef, *mut sys::nvlist>(self) }
    }

    pub fn as_ptr(&self) -> *const sys::nvlist {
        unsafe { std::mem::transmute::<&NvListRef, *const sys::nvlist>(self) }
    }

    pub fn encoded_size(&self, encoding: NvEncoding) -> io::Result<u64> {
        let mut l = 0u64;
        let v = unsafe { sys::nvlist_size(self.as_ptr() as *mut _, &mut l, encoding.as_raw()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(l)
        }
    }

    pub fn is_empty(&self) -> bool {
        let v = unsafe { sys::nvlist_empty(self.as_ptr() as *mut _) };
        v != sys::boolean_t::B_FALSE
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

    pub fn iter(&self) -> NvListIter<'_> {
        NvListIter {
            parent: self,
            pos: ptr::null_mut(),
        }
    }

    pub fn exists<S: CStrArgument>(&self, name: S) -> bool {
        let name = name.into_cstr();
        let v = unsafe { sys::nvlist_exists(self.as_ptr() as *mut _, name.as_ref().as_ptr()) };
        v != sys::boolean_t::B_FALSE
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

    pub fn try_to_owned(&self) -> io::Result<NvList> {
        let mut n = MaybeUninit::uninit();
        let v = unsafe { sys::nvlist_dup(self.as_ptr() as *mut _, n.as_mut_ptr(), 0) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { NvList::from_ptr(n.assume_init()) })
        }
    }

    pub fn lookup_nvlist<S: CStrArgument>(&self, name: S) -> io::Result<NvList> {
        let name = name.into_cstr();

        let mut n = MaybeUninit::uninit();
        let v = unsafe {
            sys::nvlist_lookup_nvlist(
                self.as_ptr() as *mut _,
                name.as_ref().as_ptr(),
                n.as_mut_ptr(),
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            let r = unsafe { NvList::from_ptr(n.assume_init()) };
            Ok(r)
        }
    }

    pub fn lookup_string<S: CStrArgument>(&self, name: S) -> io::Result<ffi::CString> {
        let name = name.into_cstr();
        let mut n = MaybeUninit::uninit();
        let v = unsafe {
            sys::nvlist_lookup_string(
                self.as_ptr() as *mut _,
                name.as_ref().as_ptr(),
                n.as_mut_ptr(),
            )
        };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            let s = unsafe { ffi::CStr::from_ptr(n.assume_init()).to_owned() };
            Ok(s)
        }
    }

    pub fn lookup_uint64<S: CStrArgument>(&self, name: S) -> io::Result<u64> {
        let name = name.into_cstr();
        let mut n = MaybeUninit::uninit();
        let v = unsafe {
            sys::nvlist_lookup_uint64(
                self.as_ptr() as *mut _,
                name.as_ref().as_ptr(),
                n.as_mut_ptr(),
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { n.assume_init() })
        }
    }

    pub fn lookup_nvlist_array<S: CStrArgument>(&self, name: S) -> io::Result<Vec<NvList>> {
        let name = name.into_cstr();
        let mut n = ptr::null_mut();
        let mut len = 0;
        let v = unsafe {
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
        let mut len = 0;
        let v = unsafe {
            sys::nvlist_lookup_uint64_array(
                self.as_ptr() as *mut _,
                name.as_ref().as_ptr(),
                &mut n,
                &mut len,
            )
        };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            let r = unsafe { ::std::slice::from_raw_parts(n, len as usize).to_vec() };

            Ok(r)
        }
    }

    // TODO: consider renaming to `try_insert()` and having a `insert()` with an inner unwrap.
    pub fn insert<S: CStrArgument, D: NvEncode + ?Sized>(
        &mut self,
        name: S,
        data: &D,
    ) -> io::Result<()> {
        data.insert_into(name, self)
    }
}

impl std::fmt::Debug for NvList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.iter()
                    .map(|pair| (pair.name().to_owned().into_string().unwrap(), pair.data())),
            )
            .finish()
    }
}

impl std::fmt::Debug for NvListRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.iter()
                    .map(|pair| (pair.name().to_owned().into_string().unwrap(), pair.data())),
            )
            .finish()
    }
}

impl<'a> IntoIterator for &'a NvListRef {
    type Item = &'a NvPair;
    type IntoIter = NvListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a NvList {
    type Item = &'a NvPair;
    type IntoIter = NvListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone)]
pub struct NvListIter<'a> {
    parent: &'a NvListRef,
    pos: *mut sys::nvpair,
}

impl<'a> fmt::Debug for NvListIter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
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
unsafe impl ForeignTypeRef for NvPair {
    type CType = sys::nvpair;
}

impl NvPair {
    pub fn name(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(sys::nvpair_name(self.as_ptr())) }
    }

    // TODO: consider defering decode here until actually requested by the caller. Users of
    // `data` might not care to decode most data types, meaning we're wasting time with the
    // various `nvpair_value_*()` calls some of the time.
    pub fn data(&self) -> NvData<'_> {
        let data_type = unsafe { sys::nvpair_type(self.as_ptr()) };

        match data_type {
            sys::data_type_t::DATA_TYPE_BOOLEAN => NvData::Bool,
            sys::data_type_t::DATA_TYPE_BOOLEAN_VALUE => {
                let v = unsafe {
                    let mut v = MaybeUninit::uninit();
                    sys::nvpair_value_boolean_value(self.as_ptr(), v.as_mut_ptr());
                    v.assume_init()
                };

                NvData::BoolV(v == sys::boolean_t::B_TRUE)
            }
            sys::data_type_t::DATA_TYPE_BYTE => {
                let v = unsafe {
                    let mut v = MaybeUninit::uninit();
                    sys::nvpair_value_byte(self.as_ptr(), v.as_mut_ptr());
                    v.assume_init()
                };

                NvData::Byte(v)
            }
            sys::data_type_t::DATA_TYPE_INT8 => {
                let v = unsafe {
                    let mut v = MaybeUninit::uninit();
                    sys::nvpair_value_int8(self.as_ptr(), v.as_mut_ptr());
                    v.assume_init()
                };

                NvData::Int8(v)
            }
            sys::data_type_t::DATA_TYPE_UINT8 => {
                let v = unsafe {
                    let mut v = MaybeUninit::uninit();
                    sys::nvpair_value_uint8(self.as_ptr(), v.as_mut_ptr());
                    v.assume_init()
                };

                NvData::Uint8(v)
            }
            sys::data_type_t::DATA_TYPE_INT16 => {
                let v = unsafe {
                    let mut v = MaybeUninit::uninit();
                    sys::nvpair_value_int16(self.as_ptr(), v.as_mut_ptr());
                    v.assume_init()
                };

                NvData::Int16(v)
            }
            sys::data_type_t::DATA_TYPE_UINT16 => {
                let v = unsafe {
                    let mut v = MaybeUninit::uninit();
                    sys::nvpair_value_uint16(self.as_ptr(), v.as_mut_ptr());
                    v.assume_init()
                };

                NvData::Uint16(v)
            }
            sys::data_type_t::DATA_TYPE_INT32 => {
                let v = unsafe {
                    let mut v = MaybeUninit::uninit();
                    sys::nvpair_value_int32(self.as_ptr(), v.as_mut_ptr());
                    v.assume_init()
                };

                NvData::Int32(v)
            }
            sys::data_type_t::DATA_TYPE_UINT32 => {
                let v = unsafe {
                    let mut v = MaybeUninit::uninit();
                    sys::nvpair_value_uint32(self.as_ptr(), v.as_mut_ptr());
                    v.assume_init()
                };

                NvData::Uint32(v)
            }
            sys::data_type_t::DATA_TYPE_INT64 => {
                let v = unsafe {
                    let mut v = MaybeUninit::uninit();
                    sys::nvpair_value_int64(self.as_ptr(), v.as_mut_ptr());
                    v.assume_init()
                };

                NvData::Int64(v)
            }
            sys::data_type_t::DATA_TYPE_UINT64 => {
                let v = unsafe {
                    let mut v = MaybeUninit::uninit();
                    sys::nvpair_value_uint64(self.as_ptr(), v.as_mut_ptr());
                    v.assume_init()
                };

                NvData::Uint64(v)
            }
            sys::data_type_t::DATA_TYPE_STRING => {
                let s = unsafe {
                    let mut n = MaybeUninit::uninit();
                    sys::nvpair_value_string(self.as_ptr(), n.as_mut_ptr());
                    ffi::CStr::from_ptr(n.assume_init())
                };

                NvData::Str(s)
            }
            sys::data_type_t::DATA_TYPE_NVLIST => {
                let l = unsafe {
                    let mut l = MaybeUninit::uninit();
                    sys::nvpair_value_nvlist(self.as_ptr(), l.as_mut_ptr());
                    NvListRef::from_ptr(l.assume_init())
                };

                NvData::NvListRef(l)
            }
            sys::data_type_t::DATA_TYPE_BYTE_ARRAY => {
                let slice = unsafe {
                    let mut array = MaybeUninit::uninit();
                    let mut len = MaybeUninit::uninit();
                    sys::nvpair_value_byte_array(
                        self.as_ptr(),
                        array.as_mut_ptr(),
                        len.as_mut_ptr(),
                    );
                    std::slice::from_raw_parts(array.assume_init(), len.assume_init() as usize)
                };

                NvData::ByteArray(slice)
            }
            sys::data_type_t::DATA_TYPE_INT8_ARRAY => {
                let slice = unsafe {
                    let mut array = MaybeUninit::uninit();
                    let mut len = MaybeUninit::uninit();
                    sys::nvpair_value_int8_array(
                        self.as_ptr(),
                        array.as_mut_ptr(),
                        len.as_mut_ptr(),
                    );
                    std::slice::from_raw_parts(array.assume_init(), len.assume_init() as usize)
                };

                NvData::Int8Array(slice)
            }
            sys::data_type_t::DATA_TYPE_UINT8_ARRAY => {
                let slice = unsafe {
                    let mut array = MaybeUninit::uninit();
                    let mut len = MaybeUninit::uninit();
                    sys::nvpair_value_uint8_array(
                        self.as_ptr(),
                        array.as_mut_ptr(),
                        len.as_mut_ptr(),
                    );
                    std::slice::from_raw_parts(array.assume_init(), len.assume_init() as usize)
                };

                NvData::Uint8Array(slice)
            }
            sys::data_type_t::DATA_TYPE_INT16_ARRAY => {
                let slice = unsafe {
                    let mut array = MaybeUninit::uninit();
                    let mut len = MaybeUninit::uninit();
                    sys::nvpair_value_int16_array(
                        self.as_ptr(),
                        array.as_mut_ptr(),
                        len.as_mut_ptr(),
                    );
                    std::slice::from_raw_parts(array.assume_init(), len.assume_init() as usize)
                };

                NvData::Int16Array(slice)
            }
            sys::data_type_t::DATA_TYPE_UINT16_ARRAY => {
                let slice = unsafe {
                    let mut array = MaybeUninit::uninit();
                    let mut len = MaybeUninit::uninit();
                    sys::nvpair_value_uint16_array(
                        self.as_ptr(),
                        array.as_mut_ptr(),
                        len.as_mut_ptr(),
                    );
                    std::slice::from_raw_parts(array.assume_init(), len.assume_init() as usize)
                };

                NvData::Uint16Array(slice)
            }
            sys::data_type_t::DATA_TYPE_INT32_ARRAY => {
                let slice = unsafe {
                    let mut array = MaybeUninit::uninit();
                    let mut len = MaybeUninit::uninit();
                    sys::nvpair_value_int32_array(
                        self.as_ptr(),
                        array.as_mut_ptr(),
                        len.as_mut_ptr(),
                    );
                    std::slice::from_raw_parts(array.assume_init(), len.assume_init() as usize)
                };

                NvData::Int32Array(slice)
            }
            sys::data_type_t::DATA_TYPE_UINT32_ARRAY => {
                let slice = unsafe {
                    let mut array = MaybeUninit::uninit();
                    let mut len = MaybeUninit::uninit();
                    sys::nvpair_value_uint32_array(
                        self.as_ptr(),
                        array.as_mut_ptr(),
                        len.as_mut_ptr(),
                    );
                    std::slice::from_raw_parts(array.assume_init(), len.assume_init() as usize)
                };

                NvData::Uint32Array(slice)
            }
            sys::data_type_t::DATA_TYPE_INT64_ARRAY => {
                let slice = unsafe {
                    let mut array = MaybeUninit::uninit();
                    let mut len = MaybeUninit::uninit();
                    sys::nvpair_value_int64_array(
                        self.as_ptr(),
                        array.as_mut_ptr(),
                        len.as_mut_ptr(),
                    );
                    std::slice::from_raw_parts(array.assume_init(), len.assume_init() as usize)
                };

                NvData::Int64Array(slice)
            }
            sys::data_type_t::DATA_TYPE_UINT64_ARRAY => {
                let slice = unsafe {
                    let mut array = MaybeUninit::uninit();
                    let mut len = MaybeUninit::uninit();
                    sys::nvpair_value_uint64_array(
                        self.as_ptr(),
                        array.as_mut_ptr(),
                        len.as_mut_ptr(),
                    );
                    std::slice::from_raw_parts(array.assume_init(), len.assume_init() as usize)
                };

                NvData::Uint64Array(slice)
            }
            sys::data_type_t::DATA_TYPE_NVLIST_ARRAY => {
                let slice = unsafe {
                    let mut array = MaybeUninit::uninit();
                    let mut len = MaybeUninit::uninit();
                    sys::nvpair_value_nvlist_array(
                        self.as_ptr(),
                        array.as_mut_ptr(),
                        len.as_mut_ptr(),
                    );
                    std::slice::from_raw_parts(array.assume_init(), len.assume_init() as usize)
                };
                let mut vec = Vec::with_capacity(slice.len());
                for p in slice {
                    vec.push(unsafe { NvListRef::from_ptr(*p) });
                }

                NvData::NvListRefArray(vec)
            }
            /* TODO:
            pub const DATA_TYPE_STRING_ARRAY: Type = 17;
            pub const DATA_TYPE_HRTIME: Type = 18;
            pub const DATA_TYPE_BOOLEAN_ARRAY: Type = 24;
            pub const DATA_TYPE_DOUBLE: Type = 27;
            */
            _ => NvData::Unknown,
        }
    }

    pub fn tuple(&self) -> (&ffi::CStr, NvData<'_>) {
        (self.name(), self.data())
    }
}

impl std::fmt::Debug for NvPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NvPair")
            .field(&self.name())
            .field(&self.data())
            .finish()
    }
}

impl<'a> NvData<'a> {
    pub fn as_str(&self) -> Option<&ffi::CStr> {
        match self {
            NvData::Str(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        self.as_str()?.to_owned().into_string().ok()
    }

    pub fn as_list(&self) -> Option<&NvListRef> {
        match self {
            NvData::NvListRef(c) => Some(c),
            _ => None,
        }
    }
}
