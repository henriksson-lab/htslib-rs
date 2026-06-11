// original: kbitset_t (htslib/htslib/kbitset.h:63)
#[repr(C)]
pub struct kbitset_t {
    pub n: usize,
    pub n_max: usize,
    pub b: [u64; 1],
}

// original: kbitset_iter_t (htslib/htslib/kbitset.h:167)
#[repr(C)]
pub struct kbitset_iter_t {
    pub mask: u64,
    pub elt: usize,
    pub i: i32,
}
