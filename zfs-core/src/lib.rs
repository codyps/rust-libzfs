#![warn(missing_debug_implementations, rust_2018_idioms)]

use cstr_argument::CStrArgument;
use foreign_types::ForeignType;
use nvpair::{NvList, NvListIter, NvListRef};
use snafu::Snafu;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::os::unix::io::RawFd;
use std::{ffi, fmt, io, ptr};
use zfs_core_sys as sys;

// TODO: consider splitting this into specific error kinds per operation
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("libzfs_core call failed with {}", source))]
    Io { source: io::Error },
    #[snafu(display("libzfs_core call failed for these entries {}", source))]
    List { source: ErrorList },
}

//pub type Result<T, E = Error> = std::result::Result<T, E>;

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

/// Generic list of errors return from various `lzc_*` calls.
///
/// The first item (the `name`) is the thing we were operating on (creating, destroying, etc) that cause the error,
/// and the second (the `error`) is the translated error code
///
/// Note that there is a special `name` "N_MORE_ERRORS" which is a count of errors not listed
#[derive(Debug)]
pub struct ErrorList {
    nv: NvList,
}

impl ErrorList {
    pub fn iter(&self) -> ErrorListIter<'_> {
        self.into_iter()
    }
}

impl std::error::Error for ErrorList {}

impl From<NvList> for ErrorList {
    fn from(nv: NvList) -> Self {
        // TODO: consider examining shape of the error list here
        Self { nv }
    }
}

impl AsRef<NvList> for ErrorList {
    fn as_ref(&self) -> &NvList {
        &self.nv
    }
}

impl AsMut<NvList> for ErrorList {
    fn as_mut(&mut self) -> &mut NvList {
        &mut self.nv
    }
}

impl<'a> IntoIterator for &'a ErrorList {
    type Item = (&'a ffi::CStr, io::Error);
    type IntoIter = ErrorListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ErrorListIter {
            nvi: self.nv.iter(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorListIter<'a> {
    nvi: NvListIter<'a>,
}

impl<'a> fmt::Display for ErrorListIter<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_list().entries(self.clone()).finish()
    }
}

impl fmt::Display for ErrorList {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.into_iter(), fmt)
    }
}

