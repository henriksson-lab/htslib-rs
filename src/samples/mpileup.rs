use crate::htslib_rs::hts::{htsFile, hts_pos_t};
use crate::htslib_rs::sam;
use std::io::Write;

// original: plpconf (htslib/samples/mpileup.c:45)
struct PlpConf {
    inname: Vec<u8>,
    infile: *mut htsFile,
    in_samhdr: *mut sam::sam_hdr_t,
}

// original: print_usage (htslib/samples/mpileup.c:38)
pub fn samples_mpileup_c_38_print_usage() {
    eprint!("Usage: mpileup infile ...\nShows the mpileup api usage.\n");
}

// original: plpconstructor (htslib/samples/mpileup.c:56)
// boundary: extern "C" callback for bam_mplp_constructor; opaque args kept as *mut ()
pub unsafe extern "C" fn samples_mpileup_c_56_plpconstructor(
    _data: *mut (),
    _b: *const sam::bam1_t,
    _cd: *mut sam::bam_pileup_cd,
) -> i32 {
    0
}

// original: plpdestructor (htslib/samples/mpileup.c:60)
// boundary: extern "C" callback for bam_mplp_destructor; opaque args kept as *mut ()
pub unsafe extern "C" fn samples_mpileup_c_60_plpdestructor(
    _data: *mut (),
    _b: *const sam::bam1_t,
    _cd: *mut sam::bam_pileup_cd,
) -> i32 {
    0
}

// original: readdata (htslib/samples/mpileup.c:68)
// boundary: extern "C" callback for bam_mplp_init; data is opaque *mut () (a *mut PlpConf slot)
pub unsafe extern "C" fn samples_mpileup_c_68_readdata(
    data: *mut (),
    b: *mut sam::bam1_t,
) -> i32 {
    let conf = data.cast::<PlpConf>();
    if conf.is_null() || (*conf).infile.is_null() {
        return -2;
    }
    sam::sam_read1((*conf).infile, (*conf).in_samhdr, b)
}

