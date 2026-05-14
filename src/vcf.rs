use std::ffi::{c_char, c_int, c_void};
use std::mem::size_of;

use super::hts::{
    htsFile, hts_idx_t, hts_pos_t, i16_to_le, i32_to_le, kbitset_t, kbs_destroy, kbs_exists,
    kbs_init, kbs_insert, kputsn, kstring_t, le_to_float, le_to_i16, le_to_i32, le_to_i64,
    le_to_i8, toupper_c, BGZF,
};

pub type bcf_hdr_t = hts_sys::bcf_hdr_t;
pub type bcf1_t = hts_sys::bcf1_t;
pub type bcf_fmt_t = hts_sys::bcf_fmt_t;
pub type bcf_info_t = hts_sys::bcf_info_t;
pub type bcf_hrec_t = hts_sys::bcf_hrec_t;
pub type bcf_sr_regions_t = hts_sys::bcf_sr_regions_t;
pub type bcf_srs_t = hts_sys::bcf_srs_t;
pub type bcf_variant_t = hts_sys::variant_t;
pub type bcf_variant_match = c_int;

pub const VCF_INS: u32 = 1 << 6;
pub const VCF_DEL: u32 = 1 << 7;

#[repr(C)]
pub struct bcf_sweep_t {
    _private: [u8; 0],
}

unsafe extern "C" {
    #[link_name = "bcf_sweep_init"]
    fn htslib_bcf_sweep_init(fname: *const c_char) -> *mut bcf_sweep_t;
    #[link_name = "bcf_sweep_destroy"]
    fn htslib_bcf_sweep_destroy(sw: *mut bcf_sweep_t);
    #[link_name = "bcf_sweep_fwd"]
    fn htslib_bcf_sweep_fwd(sw: *mut bcf_sweep_t) -> *mut bcf1_t;
    #[link_name = "bcf_sweep_bwd"]
    fn htslib_bcf_sweep_bwd(sw: *mut bcf_sweep_t) -> *mut bcf1_t;
    #[link_name = "bcf_sweep_hdr"]
    fn htslib_bcf_sweep_hdr(sw: *mut bcf_sweep_t) -> *mut bcf_hdr_t;
    #[link_name = "bcf_sr_add_hreader"]
    fn htslib_bcf_sr_add_hreader(
        readers: *mut bcf_srs_t,
        file_ptr: *mut htsFile,
        autoclose: c_int,
        idxname: *const c_char,
    ) -> c_int;
    #[link_name = "vcf_open_mode"]
    fn htslib_vcf_open_mode(mode: *mut c_char, fn_: *const c_char, format: *const c_char) -> c_int;
    #[link_name = "bcf_strerror"]
    fn htslib_bcf_strerror(
        errorcode: c_int,
        buffer: *mut c_char,
        maxbuffer: usize,
    ) -> *const c_char;
    #[link_name = "bcf_format_gt_v2"]
    fn htslib_bcf_format_gt_v2(
        hdr: *const bcf_hdr_t,
        fmt: *mut bcf_fmt_t,
        isample: c_int,
        str_: *mut kstring_t,
    ) -> c_int;
}

pub unsafe fn bcf_hdr_init(mode: *const c_char) -> *mut bcf_hdr_t {
    hts_sys::bcf_hdr_init(mode)
}

pub unsafe fn bcf_hdr_destroy(h: *mut bcf_hdr_t) {
    hts_sys::bcf_hdr_destroy(h)
}

pub unsafe fn bcf_init() -> *mut bcf1_t {
    hts_sys::bcf_init()
}

pub unsafe fn bcf_destroy(v: *mut bcf1_t) {
    hts_sys::bcf_destroy(v)
}

pub unsafe fn bcf_empty(v: *mut bcf1_t) {
    hts_sys::bcf_empty(v)
}

pub unsafe fn bcf_clear(v: *mut bcf1_t) {
    hts_sys::bcf_clear(v)
}

pub unsafe fn bcf_sweep_init(fname: *const c_char) -> *mut bcf_sweep_t {
    unsafe { htslib_bcf_sweep_init(fname) }
}

pub unsafe fn bcf_sweep_destroy(sw: *mut bcf_sweep_t) {
    unsafe { htslib_bcf_sweep_destroy(sw) }
}

pub unsafe fn bcf_sweep_fwd(sw: *mut bcf_sweep_t) -> *mut bcf1_t {
    unsafe { htslib_bcf_sweep_fwd(sw) }
}

pub unsafe fn bcf_sweep_bwd(sw: *mut bcf_sweep_t) -> *mut bcf1_t {
    unsafe { htslib_bcf_sweep_bwd(sw) }
}

pub unsafe fn bcf_sweep_hdr(sw: *mut bcf_sweep_t) -> *mut bcf_hdr_t {
    unsafe { htslib_bcf_sweep_hdr(sw) }
}

pub unsafe fn bcf_hdr_read(fp: *mut htsFile) -> *mut bcf_hdr_t {
    hts_sys::bcf_hdr_read(fp.cast())
}

pub unsafe fn bcf_hdr_set_samples(
    hdr: *mut bcf_hdr_t,
    samples: *const c_char,
    is_file: c_int,
) -> c_int {
    hts_sys::bcf_hdr_set_samples(hdr, samples, is_file)
}

pub unsafe fn bcf_sr_add_hreader(
    readers: *mut bcf_srs_t,
    file_ptr: *mut htsFile,
    autoclose: c_int,
    idxname: *const c_char,
) -> c_int {
    unsafe { htslib_bcf_sr_add_hreader(readers, file_ptr, autoclose, idxname) }
}

pub unsafe fn bcf_subset_format(hdr: *const bcf_hdr_t, rec: *mut bcf1_t) -> c_int {
    hts_sys::bcf_subset_format(hdr, rec)
}

pub unsafe fn bcf_hdr_write(fp: *mut htsFile, h: *mut bcf_hdr_t) -> c_int {
    hts_sys::bcf_hdr_write(fp.cast(), h)
}

pub unsafe fn vcf_parse(s: *mut kstring_t, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    hts_sys::vcf_parse(s.cast(), h, v)
}

pub unsafe fn vcf_format(h: *const bcf_hdr_t, v: *const bcf1_t, s: *mut kstring_t) -> c_int {
    hts_sys::vcf_format(h, v, s.cast())
}

pub unsafe fn bcf_read(fp: *mut htsFile, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    hts_sys::bcf_read(fp.cast(), h, v)
}

pub unsafe fn vcf_open_mode(mode: *mut c_char, fn_: *const c_char, format: *const c_char) -> c_int {
    unsafe { htslib_vcf_open_mode(mode, fn_, format) }
}

pub unsafe fn bcf_unpack(b: *mut bcf1_t, which: c_int) -> c_int {
    hts_sys::bcf_unpack(b, which)
}

pub unsafe fn bcf_has_variant_type(rec: *mut bcf1_t, ith_allele: c_int, bitmask: u32) -> c_int {
    vcf_c_5501_bcf_has_variant_type(rec, ith_allele, bitmask)
}

pub unsafe fn bcf_variant_length(rec: *mut bcf1_t, ith_allele: c_int) -> c_int {
    vcf_c_5513_bcf_variant_length(rec, ith_allele)
}

pub unsafe fn bcf_has_variant_types(
    rec: *mut bcf1_t,
    bitmask: u32,
    mode: bcf_variant_match,
) -> c_int {
    vcf_c_5522_bcf_has_variant_types(rec, bitmask, mode)
}

pub unsafe fn vcf_c_5373_bcf_set_variant_type(
    ref_: *const c_char,
    alt: *const c_char,
    var: *mut bcf_variant_t,
) {
    if *alt == b'*' as c_char && *alt.add(1) == 0 {
        (*var).n = 0;
        (*var).type_ = hts_sys::VCF_OVERLAP as c_int;
        return;
    }

    if *ref_.add(1) == 0 && *alt.add(1) == 0 {
        if *alt == b'.' as c_char || *ref_ == *alt {
            (*var).n = 0;
            (*var).type_ = hts_sys::VCF_REF as c_int;
            return;
        }
        if *alt == b'X' as c_char {
            (*var).n = 0;
            (*var).type_ = hts_sys::VCF_REF as c_int;
            return;
        }
        (*var).n = 1;
        (*var).type_ = hts_sys::VCF_SNP as c_int;
        return;
    }

    if *alt == b'<' as c_char {
        if *alt.add(1) == b'X' as c_char && *alt.add(2) == b'>' as c_char {
            (*var).n = 0;
            (*var).type_ = hts_sys::VCF_REF as c_int;
            return;
        }
        if *alt.add(1) == b'*' as c_char && *alt.add(2) == b'>' as c_char {
            (*var).n = 0;
            (*var).type_ = hts_sys::VCF_REF as c_int;
            return;
        }
        if libc::strcmp(c"NON_REF>".as_ptr(), alt.add(1)) == 0 {
            (*var).n = 0;
            (*var).type_ = hts_sys::VCF_REF as c_int;
            return;
        }
        (*var).type_ = hts_sys::VCF_OTHER as c_int;
        return;
    }

    if *alt == b']' as c_char || *alt == b'[' as c_char {
        (*var).type_ = hts_sys::VCF_BND as c_int;
        return;
    }

    let mut r = ref_;
    let mut a = alt;
    while *r != 0 && *a != 0 && toupper_c(*r) == toupper_c(*a) {
        r = r.add(1);
        a = a.add(1);
    }

    if *a != 0 && *r == 0 {
        while *a != 0 {
            a = a.add(1);
        }
        if *a.sub(1) == b']' as c_char || *a.sub(1) == b'[' as c_char {
            (*var).type_ = hts_sys::VCF_BND as c_int;
            return;
        }
        (*var).n = a.offset_from(alt) as c_int - r.offset_from(ref_) as c_int;
        (*var).type_ = (hts_sys::VCF_INDEL | VCF_INS) as c_int;
        return;
    } else if *r != 0 && *a == 0 {
        while *r != 0 {
            r = r.add(1);
        }
        (*var).n = a.offset_from(alt) as c_int - r.offset_from(ref_) as c_int;
        (*var).type_ = (hts_sys::VCF_INDEL | VCF_DEL) as c_int;
        return;
    } else if *r == 0 && *a == 0 {
        (*var).n = 0;
        (*var).type_ = hts_sys::VCF_REF as c_int;
        return;
    }

    let mut re = r;
    let mut ae = a;
    while *re.add(1) != 0 {
        re = re.add(1);
    }
    while *ae.add(1) != 0 {
        ae = ae.add(1);
    }
    if *ae == b']' as c_char || *ae == b'[' as c_char {
        (*var).type_ = hts_sys::VCF_BND as c_int;
        return;
    }
    while re > r && ae > a && toupper_c(*re) == toupper_c(*ae) {
        re = re.sub(1);
        ae = ae.sub(1);
    }

    if ae == a {
        if re == r {
            (*var).n = 1;
            (*var).type_ = hts_sys::VCF_SNP as c_int;
            return;
        }
        (*var).n = -re.offset_from(r) as c_int;
        if toupper_c(*re) == toupper_c(*ae) {
            (*var).type_ = (hts_sys::VCF_INDEL | VCF_DEL) as c_int;
            return;
        }
        (*var).type_ = hts_sys::VCF_OTHER as c_int;
        return;
    } else if re == r {
        (*var).n = ae.offset_from(a) as c_int;
        if toupper_c(*re) == toupper_c(*ae) {
            (*var).type_ = (hts_sys::VCF_INDEL | VCF_INS) as c_int;
            return;
        }
        (*var).type_ = hts_sys::VCF_OTHER as c_int;
        return;
    }

    (*var).type_ = if re.offset_from(r) == ae.offset_from(a) {
        hts_sys::VCF_MNP as c_int
    } else {
        hts_sys::VCF_OTHER as c_int
    };
    (*var).n = if re.offset_from(r) > ae.offset_from(a) {
        -(re.offset_from(r) as c_int + 1)
    } else {
        ae.offset_from(a) as c_int + 1
    };
}

