use crate::htslib_rs::{
    hts::{
        self, htsFile, htsThreadPool, hts_idx_t, hts_itr_t, hts_pos_t, hts_readrec_func,
        HTS_FORMAT_SEQUENCE_DATA, HTS_FORMAT_VARIANT_DATA,
    },
    sam,
    thread_pool::{hts_tpool_destroy, hts_tpool_init},
    vcf,
};
use std::ffi::{c_char, c_int, c_void};

unsafe extern "C" {
    static mut optarg: *mut c_char;
    static mut optind: c_int;
}

unsafe extern "C" fn hts_itr_query_adapter(
    idx: *const hts_idx_t,
    tid: c_int,
    beg: hts_pos_t,
    end: hts_pos_t,
    readrec: hts_readrec_func,
) -> *mut hts_itr_t {
    hts::hts_itr_query(idx, tid, beg, end, readrec)
}

unsafe extern "C" fn bcf_hdr_name2id_adapter(data: *mut c_void, name: *const c_char) -> c_int {
    vcf::bcf_hdr_name2id(data.cast(), name)
}

unsafe extern "C" fn bcf_readrec_adapter(
    fp: *mut hts::BGZF,
    data: *mut c_void,
    r: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    vcf::bcf_readrec(fp, data, r, tid, beg, end)
}

unsafe fn bcf_itr_querys1(
    idx: *const hts_idx_t,
    hdr: *mut vcf::bcf_hdr_t,
    region: *const c_char,
) -> *mut hts_itr_t {
    hts::hts_itr_querys(
        idx,
        region,
        Some(bcf_hdr_name2id_adapter),
        hdr.cast(),
        Some(hts_itr_query_adapter),
        Some(bcf_readrec_adapter),
    )
}

unsafe fn bcf_itr_next(htsfp: *mut htsFile, itr: *mut hts_itr_t, r: *mut vcf::bcf1_t) -> c_int {
    if ((*htsfp).bitfields & (1 << 4)) != 0 {
        return hts_sys::hts_itr_next(
            (*htsfp).fp.bgzf.cast(),
            itr.cast(),
            r.cast(),
            std::ptr::null_mut(),
        );
    }
    *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
    -2
}

// original: opts (htslib/test/test_view.c:40)
#[repr(C)]
pub struct opts {
    pub fn_ref: *mut c_char,
    pub flag: c_int,
    pub clevel: c_int,
    pub ignore_sam_err: c_int,
    pub nreads: c_int,
    pub extra_hdr_nuls: c_int,
    pub benchmark: c_int,
    pub nthreads: c_int,
    pub multi_reg: c_int,
    pub index: *mut c_char,
    pub min_shift: c_int,
}

const READ_COMPRESSED: c_int = 1;
const WRITE_BINARY_COMP: c_int = 2; // eg bam, bcf
const READ_CRAM: c_int = 4;
const WRITE_CRAM: c_int = 8;
const WRITE_UNCOMPRESSED: c_int = 16;
const WRITE_COMPRESSED: c_int = 32; // eg vcf.gz, sam.gz, fastq.gz
const WRITE_FASTQ: c_int = 64;
const WRITE_FASTA: c_int = 128;

