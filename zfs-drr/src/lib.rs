//! DMU Replay Record handling
//!
//! `zfs send` and `zfs receive` exchange data formatted as `dmu_replay_record`s. These

use std::io::{self, Read};
use snafu::{Snafu, ResultExt};
use std::mem::{self, MaybeUninit};

pub struct DmuReplayRecord {
    raw: DmuReplayRecordRaw,
    byte_swap: bool,
}

impl DmuReplayRecord {
    pub fn read_from<R: Read>(mut r: R) -> Result<Self, Error> {
        let mut drr = MaybeUninit::<DmuReplayRecordRaw>::uninit();
        let drr_slice = unsafe { std::slice::from_raw_parts_mut(drr.as_mut_ptr() as *mut _, mem::size_of::<DmuReplayRecordRaw>()) };
        
        r.read_exact(drr_slice).context(ReadFailed)?;

        // FIXME: this assume_init is pretty sketchy. we need to ensure we don't have any 
        let s = Self {
            raw: unsafe { drr.assume_init() },
            byte_swap: false,
        };

        // detect byte swap
        if s.raw.drr_type != DrrType::Begin as u32 {
            return NotBeginType { found_type: s.raw.drr_type }.fail();
        }

        // TODO: examine magic to determine byte swap

        Ok(s)
    }

    pub fn from_bytes(_raw: &[u8]) -> Result<Self, Error> {
        todo!()
    }

    pub fn byte_swap(&self) -> bool {
        self.byte_swap
    }
}


#[derive(Clone)]
#[repr(C)]
pub struct DmuReplayRecordRaw {
    /// [`DrrType`]
    drr_type: u32,
    payload_len: u32,
    content: DmuReplayRecordContent,
}

const MAXNAMELEN: usize = 256;

#[derive(Clone, Copy)]
#[repr(C)]
pub enum DrrType {
    Begin = 0,
    Object = 1,
    FreeObjects = 2,
    Write = 3,
    Free = 4,
    End = 5,
    WriteByref = 6,
    Spill = 7,
    WriteEmbedeed = 8,
    ObjectRange = 9,
    Redact = 10,
}

// we probably only need `begin` and `checksum` (for size)
#[derive(Clone, Copy)]
#[repr(C)]
pub union DmuReplayRecordContent {
    begin: DrrBegin,
    end: DrrEnd,
    /*
    object: DrrObject,
    free_objects: FreeObjects,
    write: DrrWrite,
    free: DrrFree,
    write_byref: DrrWriteByref,
    spill: DrrSpill,
    write_embedded: DrrWriteEmbedded,
    object_range: DrrObjectRange,
    redact: DrrRedact,
    */
    checksum: DrrChecksum,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct DrrBegin {
    magic: u64,
    versioninfo: u64,
    creation_time: u64,
    drr_type: DmuObjsetType,
    flags: u32,
    to_guid: u64,
    from_guid: u64,
    to_name: [u8;MAXNAMELEN],
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct DrrEnd {
    checksum: ZioCksum,
    to_guid: u64,
}

/*
#[repr(C)]
struct DrrObject {
    object: u64,
    type_: DmuObjectType,
    bonus_type: DmuObjectType,
    blksz: u32,
    bonus_len: u32,
    checksum_type: u8,
    compress: u8,
    dn_slots: u8,
    flags: u8,
    raw_bonus_len: u32,
    to_guid: u64,
    ind_blk_shift: u8,
    nlevels: u8,
    nblkptr: u8,
    pad: [u8;5],
    max_blkid: u64,
    /* bonus content follows */
}
*/

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct DrrChecksum {
    pad: [u64;34],
    checksum: ZioCksum,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
enum DmuObjsetType {
    None_ = 0,
    Meta = 1,
    Zfs = 2,
    Zvol = 3,
    /// For testing only
    Other = 4,
    Any = 5,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct ZioCksum {
    word: [u64; 4],
}


#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("reading data for the dmu_replay_record failed: {}", source))]
    ReadFailed { source: std::io::Error },

    #[snafu(display("dmu_replay_record type is not Begin, can't determine endian. got type: {}", found_type))]
    NotBeginType { found_type: u32 },
}
