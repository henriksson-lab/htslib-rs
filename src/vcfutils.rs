// Functions translated from htslib/vcfutils.c.

use std::ffi::{c_char, c_int, c_void};
use std::mem::size_of;

use crate::htslib_rs::hts::{
    hts_log_cstr, i32_to_le, kbitset_t, kbs_destroy, kbs_exists, kbs_init, kbs_insert, kputc,
    kputs, kputsn, kstring_t, le_to_float, le_to_i16, le_to_i32, le_to_i64, le_to_i8,
    HTS_LOG_ERROR,
};
use crate::htslib_rs::vcf::{
    self, bcf1_t, bcf_float_missing, bcf_float_vector_end, bcf_fmt_t, bcf_get_fmt, bcf_get_info,
    bcf_hdr_id2int, bcf_hdr_t, bcf_info_t, bcf_int16_missing, bcf_int16_vector_end,
    bcf_int32_missing, bcf_int32_vector_end, bcf_int8_missing, bcf_int8_vector_end, bcf_unpack,
    BCF1_DIRTY_INF, BCF_BT_CHAR, BCF_BT_FLOAT, BCF_BT_INT16, BCF_BT_INT32, BCF_BT_INT64,
    BCF_BT_INT8, BCF_DT_ID, BCF_HL_FMT, BCF_HL_INFO, BCF_HT_FLAG, BCF_HT_INT, BCF_HT_REAL,
    BCF_HT_STR, BCF_UN_ALL, BCF_UN_FMT, BCF_UN_INFO, BCF_VL_A, BCF_VL_G, BCF_VL_R, BCF_VL_VAR,
    GT_HAPL_A, GT_HAPL_R, GT_HET_AA, GT_HET_RA, GT_HOM_AA, GT_HOM_RR, GT_UNKN,
};

const BCF_VL_LA: c_int = 6;
const BCF_VL_LG: c_int = 7;
const BCF_VL_LR: c_int = 8;

struct OwnedKString {
    raw: kstring_t,
}

impl OwnedKString {
    fn new() -> Self {
        Self {
            raw: kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
        }
    }
}

impl std::ops::Deref for OwnedKString {
    type Target = kstring_t;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl std::ops::DerefMut for OwnedKString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

impl Drop for OwnedKString {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.raw.s.cast());
        }
    }
}

struct CBuffer<T> {
    ptr: *mut T,
}

impl<T> CBuffer<T> {
    fn new() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }

    fn cast<U>(&self) -> *mut U {
        self.ptr.cast::<U>()
    }

    unsafe fn add(&self, count: usize) -> *mut T {
        self.ptr.add(count)
    }

    fn as_mut_c_void_dst(&mut self) -> *mut *mut c_void {
        (&mut self.ptr as *mut *mut T).cast::<*mut c_void>()
    }
}

impl<T> Drop for CBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr.cast());
        }
    }
}