impl<'a> Iterator for ErrorListIter<'a> {
    type Item = (&'a ffi::CStr, io::Error);

    fn next(&mut self) -> Option<Self::Item> {
        match self.nvi.next() {
            Some(np) => {
                let name = np.name();
                let data = np.data();

                match data {
                    nvpair::NvData::Int32(v) => Some((name, io::Error::from_raw_os_error(v))),
                    _ => {
                        // TODO: consider validating early. alternately: consider emitting
                        // something reasonable here. we're already an error path, so being 100%
                        // precise is probably not required.
                        panic!("unhandled error type for name {:?}: {:?}", name, data);
                    }
                }
            }
            None => None,
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

    /// Create a new dataset of the given type with `props` set as properties
    ///
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
    pub fn clone_dataset<S: CStrArgument, S2: CStrArgument>(
        &self,
        name: S,
        origin: S2,
        props: &mut NvListRef,
    ) -> io::Result<()> {
        let name = name.into_cstr();
        let origin = origin.into_cstr();
        let v = unsafe {
            sys::lzc_clone(
                name.as_ref().as_ptr(),
                origin.as_ref().as_ptr(),
                props.as_mut_ptr(),
            )
        };
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
        let v = unsafe {
            sys::lzc_promote(
                fsname.as_ref().as_ptr(),
                snap_name_buf.as_mut_ptr() as *mut _,
                snap_name_buf.len().try_into().unwrap(),
            )
        };
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
    ///
    /// The snapshots must be from the same pool, and must not reference the same dataset (iow:
    /// cannot create 2 snapshots of the same filesystem).
    ///
    /// Corresponds to `lzc_snapshot()`.
    #[doc(alias = "lzc_snapshot")]
    #[doc(alias = "snapshot_raw")]
    pub fn snapshot<I: IntoIterator<Item = S>, S: CStrArgument>(
        &self,
        snaps: I,
    ) -> Result<(), Error> {
        let mut arg = NvList::new();

        for i in snaps {
            arg.insert(i.into_cstr().as_ref(), &()).unwrap();
        }

        let props = NvList::new();
        match self.snapshot_raw(&arg, &props) {
            Ok(a) => Ok(a),
            Err(Ok(v)) => Err(Error::Io { source: v }),
            Err(Err(v)) => Err(Error::List { source: v.into() }),
        }
    }

    /// Create snapshot(s). `snaps` is a list of `bool` (not `boolean_value`) entries, the names of
    /// which correspond to snapshot names.
    ///
    /// The snapshots must be from the same pool, and must not reference the same dataset (iow:
    /// cannot create 2 snapshots of the same filesystem with a single call).
    ///
    /// Corresponds to `lzc_snapshot()`.
    // TODO: this is a fairly raw interface, consider abstracting (or at least adding some
    // restrictions on the NvLists).
    #[doc(alias = "lzc_snapshot")]
    pub fn snapshot_raw(
        &self,
        snaps: &NvList,
        props: &NvList,
    ) -> Result<(), Result<io::Error, NvList>> {
        let mut nv = ptr::null_mut();
        let v = unsafe {
            sys::lzc_snapshot(snaps.as_ptr() as *mut _, props.as_ptr() as *mut _, &mut nv)
        };

        if v != 0 {
            if nv.is_null() {
                Err(Ok(io::Error::from_raw_os_error(v)))
            } else {
                Err(Err(unsafe { NvList::from_ptr(nv) }))
            }
        } else {
            Ok(())
        }
    }

    #[doc(alias = "destroy_snaps_raw")]
    #[doc(alias = "lzc_destroy_snaps")]
    pub fn destroy_snaps<I: IntoIterator<Item = S>, S: CStrArgument>(
        &self,
        snaps: I,
        defer: Defer,
    ) -> Result<(), (io::Error, NvList)> {
        let mut snaps_nv = NvList::new();

        for snap in snaps {
            snaps_nv.insert(snap, &()).unwrap();
        }

        self.destroy_snaps_raw(&snaps_nv, defer)
    }

    /// Corresponds to `lzc_destroy_snaps()`
    #[doc(alias = "lzc_destroy_snaps")]
    pub fn destroy_snaps_raw(
        &self,
        snaps: &NvList,
        defer: Defer,
    ) -> Result<(), (io::Error, NvList)> {
        let mut nv = ptr::null_mut();
        let v = unsafe {
            sys::lzc_destroy_snaps(
                snaps.as_ptr() as *mut _,
                bool::from(defer) as sys::boolean_t::Type,
                &mut nv,
            )
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
    pub fn snaprange_space<F: CStrArgument, L: CStrArgument>(
        &self,
        first_snap: F,
        last_snap: L,
    ) -> io::Result<u64> {
        let first_snap = first_snap.into_cstr();
        let last_snap = last_snap.into_cstr();

        let mut out = 0;
        let v = unsafe {
            sys::lzc_snaprange_space(
                first_snap.as_ref().as_ptr(),
                last_snap.as_ref().as_ptr(),
                &mut out,
            )
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
        let mut args = NvList::new_unique_names();

        // note: always include for compat with <=2.0.0
        args.insert("force", &force).unwrap();

        let v = unsafe {
            sys::lzc_sync(
                pool_name.as_ref().as_ptr(),
                args.as_ptr() as *mut _,
                ptr::null_mut(),
            )
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
    pub fn hold_raw(
        &self,
        holds: &NvListRef,
        cleanup_fd: Option<RawFd>,
    ) -> Result<(), Result<io::Error, NvList>> {
        let mut errs = ptr::null_mut();
        let v = unsafe {
            sys::lzc_hold(
                holds.as_ptr() as *mut _,
                cleanup_fd.unwrap_or(-1),
                &mut errs,
            )
        };
        if v != 0 {
            // if we have an error list, the return value error is just one of the errors in the
            // list.
            if errs.is_null() {
                Err(Ok(io::Error::from_raw_os_error(v)))
            } else {
                Err(Err(unsafe { NvList::from_ptr(errs) }))
            }
        } else {
            Ok(())
        }
    }

    /// Create a set of holds, each on a given snapshot
    ///
    /// Related: [`get_holds`], [`release`], [`hold_raw`], [`release_raw`].
    ///
    /// Corresponds to `lzc_hold`.
    #[doc(alias = "lzc_hold")]
    pub fn hold<'a, H, S, N>(&self, holds: H, cleanup_fd: Option<RawFd>) -> Result<(), Error>
    where
        H: IntoIterator<Item = &'a (S, N)>,
        S: 'a + CStrArgument + Clone,
        N: 'a + CStrArgument + Clone,
    {
        let mut holds_nv = NvList::new();

        for he in holds {
            let ds = he.0.clone().into_cstr();
            let name = he.1.clone().into_cstr();
            holds_nv.insert(ds.as_ref(), name.as_ref()).unwrap();
        }

        match self.hold_raw(&holds_nv, cleanup_fd) {
            Ok(a) => Ok(a),
            Err(Ok(v)) => Err(Error::Io { source: v }),
            Err(Err(v)) => Err(Error::List { source: v.into() }),
        }
    }

    /// Release holds from various snapshots
    ///
    /// The holds nvlist is `[(snap_name, [hold_names])]`, allowing multiple holds for multiple
    /// snapshots to be released with one call.
    ///
    /// Related: [`release`]
    ///
    /// Corresponds to `lzc_release`.
    #[doc(alias = "lzc_release")]
    pub fn release_raw(&self, holds: &NvListRef) -> Result<(), Result<io::Error, NvList>> {
        let mut errs = ptr::null_mut();
        let v = unsafe { sys::lzc_release(holds.as_ptr() as *mut _, &mut errs) };
        if v != 0 {
            if errs.is_null() {
                Err(Ok(io::Error::from_raw_os_error(v)))
            } else {
                Err(Err(unsafe { NvList::from_ptr(errs) }))
            }
        } else {
            Ok(())
        }
    }

    /// For a list of datasets, release one or more holds by name
    ///
    /// Corresponds to `lzc_release`.
    #[doc(alias = "lzc_release")]
    pub fn release<'a, F, C, H, N>(&self, holds: F) -> Result<(), Error>
    where
        F: IntoIterator<Item = &'a (C, H)>,
        C: 'a + CStrArgument + Clone,
        H: 'a + IntoIterator<Item = N> + Clone,
        N: 'a + CStrArgument + Clone,
    {
        let mut r_nv = NvList::new();

        for hi in holds {
            let mut hold_nv = NvList::new();

            for hold_name in hi.1.clone() {
                hold_nv.insert(hold_name, &()).unwrap();
            }

            r_nv.insert(hi.0.clone(), hold_nv.as_ref()).unwrap();
        }

        match self.release_raw(&r_nv) {
            Ok(a) => Ok(a),
            Err(Ok(v)) => Err(Error::Io { source: v }),
            Err(Err(v)) => Err(Error::List { source: v.into() }),
        }
    }

    /// Get the holds for a given snapshot
    ///
    /// The returned nvlist is `[(hold_name: String, unix_timestamp_seconds: u64)]`, where the unix
    /// timestamp is when the hold was created.
    ///
    /// Corresponds to `lzc_get_holds()`
    #[doc(alias = "lzc_get_holds")]
    pub fn get_holds<S: CStrArgument>(&self, snapname: S) -> io::Result<HoldList> {
        let snapname = snapname.into_cstr();
        let mut holds = ptr::null_mut();
        let v = unsafe { sys::lzc_get_holds(snapname.as_ref().as_ptr(), &mut holds) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(HoldList::new(unsafe { NvList::from_ptr(holds) }))
        }
    }

    /// Send the described stream
    ///
    /// Internally, is a wrapper around [`send_resume_redacted()`]
    ///
    /// Corresponds to `lzc_send()`
    #[doc(alias = "lzc_send")]
    pub fn send<S: CStrArgument, F: CStrArgument>(
        &self,
        snapname: S,
        from: Option<F>,
        fd: RawFd,
        flags: SendFlags,
    ) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let from = from.map(|a| a.into_cstr());

        let v = unsafe {
            sys::lzc_send(
                snapname.as_ref().as_ptr(),
                from.map_or(ptr::null(), |x| x.as_ref().as_ptr()),
                fd,
                flags.into(),
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Send the described redacted stream
    ///
    /// Internally, is a wrapper around [`send_resume_redacted()`]
    ///
    /// Corresponds to `lzc_send_redacted()`
    #[doc(alias = "lzc_send_redacted")]
    #[cfg(features = "v2_00")]
    pub fn send_redacted<S: CStrArgument, F: CStrArgument, R: CStrArgument>(
        &self,
        snapname: S,
        from: F,
        fd: RawFd,
        redactbook: R,
        flags: SendFlags,
    ) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let from = from.into_cstr();
        let redactbook = redactbook.into_cstr();

        let v = unsafe {
            sys::lzc_send_redacted(
                snapname.as_ref().as_ptr(),
                from.as_ref().as_ptr(),
                fd,
                redactbook.as_ref().as_ptr(),
                flags.into(),
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Send the described stream with resume information
    ///
    /// Internally, this is a wrapper around [`send_resume_redacted()`].
    ///
    /// Corresponds to `lzc_send_resume()`
    #[doc(alias = "lzc_send_resume")]
    pub fn send_resume<S: CStrArgument, F: CStrArgument>(
        &self,
        snapname: S,
        from: F,
        fd: RawFd,
        flags: SendFlags,
        resume_obj: u64,
        resume_off: u64,
    ) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let from = from.into_cstr();

        let v = unsafe {
            sys::lzc_send_resume(
                snapname.as_ref().as_ptr(),
                from.as_ref().as_ptr(),
                fd,
                flags.into(),
                resume_obj,
                resume_off,
            )
        };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Send the described stream with resume and redact info
    ///
    /// Corresponds to `lzc_send_resume_redacted()`
    #[doc(alias = "lzc_send_resume_redacted")]
    #[cfg(features = "v2_00")]
    pub fn send_resume_redacted<S: CStrArgument, F: CStrArgument, R: CStrArgument>(
        &self,
        snapname: S,
        from: F,
        fd: RawFd,
        flags: SendFlags,
        resume_obj: u64,
        resume_off: u64,
        redactbook: R,
    ) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let from = from.into_cstr();
        let redactbook = redactbook.into_cstr();

        let r = unsafe {
            sys::lzc_send_resume_redacted(
                snapname.as_ref().as_ptr(),
                from.as_ref().as_ptr(),
                fd,
                flags.into(),
                resume_obj,
                resume_off,
                redactbook.as_ref().as_ptr(),
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /// Estimate the size of a send stream
    ///
    /// Corresponds to `lzc_send_space_resume_redacted()`
    // FIXME: many parameters should probably be `Option<T>`
    // TODO: consider passing arguments here as a struct so we can use names
    #[doc(alias = "lzc_send_space_resume_redacted")]
    #[cfg(features = "v2_00")]
    pub fn send_space_resume_redacted<S: CStrArgument, F: CStrArgument, R: CStrArgument>(
        &self,
        snapname: S,
        from: F,
        flags: SendFlags,
        resume_obj: u64,
        resume_off: u64,
        resume_bytes: u64,
        redactbook: R,
        fd: RawFd,
    ) -> io::Result<u64> {
        let snapname = snapname.into_cstr();
        let from = from.into_cstr();

        let mut space = 0;

        let r = unsafe {
            sys::lzc_send_space_resume_redacted(
                snapname.as_ref().as_ptr(),
                from.as_ref().as_ptr(),
                flags.into(),
                resume_obj,
                resume_off,
                resume_bytes,
                redactbook,
                fd,
                &mut space,
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(space)
        }
    }

    /// Estimate the size of the stream to be sent if [`send`] were called with the same arguments
    ///
    /// Internally, this is a wrapper around [`send_space_resume_redacted()`].
    ///
    /// Corresponds to `lzc_send_space()`
    #[doc(alias = "lzc_send_space")]
    pub fn send_space<S: CStrArgument, F: CStrArgument>(
        &self,
        snapname: S,
        from: F,
        flags: SendFlags,
    ) -> io::Result<u64> {
        let snapname = snapname.into_cstr();
        let from = from.into_cstr();

        let mut space = 0;
        let r = unsafe {
            sys::lzc_send_space(
                snapname.as_ref().as_ptr(),
                from.as_ref().as_ptr(),
                flags.into(),
                &mut space,
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(space)
        }
    }

    /// Corresponds to `lzc_receive()`
    #[doc(alias = "lzc_receive")]
    pub fn receive<S: CStrArgument, O: CStrArgument>(
        &self,
        snapname: S,
        props: Option<&NvListRef>,
        origin: Option<O>,
        force: bool,
        raw: bool,
        fd: RawFd,
    ) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let origin = origin.map(|x| x.into_cstr());

        let r = unsafe {
            sys::lzc_receive(
                snapname.as_ref().as_ptr(),
                props.map_or(ptr::null_mut(), |x| x.as_ptr() as *mut _),
                origin.map_or(ptr::null(), |x| x.as_ref().as_ptr()),
                if force { 1 } else { 0 },
                if raw { 1 } else { 0 },
                fd,
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_receive_resumable()`
    // internally, only a flag differs from `recv`
    // consider implimenting something that takes `resumeable` as a flag
    #[doc(alias = "lzc_receive_resumable")]
    pub fn receive_resumable<S: CStrArgument, O: CStrArgument>(
        &self,
        snapname: S,
        props: &NvListRef,
        origin: O,
        force: bool,
        raw: bool,
        fd: RawFd,
    ) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let origin = origin.into_cstr();

        let r = unsafe {
            sys::lzc_receive_resumable(
                snapname.as_ref().as_ptr(),
                props.as_ptr() as *mut _,
                origin.as_ref().as_ptr(),
                if force { 1 } else { 0 },
                if raw { 1 } else { 0 },
                fd,
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
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
            sys::lzc_rollback(
                fsname.as_ref().as_ptr(),
                rname.as_mut_ptr() as *mut std::os::raw::c_char,
                rname.len() as std::os::raw::c_int,
            )
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
    pub fn rollback_to<F: CStrArgument, S: CStrArgument>(
        &self,
        fsname: F,
        snapname: S,
    ) -> io::Result<()> {
        let fsname = fsname.into_cstr();
        let snapname = snapname.into_cstr();

        let r =
            unsafe { sys::lzc_rollback_to(fsname.as_ref().as_ptr(), snapname.as_ref().as_ptr()) };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /// Create bookmarks from existing snapshot or bookmark
    #[doc(alias = "lzc_bookmark")]
    pub fn bookmark<I: IntoIterator<Item = (D, S)>, D: CStrArgument, S: CStrArgument>(
        &self,
        bookmarks: I,
    ) -> Result<(), ErrorList> {
        let mut bookmarks_nv = NvList::new();

        for (new_bm, src) in bookmarks {
            let src = src.into_cstr();
            bookmarks_nv.insert(new_bm, src.as_ref()).unwrap();
        }

        match self.bookmark_raw(&bookmarks_nv) {
            Ok(a) => Ok(a),
            Err((_, err_nv)) => Err(ErrorList::from(err_nv)),
        }
    }

    /// Create bookmarks from existing snapshot or bookmark
    ///
    /// The `bookmarks` nvlist is `[(full_name_of_new_bookmark,
    /// full_name_of_source_snap_or_bookmark)]`.
    ///
    /// Corresponds to `lzc_bookmark()`
    #[doc(alias = "lzc_bookmark")]
    pub fn bookmark_raw(&self, bookmarks: &NvListRef) -> Result<(), (io::Error, NvList)> {
        let mut err = ptr::null_mut();
        let r = unsafe { sys::lzc_bookmark(bookmarks.as_ptr() as *mut _, &mut err) };

        if r != 0 {
            Err((io::Error::from_raw_os_error(r), unsafe {
                NvList::from_ptr(err)
            }))
        } else {
            Ok(())
        }
    }

    /// Retreive bookmarks for the given filesystem
    ///
    /// `props` is a list of `[(prop_name, ())]`, where `prop_name` names a property on a bookmark.
    /// All the named properties are returned in the return value as the values of each bookmark.
    ///
    /// Corresponds to `lzc_get_bookmarks()`
    #[doc(alias = "lzc_get_bookmarks")]
    pub fn get_bookmarks_raw<F: CStrArgument>(
        &self,
        fsname: F,
        props: &NvListRef,
    ) -> io::Result<NvList> {
        let mut res = ptr::null_mut();
        let fsname = fsname.into_cstr();

        let r = unsafe {
            sys::lzc_get_bookmarks(fsname.as_ref().as_ptr(), props.as_ptr() as *mut _, &mut res)
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(unsafe { NvList::from_ptr(res) })
        }
    }

    /// Corresponds to `lzc_get_bookmark_props()`
    #[doc(alias = "lzc_get_bookmark_props")]
    pub fn get_bookmark_props<B: CStrArgument>(&self, bookmark: B) -> io::Result<NvList> {
        let mut res = ptr::null_mut();
        let bookmark = bookmark.into_cstr();

        let r = unsafe { sys::lzc_get_bookmark_props(bookmark.as_ref().as_ptr(), &mut res) };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(unsafe { NvList::from_ptr(res) })
        }
    }

    /// Corresponds to `lzc_destroy_bookmarks()`
    #[doc(alias = "lzc_destroy_bookmarks")]
    pub fn destroy_bookmarks(&self, bookmarks: &NvListRef) -> Result<(), (io::Error, NvList)> {
        let mut errs = ptr::null_mut();

        let r = unsafe { sys::lzc_destroy_bookmarks(bookmarks.as_ptr() as *mut _, &mut errs) };

        if r != 0 {
            Err((io::Error::from_raw_os_error(r), unsafe {
                NvList::from_ptr(errs)
            }))
        } else {
            Ok(())
        }
    }

    /// Execute a channel program
    ///
    /// root privlidges are required to execute a channel program
    ///
    /// Corresponds to `lzc_channel_program()`
    // 0.8.?
    #[doc(alias = "lzc_channel_program")]
    pub fn channel_program<P: CStrArgument, R: CStrArgument>(
        &self,
        pool: P,
        program: R,
        instruction_limit: u64,
        memlimit: u64,
        args: &NvListRef,
    ) -> io::Result<NvList> {
        let mut out_nv = ptr::null_mut();

        let pool = pool.into_cstr();
        let program = program.into_cstr();

        let r = unsafe {
            sys::lzc_channel_program(
                pool.as_ref().as_ptr(),
                program.as_ref().as_ptr(),
                instruction_limit,
                memlimit,
                args.as_ptr() as *mut _,
                &mut out_nv,
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(unsafe { NvList::from_ptr(out_nv) })
        }
    }

    /// Execute a read-only channel program
    ///
    /// root privlidges are required to execute a channel program (even a read-only one)
    ///
    /// Corresponds to `lzc_channel_program_nosync()`
    #[doc(alias = "lzc_channel_program_nosync")]
    pub fn channel_program_nosync<P: CStrArgument, R: CStrArgument>(
        &self,
        pool: P,
        program: R,
        instruction_limit: u64,
        memlimit: u64,
        args: &NvListRef,
    ) -> io::Result<NvList> {
        let mut out_nv = ptr::null_mut();

        let pool = pool.into_cstr();
        let program = program.into_cstr();

        let r = unsafe {
            sys::lzc_channel_program_nosync(
                pool.as_ref().as_ptr(),
                program.as_ref().as_ptr(),
                instruction_limit,
                memlimit,
                args.as_ptr() as *mut _,
                &mut out_nv,
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(unsafe { NvList::from_ptr(out_nv) })
        }
    }

    /// Create a pool checkpoint
    ///
    /// Corresponds to `lzc_pool_checkpoint()`
    ///
    // FIXME: libzfs_core.c lists the specific error returns
    // 0.8.?
    #[doc(alias = "lzc_pool_checkpoint")]
    pub fn pool_checkpoint<P: CStrArgument>(&self, pool: P) -> io::Result<()> {
        let pool = pool.into_cstr();

        let r = unsafe { sys::lzc_pool_checkpoint(pool.as_ref().as_ptr()) };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /// Discard the pool checkpoint
    ///
    /// Corresponds to `lzc_pool_checkpoint_discard()`
    #[doc(alias = "lzc_pool_checkpoint_discard")]
    pub fn pool_checkpoint_discard<P: CStrArgument>(&self, pool: P) -> io::Result<()> {
        let pool = pool.into_cstr();

        let r = unsafe { sys::lzc_pool_checkpoint_discard(pool.as_ref().as_ptr()) };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_load_key()`
    #[doc(alias = "lzc_load_key")]
    pub fn load_key<F: CStrArgument>(
        &self,
        fsname: F,
        noop: bool,
        keydata: &[u8],
    ) -> io::Result<()> {
        let fsname = fsname.into_cstr();

        let r = unsafe {
            sys::lzc_load_key(
                fsname.as_ref().as_ptr(),
                if noop {
                    sys::boolean_t::B_TRUE
                } else {
                    sys::boolean_t::B_FALSE
                },
                keydata.as_ptr() as *mut _,
                keydata.len().try_into().unwrap(),
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_unload_key()`
    #[doc(alias = "lzc_unload_key")]
    pub fn unload_key<F: CStrArgument>(&self, fsname: F) -> io::Result<()> {
        let fsname = fsname.into_cstr();

        let r = unsafe { sys::lzc_unload_key(fsname.as_ref().as_ptr()) };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_change_key()`
    #[doc(alias = "lzc_change_key")]
    pub fn change_key<F: CStrArgument>(
        &self,
        fsname: F,
        crypt_cmd: u64,
        props: &NvListRef,
        keydata: Option<&[u8]>,
    ) -> io::Result<()> {
        let fsname = fsname.into_cstr();

        let (k, l) = keydata.map_or((ptr::null_mut(), 0), |v| (v.as_ptr() as *mut _, v.len()));
        let r = unsafe {
            sys::lzc_change_key(
                fsname.as_ref().as_ptr(),
                crypt_cmd,
                props.as_ptr() as *mut _,
                k,
                l.try_into().unwrap(),
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_reopen()`
    // 0.8.0
    #[doc(alias = "lzc_reopen")]
    pub fn reopen<P: CStrArgument>(&self, pool: P, scrub_restart: bool) -> io::Result<()> {
        let pool = pool.into_cstr();

        let r = unsafe {
            sys::lzc_reopen(
                pool.as_ref().as_ptr(),
                if scrub_restart {
                    sys::boolean_t::B_TRUE
                } else {
                    sys::boolean_t::B_FALSE
                },
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_initialize()`
    // 0.8.0
    #[doc(alias = "lzc_initialize")]
    pub fn initialize<P: CStrArgument>(
        &self,
        pool: P,
        initialize_func: PoolInitializeFunc,
        vdevs: &NvListRef,
    ) -> Result<(), (io::Error, NvList)> {
        let pool = pool.into_cstr();

        let mut err_nv = ptr::null_mut();
        let r = unsafe {
            sys::lzc_initialize(
                pool.as_ref().as_ptr(),
                initialize_func.as_raw(),
                vdevs.as_ptr() as *mut _,
                &mut err_nv,
            )
        };

        if r != 0 {
            Err((io::Error::from_raw_os_error(r), unsafe {
                NvList::from_ptr(err_nv)
            }))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_trim()`
    // 0.8.0
    #[doc(alias = "lzc_trim")]
    pub fn trim<P: CStrArgument>(
        &self,
        pool: P,
        pool_trim_func: PoolTrimFunc,
        rate: u64,
        secure: bool,
        vdevs: &NvListRef,
    ) -> Result<(), (io::Error, NvList)> {
        let pool = pool.into_cstr();

        let mut err_nv = ptr::null_mut();
        let r = unsafe {
            sys::lzc_trim(
                pool.as_ref().as_ptr(),
                pool_trim_func.as_raw(),
                rate,
                if secure {
                    sys::boolean_t::B_TRUE
                } else {
                    sys::boolean_t::B_FALSE
                },
                vdevs.as_ptr() as *mut _,
                &mut err_nv,
            )
        };

        if r != 0 {
            Err((io::Error::from_raw_os_error(r), unsafe {
                NvList::from_ptr(err_nv)
            }))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_redact()`
    #[cfg(features = "v2_00")]
    #[doc(alias = "lzc_redact")]
    pub fn redact<S: CStrArgument, B: CStrArgument>(
        &self,
        snapname: S,
        bookname: B,
        snapnv: &NvListRef,
    ) -> io::Result<()> {
        let snapname = snapname.into_cstr();
        let bookname = bookname.into_cstr();

        let r = unsafe {
            sys::lzc_redact(
                snapname.as_ref().as_ptr(),
                bookname.as_ref().as_ptr(),
                snapnv.as_ptr() as *mut _,
            )
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(())
        }
    }

    /// Corresponds to `lzc_wait()`
    #[cfg(features = "v2_00")]
    #[doc(alias = "lzc_wait")]
    pub fn wait<P: CStrArgument>(&self, pool: P, activity: WaitActivity) -> io::Result<bool> {
        let pool = pool.into_cstr();

        let mut waited = sys::boolean_t::B_FALSE;
        let r = unsafe { sys::lzc_wait(pool.as_ref().as_ptr(), activity.as_raw(), &mut waited) };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(waited != sys::boolean_t::B_FALSE)
        }
    }

    /// Corresponds to `lzc_wait_tag()`
    #[cfg(features = "v2_00")]
    #[doc(alias = "lzc_wait_tag")]
    pub fn wait_tag<P: CStrArgument>(
        &self,
        pool: P,
        activity: WaitActivity,
        tag: u64,
    ) -> io::Result<bool> {
        let pool = pool.into_cstr();

        let mut waited = sys::boolean_t::B_FALSE;
        let r = unsafe {
            sys::lzc_wait_tag(pool.as_ref().as_ptr(), activity.as_raw(), tag, &mut waited)
        };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(waited != sys::boolean_t::B_FALSE)
        }
    }

    /// Corresponds to `lzc_wait_fs()`
    #[cfg(features = "v2_00")]
    #[doc(alias = "lzc_wait_fs")]
    pub fn wait_fs<F: CStrArgument>(&self, fs: F, activity: WaitActivity) -> io::Result<bool> {
        let fs = fs.into_cstr();

        let mut waited = sys::boolean_t::B_FALSE;
        let r =
            unsafe { sys::lzc_wait_fs(fs.as_ref().as_ptr(), activity.as_raw(), tag, &mut waited) };

        if r != 0 {
            Err(io::Error::from_raw_os_error(r))
        } else {
            Ok(waited != sys::boolean_t::B_FALSE)
        }
    }

    /// Corresponds to `lzc_set_bootenv()`
    #[cfg(features = "v2_00")]
    #[doc(alias = "lzc_set_bootenv")]
    pub fn set_bootenv<P: CStrArgument, E: CStrArgument>(
        &self,
        pool: P,
        env: &NvListRef,
    ) -> io::Result<()> {
        let pool = pool.into_cstr();
        let v = unsafe { sys::lzc_set_bootenv(pool.as_ref().as_ptr(), env.as_ptr()) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(())
        }
    }

    /// Corresponds `lzc_get_bootenv()`
    #[cfg(features = "v2_00")]
    #[doc(alias = "lzc_get_bootenv")]
    pub fn get_bootenv<P: CStrArgument>(&self, pool: P) -> io::Result<NvList> {
        let pool = pool.into_cstr();
        let mut env = ptr::null_mut();
        let v = unsafe { sys::lzc_get_bootenv(pool.as_ref().as_ptr(), &mut env) };
        if v != 0 {
            Err(io::Error::from_raw_os_error(v))
        } else {
            Ok(unsafe { NvList::from_ptr(env) })
        }
    }
}

impl Drop for Zfs {
    fn drop(&mut self) {
        unsafe { sys::libzfs_core_fini() }
    }
}

#[derive(Debug, PartialEq)]
pub enum PoolInitializeFunc {
    Start,
    Cancel,
    Suspend,
}

impl PoolInitializeFunc {
    pub fn as_raw(&self) -> sys::pool_initialize_func_t {
        use sys::pool_initialize_func as ifc;
        use PoolInitializeFunc::*;

        match self {
            Start => ifc::POOL_INITIALIZE_START,
            Cancel => ifc::POOL_INITIALIZE_CANCEL,
            Suspend => ifc::POOL_INITIALIZE_SUSPEND,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PoolTrimFunc {
    Start,
    Cancel,
    Suspend,
}

impl PoolTrimFunc {
    pub fn as_raw(&self) -> sys::pool_trim_func_t {
        use sys::pool_trim_func as ptf;
        use PoolTrimFunc::*;
        match self {
            Start => ptf::POOL_TRIM_START,
            Cancel => ptf::POOL_TRIM_CANCEL,
            Suspend => ptf::POOL_TRIM_SUSPEND,
        }
    }
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

    #[cfg(features = "v2_00")]
    pub saved: bool,
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
        #[cfg(features = "v2_00")]
        if sf.saved {
            f |= sys::lzc_send_flags::LZC_SEND_FLAG_SAVED;
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

/// A list of holds for a given snapshot
#[derive(Debug)]
pub struct HoldList {
    nv: NvList,
}

impl HoldList {
    fn new(nv: NvList) -> Self {
        Self { nv }
    }
}

impl From<HoldList> for NvList {
    fn from(hl: HoldList) -> Self {
        hl.nv
    }
}

impl AsRef<NvListRef> for HoldList {
    fn as_ref(&self) -> &NvListRef {
        &self.nv
    }
}

/// Iterator of holds in the [`HoldList`]
#[derive(Debug)]
pub struct HoldListIter<'a> {
    iter: NvListIter<'a>,
}

impl<'a> IntoIterator for &'a HoldList {
    type Item = (&'a ffi::CStr, std::time::SystemTime);
    type IntoIter = HoldListIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        HoldListIter {
            iter: (&self.nv).into_iter(),
        }
    }
}

impl<'a> Iterator for HoldListIter<'a> {
    type Item = (&'a ffi::CStr, std::time::SystemTime);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(nvp) => {
                let t = match nvp.data() {
                    nvpair::NvData::Uint64(time_sec) => {
                        std::time::UNIX_EPOCH + std::time::Duration::from_secs(time_sec)
                    }
                    v => panic!("unexpected datatype in hold list {:?}", v),
                };
                Some((nvp.name(), t))
            }
            None => None,
        }
    }
}
