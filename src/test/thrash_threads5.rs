use std::ffi::{c_char, c_int};
use std::ptr;

const N: usize = 1000;

unsafe fn run_thrash_threads5(
    input: *const c_char,
    n_threads: c_int,
    out_fd: c_int,
    scripted_chunks: Option<&[usize]>,
    verbose: bool,
) -> c_int {
    let mut buf = [0u8; N];
    let mut t = 0isize;

    let fpin = crate::htslib_rs::bgzf::bgzf_open(input, c"r".as_ptr());
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
        .unwrap_or_else(|| (libc::rand() % (N as c_int - 1) + 1) as usize);
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
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"expected %d bytes, got %d\n".as_ptr(),
                    n as c_int,
                    l as c_int,
                );
            }
            break;
        }

        chunk_index += 1;
        n = scripted_chunks
            .and_then(|chunks| chunks.get(chunk_index % chunks.len()).copied())
            .unwrap_or_else(|| (libc::rand() % (N as c_int - 1) + 1) as usize);
    }

    let close_ret = crate::htslib_rs::bgzf::bgzf_close(fpin) as c_int;
    if verbose {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"close=%d\n".as_ptr(),
            close_ret,
        );
    }
    if !p.is_null() {
        crate::htslib_rs::thread_pool::hts_tpool_destroy(p);
    }

    if verbose {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"wrote %d bytes\n".as_ptr(),
            t as c_int,
        );
    }

    0
}

// original: main (htslib/test/thrash_threads5.c:35)
pub unsafe fn test_thrash_threads5_c_35_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    if argc < 2 || libc::isatty(libc::STDOUT_FILENO) != 0 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Usage: thrash_threads5 input.bam num_threads | md5sum\n".as_ptr(),
        );
        libc::exit(1);
    }

    let n_threads = if argc > 2 {
        libc::atoi(*argv.add(2))
    } else {
        0
    };
    run_thrash_threads5(*argv.add(1), n_threads, libc::STDOUT_FILENO, None, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{c_void, CString};
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
        let input_c = CString::new(input.as_os_str().as_bytes()).unwrap();
        let output_c = CString::new(output.as_os_str().as_bytes()).unwrap();
        let payload = (0..8192).map(|i| (i % 251) as u8).collect::<Vec<_>>();

        unsafe {
            let fp = crate::htslib_rs::bgzf::bgzf_open(input_c.as_ptr(), c"w".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(
                crate::htslib_rs::bgzf::bgzf_write(
                    fp,
                    payload.as_ptr().cast::<c_void>(),
                    payload.len()
                ),
                payload.len() as isize
            );
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(fp), 0);

            let out_fd = libc::open(
                output_c.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                0o600,
            );
            assert!(out_fd >= 0);
            let chunks = [843, 104, 691, 17, 999, 251, 64, 512];
            assert_eq!(
                run_thrash_threads5(input_c.as_ptr(), 2, out_fd, Some(&chunks), false),
                0
            );
            assert_eq!(libc::close(out_fd), 0);
        }

        assert_eq!(std::fs::read(&output).unwrap(), payload);
        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }
}
