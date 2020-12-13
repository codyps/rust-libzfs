//! generate a small snapshot for testing parsing dmu_replay_records
use std::os::unix::io::AsRawFd;

fn main() {
    let mut args = std::env::args().into_iter();
    args.next().expect("no prgm name");
    let snap_to_make = args.next().expect("missing arg");

    let lzc = zfs_core::Zfs::new().expect("could not init zfs");

    let at_pos = snap_to_make.find('@').expect("could not find '@' in snap_to_make");

    let fs_name = &snap_to_make[..at_pos];

    let prop_nv = nvpair::NvList::new();
    lzc.create(fs_name, zfs_core::DataSetType::Zfs, &prop_nv).expect("could not create snap_to_make");

    lzc.snapshot([&snap_to_make[..]].iter().cloned()).expect("snapshot failed");


    let stdout = std::io::stdout();
    let sl = stdout.lock();

    eprintln!("sending on stdout");

    lzc.send::<_, &str>(snap_to_make, None, sl.as_raw_fd(), Default::default()).expect("send failed");

    eprintln!("done");
}