pub unsafe fn vcf_c_5444_bcf_set_variant_types(b: *mut bcf1_t) -> c_int {
    if (*b).unpacked & hts_sys::BCF_UN_STR as c_int == 0 {
        bcf_unpack(b, hts_sys::BCF_UN_STR as c_int);
    }
    let d = &mut (*b).d;
    if d.n_var < (*b).n_allele() as c_int {
        let new_var = libc::realloc(
            d.var.cast(),
            size_of::<bcf_variant_t>() * (*b).n_allele() as usize,
        )
        .cast::<bcf_variant_t>();
        if new_var.is_null() {
            return -1;
        }
        d.var = new_var;
        d.n_var = (*b).n_allele() as c_int;
    }
    (*b).d.var_type = 0;
    (*d.var).type_ = hts_sys::VCF_REF as c_int;
    (*d.var).n = 0;
    for i in 1..(*b).n_allele() as c_int {
        vcf_c_5373_bcf_set_variant_type(
            *d.allele,
            *d.allele.add(i as usize),
            d.var.add(i as usize),
        );
        (*b).d.var_type |= (*d.var.add(i as usize)).type_;
    }
    0
}

const ORIG_VAR_TYPES: u32 = hts_sys::VCF_SNP
    | hts_sys::VCF_MNP
    | hts_sys::VCF_INDEL
    | hts_sys::VCF_OTHER
    | hts_sys::VCF_BND
    | hts_sys::VCF_OVERLAP;

pub unsafe fn vcf_c_5474_bcf_get_variant_types(rec: *mut bcf1_t) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        hts_sys::hts_log(
            hts_sys::htsLogLevel_HTS_LOG_ERROR,
            c"bcf_get_variant_types".as_ptr(),
            c"Couldn't get variant types: %s".as_ptr(),
            libc::strerror(*libc::__errno_location()),
        );
        libc::exit(1);
    }
    (*rec).d.var_type & ORIG_VAR_TYPES as c_int
}

pub unsafe fn vcf_c_5485_bcf_get_variant_type(rec: *mut bcf1_t, ith_allele: c_int) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        hts_sys::hts_log(
            hts_sys::htsLogLevel_HTS_LOG_ERROR,
            c"bcf_get_variant_type".as_ptr(),
            c"Couldn't get variant types: %s".as_ptr(),
            libc::strerror(*libc::__errno_location()),
        );
        libc::exit(1);
    }
    if ith_allele < 0 || ith_allele >= (*rec).n_allele() as c_int {
        hts_sys::hts_log(
            hts_sys::htsLogLevel_HTS_LOG_ERROR,
            c"bcf_get_variant_type".as_ptr(),
            c"Requested allele outside valid range".as_ptr(),
        );
        libc::exit(1);
    }
    (*(*rec).d.var.add(ith_allele as usize)).type_ & ORIG_VAR_TYPES as c_int
}

pub unsafe fn vcf_c_5501_bcf_has_variant_type(
    rec: *mut bcf1_t,
    ith_allele: c_int,
    bitmask: u32,
) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        return -1;
    }
    if ith_allele < 0 || ith_allele >= (*rec).n_allele() as c_int {
        return -1;
    }
    if bitmask == hts_sys::VCF_REF {
        return ((*(*rec).d.var.add(ith_allele as usize)).type_ == hts_sys::VCF_REF as c_int)
            as c_int;
    }
    (bitmask as c_int) & (*(*rec).d.var.add(ith_allele as usize)).type_
}

pub unsafe fn vcf_c_5513_bcf_variant_length(rec: *mut bcf1_t, ith_allele: c_int) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        return hts_sys::bcf_int32_missing;
    }
    if ith_allele < 0 || ith_allele >= (*rec).n_allele() as c_int {
        return hts_sys::bcf_int32_missing;
    }
    (*(*rec).d.var.add(ith_allele as usize)).n
}

pub unsafe fn vcf_c_5522_bcf_has_variant_types(
    rec: *mut bcf1_t,
    bitmask: u32,
    mode: bcf_variant_match,
) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        return -1;
    }
    let mut type_ = (*rec).d.var_type as u32;
    if mode == 1 {
        return (bitmask & type_) as c_int;
    }

    if bitmask & (VCF_INS | VCF_DEL) != 0 && bitmask & hts_sys::VCF_INDEL == 0 {
        type_ &= !hts_sys::VCF_INDEL;
    } else if bitmask & hts_sys::VCF_INDEL != 0 && bitmask & (VCF_INS | VCF_DEL) == 0 {
        type_ &= !(VCF_INS | VCF_DEL);
    }

    if mode == 2 {
        if !bitmask & type_ != 0 {
            0
        } else {
            (bitmask & type_) as c_int
        }
    } else if bitmask == hts_sys::VCF_REF {
        (type_ == bitmask) as c_int
    } else if type_ == bitmask {
        type_ as c_int
    } else {
        0
    }
}

pub unsafe fn bcf_strerror(
    errorcode: c_int,
    buffer: *mut c_char,
    maxbuffer: usize,
) -> *const c_char {
    unsafe { htslib_bcf_strerror(errorcode, buffer, maxbuffer) }
}

pub unsafe fn bcf_format_gt_v2(
    hdr: *const bcf_hdr_t,
    fmt: *mut bcf_fmt_t,
    isample: c_int,
    str_: *mut kstring_t,
) -> c_int {
    unsafe { htslib_bcf_format_gt_v2(hdr, fmt, isample, str_) }
}

pub unsafe fn bcf_dup(src: *mut bcf1_t) -> *mut bcf1_t {
    hts_sys::bcf_dup(src)
}

pub unsafe fn bcf_copy(dst: *mut bcf1_t, src: *mut bcf1_t) -> *mut bcf1_t {
    hts_sys::bcf_copy(dst, src)
}

pub unsafe fn bcf_write(fp: *mut htsFile, h: *mut bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    hts_sys::bcf_write(fp.cast(), h, v)
}

pub unsafe fn vcf_hdr_read(fp: *mut htsFile) -> *mut bcf_hdr_t {
    hts_sys::vcf_hdr_read(fp.cast())
}

pub unsafe fn vcf_hdr_write(fp: *mut htsFile, h: *const bcf_hdr_t) -> c_int {
    hts_sys::vcf_hdr_write(fp.cast(), h)
}

pub unsafe fn vcf_read(fp: *mut htsFile, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    hts_sys::vcf_read(fp.cast(), h, v)
}

pub unsafe fn vcf_write(fp: *mut htsFile, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    hts_sys::vcf_write(fp.cast(), h, v)
}

pub unsafe fn bcf_readrec(
    fp: *mut BGZF,
    null: *mut c_void,
    v: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    hts_sys::bcf_readrec(fp.cast(), null, v, tid, beg, end)
}

pub unsafe fn vcf_write_line(fp: *mut htsFile, line: *mut kstring_t) -> c_int {
    hts_sys::vcf_write_line(fp.cast(), line.cast())
}

pub unsafe fn bcf_hdr_dup(hdr: *const bcf_hdr_t) -> *mut bcf_hdr_t {
    hts_sys::bcf_hdr_dup(hdr)
}

pub unsafe fn bcf_hdr_combine(dst: *mut bcf_hdr_t, src: *const bcf_hdr_t) -> c_int {
    hts_sys::bcf_hdr_combine(dst, src)
}

pub unsafe fn bcf_hdr_merge(dst: *mut bcf_hdr_t, src: *const bcf_hdr_t) -> *mut bcf_hdr_t {
    hts_sys::bcf_hdr_merge(dst, src)
}

