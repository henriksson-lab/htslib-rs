use crate::htslib_rs::hts::htsFile;
use crate::htslib_rs::sam;
use std::io::Write;

// original: plpconf (htslib/samples/pileup.c:45)
#[repr(C)]
struct PlpConf {
    inname: Vec<u8>,
    infile: *mut htsFile,
    in_samhdr: *mut sam::sam_hdr_t,
}

// original: print_usage (htslib/samples/pileup.c:38)
pub unsafe fn samples_pileup_c_38_print_usage() {
    eprint!("Usage: pileup infile\nShows the pileup api usage.\n");
}

// original: plpconstructor (htslib/samples/pileup.c:56)
pub unsafe extern "C" fn samples_pileup_c_56_plpconstructor(
    _data: *mut (),
    _b: *const sam::bam1_t,
    _cd: *mut sam::bam_pileup_cd,
) -> i32 {
    0
}

// original: plpdestructor (htslib/samples/pileup.c:65)
pub unsafe extern "C" fn samples_pileup_c_65_plpdestructor(
    _data: *mut (),
    _b: *const sam::bam1_t,
    _cd: *mut sam::bam_pileup_cd,
) -> i32 {
    0
}

// original: readdata (htslib/samples/pileup.c:76)
pub unsafe extern "C" fn samples_pileup_c_76_readdata(
    data: *mut (),
    b: *mut sam::bam1_t,
) -> i32 {
    let conf = data.cast::<PlpConf>();
    if conf.is_null() || (*conf).infile.is_null() {
        return -2;
    }
    sam::sam_read1((*conf).infile, (*conf).in_samhdr, b)
}

// original: main (htslib/samples/pileup.c:92)
pub unsafe fn samples_pileup_c_92_main(args: &[&[u8]]) -> i32 {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ret = 1;
    let mut __out = std::io::stdout();
    let mut conf = PlpConf {
        inname: Vec::new(),
        infile: std::ptr::null_mut(),
        in_samhdr: std::ptr::null_mut(),
    };

    if args.len() != 2 {
        samples_pileup_c_38_print_usage();
        return ret;
    }
    // NUL-terminated copy for the still-raw C-ABI hts_open boundary
    conf.inname = args[1].to_vec();
    conf.inname.push(0);

    let bamdata = sam::bam_init1();
    if bamdata.is_null() {
        write!(__out, "Failed to initialize bamdata\n").unwrap();
        __out.flush().unwrap();
        return ret;
    }
    conf.infile = crate::htslib_rs::hts::hts_open(
        conf.inname.as_ptr().cast(),
        b"r\0".as_ptr().cast(),
    );
    if conf.infile.is_null() {
        write!(
            __out,
            "Could not open {}\n",
            String::from_utf8_lossy(&conf.inname[..conf.inname.len() - 1])
        ).unwrap();
        __out.flush().unwrap();
        sam::bam_destroy1(bamdata);
        return ret;
    }
    conf.in_samhdr = sam::sam_hdr_read(conf.infile);
    if conf.in_samhdr.is_null() {
        write!(__out, "Failed to read header from file!\n").unwrap();
        __out.flush().unwrap();
        crate::htslib_rs::hts::hts_close(conf.infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    let plpiter = sam::bam_plp_init(
        Some(samples_pileup_c_76_readdata),
        (&mut conf as *mut PlpConf).cast(),
    );
    if plpiter.is_null() {
        write!(__out, "Failed to initialize pileup data\n").unwrap();
        __out.flush().unwrap();
        sam::sam_hdr_destroy(conf.in_samhdr);
        crate::htslib_rs::hts::hts_close(conf.infile);
        sam::bam_destroy1(bamdata);
        return ret;
    }

    sam::bam_plp_constructor(plpiter, Some(samples_pileup_c_56_plpconstructor));
    sam::bam_plp_destructor(plpiter, Some(samples_pileup_c_65_plpdestructor));

    let mut tid = -1;
    let mut refpos = -1;
    let mut n = -1;
    loop {
        let plp = sam::bam_plp_auto(plpiter, &mut tid, &mut refpos, &mut n);
        if plp.is_null() {
            break;
        }
        write!(__out, "{}\t{}\t", tid + 1, refpos + 1).unwrap();
        for j in 0..n {
            let p = plp.add(j as usize);
            if sam::bam_pileup1_is_del(p) != 0 || sam::bam_pileup1_is_refskip(p) != 0 {
                write!(__out, "*").unwrap();
                continue;
            }
            let seq = sam::bam_get_seq((*p).b);
            let base = SEQ_NT16_STR[sam::bam_seqi(seq, (*p).qpos as usize) as usize];
            let out = if sam::bam_pileup1_is_head(p) != 0 || sam::bam_pileup1_is_tail(p) != 0 {
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
            write!(__out, " ").unwrap();
        }
        write!(__out, "\n").unwrap();
        __out.flush().unwrap();
    }

    ret = 0;
    sam::sam_hdr_destroy(conf.in_samhdr);
    crate::htslib_rs::hts::hts_close(conf.infile);
    sam::bam_destroy1(bamdata);
    sam::bam_plp_destroy(plpiter);
    __out.flush().unwrap();
    ret
}
