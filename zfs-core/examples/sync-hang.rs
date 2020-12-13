fn main() {
    let z = zfs_core::Zfs::new().unwrap();

    z.sync("testpool", true).unwrap();
    println!("COMPLETE");
}