// original: sam_loop (htslib/test/test_view.c:65)
pub unsafe fn test_test_view_c_65_sam_loop(
    argc: c_int,
    argv: *mut *mut c_char,
    local_optind: c_int,
    opts: *mut opts,
    in_: *mut htsFile,
    out: *mut htsFile,
) -> c_int {
    let mut r = 0;
    let h: *mut sam::sam_hdr_t;
    let mut idx: *mut hts_idx_t = std::ptr::null_mut();
    let mut b: *mut sam::bam1_t = std::ptr::null_mut();

    h = sam::sam_hdr_read(in_);
    if h.is_null() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Couldn't read header for \"%s\"\n".as_ptr(),
            *argv.add(local_optind as usize),
        );
        return libc::EXIT_FAILURE;
    }
    (*h).ignore_sam_err = (*opts).ignore_sam_err;
    if (*opts).extra_hdr_nuls > 0 {
        let new_text = libc::realloc(
            (*h).text.cast(),
            (*h).l_text + (*opts).extra_hdr_nuls as usize,
        )
        .cast::<c_char>();
        if new_text.is_null() {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Error reallocing header text\n".as_ptr(),
            );
            if !b.is_null() {
                sam::bam_destroy1(b);
            }
            if !h.is_null() {
                sam::sam_hdr_destroy(h);
            }
            if !idx.is_null() {
                hts::hts_idx_destroy(idx);
            }
            return 1;
        }
        (*h).text = new_text;
        libc::memset(
            (*h).text.add((*h).l_text).cast(),
            0,
            (*opts).extra_hdr_nuls as usize,
        );
        (*h).l_text += (*opts).extra_hdr_nuls as usize;
    }

    b = sam::bam_init1();
    if b.is_null() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Out of memory allocating BAM struct\n".as_ptr(),
        );
        if !b.is_null() {
            sam::bam_destroy1(b);
        }
        if !h.is_null() {
            sam::sam_hdr_destroy(h);
        }
        if !idx.is_null() {
            hts::hts_idx_destroy(idx);
        }
        return 1;
    }

    /* CRAM output */
    if ((*opts).flag & WRITE_CRAM) != 0 && !(*opts).fn_ref.is_null() {
        // Create CRAM references arrays
        let ret = hts::hts_set_fai_filename(out, (*opts).fn_ref);

        if ret != 0 {
            if !b.is_null() {
                sam::bam_destroy1(b);
            }
            if !h.is_null() {
                sam::sam_hdr_destroy(h);
            }
            if !idx.is_null() {
                hts::hts_idx_destroy(idx);
            }
            return 1;
        }
    }

    if (*opts).benchmark == 0 && sam::sam_hdr_write(out, h) < 0 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Error writing output header.\n".as_ptr(),
        );
        if !b.is_null() {
            sam::bam_destroy1(b);
        }
        if !h.is_null() {
            sam::sam_hdr_destroy(h);
        }
        if !idx.is_null() {
            hts::hts_idx_destroy(idx);
        }
        return 1;
    }

    if !(*opts).index.is_null() {
        if sam::sam_idx_init(out, h, (*opts).min_shift, (*opts).index) < 0 {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Failed to initialise index\n".as_ptr(),
            );
            if !b.is_null() {
                sam::bam_destroy1(b);
            }
            if !h.is_null() {
                sam::sam_hdr_destroy(h);
            }
            if !idx.is_null() {
                hts::hts_idx_destroy(idx);
            }
            return 1;
        }
    }

    if local_optind + 1 < argc && ((*opts).flag & READ_COMPRESSED) == 0 {
        // BAM input and has a region
        if {
            idx = sam::sam_index_load(in_, *argv.add(local_optind as usize));
            idx.is_null()
        } {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"[E::%s] fail to load the BAM index\n".as_ptr(),
                c"sam_loop".as_ptr(),
            );
            if !b.is_null() {
                sam::bam_destroy1(b);
            }
            if !h.is_null() {
                sam::sam_hdr_destroy(h);
            }
            if !idx.is_null() {
                hts::hts_idx_destroy(idx);
            }
            return 1;
        }
        if (*opts).multi_reg != 0 {
            let iter = sam::sam_c_1768_sam_itr_regarray(
                idx,
                h,
                argv.add((local_optind + 1) as usize),
                (argc - local_optind - 1) as u32,
            );
            if iter.is_null() {
                if !b.is_null() {
                    sam::bam_destroy1(b);
                }
                if !h.is_null() {
                    sam::sam_hdr_destroy(h);
                }
                if !idx.is_null() {
                    hts::hts_idx_destroy(idx);
                }
                return 1;
            }
            while {
                r = sam::sam_itr_next(in_, iter, b);
                r >= 0
            } {
                if (*opts).benchmark == 0 && sam::sam_c_4553_sam_write1(out, h, b) < 0 {
                    libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error writing output.\n".as_ptr());
                    hts::hts_itr_destroy(iter);
                    if !b.is_null() {
                        sam::bam_destroy1(b);
                    }
                    if !h.is_null() {
                        sam::sam_hdr_destroy(h);
                    }
                    if !idx.is_null() {
                        hts::hts_idx_destroy(idx);
                    }
                    return 1;
                }
                if (*opts).nreads != 0 {
                    (*opts).nreads -= 1;
                    if (*opts).nreads == 0 {
                        break;
                    }
                }
            }
            hts::hts_itr_destroy(iter);
            if r < -1 {
                libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error reading input.\n".as_ptr());
                if !b.is_null() {
                    sam::bam_destroy1(b);
                }
                if !h.is_null() {
                    sam::sam_hdr_destroy(h);
                }
                if !idx.is_null() {
                    hts::hts_idx_destroy(idx);
                }
                return 1;
            }
        } else {
            let mut i = local_optind + 1;
            while i < argc {
                let iter: *mut hts_itr_t = sam::sam_itr_querys(idx, h, *argv.add(i as usize));
                if iter.is_null() {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"[E::%s] fail to parse region '%s'\n".as_ptr(),
                        c"sam_loop".as_ptr(),
                        *argv.add(i as usize),
                    );
                    if !b.is_null() {
                        sam::bam_destroy1(b);
                    }
                    if !h.is_null() {
                        sam::sam_hdr_destroy(h);
                    }
                    if !idx.is_null() {
                        hts::hts_idx_destroy(idx);
                    }
                    return 1;
                }
                while {
                    r = sam::sam_itr_next(in_, iter, b);
                    r >= 0
                } {
                    if (*opts).benchmark == 0 && sam::sam_c_4553_sam_write1(out, h, b) < 0 {
                        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error writing output.\n".as_ptr());
                        hts::hts_itr_destroy(iter);
                        if !b.is_null() {
                            sam::bam_destroy1(b);
                        }
                        if !h.is_null() {
                            sam::sam_hdr_destroy(h);
                        }
                        if !idx.is_null() {
                            hts::hts_idx_destroy(idx);
                        }
                        return 1;
                    }
                    if (*opts).nreads != 0 {
                        (*opts).nreads -= 1;
                        if (*opts).nreads == 0 {
                            break;
                        }
                    }
                }
                hts::hts_itr_destroy(iter);
                if r < -1 {
                    libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error reading input.\n".as_ptr());
                    if !b.is_null() {
                        sam::bam_destroy1(b);
                    }
                    if !h.is_null() {
                        sam::sam_hdr_destroy(h);
                    }
                    if !idx.is_null() {
                        hts::hts_idx_destroy(idx);
                    }
                    return 1;
                }
                i += 1;
            }
        }
        hts::hts_idx_destroy(idx);
        idx = std::ptr::null_mut();
    } else {
        while {
            r = sam::sam_read1(in_, h, b);
            r >= 0
        } {
            if (*opts).benchmark == 0 && sam::sam_c_4553_sam_write1(out, h, b) < 0 {
                libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error writing output.\n".as_ptr());
                if !b.is_null() {
                    sam::bam_destroy1(b);
                }
                if !h.is_null() {
                    sam::sam_hdr_destroy(h);
                }
                if !idx.is_null() {
                    hts::hts_idx_destroy(idx);
                }
                return 1;
            }
            if (*opts).nreads != 0 {
                (*opts).nreads -= 1;
                if (*opts).nreads == 0 {
                    break;
                }
            }
        }
    }

    if r < -1 {
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error parsing input.\n".as_ptr());
        if !b.is_null() {
            sam::bam_destroy1(b);
        }
        if !h.is_null() {
            sam::sam_hdr_destroy(h);
        }
        if !idx.is_null() {
            hts::hts_idx_destroy(idx);
        }
        return 1;
    }

    if !(*opts).index.is_null() {
        if sam::sam_idx_save(out) < 0 {
            libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error saving index\n".as_ptr());
            if !b.is_null() {
                sam::bam_destroy1(b);
            }
            if !h.is_null() {
                sam::sam_hdr_destroy(h);
            }
            if !idx.is_null() {
                hts::hts_idx_destroy(idx);
            }
            return 1;
        }
    }

    sam::bam_destroy1(b);
    sam::sam_hdr_destroy(h);

    0
}

