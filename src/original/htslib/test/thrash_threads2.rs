use std::ffi::{c_char, c_int};

// original: main (htslib/test/thrash_threads2.c:35)
pub unsafe fn test_thrash_threads2_c_35_main(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
    for i in 0..1000 {
        libc::printf(c"i=%d\n".as_ptr(), i);
        let fp = crate::htslib_mini_rs::bgzf::bgzf_open(c"/dev/null".as_ptr(), c"w".as_ptr());
        crate::htslib_mini_rs::bgzf::bgzf_mt(fp, 8, 256);
        if crate::htslib_mini_rs::bgzf::bgzf_close(fp) != 0 {
            libc::abort();
        }
    }

    0
}
