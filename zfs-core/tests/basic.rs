extern crate zfs_core as zfs;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::io;
use std::io::Seek;
use std::os::unix::io::AsRawFd;

fn have_root_privs() -> bool {
    // this might not be totally accurate
    unsafe { libc::geteuid() == 0 }
}

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
    std::env::var("ZFS_TEMPFS").expect(
        "ZFS_TEMPFS should be set to a zfs filesystem that can be used for temporary datasets",
    )
}

impl TempFs {
    fn with_base(base: &str, prefix: &str) -> io::Result<TempFs> {
        let z = zfs::Zfs::new()?;
        let nv = nvpair::NvList::try_new()?;

        let mut rng = thread_rng();
        for _ in 0..NUM_RETRIES {
            let suffix: String = (&mut rng)
                .sample_iter(Alphanumeric)
                .take(NUM_RAND_CHARS)
                .map(|x| x as char)
                .collect();

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
        Self::with_base(&tmp_zpool_name(), prefix)
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    /*
    pub fn mount(&self) -> TempMount {
        TempMount::new(self)
    }
    */
}

/*
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
*/

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
    let nv = nvpair::NvList::new();
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
    let nv = nvpair::NvList::new();
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
    let nv = nvpair::NvList::new();
    z.create(&b, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", b));

    assert_eq!(z.exists(&b), true);

    let mut b_snap = b.clone();
    b_snap.push_str("@a");
    z.snapshot([b_snap.as_str()].iter().cloned()).unwrap();

    assert_eq!(z.exists(&b), true);
    assert_eq!(z.exists(&b_snap), true);

    z.destroy_snaps([b_snap.as_str()].iter().cloned(), zfs::Defer::No)
        .unwrap();
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
    let nv = nvpair::NvList::new();
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
    z.snapshot([b_snap1.as_str(), b_snap2.as_str()].iter().cloned())
        .unwrap();

    assert_eq!(z.exists(&b), true);
    assert_eq!(z.exists(&b_snap1), true);
    assert_eq!(z.exists(&b_snap2), true);

    z.destroy_snaps(
        [b_snap1.as_str(), b_snap2.as_str()].iter().cloned(),
        zfs::Defer::No,
    )
    .unwrap();
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
    let nv = nvpair::NvList::new();
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
    z.snapshot([b_snap1.as_str(), b_snap2.as_str()].iter().cloned())
        .unwrap();

    assert_eq!(z.exists(&b), true);
    assert_eq!(z.exists(&b_snap1), true);
    assert_eq!(z.exists(&b_snap2), true);

    let mut hold_snaps = nvpair::NvList::new();
    hold_snaps.insert(&b_snap1, "hold-hello").unwrap();
    z.hold_raw(&hold_snaps, None).unwrap();

    z.destroy_snaps(
        [b_snap1.as_str(), b_snap2.as_str()].iter().cloned(),
        zfs::Defer::Yes,
    )
    .unwrap();

    assert_eq!(z.exists(&b_snap1), true);
    assert_eq!(z.exists(&b_snap2), false);

    let mut release_snaps = nvpair::NvList::new();
    let mut holds_for_snap = nvpair::NvList::new();
    holds_for_snap.insert("hold-hello", &()).unwrap();
    release_snaps
        .insert(&b_snap1, holds_for_snap.as_ref())
        .unwrap();
    z.release_raw(&release_snaps).unwrap();

    assert_eq!(z.exists(&b_snap1), false);
    assert_eq!(z.exists(&b_snap2), false);

    z.destroy(&b).unwrap();
}

/// hold: EINVAL: path doesn't look like a snapshot (no `@`)
#[test]
fn hold_not_snap() {
    let tmpfs = TempFs::new("hold_not_snap").unwrap();

    let z = zfs::Zfs::new().unwrap();
    let e = z.hold(
        [(tmpfs.path().to_owned() + "/2", "doesn't look like snapshot")].iter(),
        None,
    );

    let e = if let Err(e) = e {
        e
    } else {
        panic!("expected an error");
    };

    // XXX: macos vs linux zfs difference
    // I would have expected to get a list of errors here instead of a single error, like the macos
    // variant. This might be a bug in our code, or some change in how lzc handles errors for
    // single results
    match e {
        // openzfs 2.0.0:
        zfs_core::Error::Io { source: e } => {
            assert_eq!(e.kind(), io::ErrorKind::InvalidInput);
        }
        // openzfs 1.9.4:
        zfs_core::Error::List { source: el } => {
            let mut hm = std::collections::HashMap::new();
            hm.insert(tmpfs.path().to_owned() + "/2", io::ErrorKind::InvalidInput);

            for (name, error) in el.iter() {
                match hm.remove(name.to_str().unwrap()) {
                    Some(v) => {
                        assert_eq!(error.kind(), v);
                    }
                    None => panic!(),
                }
            }
        }
    }
}

/// hold: NotFound: snapshot named doesn't exist
#[test]
fn hold_not_exist() {
    let tmpfs = TempFs::new("hold_not_exist").unwrap();

    let z = zfs::Zfs::new().unwrap();

    // FIXME: when run with root perms, this does not return an error.
    let e = z.hold(
        [
            (tmpfs.path().to_owned() + "/1@snap", "snap doesn't exist"),
            (tmpfs.path().to_owned() + "/2@snap", "snap doesn't exist"),
        ]
        .iter(),
        None,
    );

    let e = if let Err(e) = e {
        e
    } else {
        // macos hits this for some reason
        /*
        #[cfg(target_os = "macos")]
        {
            eprintln!("macos zfs 1.9.4 is for some reason totally cool with creating holds on non-existent snaps");
            return;
        }
        */

        panic!("expected an error, got {:?}", e);
    };

    // XXX: macos vs linux zfs difference
    // macos for some reason returns `Ok(())`, which is somewhat concerning
    // linux (zfs 2.0.0) doesn't appear to return our error list, which is also concerning
    match e {
        // zfs 2.0.0 on linux:
        zfs_core::Error::Io { source: e } => {
            assert_eq!(e.kind(), io::ErrorKind::NotFound);
        }
        // _expected_ result
        /*
        zfs_core::Error::List { source: el } => {
            let mut hm = std::collections::HashMap::new();
            hm.insert(tmpfs.path().to_owned() + "/1@snap", io::ErrorKind::NotFound);

            for (name, error) in el.iter() {
                match hm.remove(name.to_str().unwrap()) {
                    Some(v) => {
                        assert_eq!(error.kind(), v);
                    }
                    None => panic!()
                }
            }
        }
        */
        _ => {
            panic!("unexpected error kind: {:?}", e);
        }
    }
}

#[test]
fn hold_ok() {
    let tmpfs = TempFs::new("hold_ok").unwrap();
    let z = zfs::Zfs::new().unwrap();

    let props = nvpair::NvList::new();
    z.create(
        tmpfs.path().to_owned() + "/1",
        zfs::DataSetType::Zfs,
        &props,
    )
    .unwrap();
    z.snapshot([tmpfs.path().to_owned() + "/1@snap"].iter().cloned())
        .unwrap();

    z.hold(
        [(tmpfs.path().to_owned() + "/1@snap", "test-hold-ok")].iter(),
        None,
    )
    .unwrap();

    z.release(
        [(
            tmpfs.path().to_owned() + "/1@snap",
            ["test-hold-ok"].iter().cloned(),
        )]
        .iter(),
    )
    .unwrap();
}

#[test]
fn get_holds() {
    let tmpfs = TempFs::new("get_holds").unwrap();
    let z = zfs::Zfs::new().unwrap();

    let props = nvpair::NvList::new();
    z.create(
        tmpfs.path().to_owned() + "/1",
        zfs::DataSetType::Zfs,
        &props,
    )
    .unwrap();
    z.snapshot([tmpfs.path().to_owned() + "/1@snap"].iter().cloned())
        .unwrap();

    z.hold(
        [(tmpfs.path().to_owned() + "/1@snap", "test-hold-ok")].iter(),
        None,
    )
    .unwrap();

    let holds = z.get_holds(tmpfs.path().to_owned() + "/1@snap").unwrap();

    let mut expected_holds = std::collections::HashSet::new();
    expected_holds.insert("test-hold-ok");
    for h in holds.as_ref() {
        let (v, d) = h.tuple();

        if let nvpair::NvData::Uint64(_) = d {
            // ok
        } else {
            panic!("unexpected data for hold {:?}: {:?}", v, d);
        }
        assert_eq!(true, expected_holds.remove(v.to_str().unwrap()));
    }

    z.release(
        [(
            tmpfs.path().to_owned() + "/1@snap",
            ["test-hold-ok"].iter().cloned(),
        )]
        .iter(),
    )
    .unwrap();
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
    let nv = nvpair::NvList::new();
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
    eprintln!("snap1: {}", snap1);
    z.send::<_, &str>(&snap1, None, stream.as_raw_fd(), zfs::SendFlags::default())
        .unwrap();
    stream.seek(io::SeekFrom::Start(0)).unwrap();
    eprintln!("snap2: {}", snap2);
    // FIXME: fails with macos 11.6, zfs 2.1.0
    z.receive::<_, &str>(&snap2, None, None, false, false, stream.as_raw_fd())
        .unwrap();

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
    let nv = nvpair::NvList::new();
    z.create(&fs1, zfs::DataSetType::Zfs, &nv)
        .expect(&format!("create {:?} failed", fs1));

    assert_eq!(z.exists(&fs1), true);

    let props = nvpair::NvList::new();
    let mut snap1 = fs1.clone();

    {
        snap1.push_str("@a");
        let mut snaps = nvpair::NvList::new();
        snaps.insert(&snap1, &()).unwrap();
        z.snapshot_raw(&snaps, &props).unwrap();
    }

    let mut snap2 = fs1.clone();

    {
        snap2.push_str("@b");
        let mut snaps = nvpair::NvList::new();
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
    let nv = nvpair::NvList::new();
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

// WARNING: root perms only
#[test]
fn channel_program_nosync_list() {
    if !have_root_privs() {
        eprintln!("skipping channel_program_nosync_list, need root privs");
        return;
    }

    let tmpfs = TempFs::new("channel_program_nosync_list").unwrap();
    let z = zfs::Zfs::new().unwrap();

    let props = nvpair::NvList::new();
    z.create(
        tmpfs.path().to_owned() + "/1",
        zfs::DataSetType::Zfs,
        &props,
    )
    .unwrap();

    let prgm = r#"
    function collect(...)
      local arr = {}
      local i = 1
      for v in ... do
        arr[i] = v
        i = i + 1
      end
      return arr
    end

    args = ...
    return collect(zfs.list.children(args["x"]))
    "#;

    let mut args_nv = nvpair::NvList::new();
    args_nv.insert("x", tmpfs.path()).unwrap();

    let res = z
        .channel_program_nosync(
            tmp_zpool_name(),
            std::ffi::CString::new(prgm).unwrap().as_ref(),
            0xfffff,
            0xfffff,
            &args_nv,
        )
        .unwrap();

    let mut expected_children = std::collections::HashSet::new();
    expected_children.insert(tmpfs.path().to_owned() + "/1");

    for ch in &res {
        let (v, d) = ch.tuple();
        if let nvpair::NvData::Bool = d {
        } else {
            panic!("unexpected data for {:?}: {:?}", v, d);
        }

        assert_eq!(expected_children.remove(v.to_str().unwrap()), true);
    }
}
