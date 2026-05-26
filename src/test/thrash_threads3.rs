use std::ffi::{c_char, c_int, c_void};

unsafe fn run_thrash_threads3(
    input: *const c_char,
    iterations: usize,
    n_threads: c_int,
    n_sub_blks: c_int,
    verbose: bool,
) -> c_int {
    let mut buf = vec![0_u8; iterations.saturating_mul(10).max(1)];

    for i in 0..iterations {
        if verbose {
            libc::printf(c"i=%d\n".as_ptr(), i as c_int);
        }
        let fpin = crate::htslib_rs::bgzf::bgzf_open(input, c"r".as_ptr());
        if fpin.is_null() {
            return libc::EXIT_FAILURE;
        }
        let len = i * 10;
        if crate::htslib_rs::bgzf::bgzf_read(fpin, buf.as_mut_ptr().cast::<c_void>(), len) < 0 {
            libc::abort();
        }
        if crate::htslib_rs::bgzf::bgzf_mt(fpin, n_threads, n_sub_blks) != 0 {
            crate::htslib_rs::bgzf::bgzf_close(fpin);
            return libc::EXIT_FAILURE;
        }
        if crate::htslib_rs::bgzf::bgzf_read(fpin, buf.as_mut_ptr().cast::<c_void>(), len) < 0 {
            libc::abort();
        }
        if crate::htslib_rs::bgzf::bgzf_close(fpin) < 0 {
            libc::abort();
        }
    }

    0
}

// original: main (htslib/test/thrash_threads3.c:33)
pub unsafe fn test_thrash_threads3_c_33_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    if argc <= 1 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Usage: thrash_threads3 input.bam\n".as_ptr(),
        );
        libc::exit(1);
    }

    run_thrash_threads3(*argv.add(1), 10000, 8, 256, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    fn temp_bgzf_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("htslib-rs-{label}-{}", std::process::id()))
    }

    #[test]
    fn deterministic_thrash_threads3_reads_before_and_after_mt() {
        let path = temp_bgzf_path("thrash-threads3.bgz");
        let path_c = CString::new(path.as_os_str().as_bytes()).unwrap();

        unsafe {
            let fp = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!fp.is_null());
            let data = b"bounded thrash_threads3 input\n".repeat(32);
            assert_eq!(
                crate::htslib_rs::bgzf::bgzf_write(fp, data.as_ptr().cast::<c_void>(), data.len()),
                data.len() as isize
            );
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(fp), 0);

            assert_eq!(run_thrash_threads3(path_c.as_ptr(), 6, 2, 64, false), 0);
        }

        let _ = std::fs::remove_file(path);
    }
}
