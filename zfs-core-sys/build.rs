use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    str::FromStr,
};

fn var(s: &str) -> Result<String, std::env::VarError> {
    println!("cargo:rerun-if-env-changed={}", s);
    std::env::var(s)
}

#[derive(Debug)]
enum Lookup {
    PkgConfig,
    Link,
}

#[derive(Debug)]
struct LookupParseErr;

impl FromStr for Lookup {
    type Err = LookupParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pkg-config" => Ok(Lookup::PkgConfig),
            "link" => Ok(Lookup::Link),
            _ => Err(LookupParseErr),
        }
    }
}

fn env_var_append<V: AsRef<OsStr>>(key: &str, value: V) {
    let value = value.as_ref();
    let mut v = if let Some(v) = std::env::var_os(value) {
        v
    } else {
        std::env::set_var(key, value);
        return;
    };

    if v.is_empty() {
        std::env::set_var(key, value);
        return;
    }

    v.push(":");
    v.push(value);
    std::env::set_var(key, v);
}

fn main() {
    // openzfs on osx: fixed paths, under /usr/local/zfs (has pkg-config for libzfs_core)

    let target_os = var("CARGO_CFG_TARGET_OS").expect("Could not get env var CARGO_CFG_TARGET_OS");
    let mut build_env = build_env::BuildEnv::from_env().expect("Could not determine build_env");

    let lzc_libdir = build_env.var("LIBZFS_CORE_LIBDIR");
    let lzc_lookup = if let Some(_) = lzc_libdir.as_ref() {
        // Implies users want `LIBZFS_CORE_LOOKUP_WITH=link`
        Lookup::Link
    } else {
        let lookup_with = build_env.var("LIBZFS_CORE_LOOKUP_WITH");
        let lookup_with: Option<Lookup> = lookup_with.map(|v| v.to_str().unwrap().parse().unwrap());

        lookup_with.unwrap_or_else(|| match target_os.as_str() {
            // users have reported that this is required for freebsd. I have not tested it.
            "freebsd" => Lookup::Link,

            // openzfs on osx has the `libzfs_core.pc` file, installed into
            // `/usr/local/zfs/lib/pkgconfig`. Users _must_ ensure this is part of their
            // `PKG_CONFIG_PATH`. Note that when cross compiling, this may cause some difficulty,
            // because the `pkg-config` crate doesn't allow distinguishing right now. We could
            // workaround this by hacking up std::env ourselves, or ideally the pkg-config crate would
            // use a build-env style lookup to pick the right `PKG_CONFIG_PATH` itself.
            //
            // Right now, if the link method is _not_ supplied, we tweak PKG_CONFIG_PATH so things
            // will automatically work in the common case (with openzfs on osx 2.01 at least)
            //
            // This will almost certainly behave poorly in the case of cross compilation, where
            // users should probably specify a `LIBZFS_CORE_LOOKUP_WITH` explicitly.
            "macos" => {
                let pc_path = PathBuf::from_str("/usr/local/zfs/lib/pkgconfig").unwrap();
                if pc_path.exists() {
                    env_var_append("PKG_CONFIG_PATH", pc_path);
                }
                Lookup::PkgConfig
            }
            //
            // zfs on linux: use pkg-config for libzfs_core (no pc for nvpair)
            // default to true otherwise.
            _ => Lookup::PkgConfig,
        })
    };

    match lzc_lookup {
        Lookup::PkgConfig => {
            pkg_config::probe_library("libzfs_core").unwrap();
        }
        Lookup::Link => {
            if let Some(v) = lzc_libdir {
                println!("cargo:rustc-link-search=native={}", v.to_str().unwrap());
            }
            println!("cargo:rustc-link-lib=native=zfs_core");
        }
    }

    // there is currently no nvpair pkg-config, so unconditionally link
    if target_os == "macos" {
        // TODO: this is an openzfs on osx specific path. Provide a way to disable
        println!("cargo:rustc-link-search=native=/usr/local/zfs/lib");
    }
    println!("cargo:rustc-link-lib=nvpair");
    match target_os.as_str() {
        "freebsd" => {
            println!("cargo:rustc-link-lib=dylib:-as-needed=zutil");
        }
        _ => {}
    }
}
