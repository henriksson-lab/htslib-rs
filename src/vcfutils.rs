// Functions translated from htslib/vcfutils.c.

use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::NonNull;

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

const BCF_VL_LA: i32 = 6;
const BCF_VL_LG: i32 = 7;
const BCF_VL_LR: i32 = 8;

// Owns a buffer that the `bcf_get_*_values` FFI functions allocate and
// reallocate on the C heap (via realloc). The buffer is therefore a genuine
// FFI boundary: it is handed to those functions as a raw `*mut ()`, and on
// drop it is released by reconstructing the owning `Vec` from the raw parts so
// that Rust frees it. No C memory-management calls remain in this file.
struct CBuffer<T> {
    ptr: Option<NonNull<T>>,
}

impl<T> CBuffer<T> {
    fn new() -> Self {
        Self { ptr: None }
    }

    fn as_ptr(&self) -> *mut T {
        self.ptr.map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }

    fn cast<U>(&self) -> *mut U {
        self.as_ptr().cast::<U>()
    }

    unsafe fn add(&self, count: usize) -> *mut T {
        self.as_ptr().add(count)
    }

    fn as_opaque_ptr(&self) -> *mut () {
        self.as_ptr().cast()
    }

    fn set_from_opaque(&mut self, ptr: *mut ()) {
        self.ptr = NonNull::new(ptr.cast::<T>());
    }
}

// original: bcf_remove_allele_set (htslib/vcfutils.c:659)
pub unsafe fn vcfutils_c_659_bcf_remove_allele_set(
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
    rm_set: &kbitset_t,
) -> i32 {
    vcfutils_bcf_remove_allele_set_rs(header, line, rm_set)
}