pub unsafe fn bcf_translate(
    dst_hdr: *const bcf_hdr_t,
    src_hdr: *mut bcf_hdr_t,
    src_line: *mut bcf1_t,
) -> c_int {
    let line = src_line;
    if (*line).errcode != 0 {
        let mut errordescription = [0 as c_char; 1024];
        let error = bcf_strerror(
            (*line).errcode,
            errordescription.as_mut_ptr(),
            errordescription.len(),
        );
        hts_sys::hts_log(
            hts_sys::htsLogLevel_HTS_LOG_ERROR,
            c"bcf_translate".as_ptr(),
            c"Unchecked error (%d %s) at %s:%lld, exiting".as_ptr(),
            (*line).errcode,
            error,
            bcf_seqname_safe(src_hdr, line),
            (*line).pos + 1,
        );
        libc::exit(1);
    }

    if (*src_hdr).ntransl == -1 {
        return 0;
    }

    if (*src_hdr).ntransl == 0 {
        for dict in 0..2usize {
            let n = (*src_hdr).n[dict] as usize;
            (*src_hdr).transl[dict] =
                libc::malloc(n.saturating_mul(size_of::<c_int>())).cast::<c_int>();
            if n != 0 && (*src_hdr).transl[dict].is_null() {
                return -1;
            }

            for i in 0..n {
                let key = (*(*src_hdr).id[dict].add(i)).key;
                if key.is_null() {
                    *(*src_hdr).transl[dict].add(i) = -1;
                    continue;
                }
                let dst_id = bcf_hdr_id2int(dst_hdr, dict as c_int, key);
                *(*src_hdr).transl[dict].add(i) = dst_id;
                if dst_id != -1 && i as c_int != dst_id {
                    (*src_hdr).ntransl += 1;
                }
            }
        }

        if (*src_hdr).ntransl == 0 {
            libc::free((*src_hdr).transl[0].cast());
            libc::free((*src_hdr).transl[1].cast());
            (*src_hdr).transl[0] = std::ptr::null_mut();
            (*src_hdr).transl[1] = std::ptr::null_mut();
            (*src_hdr).ntransl = -1;
        }
        if (*src_hdr).ntransl == -1 {
            return 0;
        }
    }

    bcf_unpack(line, hts_sys::BCF_UN_ALL as c_int);

    let ctg_transl = (*src_hdr).transl[hts_sys::BCF_DT_CTG as usize];
    if !ctg_transl.is_null() && (*line).rid >= 0 {
        let dst_id = *ctg_transl.add((*line).rid as usize);
        if dst_id >= 0 {
            (*line).rid = dst_id;
        }
    }

    let id_transl = (*src_hdr).transl[hts_sys::BCF_DT_ID as usize];

    for i in 0..(*line).d.n_flt {
        let flt = (*line).d.flt.add(i as usize);
        let dst_id = *id_transl.add(*flt as usize);
        if dst_id >= 0 {
            *flt = dst_id;
        }
        (*line).d.shared_dirty |= hts_sys::BCF1_DIRTY_FLT as c_int;
    }

    for i in 0..(*line).n_info() {
        let info = (*line).d.info.add(i as usize);
        let src_id = (*info).key;
        let dst_id = *id_transl.add(src_id as usize);
        if dst_id < 0 {
            continue;
        }
        (*info).key = dst_id;
        if (*info).vptr.is_null() {
            continue;
        }

        let src_size = bcf_translate_id_size(src_id);
        let dst_size = bcf_translate_id_size(dst_id);
        if src_size == dst_size {
            let vptr = (*info).vptr.sub((*info).vptr_off() as usize);
            bcf_translate_store_id(vptr, dst_id, dst_size);
        } else {
            let mut str_ = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            if bcf_enc_int1(&mut str_, dst_id) < 0
                || bcf_enc_size(&mut str_, (*info).len, (*info).type_) < 0
                || kputsn((*info).vptr.cast(), (*info).vptr_len as usize, &mut str_) < 0
            {
                super::hts::ks_free(&mut str_);
                return -1;
            }
            let vptr_off = str_.l;
            if (*info).vptr_free() != 0 {
                libc::free((*info).vptr.sub((*info).vptr_off() as usize).cast());
            }
            (*info).set_vptr_off(vptr_off as u32);
            (*info).vptr = str_.s.cast::<u8>().add((*info).vptr_off() as usize);
            (*info).set_vptr_free(1);
            (*line).d.shared_dirty |= hts_sys::BCF1_DIRTY_INF as c_int;
        }
    }

    for i in 0..(*line).n_fmt() {
        let fmt = (*line).d.fmt.add(i as usize);
        let src_id = (*fmt).id;
        let dst_id = *id_transl.add(src_id as usize);
        if dst_id < 0 {
            continue;
        }
        (*fmt).id = dst_id;
        if (*fmt).p.is_null() {
            continue;
        }

        let src_size = bcf_translate_id_size(src_id);
        let dst_size = bcf_translate_id_size(dst_id);
        if src_size == dst_size {
            let p = (*fmt).p.sub((*fmt).p_off() as usize);
            bcf_translate_store_id(p.add(1), dst_id, dst_size);
        } else {
            let mut str_ = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            if bcf_enc_int1(&mut str_, dst_id) < 0
                || bcf_enc_size(&mut str_, (*fmt).n, (*fmt).type_) < 0
                || kputsn((*fmt).p.cast(), (*fmt).p_len as usize, &mut str_) < 0
            {
                super::hts::ks_free(&mut str_);
                return -1;
            }
            let p_off = str_.l;
            if (*fmt).p_free() != 0 {
                libc::free((*fmt).p.sub((*fmt).p_off() as usize).cast());
            }
            (*fmt).set_p_off(p_off as u32);
            (*fmt).p = str_.s.cast::<u8>().add((*fmt).p_off() as usize);
            (*fmt).set_p_free(1);
            (*line).d.indiv_dirty = 1;
        }
    }

    0
}

unsafe fn bcf_translate_id_size(id: c_int) -> c_int {
    if id >> 7 != 0 {
        if id >> 15 != 0 {
            hts_sys::BCF_BT_INT32 as c_int
        } else {
            hts_sys::BCF_BT_INT16 as c_int
        }
    } else {
        hts_sys::BCF_BT_INT8 as c_int
    }
}

unsafe fn bcf_translate_store_id(ptr: *mut u8, id: c_int, size: c_int) {
    if size == hts_sys::BCF_BT_INT8 as c_int {
        *ptr = id as u8;
    } else if size == hts_sys::BCF_BT_INT16 as c_int {
        i16_to_le(id as i16, ptr);
    } else {
        i32_to_le(id, ptr);
    }
}

unsafe fn bcf_enc_size(s: *mut kstring_t, size: c_int, type_: c_int) -> c_int {
    if size < 15 {
        if super::hts::ks_resize(s, (*s).l + 1) < 0 {
            return -1;
        }
        *(*s).s.add((*s).l) = ((size << 4) | type_) as c_char;
        (*s).l += 1;
        return 0;
    }

    if super::hts::ks_resize(s, (*s).l + 6) < 0 {
        return -1;
    }
    let p = (*s).s.add((*s).l).cast::<u8>();
    *p = ((15 << 4) | type_) as u8;
    if size < 128 {
        *p.add(1) = ((1 << 4) | hts_sys::BCF_BT_INT8 as c_int) as u8;
        *p.add(2) = size as u8;
        (*s).l += 3;
    } else if size < 32768 {
        *p.add(1) = ((1 << 4) | hts_sys::BCF_BT_INT16 as c_int) as u8;
        i16_to_le(size as i16, p.add(2));
        (*s).l += 4;
    } else {
        *p.add(1) = ((1 << 4) | hts_sys::BCF_BT_INT32 as c_int) as u8;
        i32_to_le(size, p.add(2));
        (*s).l += 6;
    }
    0
}

unsafe fn bcf_enc_int1(s: *mut kstring_t, x: c_int) -> c_int {
    if super::hts::ks_resize(s, (*s).l + 5) < 0 {
        return -1;
    }
    let p = (*s).s.add((*s).l).cast::<u8>();
    if x <= 0x7f && x >= -120 {
        *p = ((1 << 4) | hts_sys::BCF_BT_INT8 as c_int) as u8;
        *p.add(1) = x as u8;
        (*s).l += 2;
    } else if x <= 0x7fff && x >= -32760 {
        *p = ((1 << 4) | hts_sys::BCF_BT_INT16 as c_int) as u8;
        i16_to_le(x as i16, p.add(1));
        (*s).l += 3;
    } else {
        *p = ((1 << 4) | hts_sys::BCF_BT_INT32 as c_int) as u8;
        i32_to_le(x, p.add(1));
        (*s).l += 5;
    }
    0
}

pub unsafe fn bcf_hdr_subset(
    h0: *const bcf_hdr_t,
    n: c_int,
    samples: *const *mut c_char,
    imap: *mut c_int,
) -> *mut bcf_hdr_t {
    hts_sys::bcf_hdr_subset(h0, n, samples, imap)
}

pub unsafe fn bcf_hdr_add_sample(hdr: *mut bcf_hdr_t, sample: *const c_char) -> c_int {
    hts_sys::bcf_hdr_add_sample(hdr, sample)
}

pub unsafe fn bcf_hdr_set(hdr: *mut bcf_hdr_t, fname: *const c_char) -> c_int {
    hts_sys::bcf_hdr_set(hdr, fname)
}

pub unsafe fn bcf_hdr_format(hdr: *const bcf_hdr_t, is_bcf: c_int, str_: *mut kstring_t) -> c_int {
    hts_sys::bcf_hdr_format(hdr, is_bcf, str_.cast())
}

pub unsafe fn bcf_hdr_fmt_text(
    hdr: *const bcf_hdr_t,
    is_bcf: c_int,
    len: *mut c_int,
) -> *mut c_char {
    hts_sys::bcf_hdr_fmt_text(hdr, is_bcf, len)
}

pub unsafe fn bcf_hdr_append(h: *mut bcf_hdr_t, line: *const c_char) -> c_int {
    hts_sys::bcf_hdr_append(h, line)
}

pub unsafe fn bcf_hdr_get_version(hdr: *const bcf_hdr_t) -> *const c_char {
    hts_sys::bcf_hdr_get_version(hdr)
}

pub unsafe fn bcf_hdr_set_version(hdr: *mut bcf_hdr_t, version: *const c_char) -> c_int {
    hts_sys::bcf_hdr_set_version(hdr, version)
}

pub unsafe fn bcf_hdr_remove(h: *mut bcf_hdr_t, type_: c_int, key: *const c_char) {
    hts_sys::bcf_hdr_remove(h, type_, key)
}

pub unsafe fn bcf_hdr_seqnames(h: *const bcf_hdr_t, nseqs: *mut c_int) -> *mut *const c_char {
    hts_sys::bcf_hdr_seqnames(h, nseqs)
}

pub unsafe fn bcf_hdr_parse(hdr: *mut bcf_hdr_t, htxt: *mut c_char) -> c_int {
    hts_sys::bcf_hdr_parse(hdr, htxt)
}

pub unsafe fn bcf_hdr_sync(h: *mut bcf_hdr_t) -> c_int {
    hts_sys::bcf_hdr_sync(h)
}

pub unsafe fn bcf_hdr_parse_line(
    h: *const bcf_hdr_t,
    line: *const c_char,
    len: *mut c_int,
) -> *mut bcf_hrec_t {
    hts_sys::bcf_hdr_parse_line(h, line, len)
}

pub unsafe fn bcf_hrec_format(hrec: *const bcf_hrec_t, str_: *mut kstring_t) -> c_int {
    hts_sys::bcf_hrec_format(hrec, str_.cast())
}

pub unsafe fn bcf_hdr_add_hrec(hdr: *mut bcf_hdr_t, hrec: *mut bcf_hrec_t) -> c_int {
    hts_sys::bcf_hdr_add_hrec(hdr, hrec)
}

