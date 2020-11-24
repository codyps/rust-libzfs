fn main() {
    let mut cfg = ctest::TestGenerator::new();

    cfg.header("libzfs_core.h");

    // TODO: populate via the same code in zfs-core-sys/build.rs
    //cfg.include(env!("LIBZFS_CORE_INCDIR"));
    //
    cfg.generate("../zfs-core-sys/src/lib.rs", "all.rs");
}
