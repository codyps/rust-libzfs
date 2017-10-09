extern crate pkg_config;

fn main()
{
    pkg_config::probe_library("libzfs_core").unwrap();
    println!("cargo:rustc-link-lib=nvpair");
}
