#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

pub mod bgzf;
pub mod cram;
pub mod errmod;
pub mod faidx;
pub mod hfile;
pub mod hts;
pub mod kfunc;
pub mod md5;
pub mod original;
pub mod original_stubs;
pub mod os_rand;
pub mod probaln;
pub mod regidx;
pub mod region;
pub mod sam;
pub mod tbx;
pub mod thread_pool;
pub mod vcf;

pub(crate) mod c_compat;

use std::path::{Path, PathBuf};

#[cfg(unix)]
pub(crate) fn path_bytes(path: &Path) -> std::borrow::Cow<'_, [u8]> {
    use std::os::unix::ffi::OsStrExt;
    std::borrow::Cow::Borrowed(path.as_os_str().as_bytes())
}

#[cfg(not(unix))]
pub(crate) fn path_bytes(path: &Path) -> std::borrow::Cow<'_, [u8]> {
    std::borrow::Cow::Owned(path.to_string_lossy().into_owned().into_bytes())
}

#[cfg(unix)]
pub(crate) fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStringExt;
    PathBuf::from(std::ffi::OsString::from_vec(bytes.to_vec()))
}

#[cfg(not(unix))]
pub(crate) fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}

pub use cram::*;
pub use errmod::*;
pub use faidx::*;
pub use hfile::*;
pub use hts::*;
pub use kfunc::*;
pub use md5::*;
pub use os_rand::*;
pub use probaln::*;
pub use regidx::*;
pub use region::*;
pub use sam::*;
pub use tbx::*;
pub use thread_pool::*;
pub use vcf::*;
