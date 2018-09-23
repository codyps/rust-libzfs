extern crate zfs_ctrl;

#[test]
fn zfs() {
    let _ = zfs_ctrl::Zfs::default();
}

#[test]
fn zfs_list() {
    let zfs = zfs_ctrl::Zfs::default();

    println!("{:?}", String::from_utf8_lossy(&zfs.list().expect("list failed")));
}
