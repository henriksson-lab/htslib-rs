#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

pub mod annot_tsv;
pub mod bgzf;
pub mod bgzip;
#[path = "cram.rs"]
pub mod cram;
pub mod errmod;
pub mod faidx;
pub mod hfile;
#[cfg(feature = "gcs")]
pub mod hfile_gcs;
#[cfg(feature = "libcurl")]
pub mod hfile_libcurl;
#[cfg(feature = "s3")]
pub mod hfile_s3;
pub mod hts;
pub mod hts_os;
pub mod htscodecs;
pub mod htsfile;
pub mod htslib;
pub mod kfunc;
pub mod kstring;
pub mod md5;
pub mod multipart;
pub mod os_rand;
pub mod plugin;
pub mod probaln;
pub mod realn;
pub mod ref_cache;
pub mod regidx;
pub mod region;
pub mod sam;
pub mod samples;
pub mod simd;
pub mod tabix;
pub mod tbx;
pub mod test;
pub mod textutils;
pub mod thread_pool;
pub mod vcf;
pub mod vcfutils;

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

// The hts_fmt_option constants below are declared with identical values in
// both `hts` and `cram` (verified bit-for-bit). To prevent the
// `ambiguous_glob_reexports` lint and to make `hts` the unambiguous canonical
// source, we explicitly re-export them from `hts` before the globs.  An
// explicit re-export shadows subsequent glob re-exports for the lint, so the
// globs no longer race.
pub use hts::{
    CRAM_OPT_BASES_PER_SLICE, CRAM_OPT_DECODE_MD, CRAM_OPT_EMBED_REF, CRAM_OPT_IGNORE_MD5,
    CRAM_OPT_LOSSY_NAMES, CRAM_OPT_MULTI_SEQ_PER_SLICE, CRAM_OPT_NO_REF, CRAM_OPT_NTHREADS,
    CRAM_OPT_PREFIX, CRAM_OPT_REFERENCE, CRAM_OPT_REQUIRED_FIELDS, CRAM_OPT_SEQS_PER_SLICE,
    CRAM_OPT_SLICES_PER_CONTAINER, CRAM_OPT_STORE_MD, CRAM_OPT_STORE_NM, CRAM_OPT_THREAD_POOL,
    CRAM_OPT_USE_BZIP2, CRAM_OPT_USE_LZMA, CRAM_OPT_USE_RANS, CRAM_OPT_VERBOSITY,
    CRAM_OPT_VERSION, HTS_OPT_COMPRESSION_LEVEL,
};

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
