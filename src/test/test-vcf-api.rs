use std::io::Write;
use std::ptr;

use crate::htslib_rs::hts::{
    self, hts_close, hts_get_log_level, hts_getline, hts_idx_destroy, hts_idx_t, hts_itr_t,
    hts_open, hts_pos_t, hts_readrec_func, hts_set_log_level, kbs_clear, kbs_destroy, kbs_init,
    kbs_insert, kputc, kputsn, ks_c_str, ks_clear, ks_free, ks_resize, kstring_t,
};
use crate::htslib_rs::vcf;

macro_rules! check0 {
    ($x:expr) => {{
        if $x != 0 {
            test_test_vcf_api_c_38_error(concat!("Failed: ", stringify!($x), "\0").as_bytes());
        }
    }};
}

const KS_SEP_LINE: i32 = 2;
const BCF_VL_P: i32 = 5;
const BCF_VL_LA: i32 = 6;
const BCF_VL_LG: i32 = 7;
const BCF_VL_LR: i32 = 8;
const BCF_VL_M: i32 = 9;

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
    vcf::bcf_hdr_name2id(data.cast(), name)
}

unsafe extern "C" fn bcf_readrec_adapter(
    fp: *mut hts::BGZF,
    data: *mut std::ffi::c_void,
    r: *mut std::ffi::c_void,
    tid: *mut i32,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> i32 {
    vcf::bcf_readrec(fp, data, r, tid, beg, end)
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

unsafe fn bcf_gt_phased(idx: i32) -> i32 {
    ((idx + 1) << 1) | 1
}

unsafe fn bcf_gt_unphased(idx: i32) -> i32 {
    (idx + 1) << 1
}

unsafe fn bcf_float_set(ptr: *mut f32, value: u32) {
    *ptr = f32::from_bits(value);
}

unsafe fn bcf_float_set_missing(ptr: *mut f32) {
    bcf_float_set(ptr, crate::htslib_rs::vcf::bcf_float_missing);
}

unsafe fn bcf_float_set_vector_end(ptr: *mut f32) {
    bcf_float_set(ptr, crate::htslib_rs::vcf::bcf_float_vector_end);
}

unsafe fn bcf_float_is_missing(f: f32) -> i32 {
    (f.to_bits() == crate::htslib_rs::vcf::bcf_float_missing) as i32
}

unsafe fn bcf_float_is_vector_end(f: f32) -> i32 {
    (f.to_bits() == crate::htslib_rs::vcf::bcf_float_vector_end) as i32
}

unsafe fn fail_open(fname: *const u8) -> ! {
    let name = std::ffi::CStr::from_ptr(fname.cast()).to_bytes();
    let err = std::ffi::CStr::from_ptr(libc::strerror(*libc::__errno_location())).to_bytes();
    eprintln!(
        "Failed to open \"{}\" : {}",
        String::from_utf8_lossy(name),
        String::from_utf8_lossy(err)
    );
    std::process::exit(-1);
}

unsafe fn fail_errno(label: *const u8) -> ! {
    let label = std::ffi::CStr::from_ptr(label.cast()).to_bytes();
    let err = std::ffi::CStr::from_ptr(libc::strerror(*libc::__errno_location())).to_bytes();
    eprintln!(
        "{} : {}",
        String::from_utf8_lossy(label),
        String::from_utf8_lossy(err)
    );
    std::process::exit(-1);
}

unsafe fn bcf_hdr_id2length(hdr: *mut vcf::bcf_hdr_t, type_: i32, int_id: i32) -> i32 {
    (((*(*(*hdr).id[crate::htslib_rs::vcf::BCF_DT_ID as usize].add(int_id as usize)).val).info
        [type_ as usize]
        >> 8)
        & 0xf) as i32
}

unsafe fn bcf_hdr_id2number(hdr: *mut vcf::bcf_hdr_t, type_: i32, int_id: i32) -> i32 {
    ((*(*(*hdr).id[crate::htslib_rs::vcf::BCF_DT_ID as usize].add(int_id as usize)).val).info
        [type_ as usize]
        >> 12) as i32
}

// original: error (htslib/test/test-vcf-api.c:38)
pub unsafe fn test_test_vcf_api_c_38_error(format: &[u8]) -> ! {
    // Trim a trailing NUL if present (callers may pass NUL-terminated bytes).
    let msg: &[u8] = match format.last() {
        Some(0) => &format[..format.len() - 1],
        _ => format,
    };
    if msg.last() == Some(&b'\n') {
        eprint!("{}", String::from_utf8_lossy(msg));
    } else {
        eprintln!("{}", String::from_utf8_lossy(msg));
    }
    std::process::exit(-1);
}

// original: check_alleles (htslib/test/test-vcf-api.c:51)
pub unsafe fn test_test_vcf_api_c_51_check_alleles(
    rec: *mut vcf::bcf1_t,
    alleles: *mut *const u8,
    num: i32,
) -> i32 {
    if (*rec).n_allele() as i32 != num {
        eprintln!(
            "Wrong number of alleles - expected {}, got {}",
            num,
            (*rec).n_allele() as i32
        );
        return -1;
    }
    if vcf::bcf_unpack(rec, crate::htslib_rs::vcf::BCF_UN_STR as i32) != 0 {
        return -1;
    }
    for i in 0..num {
        let expected = std::ffi::CStr::from_ptr((*alleles.add(i as usize)).cast()).to_bytes();
        let d = &(*rec).d;
        if expected != &d.allele[i as usize][..] {
            eprintln!(
                "Mismatch for allele {} : expected '{}' got '{}'",
                i,
                String::from_utf8_lossy(expected),
                String::from_utf8_lossy(&d.allele[i as usize])
            );
            return -1;
        }
    }
    0
}

// original: test_update_alleles (htslib/test/test-vcf-api.c:71)
pub unsafe fn test_test_vcf_api_c_71_test_update_alleles(
    hdr: *mut vcf::bcf_hdr_t,
    rec: *mut vcf::bcf1_t,
) {
    // Exercise bcf_update_alleles() a bit
    let mut alleles1: [*const u8; 2] = [c"G".as_ptr().cast(), c"A".as_ptr().cast()];
    let mut alleles2: [*const u8; 3] =
        [c"C".as_ptr().cast(), c"TGCA".as_ptr().cast(), c"CATG".as_ptr().cast()];
    let mut alleles3: [*const u8; 3] = [
        c"ATTCTAGATCATTCTAGATCATTCTAGATCATTCTAGATCATTCTAGATCATTCTAGATCATTCTAGATCATTCTAGATCATTCTAGATCATTCTAGATC".as_ptr().cast(),
        c"TGCA".as_ptr().cast(),
        c"CTATTATCTCTAATGACATGCTATTATCTCTAATGACATGCTATTATCTCTAATGACATGCTATTATCTCTAATGACATGCTATTATCTCTAATGACATGCTATTATCTCTAATGACATGCTATTATCTCTAATGACATGCTATTATCTCTAATGACATGCTATTATCTCTAATGACATGCTATTATCTCTAATGACATG".as_ptr().cast(),
    ];
    let mut alleles4 = [alleles3[2], ptr::null(), alleles3[0]];
    // Add some alleles
    check0!(vcf::bcf_update_alleles(hdr, rec, alleles1.as_mut_ptr().cast(), 2));
    check0!(test_test_vcf_api_c_51_check_alleles(
        rec,
        alleles1.as_mut_ptr(),
        2
    ));
    // Erase them
    check0!(vcf::bcf_update_alleles(hdr, rec, ptr::null_mut(), 0));
    check0!(test_test_vcf_api_c_51_check_alleles(
        rec,
        ptr::null_mut(),
        0
    ));
    // Expand to three
    check0!(vcf::bcf_update_alleles(hdr, rec, alleles2.as_mut_ptr().cast(), 3));
    check0!(test_test_vcf_api_c_51_check_alleles(
        rec,
        alleles2.as_mut_ptr(),
        3
    ));
    // Now try some bigger ones (should force a realloc)
    check0!(vcf::bcf_update_alleles(hdr, rec, alleles3.as_mut_ptr().cast(), 3));
    check0!(test_test_vcf_api_c_51_check_alleles(
        rec,
        alleles3.as_mut_ptr(),
        3
    ));
    // Ensure it works even if one of the alleles points into the
    // existing structure. The owned d.allele[1] bytes have no NUL, so build a
    // NUL-terminated temp that outlives the bcf_update_alleles FFI call.
    let alias_d = &(*rec).d;
    let mut alias1 = alias_d.allele[1].clone();
    alias1.push(0);
    alleles4[1] = alias1.as_ptr().cast::<u8>();
    check0!(vcf::bcf_update_alleles(hdr, rec, alleles4.as_mut_ptr().cast(), 3));
    alleles4[1] = alleles3[1]; // Will have been clobbered by the update
    drop(alias1);
    check0!(test_test_vcf_api_c_51_check_alleles(
        rec,
        alleles4.as_mut_ptr(),
        3
    ));
    // Ensure it works when the alleles point into the existing data,
    // rec->d.allele is used to define the input array and the
    // order of the entries is changed.  The result of this should
    // be the same as alleles2.
    // Original packed-buffer pointer arithmetic:
    //   tmp       = tail-4 bytes of old allele[0]  (== "CATG")
    //   allele[0] = tail-1 byte  of old allele[2]  (== "C")
    //   allele[2] = tmp                            (== "CATG")
    //   allele[1] unchanged                        (== "TGCA")
    // The reordered set therefore equals alleles2 = {"C","TGCA","CATG"}.
    // Reproduce on the owned bytes, then pass through the FFI as NUL temps.
    let d = &mut (*rec).d;
    let tmp = d.allele[0][d.allele[0].len() - 4..].to_vec();
    let new0 = d.allele[2][d.allele[2].len() - 1..].to_vec();
    d.allele[0] = new0;
    d.allele[2] = tmp;
    let mut reordered: Vec<Vec<u8>> = Vec::new();
    for a in &d.allele {
        let mut t = a.clone();
        t.push(0);
        reordered.push(t);
    }
    let mut reordered_ptrs: Vec<*const u8> =
        reordered.iter().map(|t| t.as_ptr().cast::<u8>()).collect();
    check0!(vcf::bcf_update_alleles(
        hdr,
        rec,
        reordered_ptrs.as_mut_ptr().cast(),
        3
    ));
    drop(reordered);
    check0!(test_test_vcf_api_c_51_check_alleles(
        rec,
        alleles2.as_mut_ptr(),
        3
    ));
}

// original: write_bcf (htslib/test/test-vcf-api.c:110)
pub unsafe fn test_test_vcf_api_c_110_write_bcf(fname: *mut u8) {
    // Init
    let fp = hts_open(fname.cast(), c"wb".as_ptr());
    if fp.is_null() {
        fail_open(fname);
    }
    let hdr = vcf::bcf_hdr_init(c"w".as_ptr());
    if hdr.is_null() {
        fail_errno(c"bcf_hdr_init".as_ptr().cast());
    }
    let rec = vcf::bcf_init();
    if rec.is_null() {
        fail_errno(c"bcf_init1".as_ptr().cast());
    }

    // Check no-op on fresh bcf1_t
    check0!(vcf::bcf_update_alleles(hdr, rec, ptr::null_mut(), 0));

    // Create VCF header
    let str_: kstring_t = kstring_t { data: Vec::new() };
    check0!(vcf::bcf_hdr_append(hdr, c"##fileDate=20090805".as_ptr()));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FORMAT=<ID=UF,Number=1,Type=Integer,Description=\"Unused FORMAT\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##INFO=<ID=UI,Number=1,Type=Integer,Description=\"Unused INFO\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FILTER=<ID=Flt,Description=\"Unused FILTER\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##unused=<XX=AA,Description=\"Unused generic\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##unused=<ID=BB,Description=\"Unused generic with ID\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##unused=unformatted text 1".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##unused=unformatted text 2".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##contig=<ID=Unused,length=1>".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##source=myImputationProgramV3.1".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##reference=file:///seq/references/1000GenomesPilot-NCBI36.fasta".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(hdr, c"##contig=<ID=20,length=62435964,assembly=B36,md5=f126cdf8a6e0c7f379d618ff66beb2da,species=\"Homo sapiens\",taxonomy=x>".as_ptr()));
    check0!(vcf::bcf_hdr_append(hdr, c"##phasing=partial".as_ptr()));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##INFO=<ID=NS,Number=1,Type=Integer,Description=\"Number of Samples With Data\">"
            .as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Total Depth\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##INFO=<ID=NEG,Number=.,Type=Integer,Description=\"Test -ve Numbers\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##INFO=<ID=AF,Number=A,Type=Float,Description=\"Allele Frequency\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##INFO=<ID=AA,Number=1,Type=String,Description=\"Ancestral Allele\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##INFO=<ID=DB,Number=0,Type=Flag,Description=\"dbSNP membership, build 129\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##INFO=<ID=H2,Number=0,Type=Flag,Description=\"HapMap2 membership\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FILTER=<ID=q10,Description=\"Quality below 10\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FILTER=<ID=s50,Description=\"Less than half of samples have data\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FORMAT=<ID=GQ,Number=1,Type=Integer,Description=\"Genotype Quality\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FORMAT=<ID=DP,Number=1,Type=Integer,Description=\"Read Depth\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FORMAT=<ID=HQ,Number=2,Type=Integer,Description=\"Haplotype Quality\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FORMAT=<ID=TS,Number=1,Type=String,Description=\"Test String 1\">".as_ptr()
    ));

    // Try a few header modifications
    vcf::bcf_hdr_remove(
        hdr,
        crate::htslib_rs::vcf::BCF_HL_CTG as i32,
        c"Unused".as_ptr(),
    );
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##contig=<ID=Unused,length=62435964>".as_ptr()
    ));
    vcf::bcf_hdr_remove(
        hdr,
        crate::htslib_rs::vcf::BCF_HL_FMT as i32,
        c"TS".as_ptr(),
    );
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FORMAT=<ID=TS,Number=1,Type=String,Description=\"Test String\">".as_ptr()
    ));
    vcf::bcf_hdr_remove(
        hdr,
        crate::htslib_rs::vcf::BCF_HL_INFO as i32,
        c"NEG".as_ptr(),
    );
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##INFO=<ID=NEG,Number=.,Type=Integer,Description=\"Test Negative Numbers\">".as_ptr()
    ));
    vcf::bcf_hdr_remove(
        hdr,
        crate::htslib_rs::vcf::BCF_HL_FLT as i32,
        c"s50".as_ptr(),
    );
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FILTER=<ID=s50,Description=\"Less than 50% of samples have data\">".as_ptr()
    ));

    check0!(vcf::bcf_hdr_add_sample(hdr, c"NA00001".as_ptr()));
    check0!(vcf::bcf_hdr_add_sample(hdr, c"NA00002".as_ptr()));
    check0!(vcf::bcf_hdr_add_sample(hdr, c"NA00003".as_ptr()));
    check0!(vcf::bcf_hdr_add_sample(hdr, ptr::null()));
    if vcf::bcf_hdr_write(fp, hdr) != 0 {
        eprintln!(
            "Failed to write to {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes())
        );
        std::process::exit(-1);
    }

    // Add a record
    // 20     14370   rs6054257 G      A       29   PASS   NS=3;DP=14;NEG=-127;AF=0.5;DB;H2           GT:GQ:DP:HQ 0|0:48:1:51,51 1|0:48:8:51,51 1/1:43:5:.,.
    // .. CHROM
    (*rec).rid = vcf::bcf_hdr_name2id(hdr, c"20".as_ptr());
    // .. POS
    (*rec).pos = 14369;
    // .. ID
    check0!(vcf::bcf_update_id(hdr, rec, c"rs6054257".as_ptr()));
    // .. REF and ALT
    test_test_vcf_api_c_71_test_update_alleles(hdr, rec);
    let mut alleles: [*const u8; 2] = [c"G".as_ptr().cast(), c"A".as_ptr().cast()];
    check0!(vcf::bcf_update_alleles_str(hdr, rec, c"G,A".as_ptr()));
    check0!(test_test_vcf_api_c_51_check_alleles(
        rec,
        alleles.as_mut_ptr(),
        2
    ));
    // .. QUAL
    (*rec).qual = 29.0;
    // .. FILTER
    let mut tmpi = vcf::bcf_hdr_id2int(
        hdr,
        crate::htslib_rs::vcf::BCF_DT_ID as i32,
        c"PASS".as_ptr(),
    );
    check0!(vcf::bcf_update_filter(hdr, rec, &mut tmpi, 1));
    // .. INFO
    tmpi = 3;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"NS".as_ptr(),
        (&tmpi as *const i32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpi = 500;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"DP".as_ptr(),
        (&tmpi as *const i32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpi = 100000;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"DP".as_ptr(),
        (&tmpi as *const i32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpi = 14;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"DP".as_ptr(),
        (&tmpi as *const i32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpi = -127;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"NEG".as_ptr(),
        (&tmpi as *const i32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    let tmpf: f32 = 0.5;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"AF".as_ptr(),
        (&tmpf as *const f32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_REAL as i32
    ));
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"DB".as_ptr(),
        ptr::null(),
        1,
        crate::htslib_rs::vcf::BCF_HT_FLAG as i32
    ));
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"H2".as_ptr(),
        ptr::null(),
        1,
        crate::htslib_rs::vcf::BCF_HT_FLAG as i32
    ));
    // .. FORMAT
    let nsamples = (*hdr).n[crate::htslib_rs::vcf::BCF_DT_SAMPLE as usize] as i32;
    let mut tmpia: Vec<i32> = vec![0; (nsamples * 2) as usize];
    tmpia[0] = bcf_gt_phased(0);
    tmpia[1] = bcf_gt_phased(0);
    tmpia[2] = bcf_gt_phased(1);
    tmpia[3] = bcf_gt_phased(0);
    tmpia[4] = bcf_gt_unphased(1);
    tmpia[5] = bcf_gt_unphased(1);
    check0!(vcf::bcf_update_format(
        hdr,
        rec,
        c"GT".as_ptr(),
        tmpia.as_ptr().cast(),
        nsamples * 2,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpia[0] = 48;
    tmpia[1] = 48;
    tmpia[2] = 43;
    check0!(vcf::bcf_update_format(
        hdr,
        rec,
        c"GQ".as_ptr(),
        tmpia.as_ptr().cast(),
        nsamples,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpia[0] = 0;
    tmpia[1] = 0;
    tmpia[2] = 1;
    check0!(vcf::bcf_update_format(
        hdr,
        rec,
        c"DP".as_ptr(),
        tmpia.as_ptr().cast(),
        nsamples,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpia[0] = 1;
    tmpia[1] = 100000;
    tmpia[2] = 1;
    check0!(vcf::bcf_update_format(
        hdr,
        rec,
        c"DP".as_ptr(),
        tmpia.as_ptr().cast(),
        nsamples,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpia[0] = 1;
    tmpia[1] = 8;
    tmpia[2] = 5;
    check0!(vcf::bcf_update_format(
        hdr,
        rec,
        c"DP".as_ptr(),
        tmpia.as_ptr().cast(),
        nsamples,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpia[0] = 51;
    tmpia[1] = 51;
    tmpia[2] = 51;
    tmpia[3] = 51;
    tmpia[4] = crate::htslib_rs::vcf::bcf_int32_missing;
    tmpia[5] = crate::htslib_rs::vcf::bcf_int32_missing;
    check0!(vcf::bcf_update_format(
        hdr,
        rec,
        c"HQ".as_ptr(),
        tmpia.as_ptr().cast(),
        nsamples * 2,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    let mut tmp_str = [
        c"String1".as_ptr(),
        c"SomeOtherString2".as_ptr(),
        c"YetAnotherString3".as_ptr(),
    ];
    check0!(vcf::bcf_update_format_string(
        hdr,
        rec,
        c"TS".as_ptr(),
        tmp_str.as_mut_ptr(),
        3
    ));
    tmp_str[0] = c"LongerStringRequiringBufferReallocation".as_ptr();
    check0!(vcf::bcf_update_format_string(
        hdr,
        rec,
        c"TS".as_ptr(),
        tmp_str.as_mut_ptr(),
        3
    ));
    tmp_str[0] = c"String1".as_ptr();
    check0!(vcf::bcf_update_format_string(
        hdr,
        rec,
        c"TS".as_ptr(),
        tmp_str.as_mut_ptr(),
        3
    ));
    if vcf::bcf_write(fp, hdr, rec) != 0 {
        eprintln!(
            "Failed to write to {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes())
        );
        std::process::exit(-1);
    }

    // 20     1110696 . A      G,T     67   .   NS=2;DP=10;NEG=-128;AF=0.333,.;AA=T;DB GT 2 1   ./.
    vcf::bcf_clear(rec);
    (*rec).rid = vcf::bcf_hdr_name2id(hdr, c"20".as_ptr());
    (*rec).pos = 1110695;
    check0!(vcf::bcf_update_alleles_str(hdr, rec, c"A,G,T".as_ptr()));
    (*rec).qual = 67.0;
    tmpi = 2;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"NS".as_ptr(),
        (&tmpi as *const i32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpi = 10;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"DP".as_ptr(),
        (&tmpi as *const i32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    tmpi = -128;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"NEG".as_ptr(),
        (&tmpi as *const i32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    let mut tmpfa: Vec<f32> = vec![0.0; 2];
    tmpfa[0] = 0.333;
    bcf_float_set_missing(&mut tmpfa[1]);
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"AF".as_ptr(),
        tmpfa.as_ptr().cast(),
        2,
        crate::htslib_rs::vcf::BCF_HT_REAL as i32
    ));
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"AA".as_ptr(),
        c"SHORT".as_ptr().cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_STR as i32
    ));
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"AA".as_ptr(),
        c"LONGSTRING".as_ptr().cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_STR as i32
    ));
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"AA".as_ptr(),
        c"T".as_ptr().cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_STR as i32
    ));
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"DB".as_ptr(),
        ptr::null(),
        1,
        crate::htslib_rs::vcf::BCF_HT_FLAG as i32
    ));
    tmpia[0] = bcf_gt_phased(2);
    tmpia[1] = crate::htslib_rs::vcf::bcf_int32_vector_end;
    tmpia[2] = bcf_gt_phased(1);
    tmpia[3] = crate::htslib_rs::vcf::bcf_int32_vector_end;
    tmpia[4] = 0;
    tmpia[5] = 0;
    check0!(vcf::bcf_update_format(
        hdr,
        rec,
        c"GT".as_ptr(),
        tmpia.as_ptr().cast(),
        nsamples * 2,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    if vcf::bcf_write(fp, hdr, rec) != 0 {
        eprintln!(
            "Failed to write to {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes())
        );
        std::process::exit(-1);
    }

    drop(tmpia);
    drop(tmpfa);

    // Clean
    drop(str_);
    vcf::bcf_destroy(rec);
    vcf::bcf_hdr_destroy(hdr);
    let ret = hts_close(fp);
    if ret != 0 {
        eprintln!(
            "hts_close({}): non-zero status {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes()),
            ret
        );
        std::process::exit(ret);
    }
}

// original: bcf_to_vcf (htslib/test/test-vcf-api.c:287)
pub unsafe fn test_test_vcf_api_c_287_bcf_to_vcf(fname: *mut u8) {
    let fp = hts_open(fname.cast(), c"rb".as_ptr());
    if fp.is_null() {
        fail_open(fname);
    }
    let hdr = vcf::bcf_hdr_read(fp);
    if hdr.is_null() {
        fail_errno(c"bcf_hdr_read".as_ptr().cast());
    }
    let rec = vcf::bcf_init();
    if rec.is_null() {
        fail_errno(c"bcf_init1".as_ptr().cast());
    }

    // gz_fname is fname + ".gz", kept NUL-terminated for the raw production APIs.
    let mut gz_fname: Vec<u8> =
        std::ffi::CStr::from_ptr(fname.cast()).to_bytes().to_vec();
    gz_fname.extend_from_slice(b".gz");
    gz_fname.push(0);
    let out = hts_open(gz_fname.as_ptr().cast(), c"wg".as_ptr());
    if out.is_null() {
        let err =
            std::ffi::CStr::from_ptr(libc::strerror(*libc::__errno_location())).to_bytes();
        eprintln!(
            "Couldn't open \"{}\" : {}",
            String::from_utf8_lossy(&gz_fname[..gz_fname.len() - 1]),
            String::from_utf8_lossy(err)
        );
        std::process::exit(-1);
    }

    let hdr_out = vcf::bcf_hdr_dup(hdr);
    if vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_STR as i32,
        c"ID".as_ptr(),
        c"BB".as_ptr(),
        c"unused".as_ptr(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(c"Missing header ##unused=<ID=BB, ...>".to_bytes());
    }
    vcf::bcf_hdr_remove(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_STR as i32,
        c"BB".as_ptr(),
    );
    if !vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_STR as i32,
        c"ID".as_ptr(),
        c"BB".as_ptr(),
        c"unused".as_ptr(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(
            c"Got pointer to deleted header ##unused=<ID=BB, ...>".to_bytes(),
        );
    }

    if vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_GEN as i32,
        c"unused".as_ptr(),
        c"unformatted text 1".as_ptr(),
        ptr::null(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(c"Missing header ##unused=unformatted text 1".to_bytes());
    }
    vcf::bcf_hdr_remove(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_GEN as i32,
        c"unused".as_ptr(),
    );
    if !vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_GEN as i32,
        c"unused".as_ptr(),
        c"unformatted text 1".as_ptr(),
        ptr::null(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(
            c"Got pointer to deleted header ##unused=unformatted text 1".to_bytes(),
        );
    }

    if vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_FLT as i32,
        c"ID".as_ptr(),
        c"Flt".as_ptr(),
        ptr::null(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(c"Missing header ##FILTER=<ID=Flt, ...>".to_bytes());
    }
    vcf::bcf_hdr_remove(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_FLT as i32,
        c"Flt".as_ptr(),
    );
    if !vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_FLT as i32,
        c"ID".as_ptr(),
        c"Flt".as_ptr(),
        ptr::null(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(
            c"Got pointer to deleted header ##FILTER=<ID=Flt, ...>".to_bytes(),
        );
    }

    if vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_INFO as i32,
        c"ID".as_ptr(),
        c"UI".as_ptr(),
        ptr::null(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(c"Missing header ##INFO=<ID=UI, ...>".to_bytes());
    }
    vcf::bcf_hdr_remove(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_INFO as i32,
        c"UI".as_ptr(),
    );
    if !vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_INFO as i32,
        c"ID".as_ptr(),
        c"UI".as_ptr(),
        ptr::null(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(c"Got pointer to deleted header ##INFO=<ID=UI, ...>".to_bytes());
    }

    if vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_FMT as i32,
        c"ID".as_ptr(),
        c"UF".as_ptr(),
        ptr::null(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(c"Missing header ##INFO=<ID=UF, ...>".to_bytes());
    }
    vcf::bcf_hdr_remove(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_FMT as i32,
        c"UF".as_ptr(),
    );
    if !vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_FMT as i32,
        c"ID".as_ptr(),
        c"UF".as_ptr(),
        ptr::null(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(c"Got pointer to deleted header ##INFO=<ID=UF, ...>".to_bytes());
    }

    if vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_CTG as i32,
        c"ID".as_ptr(),
        c"Unused".as_ptr(),
        ptr::null(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(c"Missing header ##contig=<ID=Unused,length=1>".to_bytes());
    }
    vcf::bcf_hdr_remove(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_CTG as i32,
        c"Unused".as_ptr(),
    );
    if !vcf::bcf_hdr_get_hrec(
        hdr_out,
        crate::htslib_rs::vcf::BCF_HL_FMT as i32,
        c"ID".as_ptr(),
        c"Unused".as_ptr(),
        ptr::null(),
    )
    .is_null()
    {
        test_test_vcf_api_c_38_error(
            c"Got pointer to header ##contig=<ID=Unused,length=1>".to_bytes(),
        );
    }

    if vcf::bcf_hdr_write(out, hdr_out) != 0 {
        eprintln!(
            "Failed to write to {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes())
        );
        std::process::exit(-1);
    }
    let mut r;
    loop {
        r = vcf::bcf_read(fp, hdr, rec);
        if r < 0 {
            break;
        }
        if vcf::bcf_write(out, hdr_out, rec) != 0 {
            eprintln!(
            "Failed to write to {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes())
        );
        std::process::exit(-1);
        }

        // Test problems caused by bcf1_sync: the data block
        // may be realloced, also the unpacked structures must
        // get updated.
        check0!(vcf::bcf_unpack(
            rec,
            crate::htslib_rs::vcf::BCF_UN_STR as i32
        ));
        check0!(vcf::bcf_update_id(hdr, rec, ptr::null()));
        check0!(vcf::bcf_update_format(
            hdr,
            rec,
            c"GQ".as_ptr(),
            ptr::null(),
            0,
            crate::htslib_rs::vcf::BCF_HT_INT as i32
        ));

        let dup = vcf::bcf_dup(rec); // force bcf1_sync call
        if vcf::bcf_write(out, hdr_out, dup) != 0 {
            eprintln!(
            "Failed to write to {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes())
        );
        std::process::exit(-1);
        }
        vcf::bcf_destroy(dup);

        check0!(vcf::bcf_update_alleles_str(hdr_out, rec, c"G,A".as_ptr()));
        let tmpi: i32 = 99;
        check0!(vcf::bcf_update_info(
            hdr_out,
            rec,
            c"DP".as_ptr(),
            (&tmpi as *const i32).cast(),
            1,
            crate::htslib_rs::vcf::BCF_HT_INT as i32
        ));
        let tmpia = [9i32, 9, 9];
        check0!(vcf::bcf_update_format(
            hdr_out,
            rec,
            c"DP".as_ptr(),
            tmpia.as_ptr().cast(),
            3,
            crate::htslib_rs::vcf::BCF_HT_INT as i32
        ));

        if vcf::bcf_write(out, hdr_out, rec) != 0 {
            eprintln!(
            "Failed to write to {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes())
        );
        std::process::exit(-1);
        }
    }
    if r < -1 {
        test_test_vcf_api_c_38_error(c"bcf_read1".to_bytes());
    }

    vcf::bcf_destroy(rec);
    vcf::bcf_hdr_destroy(hdr);
    vcf::bcf_hdr_destroy(hdr_out);
    let mut ret = hts_close(fp);
    if ret != 0 {
        eprintln!(
            "hts_close({}): non-zero status {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes()),
            ret
        );
        std::process::exit(ret);
    }
    ret = hts_close(out);
    if ret != 0 {
        eprintln!(
            "hts_close({}): non-zero status {}",
            String::from_utf8_lossy(&gz_fname[..gz_fname.len() - 1]),
            ret
        );
        std::process::exit(ret);
    }

    // read gzip, write stdout
    let gz_in = hts_open(gz_fname.as_ptr().cast(), c"r".as_ptr());
    if gz_in.is_null() {
        eprintln!(
            "Could not read: {}",
            String::from_utf8_lossy(&gz_fname[..gz_fname.len() - 1])
        );
        std::process::exit(1);
    }

    let mut __out = std::io::stdout();
    let mut line = kstring_t { data: Vec::new() };
    while hts_getline(gz_in, KS_SEP_LINE, &mut line) > 0 {
        kputc(b'\n' as i32, &mut line);
        write!(__out, "{}", String::from_utf8_lossy(&line.data)).unwrap();
    }
    __out.flush().unwrap();

    ret = hts_close(gz_in);
    if ret != 0 {
        eprintln!(
            "hts_close({}): non-zero status {}",
            String::from_utf8_lossy(&gz_fname[..gz_fname.len() - 1]),
            ret
        );
        std::process::exit(ret);
    }
    drop(line);
    drop(gz_fname);
}

// original: iterator (htslib/test/test-vcf-api.c:406)
pub unsafe fn test_test_vcf_api_c_406_iterator(fname: *const u8) {
    let fp = hts_open(fname.cast(), c"r".as_ptr());
    if fp.is_null() {
        fail_open(fname);
    }
    let hdr = vcf::bcf_hdr_read(fp);
    if hdr.is_null() {
        fail_errno(c"bcf_hdr_read".as_ptr().cast());
    }

    vcf::bcf_index_build(fname.cast(), 0);
    let idx = vcf::bcf_index_load2(fname.cast(), ptr::null());

    let iter = bcf_itr_querys1(idx.cast(), hdr, c"20:1110600-1110800".as_ptr().cast()).cast();
    crate::htslib_rs::hts::hts_itr_destroy(iter);

    let iter = bcf_itr_querys1(idx.cast(), hdr, c"20:1110600-1110800".as_ptr().cast()).cast();
    crate::htslib_rs::hts::hts_itr_destroy(iter);

    hts_idx_destroy(idx);
    vcf::bcf_hdr_destroy(hdr);
    let ret = hts_close(fp);
    if ret != 0 {
        eprintln!(
            "hts_close({}): non-zero status {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes()),
            ret
        );
        std::process::exit(ret);
    }
}

// original: test_get_info_values (htslib/test/test-vcf-api.c:434)
pub unsafe fn test_test_vcf_api_c_434_test_get_info_values(fname: *const u8) {
    let fp = hts_open(fname.cast(), c"r".as_ptr());
    if fp.is_null() {
        fail_open(fname);
    }
    let hdr = vcf::bcf_hdr_read(fp);
    if hdr.is_null() {
        fail_errno(c"bcf_hdr_read".as_ptr().cast());
    }
    let line = vcf::bcf_init();
    if line.is_null() {
        fail_errno(c"bcf_init".as_ptr().cast());
    }
    let mut r;
    loop {
        r = vcf::bcf_read(fp, hdr, line);
        if r != 0 {
            break;
        }
        let mut afs: *mut f32 = ptr::null_mut();
        let mut negs: *mut i32 = ptr::null_mut();
        let mut count = 0;
        let ret = vcf::bcf_get_info_values(
            hdr,
            line,
            c"AF".as_ptr(),
            (&mut afs as *mut *mut f32).cast(),
            &mut count,
            crate::htslib_rs::vcf::BCF_HT_REAL as i32,
        );

        if (*line).pos == 14369 {
            if ret != 1 || *afs.add(0) != 0.5 {
                eprintln!("AF on position 14370 should be 0.5");
                std::process::exit(-1);
            }
        } else if ret != 2 || *afs.add(0) != 0.333 || bcf_float_is_missing(*afs.add(1)) == 0 {
            eprintln!("AF on position 1110696 should be 0.333, missing");
            std::process::exit(-1);
        }

        libc::free(afs.cast());

        let expected = if (*line).pos == 14369 { -127 } else { -128 };
        count = 0;
        let ret = vcf::bcf_get_info_values(
            hdr,
            line,
            c"NEG".as_ptr(),
            (&mut negs as *mut *mut i32).cast(),
            &mut count,
            crate::htslib_rs::vcf::BCF_HT_INT as i32,
        );
        if ret != 1 || *negs.add(0) != expected {
            if ret < 0 {
                eprintln!("NEG should be {}, got error ret={}", expected, ret);
            } else if ret == 0 {
                eprintln!("NEG should be {}, got no entries", expected);
            } else {
                eprintln!(
                    "NEG should be {}, got {} entries (first is {})",
                    expected,
                    ret,
                    *negs.add(0)
                );
            }
            std::process::exit(1);
        }
        libc::free(negs.cast());
    }
    if r < -1 {
        test_test_vcf_api_c_38_error(c"bcf_read".to_bytes());
    }

    vcf::bcf_destroy(line);
    vcf::bcf_hdr_destroy(hdr);
    hts_close(fp);
}

// original: write_format_values (htslib/test/test-vcf-api.c:491)
pub unsafe fn test_test_vcf_api_c_491_write_format_values(fname: *const u8) {
    // Init
    let fp = hts_open(fname.cast(), c"wb".as_ptr());
    if fp.is_null() {
        fail_open(fname);
    }
    let hdr = vcf::bcf_hdr_init(c"w".as_ptr());
    if hdr.is_null() {
        fail_errno(c"bcf_hdr_init".as_ptr().cast());
    }
    let rec = vcf::bcf_init();
    if rec.is_null() {
        fail_errno(c"bcf_init1".as_ptr().cast());
    }

    // Create VCF header
    check0!(vcf::bcf_hdr_append(hdr, c"##contig=<ID=1>".as_ptr()));
    check0!(vcf::bcf_hdr_append(
        hdr,
        c"##FORMAT=<ID=TF,Number=1,Type=Float,Description=\"Test Float\">".as_ptr()
    ));
    check0!(vcf::bcf_hdr_add_sample(hdr, c"S".as_ptr()));
    check0!(vcf::bcf_hdr_add_sample(hdr, ptr::null()));
    if vcf::bcf_hdr_write(fp, hdr) != 0 {
        eprintln!(
            "Failed to write to {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes())
        );
        std::process::exit(-1);
    }

    // Add a record
    // .. FORMAT
    let mut test = [0f32; 4];
    bcf_float_set_missing(&mut test[0]);
    test[1] = 47.11;
    bcf_float_set_vector_end(&mut test[2]);
    test[3] = -1.2e-13;
    check0!(vcf::bcf_update_format(
        hdr,
        rec,
        c"TF".as_ptr(),
        test.as_ptr().cast(),
        4,
        crate::htslib_rs::vcf::BCF_HT_REAL as i32
    ));
    if vcf::bcf_write(fp, hdr, rec) != 0 {
        eprintln!(
            "Failed to write to {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes())
        );
        std::process::exit(-1);
    }

    vcf::bcf_destroy(rec);
    vcf::bcf_hdr_destroy(hdr);
    let ret = hts_close(fp);
    if ret != 0 {
        eprintln!(
            "hts_close({}): non-zero status {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fname.cast()).to_bytes()),
            ret
        );
        std::process::exit(ret);
    }
}

// original: check_format_values (htslib/test/test-vcf-api.c:528)
pub unsafe fn test_test_vcf_api_c_528_check_format_values(fname: *const u8) {
    let fp = hts_open(fname.cast(), c"r".as_ptr());
    let hdr = vcf::bcf_hdr_read(fp);
    let line = vcf::bcf_init();

    while vcf::bcf_read(fp, hdr, line) == 0 {
        let mut values: *mut f32 = ptr::null_mut();
        let mut count = 0;
        let ret = vcf::bcf_get_format_values(
            hdr,
            line,
            c"TF".as_ptr(),
            (&mut values as *mut *mut f32).cast(),
            &mut count,
            crate::htslib_rs::vcf::BCF_HT_REAL as i32,
        );

        // NOTE the return value from bcf_get_format_float is different from
        // bcf_get_info_float in the sense that vector-end markers also count.
        if ret != 4
            || count < ret
            || bcf_float_is_missing(*values.add(0)) == 0
            || *values.add(1) != 47.11
            || bcf_float_is_vector_end(*values.add(2)) == 0
            || bcf_float_is_vector_end(*values.add(3)) == 0
        {
            eprintln!("bcf_get_format_float didn't produce the expected output.");
            std::process::exit(-1);
        }

        libc::free(values.cast());
    }

    vcf::bcf_destroy(line);
    vcf::bcf_hdr_destroy(hdr);
    hts_close(fp);
}

// original: test_get_format_values (htslib/test/test-vcf-api.c:561)
pub unsafe fn test_test_vcf_api_c_561_test_get_format_values(fname: *const u8) {
    test_test_vcf_api_c_491_write_format_values(fname);
    test_test_vcf_api_c_528_check_format_values(fname);
}

// original: test_invalid_end_tag (htslib/test/test-vcf-api.c:567)
pub unsafe fn test_test_vcf_api_c_567_test_invalid_end_tag() {
    let vcf_data = c"data:,##fileformat=VCFv4.1
##contig=<ID=X,length=155270560>
##INFO=<ID=END,Number=1,Type=Integer,Description=\"End coordinate of this variant\">
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO
X	86470037	rs59780433a	TTTCA	TGGTT,T	.	.	END=85725113
X	86470038	rs59780433b	T	TGGTT,T	.	.	END=86470047
";

    let logging = hts_get_log_level();

    // Silence warning messages
    hts_set_log_level(crate::htslib_rs::hts::HTS_LOG_ERROR);

    let fp = hts_open(vcf_data.as_ptr(), c"r".as_ptr());
    if fp.is_null() {
        fail_errno(c"Failed to open vcf data".as_ptr().cast());
    }
    let rec = vcf::bcf_init();
    if rec.is_null() {
        fail_errno(c"Failed to allocate BCF record".as_ptr().cast());
    }

    let hdr = vcf::bcf_hdr_read(fp);
    if hdr.is_null() {
        fail_errno(c"Failed to read BCF header".as_ptr().cast());
    }

    check0!(vcf::bcf_read(fp, hdr, rec));
    // rec->rlen should ignore the bogus END tag value on the first read
    if (*rec).rlen != 5 {
        eprintln!("Incorrect rlen - expected 5 got {}", (*rec).rlen);
        std::process::exit(-1);
    }

    check0!(vcf::bcf_read(fp, hdr, rec));
    // While on the second it should use it
    if (*rec).rlen != 10 {
        eprintln!("Incorrect rlen - expected 10 got {}", (*rec).rlen);
        std::process::exit(-1);
    }

    // Try to break it - will change rlen
    let tmpi: i32 = 85725113;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"END".as_ptr(),
        (&tmpi as *const i32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));

    if (*rec).rlen != 1 {
        eprintln!("Incorrect rlen - expected 1 got {}", (*rec).rlen);
        std::process::exit(-1);
    }

    let ret = vcf::bcf_read(fp, hdr, rec);
    if ret != -1 {
        eprintln!("Unexpected return code {} from bcf_read at EOF", ret);
        std::process::exit(-1);
    }

    vcf::bcf_destroy(rec);
    vcf::bcf_hdr_destroy(hdr);
    let ret = hts_close(fp);
    if ret != 0 {
        eprintln!("Unexpected return code {} from hts_close", ret);
        std::process::exit(-1);
    }

    hts_set_log_level(logging);
}

// original: test_open_format (htslib/test/test-vcf-api.c:630)
pub unsafe fn test_test_vcf_api_c_630_test_open_format() {
    let mut mode = [0u8; 5];
    mode[0] = b'r';
    mode[1] = 0;
    let mut ret = vcf::vcf_open_mode(mode.as_mut_ptr().add(1).cast(), c"mode1.bcf".as_ptr(), ptr::null());
    if &mode[..2] != b"rb" || ret != 0 {
        eprintln!(
            "Mode '{}' does not match the expected value '{}'",
            String::from_utf8_lossy(&mode[..2]),
            "rb"
        );
        std::process::exit(-1);
    }
    mode[1] = 0;
    ret = vcf::vcf_open_mode(mode.as_mut_ptr().add(1).cast(), c"mode1.vcf".as_ptr(), ptr::null());
    if &mode[..1] != b"r" || ret != 0 {
        eprintln!(
            "Mode '{}' does not match the expected value '{}'",
            String::from_utf8_lossy(&mode[..1]),
            "r"
        );
        std::process::exit(-1);
    }
    mode[1] = 0;
    ret = vcf::vcf_open_mode(
        mode.as_mut_ptr().add(1).cast(),
        c"mode1.vcf.gz".as_ptr(),
        ptr::null(),
    );
    if &mode[..2] != b"rz" || ret != 0 {
        eprintln!(
            "Mode '{}' does not match the expected value '{}'",
            String::from_utf8_lossy(&mode[..2]),
            "rz"
        );
        std::process::exit(-1);
    }
    mode[1] = 0;
    ret = vcf::vcf_open_mode(
        mode.as_mut_ptr().add(1).cast(),
        c"mode1.vcf.bgz".as_ptr(),
        ptr::null(),
    );
    if &mode[..2] != b"rz" || ret != 0 {
        eprintln!(
            "Mode '{}' does not match the expected value '{}'",
            String::from_utf8_lossy(&mode[..2]),
            "rz"
        );
        std::process::exit(-1);
    }
    mode[1] = 0;
    ret = vcf::vcf_open_mode(mode.as_mut_ptr().add(1).cast(), c"mode1.xcf".as_ptr(), ptr::null());
    if ret == 0 {
        test_test_vcf_api_c_38_error(c"Expected failure for wrong extension 'xcf'".to_bytes());
    }
    mode[1] = 0;
    ret = vcf::vcf_open_mode(
        mode.as_mut_ptr().add(1).cast(),
        c"mode1.vcf.gbz".as_ptr(),
        ptr::null(),
    );
    if ret == 0 {
        test_test_vcf_api_c_38_error(c"Expected failure for wrong extension 'vcf.gbz'".to_bytes());
    }
    mode[1] = 0;
    ret = vcf::vcf_open_mode(
        mode.as_mut_ptr().add(1).cast(),
        c"mode1.bvcf.bgz".as_ptr(),
        ptr::null(),
    );
    if ret == 0 {
        test_test_vcf_api_c_38_error(
            c"Expected failure for wrong extension 'vcf.bvcf.bgz'".to_bytes(),
        );
    }
}

// original: test_rlen_values (htslib/test/test-vcf-api.c:664)
pub unsafe fn test_test_vcf_api_c_664_test_rlen_values() {
    let data = "##reference=file://tmp\n\
##FILTER=<ID=PASS,Description=\"All filters passed\">\n\
##INFO=<ID=END,Number=1,Type=Integer,Description=\"end\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"gt\">\n\
##INFO=<ID=SVLEN,Number=A,Type=Integer,Description=\"svlen\">\n\
##INFO=<ID=CN,Number=A,Type=Float,Description=\"Copy number\">\n\
##INFO=<ID=SVCLAIM,Number=A,Type=String,Description=\"svclaim\">\n\
##FORMAT=<ID=LEN,Number=1,Type=Integer,Description=\"fmt len\">\n\
##contig=<ID=1,Length=40>\n\
##ALT=<ID=INS,Description=\"INS\">\n\
##ALT=<ID=DEL,Description=\"DEL\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tSample1\tSample2\n\
1\t4310\t.\tG\tA\t.\t.\t.\tGT\t0/0\t0|1\n\
1\t4311\t.\tC\tCT\t.\t.\t.\tGT\t0/0\t1/0\n\
1\t4312\t.\tTC\tT\t213.73\t.\t.\tGT\t0/1\t0|0\n\
1\t4314\t.\tG\t<INS>\t213.73\t.\tSVLEN=10;SVCLAIM=J\tGT\t0/1\t0|0\n\
1\t4315\t.\tG\t<DEL>\t213.73\t.\tSVLEN=-10;SVCLAIM=D\tGT\t0/1\t0|0\n\
1\t4326\t.\tG\t<INS>\t213.73\t.\tEND=4326;SVLEN=10;SVCLAIM=J\tGT\t0/1\t0|0\n\
1\t4327\t.\tG\t<DEL>\t213.73\t.\tEND=4337;SVLEN=-10;SVCLAIM=J\tGT\t0/1\t0|0\n\
1\t4338\t.\tG\t<*>\t213.73\t.\tEND=4342;SVLEN=.;SVCLAIM=.\tGT:LEN\t0/1:7\t0|0:8\n\
1\t4353\t.\tG\t<*>\t213.73\t.\tEND=4357;SVLEN=.;SVCLAIM=.\tGT:LEN\t0/1:7\t0|0:.\n\
1\t4363\t.\tG\t<*>\t213.73\t.\tEND=4367;SVLEN=.;SVCLAIM=.\tGT:LEN\t0/1:7\t0|0:.\n\
1\t4370\t.\tG\t<INS>,<*>\t213.73\t.\tEND=4371;SVLEN=.;SVCLAIM=.\tGT:LEN\t0/1:7\t0|0:.\n\
1\t4378\t.\tG\t<DEL>,<INS>,<*>\t213.73\t.\tEND=4379;SVLEN=3,5,.;SVCLAIM=D,J,.\tGT:LEN\t0/1:7\t0|0:.\n\
1\t4385\t.\tG\tT,<DEL>\t213.73\t.\tEND=4387;SVLEN=.,180\tGT\t0/1\t0|0\n\
1\t4585\t.\tG\tT,<DEL:ME>\t213.73\t.\tEND=4587;SVLEN=.,180\tGT\t0/1\t0|0\n\
1\t4685\t.\tG\t<DUP>,<DUP>\t213.73\t.\tEND=4687;SVLEN=10,10\tGT\t0/1\t0|0\n\
1\t4705\t.\tG\t<CNV>\t213.73\t.\tEND=4707;SVLEN=11;CN=2\tGT\t0/1\t0|0\n\
1\t4725\t.\tG\t<CNV:TR>\t213.73\t.\tEND=4727;SVLEN=12;CN=1.5\tGT\t0/1\t0|0\n\
1\t4745\t.\tG\t<INV>\t213.73\t.\tEND=4747;SVLEN=10\tGT\t0/1\t0|0\n\
1\t4885\t.\tG\tT,<*>\t213.73\t.\tEND=4887\tGT:LEN\t0/1:190\t0|0:.\n\
1\t5885\t.\tG\tT\t213.73\t.\tEND=5887;SVLEN=8;SVCLAIM=.\tGT:LEN\t0/1:.\t0|0:10\n";
    // NUL-terminated byte buffers passed to the still-raw production hts_open.
    let mut d43: Vec<u8> = format!("data:,##fileformat=VCFv4.3\n{}", data).into_bytes();
    d43.push(0);
    let mut d44: Vec<u8> = format!("data:,##fileformat=VCFv4.4\n{}", data).into_bytes();
    d44.push(0);
    let mut d45: Vec<u8> = format!("data:,##fileformat=VCFv4.5\n{}", data).into_bytes();
    d45.push(0);
    let rlen = [
        1, 1, 2, 1, 11, 1, 11, 8, 7, 7, 7, 7, 181, 181, 11, 12, 13, 11, 190, 3,
    ];
    let darr = [d43.as_ptr().cast(), d44.as_ptr().cast(), d45.as_ptr().cast()];
    let rarr = [&rlen, &rlen, &rlen];

    let logging = hts_get_log_level();

    // Silence warning messages
    hts_set_log_level(crate::htslib_rs::hts::HTS_LOG_ERROR);

    let rec = vcf::bcf_init();
    let rec2 = vcf::bcf_init();
    if rec.is_null() || rec2.is_null() {
        fail_errno(c"Failed to allocate BCF record".as_ptr().cast());
    }
    //calculating rlen with different vcf versions
    for (i, (&data, &expected_lens)) in darr.iter().zip(rarr.iter()).enumerate() {
        let fp = hts_open(data, c"r".as_ptr());
        if fp.is_null() {
            fail_errno(c"Failed to open vcf data".as_ptr().cast());
        }
        vcf::bcf_clear(rec);
        let hdr = vcf::bcf_hdr_read(fp);
        if hdr.is_null() {
            fail_errno(c"Failed to read BCF header".as_ptr().cast());
        }
        for (j, &expected_rlen) in expected_lens.iter().enumerate() {
            check0!(vcf::bcf_read(fp, hdr, rec));
            if (*rec).rlen != expected_rlen {
                eprintln!(
                    "Incorrect rlen @ vcf {} on test {} - expected {} got {}",
                    j as i32 + 1,
                    i as i32 + 1,
                    expected_rlen,
                    (*rec).rlen
                );
                std::process::exit(-1);
            }
        }
        vcf::bcf_hdr_destroy(hdr);
        hts_close(fp);
    }

    //calculating rlen with update to fields
    let fp = hts_open(d45.as_ptr().cast(), c"r".as_ptr());
    let mut id = 1;
    let mut val = [1i32, 15];
    vcf::bcf_clear(rec);
    let hdr = vcf::bcf_hdr_read(fp);
    if hdr.is_null() {
        fail_errno(c"Failed to read BCF header".as_ptr().cast());
    }
    check0!(vcf::bcf_read(fp, hdr, rec));
    if (*rec).rlen != 1 {
        eprintln!("Incorrect rlen set, expected 1 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }
    id += 1;
    check0!(vcf::bcf_update_alleles_str(hdr, rec, c"G,AT".as_ptr()));
    if (*rec).rlen != 1 {
        eprintln!("Incorrect rlen set, expected 1 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }
    id += 1;
    check0!(vcf::bcf_update_alleles_str(hdr, rec, c"GC,A".as_ptr()));
    if (*rec).rlen != 2 {
        eprintln!("Incorrect rlen set, expected 2 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }
    id += 1;
    check0!(vcf::bcf_update_alleles_str(hdr, rec, c"G,<*>".as_ptr()));
    if (*rec).rlen != 1 {
        eprintln!("Incorrect rlen set, expected 1 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }
    let tmpi: i32 = 4323;
    id += 1;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"END".as_ptr(),
        (&tmpi as *const i32).cast(),
        1,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    if (*rec).rlen != 14 {
        eprintln!("Incorrect rlen set, expected 14 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }
    id += 1;
    check0!(vcf::bcf_update_format(
        hdr,
        rec,
        c"LEN".as_ptr(),
        val.as_ptr().cast(),
        2,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    if (*rec).rlen != 15 {
        eprintln!("Incorrect rlen set, expected 15 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }
    id += 1;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"END".as_ptr(),
        (&tmpi as *const i32).cast(),
        0,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    if (*rec).rlen != 15 {
        eprintln!("Incorrect rlen set, expected 15 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }
    id += 1;
    check0!(vcf::bcf_update_format(
        hdr,
        rec,
        c"LEN".as_ptr(),
        (&tmpi as *const i32).cast(),
        0,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    if (*rec).rlen != 1 {
        eprintln!("Incorrect rlen set, expected 1 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }
    id += 1;
    check0!(vcf::bcf_update_alleles_str(hdr, rec, c"G,T,<DEL>".as_ptr()));
    if (*rec).rlen != 1 {
        eprintln!("Incorrect rlen set, expected 1 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }
    val[0] = 0;
    val[1] = -5;
    id += 1;
    check0!(vcf::bcf_update_info(
        hdr,
        rec,
        c"SVLEN".as_ptr(),
        val.as_ptr().cast(),
        2,
        crate::htslib_rs::vcf::BCF_HT_INT as i32
    ));
    if (*rec).rlen != 6 {
        eprintln!("Incorrect rlen set, expected 6 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }
    id += 1;
    vcf::bcf_copy(rec2, rec);
    if (*rec2).rlen != 6 {
        eprintln!("Incorrect rlen set, expected 6 got {} @ {}", (*rec).rlen, id);
        std::process::exit(-1);
    }

    //needs update when header version change is handled
    vcf::bcf_destroy(rec);
    vcf::bcf_destroy(rec2);
    vcf::bcf_hdr_destroy(hdr);

    hts_close(fp);

    hts_set_log_level(logging);
}

// original: test_vl_types (htslib/test/test-vcf-api.c:807)
pub unsafe fn test_test_vcf_api_c_807_test_vl_types() {
    let test_vcf = c"data:,##fileformat=VCFv4.5
##FILTER=<ID=PASS,Description=\"All filters passed\">
##contig=<ID=5,length=243199373>
##INFO=<ID=FIXED_1_INFO,Number=1,Type=Integer,Description=\"Fixed number 1\">
##INFO=<ID=FIXED_4_INFO,Number=4,Type=Float,Description=\"Fixed number 4\">
##INFO=<ID=VL_DOT_INFO,Number=.,Type=Integer,Description=\"Variable number\">
##INFO=<ID=VL_A_INFO,Number=A,Type=Integer,Description=\"One value for each ALT allele\">
##INFO=<ID=VL_G_INFO,Number=G,Type=Integer,Description=\"One value for each possible genotype\">
##INFO=<ID=VL_R_INFO,Number=R,Type=Integer,Description=\"One value for each allele including REF\">
##FORMAT=<ID=FIXED_1_FMT,Number=1,Type=String,Description=\"Fixed number 1\">
##FORMAT=<ID=FIXED_4_FMT,Number=4,Type=String,Description=\"Fixed number 4\">
##FORMAT=<ID=VL_DOT_FMT,Number=.,Type=String,Description=\"Variable number\">
##FORMAT=<ID=VL_A_FMT,Number=A,Type=Integer,Description=\"One value for each ALT allele\">
##FORMAT=<ID=VL_G_FMT,Number=G,Type=Integer,Description=\"One value for each possible genotype\">
##FORMAT=<ID=VL_R_FMT,Number=R,Type=Integer,Description=\"One value for each allele including REF\">
##FORMAT=<ID=VL_P_FMT,Number=P,Type=String,Description=\"One value for each allele value defined in GT\">
##FORMAT=<ID=VL_LA_FMT,Number=LA,Type=Integer,Description=\"One value for each local ALT allele\">
##FORMAT=<ID=VL_LG_FMT,Number=LG,Type=Integer,Description=\"One value for each local genotype\">
##FORMAT=<ID=VL_LR_FMT,Number=LR,Type=Integer,Description=\"One value for each local allele including REF\">
##FORMAT=<ID=VL_M_FMT,Number=M,Type=Integer,Description=\"One value for each posible base modification of the given type\">
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	AAA
";

    struct ExpectedTypes {
        id: *const std::ffi::c_char,
        type_: i32,
        expected_vl_code: i32,
        expected_number: i32,
    }

    let expected = [
        ExpectedTypes {
            id: c"FIXED_1_INFO".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_INFO as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_FIXED as i32,
            expected_number: 1,
        },
        ExpectedTypes {
            id: c"FIXED_4_INFO".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_INFO as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_FIXED as i32,
            expected_number: 4,
        },
        ExpectedTypes {
            id: c"VL_DOT_INFO".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_INFO as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_VAR as i32,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_A_INFO".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_INFO as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_A as i32,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_G_INFO".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_INFO as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_G as i32,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_R_INFO".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_INFO as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_R as i32,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"FIXED_1_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_FIXED as i32,
            expected_number: 1,
        },
        ExpectedTypes {
            id: c"FIXED_4_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_FIXED as i32,
            expected_number: 4,
        },
        ExpectedTypes {
            id: c"VL_DOT_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_VAR as i32,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_A_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_A as i32,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_G_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_G as i32,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_R_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: crate::htslib_rs::vcf::BCF_VL_R as i32,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_P_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: BCF_VL_P,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_LA_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: BCF_VL_LA,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_LG_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: BCF_VL_LG,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_LR_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: BCF_VL_LR,
            expected_number: 0xfffff,
        },
        ExpectedTypes {
            id: c"VL_M_FMT".as_ptr(),
            type_: crate::htslib_rs::vcf::BCF_HL_FMT as i32,
            expected_vl_code: BCF_VL_M,
            expected_number: 0xfffff,
        },
    ];

    let fp = hts_open(test_vcf.as_ptr(), c"r".as_ptr());
    if fp.is_null() {
        test_test_vcf_api_c_38_error(c"Failed to open test data".to_bytes());
    }
    let hdr = vcf::bcf_hdr_read(fp);
    if hdr.is_null() {
        test_test_vcf_api_c_38_error(c"Failed to read BCF header".to_bytes());
    }
    for exp in &expected {
        let id_num = vcf::bcf_hdr_id2int(hdr, crate::htslib_rs::vcf::BCF_DT_ID as i32, exp.id);
        let id_str = String::from_utf8_lossy(std::ffi::CStr::from_ptr(exp.id.cast()).to_bytes())
            .into_owned();
        if id_num < 0 {
            eprintln!("Couldn't look up VCF header ID {}", id_str);
            std::process::exit(-1);
        }
        let vl_code = bcf_hdr_id2length(hdr, exp.type_, id_num);
        if vl_code != exp.expected_vl_code {
            let length_types: [&[u8]; 10] = [
                b"BCF_VL_FIXED",
                b"BCF_VL_VAR",
                b"BCF_VL_A",
                b"BCF_VL_G",
                b"BCF_VL_R",
                b"BCF_VL_P",
                b"BCF_VL_LA",
                b"BCF_VL_LG",
                b"BCF_VL_LR",
                b"BCF_VL_M",
            ];
            if vl_code >= 0 && (vl_code as usize) < length_types.len() {
                eprintln!(
                    "Unexpected length code for {}: expected {} got {}",
                    id_str,
                    String::from_utf8_lossy(length_types[exp.expected_vl_code as usize]),
                    String::from_utf8_lossy(length_types[vl_code as usize])
                );
            } else {
                eprintln!(
                    "Unexpected length code for {}: expected {} got {}",
                    id_str,
                    String::from_utf8_lossy(length_types[exp.expected_vl_code as usize]),
                    vl_code
                );
            }
            std::process::exit(-1);
        }
        let num = bcf_hdr_id2number(hdr, exp.type_, id_num);
        if num != exp.expected_number {
            let exp_suffix = if exp.expected_number == 0xfffff {
                " (= code for not fixed)"
            } else {
                ""
            };
            let num_suffix = if num == 0xfffff {
                " (= code for not fixed)"
            } else {
                ""
            };
            eprintln!(
                "Unexpected number for {}: expected {}{} got {}{}",
                id_str, exp.expected_number, exp_suffix, num, num_suffix
            );
            std::process::exit(-1);
        }
    }
    vcf::bcf_hdr_destroy(hdr);
    hts_close(fp);
}

// original: read_vcf_line (htslib/test/test-vcf-api.c:909)
pub unsafe fn test_test_vcf_api_c_909_read_vcf_line(
    line: *const u8,
    hdr: *mut vcf::bcf_hdr_t,
    rec: *mut vcf::bcf1_t,
    kstr: &mut kstring_t,
) -> i32 {
    let line_len = libc::strlen(line.cast());
    let line_slice = std::slice::from_raw_parts(line.cast::<u8>(), line_len);
    ks_clear(kstr);
    if kputsn(line_slice, line_len, kstr) < 0 {
        return -1;
    }

    let ret = vcf::vcf_parse(kstr, hdr, rec);
    if ret != 0 {
        eprintln!(
            "vcf_parse(\"{}\", hdr, rec) returned {}",
            String::from_utf8_lossy(ks_c_str(kstr)),
            ret
        );
    }
    ret
}

// original: chomp (htslib/test/test-vcf-api.c:924)
pub unsafe fn test_test_vcf_api_c_924_chomp(kstr: &mut kstring_t) {
    if kstr.data.len() < 1 {
        return;
    }
    if kstr.data[kstr.data.len() - 1] == b'\n' {
        kstr.data.pop();
    }
}

// original: test_bcf_remove_allele_set (htslib/test/test-vcf-api.c:933)
pub unsafe fn test_test_vcf_api_c_933_test_bcf_remove_allele_set() {
    let header = c"##fileformat=VCFv4.5
##FILTER=<ID=PASS,Description=\"All filters passed\">
##contig=<ID=5,length=243199373>
##INFO=<ID=AC,Number=A,Type=Integer,Description=\"Allele count\">
##INFO=<ID=AD,Number=R,Type=Integer,Description=\"Allele depth\">
##INFO=<ID=AF,Number=A,Type=Float,Description=\"Allele frequency\">
##INFO=<ID=CN,Number=A,Type=Float,Description=\"Copy number of CNV/breakpoint\">
##INFO=<ID=CICN,Number=.,Type=Float,Description=\"Confidence interval around copy number\">
##INFO=<ID=CIEND,Number=.,Type=Integer,Description=\"Confidence interval around the inferred END for symbolic structural variants\">
##INFO=<ID=CILEN,Number=.,Type=Integer,Description=\"Confidence interval for the SVLEN field\">
##INFO=<ID=CIPOS,Number=.,Type=Integer,Description=\"Confidence interval around POS for symbolic structural variants\">
##INFO=<ID=CIRB,Number=.,Type=Integer,Description=\"Confidence interval around RB\">
##INFO=<ID=CIRUC,Number=.,Type=Float,Description=\"Confidence interval around RUC\">
##INFO=<ID=IMPRECISE,Number=0,Type=Flag,Description=\"Imprecise structural variant\">
##INFO=<ID=MEINFO,Number=.,Type=String,Description=\"Mobile element info of the form NAME,START,END,POLARITY\">
##INFO=<ID=METRANS,Number=.,Type=String,Description=\"Mobile element transduction info of the form CHR,START,END,POLARITY\">
##INFO=<ID=RB,Number=.,Type=Integer,Description=\"Total number of bases in the corresponding repeat sequence\">
##INFO=<ID=RN,Number=A,Type=Integer,Description=\"Total number of repeat sequences in this allele\">
##INFO=<ID=RUB,Number=.,Type=Integer,Description=\"Number of bases in each individual repeat unit\">
##INFO=<ID=RUC,Number=.,Type=Float,Description=\"Repeat unit count of corresponding repeat sequence\">
##INFO=<ID=RUL,Number=.,Type=Integer,Description=\"Repeat unit length of the corresponding repeat sequence\">
##INFO=<ID=RUS,Number=.,Type=String,Description=\"Repeat unit sequence of the corresponding repeat sequence\">
##INFO=<ID=SVLEN,Number=A,Type=Integer,Description=\"Length of structural variant\">
##INFO=<ID=VL_A_STR_INFO,Number=A,Type=String,Description=\"INFO string Number=A\">
##INFO=<ID=VL_R_STR_INFO,Number=R,Type=String,Description=\"INFO string Number=R\">
##FORMAT=<ID=AD,Number=R,Type=Integer,Description=\"Allele depth\">
##FORMAT=<ID=EC,Number=A,Type=Integer,Description=\"Expected allele count\">
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">
##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"List of Phred-scaled genotype likelihoods\">
##FORMAT=<ID=LAA,Number=.,Type=Integer,Description=\"Local alleles\">
##FORMAT=<ID=LAD,Number=LR,Type=Integer,Description=\"Local allele depth\">
##FORMAT=<ID=LEC,Number=LA,Type=Integer,Description=\"Local expected allele count\">
##FORMAT=<ID=LPL,Number=LG,Type=Integer,Description=\"List of Phred-scaled local genotype likelihoods\">
##FORMAT=<ID=VL_A_STR_FMT,Number=A,Type=String,Description=\"FMT string Number=A\">
##FORMAT=<ID=VL_G_STR_FMT,Number=G,Type=String,Description=\"FMT string Number=G\">
##FORMAT=<ID=VL_LA_STR_FMT,Number=LA,Type=String,Description=\"FMT string Number=LA\">
##FORMAT=<ID=VL_LG_STR_FMT,Number=LG,Type=String,Description=\"FMT string Number=LG\">
##FORMAT=<ID=VL_LR_STR_FMT,Number=LR,Type=String,Description=\"FMT string Number=LR\">
##FORMAT=<ID=VL_R_STR_FMT,Number=R,Type=String,Description=\"FMT string Number=R\">
##ALT=<ID=CNV:TR,Description=\"Tandem repeat determined based on DNA abundance\">
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	AAA	BBB	CCC
";
    let inputs = [
        c"5	110285	.	T	C,<*>	.	PASS	AC=1,0;AD=6,5,0;AF=0.99,0.01;VL_A_STR_INFO=alt_c,alt_nonref;VL_R_STR_INFO=ref,alt_c,alt_nonref	GT:AD:EC:PL:VL_A_STR_FMT:VL_G_STR_FMT:VL_R_STR_FMT	.:.:.:.:.:.:.	0/1:6,5,0:4,0:114,0,15,35,73,113:alt_c,alt_nonref:gt_00,gt_01,gt_11,gt_02,gt_12,gt_22:ref,alt_c,alt_nonref	.:.:.:.:.:.:.".as_ptr(),
        c"5	110290	.	T	C,A	.	PASS	AC=90,1;AD=6,5,6;AF=0.009,0.0001;VL_A_STR_INFO=alt_c,alt_a;VL_R_STR_INFO=ref,alt_c,alt_a	GT:LAA:LAD:LEC:LPL:VL_LA_STR_FMT:VL_LG_STR_FMT:VL_LR_STR_FMT	0/0:.:3:.:0:.:gt_00:ref	0/1:1,2:3,2,0:44,27:114,0,15,35,73,113:alt_c,alt_a:gt_00,gt_01,gt_11,gt_02,gt_12,gt_22:ref,alt_c,alt_a	1/1:1:0,3:46:110,15,0:alt_c:gt_00,gt_01,gt_11:ref,alt_c".as_ptr(),
        c"5	110350	.	T	<INS>,<INS>	.	PASS	IMPRECISE;SVLEN=100,200;CIEND=-50,50,-25,25;CIPOS=-10,10,-20,20	GT	0/1	0/1	0/1".as_ptr(),
        c"5	110500	.	T	<CNV>,<CNV>	.	PASS	IMPRECISE;SVLEN=50,100;CILEN=0,25,-25,25;CN=2,4;CICN=-0.5,1,-1.5,1.5	GT	0/1	0/1	0/1".as_ptr(),
        c"5	110700	.	A	<INS:ME>,<INS:ME>	.	PASS	MEINFO=AluY,1,260,+,FLAM_C,1,110,-;METRANS=1,94820,95080,+,1,129678,129788,-	GT	0/1	0/1	0/1".as_ptr(),
        c"5	112000	.	C	<CNV:TR>,<CNV:TR>	.	PASS	RN=2,1;RUS=CAG,TTG,CA;RUL=3,3,2;RB=12,6,6;RUC=4,2,3;RUB=3,3,3,3,3,3,2,2,2;SVLEN=18,6".as_ptr(),
        c"5	113000	.	T	C,A	.	PASS	AC=90,1;AD=6,5,6;AF=0.009,0.0001;VL_A_STR_INFO=alt_c,alt_a;VL_R_STR_INFO=ref,alt_c,alt_a	GT:LAA:LAD:LEC:LPL:VL_LA_STR_FMT:VL_LG_STR_FMT:VL_LR_STR_FMT	0/0:.:3:.:0:.:gt_00:ref	0/1:1,2:3,2,0:44,27:114,0,15,35,73,113:alt_c,alt_a:gt_00,gt_01,gt_11,gt_02,gt_12,gt_22:ref,alt_c,alt_a	1/1:1:0,3:46:110,15,0:alt_c:gt_00,gt_01,gt_11:ref,alt_c".as_ptr(),
        c"5	114000	.	T	C,A	.	PASS	AC=90,1;AD=6,5,6;AF=0.009,0.0001;VL_A_STR_INFO=alt_c,alt_a;VL_R_STR_INFO=ref,alt_c,alt_a	GT:LAA:LAD:LEC:LPL:VL_LA_STR_FMT:VL_LG_STR_FMT:VL_LR_STR_FMT	0/0:.:3:.:0:.:gt_00:ref	0/1:1,2:3,2,0:44,27:114,0,15,35,73,113:alt_c,alt_a:gt_00,gt_01,gt_11,gt_02,gt_12,gt_22:ref,alt_c,alt_a	1/1:1:0,3:46:110,15,0:alt_c:gt_00,gt_01,gt_11:ref,alt_c".as_ptr(),
        c"5	115000	.	C	<CNV:TR>,<CNV:TR>	.	PASS	RN=2,1;RUS=CAG,TTG,CA;RUL=3,3,2;RB=12,6,6;RUC=4,2,3;RUB=3,3,3,3,3,3,2,2,2;SVLEN=18,6".as_ptr(),
    ];
    let expected = [
        c"5	110285	.	T	C	.	PASS	AC=1;AD=6,5;AF=0.99;VL_A_STR_INFO=alt_c;VL_R_STR_INFO=ref,alt_c	GT:AD:EC:PL:VL_A_STR_FMT:VL_G_STR_FMT:VL_R_STR_FMT	.:.:.:.:.:.:.	0/1:6,5:4:114,0,15:alt_c:gt_00,gt_01,gt_11:ref,alt_c	.:.:.:.:.:.:.".as_ptr(),
        c"5	110290	.	T	C	.	PASS	AC=90;AD=6,5;AF=0.009;VL_A_STR_INFO=alt_c;VL_R_STR_INFO=ref,alt_c	GT:LAA:LAD:LEC:LPL:VL_LA_STR_FMT:VL_LG_STR_FMT:VL_LR_STR_FMT	0/0:.:3:.:0:.:gt_00:ref	0/1:1:3,2:44:114,0,15:alt_c:gt_00,gt_01,gt_11:ref,alt_c	1/1:1:0,3:46:110,15,0:alt_c:gt_00,gt_01,gt_11:ref,alt_c".as_ptr(),
        c"5	110350	.	T	<INS>	.	PASS	IMPRECISE;SVLEN=100;CIEND=-50,50;CIPOS=-10,10	GT	0/1	0/1	0/1".as_ptr(),
        c"5	110500	.	T	<CNV>	.	PASS	IMPRECISE;SVLEN=50;CILEN=0,25;CN=2;CICN=-0.5,1	GT	0/1	0/1	0/1".as_ptr(),
        c"5	110700	.	A	<INS:ME>	.	PASS	MEINFO=AluY,1,260,+;METRANS=1,94820,95080,+	GT	0/1	0/1	0/1".as_ptr(),
        c"5	112000	.	C	<CNV:TR>	.	PASS	RN=2;RUS=CAG,TTG;RUL=3,3;RB=12,6;RUC=4,2;RUB=3,3,3,3,3,3;SVLEN=18".as_ptr(),
        c"5	113000	.	T	A	.	PASS	AC=1;AD=6,6;AF=0.0001;VL_A_STR_INFO=alt_a;VL_R_STR_INFO=ref,alt_a	GT:LAA:LAD:LEC:LPL:VL_LA_STR_FMT:VL_LG_STR_FMT:VL_LR_STR_FMT	0/0:.:3:.:0:.:gt_00:ref	0/.:1:3,0:27:114,35,113:alt_a:gt_00,gt_02,gt_22:ref,alt_a	./.:.:0:.:110:.:gt_00:ref".as_ptr(),
        c"5	114000	.	T	.	.	PASS	AD=6;VL_R_STR_INFO=ref	GT:LAA:LAD:LEC:LPL:VL_LA_STR_FMT:VL_LG_STR_FMT:VL_LR_STR_FMT	0/0:.:3:.:0:.:gt_00:ref	0/.:.:3:.:114:.:gt_00:ref	./.:.:0:.:110:.:gt_00:ref".as_ptr(),
        c"5	115000	.	C	.	.	PASS	.".as_ptr(),
    ];

    let mut kstr = kstring_t { data: Vec::new() };

    let hdr = vcf::bcf_hdr_init(c"r".as_ptr());
    let rec = vcf::bcf_init();
    let rm_set = kbs_init(3);

    if hdr.is_null() {
        test_test_vcf_api_c_38_error(c"bcf_hdr_init() failed".to_bytes());
    }

    if rec.is_null() {
        test_test_vcf_api_c_38_error(c"bcf_init() failed".to_bytes());
    }

    if rm_set.is_null() {
        test_test_vcf_api_c_38_error(c"kbs_init() failed".to_bytes());
    }

    check0!(ks_resize(&mut kstr, 1000));
    check0!(vcf::bcf_hdr_parse(hdr, header.as_ptr().cast_mut()));
    for (i, &input) in inputs.iter().enumerate() {
        check0!(test_test_vcf_api_c_909_read_vcf_line(
            input.cast(),
            hdr,
            rec,
            &mut kstr
        ));
        kbs_clear(&mut *rm_set);
        if (*rec).pos == 113000 - 1 {
            kbs_insert(&mut *rm_set, 1);
        } else if (*rec).pos >= 114000 - 1 {
            kbs_insert(&mut *rm_set, 1);
            kbs_insert(&mut *rm_set, 2);
        } else {
            kbs_insert(&mut *rm_set, 2);
        }
        check0!(vcf::bcf_remove_allele_set(&*hdr, &mut *rec, &*rm_set));
        ks_clear(&mut kstr);
        check0!(vcf::vcf_format(hdr, rec, &mut kstr));
        test_test_vcf_api_c_924_chomp(&mut kstr);
        let got = ks_c_str(&kstr);
        let want = std::ffi::CStr::from_ptr(expected[i].cast()).to_bytes();
        if want != got {
            eprintln!(
                "bcf_remove_allele_set() output differs\nExpected:\n{}\nGot:\n{}",
                String::from_utf8_lossy(want),
                String::from_utf8_lossy(got)
            );
            std::process::exit(-1);
        }
    }
    vcf::bcf_destroy(rec);
    vcf::bcf_hdr_destroy(hdr);
    ks_free(&mut kstr);
    kbs_destroy(rm_set);
}

// original: main (htslib/test/test-vcf-api.c:1047)
pub unsafe fn test_test_vcf_api_c_1047_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let fname = if argc > 1 {
        *argv.add(1)
    } else {
        c"rmme.bcf".as_ptr().cast_mut().cast::<u8>()
    };

    // format test. quiet unless there's a failure
    test_test_vcf_api_c_561_test_get_format_values(fname);

    // main test. writes to stdout
    test_test_vcf_api_c_110_write_bcf(fname);
    test_test_vcf_api_c_287_bcf_to_vcf(fname);
    test_test_vcf_api_c_406_iterator(fname);

    // additional tests. quiet unless there's a failure.
    test_test_vcf_api_c_807_test_vl_types();
    test_test_vcf_api_c_434_test_get_info_values(fname);
    test_test_vcf_api_c_567_test_invalid_end_tag();
    test_test_vcf_api_c_630_test_open_format();
    test_test_vcf_api_c_664_test_rlen_values();
    test_test_vcf_api_c_933_test_bcf_remove_allele_set();
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    static TEST_VCF_API_LOCK: Mutex<()> = Mutex::new(());

    fn temp_bcf_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "htslib_rs-test-vcf-api-{}-{}.bcf",
            std::process::id(),
            label
        ))
    }

    fn temp_path(label: &str, ext: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "htslib_rs-test-vcf-api-{label}-{}-{}.{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed"),
            ext
        ))
    }

    fn c_path(path: &Path) -> CString {
        CString::new(path.to_string_lossy().as_bytes()).unwrap()
    }

    fn cleanup_generated_bcf(path: &Path) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(path.with_extension("bcf.gz"));
        let _ = std::fs::remove_file(path.with_extension("bcf.csi"));
        let _ = std::fs::remove_file(path.with_extension("bcf.tbi"));
    }

    unsafe fn run_bcf_to_vcf_capture_stdout(bcf_path: &Path, out_path: &Path) -> i32 {
        let _ = std::fs::remove_file(out_path);
        libc::fflush(ptr::null_mut());
        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");

        if pid == 0 {
            let out_c = c_path(out_path);
            let out_fd = libc::open(
                out_c.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                0o600,
            );
            if out_fd < 0 {
                libc::_exit(libc::EXIT_FAILURE);
            }
            if libc::dup2(out_fd, libc::STDOUT_FILENO) < 0 {
                libc::close(out_fd);
                libc::_exit(libc::EXIT_FAILURE);
            }
            libc::close(out_fd);

            let bcf_path_c = c_path(bcf_path);
            test_test_vcf_api_c_287_bcf_to_vcf(bcf_path_c.as_ptr().cast_mut().cast::<u8>());
            libc::fflush(ptr::null_mut());
            libc::_exit(libc::EXIT_SUCCESS);
        }

        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        assert!(libc::WIFEXITED(status), "child did not exit normally");
        libc::WEXITSTATUS(status)
    }

    unsafe fn run_main_capture_stdout(args: &mut [CString], out_path: &Path) -> i32 {
        let _ = std::fs::remove_file(out_path);
        libc::fflush(ptr::null_mut());
        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");

        if pid == 0 {
            let out_c = c_path(out_path);
            let out_fd = libc::open(
                out_c.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                0o600,
            );
            if out_fd < 0 {
                libc::_exit(libc::EXIT_FAILURE);
            }
            if libc::dup2(out_fd, libc::STDOUT_FILENO) < 0 {
                libc::close(out_fd);
                libc::_exit(libc::EXIT_FAILURE);
            }
            libc::close(out_fd);

            let mut argv = args
                .iter_mut()
                .map(|arg| arg.as_ptr().cast_mut().cast::<u8>())
                .collect::<Vec<_>>();
            let ret = test_test_vcf_api_c_1047_main(argv.len() as i32, argv.as_mut_ptr());
            libc::fflush(ptr::null_mut());
            libc::_exit(ret);
        }

        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        assert!(libc::WIFEXITED(status), "child did not exit normally");
        libc::WEXITSTATUS(status)
    }

    #[test]
    fn original_test_vcf_api_quiet_helpers_cover_format_end_and_rlen() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_VCF_API_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let bcf_path = temp_bcf_path("format-values");
        let _ = std::fs::remove_file(&bcf_path);
        let bcf_path_c = CString::new(bcf_path.to_string_lossy().as_bytes()).unwrap();

        unsafe {
            test_test_vcf_api_c_561_test_get_format_values(bcf_path_c.as_ptr().cast());
            test_test_vcf_api_c_567_test_invalid_end_tag();
            test_test_vcf_api_c_630_test_open_format();
            test_test_vcf_api_c_664_test_rlen_values();
        }

        let _ = std::fs::remove_file(&bcf_path);
    }

    #[test]
    fn original_test_vcf_api_vl_header_types_cover_local_number_codes() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_VCF_API_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        unsafe {
            test_test_vcf_api_c_807_test_vl_types();
        }
    }

    #[test]
    fn original_test_vcf_api_allele_removal_matches_expected_records() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_VCF_API_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        unsafe {
            test_test_vcf_api_c_933_test_bcf_remove_allele_set();
        }
    }

    #[test]
    fn original_test_vcf_api_iterator_query_reads_indexed_record() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_VCF_API_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let bcf_path = temp_path("iterator", "bcf");
        cleanup_generated_bcf(&bcf_path);
        let bcf_path_c = c_path(&bcf_path);

        unsafe {
            test_test_vcf_api_c_110_write_bcf(bcf_path_c.as_ptr().cast_mut().cast::<u8>());
            assert_eq!(vcf::bcf_index_build(bcf_path_c.as_ptr(), 14), 0);

            let fp = hts_open(bcf_path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let hdr = vcf::bcf_hdr_read(fp);
            assert!(!hdr.is_null());
            let idx = vcf::bcf_index_load2(bcf_path_c.as_ptr(), ptr::null());
            assert!(!idx.is_null());
            let iter = bcf_itr_querys1(idx.cast(), hdr, c"20:1110600-1110800".as_ptr().cast());
            assert!(!iter.is_null());
            let rec = vcf::bcf_init();
            assert!(!rec.is_null());

            assert_eq!(
                hts::hts_itr_next(hts::hts_get_bgzfp(fp), iter, rec.cast(), hdr.cast()),
                0
            );
            assert_eq!((*rec).pos, 1_110_695);
            let mut expected_alleles: [*const u8; 3] =
                [c"A".as_ptr().cast(), c"G".as_ptr().cast(), c"T".as_ptr().cast()];
            check0!(test_test_vcf_api_c_51_check_alleles(
                rec,
                expected_alleles.as_mut_ptr(),
                3
            ));
            assert_eq!(
                hts::hts_itr_next(hts::hts_get_bgzfp(fp), iter, rec.cast(), hdr.cast()),
                -1
            );

            vcf::bcf_destroy(rec);
            crate::htslib_rs::hts::hts_itr_destroy(iter);
            hts_idx_destroy(idx);
            vcf::bcf_hdr_destroy(hdr);
            assert_eq!(hts_close(fp), 0);
        }

        cleanup_generated_bcf(&bcf_path);
    }

    #[test]
    fn original_test_vcf_api_bcf_to_vcf_writes_gzip_vcf_without_crashing() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_VCF_API_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let bcf_path = temp_path("bcf-to-vcf", "bcf");
        let out_path = temp_path("bcf-to-vcf-stdout", "out");
        cleanup_generated_bcf(&bcf_path);
        let bcf_path_c = c_path(&bcf_path);

        unsafe {
            test_test_vcf_api_c_110_write_bcf(bcf_path_c.as_ptr().cast_mut().cast::<u8>());
            assert_eq!(run_bcf_to_vcf_capture_stdout(&bcf_path, &out_path), 0);
        }

        let stdout_vcf = std::fs::read_to_string(&out_path).unwrap();
        assert!(stdout_vcf.starts_with("##fileformat=VCFv4.2\n"));
        assert!(stdout_vcf.contains(
            "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tNA00001\tNA00002\tNA00003\n"
        ));
        assert!(stdout_vcf.contains("20\t1110696\t.\tA\tG,T"));
        assert!(!stdout_vcf.contains("##unused=<ID=BB"));
        assert!(bcf_path.with_extension("bcf.gz").exists());

        cleanup_generated_bcf(&bcf_path);
        let _ = std::fs::remove_file(out_path);
    }

    #[test]
    fn original_test_vcf_api_main_writes_expected_stdout_vcf() {
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _guard = TEST_VCF_API_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let bcf_path = temp_path("main", "bcf");
        let out_path = temp_path("main-stdout", "out");
        cleanup_generated_bcf(&bcf_path);

        let mut args = vec![CString::new("test-vcf-api").unwrap(), c_path(&bcf_path)];
        unsafe {
            assert_eq!(run_main_capture_stdout(&mut args, &out_path), 0);
        }

        let stdout_vcf = std::fs::read_to_string(&out_path).unwrap();
        assert!(stdout_vcf.starts_with("##fileformat=VCFv4.2\n"));
        assert!(stdout_vcf.contains("20\t1110696\t.\tA\tG,T"));
        assert!(stdout_vcf.contains("20\t14370\trs6054257\tG\tA"));

        cleanup_generated_bcf(&bcf_path);
        let _ = std::fs::remove_file(out_path);
    }
}
