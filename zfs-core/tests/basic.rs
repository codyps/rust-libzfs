extern crate nvpair;
extern crate rand;
extern crate zfs_core as zfs;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::io;

struct TempFs {
    path: String,
}

// How many times should we (re)try finding an unused random name? It should be
// enough that an attacker will run out of luck before we run out of patience.
const NUM_RETRIES: u32 = 1 << 31;
// How many characters should we include in a random file name? It needs to
// be enough to dissuade an attacker from trying to preemptively create names
// of that length, but not so huge that we unnecessarily drain the random number
// generator of entropy.
const NUM_RAND_CHARS: usize = 12;

fn tmp_zpool_name() -> String {
    std::env::var("ZFS_TEMPFS").expect("ZFS_TEMPFS should be set to a zfs filesystem that can be used for temporary datasets")
}

impl TempFs {
    fn with_base(base: &str, prefix: &str) -> io::Result<TempFs> {
        let z = zfs::Zfs::new()?;
        let nv = nvpair::NvList::new()?;

        let rng = thread_rng();
        for _ in 0..NUM_RETRIES {
            let suffix: String = rng.sample_iter(Alphanumeric).take(NUM_RAND_CHARS).collect();

            let mut path = base.to_owned();
            path.push_str("/");

            if !prefix.is_empty() {
                path.push_str(prefix);
                path.push_str("-");
            }
            path.push_str(&suffix);

            match z.create(&path, zfs::DataSetType::Zfs, &nv) {
                Ok(_) => return Ok(TempFs { path }),
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e),
            }
        }

        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "too many temporary filesystems already exist",
        ))
    }

    pub fn new(prefix: &str) -> io::Result<TempFs> {
        Self::with_base(
            &tmp_zpool_name(),
            prefix
        )
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Drop for TempFs {
    fn drop(&mut self) {
        let z = zfs::Zfs::new().unwrap();
        if let Err(e) = z.destroy(&self.path) {
            eprintln!("Could not destroy TempFs {}: {}", self.path, e);
        }
    }
}

#[test]
fn new() {
    let _ = zfs::Zfs::new().unwrap();
}

#[test]
fn create_destroy() {
    let tmpfs = TempFs::new("create").unwrap();

    let mut b = tmpfs.path().to_owned();
    b.push_str("/");
    b.push_str("create");

    let z = zfs::Zfs::new().unwrap();
    let nv = nvpair::NvList::new().unwrap();
    z.create(&b, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", b));

    assert_eq!(z.exists(&b), true);
    let mut b2 = b.clone();
    b2.push_str("fooie");
    assert_eq!(z.exists(&b2), false);

    z.destroy(&b).unwrap();
}

#[test]
fn rename() {
    let tmpfs = TempFs::new("rename").unwrap();

    let mut b = tmpfs.path().to_owned();
    b.push_str("/");
    let mut b_new = b.clone();
    b.push_str("orig");
    b_new.push_str("new");

    let z = zfs::Zfs::new().unwrap();
    let nv = nvpair::NvList::new().unwrap();
    z.create(&b, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", b));

    assert_eq!(z.exists(&b), true);

    z.rename(&b, &b_new).unwrap();
    assert_eq!(z.exists(&b), false);
    assert_eq!(z.exists(&b_new), true);

    z.destroy(&b_new).unwrap();
}

#[test]
fn snapshot() {
    let tmpfs = TempFs::new("snapshot").unwrap();

    let mut b = tmpfs.path().to_owned();
    b.push_str("/");
    let mut b_new = b.clone();
    b.push_str("orig");
    b_new.push_str("clone");

    let z = zfs::Zfs::new().unwrap();
    let nv = nvpair::NvList::new().unwrap();
    z.create(&b, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", b));

    assert_eq!(z.exists(&b), true);

    let mut props = nvpair::NvList::new().unwrap();
    let mut snaps = nvpair::NvList::new().unwrap();
    let mut b_snap = b.clone();
    b_snap.push_str("@a");
    snaps.insert(&b_snap, ()).unwrap();
    z.snapshot(&snaps, &props).unwrap();

    assert_eq!(z.exists(&b), true);
    assert_eq!(z.exists(&b_snap), true);

    z.destroy(&b_snap).unwrap();
    z.destroy(&b).unwrap();
}

#[cfg(features = "v2_00")]
#[test]
fn bootenv() {
    let z = zfs::Zfs::new().unwrap();
    let pool = tmp_zpool_name();

    let r = z.bootenv(pool);

    println!("{:?}", r);
    panic!();
}
