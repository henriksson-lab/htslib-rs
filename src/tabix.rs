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
use std::ffi::{c_char, c_int, c_void, CStr};
use std::io::Write;
use std::ptr::{self, NonNull};

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
    pub regions_fname: *const c_char,
    pub targets_fname: *const c_char,
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
    libc::fflush(crate::htslib_rs::c_compat::stdout.cast());
    let bytes = message.as_bytes();
    libc::fwrite(
        bytes.as_ptr().cast(),
        1,
        bytes.len(),
        crate::htslib_rs::c_compat::stderr.cast(),
    );
    libc::fflush(crate::htslib_rs::c_compat::stderr.cast());
    libc::exit(libc::EXIT_FAILURE);
}

unsafe fn error_errno_message(message: Option<String>) -> ! {
    let eno = *crate::htslib_rs::c_compat::__errno_location();
    libc::fflush(crate::htslib_rs::c_compat::stdout.cast());
    if let Some(message) = message.as_ref() {
        let bytes = message.as_bytes();
        libc::fwrite(
            bytes.as_ptr().cast(),
            1,
            bytes.len(),
            crate::htslib_rs::c_compat::stderr.cast(),
        );
    }
    if eno != 0 {
        if message.is_some() {
            libc::fputs(c": ".as_ptr(), crate::htslib_rs::c_compat::stderr.cast());
        }
        libc::fputs(
            libc::strerror(eno),
            crate::htslib_rs::c_compat::stderr.cast(),
        );
        libc::fputc(b'\n' as c_int, crate::htslib_rs::c_compat::stderr.cast());
    } else {
        libc::fputc(b'\n' as c_int, crate::htslib_rs::c_compat::stderr.cast());
    }
    libc::fflush(crate::htslib_rs::c_compat::stderr.cast());
    libc::exit(libc::EXIT_FAILURE);
}

struct OwnedThreadPool {
    pool: NonNull<thread_pool::hts_tpool>,
}

impl OwnedThreadPool {
    unsafe fn new(threads: c_int) -> Option<Self> {
        NonNull::new(thread_pool::hts_tpool_init(threads)).map(|pool| Self { pool })
    }

    fn as_ptr(&self) -> *mut thread_pool::hts_tpool {
        self.pool.as_ptr()
    }
}

impl Drop for OwnedThreadPool {
    fn drop(&mut self) {
        unsafe {
            thread_pool::hts_tpool_destroy(self.pool.as_ptr());
        }
    }
}

struct OwnedConstCharPtrArray {
    ptr: NonNull<*const c_char>,
    len: usize,
}

impl OwnedConstCharPtrArray {
    unsafe fn from_tbx(tbx: &tbx::tbx_t) -> Option<Self> {
        let mut len = 0;
        let ptr = tbx::tbx_seqnames(tbx, &mut len);
        NonNull::new(ptr).map(|ptr| Self {
            ptr,
            len: len.max(0) as usize,
        })
    }

    unsafe fn from_hts_idx(
        idx: &hts_idx_t,
        hdr: &mut vcf::bcf_hdr_t,
        getid: hts::hts_id2name_f,
    ) -> Option<Self> {
        let mut len = 0;
        let ptr = hts::hts_idx_seqnames(idx, &mut len, getid, hdr as *mut _ as *mut c_void);
        NonNull::new(ptr).map(|ptr| Self {
            ptr,
            len: len.max(0) as usize,
        })
    }

    fn as_slice(&self) -> &[*const c_char] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for OwnedConstCharPtrArray {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr.as_ptr().cast());
        }
    }
}

struct RegionBytes {
    bytes: Vec<u8>,
}

