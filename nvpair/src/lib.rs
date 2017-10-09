extern crate nvpair_sys as ffi;

use std::io;
use std::ptr;
use std::os::raw::c_int;

pub enum NvEncoding {
    Native,
    Xdr
}

impl NvEncoding {
    fn as_raw(&self) -> c_int {
        match self {
            &NvEncoding::Native => ffi::NV_ENCODE_NATIVE,
            &NvEncoding::Xdr => ffi::NV_ENCODE_XDR
        }
    }
}

pub struct NvList {
    raw: *mut ffi::nvlist
}

impl NvList {
    pub fn new() -> io::Result<Self> {
        let mut n = Self { raw: ptr::null_mut() };
        let v = unsafe {
            // TODO: second arg is a bitfield of NV_UNIQUE_NAME|NV_UNIQUE_NAME_TYPE
            ffi::nvlist_alloc(&mut n.raw, 0, 0)
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
            ffi::nvlist_size(self.raw, &mut l, encoding.as_raw())
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(l)
        }
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        let mut n = Self { raw: ptr::null_mut() };
        let v = unsafe { ffi::nvlist_dup(self.raw, &mut n.raw, 0) };
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
            ffi::nvlist_free(self.raw);
        }
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
