use crate::htslib_rs::{
    hts::{htsFile, hts_close, hts_open, kstring_t},
    sam,
};
unsafe extern "C" {
    static mut optind: i32;
}

// original: ptest_t (htslib/test/pileup.c:56)
#[repr(C)]
pub struct ptest_t {
    pub fname: *const u8,
    pub fp: *mut htsFile,
    pub fp_hdr: *mut sam::sam_hdr_t,
}

// original: readaln (htslib/test/pileup.c:62)
pub unsafe extern "C" fn test_pileup_c_62_readaln(data: *mut (), b: *mut sam::bam1_t) -> i32 {
    let g = data.cast::<ptest_t>();
    let mut ret: i32;

    loop {
        ret = sam::sam_read1((*g).fp, (*g).fp_hdr, b);
        if ret < 0 {
            break;
        }
        if ((*b).core.flag as i32
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
    n: i32,
) -> i32 {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ks = kstring_t::default();
    for _ in 0..n {
        let seq = sam::bam_get_seq((*p).b);
        let is_rev = ((*(*p).b).core.flag as i32 & sam::BAM_FREVERSE) != 0;

        if sam::bam_pileup1_is_head(p) != 0 {
            print!("{}", '^');
            print!("{}", (b'!' + std::cmp::min((*(*p).b).core.qual, 93)) as char);
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
            print!("{}", c as char);
        } else {
            let byte = *seq.add((*p).qpos as usize / 2);
            let base = if ((*p).qpos & 1) == 0 {
                byte >> 4
            } else {
                byte & 0x0f
            };
            let c = SEQ_NT16_STR[base as usize];
            print!("{}", if is_rev {
                (c as char).to_ascii_lowercase()
            } else {
                (c as char).to_ascii_uppercase()
            });
        }

        let mut del_len = -(*p).indel;
        if (*p).indel > 0 {
            let len = sam::bam_plp_insertion(p, &mut ks, &mut del_len);
            if len < 0 {
                eprintln!("bam_plp_insertion");
                return -1;
            }
            print!("{:+}(", len);
            for j in 0..len {
                let c = ks.data[j as usize];
                print!("{}", if is_rev {
                    (c as u8 as char).to_ascii_lowercase()
                } else {
                    (c as u8 as char).to_ascii_uppercase()
                });
            }
            print!("{}", ')');
        }
        if del_len > 0 {
            print!("-{}()", del_len);
        }
        if sam::bam_pileup1_is_tail(p) != 0 {
            print!("{}", '$');
        }
        p = p.add(1);
    }
    0
}

// original: print_pileup_qual (htslib/test/pileup.c:122)
pub unsafe fn test_pileup_c_122_print_pileup_qual(mut p: *const sam::bam_pileup1_t, n: i32) {
    for _ in 0..n {
        let qual = sam::bam_get_qual((*p).b);
        let mut q = b'~';
        if (*p).qpos < (*(*p).b).core.l_qseq && *qual.add((*p).qpos as usize) + 33 < b'~' {
            q = *qual.add((*p).qpos as usize) + 33;
        }
        print!("{}", q as char);
        p = p.add(1);
    }
}

// original: test_pileup (htslib/test/pileup.c:135)
pub unsafe fn test_pileup_c_135_test_pileup(input: *mut ptest_t) -> i32 {
    let mut tid = 0;
    let mut pos = 0;
    let mut n = 0;

    let plp = sam::bam_plp_init(Some(test_pileup_c_62_readaln), input.cast());
    if plp.is_null() {
        eprintln!("bam_plp_init");
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
            eprintln!(
                "bam_plp_auto returned tid {} >= header n_targets {}",
                tid,
                (*(*input).fp_hdr).n_targets,
            );
            sam::bam_plp_destroy(plp);
            return -1;
        }

        let target_name = std::ffi::CStr::from_ptr(
            (*(*(*input).fp_hdr).target_name.add(tid as usize)).cast(),
        )
        .to_bytes();
        print!(
            "{}\t{}\t{}\t",
            String::from_utf8_lossy(target_name),
            pos + 1,
            n,
        );
        if test_pileup_c_76_print_pileup_seq(p, n) < 0 {
            sam::bam_plp_destroy(plp);
            return -1;
        }
        print!("{}", '\t');
        test_pileup_c_122_print_pileup_qual(p, n);
        println!();
    }
    if n < 0 {
        eprintln!(
            "bam_plp_auto failed for \"{}\"",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr((*input).fname.cast()).to_bytes()),
        );
        sam::bam_plp_destroy(plp);
        return -1;
    }

    sam::bam_plp_destroy(plp);
    0
}

