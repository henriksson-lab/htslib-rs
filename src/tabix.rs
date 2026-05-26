use crate::htslib_rs::{
    bgzf,
    hts::{
        self, htsFile, htsThreadPool, hts_idx_t, hts_itr_t, hts_pos_t, hts_readrec_func, kstring_t,
        BGZF, HTS_FORMAT_BAM, HTS_FORMAT_BCF, HTS_FORMAT_BED, HTS_FORMAT_CRAM, HTS_FORMAT_SAM,
        HTS_FORMAT_TEXT_FORMAT, HTS_FORMAT_UNKNOWN_FORMAT, HTS_FORMAT_VCF, HTS_IDX_SAVE_REMOTE,
        HTS_POS_MAX,
    },
    regidx, sam, tbx, thread_pool, vcf,
};
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;

const NO_ARGUMENT: c_int = 0;
const REQUIRED_ARGUMENT: c_int = 1;
const IS_GFF: c_int = 1 << 0;
const IS_BED: c_int = 1 << 1;
const IS_SAM: c_int = 1 << 2;
const IS_VCF: c_int = 1 << 3;
const IS_BCF: c_int = 1 << 4;
const IS_BAM: c_int = 1 << 5;
const IS_CRAM: c_int = 1 << 6;
const IS_GAF: c_int = 1 << 7;
const IS_TXT: c_int = IS_GFF | IS_BED | IS_SAM | IS_VCF;
const TBX_UCSC: c_int = 0x10000;
const TBX_GAF: c_int = 3;

#[repr(C)]
struct GetoptLongOption {
    name: *const c_char,
    has_arg: c_int,
    flag: *mut c_int,
    val: c_int,
}

unsafe extern "C" {
    fn getopt_long(
        argc: c_int,
        argv: *mut *mut c_char,
        optstring: *const c_char,
        longopts: *const GetoptLongOption,
        longindex: *mut c_int,
    ) -> c_int;
    static mut optarg: *mut c_char;
    static mut optind: c_int;
}

#[repr(C)]
pub struct args_t {
    pub regions_fname: *mut c_char,
    pub targets_fname: *mut c_char,
    pub print_header: c_int,
    pub header_only: c_int,
    pub cache_megs: c_int,
    pub download_index: c_int,
    pub separate_regs: c_int,
    pub threads: c_int,
}

unsafe fn cstr_to_string(s: *const c_char) -> String {
    if s.is_null() {
        "(null)".to_string()
    } else {
        CStr::from_ptr(s).to_string_lossy().into_owned()
    }
}

unsafe fn error_message(message: String) -> ! {
    libc::fflush(hts_sys::stdout.cast());
    let message = CString::new(message).unwrap_or_else(|_| CString::new("fatal error\n").unwrap());
    libc::fputs(message.as_ptr(), hts_sys::stderr.cast());
    libc::fflush(hts_sys::stderr.cast());
    libc::exit(libc::EXIT_FAILURE);
}

unsafe fn error_errno_message(message: Option<String>) -> ! {
    let eno = *libc::__errno_location();
    libc::fflush(hts_sys::stdout.cast());
    if let Some(message) = message.as_ref() {
        let message =
            CString::new(message.as_str()).unwrap_or_else(|_| CString::new("fatal error").unwrap());
        libc::fputs(message.as_ptr(), hts_sys::stderr.cast());
    }
    if eno != 0 {
        if message.is_some() {
            libc::fputs(c": ".as_ptr(), hts_sys::stderr.cast());
        }
        libc::fputs(libc::strerror(eno), hts_sys::stderr.cast());
        libc::fputc(b'\n' as c_int, hts_sys::stderr.cast());
    } else {
        libc::fputc(b'\n' as c_int, hts_sys::stderr.cast());
    }
    libc::fflush(hts_sys::stderr.cast());
    libc::exit(libc::EXIT_FAILURE);
}

unsafe fn release_tpool(pool: *mut thread_pool::hts_tpool) {
    if !pool.is_null() {
        thread_pool::hts_tpool_destroy(pool);
    }
}

unsafe extern "C" fn tbx_name2id_adapter(data: *mut c_void, name: *const c_char) -> c_int {
    tbx::tbx_name2id(data.cast(), name)
}

