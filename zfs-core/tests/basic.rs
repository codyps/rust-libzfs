extern crate zfs_core as zfs;
extern crate nvpair;

fn test_fsname_base(extra: &str) -> String
{
    let mut b = std::env::var("ZFS_TEST_BASE").expect("Set ZFS_TEST_BASE to a suitable zfs fspath to run tests on");
    b.push_str(module_path!());
    b.push_str(extra);
    b
}

#[test]
fn new() {
    let _ = zfs::Zfs::new().unwrap();
}

#[test]
fn create() {
    let b = test_fsname_base("create");

    let z = zfs::Zfs::new().unwrap();
    let nv = nvpair::NvList::new().unwrap();
    z.create(b, zfs::DataSetType::Zfs, &nv).unwrap();

    // XXX: libzfs_core lacks a destroy call at the moment (ZFS_IOC_DESTROY)
}