pub unsafe fn bcf_hdr_get_hrec(
    hdr: *const bcf_hdr_t,
    type_: c_int,
    key: *const c_char,
    value: *const c_char,
    str_class: *const c_char,
) -> *mut bcf_hrec_t {
    hts_sys::bcf_hdr_get_hrec(hdr, type_, key, value, str_class)
}

pub unsafe fn bcf_hrec_dup(hrec: *mut bcf_hrec_t) -> *mut bcf_hrec_t {
    hts_sys::bcf_hrec_dup(hrec)
}

pub unsafe fn bcf_hrec_add_key(hrec: *mut bcf_hrec_t, str_: *const c_char, len: usize) -> c_int {
    hts_sys::bcf_hrec_add_key(hrec, str_, len as hts_sys::size_t)
}

pub unsafe fn bcf_hrec_set_val(
    hrec: *mut bcf_hrec_t,
    i: c_int,
    str_: *const c_char,
    len: usize,
    is_quoted: c_int,
) -> c_int {
    hts_sys::bcf_hrec_set_val(hrec, i, str_, len as hts_sys::size_t, is_quoted)
}

pub unsafe fn bcf_hrec_find_key(hrec: *mut bcf_hrec_t, key: *const c_char) -> c_int {
    hts_sys::bcf_hrec_find_key(hrec, key)
}

pub unsafe fn hrec_add_idx(hrec: *mut bcf_hrec_t, idx: c_int) -> c_int {
    hts_sys::hrec_add_idx(hrec, idx)
}

pub unsafe fn bcf_hrec_destroy(hrec: *mut bcf_hrec_t) {
    hts_sys::bcf_hrec_destroy(hrec)
}

pub unsafe fn bcf_subset(h: *const bcf_hdr_t, v: *mut bcf1_t, n: c_int, imap: *mut c_int) -> c_int {
    hts_sys::bcf_subset(h, v, n, imap)
}

pub unsafe fn bcf_get_variant_types(rec: *mut bcf1_t) -> c_int {
    vcf_c_5474_bcf_get_variant_types(rec)
}

pub unsafe fn bcf_get_variant_type(rec: *mut bcf1_t, ith_allele: c_int) -> c_int {
    vcf_c_5485_bcf_get_variant_type(rec, ith_allele)
}

pub unsafe fn bcf_is_snp(v: *mut bcf1_t) -> c_int {
    hts_sys::bcf_is_snp(v)
}

pub unsafe fn bcf_update_filter(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    flt_ids: *mut c_int,
    n: c_int,
) -> c_int {
    hts_sys::bcf_update_filter(hdr, line, flt_ids, n)
}

pub unsafe fn bcf_add_filter(hdr: *const bcf_hdr_t, line: *mut bcf1_t, flt_id: c_int) -> c_int {
    hts_sys::bcf_add_filter(hdr, line, flt_id)
}

pub unsafe fn bcf_remove_filter(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    flt_id: c_int,
    pass: c_int,
) -> c_int {
    hts_sys::bcf_remove_filter(hdr, line, flt_id, pass)
}

pub unsafe fn bcf_has_filter(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    filter: *mut c_char,
) -> c_int {
    hts_sys::bcf_has_filter(hdr, line, filter)
}

pub unsafe fn bcf_update_alleles(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    alleles: *mut *const c_char,
    nals: c_int,
) -> c_int {
    hts_sys::bcf_update_alleles(hdr, line, alleles, nals)
}

pub unsafe fn bcf_update_alleles_str(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    alleles_string: *const c_char,
) -> c_int {
    hts_sys::bcf_update_alleles_str(hdr, line, alleles_string)
}

pub unsafe fn bcf_update_id(hdr: *const bcf_hdr_t, line: *mut bcf1_t, id: *const c_char) -> c_int {
    hts_sys::bcf_update_id(hdr, line, id)
}

pub unsafe fn bcf_add_id(hdr: *const bcf_hdr_t, line: *mut bcf1_t, id: *const c_char) -> c_int {
    hts_sys::bcf_add_id(hdr, line, id)
}

pub unsafe fn bcf_update_info(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
    values: *const c_void,
    n: c_int,
    type_: c_int,
) -> c_int {
    hts_sys::bcf_update_info(hdr, line, key, values, n, type_)
}

pub unsafe fn bcf_update_format_string(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
    values: *mut *const c_char,
    n: c_int,
) -> c_int {
    hts_sys::bcf_update_format_string(hdr, line, key, values, n)
}

pub unsafe fn bcf_update_format(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
    values: *const c_void,
    n: c_int,
    type_: c_int,
) -> c_int {
    hts_sys::bcf_update_format(hdr, line, key, values, n, type_)
}

pub unsafe fn bcf_get_fmt(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
) -> *mut bcf_fmt_t {
    hts_sys::bcf_get_fmt(hdr, line, key)
}

pub unsafe fn bcf_get_info(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
) -> *mut bcf_info_t {
    hts_sys::bcf_get_info(hdr, line, key)
}

pub unsafe fn bcf_get_format_string(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    tag: *const c_char,
    dst: *mut *mut *mut c_char,
    ndst: *mut c_int,
) -> c_int {
    hts_sys::bcf_get_format_string(hdr, line, tag, dst, ndst)
}

pub unsafe fn bcf_get_format_values(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    tag: *const c_char,
    dst: *mut *mut c_void,
    ndst: *mut c_int,
    type_: c_int,
) -> c_int {
    hts_sys::bcf_get_format_values(hdr, line, tag, dst, ndst, type_)
}

pub unsafe fn bcf_get_fmt_id(line: *mut bcf1_t, id: c_int) -> *mut bcf_fmt_t {
    hts_sys::bcf_get_fmt_id(line, id)
}

pub unsafe fn bcf_get_info_id(line: *mut bcf1_t, id: c_int) -> *mut bcf_info_t {
    hts_sys::bcf_get_info_id(line, id)
}

pub unsafe fn bcf_get_info_values(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    tag: *const c_char,
    dst: *mut *mut c_void,
    ndst: *mut c_int,
    type_: c_int,
) -> c_int {
    hts_sys::bcf_get_info_values(hdr, line, tag, dst, ndst, type_)
}

pub unsafe fn bcf_hdr_id2int(hdr: *const bcf_hdr_t, type_: c_int, id: *const c_char) -> c_int {
    hts_sys::bcf_hdr_id2int(hdr, type_, id)
}

pub unsafe fn bcf_hdr_name2id(hdr: *const bcf_hdr_t, id: *const c_char) -> c_int {
    bcf_hdr_id2int(hdr, hts_sys::BCF_DT_CTG as c_int, id)
}

pub unsafe fn bcf_hdr_id2name(hdr: *const bcf_hdr_t, rid: c_int) -> *const c_char {
    if hdr.is_null() || rid < 0 || rid >= (*hdr).n[hts_sys::BCF_DT_CTG as usize] {
        return std::ptr::null();
    }
    (*(*hdr).id[hts_sys::BCF_DT_CTG as usize].add(rid as usize)).key
}

pub unsafe fn bcf_seqname(hdr: *const bcf_hdr_t, rec: *const bcf1_t) -> *const c_char {
    bcf_hdr_id2name(hdr, if rec.is_null() { -1 } else { (*rec).rid })
}

pub unsafe fn bcf_seqname_safe(hdr: *const bcf_hdr_t, rec: *const bcf1_t) -> *const c_char {
    let name = bcf_seqname(hdr, rec);
    if name.is_null() {
        c"(unknown)".as_ptr()
    } else {
        name
    }
}

pub unsafe fn bcf_fmt_array(s: *mut kstring_t, n: c_int, type_: c_int, data: *mut c_void) -> c_int {
    hts_sys::bcf_fmt_array(s.cast(), n, type_, data)
}

pub unsafe fn bcf_fmt_sized_array(s: *mut kstring_t, ptr: *mut u8) -> *mut u8 {
    hts_sys::bcf_fmt_sized_array(s.cast(), ptr)
}

pub unsafe fn bcf_enc_vchar(s: *mut kstring_t, l: c_int, a: *const c_char) -> c_int {
    hts_sys::bcf_enc_vchar(s.cast(), l, a)
}

pub unsafe fn bcf_enc_vint(s: *mut kstring_t, n: c_int, a: *mut i32, wsize: c_int) -> c_int {
    hts_sys::bcf_enc_vint(s.cast(), n, a, wsize)
}

pub unsafe fn bcf_enc_vfloat(s: *mut kstring_t, n: c_int, a: *mut f32) -> c_int {
    hts_sys::bcf_enc_vfloat(s.cast(), n, a)
}

pub unsafe fn bcf_index_load2(fn_: *const c_char, fnidx: *const c_char) -> *mut hts_idx_t {
    hts_sys::bcf_index_load2(fn_, fnidx).cast()
}

pub unsafe fn bcf_index_load3(
    fn_: *const c_char,
    fnidx: *const c_char,
    flags: c_int,
) -> *mut hts_idx_t {
    hts_sys::bcf_index_load3(fn_, fnidx, flags).cast()
}

pub unsafe fn bcf_index_build(fn_: *const c_char, min_shift: c_int) -> c_int {
    hts_sys::bcf_index_build(fn_, min_shift)
}

pub unsafe fn bcf_index_build2(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
) -> c_int {
    hts_sys::bcf_index_build2(fn_, fnidx, min_shift)
}

pub unsafe fn bcf_index_build3(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
    n_threads: c_int,
) -> c_int {
    hts_sys::bcf_index_build3(fn_, fnidx, min_shift, n_threads)
}

pub unsafe fn bcf_idx_init(
    fp: *mut htsFile,
    h: *mut bcf_hdr_t,
    min_shift: c_int,
    fnidx: *const c_char,
) -> c_int {
    hts_sys::bcf_idx_init(fp.cast(), h, min_shift, fnidx)
}

pub unsafe fn bcf_idx_save(fp: *mut htsFile) -> c_int {
    hts_sys::bcf_idx_save(fp.cast())
}

pub unsafe fn vcfutils_c_254_is_special_info_type(name: *const c_char) -> c_int {
    match *name as u8 {
        b'C' => {
            if *name.add(1) == b'I' as c_char
                && (libc::strcmp(name.add(2), c"CN".as_ptr()) == 0
                    || libc::strcmp(name.add(2), c"END".as_ptr()) == 0
                    || libc::strcmp(name.add(2), c"LEN".as_ptr()) == 0
                    || libc::strcmp(name.add(2), c"POS".as_ptr()) == 0)
            {
                return 2;
            }
        }
        b'M' => {
            if *name.add(1) == b'E' as c_char
                && (libc::strcmp(name, c"MEINFO".as_ptr()) == 0
                    || libc::strcmp(name, c"METRANS".as_ptr()) == 0)
            {
                return 4;
            }
        }
        _ => {}
    }
    1
}

