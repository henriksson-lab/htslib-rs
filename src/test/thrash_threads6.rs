use std::ffi::{c_char, c_int, c_void};

use crate::htslib_rs::hts::BGZF;

const N: i64 = 1000;

unsafe fn bgzf_tell(fp: *const BGZF) -> i64 {
    (((*fp).block_address as u64) << 16 | ((*fp).block_offset as u64 & 0xffff)) as i64
}

// original: main (htslib/test/thrash_threads6.c:34)
pub unsafe fn test_thrash_threads6_c_34_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    if argc <= 1 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Usage: thrash_threads4 input.bam\n".as_ptr(),
        );
        libc::exit(1);
    }

    let mut buf = [0 as c_char; 100000];
    let fpin = crate::htslib_rs::bgzf::bgzf_open(*argv.add(1), c"r".as_ptr());
    let mut upos: u64 = 0;
    let mut uend: u64 = 0;
    for _ in 0..100 {
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

    if uend < upos + 10000000 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Please supply a bigger input file\n".as_ptr(),
        );
        libc::exit(1);
    }

    for i in 0..1000 {
        libc::printf(c"i=%d\t".as_ptr(), i);
        let fpin = crate::htslib_rs::bgzf::bgzf_open(*argv.add(1), c"r".as_ptr());
        let mut eof = 0;
        let mut mt = 0;
        for _ in 0..80 {
            let n = libc::rand() % 7;
            libc::putchar('0' as c_int + n);
            libc::fflush(hts_sys::stdout.cast());
            match n {
                0 => {
                    if crate::htslib_rs::bgzf::bgzf_seek(fpin, 0, libc::SEEK_SET) < 0 {
                        libc::puts(c"!".as_ptr());
                    }
                    eof = 0;
                }
                1 => {
                    if crate::htslib_rs::bgzf::bgzf_seek(fpin, pos, libc::SEEK_SET) < 0 {
                        libc::puts(c"!".as_ptr());
                    }
                    eof = 0;
                }
                2 => {
                    if crate::htslib_rs::bgzf::bgzf_seek(fpin, end, libc::SEEK_SET) < 0 {
                        libc::puts(c"!".as_ptr());
                    }
                    eof = 1;
                }
                3 | 4 => {
                    let len = (libc::rand() % if n == 3 { 100000 } else { 100 }) as usize;
                    if crate::htslib_rs::bgzf::bgzf_read(
                        fpin,
                        buf.as_mut_ptr().cast::<c_void>(),
                        len,
                    ) != (len as isize) * (1 - eof) as isize
                    {
                        libc::abort();
                    }
                }
                5 => {
                    crate::htslib_rs::hts::hts_usleep(N);
                }
                6 => {
                    if mt == 0 {
                        crate::htslib_rs::bgzf::bgzf_mt(fpin, 8, 256);
                    }
                    mt = 1;
                }
                _ => {}
            }
        }
        libc::printf(c"\n".as_ptr());
        if crate::htslib_rs::bgzf::bgzf_close(fpin) != 0 {
            libc::abort();
        }
    }

    0
}