unsafe fn vcfutils_bcf_remove_allele_set_rs(
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
    rm_set: &kbitset_t,
) -> i32 {
    let vl_a_g_r = (1_u32 << BCF_VL_A) | (1_u32 << BCF_VL_G) | (1_u32 << BCF_VL_R);
    let vl_la_lg_lr = (1_u32 << BCF_VL_LA) | (1_u32 << BCF_VL_LG) | (1_u32 << BCF_VL_LR);
    let vl_a_g_r_la_lg_lr = vl_a_g_r | vl_la_lg_lr;
    let vl_a_r = (1_u32 << BCF_VL_A) | (1_u32 << BCF_VL_R);
    let vl_la_lr = (1_u32 << BCF_VL_LA) | (1_u32 << BCF_VL_LR);
    let vl_a_r_la_lr = vl_a_r | vl_la_lr;
    let n_allele = line.n_allele() as usize;
    let mut map = Vec::new();
    if map.try_reserve_exact(n_allele).is_err() {
        return -1;
    }
    map.resize(n_allele, 0);
    let mut laa = CBuffer::<i32>::new();
    let mut laa_map = Vec::new();
    let mut lr_orig = Vec::new();
    let mut dat = CBuffer::<u8>::new();
    let mut laa_size = 0;
    let mut laa_map_stride = 0;
    let mut have_cnv_tr = 0;

    vcf::bcf_unpack(line, BCF_UN_ALL as i32);

    let n_sample = line.n_sample() as i32;
    let mut str_ = kstring_t::default();

    macro_rules! err {
        () => {{
            return -1;
        }};
    }

    kputs(&line.d.allele[0], &mut str_);

    let mut nrm = 0;
    map[0] = 0;
    let mut j = 1;
    for i in 1..line.n_allele() as i32 {
        if line.d.allele[i as usize].as_slice() == b"<CNV:TR>" {
            have_cnv_tr = 1;
        }

        if kbs_exists(rm_set, i) != 0 {
            map[i as usize] = -1;
            nrm += 1;
            continue;
        }
        kputc(b',' as i32, &mut str_);
        kputs(&line.d.allele[i as usize], &mut str_);
        map[i as usize] = j;
        j += 1;
    }
    if nrm == 0 {
        return 0;
    }

    let n_r_ori = line.n_allele() as i32;
    let n_r_new = line.n_allele() as i32 - nrm;
    if n_r_new <= 0 {
        let seqname = vcf::bcf_seqname_safe(header, line);
        let mut seqname_bytes = Vec::new();
        if !seqname.is_null() {
            let mut p = seqname;
            while *p != 0 {
                seqname_bytes.push(*p as u8);
                p = p.add(1);
            }
        }
        let msg = format!(
            "Cannot remove reference allele at {}:{} [{}]",
            String::from_utf8_lossy(&seqname_bytes),
            line.pos + 1,
            n_r_new
        )
        .into_bytes();
        hts_log_cstr(HTS_LOG_ERROR, b"bcf_remove_allele_set", &msg);
        return -1;
    }
    let n_a_ori = n_r_ori - 1;
    let n_a_new = n_r_new - 1;
    let n_g_ori = n_r_ori * (n_r_ori + 1) / 2;
    let mut n_g_new = n_r_new * (n_r_new + 1) / 2;

    let mut str_cstr = str_.data.clone();
    str_cstr.push(0);
    if vcf::bcf_update_alleles_str(header, line, str_cstr.as_ptr().cast::<i8>()) < 0 {
        return -1;
    }

    if have_cnv_tr != 0
        && vcfutils_c_561_fixup_cnv_tr_info_tags(header, line, n_a_ori as usize, rm_set) < 0
    {
        hts_log_cstr(HTS_LOG_ERROR, b"bcf_remove_allele_set", b"Out of memory");
        return -1;
    }

    let mut mdat_bytes = 0;
    for i in 0..line.n_info() {
        let info_key = line.d.info[i as usize].key as usize;
        let id = (*header.id[BCF_DT_ID as usize].add(info_key)).key;
        let mut vlen =
            ((*(*header.id[BCF_DT_ID as usize].add(info_key)).val).info[BCF_HL_INFO as usize] >> 8
                & 0xf) as i32;

        let multiple = if vlen == BCF_VL_VAR as i32 {
            let mut id_len = 0usize;
            while *id.add(id_len) != 0 {
                id_len += 1;
            }
            vcfutils_c_254_is_special_info_type(std::slice::from_raw_parts(id.cast::<u8>(), id_len))
        } else {
            1
        };
        if multiple > 1 {
            vlen = BCF_VL_A as i32;
        }

        if (vl_a_g_r & (1_u32 << vlen)) == 0 {
            continue;
        }
        let type_ =
            ((*(*header.id[BCF_DT_ID as usize].add(info_key)).val).info[BCF_HL_INFO as usize] >> 4
                & 0xf) as i32;
        if type_ == BCF_HT_FLAG as i32 {
            continue;
        }
        let size = if type_ == BCF_HT_REAL as i32 || type_ == BCF_HT_INT as i32 {
            4
        } else {
            1
        };

        let mut mdat = mdat_bytes / size;
        let mut dat_ptr = dat.as_opaque_ptr();
        let mut nret = vcf::bcf_get_info_values(
            header,
            line,
            id,
            (&mut dat_ptr as *mut *mut ()).cast::<*mut c_void>(),
            &mut mdat,
            type_,
        );
        dat.set_from_opaque(dat_ptr);
        mdat_bytes = mdat * size;
        if nret < 0 {
            err!();
        }
        if nret == 0 {
            continue;
        }

        if type_ == BCF_HT_STR as i32 {
            str_.data.clear();
            let mut ss = dat.cast::<u8>();
            let mut se = dat.cast::<u8>();
            let s0 = *ss;
            if vlen == BCF_VL_A as i32 || vlen == BCF_VL_R as i32 {
                let mut inc = 0;
                let nexp = if vlen == BCF_VL_A as i32 {
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
                    while *se != 0 && *se != b',' {
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
                    if !str_.data.is_empty() {
                        kputc(b',' as i32, &mut str_);
                    }
                    let len = se.offset_from(ss) as usize;
                    kputsn(
                        std::slice::from_raw_parts(ss.cast::<u8>(), len),
                        len,
                        &mut str_,
                    );
                    if *se != 0 {
                        se = se.add(1);
                    }
                    ss = se;
                    jj += 1;
                }
                if jj == 1 && s0 == b'.' {
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
                        while *se != 0 && *se != b',' {
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
                        if !str_.data.is_empty() {
                            kputc(b',' as i32, &mut str_);
                        }
                        let len = se.offset_from(ss) as usize;
                        kputsn(
                            std::slice::from_raw_parts(ss.cast::<u8>(), len),
                            len,
                            &mut str_,
                        );
                        if *se != 0 {
                            se = se.add(1);
                        }
                        ss = se;
                    }
                    if *se == 0 {
                        break;
                    }
                }
                if n == 1 && s0 == b'.' {
                    continue;
                }
                if n != n_g_ori {
                    err!();
                }
            }
            // bcf_update_info() takes the string length from strlen() for STR
            // tags (matching htslib's contract: callers pass a NUL-terminated
            // buffer). The owned kstring Vec carries exactly its bytes with no
            // trailing NUL, so terminate a copy before handing it over.
            let mut str_cstr = str_.data.clone();
            str_cstr.push(0);
            nret = vcf::bcf_update_info(
                header,
                line,
                id,
                str_cstr.as_ptr().cast(),
                str_.data.len() as i32,
                type_,
            );
            if nret < 0 {
                err!();
            }
            continue;
        }

        if nret == 1 {
            let info = &line.d.info[i as usize];
            let missing = match info.type_ {
                x if x == BCF_BT_INT8 as i32 => {
                    *info.vptr.cast::<i8>() as i32 == bcf_int8_missing
                }
                x if x == BCF_BT_INT16 as i32 => {
                    // `info.vptr` points into a packed BCF record byte buffer
                    // whose alignment is not guaranteed for multi-byte ints, so
                    // read via `read_unaligned` to avoid misaligned-pointer UB.
                    i16::from_le(core::ptr::read_unaligned(info.vptr.cast::<i16>())) as i32
                        == bcf_int16_missing
                }
                x if x == BCF_BT_INT32 as i32 => {
                    // See above: vptr is a packed-record byte pointer; use
                    // `read_unaligned` to avoid misaligned-pointer UB.
                    i32::from_le(core::ptr::read_unaligned(info.vptr.cast::<i32>()))
                        == bcf_int32_missing
                }
                x if x == BCF_BT_FLOAT as i32 => {
                    // See above: vptr is a packed-record byte pointer; use
                    // `read_unaligned` to avoid misaligned-pointer UB.
                    f32::from_bits(u32::from_le(core::ptr::read_unaligned(
                        info.vptr.cast::<u32>(),
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
        if vlen == BCF_VL_A as i32 || vlen == BCF_VL_R as i32 {
            let (inc, ntop, new_ndat) = if vlen == BCF_VL_A as i32 {
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
            if type_ == BCF_HT_INT as i32 {
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
            } else if type_ == BCF_HT_REAL as i32 {
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
            let mut l_ori: i32 = -1;
            let mut l_new = 0;
            if type_ == BCF_HT_INT as i32 {
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
            } else if type_ == BCF_HT_REAL as i32 {
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
        let mut dat_ptr = dat.as_opaque_ptr();
        let mut nret = vcf::bcf_get_format_values(
            header,
            line,
            c"GT".as_ptr(),
            (&mut dat_ptr as *mut *mut ()).cast::<*mut c_void>(),
            &mut mdat,
            BCF_HT_INT as i32,
        );
        dat.set_from_opaque(dat_ptr);
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
                BCF_HT_INT as i32,
            ) < 0
            {
                err!();
            }
        }
    }

    let mut laa_ptr = laa.as_opaque_ptr();
    let num_laa = vcf::bcf_get_format_values(
        header,
        line,
        c"LAA".as_ptr(),
        (&mut laa_ptr as *mut *mut ()).cast::<*mut c_void>(),
        &mut laa_size,
        BCF_HT_INT as i32,
    );
    laa.set_from_opaque(laa_ptr);
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
                        std::ptr::copy(
                            laa.add((sample * num_laa_vals) as usize),
                            laa.add((sample * max_k) as usize),
                            max_k as usize,
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
                BCF_HT_INT as i32,
            ) < 0
            {
                err!();
            }
        }
    }

    for fmt_i in 0..line.n_fmt() {
        let fmt_id = line.d.fmt[fmt_i as usize].id as usize;
        let id = (*header.id[BCF_DT_ID as usize].add(fmt_id)).key;
        let vlen = ((*(*header.id[BCF_DT_ID as usize].add(fmt_id)).val).info[BCF_HL_FMT as usize]
            >> 8
            & 0xf) as i32;

        if (vl_a_g_r_la_lg_lr & (1_u32 << vlen)) == 0 {
            continue;
        }
        let is_local = ((vl_la_lg_lr & (1_u32 << vlen)) != 0) as i32;
        let type_ = ((*(*header.id[BCF_DT_ID as usize].add(fmt_id)).val).info[BCF_HL_FMT as usize]
            >> 4
            & 0xf) as i32;
        if type_ == BCF_HT_FLAG as i32 {
            continue;
        }
        let size = if type_ == BCF_HT_REAL as i32 || type_ == BCF_HT_INT as i32 {
            4
        } else {
            1
        };

        let mut mdat = mdat_bytes / size;
        let mut dat_ptr = dat.as_opaque_ptr();
        let mut nret = vcf::bcf_get_format_values(
            header,
            line,
            id,
            (&mut dat_ptr as *mut *mut ()).cast::<*mut c_void>(),
            &mut mdat,
            type_,
        );
        dat.set_from_opaque(dat_ptr);
        mdat_bytes = mdat * size;
        if nret < 0 {
            err!();
        }
        if nret == 0 {
            continue;
        }

        if type_ == BCF_HT_STR as i32 {
            let width = nret / n_sample;
            str_.data.clear();
            if (vl_a_r_la_lr & (1_u32 << vlen)) != 0 {
                let mut nexp = 0;
                let mut inc = 0;
                match vlen {
                    x if x == BCF_VL_A as i32 => {
                        nexp = n_a_ori;
                        inc = 1;
                    }
                    x if x == BCF_VL_R as i32 => nexp = n_r_ori,
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
                    let mut ss = dat.add((sample * width) as usize).cast::<u8>();
                    let se = ss.add(width as usize);
                    let mut ptr = ss;
                    let s0 = *ss;
                    let mut k_dst = 0;
                    let l0 = str_.data.len();
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
                        while ptr < se && *ptr != 0 && *ptr != b',' {
                            ptr = ptr.add(1);
                        }
                        if *sample_map.add((k_src + inc) as usize) < 0 {
                            ptr = ptr.add(1);
                            ss = ptr;
                            k_src += 1;
                            continue;
                        }
                        if k_dst != 0 {
                            kputc(b',' as i32, &mut str_);
                        }
                        let len = ptr.offset_from(ss) as usize;
                        kputsn(
                            std::slice::from_raw_parts(ss.cast::<u8>(), len),
                            len,
                            &mut str_,
                        );
                        ptr = ptr.add(1);
                        ss = ptr;
                        k_dst += 1;
                        k_src += 1;
                    }
                    if k_src != nexp && !(k_src == 1 && s0 == b'.') {
                        err!();
                    }
                    let mut l = str_.data.len() - l0;
                    while l < width as usize {
                        kputc(if l == 0 { b'.' } else { 0 } as i32, &mut str_);
                        l += 1;
                    }
                }
            } else {
                for sample in 0..n_sample {
                    let mut ss = dat.add((sample * width) as usize).cast::<u8>();
                    let se = ss.add(width as usize);
                    let mut ptr = ss;
                    let s0 = *ss;
                    let mut k_dst = 0;
                    let l0 = str_.data.len();
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
                        if *ptr == b',' {
                            nexp += 1;
                        }
                        ptr = ptr.add(1);
                    }
                    if ptr != ss {
                        nexp += 1;
                    }
                    if nexp != sample_n_g_ori
                        && nexp != sample_n_r_ori
                        && !(nexp == 1 && s0 == b'.')
                    {
                        err!();
                    }
                    ptr = ss;
                    if nexp == 1 && s0 == b'.' {
                        kputc(b'.' as i32, &mut str_);
                    } else if nexp == sample_n_g_ori {
                        for ia in 0..sample_n_r_ori {
                            for ib in 0..=ia {
                                if ptr >= se || *ptr == 0 {
                                    break;
                                }
                                while ptr < se && *ptr != 0 && *ptr != b',' {
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
                                    kputc(b',' as i32, &mut str_);
                                }
                                let len = ptr.offset_from(ss) as usize;
                                kputsn(
                                    std::slice::from_raw_parts(ss.cast::<u8>(), len),
                                    len,
                                    &mut str_,
                                );
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
                            while ptr < se && *ptr != 0 && *ptr != b',' {
                                ptr = ptr.add(1);
                            }
                            if *sample_map.add(k_src as usize) < 0 {
                                ptr = ptr.add(1);
                                ss = ptr;
                                k_src += 1;
                                continue;
                            }
                            if k_dst != 0 {
                                kputc(b',' as i32, &mut str_);
                            }
                            let len = ptr.offset_from(ss) as usize;
                            kputsn(
                                std::slice::from_raw_parts(ss.cast::<u8>(), len),
                                len,
                                &mut str_,
                            );
                            ptr = ptr.add(1);
                            ss = ptr;
                            k_dst += 1;
                            k_src += 1;
                        }
                        if k_src != n_r_ori {
                            err!();
                        }
                    }
                    let mut l = str_.data.len() - l0;
                    while l < width as usize {
                        kputc(0, &mut str_);
                        l += 1;
                    }
                }
            }
            nret = vcf::bcf_update_format(
                header,
                line,
                id,
                str_.data.as_ptr().cast(),
                str_.data.len() as i32,
                type_,
            );
            if nret < 0 {
                err!();
            }
            continue;
        }

        let nori = nret / n_sample;
        if nori == 1 && !(vlen == BCF_VL_A as i32 && nori == n_a_ori) {
            let mut all_missing = 1;
            let fmt = &line.d.fmt[fmt_i as usize];
            match fmt.type_ {
                x if x == BCF_BT_INT8 as i32 => {
                    for sample in 0..n_sample {
                        let val = *fmt.p.add((sample * fmt.size) as usize).cast::<i8>();
                        if val as i32 != bcf_int8_missing {
                            all_missing = 0;
                            break;
                        }
                    }
                }
                x if x == BCF_BT_INT16 as i32 => {
                    for sample in 0..n_sample {
                        // `fmt.p` indexes into a packed BCF record byte
                        // buffer whose alignment is not guaranteed for
                        // multi-byte ints; use `read_unaligned` to avoid UB.
                        let val = i16::from_le(core::ptr::read_unaligned(
                            fmt.p.add((sample * fmt.size) as usize).cast::<i16>(),
                        ));
                        if val as i32 != bcf_int16_missing {
                            all_missing = 0;
                            break;
                        }
                    }
                }
                x if x == BCF_BT_INT32 as i32 => {
                    for sample in 0..n_sample {
                        // See above: packed-record byte pointer, use `read_unaligned`.
                        let val = i32::from_le(core::ptr::read_unaligned(
                            fmt.p.add((sample * fmt.size) as usize).cast::<i32>(),
                        ));
                        if val != bcf_int32_missing {
                            all_missing = 0;
                            break;
                        }
                    }
                }
                x if x == BCF_BT_FLOAT as i32 => {
                    for sample in 0..n_sample {
                        // See above: packed-record byte pointer, use `read_unaligned`.
                        let val = u32::from_le(core::ptr::read_unaligned(
                            fmt.p.add((sample * fmt.size) as usize).cast::<u32>(),
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
        if (vl_a_r_la_lr & (1_u32 << vlen)) != 0 || (vlen == BCF_VL_G as i32 && nori == n_r_ori) {
            let mut inc = 0;
            let nnew;
            match vlen {
                x if x == BCF_VL_A as i32 => {
                    if nori != n_a_ori {
                        err!();
                    }
                    ndat = n_a_new * n_sample;
                    nnew = n_a_new;
                    inc = 1;
                }
                x if x == BCF_VL_R as i32 => {
                    if nori != n_r_ori {
                        err!();
                    }
                    ndat = n_r_new * n_sample;
                    nnew = n_r_new;
                }
                x if x == BCF_VL_G as i32 => {
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

            if type_ == BCF_HT_INT as i32 {
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
            } else if type_ == BCF_HT_REAL as i32 {
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
            if type_ == BCF_HT_INT as i32 {
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
                        let mut k_src: i32 = -1;
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
            } else if type_ == BCF_HT_REAL as i32 {
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
                        let mut k_src: i32 = -1;
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
pub unsafe fn vcfutils_c_254_is_special_info_type(name: &[u8]) -> i32 {
    let bytes = name;
    match bytes.first().copied() {
        Some(b'C') => {
            if bytes
                .get(1..)
                .is_some_and(|suffix| matches!(suffix, b"ICN" | b"IEND" | b"ILEN" | b"IPOS"))
            {
                return 2;
            }
        }
        Some(b'M') => {
            if matches!(bytes, b"MEINFO" | b"METRANS") {
                return 4;
            }
        }
        _ => {}
    }
    1
}

pub unsafe fn vcfutils_c_280_get_int32_info_value(info: &bcf_info_t, index: usize) -> i32 {
    let len = if info.len > 0 { info.len as usize } else { 0 };
    if index >= len {
        return bcf_int32_missing;
    }

    match info.type_ {
        x if x == BCF_BT_INT8 as i32 => {
            let val = le_to_i8(info.vptr.add(index)) as i32;
            if val > bcf_int8_vector_end {
                val
            } else {
                bcf_int32_vector_end - (bcf_int8_vector_end - val)
            }
        }
        x if x == BCF_BT_INT16 as i32 => {
            let val = le_to_i16(info.vptr.add(index * size_of::<i16>())) as i32;
            if val > bcf_int16_vector_end {
                val
            } else {
                bcf_int32_vector_end - (bcf_int16_vector_end - val)
            }
        }
        x if x == BCF_BT_INT32 as i32 => le_to_i32(info.vptr.add(index * size_of::<i32>())),
        x if x == BCF_BT_FLOAT as i32 => {
            let f = le_to_float(info.vptr.add(index * size_of::<f32>()));
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

pub unsafe fn vcfutils_c_315_get_rn_value(rn: Option<&bcf_info_t>, index: usize) -> i32 {
    let val = rn.map_or(1, |rn| vcfutils_c_280_get_int32_info_value(rn, index));
    if val >= 0 {
        val
    } else {
        0
    }
}

pub unsafe fn vcfutils_c_325_set_info_v1(info: &mut bcf_info_t) {
    match info.type_ {
        x if x == BCF_BT_INT8 as i32 => info.v1.i = le_to_i8(info.vptr) as i64,
        x if x == BCF_BT_INT16 as i32 => info.v1.i = le_to_i16(info.vptr) as i64,
        x if x == BCF_BT_INT32 as i32 => info.v1.i = le_to_i32(info.vptr) as i64,
        x if x == BCF_BT_INT64 as i32 => info.v1.i = le_to_i64(info.vptr),
        x if x == BCF_BT_FLOAT as i32 => info.v1.f = le_to_float(info.vptr),
        _ => {}
    }
}

pub unsafe fn vcfutils_c_349_fixup_info_length_code(info: &mut bcf_info_t) -> i32 {
    const BCF_TYPE_SHIFT: [usize; 16] = [0, 0, 1, 2, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut buf = [0u8; 24];
    let mut ptr = buf.as_mut_ptr();

    let mut type_ = if info.key <= 0x7f && info.key >= -120 {
        BCF_BT_INT8 as i32
    } else if info.key <= 0x7fff && info.key >= -32760 {
        BCF_BT_INT16 as i32
    } else {
        BCF_BT_INT32 as i32
    };
    *ptr = ((1 << 4) | type_) as u8;
    ptr = ptr.add(1);
    i32_to_le(info.key, ptr);
    ptr = ptr.add(1 << BCF_TYPE_SHIFT[type_ as usize]);

    type_ = if info.len <= 0x7f && info.len >= -120 {
        BCF_BT_INT8 as i32
    } else if info.len <= 0x7fff && info.len >= -32760 {
        BCF_BT_INT16 as i32
    } else {
        BCF_BT_INT32 as i32
    };
    if info.len < 15 {
        *ptr = ((info.len << 4) | info.type_) as u8;
        ptr = ptr.add(1);
    } else {
        *ptr = (0xf0 | info.type_) as u8;
        ptr = ptr.add(1);
        *ptr = ((1 << 4) | type_) as u8;
        ptr = ptr.add(1);
        i32_to_le(info.len, ptr);
        ptr = ptr.add(1 << BCF_TYPE_SHIFT[type_ as usize]);
    }

    let new_len = ptr.offset_from(buf.as_ptr());
    let old_len = info.vptr_off() as isize;
    if new_len == old_len {
        std::slice::from_raw_parts_mut(info.vptr.offset(-old_len), new_len as usize)
            .copy_from_slice(&buf[..new_len as usize]);
    } else if new_len < old_len {
        let adjust = old_len - new_len;
        std::slice::from_raw_parts_mut(info.vptr.offset(-old_len), new_len as usize)
            .copy_from_slice(&buf[..new_len as usize]);
        std::ptr::copy(
            info.vptr,
            info.vptr.offset(-adjust),
            info.vptr_len as usize,
        );
        info.vptr = info.vptr.offset(-adjust);
        info.set_vptr_off((info.vptr_off() as isize - adjust) as u32);
    } else {
        // The new record buffer is owned as a Rust `Vec<u8>`; ownership is
        // transferred to the BCF record by leaking the allocation (so the raw
        // pointer survives) and flagging `vptr_free`, which marks the record
        // as the owner that must later release it.
        let total = info.vptr_len as usize + new_len as usize;
        let mut new_buf = vec![0u8; total];
        new_buf[..new_len as usize].copy_from_slice(&buf[..new_len as usize]);
        let new_info = new_buf.as_mut_ptr();
        std::ptr::copy_nonoverlapping(
            info.vptr,
            new_info.add(new_len as usize),
            info.vptr_len as usize,
        );
        std::mem::forget(new_buf);
        if info.vptr_free() != 0 {
            // Reclaim the previously-leaked owning buffer by reconstructing
            // and dropping its `Vec`.
            let base = info.vptr.sub(info.vptr_off() as usize);
            let len = info.vptr_off() as usize + info.vptr_len as usize;
            drop(Vec::from_raw_parts(base, len, len));
        }
        info.set_vptr_off(new_len as u32);
        info.vptr = new_info.add(new_len as usize);
        info.set_vptr_free(1);
    }
    0
}

pub unsafe fn vcfutils_c_407_mark_for_removal(info: &mut bcf_info_t) -> i32 {
    if info.vptr_free() != 0 {
        let base = info.vptr.sub(info.vptr_off() as usize);
        let len = info.vptr_off() as usize + info.vptr_len as usize;
        drop(Vec::from_raw_parts(base, len, len));
        info.set_vptr_free(0);
    }
    info.vptr = std::ptr::null_mut();
    info.set_vptr_off(0);
    info.vptr_len = 0;
    0
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn vcfutils_c_423_trim_int_cnv_tr_int_tags(
    info: &mut bcf_info_t,
    header: &bcf_hdr_t,
    rm_set: &kbitset_t,
    id: &[u8],
    rn: Option<&bcf_info_t>,
    ruc: Option<&bcf_info_t>,
    num_alt_orig: usize,
    orig_total: usize,
) -> i32 {
    let count = if id.first().copied() == Some(b'C') {
        2usize
    } else {
        1usize
    };
    let key = info.key as usize;
    let type_ = ((*(*(header.id[BCF_DT_ID as usize].add(key))).val).info[BCF_HL_INFO as usize] >> 4
        & 0xf) as i32;
    let vlen = ((*(*(header.id[BCF_DT_ID as usize].add(key))).val).info[BCF_HL_INFO as usize] >> 8
        & 0xf) as i32;
    let element_sizes = [0usize, 1, 2, 4, 0, 4, 0, 0];
    let element_size = element_sizes[(info.type_ & 0x7) as usize];
    let mut unit = 0usize;
    let mut orig_pos = 0usize;
    let mut new_pos = 0usize;
    let mut new_total = 0i32;

    if (type_ != BCF_HT_INT as i32 && type_ != BCF_HT_REAL as i32)
        || element_size == 0
        || vlen != BCF_VL_VAR as i32
        || info.len != orig_total as i32
        || (info.vptr_len as usize)
            < orig_total
                .saturating_mul(element_size)
                .saturating_mul(count)
    {
        return 1;
    }

    for allele in 0..num_alt_orig {
        let mut n_repeats = vcfutils_c_315_get_rn_value(rn, allele);
        let mut n_items = n_repeats;
        if let Some(ruc) = ruc {
            n_items = 0;
            while n_repeats > 0 {
                let n_units = vcfutils_c_280_get_int32_info_value(ruc, unit);
                n_items += if n_units >= 0 { n_units } else { 0 };
                unit += 1;
                n_repeats -= 1;
            }
        }

        let byte_len = (n_items as usize) * element_size * count;
        if kbs_exists(rm_set, (allele + 1) as i32) != 0 {
            orig_pos += byte_len;
            continue;
        }
        if new_pos < orig_pos {
            std::ptr::copy(info.vptr.add(orig_pos), info.vptr.add(new_pos), byte_len);
        }
        orig_pos += byte_len;
        new_pos += byte_len;
        new_total += n_items;
    }

    if new_total == 0 {
        return vcfutils_c_407_mark_for_removal(info);
    }

    info.vptr_len = new_pos as u32;
    info.len = new_total;
    if info.len == 1 {
        vcfutils_c_325_set_info_v1(info);
    }
    vcfutils_c_349_fixup_info_length_code(info)
}

pub unsafe fn vcfutils_c_498_trim_int_cnv_tr_str_tags(
    info: &mut bcf_info_t,
    header: &bcf_hdr_t,
    rm_set: &kbitset_t,
    rn: Option<&bcf_info_t>,
    num_alt_orig: usize,
    _orig_total: usize,
) -> i32 {
    let key = info.key as usize;
    let type_ = ((*(*(header.id[BCF_DT_ID as usize].add(key))).val).info[BCF_HL_INFO as usize] >> 4
        & 0xf) as i32;
    let vlen = ((*(*(header.id[BCF_DT_ID as usize].add(key))).val).info[BCF_HL_INFO as usize] >> 8
        & 0xf) as i32;
    let mut orig_pos = 0usize;
    let mut new_pos = 0usize;

    if type_ != BCF_HT_STR as i32
        || info.type_ != BCF_BT_CHAR as i32
        || vlen != BCF_VL_VAR as i32
    {
        return 1;
    }

    for allele in 0..num_alt_orig {
        let mut n_items = vcfutils_c_315_get_rn_value(rn, allele);
        let start = info.vptr.add(orig_pos);
        let mut end = start;
        let lim = info.vptr.add(info.vptr_len as usize);

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
        if kbs_exists(rm_set, (allele + 1) as i32) != 0 {
            orig_pos += span;
            continue;
        }
        if new_pos < orig_pos {
            std::ptr::copy(info.vptr.add(orig_pos), info.vptr.add(new_pos), span);
        }
        orig_pos += span;
        new_pos += span;
    }

    if new_pos == 0 {
        return vcfutils_c_407_mark_for_removal(info);
    }
    if new_pos < orig_pos {
        *info.vptr.add(new_pos) = 0;
        if new_pos > 0 && *info.vptr.add(new_pos - 1) == b',' {
            new_pos -= 1;
            *info.vptr.add(new_pos) = 0;
        }
        info.len = new_pos as i32;
        info.vptr_len = new_pos as u32;
        return vcfutils_c_349_fixup_info_length_code(info);
    }
    0
}

pub unsafe fn vcfutils_c_561_fixup_cnv_tr_info_tags(
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
    num_alt_orig: usize,
    rm_set: &kbitset_t,
) -> i32 {
    let rn_ptr = NonNull::new(bcf_get_info(header, line, c"RN".as_ptr()));
    let ruc_ptr = NonNull::new(bcf_get_info(header, line, c"RUC".as_ptr()));
    let rn = rn_ptr.map(|ptr| ptr.as_ref());
    let ruc = ruc_ptr.as_ref().map(|ptr| ptr.as_ref());
    let mut orig_total_repeats = 0i64;
    let mut orig_total_units = 0i64;
    let mut unit = 0usize;

    for allele in 0..num_alt_orig {
        let mut n_repeats = vcfutils_c_315_get_rn_value(rn, allele);
        orig_total_repeats += n_repeats as i64;
        if let Some(ruc) = ruc {
            while n_repeats > 0 {
                let n_units = vcfutils_c_280_get_int32_info_value(ruc, unit);
                orig_total_units += if n_units >= 0 { n_units as i64 } else { 0 };
                unit += 1;
                n_repeats -= 1;
            }
        }
    }

    for i in 0..line.n_info() {
        let info = &mut line.d.info[i as usize];
        let id = (*header.id[BCF_DT_ID as usize].add(info.key as usize)).key;
        let mut id_len = 0usize;
        while *id.add(id_len) != 0 {
            id_len += 1;
        }
        let id_str = std::slice::from_raw_parts(id.cast::<u8>(), id_len);
        let orig_ptr = info.vptr.sub(info.vptr_off() as usize);
        if id_str.first().copied() != Some(b'C') && id_str.first().copied() != Some(b'R') {
            continue;
        }

        if matches!(id_str, b"RB" | b"RUL" | b"CIRB" | b"CIRUC") {
            let res = vcfutils_c_423_trim_int_cnv_tr_int_tags(
                info,
                header,
                rm_set,
                id_str,
                rn,
                None,
                num_alt_orig,
                orig_total_repeats as usize,
            );
            if res < 0 {
                return res;
            }
        } else if id_str == b"RUS" {
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
        } else if ruc.is_some() && id_str == b"RUB" {
            let res = vcfutils_c_423_trim_int_cnv_tr_int_tags(
                info,
                header,
                rm_set,
                id_str,
                rn,
                ruc,
                num_alt_orig,
                orig_total_units as usize,
            );
            if res < 0 {
                return res;
            }
        }

        if info.vptr.is_null() || info.vptr.sub(info.vptr_off() as usize) != orig_ptr {
            line.d.shared_dirty |= BCF1_DIRTY_INF as i32;
        }
    }

    if let Some(mut ruc) = ruc_ptr {
        let res = vcfutils_c_423_trim_int_cnv_tr_int_tags(
            ruc.as_mut(),
            header,
            rm_set,
            b"RUC",
            rn,
            None,
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
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
) -> i32 {
    vcfutils_bcf_trim_alleles_rs(header, line)
}

unsafe fn vcfutils_bcf_trim_alleles_rs(header: &bcf_hdr_t, line: &mut bcf1_t) -> i32 {
    let mut ret = 0;
    let mut nrm = 0;
    let gt_ptr = bcf_get_fmt(header, line, c"GT".as_ptr());
    if gt_ptr.is_null() {
        return 0;
    }
    let gt = &*gt_ptr;

    let n_allele = line.n_allele() as usize;
    let mut ac = Vec::new();
    if ac.try_reserve_exact(n_allele).is_err() {
        return -1;
    }
    ac.resize(n_allele, 0);

    for i in 0..line.n_sample() as usize {
        let p = gt.p.add(i * gt.size as usize);
        for ial in 0..gt.n as usize {
            let val = match gt.type_ {
                x if x == BCF_BT_INT8 as i32 => {
                    let v = le_to_i8(p.add(ial)) as i32;
                    if v == bcf_int8_vector_end {
                        break;
                    }
                    v
                }
                x if x == BCF_BT_INT16 as i32 => {
                    let v = le_to_i16(p.add(ial * size_of::<i16>())) as i32;
                    if v == bcf_int16_vector_end {
                        break;
                    }
                    v
                }
                x if x == BCF_BT_INT32 as i32 => {
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
            if allele >= line.n_allele() as i32 {
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

    let rm_set = kbs_init(line.n_allele() as usize);
    if rm_set.is_null() {
        return -1;
    }
    for i in 1..line.n_allele() as i32 {
        if ac[i as usize] == 0 {
            kbs_insert(&mut *rm_set, i);
            nrm += 1;
        }
    }

    if nrm != 0 && vcfutils_bcf_remove_allele_set_rs(header, line, &*rm_set) != 0 {
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
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
    mask: i32,
) -> i32 {
    vcfutils_bcf_remove_alleles_rs(header, line, mask)
}

unsafe fn vcfutils_bcf_remove_alleles_rs(
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
    mask: i32,
) -> i32 {
    let rm_set = kbs_init(line.n_allele() as usize);
    if rm_set.is_null() {
        return -1;
    }
    for i in 1..line.n_allele() as i32 {
        if (mask & (1 << i)) != 0 {
            kbs_insert(&mut *rm_set, i);
        }
    }
    vcfutils_bcf_remove_allele_set_rs(header, line, &*rm_set);
    kbs_destroy(rm_set);
    0
}

pub unsafe fn vcfutils_c_32_bcf_calc_ac(
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
    ac: &mut [i32],
    which: i32,
) -> i32 {
    vcfutils_bcf_calc_ac_rs(header, line, ac, which)
}

unsafe fn vcfutils_bcf_calc_ac_rs(
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
    ac: &mut [i32],
    which: i32,
) -> i32 {
    for val in ac.iter_mut().take(line.n_allele() as usize) {
        *val = 0;
    }

    if (which & BCF_UN_INFO as i32) != 0 {
        bcf_unpack(line, BCF_UN_INFO as i32);
        let an_id = bcf_hdr_id2int(header, BCF_DT_ID as i32, c"AN".as_ptr());
        let ac_id = bcf_hdr_id2int(header, BCF_DT_ID as i32, c"AC".as_ptr());
        let mut an = -1;
        let mut ac_len = 0;
        let mut ac_type = 0;
        let mut ac_ptr: Option<*mut u8> = None;

        if an_id >= 0 && ac_id >= 0 {
            for i in 0..line.n_info() as usize {
                let z = &line.d.info[i];
                if z.key == an_id {
                    an = z.v1.i as i32;
                } else if z.key == ac_id {
                    ac_ptr = Some(z.vptr);
                    ac_len = z.len;
                    ac_type = z.type_;
                }
            }
        }

        if let (true, Some(ac_ptr)) = (an >= 0, ac_ptr) {
            if ac_len != line.n_allele() as i32 - 1 {
                return 0;
            }
            let mut nac = 0;
            for i in 0..ac_len as usize {
                let val = match ac_type {
                    x if x == BCF_BT_INT8 as i32 => le_to_i8(ac_ptr.add(i)) as i32,
                    x if x == BCF_BT_INT16 as i32 => {
                        le_to_i16(ac_ptr.add(i * size_of::<i16>())) as i32
                    }
                    x if x == BCF_BT_INT32 as i32 => le_to_i32(ac_ptr.add(i * size_of::<i32>())),
                    _ => std::process::exit(1),
                };
                ac[i + 1] = val;
                nac += val;
            }
            if an < nac {
                std::process::exit(1);
            }
            ac[0] = an - nac;
            return 1;
        }
    }

    if (which & BCF_UN_FMT as i32) != 0 {
        let gt_id = bcf_hdr_id2int(header, BCF_DT_ID as i32, c"GT".as_ptr());
        if gt_id < 0 {
            return 0;
        }
        bcf_unpack(line, BCF_UN_FMT as i32);

        let mut fmt_gt: Option<&bcf_fmt_t> = None;
        for i in 0..line.n_fmt() as usize {
            let fmt = &line.d.fmt[i];
            if fmt.id == gt_id {
                fmt_gt = Some(fmt);
                break;
            }
        }
        let Some(fmt_gt) = fmt_gt else {
            return 0;
        };

        for i in 0..line.n_sample() as usize {
            let p = fmt_gt.p.add(i * fmt_gt.size as usize);
            for ial in 0..fmt_gt.n as usize {
                let val = match fmt_gt.type_ {
                    x if x == BCF_BT_INT8 as i32 => {
                        let v = le_to_i8(p.add(ial)) as i32;
                        if v == bcf_int8_vector_end {
                            break;
                        }
                        v
                    }
                    x if x == BCF_BT_INT16 as i32 => {
                        let v = le_to_i16(p.add(ial * size_of::<i16>())) as i32;
                        if v == bcf_int16_vector_end {
                            break;
                        }
                        v
                    }
                    x if x == BCF_BT_INT32 as i32 => {
                        let v = le_to_i32(p.add(ial * size_of::<i32>()));
                        if v == bcf_int32_vector_end {
                            break;
                        }
                        v
                    }
                    _ => std::process::exit(1),
                };
                if val >> 1 == 0 {
                    continue;
                }
                if val >> 1 > line.n_allele() as i32 {
                    std::process::exit(1);
                }
                ac[((val >> 1) - 1) as usize] += 1;
            }
        }
        return 1;
    }

    0
}

pub unsafe fn vcfutils_c_134_bcf_gt_type(
    fmt: &bcf_fmt_t,
    isample: i32,
    ial_out: Option<&mut i32>,
    jal_out: Option<&mut i32>,
) -> i32 {
    let mut ial = 0;
    let mut jal = 0;
    let gt_type = vcfutils_bcf_gt_type_rs(fmt, isample, &mut ial, &mut jal);
    if let Some(ial_out) = ial_out {
        *ial_out = ial;
    }
    if let Some(jal_out) = jal_out {
        *jal_out = jal;
    }
    gt_type
}

unsafe fn vcfutils_bcf_gt_type_rs(
    fmt: &bcf_fmt_t,
    isample: i32,
    ial_out: &mut i32,
    jal_out: &mut i32,
) -> i32 {
    let mut nals = 0;
    let mut has_ref = 0;
    let mut has_alt = 0;
    let mut ial = 0;
    let mut jal = 0;
    let p = fmt.p.add(isample as usize * fmt.size as usize);

    for i in 0..fmt.n as usize {
        let val = match fmt.type_ {
            x if x == BCF_BT_INT8 as i32 => {
                let v = le_to_i8(p.add(i)) as i32;
                if v == bcf_int8_vector_end {
                    break;
                }
                v
            }
            x if x == BCF_BT_INT16 as i32 => {
                let v = le_to_i16(p.add(i * size_of::<i16>())) as i32;
                if v == bcf_int16_vector_end {
                    break;
                }
                v
            }
            x if x == BCF_BT_INT32 as i32 => {
                let v = le_to_i32(p.add(i * size_of::<i32>()));
                if v == bcf_int32_vector_end {
                    break;
                }
                v
            }
            _ => std::process::exit(1),
        };
        if val >> 1 == 0 {
            return GT_UNKN as i32;
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

    *ial_out = if ial > 0 { ial - 1 } else { ial };
    *jal_out = if jal > 0 { jal - 1 } else { jal };
    if nals == 0 {
        return GT_UNKN as i32;
    }
    if nals == 1 {
        return if has_ref != 0 { GT_HAPL_R } else { GT_HAPL_A } as i32;
    }
    if has_ref == 0 {
        return if has_alt == 1 { GT_HOM_AA } else { GT_HET_AA } as i32;
    }
    if has_alt == 0 {
        return GT_HOM_RR as i32;
    }
    GT_HET_RA as i32
}

// original: bcf_acgt2int (htslib/vcfutils.h:24, inline static)
pub fn bcf_acgt2int(mut c: u8) -> i32 {
    if c > 96 {
        c -= 32;
    }
    match c {
        b'A' => 0,
        b'C' => 1,
        b'G' => 2,
        b'T' => 3,
        _ => -1,
    }
}

pub unsafe fn bcf_trim_alleles(header: &bcf_hdr_t, line: &mut bcf1_t) -> i32 {
    vcfutils_c_186_bcf_trim_alleles(header, line)
}

pub unsafe fn bcf_remove_alleles(
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
    mask: i32,
) -> i32 {
    vcfutils_c_241_bcf_remove_alleles(header, line, mask)
}

pub unsafe fn bcf_remove_allele_set(
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
    rm_set: &super::hts::kbitset_t,
) -> i32 {
    vcfutils_c_659_bcf_remove_allele_set(header, line, rm_set)
}

pub unsafe fn bcf_calc_ac(
    header: &bcf_hdr_t,
    line: &mut bcf1_t,
    ac: &mut [i32],
    which: i32,
) -> i32 {
    vcfutils_c_32_bcf_calc_ac(header, line, ac, which)
}

pub unsafe fn bcf_gt_type(
    fmt: &bcf_fmt_t,
    isample: i32,
    ial: Option<&mut i32>,
    jal: Option<&mut i32>,
) -> i32 {
    vcfutils_c_134_bcf_gt_type(fmt, isample, ial, jal)
}
