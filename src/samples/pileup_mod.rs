use std::io::Write;

use crate::htslib_rs::hts::{htsFile, kstring_t};
use crate::htslib_rs::sam;

// original: plpconf (htslib/samples/pileup_mod.c:45)
#[repr(C)]
struct PlpConf {
    inname: Vec<u8>,
    infile: *mut htsFile,
    in_samhdr: *mut sam::sam_hdr_t,
}

// original: print_usage (htslib/samples/pileup_mod.c:38)
pub unsafe fn samples_pileup_mod_c_38_print_usage() {
    eprint!("Usage: pileup_mod infile\nShows the pileup api usage with base modification.\n");
}

// original: plpconstructor (htslib/samples/pileup_mod.c:56)
pub unsafe extern "C" fn samples_pileup_mod_c_56_plpconstructor(
    _data: *mut (),
    b: *const sam::bam1_t,
    cd: *mut sam::bam_pileup_cd,
) -> i32 {
    if cd.is_null() {
        return 1;
    }
    let mut m = sam::hts_base_mod_state_alloc();

    if sam::bam_parse_basemod(&*b, &mut m) == -1 {
        sam::hts_base_mod_state_free(Some(m));
        1
    } else {
        (*cd).p = Box::into_raw(m).cast();
        0
    }
}

// original: plpdestructor (htslib/samples/pileup_mod.c:70)
pub unsafe extern "C" fn samples_pileup_mod_c_70_plpdestructor(
    _data: *mut (),
    _b: *const sam::bam1_t,
    cd: *mut sam::bam_pileup_cd,
) -> i32 {
    if !cd.is_null() && !(*cd).p.is_null() {
        sam::hts_base_mod_state_free(Some(Box::from_raw(
            (*cd).p.cast::<sam::hts_base_mod_state>(),
        )));
        (*cd).p = std::ptr::null_mut();
    }
    0
}

// original: readdata (htslib/samples/pileup_mod.c:82)
pub unsafe extern "C" fn samples_pileup_mod_c_82_readdata(
    data: *mut (),
    b: *mut sam::bam1_t,
) -> i32 {
    let conf = data.cast::<PlpConf>();
    if conf.is_null() || (*conf).infile.is_null() {
        return -2;
    }
    sam::sam_read1((*conf).infile, (*conf).in_samhdr, b)
}

// original: main (htslib/samples/pileup_mod.c:98)
pub unsafe fn samples_pileup_mod_c_98_main(args: Vec<Vec<u8>>) -> i32 {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    const NMODS: usize = 5;
    let mut ret = 1;
    let mut __out = std::io::stdout();
    let mut conf = PlpConf {
        inname: Vec::new(),
        infile: std::ptr::null_mut(),
        in_samhdr: std::ptr::null_mut(),
    };
    let mut insdata = kstring_t { data: Vec::new() };

    if args.len() != 2 {
        samples_pileup_mod_c_38_print_usage();
        return ret;
    }
    conf.inname = args[1].clone();
    // ensure NUL-terminated for raw C-ABI callees expecting *const c_char
    conf.inname.push(0);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
        return ret;
    }
    conf.infile = crate::htslib_rs::hts::hts_open(conf.inname.as_ptr().cast(), c"r".as_ptr());
    if conf.infile.is_null() {
        write!(
            __out,
            "Could not open {}\n",
            String::from_utf8_lossy(&conf.inname[..conf.inname.len() - 1])
        ).unwrap();
        sam::bam_destroy1(bamdata);
        return ret;
    }
    conf.in_samhdr = sam::sam_hdr_read(conf.infile);
    if conf.in_samhdr.is_null() {
        write!(__out, "Failed to read header from file!\n").unwrap();
        crate::htslib_rs::hts::hts_close(conf.infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let plpiter = sam::bam_plp_init(
        Some(samples_pileup_mod_c_82_readdata),
        (&mut conf as *mut PlpConf).cast(),
    );
    if plpiter.is_null() {
        write!(__out, "Failed to initialize pileup data\n").unwrap();
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
        mods = [sam::hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; NMODS];
        write!(__out, "{}\t{}\t", tid + 1, refpos + 1).unwrap();
        for j in 0..depth {
            let p = plp.add(j as usize);
            let mut dellen = 0;
            if sam::bam_pileup1_is_del(p) != 0 || sam::bam_pileup1_is_refskip(p) != 0 {
                write!(__out, "*").unwrap();
                continue;
            }

            let modlen = sam::bam_mods_at_qpos(
                &*(*p).b,
                (*p).qpos,
                &mut *(*p).cd.p.cast::<sam::hts_base_mod_state>(),
                &mut mods,
            );
            if modlen == -1 {
                write!(__out, "Failed to get modifications\n").unwrap();
                sam::sam_hdr_destroy(conf.in_samhdr);
                crate::htslib_rs::hts::hts_close(conf.infile);
                sam::bam_destroy1(bamdata);
                sam::bam_plp_destroy(plpiter);
                crate::htslib_rs::hts::ks_free(&mut insdata);
                return ret;
            }

            let inslen = sam::bam_plp_insertion_mod(p, (*p).cd.p.cast(), &mut insdata, &mut dellen);
            if inslen == -1 {
                write!(__out, "Failed to get insertion status\n").unwrap();
                sam::sam_hdr_destroy(conf.in_samhdr);
                crate::htslib_rs::hts::hts_close(conf.infile);
                sam::bam_destroy1(bamdata);
                sam::bam_plp_destroy(plpiter);
                crate::htslib_rs::hts::ks_free(&mut insdata);
                return ret;
            }

            let seq = sam::bam_get_seq((*p).b);
            let base = SEQ_NT16_STR[sam::bam_seqi(seq, (*p).qpos as usize) as usize];
            let base_char = if sam::bam_pileup1_is_head(p) != 0 || sam::bam_pileup1_is_tail(p) != 0 {
                base.to_ascii_uppercase()
            } else {
                base.to_ascii_lowercase()
            };
            write!(__out, "{}", base_char as char).unwrap();
            if modlen > 0 {
                let strand_char = if mods[0].strand != 0 { b'-' } else { b'+' };
                write!(__out, "{}", strand_char as char).unwrap();
                if mods[0].modified_base != 0 {
                    write!(__out, "{}", (mods[0].modified_base as u8) as char).unwrap();
                }
            }
            if (*p).indel > 0 {
                write!(
                    __out,
                    "+{}{}",
                    (*p).indel,
                    String::from_utf8_lossy(&insdata.data)
                ).unwrap();
                if dellen != 0 {
                    write!(__out, "-{}", dellen).unwrap();
                    for _ in 0..dellen {
                        write!(__out, "?").unwrap();
                    }
                }
            } else if (*p).indel < 0 {
                write!(__out, "{}", (*p).indel).unwrap();
                for _ in 0..(-(*p).indel) {
                    write!(__out, "?").unwrap();
                }
            }
            write!(__out, " ").unwrap();
        }
        write!(__out, "\n").unwrap();
        let _ = __out.flush();
    }

    ret = 0;
    sam::sam_hdr_destroy(conf.in_samhdr);
    crate::htslib_rs::hts::hts_close(conf.infile);
    sam::bam_destroy1(bamdata);
    sam::bam_plp_destroy(plpiter);
    crate::htslib_rs::hts::ks_free(&mut insdata);
    __out.flush().unwrap();
    ret
}
