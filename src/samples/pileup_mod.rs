use std::ffi::{c_char, c_int, c_void};

use crate::htslib_rs::hts::{htsFile, kstring_t};
use crate::htslib_rs::sam;

// original: plpconf (htslib/samples/pileup_mod.c:45)
#[repr(C)]
struct PlpConf {
    inname: *mut c_char,
    infile: *mut htsFile,
    in_samhdr: *mut sam::sam_hdr_t,
}

// original: print_usage (htslib/samples/pileup_mod.c:38)
pub unsafe fn samples_pileup_mod_c_38_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: pileup_mod infile\nShows the pileup api usage with base modification.\n".as_ptr(),
    );
}

// original: plpconstructor (htslib/samples/pileup_mod.c:56)
pub unsafe extern "C" fn samples_pileup_mod_c_56_plpconstructor(
    _data: *mut c_void,
    b: *const sam::bam1_t,
    cd: *mut sam::bam_pileup_cd,
) -> c_int {
    if cd.is_null() {
        return 1;
    }
    (*cd).p = sam::hts_base_mod_state_alloc().cast();
    if (*cd).p.is_null() {
        libc::printf(c"Failed to allocate base modification state\n".as_ptr());
        return 1;
    }

    if sam::bam_parse_basemod(b, (*cd).p.cast()) == -1 {
        1
    } else {
        0
    }
}

// original: plpdestructor (htslib/samples/pileup_mod.c:70)
pub unsafe extern "C" fn samples_pileup_mod_c_70_plpdestructor(
    _data: *mut c_void,
    _b: *const sam::bam1_t,
    cd: *mut sam::bam_pileup_cd,
) -> c_int {
    if !cd.is_null() && !(*cd).p.is_null() {
        sam::hts_base_mod_state_free((*cd).p.cast());
        (*cd).p = std::ptr::null_mut();
    }
    0
}

// original: readdata (htslib/samples/pileup_mod.c:82)
pub unsafe extern "C" fn samples_pileup_mod_c_82_readdata(
    data: *mut c_void,
    b: *mut sam::bam1_t,
) -> c_int {
    let conf = data.cast::<PlpConf>();
    if conf.is_null() || (*conf).infile.is_null() {
        return -2;
    }
    sam::sam_read1((*conf).infile, (*conf).in_samhdr, b)
}

