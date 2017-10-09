extern crate libzfs_core_sys as ffi;
extern crate cstr_argument;

use cstr_argument::CStrArgument;
use std::marker::PhantomData;
use std::io;

/// A handle to work with Zfs filesystems
// Note: the Drop for this makes clone-by-copy unsafe. Could clone by just calling new().
//
// Internally, libzfs_core maintains a refcount for the `libzfs_core_init()` and
// `libzfs_core_fini()` calls, so we need the init to match fini. Alternatively, we could use a
// single init and never fini.
pub struct Zfs {
    i: PhantomData<()>
}

impl Zfs {
    /// Create a handle to the Zfs subsystem
    pub fn new() -> io::Result<Self> {
        let v = unsafe {
            ffi::libzfs_core_init()
        };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(Self {
                i: PhantomData
            })
        }
    }

    /*
    fn create<S: CStrArgument>(name: S, dataset_type, props, wkeydata: &[u8]) -> io::Result<()>
    {

    }
    */
}

impl Drop for Zfs {
    fn drop(&mut self) {
        unsafe {
            ffi::libzfs_core_fini()
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