// original: vcf_loop (htslib/test/test_view.c:196)
pub unsafe fn test_test_view_c_196_vcf_loop(
    argc: c_int,
    argv: *mut *mut c_char,
    local_optind: c_int,
    opts: *mut opts,
    in_: *mut htsFile,
    out: *mut htsFile,
) -> c_int {
    let h = vcf::bcf_hdr_read(in_);
    let b = vcf::bcf_init();
    let idx: *mut hts_idx_t;
    let mut exit_code = 0;
    let mut r: c_int;

    if h.is_null() {
        return 1;
    }
    if b.is_null() {
        return 1;
    }

    if (*opts).benchmark == 0 && vcf::bcf_hdr_write(out, h) < 0 {
        return 1;
    }

    if !(*opts).index.is_null() {
        if vcf::bcf_idx_init(out, h, (*opts).min_shift, (*opts).index) < 0 {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Failed to initialise index\n".as_ptr(),
            );
            return 1;
        }
    }

    if local_optind + 1 < argc {
        // A series of regions.
        idx = vcf::bcf_index_load2(*argv.add(local_optind as usize), std::ptr::null());
        if idx.is_null() {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"[E::%s] fail to load the BVCF index\n".as_ptr(),
                c"vcf_loop".as_ptr(),
            );
            return 1;
        }

        let mut i = local_optind + 1;
        while i < argc {
            let iter = bcf_itr_querys1(idx, h, *argv.add(i as usize));
            if iter.is_null() {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"[E::%s] fail to parse region '%s'\n".as_ptr(),
                    c"vcf_loop".as_ptr(),
                    *argv.add(i as usize),
                );
                exit_code = 1;
                break;
            }
            while {
                r = bcf_itr_next(in_, iter, b);
                r >= 0
            } {
                if (*opts).benchmark == 0 && vcf::bcf_write(out, h, b) < 0 {
                    libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error writing output.\n".as_ptr());
                    exit_code = 1;
                    break;
                }
                if (*opts).nreads != 0 {
                    (*opts).nreads -= 1;
                    if (*opts).nreads == 0 {
                        break;
                    }
                }
            }
            if r < -1 {
                libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error reading input.\n".as_ptr());
                exit_code = 1;
            }
            hts::hts_itr_destroy(iter);
            if exit_code != 0 {
                break;
            }
            i += 1;
        }

        hts::hts_idx_destroy(idx);
    } else {
        // Whole file
        while {
            r = vcf::bcf_read(in_, h, b);
            r >= 0
        } {
            if (*opts).benchmark == 0 && vcf::bcf_write(out, h, b) < 0 {
                libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error writing output.\n".as_ptr());
                exit_code = 1;
                break;
            }
            if (*opts).nreads != 0 {
                (*opts).nreads -= 1;
                if (*opts).nreads == 0 {
                    break;
                }
            }
        }
        if r < -1 {
            libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error reading input.\n".as_ptr());
            exit_code = 1;
        }
    }

    if exit_code == 0 && !(*opts).index.is_null() {
        if vcf::bcf_idx_save(out) < 0 {
            libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error saving index\n".as_ptr());
            exit_code = 1;
        }
    }

    vcf::bcf_destroy(b);
    vcf::bcf_hdr_destroy(h);
    exit_code
}

