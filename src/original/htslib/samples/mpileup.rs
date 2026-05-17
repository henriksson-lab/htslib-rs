use std::ffi::{c_char, c_int, c_void};

use crate::htslib_mini_rs::hts::{htsFile, hts_pos_t};
use crate::htslib_mini_rs::sam;

// original: plpconf (htslib/samples/mpileup.c:45)
#[repr(C)]
struct PlpConf {
    inname: *mut c_char,
    infile: *mut htsFile,
    in_samhdr: *mut sam::sam_hdr_t,
}

// original: print_usage (htslib/samples/mpileup.c:38)
pub unsafe fn samples_mpileup_c_38_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: mpileup infile ...\nShows the mpileup api usage.\n".as_ptr(),
    );
}

// original: plpconstructor (htslib/samples/mpileup.c:56)
pub unsafe extern "C" fn samples_mpileup_c_56_plpconstructor(
    _data: *mut c_void,
    _b: *const sam::bam1_t,
    _cd: *mut sam::bam_pileup_cd,
) -> c_int {
    0
}

// original: plpdestructor (htslib/samples/mpileup.c:60)
pub unsafe extern "C" fn samples_mpileup_c_60_plpdestructor(
    _data: *mut c_void,
    _b: *const sam::bam1_t,
    _cd: *mut sam::bam_pileup_cd,
) -> c_int {
    0
}

// original: readdata (htslib/samples/mpileup.c:68)
pub unsafe extern "C" fn samples_mpileup_c_68_readdata(
    data: *mut c_void,
    b: *mut sam::bam1_t,
) -> c_int {
    let conf = data.cast::<PlpConf>();
    if conf.is_null() || (*conf).infile.is_null() {
        return -2;
    }
    sam::sam_read1((*conf).infile, (*conf).in_samhdr, b)
}