pub unsafe fn vcfutils_c_280_get_int32_info_value(info: *const bcf_info_t, index: usize) -> i32 {
    let len = if (*info).len > 0 {
        (*info).len as usize
    } else {
        0
    };
    if index >= len {
        return hts_sys::bcf_int32_missing;
    }

    match (*info).type_ {
        x if x == hts_sys::BCF_BT_INT8 as c_int => {
            let val = le_to_i8((*info).vptr.add(index)) as i32;
            if val > hts_sys::bcf_int8_vector_end {
                val
            } else {
                hts_sys::bcf_int32_vector_end - (hts_sys::bcf_int8_vector_end - val)
            }
        }
        x if x == hts_sys::BCF_BT_INT16 as c_int => {
            let val = le_to_i16((*info).vptr.add(index * size_of::<i16>())) as i32;
            if val > hts_sys::bcf_int16_vector_end {
                val
            } else {
                hts_sys::bcf_int32_vector_end - (hts_sys::bcf_int16_vector_end - val)
            }
        }
        x if x == hts_sys::BCF_BT_INT32 as c_int => {
            le_to_i32((*info).vptr.add(index * size_of::<i32>()))
        }
        x if x == hts_sys::BCF_BT_FLOAT as c_int => {
            let f = le_to_float((*info).vptr.add(index * size_of::<f32>()));
            if f.to_bits() == hts_sys::bcf_float_missing {
                hts_sys::bcf_int32_missing
            } else if f.to_bits() == hts_sys::bcf_float_vector_end {
                hts_sys::bcf_int32_vector_end
            } else {
                f as i32
            }
        }
        _ => hts_sys::bcf_int32_missing,
    }
}

pub unsafe fn vcfutils_c_315_get_rn_value(rn: *const bcf_info_t, index: usize) -> i32 {
    let val = if rn.is_null() {
        1
    } else {
        vcfutils_c_280_get_int32_info_value(rn, index)
    };
    if val >= 0 {
        val
    } else {
        0
    }
}

pub unsafe fn vcfutils_c_325_set_info_v1(info: *mut bcf_info_t) {
    match (*info).type_ {
        x if x == hts_sys::BCF_BT_INT8 as c_int => (*info).v1.i = le_to_i8((*info).vptr) as i64,
        x if x == hts_sys::BCF_BT_INT16 as c_int => (*info).v1.i = le_to_i16((*info).vptr) as i64,
        x if x == hts_sys::BCF_BT_INT32 as c_int => (*info).v1.i = le_to_i32((*info).vptr) as i64,
        x if x == hts_sys::BCF_BT_INT64 as c_int => (*info).v1.i = le_to_i64((*info).vptr),
        x if x == hts_sys::BCF_BT_FLOAT as c_int => (*info).v1.f = le_to_float((*info).vptr),
        _ => {}
    }
}