impl RegionBytes {
    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
        }
    }

    unsafe fn from_cstr(region: *const c_char) -> Self {
        Self::from_bytes(CStr::from_ptr(region).to_bytes())
    }

    unsafe fn from_interval(seq: *const c_char, beg: hts_pos_t, end: hts_pos_t) -> Self {
        let seq = CStr::from_ptr(seq).to_bytes();
        let mut bytes = Vec::with_capacity(seq.len() + 32);
        bytes.extend_from_slice(seq);
        write!(&mut bytes, ":{}-{}", beg + 1, end + 1).unwrap();
        Self { bytes }
    }

    fn to_c_terminated_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.bytes.len() + 1);
        bytes.extend_from_slice(&self.bytes);
        bytes.push(0);
        bytes
    }
}

unsafe fn malloc_c_string_from_bytes(bytes: &[u8]) -> *mut c_char {
    if bytes.contains(&0) {
        return ptr::null_mut();
    }
    let raw = libc::malloc(bytes.len() + 1).cast::<c_char>();
    if raw.is_null() {
        error_errno_message(None);
    }
    ptr::copy_nonoverlapping(bytes.as_ptr(), raw.cast(), bytes.len());
    *raw.add(bytes.len()) = 0;
    raw
}

unsafe extern "C" fn tbx_name2id_adapter(data: *mut c_void, name: *const c_char) -> c_int {
    let Some(tbx) = data.cast::<tbx::tbx_t>().as_ref() else {
        return -1;
    };
    tbx::tbx_name2id(tbx, name)
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
    let Some(tbx_) = tbx_.as_mut() else {
        return ptr::null_mut();
    };
    tbx::tbx_itr_querys1(tbx_, region)
}

