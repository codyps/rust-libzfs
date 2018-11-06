#[cfg(not(target_os = "macos"))]
fn main() {
    println!("cargo:rustc-link-lib=nvpair");
}

#[cfg(target_os = "macos")]
fn main() {}
