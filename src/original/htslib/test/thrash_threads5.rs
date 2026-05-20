use std::ffi::{c_char, c_int};
use std::ptr;

const N: usize = 1000;

// original: main (htslib/test/thrash_threads5.c:35)
pub unsafe fn test_thrash_threads5_c_35_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut buf = [0u8; N];
    let mut t = 0isize;

    if argc < 2 || libc::isatty(libc::STDOUT_FILENO) != 0 {
        libc::fprintf(
            hts_sys::stderr,
            c"Usage: thrash_threads5 input.bam num_threads | md5sum\n".as_ptr(),
        );
        libc::exit(1);
    }

    let fpin = crate::htslib_mini_rs::bgzf::bgzf_open(*argv.add(1), c"r".as_ptr());
    let p = if argc > 2 {
        let p = crate::htslib_mini_rs::thread_pool::hts_tpool_init(libc::atoi(*argv.add(2)));
        crate::htslib_mini_rs::bgzf::bgzf_thread_pool(fpin, p, 0);
        p
    } else {
        ptr::null_mut()
    };

    let mut n = (libc::rand() % (N as c_int - 1) + 1) as usize;
    loop {
        let l = crate::htslib_mini_rs::bgzf::bgzf_read(fpin, buf.as_mut_ptr().cast(), n);
        if l <= 0 {
            break;
        }

        if l != libc::write(libc::STDOUT_FILENO, buf.as_ptr().cast(), l as usize) {
            libc::abort();
        }
        t += l;

        if l != n as isize {
            libc::fprintf(
                hts_sys::stderr,
                c"expected %d bytes, got %d\n".as_ptr(),
                n as c_int,
                l as c_int,
            );
            break;
        }

        n = (libc::rand() % (N as c_int - 1) + 1) as usize;
    }

    libc::fprintf(
        hts_sys::stderr,
        c"close=%d\n".as_ptr(),
        crate::htslib_mini_rs::bgzf::bgzf_close(fpin) as c_int,
    );
    if !p.is_null() {
        crate::htslib_mini_rs::thread_pool::hts_tpool_destroy(p);
    }

    libc::fprintf(hts_sys::stderr, c"wrote %d bytes\n".as_ptr(), t as c_int);

    0
}
