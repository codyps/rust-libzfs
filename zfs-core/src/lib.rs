#![warn(missing_debug_implementations, rust_2018_idioms)]

use cstr_argument::CStrArgument;
use foreign_types::ForeignType;
use nvpair::{NvList, NvListRef};
use std::marker::PhantomData;
use std::{fmt, io, ptr};
use std::convert::TryInto;
use std::os::unix::io::RawFd;
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

    pub fn clone<S: CStrArgument, S2: CStrArgument>(&self, name: S, origin: S2, props: &mut NvListRef) -> io::Result<()> {
        let name = name.into_cstr();
        let origin = origin.into_cstr();
        let v = unsafe { sys::lzc_clone(name.as_ref().as_ptr(), origin.as_ref().as_ptr(), props.as_mut_ptr()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    // TODO: avoid using an out-param for `snap_name_buf`
    // `snap_name_buf` is filled by a cstring
    pub fn promote<S: CStrArgument>(&self, fsname: S, snap_name_buf: &mut [u8]) -> io::Result<()> {
        let fsname = fsname.into_cstr();
        let v = unsafe { sys::lzc_promote(fsname.as_ref().as_ptr(), snap_name_buf.as_mut_ptr() as *mut _, snap_name_buf.len().try_into().unwrap()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    pub fn rename<S: CStrArgument, T: CStrArgument>(&self, source: S, target: T) -> io::Result<()> {
        let source = source.into_cstr();
        let target = target.into_cstr();

        let v = unsafe { sys::lzc_rename(source.as_ref().as_ptr(), target.as_ref().as_ptr()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// 
    pub fn destroy<S: CStrArgument>(&self, name: S) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::lzc_destroy(name.as_ref().as_ptr()) };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
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

    pub fn destroy_snaps(&self, snaps: &NvList, defer: bool) -> Result<(), (io::Error, NvList)> {
        let mut nv = ptr::null_mut();
        let v = unsafe {
            sys::lzc_destroy_snaps(snaps.as_ptr() as *mut _, defer as u32, &mut nv)
        };

        if v != 0 {
            Err((io::Error::from_raw_os_error(v), unsafe {
                NvList::from_ptr(nv)
            }))
        } else {
            Ok(())
        }
    }

    pub fn snaprange_space<F: CStrArgument, L: CStrArgument>(&self, first_snap: F, last_snap: L) -> io::Result<u64> {
        let first_snap = first_snap.into_cstr();
        let last_snap = last_snap.into_cstr();

        let mut out = 0;
        let v = unsafe {
            sys::lzc_snaprange_space(first_snap.as_ref().as_ptr(), last_snap.as_ref().as_ptr(), &mut out)
        };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(out)
        }
    }

    /// Check if a dataset (a filesystem, or a volume, or a snapshot)
    /// with the given name exists.
    ///
    /// Note: cannot check for bookmarks
    pub fn exists<S: CStrArgument>(&self, name: S) -> bool {
        let name = name.into_cstr();
        let v = unsafe { sys::lzc_exists(name.as_ref().as_ptr()) };
        v != 0
    }

    pub fn sync<S: CStrArgument>(&self, pool_name: S, nvl: &NvListRef) -> io::Result<()> {
        let pool_name = pool_name.into_cstr();
        let v = unsafe {
            sys::lzc_sync(pool_name.as_ref().as_ptr(), nvl.as_ptr() as *mut _, ptr::null_mut())
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Create "user holds" on snapshots.  If there is a hold on a snapshot,
    /// the snapshot can not be destroyed.  (However, it can be marked for deletion
    /// by lzc_destroy_snaps(defer=B_TRUE).)                               
    ///                                           
    /// The keys in the nvlist are snapshot names.                     
    /// The snapshots must all be in the same pool.
    /// The value is the name of the hold (string type).
    pub fn hold_raw<S: CStrArgument>(&self, holds: &NvListRef, cleanup_fd: Option<RawFd>) -> io::Result<()> {
        let v = unsafe { sys::lzc_hold(holds.as_ptr() as *mut _, cleanup_fd.unwrap_or(-1), /* errlist: */ptr::null_mut()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    pub fn release_raw(&self, holds: &NvListRef) -> Result<(), io::Error> {
        let v = unsafe { sys::lzc_release(holds.as_ptr() as *mut _, /* errlist: */ptr::null_mut()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_get_holds()`
    pub fn holds<S: CStrArgument>(&self, snapname: S) -> io::Result<NvList> {
        let snapname = snapname.into_cstr();
        let mut holds = ptr::null_mut();
        let v = unsafe { sys::lzc_get_holds(snapname.as_ref().as_ptr(), &mut holds) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { NvList::from_ptr(holds)})
        }
    }
    
    pub fn send<S: CStrArgument, F: CStrArgument>(&self, snapname: S, from: F, fd: RawFd) -> io::Result<()> {
        // TODO: FLAGS
        unimplemented!()
    }

    pub fn send_redacted<S: CStrArgument, F: CStrArgument, R: CStrArgument>(&self, snapname: S, from: F, fd: RawFd, redactbook: R) -> io::Result<()> {
        unimplemented!()
    }

    pub fn send_resume<S: CStrArgument, F: CStrArgument>(&self, snapname: S, from: F, fd: RawFd, resume_obj: u64, resume_off: u64) -> io::Result<()> {
        unimplemented!()
    }

    pub fn send_resume_redacted<S: CStrArgument, F: CStrArgument, R: CStrArgument>(&self, snapname: S, from: F, fd: RawFd, resume_obj: u64, resume_off: u64, redactbook: R) -> io::Result<()> {
        unimplemented!()
    }

    pub fn send_space<S: CStrArgument, F: CStrArgument>(&self, snapname: S, from: F, flags: u64) -> io::Result<u64> {
        unimplemented!()
    }

    pub fn send_space_resume_redacted<S: CStrArgument, F: CStrArgument, R: CStrArgument>(&self, snapname: S, from: F, flags: u64, resume_obj: u64, resume_off: u64, resume_bytes: u64, redactbook: R, fd: RawFd) -> io::Result<u64> {
        unimplemented!()
    }

    pub fn receive<S: CStrArgument, O: CStrArgument>(&self, snapname: S, props: &NvList, origin: O, force: bool, raw: bool, fd: RawFd) -> io::Result<()> {
        unimplemented!()
    }

    // internally, only a flag differs from `recv`
    pub fn receive_resumable<S: CStrArgument, O: CStrArgument>(&self, snapname: S, props: &NvListRef, origin: O, force: bool, raw: bool, fd: RawFd) -> io::Result<()> {
        unimplemented!()
    }

    /*
    pub fn receive_with_header<S: CStrArgument, O: CStrArgument>(&self, snapname: S, props: &NvListRef, origin: O, force: bool, resumeable: bool, raw: bool, fd: RawFd, begin_record: &DmuReplayRecordRef) -> io::Result<()> {
        unimplemented!()
    }
    */

    /*
    pub fn receive_one<S: CStrArgument, O: CStrArgument>(&self, snapname: S, cmdprops: &NvListRef, wkey: Option<&[u8]>, origin: O, force: bool, resumeable: bool, raw: bool, input_fd: RawFd, begin_record: &DmuReplayRecordRef) -> io::Result<(/* bytes */ u64, /* errflags */u64, /* errors */ NvList)> {
        unimplemented!()
    }
    */

    /*
    pub fn receive_with_cmdprops<S: CStrArgument, O: CStrArgument>(&self, snapname: S, cmdprops: &NvListRef, wkey: Option<&[u8]>, origin: O, force: bool, resumeable: bool, raw: bool, input_fd: RawFd, begin_record: &DmuReplayRecordRef) -> io::Result<(/* bytes */ u64, /* errflags */u64, /* errors */ NvList)> {
        unimplemented!()
    }
    */

    pub fn rollback<S: CStrArgument>(&self, fsname: S, snap_name_buf: &mut [u8]) -> io::Result<&[u8]> {
        unimplemented!()
    }

    pub fn rollback_to<F: CStrArgument, S: CStrArgument>(&self, fsname: F, snapname: S) -> io::Result<&[u8]> {
        unimplemented!()
    }

    pub fn bookmark(&self, bookmarks: &NvListRef) -> Result<(), (io::Error, NvList)> {
        unimplemented!()
    }

    /// Corresponds to `lzc_get_bookmarks`
    pub fn bookmarks<F: CStrArgument>(&self, fsname: F, props: &NvListRef) -> io::Result<NvList> {
        unimplemented!()
    }

    /// Corresponds to `lzc_get_bookmark_props`
    pub fn bookmark_props<B: CStrArgument>(&self, bookmark: B) -> io::Result<NvList> {
        unimplemented!()
    }

    pub fn destroy_bookmarks(&self, bookmarks: &NvListRef) -> Result<(), (io::Error, NvList)> {
        unimplemented!()
    }

    pub fn channel_program<P: CStrArgument, R: CStrArgument>(&self, pool: P, program: R, instruction_limit: u64, memlimit: u64, args: &NvListRef) -> io::Result<NvList> {
        unimplemented!()
    }

    pub fn pool_checkpoint<P: CStrArgument>(&self, pool: P) -> io::Result<()> {
        unimplemented!()
    }

    pub fn pool_checkpoint_discard<P: CStrArgument>(&self, pool: P) -> io::Result<()> {
        unimplemented!()
    }

    pub fn channel_program_nosync<P: CStrArgument, R: CStrArgument>(&self, pool: P, program: R, instruction_limit: u64, memlimit: u64, args: &NvListRef) -> io::Result<NvList> {
        unimplemented!()
    }

    pub fn load_key<F: CStrArgument>(&self, fsname: F, keydata: &[u8]) -> io::Result<()> {
        unimplemented!()
    }

    pub fn unload_key<F: CStrArgument>(&self, fsname: F) -> io::Result<()> {
        unimplemented!()
    }

    pub fn change_key<F: CStrArgument>(&self, fsname: F, crypt_cmd: u64, props: &NvListRef, keydata: Option<&[u8]>) -> io::Result<()> {
        unimplemented!()
    }

    pub fn reopen<P: CStrArgument>(&self, pool: P, scrub_restart: bool) -> io::Result<()> {
        unimplemented!()
    }

    pub fn initialize<P: CStrArgument>(&self, pool: P, initialize_func: InitializeFunc, vdevs: &NvListRef) -> Result<(), (io::Error, NvList)> {
        unimplemented!()
    }

    pub fn trim<P: CStrArgument>(&self, pool: P, pool_trim_func: PoolTrimFunc, rate: u64, secure: bool, vdevs: &NvListRef) -> Result<(), (io::Error, NvList)> {
        unimplemented!()
    }

    pub fn redact<S: CStrArgument, B: CStrArgument>(&self, snapname: S, bookname: B, snapnv: &NvListRef) -> io::Result<()> {
        unimplemented!()
    }

    pub fn wait<P: CStrArgument>(&self, pool: P, activity: WaitActivity) -> io::Result<bool> {
        unimplemented!()
    }

    pub fn wait_tag<P: CStrArgument>(&self, pool: P, activity: WaitActivity, tag: u64) -> io::Result<bool> {
        unimplemented!()
    }

    pub fn wait_fs<F: CStrArgument>(&self, fs: F, activity: WaitActivity) -> io::Result<bool> {
        unimplemented!()
    }

    pub fn set_bootenv<P: CStrArgument, E: CStrArgument>(&self, pool: P, env: E) -> io::Result<()> {
        unimplemented!()
    }

    pub fn bootenv<P: CStrArgument>(&self, pool: P) -> io::Result<NvList> {
        unimplemented!()
    }
}

impl Drop for Zfs {
    fn drop(&mut self) {
        unsafe { sys::libzfs_core_fini() }
    }
}

pub enum InitializeFunc {
    Start,
    Cancel,
    Suspend
}

pub enum TrimFunc {
    Start,
    Cancel,
    Suspend
}


