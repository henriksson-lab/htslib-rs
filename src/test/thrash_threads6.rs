use crate::htslib_rs::hts::BGZF;

const N: i64 = 1000;

unsafe fn bgzf_tell(fp: *const BGZF) -> i64 {
    (((*fp).block_address as u64) << 16 | ((*fp).block_offset as u64 & 0xffff)) as i64
}

#[allow(clippy::too_many_arguments)]
unsafe fn run_thrash_threads6(
    input: *const u8,
    pre_reads: usize,
    iterations: usize,
    inner_iterations: usize,
    n_threads: i32,
    sleep_usecs: i64,
    min_tail: u64,
    verbose: bool,
    scripted_ops: Option<&[i32]>,
) -> i32 {
    let mut buf = [0u8; 100000];
    let fpin = crate::htslib_rs::bgzf::bgzf_open(input.cast(), c"r".as_ptr());
    if fpin.is_null() {
        return libc::EXIT_FAILURE;
    }
    let mut upos: u64 = 0;
    let mut uend: u64 = 0;
    for _ in 0..pre_reads {
        let got = crate::htslib_rs::bgzf::bgzf_read(fpin, buf.as_mut_ptr().cast(), 65536);
        if got < 0 {
            libc::abort();
        }
        upos += got as u64;
    }
    let pos = bgzf_tell(fpin);

    loop {
        let got = crate::htslib_rs::bgzf::bgzf_read(fpin, buf.as_mut_ptr().cast(), 65536);
        if got <= 0 {
            if got < 0 {
                libc::abort();
            }
            break;
        }
        uend += got as u64;
    }
    let end = bgzf_tell(fpin);
    crate::htslib_rs::bgzf::bgzf_close(fpin);

    if uend < upos + min_tail {
        eprintln!("Please supply a bigger input file");
        libc::exit(1);
    }

    for i in 0..iterations {
        if verbose {
            print!("i={i}\t");
        }
        let fpin = crate::htslib_rs::bgzf::bgzf_open(input.cast(), c"r".as_ptr());
        if fpin.is_null() {
            return libc::EXIT_FAILURE;
        }
        let mut eof = 0;
        let mut mt = 0;
        for j in 0..inner_iterations {
            let n = scripted_ops
                .and_then(|ops| ops.get(j % ops.len()).copied())
                .unwrap_or_else(|| libc::rand() % 7);
            if verbose {
                print!("{}", (b'0' + n as u8) as char);
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
            match n {
                0 => {
                    if crate::htslib_rs::bgzf::bgzf_seek(fpin, 0, libc::SEEK_SET) < 0 {
                        println!("!");
                    }
                    eof = 0;
                }
                1 => {
                    if crate::htslib_rs::bgzf::bgzf_seek(fpin, pos, libc::SEEK_SET) < 0 {
                        println!("!");
                    }
                    eof = 0;
                }
                2 => {
                    if crate::htslib_rs::bgzf::bgzf_seek(fpin, end, libc::SEEK_SET) < 0 {
                        println!("!");
                    }
                    eof = 1;
                }
                3 | 4 => {
                    let len = if scripted_ops.is_some() {
                        if n == 3 {
                            8192
                        } else {
                            17
                        }
                    } else {
                        (libc::rand() % if n == 3 { 100000 } else { 100 }) as usize
                    };
                    if crate::htslib_rs::bgzf::bgzf_read(
                        fpin,
                        buf.as_mut_ptr().cast(),
                        len,
                    ) != (len as isize) * (1 - eof) as isize
                    {
                        libc::abort();
                    }
                }
                5 => {
                    crate::htslib_rs::hts::hts_usleep(sleep_usecs);
                }
                6 => {
                    if mt == 0 && crate::htslib_rs::bgzf::bgzf_mt(fpin, n_threads, 256) != 0 {
                        crate::htslib_rs::bgzf::bgzf_close(fpin);
                        return libc::EXIT_FAILURE;
                    }
                    mt = 1;
                }
                _ => {}
            }
        }
        if verbose {
            println!();
        }
        if crate::htslib_rs::bgzf::bgzf_close(fpin) != 0 {
            libc::abort();
        }
    }

    0
}

// original: main (htslib/test/thrash_threads6.c:34)
pub unsafe fn test_thrash_threads6_c_34_main(argc: i32, argv: *mut *mut u8) -> i32 {
    if argc <= 1 {
        eprintln!("Usage: thrash_threads4 input.bam");
        libc::exit(1);
    }

    run_thrash_threads6(*argv.add(1), 100, 1000, 80, 8, N, 10_000_000, true, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStrExt;

    fn temp_bgzf_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("htslib-rs-{label}-{}", std::process::id()))
    }

    #[test]
    fn deterministic_thrash_threads6_random_seeks_reads_and_mt() {
        let path = temp_bgzf_path("thrash-threads6.bgz");
        let mut path_c = path.as_os_str().as_bytes().to_vec();
        path_c.push(0);
        let payload = (0..2_000_000).map(|i| (i % 251) as u8).collect::<Vec<_>>();

        unsafe {
            let fp = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr().cast(), c"w".as_ptr());
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

            let ops = [1, 3, 6, 4, 0, 5, 2, 4, 1, 3, 6, 0, 4, 5, 1, 3];
            assert_eq!(
                run_thrash_threads6(path_c.as_ptr(), 2, 2, 16, 2, 0, 0, false, Some(&ops)),
                0
            );
        }

        let _ = std::fs::remove_file(path);
    }
}