// original: main (htslib/samples/pileup_mod.c:98)
pub unsafe fn samples_pileup_mod_c_98_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    const NMODS: usize = 5;
    let mut ret = libc::EXIT_FAILURE;
    let mut conf = PlpConf {
        inname: std::ptr::null_mut(),
        infile: std::ptr::null_mut(),
        in_samhdr: std::ptr::null_mut(),
    };
    let mut insdata = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };

    if argc != 2 {
        samples_pileup_mod_c_38_print_usage(hts_sys::stderr.cast());
        return ret;
    }
    conf.inname = *argv.add(1);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        libc::printf(c"Failed to initialize bamdata\n".as_ptr());
        return ret;
    }
    conf.infile = crate::htslib_rs::hts::hts_open(conf.inname, c"r".as_ptr());
    if conf.infile.is_null() {
        libc::printf(c"Could not open %s\n".as_ptr(), conf.inname);
        sam::bam_destroy1(bamdata);
        return ret;
    }
    conf.in_samhdr = sam::sam_hdr_read(conf.infile);
    if conf.in_samhdr.is_null() {
        libc::printf(c"Failed to read header from file!\n".as_ptr());
        crate::htslib_rs::hts::hts_close(conf.infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let plpiter = sam::bam_plp_init(
        Some(samples_pileup_mod_c_82_readdata),
        (&mut conf as *mut PlpConf).cast(),
    );
    if plpiter.is_null() {
        libc::printf(c"Failed to initialize pileup data\n".as_ptr());
        sam::sam_hdr_destroy(conf.in_samhdr);
        crate::htslib_rs::hts::hts_close(conf.infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    sam::bam_plp_constructor(plpiter, Some(samples_pileup_mod_c_56_plpconstructor));
    sam::bam_plp_destructor(plpiter, Some(samples_pileup_mod_c_70_plpdestructor));

    let mut tid = -1;
    let mut refpos = -1;
    let mut depth = -1;
    let mut mods = [sam::hts_base_mod {
        modified_base: 0,
        canonical_base: 0,
        strand: 0,
        qual: 0,
    }; NMODS];
    loop {
        let plp = sam::bam_plp_auto(plpiter, &mut tid, &mut refpos, &mut depth);
        if plp.is_null() {
            break;
        }
        libc::memset(mods.as_mut_ptr().cast(), 0, std::mem::size_of_val(&mods));
        libc::printf(c"%d\t%d\t".as_ptr(), tid + 1, refpos + 1);
        for j in 0..depth {
            let p = plp.add(j as usize);
            let mut dellen = 0;
            if sam::bam_pileup1_is_del(p) != 0 || sam::bam_pileup1_is_refskip(p) != 0 {
                libc::printf(c"*".as_ptr());
                continue;
            }

            let modlen = sam::bam_mods_at_qpos(
                (*p).b,
                (*p).qpos,
                (*p).cd.p.cast(),
                mods.as_mut_ptr(),
                NMODS as c_int,
            );
            if modlen == -1 {
                libc::printf(c"Failed to get modifications\n".as_ptr());
                sam::sam_hdr_destroy(conf.in_samhdr);
                crate::htslib_rs::hts::hts_close(conf.infile);
                sam::bam_destroy1(bamdata);
                sam::bam_plp_destroy(plpiter);
                crate::htslib_rs::hts::ks_free(&mut insdata);
                return ret;
            }

            let inslen = sam::bam_plp_insertion_mod(p, (*p).cd.p.cast(), &mut insdata, &mut dellen);
            if inslen == -1 {
                libc::printf(c"Failed to get insertion status\n".as_ptr());
                sam::sam_hdr_destroy(conf.in_samhdr);
                crate::htslib_rs::hts::hts_close(conf.infile);
                sam::bam_destroy1(bamdata);
                sam::bam_plp_destroy(plpiter);
                crate::htslib_rs::hts::ks_free(&mut insdata);
                return ret;
            }

            let seq = sam::bam_get_seq((*p).b);
            let base = SEQ_NT16_STR[sam::bam_seqi(seq, (*p).qpos as usize) as usize] as c_int;
            libc::printf(
                c"%c%c%c".as_ptr(),
                if sam::bam_pileup1_is_head(p) != 0 || sam::bam_pileup1_is_tail(p) != 0 {
                    libc::toupper(base)
                } else {
                    libc::tolower(base)
                },
                if modlen > 0 {
                    (if mods[0].strand != 0 { b'-' } else { b'+' }) as c_int
                } else {
                    0
                },
                if modlen > 0 { mods[0].modified_base } else { 0 },
            );
            if (*p).indel > 0 {
                libc::printf(c"+%d%s".as_ptr(), (*p).indel, insdata.s);
                if dellen != 0 {
                    libc::printf(c"-%d".as_ptr(), dellen);
                    for _ in 0..dellen {
                        libc::printf(c"?".as_ptr());
                    }
                }
            } else if (*p).indel < 0 {
                libc::printf(c"%d".as_ptr(), (*p).indel);
                for _ in 0..(-(*p).indel) {
                    libc::printf(c"?".as_ptr());
                }
            }
            libc::printf(c" ".as_ptr());
        }
        libc::printf(c"\n".as_ptr());
        libc::fflush(hts_sys::stdout.cast());
    }

    ret = libc::EXIT_SUCCESS;
    sam::sam_hdr_destroy(conf.in_samhdr);
    crate::htslib_rs::hts::hts_close(conf.infile);
    sam::bam_destroy1(bamdata);
    sam::bam_plp_destroy(plpiter);
    crate::htslib_rs::hts::ks_free(&mut insdata);
    ret
}