pub unsafe fn vcfutils_c_349_fixup_info_length_code(info: *mut bcf_info_t) -> c_int {
    const BCF_TYPE_SHIFT: [usize; 16] = [0, 0, 1, 2, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut buf = [0u8; 24];
    let mut ptr = buf.as_mut_ptr();

    let mut type_ = if (*info).key <= 0x7f && (*info).key >= -120 {
        hts_sys::BCF_BT_INT8 as c_int
    } else if (*info).key <= 0x7fff && (*info).key >= -32760 {
        hts_sys::BCF_BT_INT16 as c_int
    } else {
        hts_sys::BCF_BT_INT32 as c_int
    };
    *ptr = ((1 << 4) | type_) as u8;
    ptr = ptr.add(1);
    i32_to_le((*info).key, ptr);
    ptr = ptr.add(1 << BCF_TYPE_SHIFT[type_ as usize]);

    type_ = if (*info).len <= 0x7f && (*info).len >= -120 {
        hts_sys::BCF_BT_INT8 as c_int
    } else if (*info).len <= 0x7fff && (*info).len >= -32760 {
        hts_sys::BCF_BT_INT16 as c_int
    } else {
        hts_sys::BCF_BT_INT32 as c_int
    };
    if (*info).len < 15 {
        *ptr = (((*info).len << 4) | (*info).type_) as u8;
        ptr = ptr.add(1);
    } else {
        *ptr = (0xf0 | (*info).type_) as u8;
        ptr = ptr.add(1);
        *ptr = ((1 << 4) | type_) as u8;
        ptr = ptr.add(1);
        i32_to_le((*info).len, ptr);
        ptr = ptr.add(1 << BCF_TYPE_SHIFT[type_ as usize]);
    }

    let new_len = ptr.offset_from(buf.as_ptr()) as isize;
    let old_len = (*info).vptr_off() as isize;
    if new_len == old_len {
        libc::memcpy(
            (*info).vptr.offset(-old_len).cast(),
            buf.as_ptr().cast(),
            new_len as usize,
        );
    } else if new_len < old_len {
        let adjust = old_len - new_len;
        libc::memcpy(
            (*info).vptr.offset(-old_len).cast(),
            buf.as_ptr().cast(),
            new_len as usize,
        );
        libc::memmove(
            (*info).vptr.offset(-adjust).cast(),
            (*info).vptr.cast(),
            (*info).vptr_len as usize,
        );
        (*info).vptr = (*info).vptr.offset(-adjust);
        (*info).set_vptr_off(((*info).vptr_off() as isize - adjust) as u32);
    } else {
        let new_info = libc::malloc((*info).vptr_len as usize + new_len as usize).cast::<u8>();
        if new_info.is_null() {
            return -1;
        }
        libc::memcpy(new_info.cast(), buf.as_ptr().cast(), new_len as usize);
        libc::memcpy(
            new_info.add(new_len as usize).cast(),
            (*info).vptr.cast(),
            (*info).vptr_len as usize,
        );
        if (*info).vptr_free() != 0 {
            libc::free((*info).vptr.sub((*info).vptr_off() as usize).cast());
        }
        (*info).set_vptr_off(new_len as u32);
        (*info).vptr = new_info.add(new_len as usize);
        (*info).set_vptr_free(1);
    }
    0
}

pub unsafe fn vcfutils_c_407_mark_for_removal(info: *mut bcf_info_t) -> c_int {
    if (*info).vptr_free() != 0 {
        libc::free((*info).vptr.sub((*info).vptr_off() as usize).cast());
        (*info).set_vptr_free(0);
    }
    (*info).vptr = std::ptr::null_mut();
    (*info).set_vptr_off(0);
    (*info).vptr_len = 0;
    0
}

pub unsafe fn vcfutils_c_423_trim_int_cnv_tr_int_tags(
    info: *mut bcf_info_t,
    header: *const bcf_hdr_t,
    rm_set: *const hts_sys::kbitset_t,
    id: *const c_char,
    rn: *const bcf_info_t,
    ruc: *const bcf_info_t,
    num_alt_orig: usize,
    orig_total: usize,
) -> c_int {
    let count = if *id == b'C' as c_char {
        2usize
    } else {
        1usize
    };
    let key = (*info).key as usize;
    let type_ = ((*(*(*header).id[hts_sys::BCF_DT_ID as usize].add(key)).val).info
        [hts_sys::BCF_HL_INFO as usize]
        >> 4
        & 0xf) as c_int;
    let vlen = ((*(*(*header).id[hts_sys::BCF_DT_ID as usize].add(key)).val).info
        [hts_sys::BCF_HL_INFO as usize]
        >> 8
        & 0xf) as c_int;
    let element_sizes = [0usize, 1, 2, 4, 0, 4, 0, 0];
    let element_size = element_sizes[((*info).type_ & 0x7) as usize];
    let mut unit = 0usize;
    let mut orig_pos = 0usize;
    let mut new_pos = 0usize;
    let mut new_total = 0i32;

    if (type_ != hts_sys::BCF_HT_INT as c_int && type_ != hts_sys::BCF_HT_REAL as c_int)
        || element_size == 0
        || vlen != hts_sys::BCF_VL_VAR as c_int
        || (*info).len != orig_total as c_int
        || ((*info).vptr_len as usize)
            < orig_total
                .saturating_mul(element_size)
                .saturating_mul(count)
    {
        return 1;
    }

    for allele in 0..num_alt_orig {
        let mut n_repeats = vcfutils_c_315_get_rn_value(rn, allele);
        let mut n_items = n_repeats;
        if !ruc.is_null() {
            n_items = 0;
            while n_repeats > 0 {
                let n_units = vcfutils_c_280_get_int32_info_value(ruc, unit);
                n_items += if n_units >= 0 { n_units } else { 0 };
                unit += 1;
                n_repeats -= 1;
            }
        }

        let byte_len = (n_items as usize) * element_size * count;
        if kbs_exists(rm_set.cast::<kbitset_t>(), (allele + 1) as c_int) != 0 {
            orig_pos += byte_len;
            continue;
        }
        if new_pos < orig_pos {
            libc::memmove(
                (*info).vptr.add(new_pos).cast(),
                (*info).vptr.add(orig_pos).cast(),
                byte_len,
            );
        }
        orig_pos += byte_len;
        new_pos += byte_len;
        new_total += n_items;
    }

    if new_total == 0 {
        return vcfutils_c_407_mark_for_removal(info);
    }

    (*info).vptr_len = new_pos as u32;
    (*info).len = new_total;
    if (*info).len == 1 {
        vcfutils_c_325_set_info_v1(info);
    }
    vcfutils_c_349_fixup_info_length_code(info)
}

pub unsafe fn vcfutils_c_498_trim_int_cnv_tr_str_tags(
    info: *mut bcf_info_t,
    header: *const bcf_hdr_t,
    rm_set: *const hts_sys::kbitset_t,
    rn: *const bcf_info_t,
    num_alt_orig: usize,
    _orig_total: usize,
) -> c_int {
    let key = (*info).key as usize;
    let type_ = ((*(*(*header).id[hts_sys::BCF_DT_ID as usize].add(key)).val).info
        [hts_sys::BCF_HL_INFO as usize]
        >> 4
        & 0xf) as c_int;
    let vlen = ((*(*(*header).id[hts_sys::BCF_DT_ID as usize].add(key)).val).info
        [hts_sys::BCF_HL_INFO as usize]
        >> 8
        & 0xf) as c_int;
    let mut orig_pos = 0usize;
    let mut new_pos = 0usize;

    if type_ != hts_sys::BCF_HT_STR as c_int
        || (*info).type_ != hts_sys::BCF_BT_CHAR as c_int
        || vlen != hts_sys::BCF_VL_VAR as c_int
    {
        return 1;
    }

    for allele in 0..num_alt_orig {
        let mut n_items = vcfutils_c_315_get_rn_value(rn, allele);
        let start = (*info).vptr.add(orig_pos);
        let mut end = start;
        let lim = (*info).vptr.add((*info).vptr_len as usize);

        while n_items > 0 {
            while end < lim && *end != 0 && *end != b',' {
                end = end.add(1);
            }
            if end == lim || *end == 0 {
                break;
            }
            end = end.add(1);
            n_items -= 1;
        }

        let span = end.offset_from(start) as usize;
        if kbs_exists(rm_set.cast::<kbitset_t>(), (allele + 1) as c_int) != 0 {
            orig_pos += span;
            continue;
        }
        if new_pos < orig_pos {
            libc::memmove(
                (*info).vptr.add(new_pos).cast(),
                (*info).vptr.add(orig_pos).cast(),
                span,
            );
        }
        orig_pos += span;
        new_pos += span;
    }

    if new_pos == 0 {
        return vcfutils_c_407_mark_for_removal(info);
    }
    if new_pos < orig_pos {
        *(*info).vptr.add(new_pos) = 0;
        if new_pos > 0 && *(*info).vptr.add(new_pos - 1) == b',' {
            new_pos -= 1;
            *(*info).vptr.add(new_pos) = 0;
        }
        (*info).len = new_pos as c_int;
        (*info).vptr_len = new_pos as u32;
        return vcfutils_c_349_fixup_info_length_code(info);
    }
    0
}

pub unsafe fn vcfutils_c_561_fixup_cnv_tr_info_tags(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    num_alt_orig: usize,
    rm_set: *const hts_sys::kbitset_t,
) -> c_int {
    let rn = bcf_get_info(header, line, c"RN".as_ptr());
    let ruc = bcf_get_info(header, line, c"RUC".as_ptr());
    let mut orig_total_repeats = 0i64;
    let mut orig_total_units = 0i64;
    let mut unit = 0usize;

    for allele in 0..num_alt_orig {
        let mut n_repeats = vcfutils_c_315_get_rn_value(rn, allele);
        orig_total_repeats += n_repeats as i64;
        if !ruc.is_null() {
            while n_repeats > 0 {
                let n_units = vcfutils_c_280_get_int32_info_value(ruc, unit);
                orig_total_units += if n_units >= 0 { n_units as i64 } else { 0 };
                unit += 1;
                n_repeats -= 1;
            }
        }
    }

    for i in 0..(*line).n_info() {
        let info = (*line).d.info.add(i as usize);
        let id = (*(*header).id[hts_sys::BCF_DT_ID as usize].add((*info).key as usize)).key;
        let orig_ptr = (*info).vptr.sub((*info).vptr_off() as usize);
        if *id != b'C' as c_char && *id != b'R' as c_char {
            continue;
        }

        if libc::strcmp(id, c"RB".as_ptr()) == 0
            || libc::strcmp(id, c"RUL".as_ptr()) == 0
            || libc::strcmp(id, c"CIRB".as_ptr()) == 0
            || libc::strcmp(id, c"CIRUC".as_ptr()) == 0
        {
            let res = vcfutils_c_423_trim_int_cnv_tr_int_tags(
                info,
                header,
                rm_set,
                id,
                rn,
                std::ptr::null(),
                num_alt_orig,
                orig_total_repeats as usize,
            );
            if res < 0 {
                return res;
            }
        } else if libc::strcmp(id, c"RUS".as_ptr()) == 0 {
            let res = vcfutils_c_498_trim_int_cnv_tr_str_tags(
                info,
                header,
                rm_set,
                rn,
                num_alt_orig,
                orig_total_repeats as usize,
            );
            if res < 0 {
                return res;
            }
        } else if !ruc.is_null() && libc::strcmp(id, c"RUB".as_ptr()) == 0 {
            let res = vcfutils_c_423_trim_int_cnv_tr_int_tags(
                info,
                header,
                rm_set,
                id,
                rn,
                ruc,
                num_alt_orig,
                orig_total_units as usize,
            );
            if res < 0 {
                return res;
            }
        }

        if (*info).vptr.is_null() || (*info).vptr.sub((*info).vptr_off() as usize) != orig_ptr {
            (*line).d.shared_dirty |= hts_sys::BCF1_DIRTY_INF as c_int;
        }
    }

    if !ruc.is_null() {
        let res = vcfutils_c_423_trim_int_cnv_tr_int_tags(
            ruc,
            header,
            rm_set,
            c"RUC".as_ptr(),
            rn,
            std::ptr::null(),
            num_alt_orig,
            orig_total_repeats as usize,
        );
        if res < 0 {
            return res;
        }
    }
    0
}

pub unsafe fn bcf_trim_alleles(header: *const bcf_hdr_t, line: *mut bcf1_t) -> c_int {
    vcfutils_c_186_bcf_trim_alleles(header, line)
}

pub unsafe fn vcfutils_c_186_bcf_trim_alleles(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
) -> c_int {
    let mut ret = 0;
    let mut nrm = 0;
    let gt = bcf_get_fmt(header, line, c"GT".as_ptr());
    if gt.is_null() {
        return 0;
    }

    let ac = libc::calloc((*line).n_allele() as usize, size_of::<c_int>()).cast::<c_int>();
    if ac.is_null() {
        return -1;
    }

    for i in 0..(*line).n_sample() as usize {
        let p = (*gt).p.add(i * (*gt).size as usize);
        for ial in 0..(*gt).n as usize {
            let val = match (*gt).type_ {
                x if x == hts_sys::BCF_BT_INT8 as c_int => {
                    let v = le_to_i8(p.add(ial)) as c_int;
                    if v == hts_sys::bcf_int8_vector_end {
                        break;
                    }
                    v
                }
                x if x == hts_sys::BCF_BT_INT16 as c_int => {
                    let v = le_to_i16(p.add(ial * size_of::<i16>())) as c_int;
                    if v == hts_sys::bcf_int16_vector_end {
                        break;
                    }
                    v
                }
                x if x == hts_sys::BCF_BT_INT32 as c_int => {
                    let v = le_to_i32(p.add(ial * size_of::<i32>()));
                    if v == hts_sys::bcf_int32_vector_end {
                        break;
                    }
                    v
                }
                _ => {
                    ret = -1;
                    break;
                }
            };
            if val >> 1 == 0 {
                continue;
            }
            let allele = (val >> 1) - 1;
            if allele >= (*line).n_allele() as c_int {
                ret = -1;
                break;
            }
            *ac.add(allele as usize) += 1;
        }
        if ret != 0 {
            break;
        }
    }
    if ret != 0 {
        libc::free(ac.cast());
        return ret;
    }

    let rm_set = kbs_init((*line).n_allele() as usize);
    if rm_set.is_null() {
        libc::free(ac.cast());
        return -1;
    }
    for i in 1..(*line).n_allele() as c_int {
        if *ac.add(i as usize) == 0 {
            kbs_insert(rm_set, i);
            nrm += 1;
        }
    }

    if nrm != 0 && bcf_remove_allele_set(header, line, rm_set.cast()) != 0 {
        ret = -2;
    }

    libc::free(ac.cast());
    kbs_destroy(rm_set);
    if ret != 0 {
        ret
    } else {
        nrm
    }
}

pub unsafe fn bcf_remove_alleles(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    mask: c_int,
) -> c_int {
    vcfutils_c_241_bcf_remove_alleles(header, line, mask)
}

pub unsafe fn vcfutils_c_241_bcf_remove_alleles(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    mask: c_int,
) -> c_int {
    let rm_set = kbs_init((*line).n_allele() as usize);
    if rm_set.is_null() {
        return -1;
    }
    for i in 1..(*line).n_allele() as c_int {
        if (mask & (1 << i)) != 0 {
            kbs_insert(rm_set, i);
        }
    }
    bcf_remove_allele_set(header, line, rm_set.cast());
    kbs_destroy(rm_set);
    0
}

pub unsafe fn bcf_remove_allele_set(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    rm_set: *const hts_sys::kbitset_t,
) -> c_int {
    hts_sys::bcf_remove_allele_set(header, line, rm_set)
}

pub unsafe fn bcf_calc_ac(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    ac: *mut c_int,
    which: c_int,
) -> c_int {
    vcfutils_c_32_bcf_calc_ac(header, line, ac, which)
}

pub unsafe fn vcfutils_c_32_bcf_calc_ac(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    ac: *mut c_int,
    which: c_int,
) -> c_int {
    for i in 0..(*line).n_allele() as usize {
        *ac.add(i) = 0;
    }

    if (which & hts_sys::BCF_UN_INFO as c_int) != 0 {
        bcf_unpack(line, hts_sys::BCF_UN_INFO as c_int);
        let an_id = bcf_hdr_id2int(header, hts_sys::BCF_DT_ID as c_int, c"AN".as_ptr());
        let ac_id = bcf_hdr_id2int(header, hts_sys::BCF_DT_ID as c_int, c"AC".as_ptr());
        let mut an = -1;
        let mut ac_len = 0;
        let mut ac_type = 0;
        let mut ac_ptr: *mut u8 = std::ptr::null_mut();

        if an_id >= 0 && ac_id >= 0 {
            for i in 0..(*line).n_info() as usize {
                let z = (*line).d.info.add(i);
                if (*z).key == an_id {
                    an = (*z).v1.i as c_int;
                } else if (*z).key == ac_id {
                    ac_ptr = (*z).vptr;
                    ac_len = (*z).len;
                    ac_type = (*z).type_;
                }
            }
        }

        if an >= 0 && !ac_ptr.is_null() {
            if ac_len != (*line).n_allele() as c_int - 1 {
                return 0;
            }
            let mut nac = 0;
            for i in 0..ac_len as usize {
                let val = match ac_type {
                    x if x == hts_sys::BCF_BT_INT8 as c_int => le_to_i8(ac_ptr.add(i)) as c_int,
                    x if x == hts_sys::BCF_BT_INT16 as c_int => {
                        le_to_i16(ac_ptr.add(i * size_of::<i16>())) as c_int
                    }
                    x if x == hts_sys::BCF_BT_INT32 as c_int => {
                        le_to_i32(ac_ptr.add(i * size_of::<i32>()))
                    }
                    _ => libc::exit(1),
                };
                *ac.add(i + 1) = val;
                nac += val;
            }
            if an < nac {
                libc::exit(1);
            }
            *ac = an - nac;
            return 1;
        }
    }

    if (which & hts_sys::BCF_UN_FMT as c_int) != 0 {
        let gt_id = bcf_hdr_id2int(header, hts_sys::BCF_DT_ID as c_int, c"GT".as_ptr());
        if gt_id < 0 {
            return 0;
        }
        bcf_unpack(line, hts_sys::BCF_UN_FMT as c_int);

        let mut fmt_gt: *mut bcf_fmt_t = std::ptr::null_mut();
        for i in 0..(*line).n_fmt() as usize {
            let fmt = (*line).d.fmt.add(i);
            if (*fmt).id == gt_id {
                fmt_gt = fmt;
                break;
            }
        }
        if fmt_gt.is_null() {
            return 0;
        }

        for i in 0..(*line).n_sample() as usize {
            let p = (*fmt_gt).p.add(i * (*fmt_gt).size as usize);
            for ial in 0..(*fmt_gt).n as usize {
                let val = match (*fmt_gt).type_ {
                    x if x == hts_sys::BCF_BT_INT8 as c_int => {
                        let v = le_to_i8(p.add(ial)) as c_int;
                        if v == hts_sys::bcf_int8_vector_end {
                            break;
                        }
                        v
                    }
                    x if x == hts_sys::BCF_BT_INT16 as c_int => {
                        let v = le_to_i16(p.add(ial * size_of::<i16>())) as c_int;
                        if v == hts_sys::bcf_int16_vector_end {
                            break;
                        }
                        v
                    }
                    x if x == hts_sys::BCF_BT_INT32 as c_int => {
                        let v = le_to_i32(p.add(ial * size_of::<i32>()));
                        if v == hts_sys::bcf_int32_vector_end {
                            break;
                        }
                        v
                    }
                    _ => libc::exit(1),
                };
                if val >> 1 == 0 {
                    continue;
                }
                if val >> 1 > (*line).n_allele() as c_int {
                    libc::exit(1);
                }
                *ac.add(((val >> 1) - 1) as usize) += 1;
            }
        }
        return 1;
    }

    0
}

pub unsafe fn bcf_gt_type(
    fmt_ptr: *mut bcf_fmt_t,
    isample: c_int,
    ial: *mut c_int,
    jal: *mut c_int,
) -> c_int {
    vcfutils_c_134_bcf_gt_type(fmt_ptr, isample, ial, jal)
}

pub unsafe fn vcfutils_c_134_bcf_gt_type(
    fmt_ptr: *mut bcf_fmt_t,
    isample: c_int,
    ial_out: *mut c_int,
    jal_out: *mut c_int,
) -> c_int {
    let mut nals = 0;
    let mut has_ref = 0;
    let mut has_alt = 0;
    let mut ial = 0;
    let mut jal = 0;
    let p = (*fmt_ptr)
        .p
        .add(isample as usize * (*fmt_ptr).size as usize);

    for i in 0..(*fmt_ptr).n as usize {
        let val = match (*fmt_ptr).type_ {
            x if x == hts_sys::BCF_BT_INT8 as c_int => {
                let v = le_to_i8(p.add(i)) as c_int;
                if v == hts_sys::bcf_int8_vector_end {
                    break;
                }
                v
            }
            x if x == hts_sys::BCF_BT_INT16 as c_int => {
                let v = le_to_i16(p.add(i * size_of::<i16>())) as c_int;
                if v == hts_sys::bcf_int16_vector_end {
                    break;
                }
                v
            }
            x if x == hts_sys::BCF_BT_INT32 as c_int => {
                let v = le_to_i32(p.add(i * size_of::<i32>()));
                if v == hts_sys::bcf_int32_vector_end {
                    break;
                }
                v
            }
            _ => libc::exit(1),
        };
        if val >> 1 == 0 {
            return hts_sys::GT_UNKN as c_int;
        }
        let tmp = val >> 1;
        if tmp > 1 {
            if ial == 0 {
                ial = tmp;
                has_alt = 1;
            } else if tmp != ial {
                if tmp < ial {
                    jal = ial;
                    ial = tmp;
                } else {
                    jal = tmp;
                }
                has_alt = 2;
            }
        } else {
            has_ref = 1;
        }
        nals += 1;
    }

    if !ial_out.is_null() {
        *ial_out = if ial > 0 { ial - 1 } else { ial };
    }
    if !jal_out.is_null() {
        *jal_out = if jal > 0 { jal - 1 } else { jal };
    }
    if nals == 0 {
        return hts_sys::GT_UNKN as c_int;
    }
    if nals == 1 {
        return if has_ref != 0 {
            hts_sys::GT_HAPL_R
        } else {
            hts_sys::GT_HAPL_A
        } as c_int;
    }
    if has_ref == 0 {
        return if has_alt == 1 {
            hts_sys::GT_HOM_AA
        } else {
            hts_sys::GT_HET_AA
        } as c_int;
    }
    if has_alt == 0 {
        return hts_sys::GT_HOM_RR as c_int;
    }
    hts_sys::GT_HET_RA as c_int
}

pub unsafe fn bcf_sr_init() -> *mut bcf_srs_t {
    hts_sys::bcf_sr_init()
}

pub unsafe fn bcf_sr_destroy(readers: *mut bcf_srs_t) {
    hts_sys::bcf_sr_destroy(readers)
}

pub unsafe fn bcf_sr_strerror(errnum: c_int) -> *mut c_char {
    hts_sys::bcf_sr_strerror(errnum)
}

pub unsafe fn bcf_sr_set_threads(files: *mut bcf_srs_t, n_threads: c_int) -> c_int {
    hts_sys::bcf_sr_set_threads(files, n_threads)
}

pub unsafe fn bcf_sr_destroy_threads(files: *mut bcf_srs_t) {
    hts_sys::bcf_sr_destroy_threads(files)
}

pub unsafe fn bcf_sr_add_reader(readers: *mut bcf_srs_t, fname: *const c_char) -> c_int {
    hts_sys::bcf_sr_add_reader(readers, fname)
}

pub unsafe fn bcf_sr_remove_reader(files: *mut bcf_srs_t, i: c_int) {
    hts_sys::bcf_sr_remove_reader(files, i)
}

pub unsafe fn bcf_sr_next_line(readers: *mut bcf_srs_t) -> c_int {
    hts_sys::bcf_sr_next_line(readers)
}

pub unsafe fn bcf_sr_seek(readers: *mut bcf_srs_t, seq: *const c_char, pos: hts_pos_t) -> c_int {
    hts_sys::bcf_sr_seek(readers, seq, pos)
}

pub unsafe fn bcf_sr_set_samples(
    readers: *mut bcf_srs_t,
    samples: *const c_char,
    is_file: c_int,
) -> c_int {
    hts_sys::bcf_sr_set_samples(readers, samples, is_file)
}

pub unsafe fn bcf_sr_set_targets(
    readers: *mut bcf_srs_t,
    targets: *const c_char,
    is_file: c_int,
    alleles: c_int,
) -> c_int {
    hts_sys::bcf_sr_set_targets(readers, targets, is_file, alleles)
}

pub unsafe fn bcf_sr_set_regions(
    readers: *mut bcf_srs_t,
    regions: *const c_char,
    is_file: c_int,
) -> c_int {
    hts_sys::bcf_sr_set_regions(readers, regions, is_file)
}

pub unsafe fn bcf_sr_regions_init(
    regions: *const c_char,
    is_file: c_int,
    chr: c_int,
    from: c_int,
    to: c_int,
) -> *mut bcf_sr_regions_t {
    hts_sys::bcf_sr_regions_init(regions, is_file, chr, from, to)
}

pub unsafe fn bcf_sr_regions_destroy(regions: *mut bcf_sr_regions_t) {
    hts_sys::bcf_sr_regions_destroy(regions)
}

pub unsafe fn bcf_sr_regions_seek(regions: *mut bcf_sr_regions_t, chr: *const c_char) -> c_int {
    hts_sys::bcf_sr_regions_seek(regions, chr)
}

pub unsafe fn bcf_sr_regions_next(reg: *mut bcf_sr_regions_t) -> c_int {
    hts_sys::bcf_sr_regions_next(reg)
}

pub unsafe fn bcf_sr_regions_overlap(
    reg: *mut bcf_sr_regions_t,
    seq: *const c_char,
    start: hts_pos_t,
    end: hts_pos_t,
) -> c_int {
    hts_sys::bcf_sr_regions_overlap(reg, seq, start, end)
}

pub unsafe fn bcf_sr_regions_flush(regs: *mut bcf_sr_regions_t) -> c_int {
    hts_sys::bcf_sr_regions_flush(regs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vcf_header_wrappers_accept_hts_sys_headers() {
        unsafe {
            let hdr = hts_sys::bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            let dup = bcf_hdr_dup(hdr);
            assert!(!dup.is_null());
            hts_sys::bcf_hdr_destroy(dup);
            hts_sys::bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_inline_name_helpers_match_contig_table() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);
            assert_eq!(bcf_hdr_name2id(hdr, c"chr1".as_ptr()), 0);
            assert_eq!(bcf_hdr_name2id(hdr, c"missing".as_ptr()), -1);
            assert_eq!(std::ffi::CStr::from_ptr(bcf_hdr_id2name(hdr, 0)), c"chr1");
            assert!(bcf_hdr_id2name(hdr, -1).is_null());
            assert!(bcf_hdr_id2name(hdr, 1).is_null());

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = 0;
            assert_eq!(std::ffi::CStr::from_ptr(bcf_seqname(hdr, rec)), c"chr1");
            (*rec).rid = 99;
            assert_eq!(
                std::ffi::CStr::from_ptr(bcf_seqname_safe(hdr, rec)),
                c"(unknown)"
            );
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcfutils_calc_ac_uses_info_and_gt_counts() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=AN,Number=1,Type=Integer,Description=\"AN\">".as_ptr()
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=AC,Number=A,Type=Integer,Description=\"AC\">".as_ptr()
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"GT\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_add_sample(hdr, c"S1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr, c"S2".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr, std::ptr::null()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(
                    c"chr1\t1\t.\tA\tC,G\t.\t.\tAN=4;AC=1,2\tGT\t0/1\t2/2".as_ptr(),
                    &mut line,
                ) >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);

            let mut ac = [0; 3];
            assert_eq!(
                vcfutils_c_32_bcf_calc_ac(hdr, rec, ac.as_mut_ptr(), hts_sys::BCF_UN_INFO as c_int),
                1
            );
            assert_eq!(ac, [1, 1, 2]);

            let mut gt_ac = [0; 3];
            assert_eq!(
                vcfutils_c_32_bcf_calc_ac(
                    hdr,
                    rec,
                    gt_ac.as_mut_ptr(),
                    hts_sys::BCF_UN_FMT as c_int
                ),
                1
            );
            assert_eq!(gt_ac, [1, 1, 2]);

            let fmt = bcf_get_fmt(hdr, rec, c"GT".as_ptr());
            assert!(!fmt.is_null());
            let mut ial = -1;
            let mut jal = -1;
            assert_eq!(
                vcfutils_c_134_bcf_gt_type(fmt, 0, &mut ial, &mut jal),
                hts_sys::GT_HET_RA as c_int
            );
            assert_eq!(ial, 1);
            assert_eq!(jal, 0);
            assert_eq!(
                vcfutils_c_134_bcf_gt_type(fmt, 1, &mut ial, &mut jal),
                hts_sys::GT_HOM_AA as c_int
            );
            assert_eq!(ial, 2);

            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn bcf_translate_rewrites_record_ids_for_destination_header() {
        unsafe {
            let hdr1 = bcf_hdr_init(c"w".as_ptr());
            let mut hdr2 = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr1.is_null());
            assert!(!hdr2.is_null());

            for (hdr, line) in [
                (hdr1, c"##contig=<ID=1>".as_ptr()),
                (hdr1, c"##contig=<ID=2>".as_ptr()),
                (hdr2, c"##contig=<ID=2>".as_ptr()),
                (hdr2, c"##contig=<ID=1>".as_ptr()),
                (
                    hdr1,
                    c"##FILTER=<ID=FLT1,Description=\"Filter 1\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FILTER=<ID=FLT2,Description=\"Filter 2\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FILTER=<ID=FLT3,Description=\"Filter 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FILTER=<ID=FLT4,Description=\"Filter 4\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FILTER=<ID=FLT3,Description=\"Filter 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FILTER=<ID=FLT2,Description=\"Filter 2\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##INFO=<ID=INF1,Number=.,Type=Integer,Description=\"Info 1\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##INFO=<ID=INF2,Number=.,Type=Integer,Description=\"Info 2\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##INFO=<ID=INF3,Number=.,Type=Integer,Description=\"Info 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##INFO=<ID=INF4,Number=.,Type=Integer,Description=\"Info 4\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##INFO=<ID=INF3,Number=.,Type=Integer,Description=\"Info 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##INFO=<ID=INF2,Number=.,Type=Integer,Description=\"Info 2\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FORMAT=<ID=FMT1,Number=.,Type=Integer,Description=\"FMT 1\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FORMAT=<ID=FMT2,Number=.,Type=Integer,Description=\"FMT 2\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FORMAT=<ID=FMT3,Number=.,Type=Integer,Description=\"FMT 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FORMAT=<ID=FMT4,Number=.,Type=Integer,Description=\"FMT 4\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FORMAT=<ID=FMT3,Number=.,Type=Integer,Description=\"FMT 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FORMAT=<ID=FMT2,Number=.,Type=Integer,Description=\"FMT 2\">".as_ptr(),
                ),
            ] {
                assert_eq!(bcf_hdr_append(hdr, line), 0);
            }

            assert_eq!(bcf_hdr_add_sample(hdr1, c"SMPL1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr1, c"SMPL2".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr2, c"SMPL1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr2, c"SMPL2".as_ptr()), 0);
            assert_eq!(bcf_hdr_sync(hdr1), 0);
            assert_eq!(bcf_hdr_sync(hdr2), 0);
            hdr2 = bcf_hdr_merge(hdr2, hdr1);
            assert!(!hdr2.is_null());
            assert_eq!(bcf_hdr_sync(hdr2), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = bcf_hdr_name2id(hdr1, c"1".as_ptr());
            (*rec).pos = 0;
            assert_eq!(bcf_update_alleles_str(hdr1, rec, c"G,A".as_ptr()), 0);

            let mut ids = [
                bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as c_int, c"FLT1".as_ptr()),
                bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as c_int, c"FLT2".as_ptr()),
                bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as c_int, c"FLT3".as_ptr()),
            ];
            assert_eq!(bcf_update_filter(hdr1, rec, ids.as_mut_ptr(), 3), 0);

            let mut tmp = [1, 2];
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    c"INF1".as_ptr(),
                    tmp.as_ptr().cast(),
                    1,
                    hts_sys::BCF_HT_INT as c_int,
                ),
                0
            );
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    c"INF2".as_ptr(),
                    tmp.as_ptr().add(1).cast(),
                    1,
                    hts_sys::BCF_HT_INT as c_int,
                ),
                0
            );
            tmp[0] = 3;
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    c"INF3".as_ptr(),
                    tmp.as_ptr().cast(),
                    1,
                    hts_sys::BCF_HT_INT as c_int,
                ),
                0
            );
            tmp = [1, 1];
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    c"FMT1".as_ptr(),
                    tmp.as_ptr().cast(),
                    2,
                    hts_sys::BCF_HT_INT as c_int,
                ),
                0
            );
            tmp = [2, 2];
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    c"FMT2".as_ptr(),
                    tmp.as_ptr().cast(),
                    2,
                    hts_sys::BCF_HT_INT as c_int,
                ),
                0
            );
            tmp = [3, 3];
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    c"FMT3".as_ptr(),
                    tmp.as_ptr().cast(),
                    2,
                    hts_sys::BCF_HT_INT as c_int,
                ),
                0
            );

            assert_eq!(
                bcf_remove_filter(
                    hdr1,
                    rec,
                    bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as c_int, c"FLT2".as_ptr()),
                    0,
                ),
                0
            );
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    c"INF2".as_ptr(),
                    std::ptr::null(),
                    0,
                    hts_sys::BCF_HT_INT as c_int,
                ),
                0
            );
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    c"FMT2".as_ptr(),
                    std::ptr::null(),
                    0,
                    hts_sys::BCF_HT_INT as c_int,
                ),
                0
            );

            assert_eq!(bcf_translate(hdr2, hdr1, rec), 0);
            assert_eq!((*rec).rid, bcf_hdr_name2id(hdr2, c"1".as_ptr()));

            let mut out = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(vcf_format(hdr2, rec, &mut out), 0);
            assert_eq!(
                std::ffi::CStr::from_ptr(out.s),
                c"1\t1\t.\tG\tA\t0\tFLT1;FLT3\tINF1=1;INF3=3\tFMT1:FMT3\t1:3\t1:3\n"
            );

            super::super::hts::ks_free(&mut out);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr1);
            bcf_hdr_destroy(hdr2);
        }
    }

    #[test]
    fn vcf_variant_type_classifiers_match_htslib_branches() {
        unsafe {
            let mut var = bcf_variant_t { type_: -1, n: -1 };

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"C".as_ptr(), &mut var);
            assert_eq!(var.type_, hts_sys::VCF_SNP as c_int);
            assert_eq!(var.n, 1);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"A".as_ptr(), &mut var);
            assert_eq!(var.type_, hts_sys::VCF_REF as c_int);
            assert_eq!(var.n, 0);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"AC".as_ptr(), &mut var);
            assert_eq!(var.type_, (hts_sys::VCF_INDEL | VCF_INS) as c_int);
            assert_eq!(var.n, 1);

            vcf_c_5373_bcf_set_variant_type(c"AT".as_ptr(), c"A".as_ptr(), &mut var);
            assert_eq!(var.type_, (hts_sys::VCF_INDEL | VCF_DEL) as c_int);
            assert_eq!(var.n, -1);

            vcf_c_5373_bcf_set_variant_type(c"AC".as_ptr(), c"GT".as_ptr(), &mut var);
            assert_eq!(var.type_, hts_sys::VCF_MNP as c_int);
            assert_eq!(var.n, 2);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"*".as_ptr(), &mut var);
            assert_eq!(var.type_, hts_sys::VCF_OVERLAP as c_int);
            assert_eq!(var.n, 0);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"<NON_REF>".as_ptr(), &mut var);
            assert_eq!(var.type_, hts_sys::VCF_REF as c_int);
            assert_eq!(var.n, 0);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"]chr1:10]A".as_ptr(), &mut var);
            assert_eq!(var.type_, hts_sys::VCF_BND as c_int);
        }
    }

    #[test]
    fn vcf_set_variant_types_populates_record_variant_array() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(c"chr1\t1\t.\tA\tC,AC,*\t.\t.\t.".as_ptr(), &mut line,)
                    >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);
            assert_eq!(vcf_c_5444_bcf_set_variant_types(rec), 0);
            assert_eq!((*rec).d.n_var, 4);
            assert_eq!((*(*rec).d.var).type_, hts_sys::VCF_REF as c_int);
            assert_eq!((*(*rec).d.var.add(1)).type_, hts_sys::VCF_SNP as c_int);
            assert_eq!(
                (*(*rec).d.var.add(2)).type_,
                (hts_sys::VCF_INDEL | VCF_INS) as c_int
            );
            assert_eq!((*(*rec).d.var.add(3)).type_, hts_sys::VCF_OVERLAP as c_int);
            assert_eq!(
                (*rec).d.var_type,
                (hts_sys::VCF_SNP | hts_sys::VCF_INDEL | VCF_INS | hts_sys::VCF_OVERLAP) as c_int
            );

            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn synced_bcf_reader_wrappers_create_and_destroy_reader_set() {
        unsafe {
            let readers = bcf_sr_init();
            assert!(!readers.is_null());
            assert!(!bcf_sr_strerror(0).is_null());
            assert_eq!(bcf_sr_set_threads(readers, 0), 0);
            bcf_sr_destroy_threads(readers);
            bcf_sr_destroy(readers);
        }
    }

    #[test]
    fn vcfutils_trim_and_remove_alleles_update_records() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"GT\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_add_sample(hdr, c"S1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr, std::ptr::null()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(
                    c"chr1\t1\t.\tA\tC,G\t.\t.\t.\tGT\t0/1".as_ptr(),
                    &mut line,
                ) >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);

            assert_eq!(vcfutils_c_186_bcf_trim_alleles(hdr, rec), 1);
            assert_eq!((*rec).n_allele(), 2);
            assert_eq!(std::ffi::CStr::from_ptr(*(*rec).d.allele.add(1)), c"C");

            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }
}
