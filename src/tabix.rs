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
use std::ffi::{c_char, c_int, c_void};
use std::io::Write;
use std::ptr::{self, NonNull};

const NO_ARGUMENT: i32 = 0;
const REQUIRED_ARGUMENT: i32 = 1;
const IS_GFF: i32 = 1 << 0;
const IS_BED: i32 = 1 << 1;
const IS_SAM: i32 = 1 << 2;
const IS_VCF: i32 = 1 << 3;
const IS_BCF: i32 = 1 << 4;
const IS_BAM: i32 = 1 << 5;
const IS_CRAM: i32 = 1 << 6;
const IS_GAF: i32 = 1 << 7;
const IS_TXT: i32 = IS_GFF | IS_BED | IS_SAM | IS_VCF;
const TBX_UCSC: i32 = 0x10000;
const TBX_GAF: i32 = 3;

#[repr(C)]
struct GetoptLongOption {
    name: *const c_char,
    has_arg: i32,
    flag: *mut i32,
    val: i32,
}

unsafe extern "C" {
    fn getopt_long(
        argc: i32,
        argv: *mut *mut c_char,
        optstring: *const c_char,
        longopts: *const GetoptLongOption,
        longindex: *mut i32,
    ) -> i32;
    static mut optarg: *mut c_char;
    static mut optind: i32;
}

pub struct args_t {
    pub regions_fname: Option<Vec<u8>>,
    pub targets_fname: Option<Vec<u8>>,
    pub print_header: i32,
    pub header_only: i32,
    pub cache_megs: i32,
    pub download_index: i32,
    pub separate_regs: i32,
    pub threads: i32,
}

fn bytes_to_string(s: &[u8]) -> String {
    String::from_utf8_lossy(s).into_owned()
}

fn error_message(message: String) -> ! {
    let _ = std::io::stdout().flush();
    eprint!("{message}");
    let _ = std::io::stderr().flush();
    std::process::exit(1);
}

fn error_errno_message(message: Option<String>) -> ! {
    let eno = std::io::Error::last_os_error();
    let raw = eno.raw_os_error().unwrap_or(0);
    let _ = std::io::stdout().flush();
    if let Some(message) = message.as_ref() {
        eprint!("{message}");
    }
    if raw != 0 {
        if message.is_some() {
            eprint!(": ");
        }
        eprintln!("{eno}");
    } else {
        eprintln!();
    }
    let _ = std::io::stderr().flush();
    std::process::exit(1);
}

struct OwnedThreadPool {
    pool: NonNull<thread_pool::hts_tpool>,
}

impl OwnedThreadPool {
    unsafe fn new(threads: i32) -> Option<Self> {
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

struct RegionBytes {
    bytes: Vec<u8>,
}

impl RegionBytes {
    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
        }
    }

    fn from_c_bytes(region: &[u8]) -> Self {
        Self::from_bytes(region)
    }
}

// These adapters must match the C-ABI callback typedefs (hts_name2id_f,
// hts_id2name_f, hts_readrec_func, hts_itr_query_func) defined in `hts`, so
// they keep the opaque `*mut c_void` / `c_int` ABI shape at this boundary.
unsafe extern "C" fn tbx_name2id_adapter(data: *mut c_void, name: *const c_char) -> c_int {
    let Some(tbx) = data.cast::<tbx::tbx_t>().as_ref() else {
        return -1;
    };
    tbx::tbx_name2id(tbx, std::ffi::CStr::from_ptr(name).to_bytes())
}

unsafe extern "C" fn tbx_readrec_adapter(
    fp: *mut BGZF,
    data: *mut c_void,
    r: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    tbx::tbx_readrec(fp, data.cast(), r.cast(), tid, beg, end)
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
    region: &[u8],
) -> *mut hts_itr_t {
    let mut region = region.to_vec();
    region.push(0);
    hts::hts_itr_querys(
        idx,
        region.as_ptr().cast(),
        Some(bcf_hdr_name2id_adapter),
        hdr.cast(),
        Some(hts_itr_query_adapter),
        Some(bcf_readrec_adapter),
    )
}

unsafe fn tbx_itr_querys1(tbx_: *mut tbx::tbx_t, region: &[u8]) -> *mut hts_itr_t {
    tbx::tbx_itr_querys1(tbx_, region)
}

unsafe fn bcf_itr_next(htsfp: *mut htsFile, itr: *mut hts_itr_t, r: *mut vcf::bcf1_t) -> i32 {
    if ((*htsfp).bitfields & (1 << 4)) != 0 {
        return hts::hts_itr_next(
            (*htsfp).fp.bgzf.cast(),
            itr.cast(),
            r.cast(),
            ptr::null_mut(),
        );
    }
    -2
}

