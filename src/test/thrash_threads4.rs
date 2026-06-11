unsafe fn run_thrash_threads4(
    input: *const u8,
    pre_reads: usize,
    iterations: usize,
    n_threads: i32,
    sleep_usecs: i64,
    verbose: bool,
) -> i32 {
    let mut fpin = crate::htslib_rs::bgzf::bgzf_open(input.cast(), c"r".as_ptr());
    if fpin.is_null() {
        return libc::EXIT_FAILURE;
    }

    let mut buf = [0_u8; 65536];
    for _ in 0..pre_reads {
        if crate::htslib_rs::bgzf::bgzf_read(fpin, buf.as_mut_ptr().cast(), buf.len()) < 0
        {
            libc::abort();
        }
    }
    let pos = ((*fpin).block_address << 16) | ((*fpin).block_offset as i64 & 0xffff);
    if crate::htslib_rs::bgzf::bgzf_close(fpin) != 0 {
        libc::abort();
    }

    for i in 0..iterations {
        if verbose {
            eprintln!("i={i}");
        }
        fpin = crate::htslib_rs::bgzf::bgzf_open(input.cast(), c"r".as_ptr());
        if fpin.is_null() {
            return libc::EXIT_FAILURE;
        }
        if crate::htslib_rs::bgzf::bgzf_mt(fpin, n_threads, 256) != 0 {
            crate::htslib_rs::bgzf::bgzf_close(fpin);
            return libc::EXIT_FAILURE;
        }
        if crate::htslib_rs::bgzf::bgzf_seek(fpin, pos, libc::SEEK_SET) < 0 {
            println!("!");
        }
        crate::htslib_rs::hts::hts_usleep(sleep_usecs);
        if crate::htslib_rs::bgzf::bgzf_seek(fpin, 0, libc::SEEK_SET) < 0 {
            println!("!");
        }
        crate::htslib_rs::hts::hts_usleep(sleep_usecs);
        if crate::htslib_rs::bgzf::bgzf_close(fpin) != 0 {
            libc::abort();
        }
    }

    0
}

// original: main (htslib/test/thrash_threads4.c:34)
pub unsafe fn test_thrash_threads4_c_34_main(argc: i32, argv: *mut *mut u8) -> i32 {
    if argc <= 1 {
        eprintln!("Usage: thrash_threads4 input.bam");
        libc::exit(1);
    }

    run_thrash_threads4(*argv.add(1), 1000, 1000, 8, 1000, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStrExt;

    fn temp_bgzf_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("htslib-rs-{label}-{}", std::process::id()))
    }

    #[test]
    fn deterministic_thrash_threads4_reopens_and_seeks_threaded_reader() {
        let path = temp_bgzf_path("thrash-threads4.bgz");
        let mut path_c = path.as_os_str().as_bytes().to_vec();
        path_c.push(0);

        unsafe {
            let fp = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr().cast(), c"w".as_ptr());
            assert!(!fp.is_null());
            let data = vec![b'A'; 70_000];
            assert_eq!(
                crate::htslib_rs::bgzf::bgzf_write(fp, data.as_ptr().cast::<libc::c_void>(), data.len()),
                data.len() as isize
            );
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(fp), 0);

            assert_eq!(run_thrash_threads4(path_c.as_ptr().cast(), 1, 3, 2, 0, false), 0);
        }

        let _ = std::fs::remove_file(path);
    }
}
