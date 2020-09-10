extern crate zfs_core_sys as sys;
extern crate nvpair;
extern crate cstr_argument;
extern crate foreign_types;

use nvpair::NvList;
use foreign_types::{ForeignType};
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
            sys::lzc_create(name.as_ref().as_ptr(), dataset_type.as_raw(), props.as_ptr() as *mut _, ptr::null_mut(), 0)
        };
    
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    // TODO: this is a fairly raw interface, consider abstracting (or at least adding some
    // restrictions on the NvLists).
    pub fn snapshot(&self, snaps: &NvList, props: &NvList) -> Result<(), (io::Error, NvList)>
    {
        let mut nv = ptr::null_mut();
        let v = unsafe {
            sys::lzc_snapshot(snaps.as_ptr() as *mut _, props.as_ptr() as *mut _, &mut nv)
        };

        if v != 0 {
            Err((io::Error::from_raw_os_error(v), unsafe { NvList::from_ptr(nv) }))
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
