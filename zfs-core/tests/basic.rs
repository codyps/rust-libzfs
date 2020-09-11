extern crate zfs_core as zfs;
extern crate rand;
extern crate nvpair;

use std::iter;
use std::io;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

fn test_fsname_base(extra: &str) -> String
{
    let mut b = std::env::var("ZFS_TEST_BASE").expect("Set ZFS_TEST_BASE to a suitable zfs fspath to run tests on");
    b.push_str("/");
    b.push_str(module_path!());
    b.push_str("-");
    b.push_str(extra);
    b
}

struct TempFs {
    path: String
}

// How many times should we (re)try finding an unused random name? It should be
// enough that an attacker will run out of luck before we run out of patience.
const NUM_RETRIES: u32 = 1 << 31;
// How many characters should we include in a random file name? It needs to
// be enough to dissuade an attacker from trying to preemptively create names
// of that length, but not so huge that we unnecessarily drain the random number
// generator of entropy.
const NUM_RAND_CHARS: usize = 12;

impl TempFs {
    fn with_base(base: &str, prefix: &str) -> io::Result<TempFs> {
        let z = zfs::Zfs::new()?;
        let nv = nvpair::NvList::new()?;

        let rng = thread_rng();
        for _ in 0..NUM_RETRIES {
            let suffix: String = rng.sample_iter(Alphanumeric)
                .take(NUM_RAND_CHARS)
                .collect();

            let mut path = base.to_owned();
            path.push_str("/");

            if !prefix.is_empty() {
                path.push_str(prefix);
                path.push_str("-");
            }
            path.push_str(&suffix);

            match z.create(&path , zfs::DataSetType::Zfs, &nv) {
                Ok(_) => return Ok(TempFs { path }),
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {},
                Err(e) => return Err(e),

            }
        }

        Err(io::Error::new(io::ErrorKind::AlreadyExists, "too many temporary filesystems already exist"))
    }

    pub fn new(prefix: &str) -> io::Result<TempFs> {
        Self::with_base(
            &std::env::var("ZFS_TEMPFS").expect("ZFS_TEMPFS should be set to a zfs filesystem that can be used for temporary datasets"),
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
    z.create(&b, zfs::DataSetType::Zfs, &nv).expect(&format!("create {:?} failed", b));
    z.destroy(&b).unwrap();
}
