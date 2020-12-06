#![warn(missing_debug_implementations, rust_2018_idioms)]

use cstr_argument::CStrArgument;
use foreign_types::ForeignType;
use nvpair::{NvList, NvListRef};
use std::marker::PhantomData;
use std::{fmt, io, ptr};
use std::convert::TryInto;
use std::os::unix::io::RawFd;
use zfs_core_sys as sys;

/// A handle to work with Zfs pools, datasets, etc
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
    #[doc(alias = "libzfs_core_init")]
    pub fn new() -> io::Result<Self> {
        let v = unsafe { sys::libzfs_core_init() };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(Self { i: PhantomData })
        }
    }

    /// Corresponds to `lzc_create()`
    #[doc(alias = "lzc_create")]
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

    /// Corresponds to `lzc_clone()`
    #[doc(alias = "lzc_clone")]
    pub fn clone_dataset<S: CStrArgument, S2: CStrArgument>(&self, name: S, origin: S2, props: &mut NvListRef) -> io::Result<()> {
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
    /// Corresponds to `lzc_promote()`
    #[doc(alias = "lzc_promote")]
    pub fn promote<S: CStrArgument>(&self, fsname: S, snap_name_buf: &mut [u8]) -> io::Result<()> {
        let fsname = fsname.into_cstr();
        let v = unsafe { sys::lzc_promote(fsname.as_ref().as_ptr(), snap_name_buf.as_mut_ptr() as *mut _, snap_name_buf.len().try_into().unwrap()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_rename()`
    #[doc(alias = "lzc_rename")]
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

    /// Destroy the given dataset (which may be a filesystem, snapshot, bookmark, volume, etc)
    ///
    /// Corresponds to `lzc_destroy()`
    #[doc(alias = "lzc_destroy")]
    pub fn destroy<S: CStrArgument>(&self, name: S) -> io::Result<()> {
        let name = name.into_cstr();
        let v = unsafe { sys::lzc_destroy(name.as_ref().as_ptr()) };

        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Create snapshot(s)
    #[doc(alias = "lzc_snapshot")]
    #[doc(alias = "snapshot_raw")]
    pub fn snapshot<I: IntoIterator<Item=S>, S: CStrArgument>(&self, snaps: I) -> Result<(), (io::Error, Option<NvList>)> {
        let mut arg = NvList::new().unwrap();

        for i in snaps {
            arg.insert(i.into_cstr().as_ref(), ()).unwrap();
        }

        let props = NvList::new().unwrap();
        self.snapshot_raw(&arg, &props)
    }

    /// Create snapshot(s). `snaps` is a list of booleans, the names of which correspond to
    /// snapshot names.
    ///
    /// The snapshots must be from the same pool, and must not reference the same dataset (iow:
    /// cannot create 2 snapshots of the same filesystem).
    ///
    /// Corresponds to `lzc_snapshot()`.
    // TODO: this is a fairly raw interface, consider abstracting (or at least adding some
    // restrictions on the NvLists).
    #[doc(alias = "lzc_snapshot")]
    pub fn snapshot_raw(&self, snaps: &NvList, props: &NvList) -> Result<(), (io::Error, Option<NvList>)> {
        let mut nv = ptr::null_mut();
        let v = unsafe {
            sys::lzc_snapshot(snaps.as_ptr() as *mut _, props.as_ptr() as *mut _, &mut nv)
        };

        if v != 0 {
            let e = if nv.is_null() {
                None
            } else {
                Some(unsafe { NvList::from_ptr(nv) })
            };
            Err((io::Error::from_raw_os_error(v), e))
        } else {
            Ok(())
        }
    }

    #[doc(alias = "destroy_snaps_raw")]
    #[doc(alias = "lzc_destroy_snaps")]
    pub fn destroy_snaps<I: IntoIterator<Item = S>, S: CStrArgument>(&self, snaps: I, defer: Defer) -> Result<(), (io::Error, NvList)> {
        let mut snaps_nv = NvList::new().unwrap();

        for snap in snaps {
            snaps_nv.insert(snap, ()).unwrap();
        }

        self.destroy_snaps_raw(&snaps_nv, defer)
    }

    /// Corresponds to `lzc_destroy_snaps()`
    #[doc(alias = "lzc_destroy_snaps")]
    pub fn destroy_snaps_raw(&self, snaps: &NvList, defer: Defer) -> Result<(), (io::Error, NvList)> {
        let mut nv = ptr::null_mut();
        let v = unsafe {
            sys::lzc_destroy_snaps(snaps.as_ptr() as *mut _, bool::from(defer) as sys::boolean_t::Type, &mut nv)
        };

        if v != 0 {
            Err((io::Error::from_raw_os_error(v), unsafe {
                NvList::from_ptr(nv)
            }))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_snaprange_space()`
    #[doc(alias = "lzc_snaprange_space")]
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
    ///
    /// Corresponds to `lzc_exists()`.
    #[doc(alias = "lzc_exists")]
    pub fn exists<S: CStrArgument>(&self, name: S) -> bool {
        let name = name.into_cstr();
        let v = unsafe { sys::lzc_exists(name.as_ref().as_ptr()) };
        v != 0
    }

    // 0.8.?
    /// Corresponds to `lzc_sync()`.
    #[doc(alias = "lzc_sync")]
    pub fn sync<S: CStrArgument>(&self, pool_name: S, force: bool) -> io::Result<()> {
        let pool_name = pool_name.into_cstr();
        let mut args = NvList::new_unique_names().unwrap();

        // note: always include for compat with <=2.0.0
        args.insert("force", force).unwrap();

        let v = unsafe {
            sys::lzc_sync(pool_name.as_ref().as_ptr(), args.as_ptr() as *mut _, ptr::null_mut())
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
    #[doc(alias = "lzc_hold")]
    pub fn hold_raw(&self, holds: &NvListRef, cleanup_fd: Option<RawFd>) -> Result<(), (io::Error, Option<NvList>)> {
        let mut errs = ptr::null_mut();
        let v = unsafe { sys::lzc_hold(holds.as_ptr() as *mut _, cleanup_fd.unwrap_or(-1), &mut errs) };
        if v != 0 {
            let e = if errs.is_null() {
                None
            } else {
                Some(unsafe { NvList::from_ptr(errs)})
            };
            Err((io::Error::from_raw_os_error(v), e))
        } else {
            Ok(())
        }
    }

    #[doc(alias = "lzc_release")]
    pub fn release_raw(&self, holds: &NvListRef) -> Result<(), io::Error> {
        let v = unsafe { sys::lzc_release(holds.as_ptr() as *mut _, /* errlist: */ptr::null_mut()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Get the holds for a given snapshot
    ///
    /// Corresponds to `lzc_get_holds()`
    #[doc(alias = "lzc_get_holds")]
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
    
    /// Corresponds to `lzc_send()`
    #[doc(alias = "lzc_send")]
    pub fn send<S: CStrArgument, F: CStrArgument>(&self, snapname: S, from: Option<F>, fd: RawFd, flags: SendFlags) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let from = from.map(|a| a.into_cstr());

        let v = unsafe { sys::lzc_send(snapname.as_ref().as_ptr(), from.map_or(ptr::null(), |x| x.as_ref().as_ptr()), fd, flags.into()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_send_redacted()`
    #[doc(alias = "lzc_send_redacted")]
    #[cfg(features = "v2_00")]
    pub fn send_redacted<S: CStrArgument, F: CStrArgument, R: CStrArgument>(&self, snapname: S, from: F, fd: RawFd, redactbook: R, flags: SendFlags) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let from = from.into_cstr();
        let redactbook = redactbook.into_cstr();

        let v = unsafe { sys::lzc_send_redacted(snapname.as_ref().as_ptr(), from.as_ref().as_ptr(), fd, redactbook.as_ref().as_ptr(), flags.into()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_send_resume()`
    #[doc(alias = "lzc_send_resume")]
    pub fn send_resume<S: CStrArgument, F: CStrArgument>(&self, snapname: S, from: F, fd: RawFd, flags: SendFlags, resume_obj: u64, resume_off: u64) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let from = from.into_cstr();

        let v = unsafe {
            sys::lzc_send_resume(snapname.as_ref().as_ptr(), from.as_ref().as_ptr(), fd, flags.into(), resume_obj, resume_off)
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /*
    /// Corresponds to `lzc_send_resume_redacted()`
    #[cfg(features = "v2_00")]
    pub fn send_resume_redacted<S: CStrArgument, F: CStrArgument, R: CStrArgument>(&self, _snapname: S, _from: F, _fd: RawFd, _resume_obj: u64, _resume_off: u64, _redactbook: R) -> io::Result<()> {
        unimplemented!()
    }

    /// Corresponds to `lzc_send_space()`
    pub fn send_space<S: CStrArgument, F: CStrArgument>(&self, _snapname: S, _from: F, _flags: u64) -> io::Result<u64> {
        unimplemented!()
    }

    #[cfg(features = "v2_00")]
    pub fn send_space_resume_redacted<S: CStrArgument, F: CStrArgument, R: CStrArgument>(&self, snapname: S, from: F, flags: u64, resume_obj: u64, resume_off: u64, resume_bytes: u64, redactbook: R, fd: RawFd) -> io::Result<u64> {
        unimplemented!()
    }
    */

    /// Corresponds to `lzc_receive()`
    #[doc(alias = "lzc_receive")]
    pub fn receive<S: CStrArgument, O: CStrArgument>(&self, snapname: S, props: Option<&NvListRef>, origin: Option<O>, force: bool, raw: bool, fd: RawFd) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let origin = origin.map(|x| x.into_cstr());

        let r = unsafe {
            sys::lzc_receive(snapname.as_ref().as_ptr(), props.map_or(ptr::null_mut(), |x| x.as_ptr() as *mut _), origin.map_or(ptr::null(),
                |x| x.as_ref().as_ptr()), if force { 1 } else { 0 }, if raw { 1 } else { 0 }, fd)
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(-r))
        } else {
            Ok(())
        }
    }


    /// Corresponds to `lzc_receive_resumable()`
    // internally, only a flag differs from `recv`
    // consider implimenting something that takes `resumeable` as a flag
    #[doc(alias = "lzc_receive_resumable")]
    pub fn receive_resumable<S: CStrArgument, O: CStrArgument>(&self, snapname: S, props: &NvListRef, origin: O, force: bool, raw: bool, fd: RawFd) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let origin = origin.into_cstr();

        let r = unsafe {
            sys::lzc_receive_resumable(snapname.as_ref().as_ptr(), props.as_ptr() as *mut _, origin.as_ref().as_ptr(), if force { 1 } else { 0 }, if raw { 1 } else { 0 }, fd)
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(-r))
        } else {
            Ok(())
        }
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

    /// Corresponds to `lzc_rollback()`
    #[doc(alias = "lzc_rollback")]
    pub fn rollback<S: CStrArgument>(&self, fsname: S) -> io::Result<std::ffi::CString> {
        let fsname = fsname.into_cstr();
        let mut rname = vec![0u8; sys::ZFS_MAX_DATASET_NAME_LEN as usize + 1];

        let r = unsafe {
            sys::lzc_rollback(fsname.as_ref().as_ptr(),
                rname.as_mut_ptr() as *mut std::os::raw::c_char, rname.len() as std::os::raw::c_int)
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            let p = rname.iter().position(|x| *x == b'\0').unwrap();
            rname.resize(p, 0);
            Ok(std::ffi::CString::new(rname).unwrap())
        }
    }

    /// Corresponds to `lzc_rollback_to()`
    #[doc(alias = "lzc_rollback_to")]
    pub fn rollback_to<F: CStrArgument, S: CStrArgument>(&self, fsname: F, snapname: S) -> io::Result<()> {
        let fsname = fsname.into_cstr();
        let snapname = snapname.into_cstr();

        let r = unsafe {
            sys::lzc_rollback_to(fsname.as_ref().as_ptr(), snapname.as_ref().as_ptr())
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /*
    #[doc(alias = "lzc_bookmark")]
    pub fn bookmark(&self, bookmarks: &NvListRef) -> Result<(), (io::Error, NvList)> {
        unimplemented!()
    }

    /// Corresponds to `lzc_get_bookmarks`
    #[doc(alias = "lzc_get_bookmarks")]
    pub fn bookmarks<F: CStrArgument>(&self, fsname: F, props: &NvListRef) -> io::Result<NvList> {
        unimplemented!()
    }

    /// Corresponds to `lzc_get_bookmark_props`
    #[doc(alias = "lzc_get_bookmark_props")]
    pub fn bookmark_props<B: CStrArgument>(&self, bookmark: B) -> io::Result<NvList> {
        unimplemented!()
    }

    #[doc(alias = "lzc_destroy_bookmarks")]
    pub fn destroy_bookmarks(&self, bookmarks: &NvListRef) -> Result<(), (io::Error, NvList)> {
        unimplemented!()
    }

    // 0.8.?
    #[doc(alias = "lzc_channel_program")]
    pub fn channel_program<P: CStrArgument, R: CStrArgument>(&self, pool: P, program: R, instruction_limit: u64, memlimit: u64, args: &NvListRef) -> io::Result<NvList> {
        unimplemented!()
    }

    // 0.8.?
    #[doc(alias = "lzc_pool_checkpoint")]
    pub fn pool_checkpoint<P: CStrArgument>(&self, pool: P) -> io::Result<()> {
        unimplemented!()
    }

    #[doc(alias = "lzc_pool_checkpoint_discard")]
    pub fn pool_checkpoint_discard<P: CStrArgument>(&self, pool: P) -> io::Result<()> {
        unimplemented!()
    }

    #[doc(alias = "lzc_channel_program_nosync")]
    pub fn channel_program_nosync<P: CStrArgument, R: CStrArgument>(&self, pool: P, program: R, instruction_limit: u64, memlimit: u64, args: &NvListRef) -> io::Result<NvList> {
        unimplemented!()
    }

    #[doc(alias = "lzc_load_key")]
    pub fn load_key<F: CStrArgument>(&self, fsname: F, keydata: &[u8]) -> io::Result<()> {
        unimplemented!()
    }

    #[doc(alias = "lzc_unload_key")]
    pub fn unload_key<F: CStrArgument>(&self, fsname: F) -> io::Result<()> {
        unimplemented!()
    }

    #[doc(alias = "lzc_change_key")]
    pub fn change_key<F: CStrArgument>(&self, fsname: F, crypt_cmd: u64, props: &NvListRef, keydata: Option<&[u8]>) -> io::Result<()> {
        unimplemented!()
    }

    // 0.8.?
    #[doc(alias = "lzc_reopen")]
    pub fn reopen<P: CStrArgument>(&self, pool: P, scrub_restart: bool) -> io::Result<()> {
        unimplemented!()
    }

    // 0.8.?
    #[doc(alias = "lzc_initialize")]
    pub fn initialize<P: CStrArgument>(&self, pool: P, initialize_func: InitializeFunc, vdevs: &NvListRef) -> Result<(), (io::Error, NvList)> {
        unimplemented!()
    }

    // 0.8.?
    #[doc(alias = "lzc_trim")]
    pub fn trim<P: CStrArgument>(&self, pool: P, pool_trim_func: PoolTrimFunc, rate: u64, secure: bool, vdevs: &NvListRef) -> Result<(), (io::Error, NvList)> {
        unimplemented!()
    }

    #[doc(alias = "lzc_redact")]
    pub fn redact<S: CStrArgument, B: CStrArgument>(&self, snapname: S, bookname: B, snapnv: &NvListRef) -> io::Result<()> {
        unimplemented!()
    }

    #[doc(alias = "lzc_wait")]
    pub fn wait<P: CStrArgument>(&self, pool: P, activity: WaitActivity) -> io::Result<bool> {
        unimplemented!()
    }

    #[doc(alias = "lzc_wait_tag")]
    pub fn wait_tag<P: CStrArgument>(&self, pool: P, activity: WaitActivity, tag: u64) -> io::Result<bool> {
        unimplemented!()
    }

    #[doc(alias = "lzc_wait_fs")]
    pub fn wait_fs<F: CStrArgument>(&self, fs: F, activity: WaitActivity) -> io::Result<bool> {
        unimplemented!()
    }
    */

    #[cfg(features = "v2_00")]
    #[doc(alias = "lzc_set_bootenv")]
    pub fn set_bootenv<P: CStrArgument, E: CStrArgument>(&self, pool: P, env: &NvListRef) -> io::Result<()> {
        let pool = pool.into_cstr();
        let v = unsafe { sys::lzc_set_bootenv(pool.as_ref().as_ptr(), env.as_ptr()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    #[cfg(features = "v2_00")]
    #[doc(alias = "lzc_get_bootenv")]
    pub fn bootenv<P: CStrArgument>(&self, pool: P) -> io::Result<NvList> {
        let pool = pool.into_cstr();
        let mut env = ptr::null_mut();
        let v = unsafe { sys::lzc_get_bootenv(pool.as_ref().as_ptr(), &mut env) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { NvList::from_ptr(env)})
        }
    }
}

impl Drop for Zfs {
    fn drop(&mut self) {
        unsafe { sys::libzfs_core_fini() }
    }
}

#[derive(Debug, PartialEq)]
pub enum InitializeFunc {
    Start,
    Cancel,
    Suspend
}

#[derive(Debug, PartialEq)]
pub enum PoolTrimFunc {
    Start,
    Cancel,
    Suspend
}

#[derive(Debug, PartialEq)]
pub enum WaitActivity {
    Discard,
    Free,
    Initialize,
    Replace,
    Remove,
    Resliver,
    Scrub,
    Trim,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SendFlags {
    pub embed_data: bool,
    pub large_block: bool,
    pub compress: bool,
    pub raw: bool,
}

impl From<SendFlags> for u32 {
    fn from(sf: SendFlags) -> Self {
        let mut f = 0;
        if sf.embed_data {
            f |= sys::lzc_send_flags::LZC_SEND_FLAG_EMBED_DATA;
        }
        if sf.large_block {
            f |= sys::lzc_send_flags::LZC_SEND_FLAG_LARGE_BLOCK;
        }
        if sf.compress {
            f |= sys::lzc_send_flags::LZC_SEND_FLAG_COMPRESS;
        }
        if sf.raw {
            f |= sys::lzc_send_flags::LZC_SEND_FLAG_RAW;
        }

        f
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Defer {
    No,
    Yes,
}

impl Default for Defer {
    fn default() -> Self {
        Defer::No
    }
}

impl From<Defer> for bool {
    fn from(d: Defer) -> Self {
        match d {
            Defer::No => false,
            Defer::Yes => true,
        }
    }
}
