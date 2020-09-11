extern crate pkg_config;

fn main() {
    if cfg!(target_os = "freebsd") || cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=zfs_core");
    } else {
        pkg_config::probe_library("libzfs_core").unwrap();
    }

    println!("cargo:rustc-link-lib=nvpair");
}
