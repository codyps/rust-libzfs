extern crate zfs_core as zfs;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::io;
use std::os::unix::io::AsRawFd;
use std::io::Seek;

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

    pub fn mount(&self) -> TempMount {
        TempMount::new(self)
    }
}

struct TempMount<'a> {
    tempfs: &'a TempFs,
    mount_dir: tempfile::TempDir,
}

impl<'a> TempMount<'a> {
    pub fn new(tempfs: &'a TempFs) -> Self {
        let mount_dir = tempfile::tempdir().unwrap();

        std::process::Command::new("mount")
            .arg("-t").arg("zfs")
            .arg(&tempfs.path)
            .arg(mount_dir.path())
            .status().unwrap();

        Self {
            tempfs,
            mount_dir,
        }
    }
}

impl<'a> Drop for TempMount<'a> {
    fn drop(&mut self) {
        if let Err(e) = std::process::Command::new("umount")
            .arg(self.mount_dir.path()).status() {
            eprintln!("Could not unmount TempMount: {:?}: {}", self.mount_dir.path(), e);
        }
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

    let mut b_snap = b.clone();
    b_snap.push_str("@a");
    z.snapshot([b_snap.as_str()].iter().cloned()).unwrap();

    assert_eq!(z.exists(&b), true);
    assert_eq!(z.exists(&b_snap), true);

    z.destroy_snaps([b_snap.as_str()].iter().cloned(), zfs::Defer::No).unwrap();
    z.destroy(&b).unwrap();
}

#[test]
fn snapshot_multi() {
    let tmpfs = TempFs::new("snapshot_multi").unwrap();

    let mut b = tmpfs.path().to_owned();
    b.push_str("/");
    let mut b_alt = b.clone();
    b.push_str("1");
    b_alt.push_str("2");

    let z = zfs::Zfs::new().unwrap();
    let nv = nvpair::NvList::new().unwrap();
    z.create(&b, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", b));
    z.create(&b_alt, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", b_alt));

    assert_eq!(z.exists(&b), true);
    assert_eq!(z.exists(&b_alt), true);

    let mut b_snap1 = b.clone();
    b_snap1.push_str("@a");
    let mut b_snap2 = b_alt.clone();
    b_snap2.push_str("@b");
    z.snapshot([b_snap1.as_str(), b_snap2.as_str()].iter().cloned()).unwrap();

    assert_eq!(z.exists(&b), true);
    assert_eq!(z.exists(&b_snap1), true);
    assert_eq!(z.exists(&b_snap2), true);

    z.destroy_snaps([b_snap1.as_str(), b_snap2.as_str()].iter().cloned(), zfs::Defer::No).unwrap();
    z.destroy(&b).unwrap();
}

#[test]
fn hold_raw() {
    let tmpfs = TempFs::new("hold-raw").unwrap();

    let mut b = tmpfs.path().to_owned();
    b.push_str("/");
    let mut b_alt = b.clone();
    b.push_str("1");
    b_alt.push_str("2");

    let z = zfs::Zfs::new().unwrap();
    let nv = nvpair::NvList::new().unwrap();
    z.create(&b, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", b));
    z.create(&b_alt, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", b_alt));

    assert_eq!(z.exists(&b), true);
    assert_eq!(z.exists(&b_alt), true);

    let mut b_snap1 = b.clone();
    b_snap1.push_str("@a");
    let mut b_snap2 = b_alt.clone();
    b_snap2.push_str("@b");
    z.snapshot([b_snap1.as_str(), b_snap2.as_str()].iter().cloned()).unwrap();

    assert_eq!(z.exists(&b), true);
    assert_eq!(z.exists(&b_snap1), true);
    assert_eq!(z.exists(&b_snap2), true);

    let mut hold_snaps = nvpair::NvList::new().unwrap();
    hold_snaps.insert(&b_snap1, "hold-hello").unwrap();
    z.hold_raw(&hold_snaps, None).unwrap();

    z.destroy_snaps([b_snap1.as_str(), b_snap2.as_str()].iter().cloned(), zfs::Defer::Yes).unwrap();

    assert_eq!(z.exists(&b_snap1), true);
    assert_eq!(z.exists(&b_snap2), false);

    let mut release_snaps = nvpair::NvList::new().unwrap();
    let mut holds_for_snap = nvpair::NvList::new().unwrap();
    holds_for_snap.insert("hold-hello", &()).unwrap();
    release_snaps.insert(&b_snap1, holds_for_snap.as_ref()).unwrap();
    z.release_raw(&release_snaps).unwrap();

    assert_eq!(z.exists(&b_snap1), false);
    assert_eq!(z.exists(&b_snap2), false);

    z.destroy(&b).unwrap();
}

#[test]
fn send_recv() {
    let tmpfs = TempFs::new("send_recv").unwrap();
    let mut fs1 = tmpfs.path().to_owned();
    fs1.push_str("/");
    let mut fs2 = fs1.clone();
    fs1.push_str("1");
    fs2.push_str("2");

    let z = zfs::Zfs::new().unwrap();
    let nv = nvpair::NvList::new().unwrap();
    z.create(&fs1, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", fs1));

    assert_eq!(z.exists(&fs1), true);
    assert_eq!(z.exists(&fs2), false);

    let mut snap1 = fs1.clone();
    snap1.push_str("@a");
    z.snapshot([snap1.as_str()].iter().cloned()).unwrap();

    let mut snap2 = fs2.clone();
    snap2.push_str("@b");

    let mut stream = tempfile::tempfile().unwrap();
    z.send::<_, &str>(&snap1, None, stream.as_raw_fd(), zfs::SendFlags::default()).unwrap();
    stream.seek(io::SeekFrom::Start(0)).unwrap();
    z.receive::<_, &str>(&snap2, None, None, false, false, stream.as_raw_fd()).unwrap();

    assert_eq!(z.exists(&fs1), true);
    assert_eq!(z.exists(&fs2), true);
    assert_eq!(z.exists(&snap1), true);
    assert_eq!(z.exists(&snap2), true);

    z.destroy(&snap1).unwrap();
    z.destroy(&snap2).unwrap();
    z.destroy(&fs1).unwrap();
    z.destroy(&fs2).unwrap();
}

#[test]
fn rollback() {
    let tmpfs = TempFs::new("rollback").unwrap();
    let mut fs1 = tmpfs.path().to_owned();
    fs1.push_str("/");
    fs1.push_str("1");

    let z = zfs::Zfs::new().unwrap();
    let nv = nvpair::NvList::new().unwrap();
    z.create(&fs1, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", fs1));

    assert_eq!(z.exists(&fs1), true);

    let props = nvpair::NvList::new().unwrap();
    let mut snap1 = fs1.clone();

    {
        snap1.push_str("@a");
        let mut snaps = nvpair::NvList::new().unwrap();
        snaps.insert(&snap1, &()).unwrap();
        z.snapshot_raw(&snaps, &props).unwrap();
    }

    let mut snap2 = fs1.clone();

    {
        snap2.push_str("@b");
        let mut snaps = nvpair::NvList::new().unwrap();
        snaps.insert(&snap2, &()).unwrap();
        z.snapshot_raw(&snaps, &props).unwrap();
    }

    assert_eq!(z.rollback(&fs1).unwrap().to_str().unwrap(), snap2);

    assert_eq!(z.exists(&fs1), true);
    assert_eq!(z.exists(&snap1), true);
    assert_eq!(z.exists(&snap2), true);

    z.destroy(&snap2).unwrap();
    z.destroy(&snap1).unwrap();
    z.destroy(&fs1).unwrap();
}

#[test]
fn rollback_to() {
    let tmpfs = TempFs::new("rollback_to").unwrap();
    let mut fs1 = tmpfs.path().to_owned();
    fs1.push_str("/");
    fs1.push_str("1");

    let z = zfs::Zfs::new().unwrap();
    let nv = nvpair::NvList::new().unwrap();
    z.create(&fs1, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", fs1));

    assert_eq!(z.exists(&fs1), true);

    let mut snap1 = fs1.clone();
    snap1.push_str("@a");
    z.snapshot([&snap1].iter().cloned()).unwrap();

    let mut snap2 = fs1.clone();
    snap2.push_str("@b");
    z.snapshot([&snap2].iter().cloned()).unwrap();

    z.destroy(&snap2).unwrap();
    z.rollback_to(&fs1, &snap1).unwrap();

    assert_eq!(z.exists(&fs1), true);
    assert_eq!(z.exists(&snap1), true);
    assert_eq!(z.exists(&snap2), false);

    z.destroy(&snap1).unwrap();
    z.destroy(&fs1).unwrap();
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
