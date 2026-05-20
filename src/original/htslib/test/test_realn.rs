use std::ffi::{c_char, c_int};

use crate::htslib_mini_rs::{faidx, hts, sam};

// original: usage (htslib/test/test_realn.c:38)
pub unsafe fn test_test_realn_c_38_usage(prog: *const c_char) {
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"Usage: %s -i <in.sam> -o <out.sam> -f <ref.fa>\n".as_ptr(),
        prog,
    );
}

// original: main (htslib/test/test_realn.c:42)
pub unsafe fn test_test_realn_c_42_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut in_: *mut hts::htsFile = std::ptr::null_mut();
    let mut out: *mut hts::htsFile = std::ptr::null_mut();
    let mut in_name = c"-".as_ptr();
    let mut out_name = c"-".as_ptr();
    let mut ref_name: *mut c_char = std::ptr::null_mut();
    let mut ref_seq: *mut c_char = std::ptr::null_mut();
    let modew = c"w".as_ptr();
    let mut fai: *mut faidx::faidx_t = std::ptr::null_mut();
    let mut hdr: *mut sam::sam_hdr_t = std::ptr::null_mut();
    let mut rec: *mut sam::bam1_t = std::ptr::null_mut();
    let mut res: c_int;
    let mut last_ref = -1;
    let mut ref_len = 0;
    let mut adjust = 0;
    let mut extended = 0;
    let mut recalc = 0;

    let mut exit_status = libc::EXIT_FAILURE;

    loop {
        let c = libc::getopt(argc, argv, c"aef:hi:o:r".as_ptr());
        if c < 0 {
            break;
        }
        match c as u8 {
            b'a' => adjust = 1,
            b'e' => extended = 1,
            b'f' => ref_name = libc::optarg,
            b'h' => {
                test_test_realn_c_38_usage(*argv);
                return libc::EXIT_SUCCESS;
            }
            b'i' => in_name = libc::optarg,
            b'o' => out_name = libc::optarg,
            b'r' => recalc = 1,
            _ => {
                test_test_realn_c_38_usage(*argv);
                return libc::EXIT_FAILURE;
            }
        }
    }

    if ref_name.is_null() {
        test_test_realn_c_38_usage(*argv);
        return libc::EXIT_FAILURE;
    }

    let flags = (if adjust != 0 { 1 } else { 0 })
        | (if extended != 0 { 2 } else { 0 })
        | (if recalc != 0 { 4 } else { 0 });

    'run: {
        fai = faidx::fai_load(ref_name);
        if fai.is_null() {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't load reference %s\n".as_ptr(),
                ref_name,
            );
            break 'run;
        }

        rec = sam::bam_init1();
        if rec.is_null() {
            libc::perror(std::ptr::null());
            break 'run;
        }

        in_ = hts::hts_open(in_name, c"r".as_ptr());
        if in_.is_null() {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't open %s : %s\n".as_ptr(),
                in_name,
                libc::strerror(*libc::__errno_location()),
            );
            break 'run;
        }

        hdr = sam::sam_hdr_read(in_);
        if hdr.is_null() {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't read header for %s\n".as_ptr(),
                in_name,
            );
            break 'run;
        }

        out = hts::hts_open(out_name, modew);
        if out.is_null() {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't open %s : %s\n".as_ptr(),
                out_name,
                libc::strerror(*libc::__errno_location()),
            );
            break 'run;
        }

        if sam::sam_hdr_write(out, hdr) < 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't write header to %s : %s\n".as_ptr(),
                out_name,
                libc::strerror(*libc::__errno_location()),
            );
            break 'run;
        }

        loop {
            res = sam::sam_read1(in_, hdr, rec);
            if res < 0 {
                break;
            }
            if (*rec).core.tid >= (*hdr).n_targets {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Invalid BAM reference id %d\n".as_ptr(),
                    (*rec).core.tid,
                );
                break 'run;
            }
            if last_ref != (*rec).core.tid && (*rec).core.tid >= 0 {
                libc::free(ref_seq.cast());
                ref_seq = faidx::faidx_fetch_seq(
                    fai,
                    *(*hdr).target_name.add((*rec).core.tid as usize),
                    0,
                    c_int::MAX,
                    &mut ref_len,
                );
                if ref_seq.is_null() {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Couldn't get reference %s\n".as_ptr(),
                        *(*hdr).target_name.add((*rec).core.tid as usize),
                    );
                    break 'run;
                }
                last_ref = (*rec).core.tid;
            }
            if (*rec).core.tid >= 0 {
                res = sam::sam_prob_realn(rec, ref_seq, ref_len, flags);
                if res <= -4 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Error running sam_prob_realn : %s\n".as_ptr(),
                        libc::strerror(*libc::__errno_location()),
                    );
                    break 'run;
                }
            }
            if sam::sam_c_4553_sam_write1(out, hdr, rec) < 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Error writing to %s\n".as_ptr(),
                    out_name,
                );
                break 'run;
            }
        }

        res = hts::hts_close(in_);
        in_ = std::ptr::null_mut();
        if res < 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Error closing %s\n".as_ptr(),
                in_name,
            );
            break 'run;
        }

        res = hts::hts_close(out);
        out = std::ptr::null_mut();
        if res < 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Error closing %s\n".as_ptr(),
                out_name,
            );
            break 'run;
        }

        exit_status = libc::EXIT_SUCCESS;
    }

    if !hdr.is_null() {
        sam::sam_hdr_destroy(hdr);
    }
    if !rec.is_null() {
        sam::bam_destroy1(rec);
    }
    if !in_.is_null() {
        hts::hts_close(in_);
    }
    if !out.is_null() {
        hts::hts_close(out);
    }
    libc::free(ref_seq.cast());
    faidx::fai_destroy(fai);

    exit_status
}
