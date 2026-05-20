use std::ffi::{c_char, c_int};

// original: ks_tokaux_t (htslib/htslib/kstring.h:86)
#[repr(C)]
pub struct ks_tokaux_t {
    pub tab: [u64; 4],
    pub sep: c_int,
    pub finished: c_int,
    pub p: *const c_char,
}
