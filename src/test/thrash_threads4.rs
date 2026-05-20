use std::ffi::{c_char, c_int, c_void};

// original: main (htslib/test/thrash_threads4.c:34)
pub unsafe fn test_thrash_threads4_c_34_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    if argc <= 1 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Usage: thrash_threads4 input.bam\n".as_ptr(),
        );
        libc::exit(1);
    }

    let mut fpin = crate::htslib_rs::bgzf::bgzf_open(*argv.add(1), c"r".as_ptr());
    let mut buf = [0_u8; 65536];
    for _ in 0..1000 {
        if crate::htslib_rs::bgzf::bgzf_read(fpin, buf.as_mut_ptr().cast::<c_void>(), buf.len()) < 0
        {
            libc::abort();
        }
    }
    let pos = ((*fpin).block_address << 16) | ((*fpin).block_offset as i64 & 0xffff);
    crate::htslib_rs::bgzf::bgzf_close(fpin);

    const N: i64 = 1000;

    for i in 0..1000 {
        libc::printf(c"i=%d\n".as_ptr(), i);
        fpin = crate::htslib_rs::bgzf::bgzf_open(*argv.add(1), c"r".as_ptr());
        crate::htslib_rs::bgzf::bgzf_mt(fpin, 8, 256);
        if crate::htslib_rs::bgzf::bgzf_seek(fpin, pos, libc::SEEK_SET) < 0 {
            libc::puts(c"!".as_ptr());
        }
        crate::htslib_rs::hts::hts_usleep(N);
        if crate::htslib_rs::bgzf::bgzf_seek(fpin, 0, libc::SEEK_SET) < 0 {
            libc::puts(c"!".as_ptr());
        }
        crate::htslib_rs::hts::hts_usleep(N);
        if crate::htslib_rs::bgzf::bgzf_close(fpin) != 0 {
            libc::abort();
        }
    }

    0
}
