unsafe fn run_thrash_threads2(iterations: usize, n_threads: i32, verbose: bool) -> i32 {
    for i in 0..iterations {
        if verbose {
            eprintln!("i={}", i);
        }
        let fp = crate::htslib_rs::bgzf::bgzf_open(c"/dev/null".as_ptr(), c"w".as_ptr());
        if crate::htslib_rs::bgzf::bgzf_mt(fp, n_threads, 256) != 0 {
            crate::htslib_rs::bgzf::bgzf_close(fp);
            return libc::EXIT_FAILURE;
        }
        if crate::htslib_rs::bgzf::bgzf_close(fp) != 0 {
            libc::abort();
        }
    }

    0
}

// original: main (htslib/test/thrash_threads2.c:35)
pub unsafe fn test_thrash_threads2_c_35_main(_argc: i32, _argv: *mut *mut u8) -> i32 {
    run_thrash_threads2(1000, 8, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_thrash_threads2_opens_multithreaded_writers() {
        unsafe {
            assert_eq!(run_thrash_threads2(4, 1, false), 0);
        }
    }
}
