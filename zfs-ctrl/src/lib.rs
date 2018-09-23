extern crate failure;
#[macro_use] extern crate failure_derive;

use std::path::PathBuf;
use std::env;
use std::ffi::OsStr;
use std::process;
use std::io;

mod zpool;

#[derive(Debug)]
pub struct Zfs {
    zfs_cmd: PathBuf,
}

#[derive(Debug)]
pub enum ListTypes {
    Filesystem,
    Snapshot,
    Volume,
    Bookmark,
    All,
}

#[derive(Debug,Fail)]
pub enum ZfsError {
    #[fail(display = "execution of zfs command failed: {}", io)]
    Exec {
        io: io::Error
    },

    #[fail(display = "zfs command returned an error: {}", status)]
    Process {
        status: process::ExitStatus
    },
}

impl Zfs {
    pub fn list(&self) -> Result<impl Iterator<Item=impl Iterator<Item=&[u8]>>, ZfsError>
    {
        // zfs list -H
        // '-s <prop>' sort by property (multiple allowed)
        // '-d <depth>' recurse to depth
        // '-r' 
        let output = process::Command::new(&self.zfs_cmd)
            .arg("list")
            // +parsable, +scripting mode
            .arg("-pH")
            // only name
            .arg("-o").arg("name")
            .output().map_err(|e| ZfsError::Exec{ io: e})?;

        if !output.status.success() {
            return Err(ZfsError::Process { status: output.status });
        }

        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

        Ok(output.stdout.into_iter().split(|&x| x==b'\n').map(|x| x.split(|&y| y==b'\t')))
    }

    // delete
    //
    // hold
    // release
    //
    // create
    //
    // send
    // recv
    //
    // get (for resume)
}

impl Default for Zfs {
    fn default() -> Self {
        Zfs {
            zfs_cmd: From::from(env::var_os("ZFS_CMD").unwrap_or(OsStr::new("zfs").to_owned())),
        }
    }
}