// original: bcf_remove_allele_set (htslib/vcfutils.c:659)
pub unsafe fn vcfutils_c_659_bcf_remove_allele_set(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    rm_set: *const kbitset_t,
) -> c_int {
    let vl_a_g_r = (1_u32 << BCF_VL_A) | (1_u32 << BCF_VL_G) | (1_u32 << BCF_VL_R);
    let vl_la_lg_lr = (1_u32 << BCF_VL_LA) | (1_u32 << BCF_VL_LG) | (1_u32 << BCF_VL_LR);
    let vl_a_g_r_la_lg_lr = vl_a_g_r | vl_la_lg_lr;
    let vl_a_r = (1_u32 << BCF_VL_A) | (1_u32 << BCF_VL_R);
    let vl_la_lr = (1_u32 << BCF_VL_LA) | (1_u32 << BCF_VL_LR);
    let vl_a_r_la_lr = vl_a_r | vl_la_lr;
    let n_allele = (*line).n_allele() as usize;
    let mut map = Vec::new();
    if map.try_reserve_exact(n_allele).is_err() {
        return -1;
    }
    map.resize(n_allele, 0);
    let mut laa = CBuffer::<c_int>::new();
    let mut laa_map = Vec::new();
    let mut lr_orig = Vec::new();
    let mut dat = CBuffer::<u8>::new();
    let mut laa_size = 0;
    let mut laa_map_stride = 0;
    let mut have_cnv_tr = 0;

    vcf::bcf_unpack(line, BCF_UN_ALL as c_int);

    let n_sample = (*line).n_sample() as c_int;
    let mut str_ = OwnedKString::new();

    macro_rules! err {
        () => {{
            return -1;
        }};
    }

    kputs(*(*line).d.allele.add(0), &mut *str_);

    let mut nrm = 0;
    map[0] = 0;
    let mut j = 1;
    for i in 1..(*line).n_allele() as c_int {
        if libc::strcmp(*(*line).d.allele.add(i as usize), c"<CNV:TR>".as_ptr()) == 0 {
            have_cnv_tr = 1;
        }

        if kbs_exists(rm_set, i) != 0 {
            *(*line).d.allele.add(i as usize) = std::ptr::null_mut();
            map[i as usize] = -1;
            nrm += 1;
            continue;
        }
        kputc(b',' as c_int, &mut *str_);
        kputs(*(*line).d.allele.add(i as usize), &mut *str_);
        map[i as usize] = j;
        j += 1;
    }
    if nrm == 0 {
        return 0;
    }

    let n_r_ori = (*line).n_allele() as c_int;
    let n_r_new = (*line).n_allele() as c_int - nrm;
    if n_r_new <= 0 {
        let seqname = vcf::bcf_seqname_safe(header, line);
        let seqname_str = if seqname.is_null() {
            std::borrow::Cow::Borrowed("")
        } else {
            std::ffi::CStr::from_ptr(seqname).to_string_lossy()
        };
        let msg = std::ffi::CString::new(format!(
            "Cannot remove reference allele at {}:{} [{}]",
            seqname_str,
            (*line).pos + 1,
            n_r_new
        ))
        .unwrap_or_default();
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"bcf_remove_allele_set".as_ptr(),
            msg.as_ptr(),
        );
        return -1;
    }
    let n_a_ori = n_r_ori - 1;
    let n_a_new = n_r_new - 1;
    let n_g_ori = n_r_ori * (n_r_ori + 1) / 2;
    let mut n_g_new = n_r_new * (n_r_new + 1) / 2;

    if vcf::bcf_update_alleles_str(header, line, str_.s) < 0 {
        return -1;
    }

    if have_cnv_tr != 0
        && vcfutils_c_561_fixup_cnv_tr_info_tags(header, line, n_a_ori as usize, rm_set.cast()) < 0
    {
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"bcf_remove_allele_set".as_ptr(),
            c"Out of memory".as_ptr(),
        );
        return -1;
    }

    let mut mdat_bytes = 0;
    for i in 0..(*line).n_info() {
        let info = (*line).d.info.add(i as usize);
        let info_key = (*info).key as usize;
        let id = (*(*header).id[BCF_DT_ID as usize].add(info_key)).key;
        let mut vlen = ((*(*(*header).id[BCF_DT_ID as usize].add(info_key)).val).info
            [BCF_HL_INFO as usize]
            >> 8
            & 0xf) as c_int;

        let multiple = if vlen == BCF_VL_VAR as c_int {
            vcfutils_c_254_is_special_info_type(id)
        } else {
            1
        };
        if multiple > 1 {
            vlen = BCF_VL_A as c_int;
        }

        if (vl_a_g_r & (1_u32 << vlen)) == 0 {
            continue;
        }
        let type_ = ((*(*(*header).id[BCF_DT_ID as usize].add(info_key)).val).info
            [BCF_HL_INFO as usize]
            >> 4
            & 0xf) as c_int;
        if type_ == BCF_HT_FLAG as c_int {
            continue;
        }
        let size = if type_ == BCF_HT_REAL as c_int || type_ == BCF_HT_INT as c_int {
            4
        } else {
            1
        };

        let mut mdat = mdat_bytes / size;
        let mut nret =
            vcf::bcf_get_info_values(header, line, id, dat.as_mut_c_void_dst(), &mut mdat, type_);
        mdat_bytes = mdat * size;
        if nret < 0 {
            err!();
        }
        if nret == 0 {
            continue;
        }

        if type_ == BCF_HT_STR as c_int {
            str_.l = 0;
            let mut ss = dat.cast::<c_char>();
            let mut se = dat.cast::<c_char>();
            let s0 = *ss;
            if vlen == BCF_VL_A as c_int || vlen == BCF_VL_R as c_int {
                let mut inc = 0;
                let nexp = if vlen == BCF_VL_A as c_int {
                    inc = 1;
                    n_a_ori * multiple
                } else {
                    n_r_ori
                };
                let mut jj = 0;
                while jj < nexp {
                    if *se == 0 {
                        break;
                    }
                    while *se != 0 && *se != b',' as c_char {
                        se = se.add(1);
                    }
                    if kbs_exists(rm_set, jj / multiple + inc) != 0 {
                        if *se != 0 {
                            se = se.add(1);
                        }
                        ss = se;
                        jj += 1;
                        continue;
                    }
                    if str_.l != 0 {
                        kputc(b',' as c_int, &mut *str_);
                    }
                    kputsn(ss, se.offset_from(ss) as usize, &mut *str_);
                    if *se != 0 {
                        se = se.add(1);
                    }
                    ss = se;
                    jj += 1;
                }
                if jj == 1 && s0 == b'.' as c_char {
                    continue;
                }
                if jj != nexp {
                    err!();
                }
            } else {
                let mut n = 0;
                for jj in 0..n_r_ori {
                    for k in 0..=jj {
                        if *se == 0 {
                            break;
                        }
                        while *se != 0 && *se != b',' as c_char {
                            se = se.add(1);
                        }
                        n += 1;
                        if kbs_exists(rm_set, jj) != 0 || kbs_exists(rm_set, k) != 0 {
                            if *se != 0 {
                                se = se.add(1);
                            }
                            ss = se;
                            continue;
                        }
                        if str_.l != 0 {
                            kputc(b',' as c_int, &mut *str_);
                        }
                        kputsn(ss, se.offset_from(ss) as usize, &mut *str_);
                        if *se != 0 {
                            se = se.add(1);
                        }
                        ss = se;
                    }
                    if *se == 0 {
                        break;
                    }
                }
                if n == 1 && s0 == b'.' as c_char {
                    continue;
                }
                if n != n_g_ori {
                    err!();
                }
            }
            nret = vcf::bcf_update_info(header, line, id, str_.s.cast(), str_.l as c_int, type_);
            if nret < 0 {
                err!();
            }
            continue;
        }

        if nret == 1 {
            let missing = match (*info).type_ {
                x if x == BCF_BT_INT8 as c_int => {
                    *(*info).vptr.cast::<i8>() as c_int == bcf_int8_missing
                }
                x if x == BCF_BT_INT16 as c_int => {
                    // `(*info).vptr` points into a packed BCF record byte buffer
                    // whose alignment is not guaranteed for multi-byte ints, so
                    // read via `read_unaligned` to avoid misaligned-pointer UB.
                    i16::from_le(core::ptr::read_unaligned((*info).vptr.cast::<i16>())) as c_int
                        == bcf_int16_missing
                }
                x if x == BCF_BT_INT32 as c_int => {
                    // See above: vptr is a packed-record byte pointer; use
                    // `read_unaligned` to avoid misaligned-pointer UB.
                    i32::from_le(core::ptr::read_unaligned((*info).vptr.cast::<i32>()))
                        == bcf_int32_missing
                }
                x if x == BCF_BT_FLOAT as c_int => {
                    // See above: vptr is a packed-record byte pointer; use
                    // `read_unaligned` to avoid misaligned-pointer UB.
                    f32::from_bits(u32::from_le(core::ptr::read_unaligned(
                        (*info).vptr.cast::<u32>(),
                    )))
                    .to_bits()
                        == bcf_float_missing
                }
                _ => {
                    err!();
                }
            };
            if missing {
                continue;
            }
        }

        let ndat;
        if vlen == BCF_VL_A as c_int || vlen == BCF_VL_R as c_int {
            let (inc, ntop, new_ndat) = if vlen == BCF_VL_A as c_int {
                if nret != n_a_ori * multiple {
                    err!();
                }
                (1, n_a_ori * multiple, n_a_new * multiple)
            } else {
                if nret != n_r_ori {
                    err!();
                }
                (0, n_r_ori, n_r_new)
            };
            ndat = new_ndat;
            let mut k = 0;
            if type_ == BCF_HT_INT as c_int {
                let ptr = dat.cast::<i32>();
                for jj in 0..ntop {
                    if *ptr.add(jj as usize) == bcf_int32_vector_end {
                        *ptr.add(k as usize) = *ptr.add(jj as usize);
                        break;
                    }
                    if kbs_exists(rm_set, jj / multiple + inc) != 0 {
                        continue;
                    }
                    if jj != k {
                        *ptr.add(k as usize) = *ptr.add(jj as usize);
                    }
                    k += 1;
                }
            } else if type_ == BCF_HT_REAL as c_int {
                let ptr = dat.cast::<f32>();
                for jj in 0..ntop {
                    if (*ptr.add(jj as usize)).to_bits() == bcf_float_vector_end {
                        *ptr.add(k as usize) = *ptr.add(jj as usize);
                        break;
                    }
                    if kbs_exists(rm_set, jj / multiple + inc) != 0 {
                        continue;
                    }
                    if jj != k {
                        *ptr.add(k as usize) = *ptr.add(jj as usize);
                    }
                    k += 1;
                }
            }
        } else {
            if nret != n_g_ori {
                err!();
            }
            ndat = n_g_new;
            let mut l_ori: c_int = -1;
            let mut l_new = 0;
            if type_ == BCF_HT_INT as c_int {
                let ptr = dat.cast::<i32>();
                for jj in 0..n_r_ori {
                    for k in 0..=jj {
                        l_ori += 1;
                        if *ptr.add(l_ori as usize) == bcf_int32_vector_end {
                            *ptr.add(l_new as usize) = *ptr.add(l_ori as usize);
                            break;
                        }
                        if kbs_exists(rm_set, jj) != 0 || kbs_exists(rm_set, k) != 0 {
                            continue;
                        }
                        if l_ori != l_new {
                            *ptr.add(l_new as usize) = *ptr.add(l_ori as usize);
                        }
                        l_new += 1;
                    }
                }
            } else if type_ == BCF_HT_REAL as c_int {
                let ptr = dat.cast::<f32>();
                for jj in 0..n_r_ori {
                    for k in 0..=jj {
                        l_ori += 1;
                        if (*ptr.add(l_ori as usize)).to_bits() == bcf_float_vector_end {
                            *ptr.add(l_new as usize) = *ptr.add(l_ori as usize);
                            break;
                        }
                        if kbs_exists(rm_set, jj) != 0 || kbs_exists(rm_set, k) != 0 {
                            continue;
                        }
                        if l_ori != l_new {
                            *ptr.add(l_new as usize) = *ptr.add(l_ori as usize);
                        }
                        l_new += 1;
                    }
                }
            }
        }
        nret = vcf::bcf_update_info(header, line, id, dat.cast(), ndat, type_);
        if nret < 0 {
            err!();
        }
    }

    let mut i = 1;
    while i < n_r_ori {
        if *map.as_ptr().add(i as usize) != i {
            break;
        }
        i += 1;
    }
    if i < n_r_ori {
        let mut mdat = mdat_bytes / 4;
        let mut nret = vcf::bcf_get_format_values(
            header,
            line,
            c"GT".as_ptr(),
            dat.as_mut_c_void_dst(),
            &mut mdat,
            BCF_HT_INT as c_int,
        );
        mdat_bytes = mdat * 4;
        if nret > 0 {
            nret /= n_sample;
            let mut ptr = dat.cast::<i32>();
            for _sample in 0..n_sample {
                for jj in 0..nret {
                    let v = *ptr.add(jj as usize);
                    if (v >> 1) == 0 {
                        continue;
                    }
                    if v == bcf_int32_vector_end {
                        break;
                    }
                    let al = (v >> 1) - 1;
                    if !(al < n_r_ori && *map.as_ptr().add(al as usize) >= -1) {
                        err!();
                    }
                    *ptr.add(jj as usize) = if *map.as_ptr().add(al as usize) < 0 {
                        0
                    } else {
                        ((*map.as_ptr().add(al as usize) + 1) << 1) | (v & 1)
                    };
                }
                ptr = ptr.add(nret as usize);
            }
            if vcf::bcf_update_format(
                header,
                line,
                c"GT".as_ptr(),
                dat.cast(),
                nret * n_sample,
                BCF_HT_INT as c_int,
            ) < 0
            {
                err!();
            }
        }
    }

    let num_laa = vcf::bcf_get_format_values(
        header,
        line,
        c"LAA".as_ptr(),
        laa.as_mut_c_void_dst(),
        &mut laa_size,
        BCF_HT_INT as c_int,
    );
    if num_laa < -1 && num_laa != -3 {
        err!();
    }
    if num_laa > 0 {
        let num_laa_vals = num_laa / n_sample;
        laa_map_stride = num_laa_vals + 1;
        let mut max_k = 0;
        let Some(laa_map_len) = (laa_map_stride as usize).checked_mul(n_sample as usize) else {
            err!();
        };
        if laa_map.try_reserve_exact(laa_map_len).is_err() {
            err!();
        }
        laa_map.resize(laa_map_len, 0);
        if lr_orig.try_reserve_exact(n_sample as usize).is_err() {
            err!();
        }
        lr_orig.resize(n_sample as usize, 0);
        let mut laa_changed = 0;
        for sample in 0..n_sample {
            let sample_laa = laa.add((sample * num_laa_vals) as usize);
            let sample_laa_map = laa_map.as_mut_ptr().add((sample * laa_map_stride) as usize);
            *sample_laa_map = 0;
            let mut k = 0;
            let mut jj = 0;
            while jj < num_laa_vals {
                let val = *sample_laa.add(jj as usize);
                if val == bcf_int32_vector_end || val == bcf_int32_missing {
                    break;
                }
                let allele = if val > 0 && val < n_r_ori { val } else { 0 };
                if allele == 0 || *map.as_ptr().add(allele as usize) < 0 {
                    *sample_laa_map.add(jj as usize + 1) = -1;
                    laa_changed = 1;
                    jj += 1;
                    continue;
                }
                if allele != *map.as_ptr().add(allele as usize) {
                    laa_changed = 1;
                }
                *sample_laa.add(k as usize) = *map.as_ptr().add(allele as usize);
                k += 1;
                *sample_laa_map.add(jj as usize + 1) = k;
                jj += 1;
            }
            lr_orig[sample as usize] = jj + 1;
            if max_k < k {
                max_k = k;
            }
            while jj < num_laa_vals {
                *sample_laa_map.add(jj as usize + 1) = -1;
                jj += 1;
            }
            while k < num_laa_vals {
                *sample_laa.add(k as usize) = if k > 0 {
                    bcf_int32_vector_end
                } else {
                    bcf_int32_missing
                };
                k += 1;
            }
        }
        if laa_changed != 0 {
            let mut new_num_laa = num_laa;
            if max_k < num_laa_vals {
                if max_k > 0 {
                    for sample in 1..n_sample {
                        libc::memmove(
                            laa.add((sample * max_k) as usize).cast(),
                            laa.add((sample * num_laa_vals) as usize).cast(),
                            max_k as usize * size_of::<c_int>(),
                        );
                    }
                    new_num_laa = n_sample * max_k;
                } else {
                    for sample in 0..n_sample {
                        *laa.add(sample as usize) = bcf_int32_missing;
                    }
                    new_num_laa = n_sample;
                }
            }
            if vcf::bcf_update_format(
                header,
                line,
                c"LAA".as_ptr(),
                laa.cast(),
                new_num_laa,
                BCF_HT_INT as c_int,
            ) < 0
            {
                err!();
            }
        }
    }

    for fmt_i in 0..(*line).n_fmt() {
        let fmt = (*line).d.fmt.add(fmt_i as usize);
        let fmt_id = (*fmt).id as usize;
        let id = (*(*header).id[BCF_DT_ID as usize].add(fmt_id)).key;
        let vlen =
            ((*(*(*header).id[BCF_DT_ID as usize].add(fmt_id)).val).info[BCF_HL_FMT as usize] >> 8
                & 0xf) as c_int;

        if (vl_a_g_r_la_lg_lr & (1_u32 << vlen)) == 0 {
            continue;
        }
        let is_local = ((vl_la_lg_lr & (1_u32 << vlen)) != 0) as c_int;
        let type_ =
            ((*(*(*header).id[BCF_DT_ID as usize].add(fmt_id)).val).info[BCF_HL_FMT as usize] >> 4
                & 0xf) as c_int;
        if type_ == BCF_HT_FLAG as c_int {
            continue;
        }
        let size = if type_ == BCF_HT_REAL as c_int || type_ == BCF_HT_INT as c_int {
            4
        } else {
            1
        };

        let mut mdat = mdat_bytes / size;
        let mut nret =
            vcf::bcf_get_format_values(header, line, id, dat.as_mut_c_void_dst(), &mut mdat, type_);
        mdat_bytes = mdat * size;
        if nret < 0 {
            err!();
        }
        if nret == 0 {
            continue;
        }

        if type_ == BCF_HT_STR as c_int {
            let width = nret / n_sample;
            str_.l = 0;
            if (vl_a_r_la_lr & (1_u32 << vlen)) != 0 {
                let mut nexp = 0;
                let mut inc = 0;
                match vlen {
                    x if x == BCF_VL_A as c_int => {
                        nexp = n_a_ori;
                        inc = 1;
                    }
                    x if x == BCF_VL_R as c_int => nexp = n_r_ori,
                    x if x == BCF_VL_LA => {
                        inc = 1;
                        if laa_map.is_empty() {
                            err!();
                        }
                    }
                    x if x == BCF_VL_LR => {
                        if laa_map.is_empty() {
                            err!();
                        }
                    }
                    _ => {}
                }
                for sample in 0..n_sample {
                    let mut ss = dat.add((sample * width) as usize).cast::<c_char>();
                    let se = ss.add(width as usize);
                    let mut ptr = ss;
                    let s0 = *ss;
                    let mut k_dst = 0;
                    let l0 = str_.l;
                    let sample_map = if is_local != 0 {
                        laa_map.as_mut_ptr().add((sample * laa_map_stride) as usize)
                    } else {
                        map.as_mut_ptr()
                    };
                    if is_local != 0 {
                        nexp = lr_orig[sample as usize] - inc;
                    }
                    let mut k_src = 0;
                    while k_src < nexp {
                        if ptr >= se || *ptr == 0 {
                            break;
                        }
                        while ptr < se && *ptr != 0 && *ptr != b',' as c_char {
                            ptr = ptr.add(1);
                        }
                        if *sample_map.add((k_src + inc) as usize) < 0 {
                            ptr = ptr.add(1);
                            ss = ptr;
                            k_src += 1;
                            continue;
                        }
                        if k_dst != 0 {
                            kputc(b',' as c_int, &mut *str_);
                        }
                        kputsn(ss, ptr.offset_from(ss) as usize, &mut *str_);
                        ptr = ptr.add(1);
                        ss = ptr;
                        k_dst += 1;
                        k_src += 1;
                    }
                    if k_src != nexp && !(k_src == 1 && s0 == b'.' as c_char) {
                        err!();
                    }
                    let mut l = str_.l - l0;
                    while l < width as usize {
                        kputc(if l == 0 { b'.' } else { 0 } as c_int, &mut *str_);
                        l += 1;
                    }
                }
            } else {
                for sample in 0..n_sample {
                    let mut ss = dat.add((sample * width) as usize).cast::<c_char>();
                    let se = ss.add(width as usize);
                    let mut ptr = ss;
                    let s0 = *ss;
                    let mut k_dst = 0;
                    let l0 = str_.l;
                    let mut nexp = 0;
                    let sample_n_r_ori = if is_local != 0 {
                        lr_orig[sample as usize]
                    } else {
                        n_r_ori
                    };
                    let sample_n_g_ori = if is_local != 0 {
                        sample_n_r_ori * (sample_n_r_ori + 1) / 2
                    } else {
                        n_g_ori
                    };
                    let sample_map = if is_local != 0 {
                        laa_map.as_mut_ptr().add((sample * laa_map_stride) as usize)
                    } else {
                        map.as_mut_ptr()
                    };
                    while ptr < se {
                        if *ptr == 0 {
                            break;
                        }
                        if *ptr == b',' as c_char {
                            nexp += 1;
                        }
                        ptr = ptr.add(1);
                    }
                    if ptr != ss {
                        nexp += 1;
                    }
                    if nexp != sample_n_g_ori
                        && nexp != sample_n_r_ori
                        && !(nexp == 1 && s0 == b'.' as c_char)
                    {
                        err!();
                    }
                    ptr = ss;
                    if nexp == 1 && s0 == b'.' as c_char {
                        kputc(b'.' as c_int, &mut *str_);
                    } else if nexp == sample_n_g_ori {
                        for ia in 0..sample_n_r_ori {
                            for ib in 0..=ia {
                                if ptr >= se || *ptr == 0 {
                                    break;
                                }
                                while ptr < se && *ptr != 0 && *ptr != b',' as c_char {
                                    ptr = ptr.add(1);
                                }
                                if *sample_map.add(ia as usize) < 0
                                    || *sample_map.add(ib as usize) < 0
                                {
                                    ptr = ptr.add(1);
                                    ss = ptr;
                                    continue;
                                }
                                if k_dst != 0 {
                                    kputc(b',' as c_int, &mut *str_);
                                }
                                kputsn(ss, ptr.offset_from(ss) as usize, &mut *str_);
                                ptr = ptr.add(1);
                                ss = ptr;
                                k_dst += 1;
                            }
                            if ptr >= se || *ptr == 0 {
                                break;
                            }
                        }
                    } else {
                        let mut k_src = 0;
                        while k_src < sample_n_r_ori {
                            if ptr >= se || *ptr == 0 {
                                break;
                            }
                            while ptr < se && *ptr != 0 && *ptr != b',' as c_char {
                                ptr = ptr.add(1);
                            }
                            if *sample_map.add(k_src as usize) < 0 {
                                ptr = ptr.add(1);
                                ss = ptr;
                                k_src += 1;
                                continue;
                            }
                            if k_dst != 0 {
                                kputc(b',' as c_int, &mut *str_);
                            }
                            kputsn(ss, ptr.offset_from(ss) as usize, &mut *str_);
                            ptr = ptr.add(1);
                            ss = ptr;
                            k_dst += 1;
                            k_src += 1;
                        }
                        if k_src != n_r_ori {
                            err!();
                        }
                    }
                    let mut l = str_.l - l0;
                    while l < width as usize {
                        kputc(0, &mut *str_);
                        l += 1;
                    }
                }
            }
            nret = vcf::bcf_update_format(header, line, id, str_.s.cast(), str_.l as c_int, type_);
            if nret < 0 {
                err!();
            }
            continue;
        }

        let nori = nret / n_sample;
        if nori == 1 && !(vlen == BCF_VL_A as c_int && nori == n_a_ori) {
            let mut all_missing = 1;
            match (*fmt).type_ {
                x if x == BCF_BT_INT8 as c_int => {
                    for sample in 0..n_sample {
                        let val = *(*fmt).p.add((sample * (*fmt).size) as usize).cast::<i8>();
                        if val as c_int != bcf_int8_missing {
                            all_missing = 0;
                            break;
                        }
                    }
                }
                x if x == BCF_BT_INT16 as c_int => {
                    for sample in 0..n_sample {
                        // `(*fmt).p` indexes into a packed BCF record byte
                        // buffer whose alignment is not guaranteed for
                        // multi-byte ints; use `read_unaligned` to avoid UB.
                        let val = i16::from_le(core::ptr::read_unaligned(
                            (*fmt).p.add((sample * (*fmt).size) as usize).cast::<i16>(),
                        ));
                        if val as c_int != bcf_int16_missing {
                            all_missing = 0;
                            break;
                        }
                    }
                }
                x if x == BCF_BT_INT32 as c_int => {
                    for sample in 0..n_sample {
                        // See above: packed-record byte pointer, use `read_unaligned`.
                        let val = i32::from_le(core::ptr::read_unaligned(
                            (*fmt).p.add((sample * (*fmt).size) as usize).cast::<i32>(),
                        ));
                        if val != bcf_int32_missing {
                            all_missing = 0;
                            break;
                        }
                    }
                }
                x if x == BCF_BT_FLOAT as c_int => {
                    for sample in 0..n_sample {
                        // See above: packed-record byte pointer, use `read_unaligned`.
                        let val = u32::from_le(core::ptr::read_unaligned(
                            (*fmt).p.add((sample * (*fmt).size) as usize).cast::<u32>(),
                        ));
                        if val != bcf_float_missing {
                            all_missing = 0;
                            break;
                        }
                    }
                }
                _ => {
                    err!();
                }
            }
            if all_missing != 0 {
                continue;
            }
        }

        let ndat;
        if (vl_a_r_la_lr & (1_u32 << vlen)) != 0 || (vlen == BCF_VL_G as c_int && nori == n_r_ori) {
            let mut inc = 0;
            let nnew;
            match vlen {
                x if x == BCF_VL_A as c_int => {
                    if nori != n_a_ori {
                        err!();
                    }
                    ndat = n_a_new * n_sample;
                    nnew = n_a_new;
                    inc = 1;
                }
                x if x == BCF_VL_R as c_int => {
                    if nori != n_r_ori {
                        err!();
                    }
                    ndat = n_r_new * n_sample;
                    nnew = n_r_new;
                }
                x if x == BCF_VL_G as c_int => {
                    ndat = n_r_new * n_sample;
                    nnew = n_r_new;
                }
                x if x == BCF_VL_LA => {
                    inc = 1;
                    if laa_map.is_empty() {
                        err!();
                    }
                    nnew = nori;
                    ndat = nori * n_sample;
                }
                x if x == BCF_VL_LR => {
                    if laa_map.is_empty() {
                        err!();
                    }
                    nnew = nori;
                    ndat = nori * n_sample;
                }
                _ => {
                    ndat = nret;
                    nnew = nori;
                }
            }

            if type_ == BCF_HT_INT as c_int {
                for sample in 0..n_sample {
                    let ptr_src = dat.cast::<i32>().add((sample * nori) as usize);
                    let ptr_dst = dat.cast::<i32>().add((sample * nnew) as usize);
                    let sample_map = if is_local != 0 {
                        laa_map.as_mut_ptr().add((sample * laa_map_stride) as usize)
                    } else {
                        map.as_mut_ptr()
                    };
                    let sample_nori = if is_local != 0 {
                        std::cmp::min(lr_orig[sample as usize] - inc, nori)
                    } else {
                        nori
                    };
                    let mut k_dst = 0;
                    for k_src in 0..sample_nori {
                        if *ptr_src.add(k_src as usize) == bcf_int32_vector_end {
                            break;
                        }
                        if *sample_map.add((k_src + inc) as usize) < 0 {
                            continue;
                        }
                        *ptr_dst.add(k_dst as usize) = *ptr_src.add(k_src as usize);
                        k_dst += 1;
                    }
                    if k_dst == 0 {
                        *ptr_dst.add(k_dst as usize) = bcf_int32_missing;
                        k_dst += 1;
                    }
                    while k_dst < nnew {
                        *ptr_dst.add(k_dst as usize) = bcf_int32_vector_end;
                        k_dst += 1;
                    }
                }
            } else if type_ == BCF_HT_REAL as c_int {
                for sample in 0..n_sample {
                    let ptr_src = dat.cast::<f32>().add((sample * nori) as usize);
                    let ptr_dst = dat.cast::<f32>().add((sample * nnew) as usize);
                    let sample_map = if is_local != 0 {
                        laa_map.as_mut_ptr().add((sample * laa_map_stride) as usize)
                    } else {
                        map.as_mut_ptr()
                    };
                    let sample_nori = if is_local != 0 {
                        std::cmp::min(lr_orig[sample as usize] - inc, nori)
                    } else {
                        nori
                    };
                    let mut k_dst = 0;
                    for k_src in 0..sample_nori {
                        if (*ptr_src.add(k_src as usize)).to_bits() == bcf_float_vector_end {
                            break;
                        }
                        if *sample_map.add((k_src + inc) as usize) < 0 {
                            continue;
                        }
                        *ptr_dst.add(k_dst as usize) = *ptr_src.add(k_src as usize);
                        k_dst += 1;
                    }
                    if k_dst == 0 {
                        *ptr_dst.add(k_dst as usize) = f32::from_bits(bcf_float_missing);
                        k_dst += 1;
                    }
                    while k_dst < nnew {
                        *ptr_dst.add(k_dst as usize) = f32::from_bits(bcf_float_vector_end);
                        k_dst += 1;
                    }
                }
            }
        } else {
            if is_local == 0 && nori != n_g_ori {
                err!();
            }
            if is_local != 0 {
                n_g_new = nori;
            }
            ndat = n_g_new * n_sample;
            if type_ == BCF_HT_INT as c_int {
                for sample in 0..n_sample {
                    let ptr_src = dat.cast::<i32>().add((sample * nori) as usize);
                    let ptr_dst = dat.cast::<i32>().add((sample * n_g_new) as usize);
                    let sample_map = if is_local != 0 {
                        laa_map.as_mut_ptr().add((sample * laa_map_stride) as usize)
                    } else {
                        map.as_mut_ptr()
                    };
                    let sample_n_r_ori = if is_local != 0 {
                        lr_orig[sample as usize]
                    } else {
                        n_r_ori
                    };
                    let sample_n_g_ori = if is_local != 0 {
                        sample_n_r_ori * (sample_n_r_ori + 1) / 2
                    } else {
                        n_g_ori
                    };
                    let mut nset = 0;
                    for k_src in 0..sample_n_g_ori {
                        if *ptr_src.add(k_src as usize) == bcf_int32_vector_end {
                            break;
                        }
                        nset += 1;
                    }
                    let mut k_dst = 0;
                    if nset == sample_n_r_ori {
                        for k_src in 0..sample_n_r_ori {
                            if *sample_map.add(k_src as usize) < 0 {
                                continue;
                            }
                            *ptr_dst.add(k_dst as usize) = *ptr_src.add(k_src as usize);
                            k_dst += 1;
                        }
                    } else {
                        let mut k_src: c_int = -1;
                        'outer_int: for ia in 0..sample_n_r_ori {
                            for ib in 0..=ia {
                                k_src += 1;
                                if *ptr_src.add(k_src as usize) == bcf_int32_vector_end {
                                    *ptr_dst.add(k_dst as usize) = *ptr_src.add(k_src as usize);
                                    break 'outer_int;
                                }
                                if *sample_map.add(ia as usize) < 0
                                    || *sample_map.add(ib as usize) < 0
                                {
                                    continue;
                                }
                                *ptr_dst.add(k_dst as usize) = *ptr_src.add(k_src as usize);
                                k_dst += 1;
                            }
                        }
                    }
                    while k_dst < n_g_new {
                        *ptr_dst.add(k_dst as usize) = bcf_int32_vector_end;
                        k_dst += 1;
                    }
                }
            } else if type_ == BCF_HT_REAL as c_int {
                for sample in 0..n_sample {
                    let ptr_src = dat.cast::<f32>().add((sample * nori) as usize);
                    let ptr_dst = dat.cast::<f32>().add((sample * n_g_new) as usize);
                    let sample_map = if is_local != 0 {
                        laa_map.as_mut_ptr().add((sample * laa_map_stride) as usize)
                    } else {
                        map.as_mut_ptr()
                    };
                    let sample_n_r_ori = if is_local != 0 {
                        lr_orig[sample as usize]
                    } else {
                        n_r_ori
                    };
                    let sample_n_g_ori = if is_local != 0 {
                        sample_n_r_ori * (sample_n_r_ori + 1) / 2
                    } else {
                        n_g_ori
                    };
                    let mut nset = 0;
                    for k_src in 0..sample_n_g_ori {
                        if (*ptr_src.add(k_src as usize)).to_bits() == bcf_float_vector_end {
                            break;
                        }
                        nset += 1;
                    }
                    let mut k_dst = 0;
                    if nset == sample_n_r_ori {
                        for k_src in 0..sample_n_r_ori {
                            if *sample_map.add(k_src as usize) < 0 {
                                continue;
                            }
                            *ptr_dst.add(k_dst as usize) = *ptr_src.add(k_src as usize);
                            k_dst += 1;
                        }
                    } else {
                        let mut k_src: c_int = -1;
                        'outer_real: for ia in 0..sample_n_r_ori {
                            for ib in 0..=ia {
                                k_src += 1;
                                if (*ptr_src.add(k_src as usize)).to_bits() == bcf_float_vector_end
                                {
                                    *ptr_dst.add(k_dst as usize) = *ptr_src.add(k_src as usize);
                                    break 'outer_real;
                                }
                                if *sample_map.add(ia as usize) < 0
                                    || *sample_map.add(ib as usize) < 0
                                {
                                    continue;
                                }
                                *ptr_dst.add(k_dst as usize) = *ptr_src.add(k_src as usize);
                                k_dst += 1;
                            }
                        }
                    }
                    while k_dst < n_g_new {
                        *ptr_dst.add(k_dst as usize) = f32::from_bits(bcf_float_vector_end);
                        k_dst += 1;
                    }
                }
            }
        }
        if vcf::bcf_update_format(header, line, id, dat.cast(), ndat, type_) < 0 {
            err!();
        }
    }

    0
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
        return bcf_int32_missing;
    }

    match (*info).type_ {
        x if x == BCF_BT_INT8 as c_int => {
            let val = le_to_i8((*info).vptr.add(index)) as i32;
            if val > bcf_int8_vector_end {
                val
            } else {
                bcf_int32_vector_end - (bcf_int8_vector_end - val)
            }
        }
        x if x == BCF_BT_INT16 as c_int => {
            let val = le_to_i16((*info).vptr.add(index * size_of::<i16>())) as i32;
            if val > bcf_int16_vector_end {
                val
            } else {
                bcf_int32_vector_end - (bcf_int16_vector_end - val)
            }
        }
        x if x == BCF_BT_INT32 as c_int => le_to_i32((*info).vptr.add(index * size_of::<i32>())),
        x if x == BCF_BT_FLOAT as c_int => {
            let f = le_to_float((*info).vptr.add(index * size_of::<f32>()));
            if f.to_bits() == bcf_float_missing {
                bcf_int32_missing
            } else if f.to_bits() == bcf_float_vector_end {
                bcf_int32_vector_end
            } else {
                f as i32
            }
        }
        _ => bcf_int32_missing,
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
        x if x == BCF_BT_INT8 as c_int => (*info).v1.i = le_to_i8((*info).vptr) as i64,
        x if x == BCF_BT_INT16 as c_int => (*info).v1.i = le_to_i16((*info).vptr) as i64,
        x if x == BCF_BT_INT32 as c_int => (*info).v1.i = le_to_i32((*info).vptr) as i64,
        x if x == BCF_BT_INT64 as c_int => (*info).v1.i = le_to_i64((*info).vptr),
        x if x == BCF_BT_FLOAT as c_int => (*info).v1.f = le_to_float((*info).vptr),
        _ => {}
    }
}