unsafe extern "C" fn tbx_readrec_adapter(
    fp: *mut BGZF,
    data: *mut c_void,
    r: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    tbx::tbx_readrec(fp, data, r, tid, beg, end)
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

unsafe extern "C" fn bcf_hdr_id2name_adapter(data: *mut c_void, id: c_int) -> *const c_char {
    vcf::bcf_hdr_id2name(data.cast(), id)
}

unsafe extern "C" fn bcf_readrec_adapter(
    fp: *mut BGZF,
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

unsafe fn tbx_itr_querys1(tbx_: *mut tbx::tbx_t, region: *const c_char) -> *mut hts_itr_t {
    hts::hts_itr_querys(
        (*tbx_).idx.cast(),
        region,
        Some(tbx_name2id_adapter),
        tbx_.cast(),
        Some(hts_itr_query_adapter),
        Some(tbx_readrec_adapter),
    )
}

unsafe fn bcf_itr_next(htsfp: *mut htsFile, itr: *mut hts_itr_t, r: *mut vcf::bcf1_t) -> c_int {
    if ((*htsfp).bitfields & (1 << 4)) != 0 {
        return hts_sys::hts_itr_next(
            (*htsfp).fp.bgzf.cast(),
            itr.cast(),
            r.cast(),
            ptr::null_mut(),
        );
    }
    *libc::__errno_location() = libc::EINVAL;
    -2
}

unsafe fn tbx_itr_next(
    fp: *mut htsFile,
    tbx_: *mut tbx::tbx_t,
    itr: *mut hts_itr_t,
    str_: *mut kstring_t,
) -> c_int {
    hts::hts_itr_next(
        (*fp).fp.bgzf.cast(),
        itr,
        str_.cast(),
        tbx_.cast::<c_void>(),
    )
}

unsafe fn tbx_conf_gaf() -> tbx::tbx_conf_t {
    tbx::tbx_conf_t {
        preset: TBX_GAF,
        sc: 1,
        bc: 6,
        ec: 0,
        meta_char: b'#' as c_int,
        line_skip: 0,
    }
}

// original: file_type (htslib/tabix.c:102)
pub unsafe fn tabix_c_102_file_type(fname: *const c_char) -> c_int {
    let l = libc::strlen(fname);
    if l >= 7 && libc::strcasecmp(fname.add(l - 7), c".gff.gz".as_ptr()) == 0 {
        return IS_GFF;
    } else if l >= 7 && libc::strcasecmp(fname.add(l - 7), c".bed.gz".as_ptr()) == 0 {
        return IS_BED;
    } else if l >= 7 && libc::strcasecmp(fname.add(l - 7), c".sam.gz".as_ptr()) == 0 {
        return IS_SAM;
    } else if l >= 7 && libc::strcasecmp(fname.add(l - 7), c".vcf.gz".as_ptr()) == 0 {
        return IS_VCF;
    } else if l >= 4 && libc::strcasecmp(fname.add(l - 4), c".bcf".as_ptr()) == 0 {
        return IS_BCF;
    } else if l >= 4 && libc::strcasecmp(fname.add(l - 4), c".bam".as_ptr()) == 0 {
        return IS_BAM;
    } else if l >= 5 && libc::strcasecmp(fname.add(l - 5), c".cram".as_ptr()) == 0 {
        return IS_CRAM;
    } else if l >= 7 && libc::strcasecmp(fname.add(l - 7), c".gaf.gz".as_ptr()) == 0 {
        return IS_GAF;
    }

    let fp = hts::hts_open(fname, c"r".as_ptr());
    if fp.is_null() {
        if *libc::__errno_location() == libc::ENOEXEC {
            error_message(format!(
                "Couldn't understand format of \"{}\"\n",
                cstr_to_string(fname)
            ));
        } else {
            error_errno_message(Some(format!("Couldn't open \"{}\"", cstr_to_string(fname))));
        }
    }
    let format = (*hts::hts_get_format(fp)).format;
    hts::hts_close(fp);
    if format == HTS_FORMAT_BCF {
        return IS_BCF;
    }
    if format == HTS_FORMAT_BAM {
        return IS_BAM;
    }
    if format == HTS_FORMAT_CRAM {
        return IS_CRAM;
    }
    if format == HTS_FORMAT_VCF {
        return IS_VCF;
    }

    0
}

// original: parse_regions (htslib/tabix.c:135)
pub unsafe fn tabix_c_135_parse_regions(
    regions_fname: *mut c_char,
    argv: *mut *mut c_char,
    argc: c_int,
    nregs: *mut c_int,
) -> *mut *mut c_char {
    let mut ireg = 0;
    let mut regs: *mut *mut c_char = ptr::null_mut();
    *nregs = argc;

    if !regions_fname.is_null() {
        // improve me: this is a too heavy machinery for parsing regions...

        let idx = regidx::regidx_c_246_regidx_init(regions_fname, None, None, 0, ptr::null_mut());
        if idx.is_null() {
            error_errno_message(Some(format!(
                "Could not build region list for \"{}\"",
                cstr_to_string(regions_fname)
            )));
        }
        let itr = regidx::regidx_c_584_regitr_init(idx);
        if itr.is_null() {
            error_errno_message(Some(format!(
                "Could not initialize an iterator over \"{}\"",
                cstr_to_string(regions_fname)
            )));
        }

        *nregs += regidx::regidx_c_98_regidx_nregs(idx);
        regs = libc::malloc((std::mem::size_of::<*mut c_char>() as c_int * *nregs) as usize)
            .cast::<*mut c_char>();
        if regs.is_null() {
            error_errno_message(None);
        }

        let mut nseq = 0;
        let seqs = regidx::regidx_c_105_regidx_seq_names(idx, &mut nseq);
        let mut iseq = 0;
        while iseq < nseq {
            if regidx::regidx_c_401_regidx_overlap(
                idx,
                *seqs.add(iseq as usize),
                0,
                HTS_POS_MAX,
                itr,
            ) < 0
            {
                error_errno_message(Some("Failed to build overlapping regions list".to_string()));
            }

            while regidx::regidx_c_612_regitr_overlap(itr) != 0 {
                let reg = CString::new(format!(
                    "{}:{}-{}",
                    cstr_to_string(*seqs.add(iseq as usize)),
                    (*itr).beg + 1,
                    (*itr).end + 1
                ))
                .unwrap();
                *regs.add(ireg as usize) = libc::strdup(reg.as_ptr());
                if (*regs.add(ireg as usize)).is_null() {
                    error_errno_message(None);
                }
                ireg += 1;
            }
            iseq += 1;
        }
        regidx::regidx_c_606_regitr_destroy(itr);
        regidx::regidx_c_311_regidx_destroy(idx);
    }

    if ireg == 0 {
        if argc != 0 {
            regs = libc::malloc((std::mem::size_of::<*mut c_char>() as c_int * argc) as usize)
                .cast::<*mut c_char>();
            if regs.is_null() {
                error_errno_message(None);
            }
        } else {
            regs = libc::malloc(std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
            if regs.is_null() {
                error_errno_message(None);
            }
            *regs = libc::strdup(c".".as_ptr());
            if (*regs).is_null() {
                error_errno_message(None);
            }
            *nregs = 1;
        }
    }

    let mut iseq = 0;
    while iseq < argc {
        *regs.add(ireg as usize) = libc::strdup(*argv.add(iseq as usize));
        if (*regs.add(ireg as usize)).is_null() {
            error_errno_message(None);
        }
        iseq += 1;
        ireg += 1;
    }
    regs
}

// original: query_regions (htslib/tabix.c:206)
pub unsafe fn tabix_c_206_query_regions(
    args: *mut args_t,
    conf: *mut tbx::tbx_conf_t,
    fname: *mut c_char,
    regs: *mut *mut c_char,
    nregs: c_int,
) -> c_int {
    let mut tpool = htsThreadPool {
        pool: ptr::null_mut(),
        qsize: 0,
    };
    let fp = hts::hts_open(fname, c"r".as_ptr());
    if fp.is_null() {
        error_errno_message(Some(format!(
            "Could not open \"{}\"",
            cstr_to_string(fname)
        )));
    }
    let format = (*hts::hts_get_format(fp)).format;
    if (*args).cache_megs != 0 {
        hts::hts_set_cache_size(fp, (*args).cache_megs * 1048576);
    }

    //set threads if needed, errors are logged and ignored
    if (*args).threads >= 1 {
        tpool.pool = thread_pool::hts_tpool_init((*args).threads);
        if !tpool.pool.is_null() {
            if hts::hts_set_thread_pool(fp, &mut tpool) < 0 {
                release_tpool(tpool.pool.cast());
                error_errno_message(Some(
                    "Threaded BGZF is not yet supported in this translation".to_string(),
                ));
            }
        }
    }

    let mut reg_idx: *mut regidx::regidx_t = ptr::null_mut();
    if !(*args).targets_fname.is_null() {
        reg_idx =
            regidx::regidx_c_246_regidx_init((*args).targets_fname, None, None, 0, ptr::null_mut());
        if reg_idx.is_null() {
            release_tpool(tpool.pool.cast());
            error_errno_message(Some(format!(
                "Could not build region list for \"{}\"",
                cstr_to_string((*args).targets_fname)
            )));
        }
    }

    if format == HTS_FORMAT_BCF {
        let out = hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
        if out.is_null() {
            release_tpool(tpool.pool.cast());
            error_errno_message(Some("Could not open stdout".to_string()));
        }
        if !tpool.pool.is_null() && hts::hts_set_thread_pool(out, &mut tpool) < 0 {
            release_tpool(tpool.pool.cast());
            error_errno_message(Some(
                "Threaded BGZF is not yet supported in this translation".to_string(),
            ));
        }
        let idx = vcf::bcf_index_load3(
            fname,
            ptr::null(),
            if (*args).download_index != 0 {
                HTS_IDX_SAVE_REMOTE
            } else {
                0
            },
        );
        if idx.is_null() {
            release_tpool(tpool.pool.cast());
            error_errno_message(Some(format!(
                "Could not load .csi index of \"{}\"",
                cstr_to_string(fname)
            )));
        }

        let hdr = vcf::bcf_hdr_read(fp);
        if hdr.is_null() {
            release_tpool(tpool.pool.cast());
            error_errno_message(Some(format!(
                "Could not read the header from \"{}\"",
                cstr_to_string(fname)
            )));
        }

        if (*args).print_header != 0 && vcf::bcf_hdr_write(out, hdr) != 0 {
            release_tpool(tpool.pool.cast());
            error_errno_message(Some("Failed to write to stdout".to_string()));
        }
        if (*args).header_only == 0 {
            assert!(!regs.is_null());
            let rec = vcf::bcf_init();
            if rec.is_null() {
                release_tpool(tpool.pool.cast());
                error_errno_message(None);
            }
            let mut i = 0;
            while i < nregs {
                let mut found = 0;
                let itr = bcf_itr_querys1(idx, hdr, *regs.add(i as usize));
                if itr.is_null() {
                    i += 1;
                    continue;
                }
                let mut ret = bcf_itr_next(fp, itr, rec);
                while ret >= 0 {
                    if !reg_idx.is_null() {
                        let chr = vcf::bcf_seqname(hdr, rec);
                        if chr.is_null() {
                            release_tpool(tpool.pool.cast());
                            error_message(format!(
                                "Bad BCF record in \"{}\" : Invalid CONTIG id {}\n",
                                cstr_to_string(fname),
                                (*rec).rid
                            ));
                        }
                        if regidx::regidx_c_401_regidx_overlap(
                            reg_idx,
                            chr,
                            (*rec).pos,
                            (*rec).pos + (*rec).rlen as hts_pos_t - 1,
                            ptr::null_mut(),
                        ) == 0
                        {
                            ret = bcf_itr_next(fp, itr, rec);
                            continue;
                        }
                    }
                    if found == 0 {
                        if (*args).separate_regs != 0 {
                            libc::printf(
                                c"%c%s\n".as_ptr(),
                                (*conf).meta_char,
                                *regs.add(i as usize),
                            );
                        }
                        found = 1;
                    }
                    if vcf::bcf_write(out, hdr, rec) != 0 {
                        release_tpool(tpool.pool.cast());
                        error_errno_message(Some("Failed to write to stdout".to_string()));
                    }
                    ret = bcf_itr_next(fp, itr, rec);
                }
                if ret < -1 {
                    release_tpool(tpool.pool.cast());
                    error_errno_message(Some(format!(
                        "Reading \"{}\" failed",
                        cstr_to_string(fname)
                    )));
                }
                hts::hts_itr_destroy(itr);
                i += 1;
            }
            vcf::bcf_destroy(rec);
        }
        if hts::hts_close(out) != 0 {
            release_tpool(tpool.pool.cast());
            error_errno_message(Some(
                "hts_close returned non-zero status for stdout".to_string(),
            ));
        }

        vcf::bcf_hdr_destroy(hdr);
        hts::hts_idx_destroy(idx);
    } else if format == HTS_FORMAT_VCF
        || format == HTS_FORMAT_SAM
        || format == HTS_FORMAT_BED
        || format == HTS_FORMAT_TEXT_FORMAT
        || format == HTS_FORMAT_UNKNOWN_FORMAT
    {
        let tbx_ = tbx::tbx_index_load3(
            fname,
            ptr::null(),
            if (*args).download_index != 0 {
                HTS_IDX_SAVE_REMOTE
            } else {
                0
            },
        );
        if tbx_.is_null() {
            release_tpool(tpool.pool.cast());
            error_errno_message(Some(format!(
                "Could not load .tbi/.csi index of {}",
                cstr_to_string(fname)
            )));
        }
        let mut str_: kstring_t = std::mem::zeroed();
        if (*args).print_header != 0 {
            let mut ret = hts::hts_getline(fp, 2, &mut str_);
            while ret >= 0 {
                if str_.l == 0 || *str_.s != (*tbx_).conf.meta_char as c_char {
                    break;
                }
                if libc::puts(str_.s) < 0 {
                    release_tpool(tpool.pool.cast());
                    error_errno_message(Some("Error writing to stdout".to_string()));
                }
                ret = hts::hts_getline(fp, 2, &mut str_);
            }
            if ret < -1 {
                release_tpool(tpool.pool.cast());
                error_errno_message(Some(format!(
                    "Reading \"{}\" failed",
                    cstr_to_string(fname)
                )));
            }
        }
        if (*args).header_only == 0 {
            let mut nseq = 0;
            let mut seq: *mut *const c_char = ptr::null_mut();
            if !reg_idx.is_null() {
                seq = tbx::tbx_seqnames(tbx_, &mut nseq);
                if seq.is_null() {
                    release_tpool(tpool.pool.cast());
                    error_errno_message(Some("Failed to get sequence names list".to_string()));
                }
            }
            let mut i = 0;
            while i < nregs {
                let mut found = 0;
                let itr = tbx_itr_querys1(tbx_, *regs.add(i as usize));
                if itr.is_null() {
                    i += 1;
                    continue;
                }
                let mut ret = tbx_itr_next(fp, tbx_, itr, &mut str_);
                while ret >= 0 {
                    if !reg_idx.is_null()
                        && regidx::regidx_c_401_regidx_overlap(
                            reg_idx,
                            *seq.add((*itr).curr_tid as usize),
                            (*itr).curr_beg,
                            (*itr).curr_end - 1,
                            ptr::null_mut(),
                        ) == 0
                    {
                        ret = tbx_itr_next(fp, tbx_, itr, &mut str_);
                        continue;
                    }
                    if found == 0 {
                        if (*args).separate_regs != 0 {
                            libc::printf(
                                c"%c%s\n".as_ptr(),
                                (*conf).meta_char,
                                *regs.add(i as usize),
                            );
                        }
                        found = 1;
                    }
                    if libc::puts(str_.s) < 0 {
                        release_tpool(tpool.pool.cast());
                        error_errno_message(Some("Failed to write to stdout".to_string()));
                    }
                    ret = tbx_itr_next(fp, tbx_, itr, &mut str_);
                }
                if ret < -1 {
                    release_tpool(tpool.pool.cast());
                    error_errno_message(Some(format!(
                        "Reading \"{}\" failed",
                        cstr_to_string(fname)
                    )));
                }
                hts::hts_itr_destroy(itr);
                i += 1;
            }
            libc::free(seq.cast());
        }
        libc::free(str_.s.cast());
        tbx::tbx_destroy(tbx_);
    } else if format == HTS_FORMAT_BAM {
        release_tpool(tpool.pool.cast());
        error_message("Please use \"samtools view\" for querying BAM files.\n".to_string());
    }

    if !reg_idx.is_null() {
        regidx::regidx_c_311_regidx_destroy(reg_idx);
    }
    if hts::hts_close(fp) != 0 {
        release_tpool(tpool.pool.cast());
        error_errno_message(Some(format!(
            "hts_close returned non-zero status: {}",
            cstr_to_string(fname)
        )));
    }

    let mut i = 0;
    while i < nregs {
        libc::free((*regs.add(i as usize)).cast());
        i += 1;
    }
    libc::free(regs.cast());
    release_tpool(tpool.pool.cast());
    0
}

// original: query_chroms (htslib/tabix.c:396)
pub unsafe fn tabix_c_396_query_chroms(fname: *mut c_char, download: c_int) -> c_int {
    let mut nseq = 0;
    let ftype = tabix_c_102_file_type(fname);
    if (ftype & IS_TXT) != 0 || ftype == 0 {
        let tbx_ = tbx::tbx_index_load3(
            fname,
            ptr::null(),
            if download != 0 {
                HTS_IDX_SAVE_REMOTE
            } else {
                0
            },
        );
        if tbx_.is_null() {
            error_errno_message(Some(format!(
                "Could not load .tbi index of {}",
                cstr_to_string(fname)
            )));
        }
        let seq = tbx::tbx_seqnames(tbx_, &mut nseq);
        if seq.is_null() {
            error_errno_message(Some("Couldn't get list of sequence names".to_string()));
        }
        let mut i = 0;
        while i < nseq {
            if libc::printf(c"%s\n".as_ptr(), *seq.add(i as usize)) < 0 {
                error_errno_message(Some("Couldn't write to stdout".to_string()));
            }
            i += 1;
        }
        libc::free(seq.cast());
        tbx::tbx_destroy(tbx_);
    } else if ftype == IS_BCF {
        let fp = hts::hts_open(fname, c"r".as_ptr());
        if fp.is_null() {
            error_errno_message(Some(format!(
                "Could not open \"{}\"",
                cstr_to_string(fname)
            )));
        }
        let hdr = vcf::bcf_hdr_read(fp);
        if hdr.is_null() {
            error_errno_message(Some(format!(
                "Could not read the header: \"{}\"",
                cstr_to_string(fname)
            )));
        }
        hts::hts_close(fp);
        let idx = vcf::bcf_index_load3(
            fname,
            ptr::null(),
            if download != 0 {
                HTS_IDX_SAVE_REMOTE
            } else {
                0
            },
        );
        if idx.is_null() {
            error_errno_message(Some(format!(
                "Could not load .csi index of \"{}\"",
                cstr_to_string(fname)
            )));
        }
        let seq = hts::hts_idx_seqnames(idx, &mut nseq, Some(bcf_hdr_id2name_adapter), hdr.cast());
        if seq.is_null() {
            error_errno_message(Some("Couldn't get list of sequence names".to_string()));
        }
        let mut i = 0;
        while i < nseq {
            if libc::printf(c"%s\n".as_ptr(), *seq.add(i as usize)) < 0 {
                error_errno_message(Some("Couldn't write to stdout".to_string()));
            }
            i += 1;
        }
        libc::free(seq.cast());
        vcf::bcf_hdr_destroy(hdr);
        hts::hts_idx_destroy(idx);
    } else if ftype == IS_BAM {
        error_message("BAM: todo\n".to_string());
    }
    0
}

// original: reheader_file (htslib/tabix.c:437)
pub unsafe fn tabix_c_437_reheader_file(
    fname: *const c_char,
    header: *const c_char,
    ftype: c_int,
    conf: *mut tbx::tbx_conf_t,
    threads: c_int,
) -> c_int {
    let mut tpool: *mut thread_pool::hts_tpool = ptr::null_mut();
    if threads >= 1 {
        tpool = thread_pool::hts_tpool_init(threads);
    }
    if (ftype & IS_TXT) != 0 || ftype == 0 {
        let fp = bgzf::bgzf_open(fname, c"r".as_ptr());
        if fp.is_null() {
            release_tpool(tpool);
            return -1;
        }
        if !tpool.is_null() && bgzf::bgzf_thread_pool(fp, tpool, 0) < 0 {
            release_tpool(tpool);
            bgzf::bgzf_close(fp);
            return -1;
        }
        if hts_sys::bgzf_read_block(fp.cast()) != 0 || (*fp).block_length == 0 {
            release_tpool(tpool);
            return -1;
        }

        let buffer = (*fp).uncompressed_block.cast::<c_char>();
        let mut skip_until = 0;

        // Skip the header: find out the position of the data block
        if *buffer == (*conf).meta_char as c_char {
            skip_until = 1;
            loop {
                if *buffer.add(skip_until as usize) == b'\n' as c_char {
                    skip_until += 1;
                    if skip_until >= (*fp).block_length {
                        if hts_sys::bgzf_read_block(fp.cast()) != 0 || (*fp).block_length == 0 {
                            release_tpool(tpool);
                            error_message(format!(
                                "FIXME: No body in the file: {}\n",
                                cstr_to_string(fname)
                            ));
                        }
                        skip_until = 0;
                    }
                    // The header has finished
                    if *buffer.add(skip_until as usize) != (*conf).meta_char as c_char {
                        break;
                    }
                }
                skip_until += 1;
                if skip_until >= (*fp).block_length {
                    if hts_sys::bgzf_read_block(fp.cast()) != 0 || (*fp).block_length == 0 {
                        release_tpool(tpool);
                        error_message(format!(
                            "FIXME: No body in the file: {}\n",
                            cstr_to_string(fname)
                        ));
                    }
                    skip_until = 0;
                }
            }
        }

        // Output the new header
        let hdr = libc::fopen(header, c"r".as_ptr());
        if hdr.is_null() {
            release_tpool(tpool);
            error_message(format!(
                "{}: {}",
                cstr_to_string(header),
                cstr_to_string(libc::strerror(*libc::__errno_location()))
            ));
        }
        let page_size = 32768usize;
        let buf = libc::malloc(page_size).cast::<c_char>();
        let bgzf_out = bgzf::bgzf_open(c"-".as_ptr(), c"w".as_ptr());

        if buf.is_null() {
            release_tpool(tpool);
            error_message(format!(
                "{}\n",
                cstr_to_string(libc::strerror(*libc::__errno_location()))
            ));
        }
        if bgzf_out.is_null() {
            release_tpool(tpool);
            error_errno_message(Some("Couldn't open output stream".to_string()));
        }
        if !tpool.is_null() && bgzf::bgzf_thread_pool(bgzf_out, tpool, 0) < 0 {
            release_tpool(tpool);
            bgzf::bgzf_close(fp);
            bgzf::bgzf_close(bgzf_out);
            return -1;
        }
        let mut nread = libc::fread(buf.cast(), 1, page_size - 1, hdr) as isize;
        while nread > 0 {
            if nread < (page_size - 1) as isize && *buf.add((nread - 1) as usize) != b'\n' as c_char
            {
                *buf.add(nread as usize) = b'\n' as c_char;
                nread += 1;
            }
            if bgzf::bgzf_write(bgzf_out, buf.cast(), nread as usize) < 0 {
                release_tpool(tpool);
                error_errno_message(Some(format!("Write error {}", (*bgzf_out).bitfields >> 16)));
            }
            nread = libc::fread(buf.cast(), 1, page_size - 1, hdr) as isize;
        }
        if libc::ferror(hdr) != 0 {
            release_tpool(tpool);
            error_errno_message(Some(format!(
                "Failed to read \"{}\"",
                cstr_to_string(header)
            )));
        }
        if libc::fclose(hdr) != 0 {
            release_tpool(tpool);
            error_errno_message(Some(format!(
                "Closing \"{}\" failed",
                cstr_to_string(header)
            )));
        }

        // Output all remaining data read with the header block
        if (*fp).block_length - skip_until > 0
            && bgzf::bgzf_write(
                bgzf_out,
                buffer.add(skip_until as usize).cast(),
                ((*fp).block_length - skip_until) as usize,
            ) < 0
        {
            release_tpool(tpool);
            error_errno_message(Some(format!("Write error {}", (*fp).bitfields >> 16)));
        }
        if bgzf::bgzf_flush(bgzf_out) < 0 {
            release_tpool(tpool);
            error_errno_message(Some(format!("Write error {}", (*bgzf_out).bitfields >> 16)));
        }

        loop {
            nread = bgzf::bgzf_raw_read(fp, buf.cast(), page_size);
            if nread <= 0 {
                break;
            }

            let count = bgzf::bgzf_raw_write(bgzf_out, buf.cast(), nread as usize);
            if count != nread {
                release_tpool(tpool);
                error_errno_message(Some(format!(
                    "Write failed, wrote {} instead of {} bytes",
                    count, nread as c_int
                )));
            }
        }
        if nread < 0 {
            release_tpool(tpool);
            error_errno_message(Some(format!("Error reading \"{}\"", cstr_to_string(fname))));
        }
        if bgzf::bgzf_close(bgzf_out) < 0 {
            release_tpool(tpool);
            error_errno_message(Some(format!(
                "Error {} closing output",
                (*bgzf_out).bitfields >> 16
            )));
        }
        if bgzf::bgzf_close(fp) < 0 {
            release_tpool(tpool);
            error_errno_message(Some(format!(
                "Error {} closing \"{}\"",
                (*bgzf_out).bitfields >> 16,
                cstr_to_string(fname)
            )));
        }
        libc::free(buf.cast());
    } else {
        release_tpool(tpool);
        error_message("todo: reheader BCF, BAM\n".to_string());
    }
    release_tpool(tpool);
    0
}

// original: usage (htslib/tabix.c:580)
pub unsafe fn tabix_c_580_usage(fp: *mut libc::FILE, status: c_int) -> c_int {
    libc::fprintf(fp, c"\n".as_ptr());
    libc::fprintf(
        fp,
        c"Version: %s\n".as_ptr(),
        crate::htslib_rs::hts::hts_version(),
    );
    libc::fprintf(
        fp,
        c"Usage:   tabix [OPTIONS] [FILE] [REGION [...]]\n".as_ptr(),
    );
    libc::fprintf(fp, c"\n".as_ptr());
    libc::fprintf(fp, c"Indexing Options:\n".as_ptr());
    libc::fprintf(
        fp,
        c"   -0, --zero-based           coordinates are zero-based\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -b, --begin INT            column number for region start [4]\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -c, --comment CHAR         skip comment lines starting with CHAR [null]\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -C, --csi                  generate CSI index for VCF (default is TBI)\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -e, --end INT              column number for region end (if no end, set INT to -b) [5]\n"
            .as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -f, --force                overwrite existing index without asking\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -m, --min-shift INT        set minimal interval size for CSI indices to 2^INT [14]\n"
            .as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -p, --preset STR           gff, bed, sam, vcf, gaf\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -s, --sequence INT         column number for sequence names (suppressed by -p) [1]\n"
            .as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -S, --skip-lines INT       skip first INT lines [0]\n".as_ptr(),
    );
    libc::fprintf(fp, c"\n".as_ptr());
    libc::fprintf(fp, c"Querying and other options:\n".as_ptr());
    libc::fprintf(
        fp,
        c"   -h, --print-header         print also the header lines\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -H, --only-header          print only the header lines\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -l, --list-chroms          list chromosome names\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -r, --reheader FILE        replace the header with the content of FILE\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -R, --regions FILE         restrict to regions listed in the file\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -T, --targets FILE         similar to -R but streams rather than index-jumps\n"
            .as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -D                         do not download the index file\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"       --cache INT            set cache size to INT megabytes (0 disables) [10]\n"
            .as_ptr(),
    );
    libc::fprintf(
        fp,
        c"       --separate-regions     separate the output by corresponding regions\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"       --verbosity INT        set verbosity [3]\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -@, --threads INT          number of additional threads to use [0]\n".as_ptr(),
    );
    libc::fprintf(fp, c"\n".as_ptr());
    status
}

// original: main (htslib/tabix.c:614)
pub unsafe fn tabix_c_614_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut detect = 1;
    let mut min_shift = 0;
    let mut is_force = 0;
    let mut list_chroms = 0;
    let mut do_csi = 0;
    let mut conf = tbx::tbx_conf_gff();
    let mut reheader: *mut c_char = ptr::null_mut();
    let mut args: args_t = std::mem::zeroed();
    args.cache_megs = 10;
    args.download_index = 1;
    let mut new_line_skip = -1;

    let mut loptions = [
        GetoptLongOption {
            name: c"help".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: 2,
        },
        GetoptLongOption {
            name: c"regions".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'R' as c_int,
        },
        GetoptLongOption {
            name: c"targets".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'T' as c_int,
        },
        GetoptLongOption {
            name: c"csi".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'C' as c_int,
        },
        GetoptLongOption {
            name: c"zero-based".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'0' as c_int,
        },
        GetoptLongOption {
            name: c"print-header".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'h' as c_int,
        },
        GetoptLongOption {
            name: c"only-header".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'H' as c_int,
        },
        GetoptLongOption {
            name: c"begin".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'b' as c_int,
        },
        GetoptLongOption {
            name: c"comment".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'c' as c_int,
        },
        GetoptLongOption {
            name: c"end".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'e' as c_int,
        },
        GetoptLongOption {
            name: c"force".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'f' as c_int,
        },
        GetoptLongOption {
            name: c"min-shift".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'm' as c_int,
        },
        GetoptLongOption {
            name: c"preset".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'p' as c_int,
        },
        GetoptLongOption {
            name: c"sequence".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b's' as c_int,
        },
        GetoptLongOption {
            name: c"skip-lines".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'S' as c_int,
        },
        GetoptLongOption {
            name: c"list-chroms".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'l' as c_int,
        },
        GetoptLongOption {
            name: c"reheader".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'r' as c_int,
        },
        GetoptLongOption {
            name: c"version".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: 1,
        },
        GetoptLongOption {
            name: c"verbosity".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: 3,
        },
        GetoptLongOption {
            name: c"cache".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: 4,
        },
        GetoptLongOption {
            name: c"separate-regions".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: 5,
        },
        GetoptLongOption {
            name: c"threads".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'@' as c_int,
        },
        GetoptLongOption {
            name: ptr::null(),
            has_arg: 0,
            flag: ptr::null_mut(),
            val: 0,
        },
    ];

    let mut tmp: *mut c_char = ptr::null_mut();
    loop {
        let c = getopt_long(
            argc,
            argv,
            c"hH?0b:c:e:fm:p:s:S:lr:CR:T:D@:".as_ptr(),
            loptions.as_mut_ptr(),
            ptr::null_mut(),
        );
        if c < 0 {
            break;
        }
        match c {
            x if x == b'R' as c_int => args.regions_fname = optarg,
            x if x == b'T' as c_int => args.targets_fname = optarg,
            x if x == b'C' as c_int => do_csi = 1,
            x if x == b'r' as c_int => reheader = optarg,
            x if x == b'h' as c_int => args.print_header = 1,
            x if x == b'H' as c_int => {
                args.print_header = 1;
                args.header_only = 1;
            }
            x if x == b'l' as c_int => list_chroms = 1,
            x if x == b'0' as c_int => {
                conf.preset |= TBX_UCSC;
                detect = 0;
            }
            x if x == b'b' as c_int => {
                conf.bc = libc::strtol(optarg, &mut tmp, 10) as c_int;
                if *tmp != 0 {
                    error_message(format!(
                        "Could not parse argument: -b {}\n",
                        cstr_to_string(optarg)
                    ));
                }
                detect = 0;
            }
            x if x == b'e' as c_int => {
                conf.ec = libc::strtol(optarg, &mut tmp, 10) as c_int;
                if *tmp != 0 {
                    error_message(format!(
                        "Could not parse argument: -e {}\n",
                        cstr_to_string(optarg)
                    ));
                }
                detect = 0;
            }
            x if x == b'c' as c_int => {
                conf.meta_char = *optarg as c_int;
                detect = 0;
            }
            x if x == b'f' as c_int => is_force = 1,
            x if x == b'm' as c_int => {
                min_shift = libc::strtol(optarg, &mut tmp, 10) as c_int;
                if *tmp != 0 {
                    error_message(format!(
                        "Could not parse argument: -m {}\n",
                        cstr_to_string(optarg)
                    ));
                }
            }
            x if x == b'p' as c_int => {
                detect = 0;
                if libc::strcmp(optarg, c"gff".as_ptr()) == 0 {
                    conf = tbx::tbx_conf_gff();
                } else if libc::strcmp(optarg, c"bed".as_ptr()) == 0 {
                    conf = tbx::tbx_conf_bed();
                } else if libc::strcmp(optarg, c"sam".as_ptr()) == 0 {
                    conf = tbx::tbx_conf_sam();
                } else if libc::strcmp(optarg, c"vcf".as_ptr()) == 0 {
                    conf = tbx::tbx_conf_vcf();
                } else if libc::strcmp(optarg, c"gaf".as_ptr()) == 0 {
                    conf = tbx_conf_gaf();
                } else if libc::strcmp(optarg, c"bcf".as_ptr()) == 0 {
                    detect = 1;
                } else if libc::strcmp(optarg, c"bam".as_ptr()) == 0 {
                    detect = 1;
                } else {
                    error_message(format!(
                        "The preset string not recognised: '{}'\n",
                        cstr_to_string(optarg)
                    ));
                }
            }
            x if x == b's' as c_int => {
                conf.sc = libc::strtol(optarg, &mut tmp, 10) as c_int;
                if *tmp != 0 {
                    error_message(format!(
                        "Could not parse argument: -s {}\n",
                        cstr_to_string(optarg)
                    ));
                }
                detect = 0;
            }
            x if x == b'S' as c_int => {
                new_line_skip = libc::strtol(optarg, &mut tmp, 10) as c_int;
                if *tmp != 0 {
                    error_message(format!(
                        "Could not parse argument: -S {}\n",
                        cstr_to_string(optarg)
                    ));
                }
                detect = 0;
            }
            x if x == b'D' as c_int => args.download_index = 0,
            1 => {
                libc::printf(
                    c"tabix (htslib) %s\nCopyright (C) 2025 Genome Research Ltd.\n".as_ptr(),
                    hts::hts_version(),
                );
                return libc::EXIT_SUCCESS;
            }
            2 => return tabix_c_580_usage(hts_sys::stdout.cast(), libc::EXIT_SUCCESS),
            3 => {
                let mut v = libc::atoi(optarg);
                if v < 0 {
                    v = 0;
                }
                hts::hts_set_log_level(v);
            }
            4 => {
                args.cache_megs = libc::atoi(optarg);
                if args.cache_megs < 0 {
                    args.cache_megs = 0;
                } else if args.cache_megs >= c_int::MAX / 1048576 {
                    args.cache_megs = c_int::MAX / 1048576;
                }
            }
            5 => args.separate_regs = 1,
            x if x == b'@' as c_int => args.threads = libc::atoi(optarg),
            _ => return tabix_c_580_usage(hts_sys::stderr.cast(), libc::EXIT_FAILURE),
        }
    }

    if new_line_skip >= 0 {
        conf.line_skip = new_line_skip;
    }

    if optind == argc {
        return tabix_c_580_usage(hts_sys::stderr.cast(), libc::EXIT_FAILURE);
    }

    if list_chroms != 0 {
        return tabix_c_396_query_chroms(*argv.add(optind as usize), args.download_index);
    }

    let fname = *argv.add(optind as usize);
    let ftype = tabix_c_102_file_type(fname);
    if detect != 0 {
        if ftype == IS_GFF {
            conf = tbx::tbx_conf_gff();
        } else if ftype == IS_BED {
            conf = tbx::tbx_conf_bed();
        } else if ftype == IS_GAF {
            conf = tbx_conf_gaf();
        } else if ftype == IS_SAM {
            conf = tbx::tbx_conf_sam();
        } else if ftype == IS_VCF {
            conf = tbx::tbx_conf_vcf();
            if min_shift == 0 && do_csi != 0 {
                min_shift = 14;
            }
        } else if ftype == IS_BCF {
            if min_shift == 0 {
                min_shift = 14;
            }
        } else if ftype == IS_BAM && min_shift == 0 {
            min_shift = 14;
        }
    }
    if argc > optind + 1
        || args.header_only != 0
        || !args.regions_fname.is_null()
        || !args.targets_fname.is_null()
    {
        let mut nregs = 0;
        let mut regs: *mut *mut c_char = ptr::null_mut();
        if args.header_only == 0 {
            regs = tabix_c_135_parse_regions(
                args.regions_fname,
                argv.add((optind + 1) as usize),
                argc - optind - 1,
                &mut nregs,
            );
        }
        return tabix_c_206_query_regions(&mut args, &mut conf, fname, regs, nregs);
    }
    if do_csi != 0 {
        if min_shift == 0 {
            min_shift = 14;
        }
        min_shift *= do_csi;
    }
    if min_shift != 0 && do_csi == 0 {
        do_csi = 1;
    }

    if !reheader.is_null() {
        return tabix_c_437_reheader_file(fname, reheader, ftype, &mut conf, args.threads);
    }

    let mut suffix = c".tbi".as_ptr();
    if do_csi != 0 {
        suffix = c".csi".as_ptr();
    } else if ftype == IS_BAM {
        suffix = c".bai".as_ptr();
    } else if ftype == IS_CRAM {
        suffix = c".crai".as_ptr();
    }

    let idx_fname = libc::calloc(libc::strlen(fname) + 6, 1).cast::<c_char>();
    if idx_fname.is_null() {
        error_message(format!(
            "{}\n",
            cstr_to_string(libc::strerror(*libc::__errno_location()))
        ));
    }
    libc::strcat(libc::strcpy(idx_fname, fname), suffix);

    let mut stat_tbi: libc::stat = std::mem::zeroed();
    let mut stat_file: libc::stat = std::mem::zeroed();
    if is_force == 0 && libc::stat(idx_fname, &mut stat_tbi) == 0 {
        // Before complaining about existing index, check if the VCF file isn't
        // newer. This is a common source of errors, people tend not to notice
        // that tabix failed
        libc::stat(fname, &mut stat_file);
        if stat_file.st_mtime <= stat_tbi.st_mtime {
            error_message(
                "[tabix] the index file exists. Please use '-f' to overwrite.\n".to_string(),
            );
        }
    }
    libc::free(idx_fname.cast());

    let ret;
    if ftype == IS_CRAM {
        if sam::sam_index_build3(fname, ptr::null(), min_shift, args.threads) != 0 {
            error_message(format!(
                "bam_index_build failed: {}\n",
                cstr_to_string(fname)
            ));
        }
        return 0;
    } else if do_csi != 0 {
        if ftype == IS_BCF {
            if vcf::bcf_index_build3(fname, ptr::null(), min_shift, args.threads) != 0 {
                error_message(format!(
                    "bcf_index_build failed: {}\n",
                    cstr_to_string(fname)
                ));
            }
            return 0;
        }
        if ftype == IS_BAM {
            if sam::sam_index_build3(fname, ptr::null(), min_shift, args.threads) != 0 {
                error_message(format!(
                    "bam_index_build failed: {}\n",
                    cstr_to_string(fname)
                ));
            }
            return 0;
        }

        ret = tbx::tbx_index_build3(fname, ptr::null(), min_shift, args.threads, &conf);
        match ret {
            0 => return 0,
            -2 => error_message(format!(
                "[tabix] the compression of '{}' is not BGZF\n",
                cstr_to_string(fname)
            )),
            _ => error_message(format!(
                "tbx_index_build3 failed: {}\n",
                cstr_to_string(fname)
            )),
        }
    } else {
        ret = tbx::tbx_index_build3(fname, ptr::null(), min_shift, args.threads, &conf);
        match ret {
            0 => return 0,
            -2 => error_message(format!(
                "[tabix] the compression of '{}' is not BGZF\n",
                cstr_to_string(fname)
            )),
            _ => error_message(format!(
                "tbx_index_build3 failed: {}\n",
                cstr_to_string(fname)
            )),
        }
    }
}
