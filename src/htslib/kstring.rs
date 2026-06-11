// original: ks_tokaux_t (htslib/htslib/kstring.h:86)
#[repr(C)]
pub struct ks_tokaux_t {
    pub tab: [u64; 4],
    pub sep: i32,
    pub finished: i32,
    pub p: *const i8,
}
