use crate::htslib_rs::{
    hts::{htsFile, hts_close, hts_open, kstring_t},
    sam,
};
use std::ffi::{c_char, c_int, c_void};

unsafe extern "C" {
    static mut optind: c_int;
}

// original: ptest_t (htslib/test/pileup.c:56)
#[repr(C)]
pub struct ptest_t {
    pub fname: *const c_char,
    pub fp: *mut htsFile,
    pub fp_hdr: *mut sam::sam_hdr_t,
}

// original: readaln (htslib/test/pileup.c:62)
pub unsafe extern "C" fn test_pileup_c_62_readaln(data: *mut c_void, b: *mut sam::bam1_t) -> c_int {
    let g = data.cast::<ptest_t>();
    let mut ret: c_int;

    loop {
        ret = sam::sam_read1((*g).fp, (*g).fp_hdr, b);
        if ret < 0 {
            break;
        }
        if ((*b).core.flag as c_int
            & (sam::BAM_FUNMAP | sam::BAM_FSECONDARY | sam::BAM_FQCFAIL | sam::BAM_FDUP))
            != 0
        {
            continue;
        }
        break;
    }

    ret
}

// original: print_pileup_seq (htslib/test/pileup.c:76)
pub unsafe fn test_pileup_c_76_print_pileup_seq(
    mut p: *const sam::bam_pileup1_t,
    n: c_int,
) -> c_int {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ks = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    for _ in 0..n {
        let seq = sam::bam_get_seq((*p).b);
        let is_rev = ((*(*p).b).core.flag as c_int & sam::BAM_FREVERSE) != 0;

        if sam::bam_pileup1_is_head(p) != 0 {
            libc::putchar(b'^' as c_int);
            libc::putchar(b'!' as c_int + std::cmp::min((*(*p).b).core.qual, 93) as c_int);
        }

        if sam::bam_pileup1_is_del(p) != 0 {
            let c = if sam::bam_pileup1_is_refskip(p) != 0 {
                if is_rev {
                    b'<'
                } else {
                    b'>'
                }
            } else {
                b'*'
            };
            libc::putchar(c as c_int);
        } else {
            let byte = *seq.add((*p).qpos as usize / 2);
            let base = if ((*p).qpos & 1) == 0 {
                byte >> 4
            } else {
                byte & 0x0f
            };
            let c = SEQ_NT16_STR[base as usize];
            libc::putchar(if is_rev {
                libc::tolower(c as c_int)
            } else {
                libc::toupper(c as c_int)
            });
        }

        let mut del_len = -(*p).indel;
        if (*p).indel > 0 {
            let len = sam::bam_plp_insertion(p, &mut ks, &mut del_len);
            if len < 0 {
                libc::perror(c"bam_plp_insertion".as_ptr());
                libc::free(ks.s.cast());
                return -1;
            }
            libc::printf(c"%+d(".as_ptr(), len);
            for j in 0..len {
                let c = *ks.s.add(j as usize) as u8;
                libc::putchar(if is_rev {
                    libc::tolower(c as c_int)
                } else {
                    libc::toupper(c as c_int)
                });
            }
            libc::putchar(b')' as c_int);
        }
        if del_len > 0 {
            libc::printf(c"-%d()".as_ptr(), del_len);
        }
        if sam::bam_pileup1_is_tail(p) != 0 {
            libc::putchar(b'$' as c_int);
        }
        p = p.add(1);
    }
    libc::free(ks.s.cast());
    0
}

// original: print_pileup_qual (htslib/test/pileup.c:122)
pub unsafe fn test_pileup_c_122_print_pileup_qual(mut p: *const sam::bam_pileup1_t, n: c_int) {
    for _ in 0..n {
        let qual = sam::bam_get_qual((*p).b);
        let mut q = b'~';
        if (*p).qpos < (*(*p).b).core.l_qseq && *qual.add((*p).qpos as usize) + 33 < b'~' {
            q = *qual.add((*p).qpos as usize) + 33;
        }
        libc::putchar(q as c_int);
        p = p.add(1);
    }
}

// original: test_pileup (htslib/test/pileup.c:135)
pub unsafe fn test_pileup_c_135_test_pileup(input: *mut ptest_t) -> c_int {
    let mut tid = 0;
    let mut pos = 0;
    let mut n = 0;

    let plp = sam::bam_plp_init(Some(test_pileup_c_62_readaln), input.cast());
    if plp.is_null() {
        libc::perror(c"bam_plp_init".as_ptr());
        return -1;
    }
    loop {
        let p = sam::bam_plp_auto(plp, &mut tid, &mut pos, &mut n);
        if p.is_null() {
            break;
        }
        if tid < 0 {
            break;
        }
        if tid >= (*(*input).fp_hdr).n_targets {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"bam_plp_auto returned tid %d >= header n_targets %d\n".as_ptr(),
                tid,
                (*(*input).fp_hdr).n_targets,
            );
            sam::bam_plp_destroy(plp);
            return -1;
        }

        libc::printf(
            c"%s\t%d\t%d\t".as_ptr(),
            *(*(*input).fp_hdr).target_name.add(tid as usize),
            pos + 1,
            n,
        );
        if test_pileup_c_76_print_pileup_seq(p, n) < 0 {
            sam::bam_plp_destroy(plp);
            return -1;
        }
        libc::putchar(b'\t' as c_int);
        test_pileup_c_122_print_pileup_qual(p, n);
        libc::putchar(b'\n' as c_int);
    }
    if n < 0 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"bam_plp_auto failed for \"%s\"\n".as_ptr(),
            (*input).fname,
        );
        sam::bam_plp_destroy(plp);
        return -1;
    }

    sam::bam_plp_destroy(plp);
    0
}