pub unsafe fn vcfutils_c_349_fixup_info_length_code(info: *mut bcf_info_t) -> c_int {
    const BCF_TYPE_SHIFT: [usize; 16] = [0, 0, 1, 2, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut buf = [0u8; 24];
    let mut ptr = buf.as_mut_ptr();

    let mut type_ = if (*info).key <= 0x7f && (*info).key >= -120 {
        BCF_BT_INT8 as c_int
    } else if (*info).key <= 0x7fff && (*info).key >= -32760 {
        BCF_BT_INT16 as c_int
    } else {
        BCF_BT_INT32 as c_int
    };
    *ptr = ((1 << 4) | type_) as u8;
    ptr = ptr.add(1);
    i32_to_le((*info).key, ptr);
    ptr = ptr.add(1 << BCF_TYPE_SHIFT[type_ as usize]);

    type_ = if (*info).len <= 0x7f && (*info).len >= -120 {
        BCF_BT_INT8 as c_int
    } else if (*info).len <= 0x7fff && (*info).len >= -32760 {
        BCF_BT_INT16 as c_int
    } else {
        BCF_BT_INT32 as c_int
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

    let new_len = ptr.offset_from(buf.as_ptr());
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

#[allow(clippy::too_many_arguments)]
pub unsafe fn vcfutils_c_423_trim_int_cnv_tr_int_tags(
    info: *mut bcf_info_t,
    header: *const bcf_hdr_t,
    rm_set: *const crate::htslib_rs::hts::kbitset_t,
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
    let type_ = ((*(*(*header).id[BCF_DT_ID as usize].add(key)).val).info[BCF_HL_INFO as usize]
        >> 4
        & 0xf) as c_int;
    let vlen = ((*(*(*header).id[BCF_DT_ID as usize].add(key)).val).info[BCF_HL_INFO as usize] >> 8
        & 0xf) as c_int;
    let element_sizes = [0usize, 1, 2, 4, 0, 4, 0, 0];
    let element_size = element_sizes[((*info).type_ & 0x7) as usize];
    let mut unit = 0usize;
    let mut orig_pos = 0usize;
    let mut new_pos = 0usize;
    let mut new_total = 0i32;

    if (type_ != BCF_HT_INT as c_int && type_ != BCF_HT_REAL as c_int)
        || element_size == 0
        || vlen != BCF_VL_VAR as c_int
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
    rm_set: *const crate::htslib_rs::hts::kbitset_t,
    rn: *const bcf_info_t,
    num_alt_orig: usize,
    _orig_total: usize,
) -> c_int {
    let key = (*info).key as usize;
    let type_ = ((*(*(*header).id[BCF_DT_ID as usize].add(key)).val).info[BCF_HL_INFO as usize]
        >> 4
        & 0xf) as c_int;
    let vlen = ((*(*(*header).id[BCF_DT_ID as usize].add(key)).val).info[BCF_HL_INFO as usize] >> 8
        & 0xf) as c_int;
    let mut orig_pos = 0usize;
    let mut new_pos = 0usize;

    if type_ != BCF_HT_STR as c_int
        || (*info).type_ != BCF_BT_CHAR as c_int
        || vlen != BCF_VL_VAR as c_int
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
    rm_set: *const crate::htslib_rs::hts::kbitset_t,
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
        let id = (*(*header).id[BCF_DT_ID as usize].add((*info).key as usize)).key;
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
            (*line).d.shared_dirty |= BCF1_DIRTY_INF as c_int;
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

    let n_allele = (*line).n_allele() as usize;
    let mut ac = Vec::new();
    if ac.try_reserve_exact(n_allele).is_err() {
        return -1;
    }
    ac.resize(n_allele, 0);

    for i in 0..(*line).n_sample() as usize {
        let p = (*gt).p.add(i * (*gt).size as usize);
        for ial in 0..(*gt).n as usize {
            let val = match (*gt).type_ {
                x if x == BCF_BT_INT8 as c_int => {
                    let v = le_to_i8(p.add(ial)) as c_int;
                    if v == bcf_int8_vector_end {
                        break;
                    }
                    v
                }
                x if x == BCF_BT_INT16 as c_int => {
                    let v = le_to_i16(p.add(ial * size_of::<i16>())) as c_int;
                    if v == bcf_int16_vector_end {
                        break;
                    }
                    v
                }
                x if x == BCF_BT_INT32 as c_int => {
                    let v = le_to_i32(p.add(ial * size_of::<i32>()));
                    if v == bcf_int32_vector_end {
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
            ac[allele as usize] += 1;
        }
        if ret != 0 {
            break;
        }
    }
    if ret != 0 {
        return ret;
    }

    let rm_set = kbs_init((*line).n_allele() as usize);
    if rm_set.is_null() {
        return -1;
    }
    for i in 1..(*line).n_allele() as c_int {
        if ac[i as usize] == 0 {
            kbs_insert(rm_set, i);
            nrm += 1;
        }
    }

    if nrm != 0 && bcf_remove_allele_set(header, line, rm_set.cast()) != 0 {
        ret = -2;
    }

    kbs_destroy(rm_set);
    if ret != 0 {
        ret
    } else {
        nrm
    }
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

pub unsafe fn vcfutils_c_32_bcf_calc_ac(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    ac: *mut c_int,
    which: c_int,
) -> c_int {
    for i in 0..(*line).n_allele() as usize {
        *ac.add(i) = 0;
    }

    if (which & BCF_UN_INFO as c_int) != 0 {
        bcf_unpack(line, BCF_UN_INFO as c_int);
        let an_id = bcf_hdr_id2int(header, BCF_DT_ID as c_int, c"AN".as_ptr());
        let ac_id = bcf_hdr_id2int(header, BCF_DT_ID as c_int, c"AC".as_ptr());
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
                    x if x == BCF_BT_INT8 as c_int => le_to_i8(ac_ptr.add(i)) as c_int,
                    x if x == BCF_BT_INT16 as c_int => {
                        le_to_i16(ac_ptr.add(i * size_of::<i16>())) as c_int
                    }
                    x if x == BCF_BT_INT32 as c_int => le_to_i32(ac_ptr.add(i * size_of::<i32>())),
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

    if (which & BCF_UN_FMT as c_int) != 0 {
        let gt_id = bcf_hdr_id2int(header, BCF_DT_ID as c_int, c"GT".as_ptr());
        if gt_id < 0 {
            return 0;
        }
        bcf_unpack(line, BCF_UN_FMT as c_int);

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
                    x if x == BCF_BT_INT8 as c_int => {
                        let v = le_to_i8(p.add(ial)) as c_int;
                        if v == bcf_int8_vector_end {
                            break;
                        }
                        v
                    }
                    x if x == BCF_BT_INT16 as c_int => {
                        let v = le_to_i16(p.add(ial * size_of::<i16>())) as c_int;
                        if v == bcf_int16_vector_end {
                            break;
                        }
                        v
                    }
                    x if x == BCF_BT_INT32 as c_int => {
                        let v = le_to_i32(p.add(ial * size_of::<i32>()));
                        if v == bcf_int32_vector_end {
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
            x if x == BCF_BT_INT8 as c_int => {
                let v = le_to_i8(p.add(i)) as c_int;
                if v == bcf_int8_vector_end {
                    break;
                }
                v
            }
            x if x == BCF_BT_INT16 as c_int => {
                let v = le_to_i16(p.add(i * size_of::<i16>())) as c_int;
                if v == bcf_int16_vector_end {
                    break;
                }
                v
            }
            x if x == BCF_BT_INT32 as c_int => {
                let v = le_to_i32(p.add(i * size_of::<i32>()));
                if v == bcf_int32_vector_end {
                    break;
                }
                v
            }
            _ => libc::exit(1),
        };
        if val >> 1 == 0 {
            return GT_UNKN as c_int;
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
        return GT_UNKN as c_int;
    }
    if nals == 1 {
        return if has_ref != 0 { GT_HAPL_R } else { GT_HAPL_A } as c_int;
    }
    if has_ref == 0 {
        return if has_alt == 1 { GT_HOM_AA } else { GT_HET_AA } as c_int;
    }
    if has_alt == 0 {
        return GT_HOM_RR as c_int;
    }
    GT_HET_RA as c_int
}

// original: bcf_acgt2int (htslib/vcfutils.h:24, inline static)
pub fn bcf_acgt2int(mut c: c_char) -> c_int {
    if c > 96 {
        c -= 32;
    }
    match c as u8 {
        b'A' => 0,
        b'C' => 1,
        b'G' => 2,
        b'T' => 3,
        _ => -1,
    }
}

pub unsafe fn bcf_trim_alleles(header: *const bcf_hdr_t, line: *mut bcf1_t) -> c_int {
    vcfutils_c_186_bcf_trim_alleles(header, line)
}

pub unsafe fn bcf_remove_alleles(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    mask: c_int,
) -> c_int {
    vcfutils_c_241_bcf_remove_alleles(header, line, mask)
}

pub unsafe fn bcf_remove_allele_set(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    rm_set: *const super::hts::kbitset_t,
) -> c_int {
    vcfutils_c_659_bcf_remove_allele_set(header, line, rm_set.cast())
}

pub unsafe fn bcf_calc_ac(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    ac: *mut c_int,
    which: c_int,
) -> c_int {
    vcfutils_c_32_bcf_calc_ac(header, line, ac, which)
}

pub unsafe fn bcf_gt_type(
    fmt_ptr: *mut bcf_fmt_t,
    isample: c_int,
    ial: *mut c_int,
    jal: *mut c_int,
) -> c_int {
    vcfutils_c_134_bcf_gt_type(fmt_ptr, isample, ial, jal)
}
