use crate::htslib_rs::{
    hts::{
        self, htsFile, htsThreadPool, hts_idx_t, hts_itr_t, hts_pos_t, hts_readrec_func,
        HTS_FORMAT_SEQUENCE_DATA, HTS_FORMAT_VARIANT_DATA,
    },
    sam,
    thread_pool::{hts_tpool_destroy, hts_tpool_init},
    vcf,
};
unsafe extern "C" {
    static mut optarg: *mut u8;
    static mut optind: i32;
}

unsafe extern "C" fn hts_itr_query_adapter(
    idx: *const hts_idx_t,
    tid: i32,
    beg: hts_pos_t,
    end: hts_pos_t,
    readrec: hts_readrec_func,
) -> *mut hts_itr_t {
    hts::hts_itr_query(idx, tid, beg, end, readrec)
}

unsafe extern "C" fn bcf_hdr_name2id_adapter(
    data: *mut std::ffi::c_void,
    name: *const std::ffi::c_char,
) -> i32 {
    vcf::bcf_hdr_name2id(data.cast(), name.cast())
}

unsafe extern "C" fn bcf_readrec_adapter(
    fp: *mut hts::BGZF,
    data: *mut std::ffi::c_void,
    r: *mut std::ffi::c_void,
    tid: *mut i32,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> i32 {
    vcf::bcf_readrec(fp, data.cast(), r.cast(), tid, beg, end)
}

unsafe fn bcf_itr_querys1(
    idx: *const hts_idx_t,
    hdr: *mut vcf::bcf_hdr_t,
    region: *const u8,
) -> *mut hts_itr_t {
    hts::hts_itr_querys(
        idx,
        region.cast(),
        Some(bcf_hdr_name2id_adapter),
        hdr.cast(),
        Some(hts_itr_query_adapter),
        Some(bcf_readrec_adapter),
    )
}

unsafe fn bcf_itr_next(htsfp: *mut htsFile, itr: *mut hts_itr_t, r: *mut vcf::bcf1_t) -> i32 {
    if ((*htsfp).bitfields & (1 << 4)) != 0 {
        return crate::htslib_rs::hts::hts_itr_next(
            (*htsfp).fp.bgzf,
            itr,
            r.cast(),
            std::ptr::null_mut(),
        );
    }
    *libc::__errno_location() = libc::EINVAL;
    -2
}

// original: opts (htslib/test/test_view.c:40)
#[repr(C)]
pub struct opts {
    pub fn_ref: *mut u8,
    pub flag: i32,
    pub clevel: i32,
    pub ignore_sam_err: i32,
    pub nreads: i32,
    pub extra_hdr_nuls: i32,
    pub benchmark: i32,
    pub nthreads: i32,
    pub multi_reg: i32,
    pub index: *mut u8,
    pub min_shift: i32,
}

const READ_COMPRESSED: i32 = 1;
const WRITE_BINARY_COMP: i32 = 2; // eg bam, bcf
const READ_CRAM: i32 = 4;
const WRITE_CRAM: i32 = 8;
const WRITE_UNCOMPRESSED: i32 = 16;
const WRITE_COMPRESSED: i32 = 32; // eg vcf.gz, sam.gz, fastq.gz
const WRITE_FASTQ: i32 = 64;
const WRITE_FASTA: i32 = 128;

// original: sam_loop (htslib/test/test_view.c:65)
pub unsafe fn test_test_view_c_65_sam_loop(
    argc: i32,
    argv: *mut *mut u8,
    local_optind: i32,
    opts: *mut opts,
    in_: *mut htsFile,
    out: *mut htsFile,
) -> i32 {
    let mut r = 0;
    let mut idx: *mut hts_idx_t = std::ptr::null_mut();
    let mut b: *mut sam::bam1_t = std::ptr::null_mut();

    let h: *mut sam::sam_hdr_t = sam::sam_hdr_read(in_);
    if h.is_null() {
        eprintln!(
            "Couldn't read header for {:?}",
            std::ffi::CStr::from_ptr((*argv.add(local_optind as usize)).cast())
        );
        return libc::EXIT_FAILURE;
    }
    (*h).ignore_sam_err = (*opts).ignore_sam_err;
    if (*opts).extra_hdr_nuls > 0 {
        let new_len = (*h).l_text + (*opts).extra_hdr_nuls as usize;
        let mut text: Vec<u8> = Vec::with_capacity(new_len);
        text.extend_from_slice(std::slice::from_raw_parts((*h).text, (*h).l_text));
        text.resize(new_len, 0);
        // Leak the Vec so the (still raw) header keeps owning the buffer.
        (*h).text = text.as_mut_ptr();
        std::mem::forget(text);
        (*h).l_text = new_len;
    }

    b = sam::bam_init1();
    if b.is_null() {
        eprintln!("Out of memory allocating BAM struct");
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
        let ret = hts::hts_set_fai_filename(out, (*opts).fn_ref.cast());

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
        eprintln!("Error writing output header.");
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

    if !(*opts).index.is_null() && sam::sam_idx_init(out, h, (*opts).min_shift, (*opts).index) < 0 {
        eprintln!("Failed to initialise index");
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

    if local_optind + 1 < argc && ((*opts).flag & READ_COMPRESSED) == 0 {
        // BAM input and has a region
        let index_missing = {
            idx = sam::sam_index_load(in_, *argv.add(local_optind as usize));
            idx.is_null()
        };
        if index_missing {
            eprintln!("[E::sam_loop] fail to load the BAM index");
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
                    eprintln!("Error writing output.");
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
                eprintln!("Error reading input.");
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
                    eprintln!(
                        "[E::sam_loop] fail to parse region {:?}",
                        std::ffi::CStr::from_ptr((*argv.add(i as usize)).cast())
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
                        eprintln!("Error writing output.");
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
                    eprintln!("Error reading input.");
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
                eprintln!("Error writing output.");
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
        eprintln!("Error parsing input.");
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

    if !(*opts).index.is_null() && sam::sam_idx_save(out) < 0 {
        eprintln!("Error saving index");
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

    sam::bam_destroy1(b);
    sam::sam_hdr_destroy(h);

    0
}

// original: vcf_loop (htslib/test/test_view.c:196)
pub unsafe fn test_test_view_c_196_vcf_loop(
    argc: i32,
    argv: *mut *mut u8,
    local_optind: i32,
    opts: *mut opts,
    in_: *mut htsFile,
    out: *mut htsFile,
) -> i32 {
    let h = vcf::bcf_hdr_read(in_);
    let b = vcf::bcf_init();
    let idx: *mut hts_idx_t;
    let mut exit_code = 0;
    let mut r: i32;

    if h.is_null() {
        return 1;
    }
    if b.is_null() {
        return 1;
    }

    if (*opts).benchmark == 0 && vcf::bcf_hdr_write(out, h) < 0 {
        return 1;
    }

    if !(*opts).index.is_null()
        && vcf::bcf_idx_init(out, h, (*opts).min_shift, (*opts).index.cast()) < 0
    {
        eprintln!("Failed to initialise index");
        return 1;
    }

    if local_optind + 1 < argc {
        // A series of regions.
        idx = vcf::bcf_index_load2((*argv.add(local_optind as usize)).cast(), std::ptr::null());
        if idx.is_null() {
            eprintln!("[E::vcf_loop] fail to load the BVCF index");
            return 1;
        }

        let mut i = local_optind + 1;
        while i < argc {
            let iter = bcf_itr_querys1(idx, h, *argv.add(i as usize));
            if iter.is_null() {
                eprintln!(
                    "[E::vcf_loop] fail to parse region {:?}",
                    std::ffi::CStr::from_ptr((*argv.add(i as usize)).cast())
                );
                exit_code = 1;
                break;
            }
            while {
                r = bcf_itr_next(in_, iter, b);
                r >= 0
            } {
                if (*opts).benchmark == 0 && vcf::bcf_write(out, h, b) < 0 {
                    eprintln!("Error writing output.");
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
                eprintln!("Error reading input.");
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
                eprintln!("Error writing output.");
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
            eprintln!("Error reading input.");
            exit_code = 1;
        }
    }

    if exit_code == 0 && !(*opts).index.is_null() && vcf::bcf_idx_save(out) < 0 {
        eprintln!("Error saving index");
        exit_code = 1;
    }

    vcf::bcf_destroy(b);
    vcf::bcf_hdr_destroy(h);
    exit_code
}

// original: main (htslib/test/test_view.c:279)
pub unsafe fn test_test_view_c_279_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut moder = [0u8; 8];
    let mut modew = [0u8; 800];
    let mut c: i32;
    let mut exit_code = libc::EXIT_SUCCESS;
    let mut in_opts: *mut hts::hts_opt = std::ptr::null_mut();
    let mut out_opts: *mut hts::hts_opt = std::ptr::null_mut();
    let mut out_fn = c"-".as_ptr() as *mut u8;

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
        c = libc::getopt(argc, argv.cast(), c"DSIt:i:bzCfFul:o:N:BZ:@:Mx:m:p:v".as_ptr());
        if c < 0 {
            break;
        }
        match c as u8 {
            b'D' => opts.flag |= READ_CRAM,
            b'S' => opts.flag |= READ_COMPRESSED,
            b'I' => opts.ignore_sam_err = 1,
            b't' => opts.fn_ref = optarg,
            b'i' => {
                if hts::hts_opt_add(&mut in_opts, optarg.cast()) != 0 {
                    return 1;
                }
            }
            b'b' => opts.flag |= WRITE_BINARY_COMP,
            b'z' => opts.flag |= WRITE_COMPRESSED,
            b'C' => opts.flag |= WRITE_CRAM,
            b'f' => opts.flag |= WRITE_FASTQ,
            b'F' => opts.flag |= WRITE_FASTA,
            b'u' => opts.flag |= WRITE_UNCOMPRESSED, // eg u-BAM not SAM
            b'l' => opts.clevel = libc::atoi(optarg.cast()),
            b'o' => {
                if hts::hts_opt_add(&mut out_opts, optarg.cast()) != 0 {
                    return 1;
                }
            }
            b'N' => opts.nreads = libc::atoi(optarg.cast()),
            b'B' => opts.benchmark = 1,
            b'Z' => opts.extra_hdr_nuls = libc::atoi(optarg.cast()),
            b'M' => opts.multi_reg = 1,
            b'@' => opts.nthreads = libc::atoi(optarg.cast()),
            b'x' => opts.index = optarg,
            b'm' => opts.min_shift = libc::atoi(optarg.cast()),
            b'p' => out_fn = optarg,
            b'v' => hts::hts_verbose += 1,
            _ => {}
        }
    }
    let local_optind = optind;
    if argc == local_optind {
        eprintln!("Usage: test_view [-DSI] [-t fn_ref] [-i option=value] [-bC] [-l level] [-o option=value] [-N num_reads] [-B] [-Z hdr_nuls] [-@ num_threads] [-x index_fn] [-m min_shift] [-p out] [-v] <in.bam>|<in.sam>|<in.cram> [region]");
        eprintln!();
        eprintln!("-D: read CRAM format (mode 'c')");
        eprintln!("-S: read compressed BCF, BAM, FAI (mode 'b')");
        eprintln!("-I: ignore SAM parsing errors");
        eprintln!("-t: fn_ref: load CRAM references from the specified fasta file instead of @SQ headers when writing a CRAM file");
        eprintln!("-i: option=value: set an option for CRAM input");
        eprintln!();
        eprintln!("-b: write binary compressed BCF, BAM, FAI (mode 'b')");
        eprintln!("-z: write text compressed VCF.gz, SAM.gz or FASTQ.gz (mode 'z')");
        eprintln!("-C: write CRAM format (mode 'c')");
        eprintln!("-f: write FASTQ format (mode 'f')");
        eprintln!("-l 0-9: set zlib compression level");
        eprintln!("-o option=value: set an option for CRAM output");
        eprintln!("-N: num_reads: limit the output to the first num_reads reads");
        eprintln!();
        eprintln!("-B: enable benchmarking");
        eprintln!("-M: use hts_itr_multi iterator");
        eprintln!("-Z hdr_nuls: append specified number of null bytes to the SAM header");
        eprintln!("-@ num_threads: use thread pool with specified number of threads\n");
        eprintln!("-x fn: write index to fn");
        eprintln!("-m min_shift: specifies BAI/CSI bin size; 0 is BAI(BAM) or TBI(VCF), 14 is CSI default");
        eprintln!("-p out_fn: output to out_fn instead of stdout");
        eprintln!("-v: increase verbosity");
        eprintln!("The region list entries should be specified as 'reg:beg-end', with intervals of a region being disjunct and sorted by the starting coordinate.");
        return 1;
    }
    libc::strcpy(moder.as_mut_ptr().cast(), c"r".as_ptr());
    if (opts.flag & READ_CRAM) != 0 {
        libc::strcat(moder.as_mut_ptr().cast(), c"c".as_ptr());
    } else if (opts.flag & READ_COMPRESSED) == 0 {
        libc::strcat(moder.as_mut_ptr().cast(), c"b".as_ptr());
    }

    let in_: *mut htsFile =
        hts::hts_open((*argv.add(local_optind as usize)).cast(), moder.as_ptr().cast());
    if in_.is_null() {
        eprintln!(
            "Error opening {:?}",
            std::ffi::CStr::from_ptr((*argv.add(local_optind as usize)).cast())
        );
        return libc::EXIT_FAILURE;
    }

    libc::strcpy(modew.as_mut_ptr().cast(), c"w".as_ptr());
    if opts.clevel >= 0 && opts.clevel <= 9 {
        libc::snprintf(
            modew.as_mut_ptr().add(1).cast(),
            modew.len() - 1,
            c"%d".as_ptr(),
            opts.clevel,
        );
    }
    if (opts.flag & WRITE_CRAM) != 0 {
        libc::strcat(modew.as_mut_ptr().cast(), c"c".as_ptr());
    } else if (opts.flag & WRITE_BINARY_COMP) != 0 {
        libc::strcat(modew.as_mut_ptr().cast(), c"b".as_ptr());
    } else if (opts.flag & WRITE_COMPRESSED) != 0 {
        libc::strcat(modew.as_mut_ptr().cast(), c"z".as_ptr());
    } else if (opts.flag & WRITE_UNCOMPRESSED) != 0 {
        libc::strcat(modew.as_mut_ptr().cast(), c"bu".as_ptr());
    }
    if (opts.flag & WRITE_FASTQ) != 0 {
        libc::strcat(modew.as_mut_ptr().cast(), c"f".as_ptr());
    } else if (opts.flag & WRITE_FASTA) != 0 {
        libc::strcat(modew.as_mut_ptr().cast(), c"F".as_ptr());
    }
    let out: *mut htsFile = hts::hts_open(out_fn.cast(), modew.as_ptr().cast());
    if out.is_null() {
        eprintln!("Error opening standard output");
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
            eprintln!("Error creating thread pool");
            exit_code = 1;
        } else if hts::hts_set_thread_pool(in_, &mut p) < 0
            || hts::hts_set_thread_pool(out, &mut p) < 0
        {
            eprintln!("Threaded BGZF is not yet supported in this translation");
            exit_code = 1;
            thread_pool_failed = 1;
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
                eprintln!("Unsupported or unknown category of data in input file");
                return libc::EXIT_FAILURE;
            }
        }

        if ret != 0 {
            exit_code = libc::EXIT_FAILURE;
        }
    }

    ret = hts::hts_close(out);
    if ret < 0 {
        eprintln!("Error closing output.");
        exit_code = libc::EXIT_FAILURE;
    }
    ret = hts::hts_close(in_);
    if ret < 0 {
        eprintln!("Error closing input.");
        exit_code = libc::EXIT_FAILURE;
    }

    if !p.pool.is_null() {
        hts_tpool_destroy(p.pool);
    }

    use std::io::Write as _;
    if std::io::stdout().flush().is_err() && *libc::__errno_location() != libc::EBADF {
        eprintln!("Error closing standard output.");
        exit_code = libc::EXIT_FAILURE;
    }

    exit_code
}

#[cfg(test)]
mod tests {
    use super::*;
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

    unsafe fn run_main_in_child(args: &mut [Vec<u8>]) -> i32 {
        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");
        if pid == 0 {
            optind = 1;
            let mut argv = args
                .iter_mut()
                .map(|arg| arg.as_mut_ptr())
                .collect::<Vec<*mut u8>>();
            let rc = test_test_view_c_279_main(argv.len() as i32, argv.as_mut_ptr());
            libc::_exit(rc);
        }

        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        assert!(libc::WIFEXITED(status), "child did not exit normally");
        libc::WEXITSTATUS(status)
    }

    #[test]
    fn original_test_view_main_writes_limited_bam_as_sam() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            let out = temp_path("limited-bam");
            // NUL-terminated byte strings: main() still parses these via getopt/hts_open
            // as raw C strings at the FFI boundary.
            let nul_term = |s: &[u8]| {
                let mut v = s.to_vec();
                v.push(0);
                v
            };
            let mut args = vec![
                nul_term(b"test_view"),
                nul_term(b"-N"),
                nul_term(b"2"),
                nul_term(b"-p"),
                nul_term(out.to_string_lossy().as_bytes()),
                nul_term(fixture("htslib/test/range.bam").to_string_lossy().as_bytes()),
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
