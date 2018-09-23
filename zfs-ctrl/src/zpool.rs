#![allow(dead_code)]

use std::path::PathBuf;
use std::env;
use std::ffi::OsStr;

pub struct Zpool {
    _zpool_cmd: PathBuf, 
}

pub struct PoolProperty {
    _property: String,
    _value: String,
}

impl Zpool {
    // -g   Display vdev, GUIDs
    // -L   Display real paths of vdevs resolving all symbolic links
    // -n   Display config that would be used without adding
    // -P   Display real paths of vdevs instead of last component
    pub fn add(&self, _force: bool, _pool: &str, _vdev: &str) -> Result<(),()>
    {
        unimplemented!();
    }

    pub fn attach(&self, _force: bool, _pool_properties: Vec<PoolProperty>, _pool: &str, _device: &str, _new_device: &str) -> Result<(),()>
    {
        unimplemented!();
    }

    pub fn clear(&self, _pool: &str, _device: Option<&str>) -> Result<(),()>
    {
        unimplemented!();
    }

    pub fn list(&self) -> Result<(),()>
    {
        unimplemented!();
    }
}

impl Default for Zpool {
    fn default() -> Self {
        Zpool {
            _zpool_cmd: From::from(env::var_os("ZPOOL_CMD").unwrap_or(OsStr::new("zpool").to_owned())),
        }
    }
}
