use std::ffi::{c_char, c_int, c_void};

// original: main (htslib/test/thrash_threads3.c:33)
pub unsafe fn test_thrash_threads3_c_33_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut buf = [0_u8; 1_000_000];

    if argc <= 1 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Usage: thrash_threads3 input.bam\n".as_ptr(),
        );
        libc::exit(1);
    }

    for i in 0..10000 {
        libc::printf(c"i=%d\n".as_ptr(), i);
        let fpin = crate::htslib_rs::bgzf::bgzf_open(*argv.add(1), c"r".as_ptr());
        let len = (i * 10) as usize;
        if crate::htslib_rs::bgzf::bgzf_read(fpin, buf.as_mut_ptr().cast::<c_void>(), len) < 0 {
            libc::abort();
        }
        crate::htslib_rs::bgzf::bgzf_mt(fpin, 8, 256);
        if crate::htslib_rs::bgzf::bgzf_read(fpin, buf.as_mut_ptr().cast::<c_void>(), len) < 0 {
            libc::abort();
        }
        if crate::htslib_rs::bgzf::bgzf_close(fpin) < 0 {
            libc::abort();
        }
    }

    0
}