unsafe fn bcf_itr_next(htsfp: *mut htsFile, itr: *mut hts_itr_t, r: *mut vcf::bcf1_t) -> c_int {
    if ((*htsfp).bitfields & (1 << 4)) != 0 {
        return hts::hts_itr_next(
            (*htsfp).fp.bgzf.cast(),
            itr.cast(),
            r.cast(),
            ptr::null_mut(),
        );
    }
    *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
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

unsafe fn cstr_ascii_ends_with(s: *const c_char, suffix: &[u8]) -> bool {
    let bytes = CStr::from_ptr(s).to_bytes();
    bytes.len() >= suffix.len() && bytes[bytes.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

// original: file_type (htslib/tabix.c:102)
pub unsafe fn tabix_c_102_file_type(fname: *const c_char) -> c_int {
    if cstr_ascii_ends_with(fname, b".gff.gz") {
        return IS_GFF;
    } else if cstr_ascii_ends_with(fname, b".bed.gz") {
        return IS_BED;
    } else if cstr_ascii_ends_with(fname, b".sam.gz") {
        return IS_SAM;
    } else if cstr_ascii_ends_with(fname, b".vcf.gz") {
        return IS_VCF;
    } else if cstr_ascii_ends_with(fname, b".bcf") {
        return IS_BCF;
    } else if cstr_ascii_ends_with(fname, b".bam") {
        return IS_BAM;
    } else if cstr_ascii_ends_with(fname, b".cram") {
        return IS_CRAM;
    } else if cstr_ascii_ends_with(fname, b".gaf.gz") {
        return IS_GAF;
    }

    let fp = hts::hts_open(fname, c"r".as_ptr());
    if fp.is_null() {
        if *crate::htslib_rs::c_compat::__errno_location() == libc::ENOEXEC {
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
    regions_fname: *const c_char,
    argv: *mut *mut c_char,
    argc: c_int,
    nregs: *mut c_int,
) -> *mut *mut c_char {
    let Some(nregs) = nregs.as_mut() else {
        return ptr::null_mut();
    };
    let Some(regions) = tabix_parse_regions_owned(regions_fname, argv, argc) else {
        return ptr::null_mut();
    };
    *nregs = regions.len() as c_int;
    let regs = libc::malloc(regions.len().max(1) * std::mem::size_of::<*mut c_char>())
        .cast::<*mut c_char>();
    if regs.is_null() {
        error_errno_message(None);
    }
    for (i, region) in regions.into_iter().enumerate() {
        let raw = malloc_c_string_from_bytes(&region.bytes);
        if raw.is_null() {
            error_message("region contains an interior NUL\n".to_string());
        }
        *regs.add(i) = raw;
    }
    regs
}

unsafe fn tabix_parse_regions_owned(
    regions_fname: *const c_char,
    argv: *mut *mut c_char,
    argc: c_int,
) -> Option<Vec<RegionBytes>> {
    if argc < 0 || (argc != 0 && argv.is_null()) {
        return None;
    }
    let mut regs = Vec::<RegionBytes>::new();
    if argc > 0 {
        regs.try_reserve(argc as usize).ok()?;
    }

    if !regions_fname.is_null() {
        // improve me: this is a too heavy machinery for parsing regions...

        let idx = regidx::regidx_c_246_regidx_init(
            regions_fname.cast_mut(),
            None,
            None,
            0,
            ptr::null_mut(),
        );
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

        let extra = regidx::regidx_c_98_regidx_nregs(idx).max(0) as usize;
        if regs.try_reserve(extra).is_err() {
            error_errno_message(None);
        }

        let mut nseq = 0;
        let seqs_raw = regidx::regidx_c_105_regidx_seq_names(idx, &mut nseq);
        if nseq > 0 && seqs_raw.is_null() {
            error_errno_message(Some("Failed to get region sequence names".to_string()));
        }
        let seqs = if nseq > 0 {
            std::slice::from_raw_parts(seqs_raw, nseq as usize)
        } else {
            &[]
        };
        for &seq in seqs {
            if regidx::regidx_c_401_regidx_overlap(idx, seq, 0, HTS_POS_MAX, itr) < 0 {
                error_errno_message(Some("Failed to build overlapping regions list".to_string()));
            }

            while regidx::regidx_c_612_regitr_overlap(itr) != 0 {
                regs.push(RegionBytes::from_interval(seq, (*itr).beg, (*itr).end));
            }
        }
        regidx::regidx_c_606_regitr_destroy(itr);
        regidx::regidx_c_311_regidx_destroy(idx);
    }

    if regs.is_empty() && argc == 0 {
        regs.push(RegionBytes::from_bytes(b"."));
    }

    let mut iseq = 0;
    while iseq < argc {
        let arg = *argv.add(iseq as usize);
        if arg.is_null() {
            return None;
        }
        regs.push(RegionBytes::from_cstr(arg));
        iseq += 1;
    }
    Some(regs)
}

// original: query_regions (htslib/tabix.c:206)
unsafe fn tabix_c_206_query_regions(
    args: &mut args_t,
    conf: &mut tbx::tbx_conf_t,
    fname: *const c_char,
    regs: &[RegionBytes],
) -> c_int {
    let tpool_owner = if args.threads >= 1 {
        OwnedThreadPool::new(args.threads)
    } else {
        None
    };
    let mut tpool = htsThreadPool {
        pool: tpool_owner
            .as_ref()
            .map_or(ptr::null_mut(), OwnedThreadPool::as_ptr),
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
    if args.cache_megs != 0 {
        hts::hts_set_cache_size(fp, args.cache_megs * 1048576);
    }

    //set threads if needed, errors are logged and ignored
    if tpool_owner.is_some() {
        if hts::hts_set_thread_pool(fp, &mut tpool) < 0 {
            error_errno_message(Some(
                "Threaded BGZF is not yet supported in this translation".to_string(),
            ));
        }
    }

    let reg_idx = if !args.targets_fname.is_null() {
        let reg_idx = regidx::regidx_c_246_regidx_init(
            args.targets_fname.cast_mut(),
            None,
            None,
            0,
            ptr::null_mut(),
        );
        if reg_idx.is_null() {
            error_errno_message(Some(format!(
                "Could not build region list for \"{}\"",
                cstr_to_string(args.targets_fname)
            )));
        }
        NonNull::new(reg_idx)
    } else {
        None
    };

    if format == HTS_FORMAT_BCF {
        let out = hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
        if out.is_null() {
            error_errno_message(Some("Could not open stdout".to_string()));
        }
        if tpool_owner.is_some() && hts::hts_set_thread_pool(out, &mut tpool) < 0 {
            error_errno_message(Some(
                "Threaded BGZF is not yet supported in this translation".to_string(),
            ));
        }
        let idx = vcf::bcf_index_load3(
            fname,
            ptr::null(),
            if args.download_index != 0 {
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

        let hdr = vcf::bcf_hdr_read(fp);
        if hdr.is_null() {
            error_errno_message(Some(format!(
                "Could not read the header from \"{}\"",
                cstr_to_string(fname)
            )));
        }

        if args.print_header != 0 && vcf::bcf_hdr_write(out, hdr) != 0 {
            error_errno_message(Some("Failed to write to stdout".to_string()));
        }
        if args.header_only == 0 {
            let rec = vcf::bcf_init();
            if rec.is_null() {
                error_errno_message(None);
            }
            for reg in regs {
                let c_reg = reg.to_c_terminated_bytes();
                let reg_ptr = c_reg.as_ptr().cast::<c_char>();
                let mut found = 0;
                let itr = bcf_itr_querys1(idx, hdr, reg_ptr);
                if itr.is_null() {
                    continue;
                }
                let mut ret = bcf_itr_next(fp, itr, rec);
                while ret >= 0 {
                    if let Some(reg_idx) = reg_idx {
                        let chr = vcf::bcf_seqname(hdr, rec);
                        if chr.is_null() {
                            error_message(format!(
                                "Bad BCF record in \"{}\" : Invalid CONTIG id {}\n",
                                cstr_to_string(fname),
                                (*rec).rid
                            ));
                        }
                        if regidx::regidx_c_401_regidx_overlap(
                            reg_idx.as_ptr(),
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
                        if args.separate_regs != 0 {
                            libc::printf(c"%c%s\n".as_ptr(), conf.meta_char, reg_ptr);
                        }
                        found = 1;
                    }
                    if vcf::bcf_write(out, hdr, rec) != 0 {
                        error_errno_message(Some("Failed to write to stdout".to_string()));
                    }
                    ret = bcf_itr_next(fp, itr, rec);
                }
                if ret < -1 {
                    error_errno_message(Some(format!(
                        "Reading \"{}\" failed",
                        cstr_to_string(fname)
                    )));
                }
                hts::hts_itr_destroy(itr);
            }
            vcf::bcf_destroy(rec);
        }
        if hts::hts_close(out) != 0 {
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
            if args.download_index != 0 {
                HTS_IDX_SAVE_REMOTE
            } else {
                0
            },
        );
        if tbx_.is_null() {
            error_errno_message(Some(format!(
                "Could not load .tbi/.csi index of {}",
                cstr_to_string(fname)
            )));
        }
        let mut str_: kstring_t = std::mem::zeroed();
        if args.print_header != 0 {
            let mut ret = hts::hts_getline(fp, 2, &mut str_);
            while ret >= 0 {
                if str_.l == 0 || *str_.s != (*tbx_).conf.meta_char as c_char {
                    break;
                }
                if libc::puts(str_.s) < 0 {
                    error_errno_message(Some("Error writing to stdout".to_string()));
                }
                ret = hts::hts_getline(fp, 2, &mut str_);
            }
            if ret < -1 {
                error_errno_message(Some(format!(
                    "Reading \"{}\" failed",
                    cstr_to_string(fname)
                )));
            }
        }
        if args.header_only == 0 {
            let seqs = if reg_idx.is_some() {
                let seqs = OwnedConstCharPtrArray::from_tbx(&*tbx_);
                if seqs.is_none() {
                    error_errno_message(Some("Failed to get sequence names list".to_string()));
                }
                seqs
            } else {
                None
            };
            for reg in regs {
                let c_reg = reg.to_c_terminated_bytes();
                let reg_ptr = c_reg.as_ptr().cast::<c_char>();
                let mut found = 0;
                let itr = tbx_itr_querys1(tbx_, reg_ptr);
                if itr.is_null() {
                    continue;
                }
                let mut ret = tbx_itr_next(fp, tbx_, itr, &mut str_);
                while ret >= 0 {
                    if let (Some(reg_idx), Some(seqs)) = (reg_idx, seqs.as_ref()) {
                        if let Some(&seq) = seqs.as_slice().get((*itr).curr_tid as usize) {
                            if regidx::regidx_c_401_regidx_overlap(
                                reg_idx.as_ptr(),
                                seq,
                                (*itr).curr_beg,
                                (*itr).curr_end - 1,
                                ptr::null_mut(),
                            ) == 0
                            {
                                ret = tbx_itr_next(fp, tbx_, itr, &mut str_);
                                continue;
                            }
                        }
                    }
                    if found == 0 {
                        if args.separate_regs != 0 {
                            libc::printf(c"%c%s\n".as_ptr(), conf.meta_char, reg_ptr);
                        }
                        found = 1;
                    }
                    if libc::puts(str_.s) < 0 {
                        error_errno_message(Some("Failed to write to stdout".to_string()));
                    }
                    ret = tbx_itr_next(fp, tbx_, itr, &mut str_);
                }
                if ret < -1 {
                    error_errno_message(Some(format!(
                        "Reading \"{}\" failed",
                        cstr_to_string(fname)
                    )));
                }
                hts::hts_itr_destroy(itr);
            }
        }
        libc::free(str_.s.cast());
        tbx::tbx_destroy(tbx_);
    } else if format == HTS_FORMAT_BAM {
        error_message("Please use \"samtools view\" for querying BAM files.\n".to_string());
    }

    if let Some(reg_idx) = reg_idx {
        regidx::regidx_c_311_regidx_destroy(reg_idx.as_ptr());
    }
    if hts::hts_close(fp) != 0 {
        error_errno_message(Some(format!(
            "hts_close returned non-zero status: {}",
            cstr_to_string(fname)
        )));
    }

    0
}

// original: query_chroms (htslib/tabix.c:396)
pub unsafe fn tabix_c_396_query_chroms(fname: *const c_char, download: c_int) -> c_int {
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
        let seqs = OwnedConstCharPtrArray::from_tbx(&*tbx_);
        let Some(seqs) = seqs else {
            error_errno_message(Some("Couldn't get list of sequence names".to_string()));
        };
        for &seq in seqs.as_slice() {
            if libc::printf(c"%s\n".as_ptr(), seq) < 0 {
                error_errno_message(Some("Couldn't write to stdout".to_string()));
            }
        }
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
        let Some(hdr_ref) = hdr.as_mut() else {
            error_errno_message(Some("Could not read the header".to_string()));
        };
        let idx_ref = &*idx;
        let seqs =
            OwnedConstCharPtrArray::from_hts_idx(idx_ref, hdr_ref, Some(bcf_hdr_id2name_adapter));
        let Some(seqs) = seqs else {
            error_errno_message(Some("Couldn't get list of sequence names".to_string()));
        };
        for &seq in seqs.as_slice() {
            if libc::printf(c"%s\n".as_ptr(), seq) < 0 {
                error_errno_message(Some("Couldn't write to stdout".to_string()));
            }
        }
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
    conf: &mut tbx::tbx_conf_t,
    threads: c_int,
) -> c_int {
    let tpool = if threads >= 1 {
        OwnedThreadPool::new(threads)
    } else {
        None
    };
    if (ftype & IS_TXT) != 0 || ftype == 0 {
        let fp = bgzf::bgzf_open(fname, c"r".as_ptr());
        if fp.is_null() {
            return -1;
        }
        if let Some(tpool) = tpool.as_ref() {
            if bgzf::bgzf_thread_pool(fp, tpool.as_ptr(), 0) < 0 {
                bgzf::bgzf_close(fp);
                return -1;
            }
        }
        if bgzf::bgzf_c_1485_bgzf_mt_read_block(fp) != 0 || (*fp).block_length == 0 {
            return -1;
        }

        let buffer = (*fp).uncompressed_block.cast::<c_char>();
        let mut skip_until = 0;

        // Skip the header: find out the position of the data block
        if *buffer == conf.meta_char as c_char {
            skip_until = 1;
            loop {
                if *buffer.add(skip_until as usize) == b'\n' as c_char {
                    skip_until += 1;
                    if skip_until >= (*fp).block_length {
                        if bgzf::bgzf_c_1485_bgzf_mt_read_block(fp) != 0 || (*fp).block_length == 0
                        {
                            error_message(format!(
                                "FIXME: No body in the file: {}\n",
                                cstr_to_string(fname)
                            ));
                        }
                        skip_until = 0;
                    }
                    // The header has finished
                    if *buffer.add(skip_until as usize) != conf.meta_char as c_char {
                        break;
                    }
                }
                skip_until += 1;
                if skip_until >= (*fp).block_length {
                    if bgzf::bgzf_c_1485_bgzf_mt_read_block(fp) != 0 || (*fp).block_length == 0 {
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
            error_message(format!(
                "{}: {}",
                cstr_to_string(header),
                cstr_to_string(libc::strerror(
                    *crate::htslib_rs::c_compat::__errno_location()
                ))
            ));
        }
        let page_size = 32768usize;
        let mut buf = vec![0u8; page_size];
        let bgzf_out = bgzf::bgzf_open(c"-".as_ptr(), c"w".as_ptr());

        if bgzf_out.is_null() {
            error_errno_message(Some("Couldn't open output stream".to_string()));
        }
        if let Some(tpool) = tpool.as_ref() {
            if bgzf::bgzf_thread_pool(bgzf_out, tpool.as_ptr(), 0) < 0 {
                bgzf::bgzf_close(fp);
                bgzf::bgzf_close(bgzf_out);
                return -1;
            }
        }
        let mut nread = libc::fread(buf.as_mut_ptr().cast(), 1, page_size - 1, hdr) as isize;
        while nread > 0 {
            if nread < (page_size - 1) as isize && buf[(nread - 1) as usize] != b'\n' {
                buf[nread as usize] = b'\n';
                nread += 1;
            }
            if bgzf::bgzf_write(bgzf_out, buf.as_ptr().cast(), nread as usize) < 0 {
                error_errno_message(Some(format!("Write error {}", (*bgzf_out).bitfields >> 16)));
            }
            nread = libc::fread(buf.as_mut_ptr().cast(), 1, page_size - 1, hdr) as isize;
        }
        if libc::ferror(hdr) != 0 {
            error_errno_message(Some(format!(
                "Failed to read \"{}\"",
                cstr_to_string(header)
            )));
        }
        if libc::fclose(hdr) != 0 {
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
            error_errno_message(Some(format!("Write error {}", (*fp).bitfields >> 16)));
        }
        if bgzf::bgzf_flush(bgzf_out) < 0 {
            error_errno_message(Some(format!("Write error {}", (*bgzf_out).bitfields >> 16)));
        }

        loop {
            nread = bgzf::bgzf_raw_read(fp, buf.as_mut_ptr().cast(), page_size);
            if nread <= 0 {
                break;
            }

            let count = bgzf::bgzf_raw_write(bgzf_out, buf.as_ptr().cast(), nread as usize);
            if count != nread {
                error_errno_message(Some(format!(
                    "Write failed, wrote {} instead of {} bytes",
                    count, nread as c_int
                )));
            }
        }
        if nread < 0 {
            error_errno_message(Some(format!("Error reading \"{}\"", cstr_to_string(fname))));
        }
        if bgzf::bgzf_close(bgzf_out) < 0 {
            error_errno_message(Some(format!(
                "Error {} closing output",
                (*bgzf_out).bitfields >> 16
            )));
        }
        if bgzf::bgzf_close(fp) < 0 {
            error_errno_message(Some(format!(
                "Error {} closing \"{}\"",
                (*bgzf_out).bitfields >> 16,
                cstr_to_string(fname)
            )));
        }
    } else {
        error_message("todo: reheader BCF, BAM\n".to_string());
    }
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
    let mut reheader: *const c_char = ptr::null();
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
                } else if libc::strcmp(optarg, c"bcf".as_ptr()) == 0
                    || libc::strcmp(optarg, c"bam".as_ptr()) == 0
                {
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
            2 => {
                return tabix_c_580_usage(
                    crate::htslib_rs::c_compat::stdout.cast(),
                    libc::EXIT_SUCCESS,
                );
            }
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
            _ => {
                return tabix_c_580_usage(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    libc::EXIT_FAILURE,
                );
            }
        }
    }

    if new_line_skip >= 0 {
        conf.line_skip = new_line_skip;
    }

    if optind == argc {
        return tabix_c_580_usage(
            crate::htslib_rs::c_compat::stderr.cast(),
            libc::EXIT_FAILURE,
        );
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
        let regs = if args.header_only == 0 {
            tabix_parse_regions_owned(
                args.regions_fname,
                argv.add((optind + 1) as usize),
                argc - optind - 1,
            )
            .unwrap_or_default()
        } else {
            Vec::new()
        };
        return tabix_c_206_query_regions(&mut args, &mut conf, fname, &regs);
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

    let mut idx_bytes = CStr::from_ptr(fname).to_bytes().to_vec();
    idx_bytes.extend_from_slice(CStr::from_ptr(suffix).to_bytes());
    idx_bytes.push(0);

    let mut stat_tbi: libc::stat = std::mem::zeroed();
    let mut stat_file: libc::stat = std::mem::zeroed();
    if is_force == 0 && libc::stat(idx_bytes.as_ptr().cast(), &mut stat_tbi) == 0 {
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

    let ret;
    if ftype == IS_CRAM {
        if sam::sam_index_build3(fname, ptr::null(), min_shift, args.threads) != 0 {
            error_message(format!(
                "bam_index_build failed: {}\n",
                cstr_to_string(fname)
            ));
        }
        0
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
            0 => 0,
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
            0 => 0,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstr_ascii_suffix_match_preserves_strcasecmp_behavior() {
        unsafe {
            assert!(cstr_ascii_ends_with(c"sample.VCF.GZ".as_ptr(), b".vcf.gz"));
            assert!(cstr_ascii_ends_with(c"sample.BAM".as_ptr(), b".bam"));
            assert!(!cstr_ascii_ends_with(c"sample.bam".as_ptr(), b".bcf"));
            assert!(!cstr_ascii_ends_with(c".gz".as_ptr(), b".vcf.gz"));
        }
    }

    #[test]
    fn file_type_uses_ascii_suffixes_before_format_sniffing() {
        unsafe {
            assert_eq!(tabix_c_102_file_type(c"sample.GFF.GZ".as_ptr()), IS_GFF);
            assert_eq!(tabix_c_102_file_type(c"sample.BED.GZ".as_ptr()), IS_BED);
            assert_eq!(tabix_c_102_file_type(c"sample.SAM.GZ".as_ptr()), IS_SAM);
            assert_eq!(tabix_c_102_file_type(c"sample.VCF.GZ".as_ptr()), IS_VCF);
            assert_eq!(tabix_c_102_file_type(c"sample.BCF".as_ptr()), IS_BCF);
            assert_eq!(tabix_c_102_file_type(c"sample.BAM".as_ptr()), IS_BAM);
            assert_eq!(tabix_c_102_file_type(c"sample.CRAM".as_ptr()), IS_CRAM);
            assert_eq!(tabix_c_102_file_type(c"sample.GAF.GZ".as_ptr()), IS_GAF);
        }
    }
}
