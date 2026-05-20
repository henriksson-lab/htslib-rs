use std::ffi::{c_char, c_int};

// original: main (htslib/test/thrash_threads1.c:34)
pub unsafe fn test_thrash_threads1_c_34_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    if argc <= 1 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Usage: thrash_threads1 input.bam\n".as_ptr(),
        );
        libc::exit(1);
    }

    for i in 0..10000 {
        libc::printf(c"i=%d\n".as_ptr(), i);
        let fpin = crate::htslib_rs::bgzf::bgzf_open(*argv.add(1), c"r".as_ptr());
        crate::htslib_rs::bgzf::bgzf_mt(fpin, 2, 256);
        if crate::htslib_rs::bgzf::bgzf_close(fpin) < 0 {
            libc::abort();
        }
    }

    0
}