// original: test_mpileup (htslib/test/pileup.c:177)
pub unsafe fn test_pileup_c_177_test_mpileup(input: *mut ptest_t) -> i32 {
    let mut data = input.cast::<()>();
    let data_ptr = &mut data as *mut *mut ();
    let mut pileups = [std::ptr::null::<sam::bam_pileup1_t>(); 1];
    let mut n_plp = [0 as i32; 1];
    let mut tid = 0;
    let mut pos = 0;

    let iter = sam::bam_mplp_init(1, Some(test_pileup_c_62_readaln), data_ptr);
    if iter.is_null() {
        eprintln!("bam_plp_init");
        return -1;
    }
    if sam::bam_mplp_init_overlaps(iter) < 0 {
        eprintln!("bam_mplp_init_overlaps");
        sam::bam_mplp_destroy(iter);
        return -1;
    }

    let mut n: i32;
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
            eprintln!(
                "bam_mplp_auto returned tid {} >= header n_targets {}",
                tid,
                (*(*input).fp_hdr).n_targets,
            );
            sam::bam_mplp_destroy(iter);
            return -1;
        }

        let target_name = std::ffi::CStr::from_ptr(
            (*(*(*input).fp_hdr).target_name.add(tid as usize)).cast(),
        )
        .to_bytes();
        print!(
            "{}\t{}\t{}\t",
            String::from_utf8_lossy(target_name),
            pos + 1,
            n_plp[0],
        );
        if test_pileup_c_76_print_pileup_seq(pileups[0], n_plp[0]) < 0 {
            sam::bam_mplp_destroy(iter);
            return -1;
        }
        print!("{}", '\t');
        test_pileup_c_122_print_pileup_qual(pileups[0], n_plp[0]);
        println!();
    }
    if n < 0 {
        eprintln!(
            "bam_plp_auto failed for \"{}\"",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr((*input).fname.cast()).to_bytes()),
        );
        sam::bam_mplp_destroy(iter);
        return -1;
    }

    sam::bam_mplp_destroy(iter);
    0
}

// original: main (htslib/test/pileup.c:225)
pub unsafe fn test_pileup_c_225_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut g = ptest_t {
        fname: std::ptr::null(),
        fp: std::ptr::null_mut(),
        fp_hdr: std::ptr::null_mut(),
    };
    let mut use_mpileup = 0;

    loop {
        let opt = libc::getopt(argc, argv.cast(), c"m".as_ptr());
        if opt == -1 {
            break;
        }
        match opt {
            c if c == b'm' as i32 => use_mpileup = 1,
            _ => {
                eprintln!(
                    "Usage: {} [-m] <sorted.sam>",
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr((*argv).cast()).to_bytes()),
                );
                return libc::EXIT_FAILURE;
            }
        }
    }

    if optind >= argc {
        eprintln!(
            "Usage: {} [-m] <sorted.sam>",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr((*argv).cast()).to_bytes()),
        );
        return libc::EXIT_FAILURE;
    }

    g.fname = *argv.add(optind as usize);
    g.fp = hts_open(g.fname.cast(), c"r".as_ptr());
    if g.fp.is_null() {
        eprint!(
            "Couldn't open \"{}\" : {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(g.fname.cast()).to_bytes()),
            std::io::Error::last_os_error(),
        );
        return libc::EXIT_FAILURE;
    }
    g.fp_hdr = sam::sam_hdr_read(g.fp);
    if g.fp_hdr.is_null() {
        eprint!(
            "Couldn't read header from \"{}\" : {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(g.fname.cast()).to_bytes()),
            std::io::Error::last_os_error(),
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
