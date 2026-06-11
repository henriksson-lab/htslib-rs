use std::ptr;

const N: usize = 1000;

unsafe fn run_thrash_threads5(
    input: *const u8,
    n_threads: i32,
    out_fd: i32,
    scripted_chunks: Option<&[usize]>,
    verbose: bool,
) -> i32 {
    let mut buf = [0u8; N];
    let mut t = 0isize;

    let fpin = crate::htslib_rs::bgzf::bgzf_open(input.cast(), b"r\0".as_ptr().cast());
    if fpin.is_null() {
        return libc::EXIT_FAILURE;
    }
    let p = if n_threads > 0 {
        let p = crate::htslib_rs::thread_pool::hts_tpool_init(n_threads);
        if p.is_null() {
            crate::htslib_rs::bgzf::bgzf_close(fpin);
            return libc::EXIT_FAILURE;
        }
        if crate::htslib_rs::bgzf::bgzf_thread_pool(fpin, p, 0) != 0 {
            crate::htslib_rs::bgzf::bgzf_close(fpin);
            crate::htslib_rs::thread_pool::hts_tpool_destroy(p);
            return libc::EXIT_FAILURE;
        }
        p
    } else {
        ptr::null_mut()
    };

    let mut chunk_index = 0usize;
    let mut n = scripted_chunks
        .and_then(|chunks| chunks.first().copied())
        .unwrap_or_else(|| (libc::rand() % (N as i32 - 1) + 1) as usize);
    loop {
        let l = crate::htslib_rs::bgzf::bgzf_read(fpin, buf.as_mut_ptr().cast(), n);
        if l <= 0 {
            break;
        }

        if l != libc::write(out_fd, buf.as_ptr().cast(), l as usize) {
            libc::abort();
        }
        t += l;

        if l != n as isize {
            if verbose {
                eprintln!("expected {} bytes, got {}", n as i32, l as i32);
            }
            break;
        }

        chunk_index += 1;
        n = scripted_chunks
            .and_then(|chunks| chunks.get(chunk_index % chunks.len()).copied())
            .unwrap_or_else(|| (libc::rand() % (N as i32 - 1) + 1) as usize);
    }

    let close_ret = crate::htslib_rs::bgzf::bgzf_close(fpin) as i32;
    if verbose {
        eprintln!("close={}", close_ret);
    }
    if !p.is_null() {
        crate::htslib_rs::thread_pool::hts_tpool_destroy(p);
    }

    if verbose {
        eprintln!("wrote {} bytes", t as i32);
    }

    0
}

// original: main (htslib/test/thrash_threads5.c:35)
pub unsafe fn test_thrash_threads5_c_35_main(argc: i32, argv: *mut *mut u8) -> i32 {
    if argc < 2 || libc::isatty(libc::STDOUT_FILENO) != 0 {
        eprintln!("Usage: thrash_threads5 input.bam num_threads | md5sum");
        libc::exit(1);
    }

    let n_threads = if argc > 2 {
        libc::atoi((*argv.add(2)).cast())
    } else {
        0
    };
    run_thrash_threads5(*argv.add(1), n_threads, libc::STDOUT_FILENO, None, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStrExt;

    fn temp_path(label: &str, suffix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "htslib-rs-{label}-{}{}",
            std::process::id(),
            suffix
        ))
    }

    #[test]
    fn deterministic_thrash_threads5_streams_all_data_with_thread_pool() {
        let input = temp_path("thrash-threads5-input", ".bgz");
        let output = temp_path("thrash-threads5-output", ".bin");
        let mut input_c = input.as_os_str().as_bytes().to_vec();
        input_c.push(0);
        let mut output_c = output.as_os_str().as_bytes().to_vec();
        output_c.push(0);
        let payload = (0..8192).map(|i| (i % 251) as u8).collect::<Vec<_>>();

        unsafe {
            let fp = crate::htslib_rs::bgzf::bgzf_open(input_c.as_ptr().cast(), b"w\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(
                crate::htslib_rs::bgzf::bgzf_write(
                    fp,
                    payload.as_ptr().cast(),
                    payload.len()
                ),
                payload.len() as isize
            );
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(fp), 0);

            let out_fd = libc::open(
                output_c.as_ptr().cast(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                0o600,
            );
            assert!(out_fd >= 0);
            let chunks = [843, 104, 691, 17, 999, 251, 64, 512];
            assert_eq!(
                run_thrash_threads5(input_c.as_ptr().cast(), 2, out_fd, Some(&chunks), false),
                0
            );
            assert_eq!(libc::close(out_fd), 0);
        }

        assert_eq!(std::fs::read(&output).unwrap(), payload);
        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }
}
