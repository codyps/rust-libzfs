extern crate libzfs_core as zfs;

#[test]
fn new() {
    let _ = zfs::Zfs::new();
}