// original: main (htslib/samples/mpileup.c:84)
pub unsafe fn samples_mpileup_c_84_main(argc: i32, argv: *mut *mut u8) -> i32 {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ret = 1;
    let mut __out = std::io::stdout();
    let n_input = argc - 1;
    let mut bamdata: *mut sam::bam1_t = std::ptr::null_mut();
    let mut mplpiter: sam::bam_mplp_t = std::ptr::null_mut();

    if argc < 2 {
        samples_mpileup_c_38_print_usage();
        return ret;
    }

    // Owned per-input configuration. The mpileup readdata callback receives a
    // pointer to one of these slots as opaque data.
    let mut conf: Vec<PlpConf> = Vec::with_capacity(n_input as usize);
    let mut depth: Vec<i32> = vec![0; n_input as usize];
    let mut plp: Vec<*const sam::bam_pileup1_t> = vec![std::ptr::null(); n_input as usize];

    for input in 0..n_input {
        // inname is a raw C-string from argv; copy its bytes including the NUL
        // terminator so it can be handed back to the raw-ptr hts_open API.
        let raw = *argv.add((input + 1) as usize);
        let mut len = 0usize;
        while *raw.add(len) != 0 {
            len += 1;
        }
        let inname = std::slice::from_raw_parts(raw, len + 1).to_vec();
        conf.push(PlpConf {
            inname,
            infile: std::ptr::null_mut(),
            in_samhdr: std::ptr::null_mut(),
        });
    }

    bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
        __out.flush().unwrap();
        goto_mpileup_cleanup(&mut conf, bamdata, mplpiter);
        return ret;
    }

    for slot in conf.iter_mut() {
        slot.infile =
            crate::htslib_rs::hts::hts_open(slot.inname.as_ptr().cast(), c"r".as_ptr());
        if slot.infile.is_null() {
            write!(
                __out,
                "Could not open {}\n",
                String::from_utf8_lossy(&slot.inname[..slot.inname.len() - 1])
            ).unwrap();
            __out.flush().unwrap();
            goto_mpileup_cleanup(&mut conf, bamdata, mplpiter);
            return ret;
        }
        slot.in_samhdr = sam::sam_hdr_read(slot.infile);
        if slot.in_samhdr.is_null() {
            write!(__out, "Failed to read header from file!\n").unwrap();
            __out.flush().unwrap();
            goto_mpileup_cleanup(&mut conf, bamdata, mplpiter);
            return ret;
        }
    }

    mplpiter = sam::bam_mplp_init(
        n_input,
        Some(samples_mpileup_c_68_readdata),
        conf.as_mut_ptr().cast(),
    );
    if mplpiter.is_null() {
        write!(__out, "Failed to initialize mpileup data\n").unwrap();
        __out.flush().unwrap();
        goto_mpileup_cleanup(&mut conf, bamdata, mplpiter);
        return ret;
    }

    sam::bam_mplp_constructor(mplpiter, Some(samples_mpileup_c_56_plpconstructor));
    sam::bam_mplp_destructor(mplpiter, Some(samples_mpileup_c_60_plpdestructor));

    let mut tid = -1;
    let mut refpos: hts_pos_t = -1;
    while sam::bam_mplp64_auto(
        mplpiter,
        &mut tid,
        &mut refpos,
        depth.as_mut_ptr(),
        plp.as_mut_ptr(),
    ) > 0
    {
        write!(__out, "{}\t{}\t", tid + 1, refpos + 1).unwrap();
        for input in 0..n_input as usize {
            let plp_input = plp[input];
            for dpt in 0..depth[input] {
                let p = plp_input.add(dpt as usize);
                if sam::bam_pileup1_is_del(p) != 0 || sam::bam_pileup1_is_refskip(p) != 0 {
                    write!(__out, "*").unwrap();
                    continue;
                }
                let seq = sam::bam_get_seq((*p).b);
                let base = SEQ_NT16_STR[sam::bam_seqi(seq, (*p).qpos as usize) as usize];
                let out = if sam::bam_pileup1_is_head(p) != 0
                    || sam::bam_pileup1_is_tail(plp_input) != 0
                {
                    base.to_ascii_uppercase()
                } else {
                    base.to_ascii_lowercase()
                };
                write!(__out, "{}", out as char).unwrap();
                if (*p).indel > 0 {
                    write!(__out, "+{}", (*p).indel).unwrap();
                    for k in 0..(*p).indel {
                        let ins_base = SEQ_NT16_STR
                            [sam::bam_seqi(seq, ((*p).qpos + k + 1) as usize) as usize];
                        write!(__out, "{}", ins_base.to_ascii_lowercase() as char).unwrap();
                    }
                } else if (*p).indel < 0 {
                    write!(__out, "{}", (*p).indel).unwrap();
                    for _ in 0..(-(*p).indel) {
                        write!(__out, "?").unwrap();
                    }
                }
            }
            write!(__out, " ").unwrap();
        }
        write!(__out, "\n").unwrap();
        __out.flush().unwrap();
    }

    ret = 0;
    goto_mpileup_cleanup(&mut conf, bamdata, mplpiter);
    __out.flush().unwrap();
    ret
}

unsafe fn goto_mpileup_cleanup(
    conf: &mut Vec<PlpConf>,
    bamdata: *mut sam::bam1_t,
    mplpiter: sam::bam_mplp_t,
) {
    for slot in conf.iter() {
        if !slot.in_samhdr.is_null() {
            sam::sam_hdr_destroy(slot.in_samhdr);
        }
        if !slot.infile.is_null() {
            crate::htslib_rs::hts::hts_close(slot.infile);
        }
    }
    // The PlpConf slots (and their inname buffers) are owned Rust values; they
    // are dropped when `conf` goes out of scope.
    conf.clear();
    if !bamdata.is_null() {
        sam::bam_destroy1(bamdata);
    }
    if !mplpiter.is_null() {
        sam::bam_mplp_destroy(mplpiter);
    }
    // depth and plp are owned Vecs in the caller and freed on scope exit.
}