unsafe fn tbx_itr_next(
    fp: *mut htsFile,
    tbx_: *mut tbx::tbx_t,
    itr: *mut hts_itr_t,
    str_: &mut kstring_t,
) -> i32 {
    hts::hts_itr_next(
        (*fp).fp.bgzf.cast(),
        itr,
        (str_ as *mut kstring_t).cast(),
        tbx_.cast(),
    )
}

fn tbx_conf_gaf() -> tbx::tbx_conf_t {
    tbx::tbx_conf_t {
        preset: TBX_GAF,
        sc: 1,
        bc: 6,
        ec: 0,
        meta_char: b'#' as i32,
        line_skip: 0,
    }
}

fn bytes_ascii_ends_with(bytes: &[u8], suffix: &[u8]) -> bool {
    bytes.len() >= suffix.len() && bytes[bytes.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

// original: file_type (htslib/tabix.c:102)
pub unsafe fn tabix_c_102_file_type(fname: &[u8]) -> i32 {
    if bytes_ascii_ends_with(fname, b".gff.gz") {
        return IS_GFF;
    } else if bytes_ascii_ends_with(fname, b".bed.gz") {
        return IS_BED;
    } else if bytes_ascii_ends_with(fname, b".sam.gz") {
        return IS_SAM;
    } else if bytes_ascii_ends_with(fname, b".vcf.gz") {
        return IS_VCF;
    } else if bytes_ascii_ends_with(fname, b".bcf") {
        return IS_BCF;
    } else if bytes_ascii_ends_with(fname, b".bam") {
        return IS_BAM;
    } else if bytes_ascii_ends_with(fname, b".cram") {
        return IS_CRAM;
    } else if bytes_ascii_ends_with(fname, b".gaf.gz") {
        return IS_GAF;
    }

    let mut fname_c = fname.to_vec();
    fname_c.push(0);
    let fp = hts::hts_open(fname_c.as_ptr().cast(), c"r".as_ptr());
    if fp.is_null() {
        if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOEXEC) {
            error_message(format!(
                "Couldn't understand format of \"{}\"\n",
                bytes_to_string(fname)
            ));
        } else {
            error_errno_message(Some(format!("Couldn't open \"{}\"", bytes_to_string(fname))));
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
    regions_fname: Option<&[u8]>,
    args: &[Vec<u8>],
) -> Option<Vec<Vec<u8>>> {
    let regions = tabix_parse_regions_owned(regions_fname, args)?;
    let mut out = Vec::with_capacity(regions.len());
    for region in regions {
        if region.bytes.contains(&0) {
            error_message("region contains an interior NUL\n".to_string());
        }
        out.push(region.bytes);
    }
    Some(out)
}

unsafe fn tabix_parse_regions_owned(
    regions_fname: Option<&[u8]>,
    args: &[Vec<u8>],
) -> Option<Vec<RegionBytes>> {
    let argc = args.len();
    let mut regs = Vec::<RegionBytes>::new();
    if argc > 0 {
        regs.try_reserve(argc).ok()?;
    }

    if let Some(regions_fname) = regions_fname {
        // improve me: this is a too heavy machinery for parsing regions...

        let Some(mut idx) = regidx::regidx_c_246_regidx_init(
            Some(regions_fname),
            None,
            None,
            0,
            None,
        ) else {
            error_errno_message(Some(format!(
                "Could not build region list for \"{}\"",
                bytes_to_string(regions_fname)
            )));
        };
        let mut itr = regidx::regidx_c_584_regitr_init(&mut idx);

        let extra = regidx::regidx_c_98_regidx_nregs(&idx).max(0) as usize;
        if regs.try_reserve(extra).is_err() {
            error_errno_message(None);
        }

        let seqs: Vec<Vec<u8>> = regidx::regidx_c_105_regidx_seq_names(&idx).to_vec();
        for seq in &seqs {
            if regidx::regidx_c_401_regidx_overlap(
                &mut idx,
                seq,
                0,
                HTS_POS_MAX,
                Some(&mut itr),
            ) < 0
            {
                error_errno_message(Some("Failed to build overlapping regions list".to_string()));
            }

            while regidx::regidx_c_612_regitr_overlap(&mut itr) != 0 {
                let mut bytes = Vec::with_capacity(seq.len() + 32);
                bytes.extend_from_slice(seq);
                write!(&mut bytes, ":{}-{}", itr.beg + 1, itr.end + 1).unwrap();
                regs.push(RegionBytes { bytes });
            }
        }
        regidx::regidx_c_606_regitr_destroy(itr);
        regidx::regidx_c_311_regidx_destroy(idx);
    }

    if regs.is_empty() && argc == 0 {
        regs.push(RegionBytes::from_bytes(b"."));
    }

    for arg in args {
        regs.push(RegionBytes::from_c_bytes(arg));
    }
    Some(regs)
}

// original: query_regions (htslib/tabix.c:206)
unsafe fn tabix_c_206_query_regions(
    args: &mut args_t,
    conf: &mut tbx::tbx_conf_t,
    fname: &[u8],
    regs: &[RegionBytes],
) -> i32 {
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
    let mut fname_c = fname.to_vec();
    fname_c.push(0);
    let fname_ptr = fname_c.as_ptr().cast::<c_char>();
    let fp = hts::hts_open(fname_ptr, c"r".as_ptr());
    if fp.is_null() {
        error_errno_message(Some(format!(
            "Could not open \"{}\"",
            bytes_to_string(fname)
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

    let mut reg_idx = if let Some(targets_fname) = args.targets_fname.as_deref() {
        let reg_idx = regidx::regidx_c_246_regidx_init(
            Some(targets_fname),
            None,
            None,
            0,
            None,
        );
        if reg_idx.is_none() {
            error_errno_message(Some(format!(
                "Could not build region list for \"{}\"",
                bytes_to_string(targets_fname)
            )));
        }
        reg_idx
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
            fname_ptr,
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
                bytes_to_string(fname)
            )));
        }

        let hdr = vcf::bcf_hdr_read(fp);
        if hdr.is_null() {
            error_errno_message(Some(format!(
                "Could not read the header from \"{}\"",
                bytes_to_string(fname)
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
                let mut found = 0;
                let itr = bcf_itr_querys1(idx, hdr, &reg.bytes);
                if itr.is_null() {
                    continue;
                }
                let mut ret = bcf_itr_next(fp, itr, rec);
                while ret >= 0 {
                    if let Some(reg_idx) = reg_idx.as_deref_mut() {
                        let chr = vcf::bcf_seqname(hdr, rec);
                        if chr.is_null() {
                            error_message(format!(
                                "Bad BCF record in \"{}\" : Invalid CONTIG id {}\n",
                                bytes_to_string(fname),
                                (*rec).rid
                            ));
                        }
                        if regidx::regidx_c_401_regidx_overlap(
                            reg_idx,
                            std::ffi::CStr::from_ptr(chr).to_bytes(),
                            (*rec).pos,
                            (*rec).pos + (*rec).rlen as hts_pos_t - 1,
                            None,
                        ) == 0
                        {
                            ret = bcf_itr_next(fp, itr, rec);
                            continue;
                        }
                    }
                    if found == 0 {
                        if args.separate_regs != 0 {
                            let mut line = vec![conf.meta_char as u8];
                            line.extend_from_slice(&reg.bytes);
                            line.push(b'\n');
                            std::io::stdout().write_all(&line).unwrap();
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
                        bytes_to_string(fname)
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
            None,
            if args.download_index != 0 {
                HTS_IDX_SAVE_REMOTE
            } else {
                0
            },
        );
        if tbx_.is_null() {
            error_errno_message(Some(format!(
                "Could not load .tbi/.csi index of {}",
                bytes_to_string(fname)
            )));
        }
        let mut str_: kstring_t = kstring_t::default();
        if args.print_header != 0 {
            let mut ret = hts::hts_getline(fp, 2, &mut str_);
            while ret >= 0 {
                if str_.data.is_empty() || str_.data[0] != (*tbx_).conf.meta_char as u8 {
                    break;
                }
                let line = &str_.data[..];
                let mut out = std::io::stdout();
                if out.write_all(line).and_then(|()| out.write_all(b"\n")).is_err() {
                    error_errno_message(Some("Error writing to stdout".to_string()));
                }
                ret = hts::hts_getline(fp, 2, &mut str_);
            }
            if ret < -1 {
                error_errno_message(Some(format!(
                    "Reading \"{}\" failed",
                    bytes_to_string(fname)
                )));
            }
        }
        if args.header_only == 0 {
            let seqs: Option<Vec<Vec<u8>>> = if reg_idx.is_some() {
                Some(tbx::tbx_seqnames(tbx_))
            } else {
                None
            };
            for reg in regs {
                let mut found = 0;
                let itr = tbx_itr_querys1(tbx_, &reg.bytes);
                if itr.is_null() {
                    continue;
                }
                let mut ret = tbx_itr_next(fp, tbx_, itr, &mut str_);
                while ret >= 0 {
                    if let (Some(reg_idx), Some(seqs)) = (reg_idx.as_deref_mut(), seqs.as_ref()) {
                        if let Some(seq) = seqs.get((*itr).curr_tid as usize) {
                            if regidx::regidx_c_401_regidx_overlap(
                                reg_idx,
                                seq,
                                (*itr).curr_beg,
                                (*itr).curr_end - 1,
                                None,
                            ) == 0
                            {
                                ret = tbx_itr_next(fp, tbx_, itr, &mut str_);
                                continue;
                            }
                        }
                    }
                    if found == 0 {
                        if args.separate_regs != 0 {
                            let mut line = vec![conf.meta_char as u8];
                            line.extend_from_slice(&reg.bytes);
                            line.push(b'\n');
                            std::io::stdout().write_all(&line).unwrap();
                        }
                        found = 1;
                    }
                    let line = &str_.data[..];
                    let mut out = std::io::stdout();
                    if out.write_all(line).and_then(|()| out.write_all(b"\n")).is_err() {
                        error_errno_message(Some("Failed to write to stdout".to_string()));
                    }
                    ret = tbx_itr_next(fp, tbx_, itr, &mut str_);
                }
                if ret < -1 {
                    error_errno_message(Some(format!(
                        "Reading \"{}\" failed",
                        bytes_to_string(fname)
                    )));
                }
                hts::hts_itr_destroy(itr);
            }
        }
        drop(str_);
        tbx::tbx_destroy(tbx_);
    } else if format == HTS_FORMAT_BAM {
        error_message("Please use \"samtools view\" for querying BAM files.\n".to_string());
    }

    if let Some(reg_idx) = reg_idx {
        regidx::regidx_c_311_regidx_destroy(reg_idx);
    }
    if hts::hts_close(fp) != 0 {
        error_errno_message(Some(format!(
            "hts_close returned non-zero status: {}",
            bytes_to_string(fname)
        )));
    }

    0
}

// original: query_chroms (htslib/tabix.c:396)
pub unsafe fn tabix_c_396_query_chroms(fname: &[u8], download: i32) -> i32 {
    let mut fname_c = fname.to_vec();
    fname_c.push(0);
    let fname_ptr = fname_c.as_ptr().cast::<c_char>();
    let ftype = tabix_c_102_file_type(fname);
    if (ftype & IS_TXT) != 0 || ftype == 0 {
        let tbx_ = tbx::tbx_index_load3(
            fname,
            None,
            if download != 0 {
                HTS_IDX_SAVE_REMOTE
            } else {
                0
            },
        );
        if tbx_.is_null() {
            error_errno_message(Some(format!(
                "Could not load .tbi index of {}",
                bytes_to_string(fname)
            )));
        }
        for name in tbx::tbx_seqnames(tbx_) {
            let mut out = std::io::stdout();
            if out.write_all(&name).and_then(|()| out.write_all(b"\n")).is_err() {
                error_errno_message(Some("Couldn't write to stdout".to_string()));
            }
        }
        tbx::tbx_destroy(tbx_);
    } else if ftype == IS_BCF {
        let fp = hts::hts_open(fname_ptr, c"r".as_ptr());
        if fp.is_null() {
            error_errno_message(Some(format!(
                "Could not open \"{}\"",
                bytes_to_string(fname)
            )));
        }
        let hdr = vcf::bcf_hdr_read(fp);
        if hdr.is_null() {
            error_errno_message(Some(format!(
                "Could not read the header: \"{}\"",
                bytes_to_string(fname)
            )));
        }
        hts::hts_close(fp);
        let idx = vcf::bcf_index_load3(
            fname_ptr,
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
                bytes_to_string(fname)
            )));
        }
        if hdr.as_mut().is_none() {
            error_errno_message(Some("Could not read the header".to_string()));
        }
        // hts_idx_seqnames returns a C-allocated array of pointers into
        // header-owned strings; copy the names out and free that array.
        let mut n = 0;
        let names_ptr = hts::hts_idx_seqnames(
            &*idx,
            &mut n,
            Some(bcf_hdr_id2name_adapter),
            hdr.cast(),
        );
        if names_ptr.is_null() {
            error_errno_message(Some("Couldn't get list of sequence names".to_string()));
        }
        let names = std::slice::from_raw_parts(names_ptr, n.max(0) as usize);
        let seqs: Vec<Vec<u8>> = names
            .iter()
            .map(|&p| std::ffi::CStr::from_ptr(p).to_bytes().to_vec())
            .collect();
        libc::free(names_ptr.cast());
        for name in &seqs {
            let mut out = std::io::stdout();
            if out.write_all(name).and_then(|()| out.write_all(b"\n")).is_err() {
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
    fname: &[u8],
    header: &[u8],
    ftype: i32,
    conf: &mut tbx::tbx_conf_t,
    threads: i32,
) -> i32 {
    let mut fname_c = fname.to_vec();
    fname_c.push(0);
    let fname_ptr = fname_c.as_ptr().cast::<c_char>();
    let tpool = if threads >= 1 {
        OwnedThreadPool::new(threads)
    } else {
        None
    };
    if (ftype & IS_TXT) != 0 || ftype == 0 {
        let fp = bgzf::bgzf_open(fname_ptr, c"r".as_ptr());
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

        let meta = conf.meta_char as u8;
        let mut skip_until: i32 = 0;

        // Skip the header: find out the position of the data block
        if (&(*fp).uncompressed_block)[0] == meta {
            skip_until = 1;
            loop {
                if (&(*fp).uncompressed_block)[skip_until as usize] == b'\n' {
                    skip_until += 1;
                    if skip_until >= (*fp).block_length {
                        if bgzf::bgzf_c_1485_bgzf_mt_read_block(fp) != 0 || (*fp).block_length == 0
                        {
                            error_message(format!(
                                "FIXME: No body in the file: {}\n",
                                bytes_to_string(fname)
                            ));
                        }
                        skip_until = 0;
                    }
                    // The header has finished
                    if (&(*fp).uncompressed_block)[skip_until as usize] != meta {
                        break;
                    }
                }
                skip_until += 1;
                if skip_until >= (*fp).block_length {
                    if bgzf::bgzf_c_1485_bgzf_mt_read_block(fp) != 0 || (*fp).block_length == 0 {
                        error_message(format!(
                            "FIXME: No body in the file: {}\n",
                            bytes_to_string(fname)
                        ));
                    }
                    skip_until = 0;
                }
            }
        }

        // Output the new header
        use std::io::Read as _;
        use std::os::unix::ffi::OsStrExt as _;
        let header_path = std::ffi::OsStr::from_bytes(header).to_owned();
        let mut hdr = match std::fs::File::open(&header_path) {
            Ok(f) => f,
            Err(e) => error_message(format!("{}: {}", bytes_to_string(header), e)),
        };
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
        loop {
            let mut nread = match hdr.read(&mut buf[..page_size - 1]) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => error_errno_message(Some(format!(
                    "Failed to read \"{}\"",
                    bytes_to_string(header)
                ))),
            };
            if nread < page_size - 1 && buf[nread - 1] != b'\n' {
                buf[nread] = b'\n';
                nread += 1;
            }
            if bgzf::bgzf_write(bgzf_out, buf.as_ptr().cast(), nread) < 0 {
                error_errno_message(Some(format!("Write error {}", (*bgzf_out).bitfields >> 16)));
            }
        }
        drop(hdr);

        // Output all remaining data read with the header block
        let remaining = (*fp).block_length - skip_until;
        if remaining > 0
            && bgzf::bgzf_write(
                bgzf_out,
                (&(*fp).uncompressed_block)[skip_until as usize..].as_ptr().cast(),
                remaining as usize,
            ) < 0
        {
            error_errno_message(Some(format!("Write error {}", (*fp).bitfields >> 16)));
        }
        if bgzf::bgzf_flush(bgzf_out) < 0 {
            error_errno_message(Some(format!("Write error {}", (*bgzf_out).bitfields >> 16)));
        }

        loop {
            let nread = bgzf::bgzf_raw_read(fp, buf.as_mut_ptr().cast(), page_size);
            if nread == 0 {
                break;
            }
            if nread < 0 {
                error_errno_message(Some(format!("Error reading \"{}\"", bytes_to_string(fname))));
            }

            let count = bgzf::bgzf_raw_write(bgzf_out, buf.as_ptr().cast(), nread as usize);
            if count != nread {
                error_errno_message(Some(format!(
                    "Write failed, wrote {} instead of {} bytes",
                    count, nread as i32
                )));
            }
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
                bytes_to_string(fname)
            )));
        }
    } else {
        error_message("todo: reheader BCF, BAM\n".to_string());
    }
    0
}

// original: usage (htslib/tabix.c:580)
pub unsafe fn tabix_c_580_usage(to_stdout: bool, status: i32) -> i32 {
    let version =
        String::from_utf8_lossy(std::ffi::CStr::from_ptr(crate::htslib_rs::hts::hts_version()).to_bytes())
            .into_owned();
    let text = format!(
        "\n\
Version: {version}\n\
Usage:   tabix [OPTIONS] [FILE] [REGION [...]]\n\
\n\
Indexing Options:\n\
   -0, --zero-based           coordinates are zero-based\n\
   -b, --begin INT            column number for region start [4]\n\
   -c, --comment CHAR         skip comment lines starting with CHAR [null]\n\
   -C, --csi                  generate CSI index for VCF (default is TBI)\n\
   -e, --end INT              column number for region end (if no end, set INT to -b) [5]\n\
   -f, --force                overwrite existing index without asking\n\
   -m, --min-shift INT        set minimal interval size for CSI indices to 2^INT [14]\n\
   -p, --preset STR           gff, bed, sam, vcf, gaf\n\
   -s, --sequence INT         column number for sequence names (suppressed by -p) [1]\n\
   -S, --skip-lines INT       skip first INT lines [0]\n\
\n\
Querying and other options:\n\
   -h, --print-header         print also the header lines\n\
   -H, --only-header          print only the header lines\n\
   -l, --list-chroms          list chromosome names\n\
   -r, --reheader FILE        replace the header with the content of FILE\n\
   -R, --regions FILE         restrict to regions listed in the file\n\
   -T, --targets FILE         similar to -R but streams rather than index-jumps\n\
   -D                         do not download the index file\n\
       --cache INT            set cache size to INT megabytes (0 disables) [10]\n\
       --separate-regions     separate the output by corresponding regions\n\
       --verbosity INT        set verbosity [3]\n\
   -@, --threads INT          number of additional threads to use [0]\n\
\n"
    );
    if to_stdout {
        print!("{text}");
    } else {
        eprint!("{text}");
    }
    status
}

// original: main (htslib/tabix.c:614)
pub unsafe fn tabix_c_614_main(argc: i32, argv: *mut *mut c_char) -> i32 {
    let mut detect = 1;
    let mut min_shift = 0;
    let mut is_force = 0;
    let mut list_chroms = 0;
    let mut do_csi = 0;
    let mut conf = tbx::tbx_conf_gff();
    let mut reheader: Option<Vec<u8>> = None;
    let mut args = args_t {
        regions_fname: None,
        targets_fname: None,
        print_header: 0,
        header_only: 0,
        cache_megs: 10,
        download_index: 1,
        separate_regs: 0,
        threads: 0,
    };
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
            val: b'R' as i32,
        },
        GetoptLongOption {
            name: c"targets".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'T' as i32,
        },
        GetoptLongOption {
            name: c"csi".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'C' as i32,
        },
        GetoptLongOption {
            name: c"zero-based".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'0' as i32,
        },
        GetoptLongOption {
            name: c"print-header".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'h' as i32,
        },
        GetoptLongOption {
            name: c"only-header".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'H' as i32,
        },
        GetoptLongOption {
            name: c"begin".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'b' as i32,
        },
        GetoptLongOption {
            name: c"comment".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'c' as i32,
        },
        GetoptLongOption {
            name: c"end".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'e' as i32,
        },
        GetoptLongOption {
            name: c"force".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'f' as i32,
        },
        GetoptLongOption {
            name: c"min-shift".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'm' as i32,
        },
        GetoptLongOption {
            name: c"preset".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'p' as i32,
        },
        GetoptLongOption {
            name: c"sequence".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b's' as i32,
        },
        GetoptLongOption {
            name: c"skip-lines".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'S' as i32,
        },
        GetoptLongOption {
            name: c"list-chroms".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'l' as i32,
        },
        GetoptLongOption {
            name: c"reheader".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'r' as i32,
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
            val: b'@' as i32,
        },
        GetoptLongOption {
            name: ptr::null(),
            has_arg: 0,
            flag: ptr::null_mut(),
            val: 0,
        },
    ];

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
            x if x == b'R' as i32 => {
                args.regions_fname = Some(std::ffi::CStr::from_ptr(optarg).to_bytes().to_vec())
            }
            x if x == b'T' as i32 => {
                args.targets_fname = Some(std::ffi::CStr::from_ptr(optarg).to_bytes().to_vec())
            }
            x if x == b'C' as i32 => do_csi = 1,
            x if x == b'r' as i32 => {
                reheader = Some(std::ffi::CStr::from_ptr(optarg).to_bytes().to_vec())
            }
            x if x == b'h' as i32 => args.print_header = 1,
            x if x == b'H' as i32 => {
                args.print_header = 1;
                args.header_only = 1;
            }
            x if x == b'l' as i32 => list_chroms = 1,
            x if x == b'0' as i32 => {
                conf.preset |= TBX_UCSC;
                detect = 0;
            }
            x if x == b'b' as i32 => {
                let s = std::str::from_utf8(std::ffi::CStr::from_ptr(optarg).to_bytes()).unwrap_or("\0");
                match s.trim_start().parse::<i64>() {
                    Ok(v) => conf.bc = v as i32,
                    Err(_) => error_message(format!(
                        "Could not parse argument: -b {}\n",
                        bytes_to_string(std::ffi::CStr::from_ptr(optarg).to_bytes())
                    )),
                }
                detect = 0;
            }
            x if x == b'e' as i32 => {
                let s = std::str::from_utf8(std::ffi::CStr::from_ptr(optarg).to_bytes()).unwrap_or("\0");
                match s.trim_start().parse::<i64>() {
                    Ok(v) => conf.ec = v as i32,
                    Err(_) => error_message(format!(
                        "Could not parse argument: -e {}\n",
                        bytes_to_string(std::ffi::CStr::from_ptr(optarg).to_bytes())
                    )),
                }
                detect = 0;
            }
            x if x == b'c' as i32 => {
                conf.meta_char = std::ffi::CStr::from_ptr(optarg).to_bytes().first().copied().unwrap_or(0)
                    as i32;
                detect = 0;
            }
            x if x == b'f' as i32 => is_force = 1,
            x if x == b'm' as i32 => {
                let s = std::str::from_utf8(std::ffi::CStr::from_ptr(optarg).to_bytes()).unwrap_or("\0");
                match s.trim_start().parse::<i64>() {
                    Ok(v) => min_shift = v as i32,
                    Err(_) => error_message(format!(
                        "Could not parse argument: -m {}\n",
                        bytes_to_string(std::ffi::CStr::from_ptr(optarg).to_bytes())
                    )),
                }
            }
            x if x == b'p' as i32 => {
                detect = 0;
                let preset = std::ffi::CStr::from_ptr(optarg).to_bytes();
                if preset == b"gff" {
                    conf = tbx::tbx_conf_gff();
                } else if preset == b"bed" {
                    conf = tbx::tbx_conf_bed();
                } else if preset == b"sam" {
                    conf = tbx::tbx_conf_sam();
                } else if preset == b"vcf" {
                    conf = tbx::tbx_conf_vcf();
                } else if preset == b"gaf" {
                    conf = tbx_conf_gaf();
                } else if preset == b"bcf" || preset == b"bam" {
                    detect = 1;
                } else {
                    error_message(format!(
                        "The preset string not recognised: '{}'\n",
                        bytes_to_string(std::ffi::CStr::from_ptr(optarg).to_bytes())
                    ));
                }
            }
            x if x == b's' as i32 => {
                let s = std::str::from_utf8(std::ffi::CStr::from_ptr(optarg).to_bytes()).unwrap_or("\0");
                match s.trim_start().parse::<i64>() {
                    Ok(v) => conf.sc = v as i32,
                    Err(_) => error_message(format!(
                        "Could not parse argument: -s {}\n",
                        bytes_to_string(std::ffi::CStr::from_ptr(optarg).to_bytes())
                    )),
                }
                detect = 0;
            }
            x if x == b'S' as i32 => {
                let s = std::str::from_utf8(std::ffi::CStr::from_ptr(optarg).to_bytes()).unwrap_or("\0");
                match s.trim_start().parse::<i64>() {
                    Ok(v) => new_line_skip = v as i32,
                    Err(_) => error_message(format!(
                        "Could not parse argument: -S {}\n",
                        bytes_to_string(std::ffi::CStr::from_ptr(optarg).to_bytes())
                    )),
                }
                detect = 0;
            }
            x if x == b'D' as i32 => args.download_index = 0,
            1 => {
                let version = bytes_to_string(std::ffi::CStr::from_ptr(hts::hts_version()).to_bytes());
                print!("tabix (htslib) {version}\nCopyright (C) 2025 Genome Research Ltd.\n");
                return 0;
            }
            2 => {
                return tabix_c_580_usage(true, 0);
            }
            3 => {
                let bytes = std::ffi::CStr::from_ptr(optarg).to_bytes();
                let s = std::str::from_utf8(bytes).unwrap_or("");
                let s = s.trim_start();
                let end = s
                    .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '+')
                    .unwrap_or(s.len());
                let mut v: i32 = s[..end].parse().unwrap_or(0);
                if v < 0 {
                    v = 0;
                }
                hts::hts_set_log_level(v);
            }
            4 => {
                let bytes = std::ffi::CStr::from_ptr(optarg).to_bytes();
                let s = std::str::from_utf8(bytes).unwrap_or("");
                let s = s.trim_start();
                let end = s
                    .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '+')
                    .unwrap_or(s.len());
                args.cache_megs = s[..end].parse().unwrap_or(0);
                if args.cache_megs < 0 {
                    args.cache_megs = 0;
                } else if args.cache_megs >= i32::MAX / 1048576 {
                    args.cache_megs = i32::MAX / 1048576;
                }
            }
            5 => args.separate_regs = 1,
            x if x == b'@' as i32 => {
                let bytes = std::ffi::CStr::from_ptr(optarg).to_bytes();
                let s = std::str::from_utf8(bytes).unwrap_or("");
                let s = s.trim_start();
                let end = s
                    .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '+')
                    .unwrap_or(s.len());
                args.threads = s[..end].parse().unwrap_or(0);
            }
            _ => {
                return tabix_c_580_usage(false, 1);
            }
        }
    }

    if new_line_skip >= 0 {
        conf.line_skip = new_line_skip;
    }

    if optind == argc {
        return tabix_c_580_usage(false, 1);
    }

    if list_chroms != 0 {
        let fname = std::ffi::CStr::from_ptr(*argv.add(optind as usize)).to_bytes();
        return tabix_c_396_query_chroms(fname, args.download_index);
    }

    let fname = std::ffi::CStr::from_ptr(*argv.add(optind as usize)).to_bytes().to_vec();
    let mut fname_c = fname.clone();
    fname_c.push(0);
    let fname_ptr = fname_c.as_ptr().cast::<c_char>();
    let ftype = tabix_c_102_file_type(&fname);
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
        || args.regions_fname.is_some()
        || args.targets_fname.is_some()
    {
        let regs = if args.header_only == 0 {
            let extra: Vec<Vec<u8>> = (optind + 1..argc)
                .map(|i| std::ffi::CStr::from_ptr(*argv.add(i as usize)).to_bytes().to_vec())
                .collect();
            tabix_parse_regions_owned(args.regions_fname.as_deref(), &extra).unwrap_or_default()
        } else {
            Vec::new()
        };
        return tabix_c_206_query_regions(&mut args, &mut conf, &fname, &regs);
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

    if let Some(reheader) = reheader.as_deref() {
        return tabix_c_437_reheader_file(&fname, reheader, ftype, &mut conf, args.threads);
    }

    let suffix: &[u8] = if do_csi != 0 {
        b".csi"
    } else if ftype == IS_BAM {
        b".bai"
    } else if ftype == IS_CRAM {
        b".crai"
    } else {
        b".tbi"
    };

    let mut idx_bytes = fname.clone();
    idx_bytes.extend_from_slice(suffix);
    idx_bytes.push(0);

    let mut stat_tbi: libc::stat = std::mem::zeroed();
    let mut stat_file: libc::stat = std::mem::zeroed();
    if is_force == 0 && libc::stat(idx_bytes.as_ptr().cast(), &mut stat_tbi) == 0 {
        // Before complaining about existing index, check if the VCF file isn't
        // newer. This is a common source of errors, people tend not to notice
        // that tabix failed
        libc::stat(fname_ptr, &mut stat_file);
        if stat_file.st_mtime <= stat_tbi.st_mtime {
            error_message(
                "[tabix] the index file exists. Please use '-f' to overwrite.\n".to_string(),
            );
        }
    }

    let ret;
    if ftype == IS_CRAM {
        if sam::sam_index_build3(fname_ptr.cast::<u8>(), ptr::null(), min_shift, args.threads) != 0 {
            error_message(format!(
                "bam_index_build failed: {}\n",
                bytes_to_string(&fname)
            ));
        }
        0
    } else if do_csi != 0 {
        if ftype == IS_BCF {
            if vcf::bcf_index_build3(fname_ptr, ptr::null(), min_shift, args.threads) != 0 {
                error_message(format!(
                    "bcf_index_build failed: {}\n",
                    bytes_to_string(&fname)
                ));
            }
            return 0;
        }
        if ftype == IS_BAM {
            if sam::sam_index_build3(fname_ptr.cast::<u8>(), ptr::null(), min_shift, args.threads) != 0 {
                error_message(format!(
                    "bam_index_build failed: {}\n",
                    bytes_to_string(&fname)
                ));
            }
            return 0;
        }

        ret = tbx::tbx_index_build3(&fname, None, min_shift, args.threads, &conf);
        match ret {
            0 => 0,
            -2 => error_message(format!(
                "[tabix] the compression of '{}' is not BGZF\n",
                bytes_to_string(&fname)
            )),
            _ => error_message(format!(
                "tbx_index_build3 failed: {}\n",
                bytes_to_string(&fname)
            )),
        }
    } else {
        ret = tbx::tbx_index_build3(&fname, None, min_shift, args.threads, &conf);
        match ret {
            0 => 0,
            -2 => error_message(format!(
                "[tabix] the compression of '{}' is not BGZF\n",
                bytes_to_string(&fname)
            )),
            _ => error_message(format!(
                "tbx_index_build3 failed: {}\n",
                bytes_to_string(&fname)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstr_ascii_suffix_match_preserves_strcasecmp_behavior() {
        assert!(bytes_ascii_ends_with(b"sample.VCF.GZ", b".vcf.gz"));
        assert!(bytes_ascii_ends_with(b"sample.BAM", b".bam"));
        assert!(!bytes_ascii_ends_with(b"sample.bam", b".bcf"));
        assert!(!bytes_ascii_ends_with(b".gz", b".vcf.gz"));
    }

    #[test]
    fn file_type_uses_ascii_suffixes_before_format_sniffing() {
        unsafe {
            assert_eq!(tabix_c_102_file_type(b"sample.GFF.GZ"), IS_GFF);
            assert_eq!(tabix_c_102_file_type(b"sample.BED.GZ"), IS_BED);
            assert_eq!(tabix_c_102_file_type(b"sample.SAM.GZ"), IS_SAM);
            assert_eq!(tabix_c_102_file_type(b"sample.VCF.GZ"), IS_VCF);
            assert_eq!(tabix_c_102_file_type(b"sample.BCF"), IS_BCF);
            assert_eq!(tabix_c_102_file_type(b"sample.BAM"), IS_BAM);
            assert_eq!(tabix_c_102_file_type(b"sample.CRAM"), IS_CRAM);
            assert_eq!(tabix_c_102_file_type(b"sample.GAF.GZ"), IS_GAF);
        }
    }
}
