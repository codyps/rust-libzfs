fn main() {
    println!("cargo:rustc-link-lib=nvpair");
    // FIXME: a bug exists in some versions of libnvpair causing it to depend on a symbol called
    // `aok`, which is in `libzfs`.
    println!("cargo:rustc-link-lib=zfs");
}
