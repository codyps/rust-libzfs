extern crate libzfs_core_sys as sys;
extern crate nvpair;
extern crate cstr_argument;

use nvpair::NvList;
use cstr_argument::CStrArgument;
use std::marker::PhantomData;
use std::io;
use std::ptr;

/// A handle to work with Zfs filesystems
// Note: the Drop for this makes clone-by-copy unsafe. Could clone by just calling new().
//
// Internally, libzfs_core maintains a refcount for the `libzfs_core_init()` and
// `libzfs_core_fini()` calls, so we need the init to match fini. Alternatively, we could use a
// single init and never fini.
pub struct Zfs {
    i: PhantomData<()>
}

pub enum DataSetType {
    Zfs,
    Zvol,
}

impl DataSetType {
    fn as_raw(&self) -> ::std::os::raw::c_uint
    {
        match self {
            &DataSetType::Zfs => sys::lzc_dataset_type::LZC_DATSET_TYPE_ZFS,
            &DataSetType::Zvol => sys::lzc_dataset_type::LZC_DATSET_TYPE_ZVOL
        }
    }
}

impl Zfs {
    /// Create a handle to the Zfs subsystem
    pub fn new() -> io::Result<Self> {
        let v = unsafe {
            sys::libzfs_core_init()
        };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(Self {
                i: PhantomData
            })
        }
    }

    pub fn create<S: CStrArgument>(&self, name: S, dataset_type: DataSetType, props: &NvList) -> io::Result<()>
    {
        let name = name.into_cstr();
        let v = unsafe {
            sys::lzc_create(name.as_ref().as_ptr(), dataset_type.as_raw(), props.as_ptr())
        };
    
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }
}

impl Drop for Zfs {
    fn drop(&mut self) {
        unsafe {
            sys::libzfs_core_fini()
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
