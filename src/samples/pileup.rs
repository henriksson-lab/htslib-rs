use std::ffi::{c_char, c_int, c_void};

use crate::htslib_rs::hts::htsFile;
use crate::htslib_rs::sam;

// original: plpconf (htslib/samples/pileup.c:45)
#[repr(C)]
struct PlpConf {
    inname: *mut c_char,
    infile: *mut htsFile,
    in_samhdr: *mut sam::sam_hdr_t,
}

// original: print_usage (htslib/samples/pileup.c:38)
pub unsafe fn samples_pileup_c_38_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: pileup infile\nShows the pileup api usage.\n".as_ptr(),
    );
}

// original: plpconstructor (htslib/samples/pileup.c:56)
pub unsafe extern "C" fn samples_pileup_c_56_plpconstructor(
    _data: *mut c_void,
    _b: *const sam::bam1_t,
    _cd: *mut sam::bam_pileup_cd,
) -> c_int {
    0
}

// original: plpdestructor (htslib/samples/pileup.c:65)
pub unsafe extern "C" fn samples_pileup_c_65_plpdestructor(
    _data: *mut c_void,
    _b: *const sam::bam1_t,
    _cd: *mut sam::bam_pileup_cd,
) -> c_int {
    0
}

// original: readdata (htslib/samples/pileup.c:76)
pub unsafe extern "C" fn samples_pileup_c_76_readdata(
    data: *mut c_void,
    b: *mut sam::bam1_t,
) -> c_int {
    let conf = data.cast::<PlpConf>();
    if conf.is_null() || (*conf).infile.is_null() {
        return -2;
    }
    sam::sam_read1((*conf).infile, (*conf).in_samhdr, b)
}

// original: main (htslib/samples/pileup.c:92)
pub unsafe fn samples_pileup_c_92_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ret = libc::EXIT_FAILURE;
    let mut conf = PlpConf {
        inname: std::ptr::null_mut(),
        infile: std::ptr::null_mut(),
        in_samhdr: std::ptr::null_mut(),
    };

    if argc != 2 {
        samples_pileup_c_38_print_usage(hts_sys::stderr.cast());
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
        Some(samples_pileup_c_76_readdata),
        (&mut conf as *mut PlpConf).cast(),
    );
    if plpiter.is_null() {
        libc::printf(c"Failed to initialize pileup data\n".as_ptr());
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
        libc::printf(c"%d\t%d\t".as_ptr(), tid + 1, refpos + 1);
        for j in 0..n {
            let p = plp.add(j as usize);
            if sam::bam_pileup1_is_del(p) != 0 || sam::bam_pileup1_is_refskip(p) != 0 {
                libc::printf(c"*".as_ptr());
                continue;
            }
            let seq = sam::bam_get_seq((*p).b);
            let base = SEQ_NT16_STR[sam::bam_seqi(seq, (*p).qpos as usize) as usize] as c_int;
            let out = if sam::bam_pileup1_is_head(p) != 0 || sam::bam_pileup1_is_tail(p) != 0 {
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
    ret
}