// original: main (htslib/test/test_view.c:279)
pub unsafe fn test_test_view_c_279_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let in_: *mut htsFile;
    let out: *mut htsFile;
    let mut moder = [0 as c_char; 8];
    let mut modew = [0 as c_char; 800];
    let mut c: c_int;
    let mut exit_code = libc::EXIT_SUCCESS;
    let mut in_opts: *mut hts::hts_opt = std::ptr::null_mut();
    let mut out_opts: *mut hts::hts_opt = std::ptr::null_mut();
    let mut out_fn = c"-".as_ptr() as *mut c_char;

    let mut opts = opts {
        fn_ref: std::ptr::null_mut(),
        flag: 0,
        clevel: -1,
        ignore_sam_err: 0,
        nreads: 0,
        extra_hdr_nuls: 0,
        benchmark: 0,
        nthreads: 0, // shared pool
        multi_reg: 0,
        index: std::ptr::null_mut(),
        min_shift: 0,
    };

    loop {
        c = libc::getopt(argc, argv, c"DSIt:i:bzCfFul:o:N:BZ:@:Mx:m:p:v".as_ptr());
        if c < 0 {
            break;
        }
        match c as u8 {
            b'D' => opts.flag |= READ_CRAM,
            b'S' => opts.flag |= READ_COMPRESSED,
            b'I' => opts.ignore_sam_err = 1,
            b't' => opts.fn_ref = optarg,
            b'i' => {
                if hts::hts_opt_add(&mut in_opts, optarg) != 0 {
                    return 1;
                }
            }
            b'b' => opts.flag |= WRITE_BINARY_COMP,
            b'z' => opts.flag |= WRITE_COMPRESSED,
            b'C' => opts.flag |= WRITE_CRAM,
            b'f' => opts.flag |= WRITE_FASTQ,
            b'F' => opts.flag |= WRITE_FASTA,
            b'u' => opts.flag |= WRITE_UNCOMPRESSED, // eg u-BAM not SAM
            b'l' => opts.clevel = libc::atoi(optarg),
            b'o' => {
                if hts::hts_opt_add(&mut out_opts, optarg) != 0 {
                    return 1;
                }
            }
            b'N' => opts.nreads = libc::atoi(optarg),
            b'B' => opts.benchmark = 1,
            b'Z' => opts.extra_hdr_nuls = libc::atoi(optarg),
            b'M' => opts.multi_reg = 1,
            b'@' => opts.nthreads = libc::atoi(optarg),
            b'x' => opts.index = optarg,
            b'm' => opts.min_shift = libc::atoi(optarg),
            b'p' => out_fn = optarg,
            b'v' => hts::hts_verbose += 1,
            _ => {}
        }
    }
    let local_optind = optind;
    if argc == local_optind {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Usage: test_view [-DSI] [-t fn_ref] [-i option=value] [-bC] [-l level] [-o option=value] [-N num_reads] [-B] [-Z hdr_nuls] [-@ num_threads] [-x index_fn] [-m min_shift] [-p out] [-v] <in.bam>|<in.sam>|<in.cram> [region]\n".as_ptr(),
        );
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"\n".as_ptr());
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-D: read CRAM format (mode 'c')\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-S: read compressed BCF, BAM, FAI (mode 'b')\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-I: ignore SAM parsing errors\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-t: fn_ref: load CRAM references from the specified fasta file instead of @SQ headers when writing a CRAM file\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-i: option=value: set an option for CRAM input\n".as_ptr(),
        );
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"\n".as_ptr());
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-b: write binary compressed BCF, BAM, FAI (mode 'b')\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-z: write text compressed VCF.gz, SAM.gz or FASTQ.gz (mode 'z')\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-C: write CRAM format (mode 'c')\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-f: write FASTQ format (mode 'f')\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-l 0-9: set zlib compression level\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-o option=value: set an option for CRAM output\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-N: num_reads: limit the output to the first num_reads reads\n".as_ptr(),
        );
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"\n".as_ptr());
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-B: enable benchmarking\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-M: use hts_itr_multi iterator\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-Z hdr_nuls: append specified number of null bytes to the SAM header\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-@ num_threads: use thread pool with specified number of threads\n\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-x fn: write index to fn\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-m min_shift: specifies BAI/CSI bin size; 0 is BAI(BAM) or TBI(VCF), 14 is CSI default\n".as_ptr(),
        );
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"-p out_fn: output to out_fn instead of stdout\n".as_ptr(),
        );
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"-v: increase verbosity\n".as_ptr());
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"The region list entries should be specified as 'reg:beg-end', with intervals of a region being disjunct and sorted by the starting coordinate.\n".as_ptr(),
        );
        return 1;
    }
    libc::strcpy(moder.as_mut_ptr(), c"r".as_ptr());
    if (opts.flag & READ_CRAM) != 0 {
        libc::strcat(moder.as_mut_ptr(), c"c".as_ptr());
    } else if (opts.flag & READ_COMPRESSED) == 0 {
        libc::strcat(moder.as_mut_ptr(), c"b".as_ptr());
    }

    in_ = hts::hts_open(*argv.add(local_optind as usize), moder.as_ptr());
    if in_.is_null() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Error opening \"%s\"\n".as_ptr(),
            *argv.add(local_optind as usize),
        );
        return libc::EXIT_FAILURE;
    }

    libc::strcpy(modew.as_mut_ptr(), c"w".as_ptr());
    if opts.clevel >= 0 && opts.clevel <= 9 {
        libc::snprintf(
            modew.as_mut_ptr().add(1),
            modew.len() - 1,
            c"%d".as_ptr(),
            opts.clevel,
        );
    }
    if (opts.flag & WRITE_CRAM) != 0 {
        libc::strcat(modew.as_mut_ptr(), c"c".as_ptr());
    } else if (opts.flag & WRITE_BINARY_COMP) != 0 {
        libc::strcat(modew.as_mut_ptr(), c"b".as_ptr());
    } else if (opts.flag & WRITE_COMPRESSED) != 0 {
        libc::strcat(modew.as_mut_ptr(), c"z".as_ptr());
    } else if (opts.flag & WRITE_UNCOMPRESSED) != 0 {
        libc::strcat(modew.as_mut_ptr(), c"bu".as_ptr());
    }
    if (opts.flag & WRITE_FASTQ) != 0 {
        libc::strcat(modew.as_mut_ptr(), c"f".as_ptr());
    } else if (opts.flag & WRITE_FASTA) != 0 {
        libc::strcat(modew.as_mut_ptr(), c"F".as_ptr());
    }
    out = hts::hts_open(out_fn, modew.as_ptr());
    if out.is_null() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Error opening standard output\n".as_ptr(),
        );
        return libc::EXIT_FAILURE;
    }

    // Process any options; currently cram only.
    if hts::hts_opt_apply(in_, in_opts) != 0 {
        return libc::EXIT_FAILURE;
    }
    hts::hts_opt_free(in_opts);

    if hts::hts_opt_apply(out, out_opts) != 0 {
        return libc::EXIT_FAILURE;
    }
    hts::hts_opt_free(out_opts);

    // Create and share the thread pool
    let mut p = htsThreadPool {
        pool: std::ptr::null_mut(),
        qsize: 0,
    };
    let mut thread_pool_failed = 0;
    if opts.nthreads > 0 {
        p.pool = hts_tpool_init(opts.nthreads);
        if p.pool.is_null() {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Error creating thread pool\n".as_ptr(),
            );
            exit_code = 1;
        } else {
            if hts::hts_set_thread_pool(in_, &mut p) < 0
                || hts::hts_set_thread_pool(out, &mut p) < 0
            {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Threaded BGZF is not yet supported in this translation\n".as_ptr(),
                );
                exit_code = 1;
                thread_pool_failed = 1;
            }
        }
    }

    let mut ret;
    if thread_pool_failed == 0 {
        match (*hts::hts_get_format(in_)).category {
            HTS_FORMAT_SEQUENCE_DATA => {
                ret = test_test_view_c_65_sam_loop(argc, argv, local_optind, &mut opts, in_, out);
            }

            HTS_FORMAT_VARIANT_DATA => {
                ret = test_test_view_c_196_vcf_loop(argc, argv, local_optind, &mut opts, in_, out);
            }

            _ => {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Unsupported or unknown category of data in input file\n".as_ptr(),
                );
                return libc::EXIT_FAILURE;
            }
        }

        if ret != 0 {
            exit_code = libc::EXIT_FAILURE;
        }
    }

    ret = hts::hts_close(out);
    if ret < 0 {
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error closing output.\n".as_ptr());
        exit_code = libc::EXIT_FAILURE;
    }
    ret = hts::hts_close(in_);
    if ret < 0 {
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"Error closing input.\n".as_ptr());
        exit_code = libc::EXIT_FAILURE;
    }

    if !p.pool.is_null() {
        hts_tpool_destroy(p.pool);
    }

    if libc::fclose(crate::htslib_rs::c_compat::stdout.cast()) != 0
        && *crate::htslib_rs::c_compat::__errno_location() != libc::EBADF
    {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Error closing standard output.\n".as_ptr(),
        );
        exit_code = libc::EXIT_FAILURE;
    }

    exit_code
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::path::{Path, PathBuf};

    fn fixture(path: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "htslib-rs-test-view-{label}-{}-{nanos}.sam",
            std::process::id()
        ))
    }

    unsafe fn run_main_in_child(args: &mut [CString]) -> c_int {
        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");
        if pid == 0 {
            optind = 1;
            let mut argv = args
                .iter_mut()
                .map(|arg| arg.as_ptr().cast_mut())
                .collect::<Vec<_>>();
            let rc = test_test_view_c_279_main(argv.len() as c_int, argv.as_mut_ptr());
            libc::_exit(rc);
        }

        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        assert!(libc::WIFEXITED(status), "child did not exit normally");
        libc::WEXITSTATUS(status)
    }

    #[test]
    fn original_test_view_main_writes_limited_bam_as_sam() {
        unsafe {
            let out = temp_path("limited-bam");
            let mut args = vec![
                CString::new("test_view").unwrap(),
                CString::new("-N").unwrap(),
                CString::new("2").unwrap(),
                CString::new("-p").unwrap(),
                CString::new(out.to_string_lossy().as_bytes()).unwrap(),
                CString::new(
                    fixture("htslib/test/range.bam")
                        .to_string_lossy()
                        .as_bytes(),
                )
                .unwrap(),
            ];

            assert_eq!(run_main_in_child(&mut args), libc::EXIT_SUCCESS);

            let actual = std::fs::read_to_string(&out).unwrap();
            assert!(actual.starts_with("@HD\tVN:1.4\tSO:coordinate\n"));
            assert!(actual.contains("@SQ\tSN:CHROMOSOME_I\tLN:1009800\t"));

            let records = actual
                .lines()
                .filter(|line| !line.starts_with('@'))
                .collect::<Vec<_>>();
            assert_eq!(records.len(), 2);
            for record in records {
                let fields = record.split('\t').collect::<Vec<_>>();
                assert!(fields.len() >= 11, "malformed SAM record: {record}");
                assert!(!fields[0].is_empty(), "record has empty query name");
            }

            let _ = std::fs::remove_file(out);
        }
    }
}