// original: main (htslib/samples/mpileup.c:84)
pub unsafe fn samples_mpileup_c_84_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ret = libc::EXIT_FAILURE;
    let n_input = argc - 1;
    let mut bamdata: *mut sam::bam1_t = std::ptr::null_mut();
    let mut mplpiter: sam::bam_mplp_t = std::ptr::null_mut();

    if argc < 2 {
        samples_mpileup_c_38_print_usage(hts_sys::stderr.cast());
        return ret;
    }

    let conf: *mut *mut PlpConf = crate::htslib_mini_rs::c_compat::calloc(
        n_input as u64,
        std::mem::size_of::<*mut PlpConf>() as u64,
    )
    .cast();
    if !conf.is_null() {
        for input in 0..n_input {
            *conf.add(input as usize) =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<PlpConf>() as u64)
                    .cast();
        }
    }
    let depth: *mut c_int = crate::htslib_mini_rs::c_compat::calloc(
        n_input as u64,
        std::mem::size_of::<c_int>() as u64,
    )
    .cast();
    let plp: *mut *const sam::bam_pileup1_t = crate::htslib_mini_rs::c_compat::calloc(
        n_input as u64,
        std::mem::size_of::<*const sam::bam_pileup1_t>() as u64,
    )
    .cast();
    if conf.is_null() || depth.is_null() || plp.is_null() {
        libc::printf(c"Failed to allocate memory\n".as_ptr());
        goto_mpileup_cleanup(n_input, conf, depth, plp, bamdata, mplpiter);
        return ret;
    }

    for input in 0..n_input {
        let slot = *conf.add(input as usize);
        if slot.is_null() {
            libc::printf(c"Failed to allocate memory\n".as_ptr());
            goto_mpileup_cleanup(n_input, conf, depth, plp, bamdata, mplpiter);
            return ret;
        }
        (*slot).inname = *argv.add((input + 1) as usize);
    }

    bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to initialize bamdata\n".as_ptr());
        goto_mpileup_cleanup(n_input, conf, depth, plp, bamdata, mplpiter);
        return ret;
    }

    for input in 0..n_input {
        let slot = *conf.add(input as usize);
        (*slot).infile = crate::htslib_mini_rs::hts::hts_open((*slot).inname, c"r".as_ptr());
        if (*slot).infile.is_null() {
            libc::printf(c"Could not open %s\n".as_ptr(), (*slot).inname);
            goto_mpileup_cleanup(n_input, conf, depth, plp, bamdata, mplpiter);
            return ret;
        }
        (*slot).in_samhdr = sam::sam_hdr_read((*slot).infile);
        if (*slot).in_samhdr.is_null() {
            libc::printf(c"Failed to read header from file!\n".as_ptr());
            goto_mpileup_cleanup(n_input, conf, depth, plp, bamdata, mplpiter);
            return ret;
        }
    }

    mplpiter = sam::bam_mplp_init(
        n_input,
        Some(samples_mpileup_c_68_readdata),
        conf.cast::<*mut c_void>(),
    );
    if mplpiter.is_null() {
        libc::printf(c"Failed to initialize mpileup data\n".as_ptr());
        goto_mpileup_cleanup(n_input, conf, depth, plp, bamdata, mplpiter);
        return ret;
    }

    sam::bam_mplp_constructor(mplpiter, Some(samples_mpileup_c_56_plpconstructor));
    sam::bam_mplp_destructor(mplpiter, Some(samples_mpileup_c_60_plpdestructor));

    let mut tid = -1;
    let mut refpos: hts_pos_t = -1;
    while sam::bam_mplp64_auto(mplpiter, &mut tid, &mut refpos, depth, plp) > 0 {
        libc::printf(c"%d\t%ld\t".as_ptr(), tid + 1, refpos as libc::c_long + 1);
        for input in 0..n_input {
            let plp_input = *plp.add(input as usize);
            for dpt in 0..*depth.add(input as usize) {
                let p = plp_input.add(dpt as usize);
                if sam::bam_pileup1_is_del(p) != 0 || sam::bam_pileup1_is_refskip(p) != 0 {
                    libc::printf(c"*".as_ptr());
                    continue;
                }
                let seq = sam::bam_get_seq((*p).b);
                let base = SEQ_NT16_STR[sam::bam_seqi(seq, (*p).qpos as usize) as usize] as c_int;
                let out = if sam::bam_pileup1_is_head(p) != 0
                    || sam::bam_pileup1_is_tail(plp_input) != 0
                {
                    libc::toupper(base)
                } else {
                    libc::tolower(base)
                };
                libc::printf(c"%c".as_ptr(), out);
                if (*p).indel > 0 {
                    libc::printf(c"+%d".as_ptr(), (*p).indel);
                    for k in 0..(*p).indel {
                        let ins_base = SEQ_NT16_STR
                            [sam::bam_seqi(seq, ((*p).qpos + k + 1) as usize) as usize]
                            as c_int;
                        libc::printf(c"%c".as_ptr(), libc::tolower(ins_base));
                    }
                } else if (*p).indel < 0 {
                    libc::printf(c"%d".as_ptr(), (*p).indel);
                    for _ in 0..(-(*p).indel) {
                        libc::printf(c"?".as_ptr());
                    }
                }
            }
            libc::printf(c" ".as_ptr());
        }
        libc::printf(c"\n".as_ptr());
        libc::fflush(hts_sys::stdout.cast());
    }

    ret = libc::EXIT_SUCCESS;
    goto_mpileup_cleanup(n_input, conf, depth, plp, bamdata, mplpiter);
    ret
}

unsafe fn goto_mpileup_cleanup(
    n_input: c_int,
    conf: *mut *mut PlpConf,
    depth: *mut c_int,
    plp: *mut *const sam::bam_pileup1_t,
    bamdata: *mut sam::bam1_t,
    mplpiter: sam::bam_mplp_t,
) {
    if !conf.is_null() {
        for input in 0..n_input {
            let slot = *conf.add(input as usize);
            if !slot.is_null() {
                if !(*slot).in_samhdr.is_null() {
                    sam::sam_hdr_destroy((*slot).in_samhdr);
                }
                if !(*slot).infile.is_null() {
                    crate::htslib_mini_rs::hts::hts_close((*slot).infile);
                }
                crate::htslib_mini_rs::c_compat::free(slot.cast());
            }
        }
        crate::htslib_mini_rs::c_compat::free(conf.cast());
    }
    if !bamdata.is_null() {
        sam::bam_destroy1(bamdata);
    }
    if !mplpiter.is_null() {
        sam::bam_mplp_destroy(mplpiter);
    }
    crate::htslib_mini_rs::c_compat::free(depth.cast());
    crate::htslib_mini_rs::c_compat::free(plp.cast());
}
