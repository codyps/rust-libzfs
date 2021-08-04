fn var(s: &str) -> Result<String, std::env::VarError> {
    println!("cargo:rerun-if-env-changed={}", s);
    std::env::var(s)
}

fn main() {
    let target_os = var("CARGO_CFG_TARGET_OS").expect("Could not get env var CARGO_CFG_TARGET_OS");

    // when using "openzfs on macos", zfs libs are installed outside the default link path. Add it
    // in
    // TODO: Provide a way to disable
    if target_os == "macos" {
        println!("cargo:rustc-link-search=native=/usr/local/zfs/lib");
    }

    println!("cargo:rustc-link-lib=nvpair");
    // FIXME: a bug exists in some versions of libnvpair causing it to depend on a symbol called
    // `aok`, which is in `libzfs`.
    println!("cargo:rustc-link-lib=zfs");
    // nvpair uses functions from libspl on FreeBSD
    if target_os == "freebsd" {
        println!("cargo:rustc-link-lib=spl");
    };
}
