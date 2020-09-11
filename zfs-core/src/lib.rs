#![warn(missing_debug_implementations, rust_2018_idioms)]

use cstr_argument::CStrArgument;
use foreign_types::ForeignType;
use nvpair::NvList;
use std::marker::PhantomData;
use std::{fmt, io, ptr};
use zfs_core_sys as sys;

/// A handle to work with Zfs filesystems
// Note: the Drop for this makes clone-by-copy unsafe. Could clone by just calling new().
//
// Internally, libzfs_core maintains a refcount for the `libzfs_core_init()` and
// `libzfs_core_fini()` calls, so we need the init to match fini. Alternatively, we could use a
// single init and never fini.
pub struct Zfs {
    i: PhantomData<()>,
}

impl fmt::Debug for Zfs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zfs").finish()
    }
}

#[derive(Debug)]
pub enum DataSetType {
    Zfs,
    Zvol,
}

impl DataSetType {
    fn as_raw(&self) -> ::std::os::raw::c_uint {
        match self {
            DataSetType::Zfs => sys::lzc_dataset_type::LZC_DATSET_TYPE_ZFS,
            DataSetType::Zvol => sys::lzc_dataset_type::LZC_DATSET_TYPE_ZVOL,
        }
    }
}

impl Zfs {
    /// Create a handle to the Zfs subsystem
    pub fn new() -> io::Result<Self> {
        let v = unsafe { sys::libzfs_core_init() };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(Self { i: PhantomData })
        }
    }

    pub fn create<S: CStrArgument>(
        &self,
        name: S,
        dataset_type: DataSetType,
        props: &NvList,
    ) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe {
            sys::lzc_create(
                name.as_ref().as_ptr(),
                dataset_type.as_raw(),
                props.as_ptr() as *mut _,
                ptr::null_mut(),
                0,
            )
        };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    pub fn destroy<S: CStrArgument>(&self, name: S) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::lzc_destroy(name.as_ref().as_ptr()) };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    pub fn exists<S: CStrArgument>(&self, name: S) -> bool {
        let name = name.into_cstr();
        let v = unsafe { sys::lzc_exists(name.as_ref().as_ptr()) };
        v != 0
    }

    // TODO: this is a fairly raw interface, consider abstracting (or at least adding some
    // restrictions on the NvLists).
    pub fn snapshot(&self, snaps: &NvList, props: &NvList) -> Result<(), (io::Error, NvList)> {
        let mut nv = ptr::null_mut();
        let v = unsafe {
            sys::lzc_snapshot(snaps.as_ptr() as *mut _, props.as_ptr() as *mut _, &mut nv)
        };

        if v != 0 {
            Err((io::Error::from_raw_os_error(v), unsafe {
                NvList::from_ptr(nv)
            }))
        } else {
            Ok(())
        }
    }

    // clone
    // promote
    // destroy_snaps
    // bookmark
    // get_bookmarks
    // destroy_bookmarks
    // snaprange_space
    // hold
    // release
    // get_holds
    // send
    // send_resume
    // send_space
    // receive
    // receive_resumable
    // receive_with_header
    // receive_once
    // receive_with_cmdprops
    // exists
    // rollback
    // rollback_to
    // sync
}

impl Drop for Zfs {
    fn drop(&mut self) {
        unsafe { sys::libzfs_core_fini() }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
