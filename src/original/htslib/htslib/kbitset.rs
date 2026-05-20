use std::ffi::{c_int, c_ulong};

// original: kbitset_t (htslib/htslib/kbitset.h:63)
#[repr(C)]
pub struct kbitset_t {
    pub n: usize,
    pub n_max: usize,
    pub b: [c_ulong; 1],
}

// original: kbitset_iter_t (htslib/htslib/kbitset.h:167)
#[repr(C)]
pub struct kbitset_iter_t {
    pub mask: c_ulong,
    pub elt: usize,
    pub i: c_int,
}
