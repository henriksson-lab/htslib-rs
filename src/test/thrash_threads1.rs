unsafe fn run_thrash_threads1(
    input: *const u8,
    iterations: usize,
    n_threads: i32,
    verbose: bool,
) -> i32 {
    for i in 0..iterations {
        if verbose {
            eprintln!("i={}", i);
        }
        let fpin = crate::htslib_rs::bgzf::bgzf_open(input.cast(), b"r\0".as_ptr().cast());
        if fpin.is_null() {
            return libc::EXIT_FAILURE;
        }
        if crate::htslib_rs::bgzf::bgzf_mt(fpin, n_threads, 256) != 0 {
            crate::htslib_rs::bgzf::bgzf_close(fpin);
            return libc::EXIT_FAILURE;
        }
        if crate::htslib_rs::bgzf::bgzf_close(fpin) < 0 {
            libc::abort();
        }
    }

    0
}

// original: main (htslib/test/thrash_threads1.c:34)
pub unsafe fn test_thrash_threads1_c_34_main(argc: i32, argv: *mut *mut u8) -> i32 {
    if argc <= 1 {
        eprintln!("Usage: thrash_threads1 input.bam");
        libc::exit(1);
    }

    run_thrash_threads1(*argv.add(1), 10000, 2, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStrExt;

    fn temp_bgzf_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("htslib-rs-{label}-{}", std::process::id()))
    }

    #[test]
    fn deterministic_thrash_threads1_reopens_bgzf_reader() {
        let path = temp_bgzf_path("thrash-threads1.bgz");
        let mut path_c = path.as_os_str().as_bytes().to_vec();
        path_c.push(0);

        unsafe {
            let fp = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr().cast(), b"w\0".as_ptr().cast());
            assert!(!fp.is_null());
            let data = b"bounded thrash_threads1 input\n";
            assert_eq!(
                crate::htslib_rs::bgzf::bgzf_write(fp, data.as_ptr().cast::<()>(), data.len()),
                data.len() as isize
            );
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(fp), 0);

            assert_eq!(run_thrash_threads1(path_c.as_ptr().cast(), 4, 1, false), 0);
        }

        let _ = std::fs::remove_file(path);
    }
}