// original: test_mpileup (htslib/test/pileup.c:177)
pub unsafe fn test_pileup_c_177_test_mpileup(input: *mut ptest_t) -> c_int {
    let mut data = input.cast::<c_void>();
    let data_ptr = &mut data as *mut *mut c_void;
    let mut pileups = [std::ptr::null::<sam::bam_pileup1_t>(); 1];
    let mut n_plp = [0 as c_int; 1];
    let mut tid = 0;
    let mut pos = 0;

    let iter = sam::bam_mplp_init(1, Some(test_pileup_c_62_readaln), data_ptr);
    if iter.is_null() {
        libc::perror(c"bam_plp_init".as_ptr());
        return -1;
    }
    if sam::bam_mplp_init_overlaps(iter) < 0 {
        libc::perror(c"bam_mplp_init_overlaps".as_ptr());
        sam::bam_mplp_destroy(iter);
        return -1;
    }

    let mut n: c_int;
    loop {
        n = sam::bam_mplp_auto(
            iter,
            &mut tid,
            &mut pos,
            n_plp.as_mut_ptr(),
            pileups.as_mut_ptr(),
        );
        if n <= 0 {
            break;
        }
        if tid < 0 {
            break;
        }
        if tid >= (*(*input).fp_hdr).n_targets {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"bam_mplp_auto returned tid %d >= header n_targets %d\n".as_ptr(),
                tid,
                (*(*input).fp_hdr).n_targets,
            );
            sam::bam_mplp_destroy(iter);
            return -1;
        }

        libc::printf(
            c"%s\t%d\t%d\t".as_ptr(),
            *(*(*input).fp_hdr).target_name.add(tid as usize),
            pos + 1,
            n_plp[0],
        );
        if test_pileup_c_76_print_pileup_seq(pileups[0], n_plp[0]) < 0 {
            sam::bam_mplp_destroy(iter);
            return -1;
        }
        libc::putchar(b'\t' as c_int);
        test_pileup_c_122_print_pileup_qual(pileups[0], n_plp[0]);
        libc::putchar(b'\n' as c_int);
    }
    if n < 0 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"bam_plp_auto failed for \"%s\"\n".as_ptr(),
            (*input).fname,
        );
        sam::bam_mplp_destroy(iter);
        return -1;
    }

    sam::bam_mplp_destroy(iter);
    0
}

// original: main (htslib/test/pileup.c:225)
pub unsafe fn test_pileup_c_225_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut g = ptest_t {
        fname: std::ptr::null(),
        fp: std::ptr::null_mut(),
        fp_hdr: std::ptr::null_mut(),
    };
    let mut use_mpileup = 0;

    loop {
        let opt = libc::getopt(argc, argv, c"m".as_ptr());
        if opt == -1 {
            break;
        }
        match opt {
            c if c == b'm' as c_int => use_mpileup = 1,
            _ => {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Usage: %s [-m] <sorted.sam>\n".as_ptr(),
                    *argv,
                );
                return libc::EXIT_FAILURE;
            }
        }
    }

    if optind >= argc {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Usage: %s [-m] <sorted.sam>\n".as_ptr(),
            *argv,
        );
        return libc::EXIT_FAILURE;
    }

    g.fname = *argv.add(optind as usize);
    g.fp = hts_open(g.fname, c"r".as_ptr());
    if g.fp.is_null() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Couldn't open \"%s\" : %s".as_ptr(),
            g.fname,
            libc::strerror(*libc::__errno_location()),
        );
        return libc::EXIT_FAILURE;
    }
    g.fp_hdr = sam::sam_hdr_read(g.fp);
    if g.fp_hdr.is_null() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Couldn't read header from \"%s\" : %s".as_ptr(),
            g.fname,
            libc::strerror(*libc::__errno_location()),
        );
        hts_close(g.fp);
        return libc::EXIT_FAILURE;
    }

    let ret = if use_mpileup != 0 {
        test_pileup_c_177_test_mpileup(&mut g)
    } else {
        test_pileup_c_135_test_pileup(&mut g)
    };
    if ret < 0 {
        sam::sam_hdr_destroy(g.fp_hdr);
        hts_close(g.fp);
        return libc::EXIT_FAILURE;
    }

    sam::sam_hdr_destroy(g.fp_hdr);
    hts_close(g.fp);
    libc::EXIT_SUCCESS
}
