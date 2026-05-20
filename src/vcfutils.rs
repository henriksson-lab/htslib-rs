use std::ffi::{c_char, c_int, c_void};
use std::mem::size_of;

use crate::htslib_rs::hts::{kbitset_t, kbs_exists, kputc, kputs, kputsn, kstring_t};
use crate::htslib_rs::vcf::{self, bcf1_t, bcf_hdr_t};

const BCF_VL_LA: c_int = 6;
const BCF_VL_LG: c_int = 7;
const BCF_VL_LR: c_int = 8;

// original: bcf_remove_allele_set (htslib/vcfutils.c:659)
pub unsafe fn vcfutils_c_659_bcf_remove_allele_set(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    rm_set: *const kbitset_t,
) -> c_int {
    let vl_a_g_r =
        (1_u32 << hts_sys::BCF_VL_A) | (1_u32 << hts_sys::BCF_VL_G) | (1_u32 << hts_sys::BCF_VL_R);
    let vl_la_lg_lr = (1_u32 << BCF_VL_LA) | (1_u32 << BCF_VL_LG) | (1_u32 << BCF_VL_LR);
    let vl_a_g_r_la_lg_lr = vl_a_g_r | vl_la_lg_lr;
    let vl_a_r = (1_u32 << hts_sys::BCF_VL_A) | (1_u32 << hts_sys::BCF_VL_R);
    let vl_la_lr = (1_u32 << BCF_VL_LA) | (1_u32 << BCF_VL_LR);
    let vl_a_r_la_lr = vl_a_r | vl_la_lr;
    let map = libc::malloc((*line).n_allele() as usize * size_of::<c_int>()).cast::<c_int>();
    let mut laa: *mut c_int = std::ptr::null_mut();
    let mut laa_map: *mut c_int = std::ptr::null_mut();
    let mut lr_orig: *mut c_int = std::ptr::null_mut();
    let mut dat: *mut u8 = std::ptr::null_mut();
    let mut laa_size = 0;
    let mut laa_map_stride = 0;
    let mut have_cnv_tr = 0;

    if map.is_null() {
        return -1;
    }

    vcf::bcf_unpack(line, hts_sys::BCF_UN_ALL as c_int);

    let n_sample = (*line).n_sample() as c_int;
    let mut str_: kstring_t = std::mem::zeroed();

    macro_rules! err {
        () => {{
            libc::free(str_.s.cast());
            libc::free(map.cast());
            libc::free(laa_map.cast());
            libc::free(lr_orig.cast());
            libc::free(laa.cast());
            libc::free(dat.cast());
            return -1;
        }};
    }

    kputs(*(*line).d.allele.add(0), &mut str_);

    let mut nrm = 0;
    *map.add(0) = 0;
    let mut j = 1;
    for i in 1..(*line).n_allele() as c_int {
        if libc::strcmp(*(*line).d.allele.add(i as usize), c"<CNV:TR>".as_ptr()) == 0 {
            have_cnv_tr = 1;
        }

        if kbs_exists(rm_set, i) != 0 {
            *(*line).d.allele.add(i as usize) = std::ptr::null_mut();
            *map.add(i as usize) = -1;
            nrm += 1;
            continue;
        }
        kputc(b',' as c_int, &mut str_);
        kputs(*(*line).d.allele.add(i as usize), &mut str_);
        *map.add(i as usize) = j;
        j += 1;
    }
    if nrm == 0 {
        libc::free(str_.s.cast());
        libc::free(map.cast());
        return 0;
    }

    let n_r_ori = (*line).n_allele() as c_int;
    let n_r_new = (*line).n_allele() as c_int - nrm;
    if n_r_new <= 0 {
        hts_sys::hts_log(
            hts_sys::htsLogLevel_HTS_LOG_ERROR,
            c"bcf_remove_allele_set".as_ptr(),
            c"Cannot remove reference allele at %s:%ld [%d]".as_ptr(),
            vcf::bcf_seqname_safe(header, line),
            (*line).pos + 1,
            n_r_new,
        );
        libc::free(str_.s.cast());
        libc::free(map.cast());
        return -1;
    }
    let n_a_ori = n_r_ori - 1;
    let n_a_new = n_r_new - 1;
    let n_g_ori = n_r_ori * (n_r_ori + 1) / 2;
    let mut n_g_new = n_r_new * (n_r_new + 1) / 2;

    if vcf::bcf_update_alleles_str(header, line, str_.s) < 0 {
        libc::free(str_.s.cast());
        libc::free(map.cast());
        return -1;
    }

    if have_cnv_tr != 0
        && vcf::vcfutils_c_561_fixup_cnv_tr_info_tags(header, line, n_a_ori as usize, rm_set.cast())
            < 0
    {
        hts_sys::hts_log(
            hts_sys::htsLogLevel_HTS_LOG_ERROR,
            c"bcf_remove_allele_set".as_ptr(),
            c"Out of memory".as_ptr(),
        );
        libc::free(str_.s.cast());
        libc::free(map.cast());
        return -1;
    }

    let mut mdat = 0;
    let mut mdat_bytes = 0;
    for i in 0..(*line).n_info() {
        let info = (*line).d.info.add(i as usize);
        let info_key = (*info).key as usize;
        let id = (*(*header).id[hts_sys::BCF_DT_ID as usize].add(info_key)).key;
        let mut vlen = ((*(*(*header).id[hts_sys::BCF_DT_ID as usize].add(info_key)).val).info
            [hts_sys::BCF_HL_INFO as usize]
            >> 8
            & 0xf) as c_int;

        let multiple = if vlen == hts_sys::BCF_VL_VAR as c_int {
            vcf::vcfutils_c_254_is_special_info_type(id)
        } else {
            1
        };
        if multiple > 1 {
            vlen = hts_sys::BCF_VL_A as c_int;
        }

        if (vl_a_g_r & (1_u32 << vlen)) == 0 {
            continue;
        }
        let type_ = ((*(*(*header).id[hts_sys::BCF_DT_ID as usize].add(info_key)).val).info
            [hts_sys::BCF_HL_INFO as usize]
            >> 4
            & 0xf) as c_int;
        if type_ == hts_sys::BCF_HT_FLAG as c_int {
            continue;
        }
        let size =
            if type_ == hts_sys::BCF_HT_REAL as c_int || type_ == hts_sys::BCF_HT_INT as c_int {
                4
            } else {
                1
            };

        mdat = mdat_bytes / size;
        let mut nret = vcf::bcf_get_info_values(
            header,
            line,
            id,
            (&mut dat as *mut *mut u8).cast::<*mut c_void>(),
            &mut mdat,
            type_,
        );
        mdat_bytes = mdat * size;
        if nret < 0 {
            err!();
        }
        if nret == 0 {
            continue;
        }

        if type_ == hts_sys::BCF_HT_STR as c_int {
            str_.l = 0;
            let mut ss = dat.cast::<c_char>();
            let mut se = dat.cast::<c_char>();
            let s0 = *ss;
            if vlen == hts_sys::BCF_VL_A as c_int || vlen == hts_sys::BCF_VL_R as c_int {
                let mut inc = 0;
                let nexp = if vlen == hts_sys::BCF_VL_A as c_int {
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
                        kputc(b',' as c_int, &mut str_);
                    }
                    kputsn(ss, se.offset_from(ss) as usize, &mut str_);
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
                            kputc(b',' as c_int, &mut str_);
                        }
                        kputsn(ss, se.offset_from(ss) as usize, &mut str_);
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
                x if x == hts_sys::BCF_BT_INT8 as c_int => {
                    *(*info).vptr.cast::<i8>() as c_int == hts_sys::bcf_int8_missing
                }
                x if x == hts_sys::BCF_BT_INT16 as c_int => {
                    i16::from_le(*(*info).vptr.cast::<i16>()) as c_int == hts_sys::bcf_int16_missing
                }
                x if x == hts_sys::BCF_BT_INT32 as c_int => {
                    i32::from_le(*(*info).vptr.cast::<i32>()) == hts_sys::bcf_int32_missing
                }
                x if x == hts_sys::BCF_BT_FLOAT as c_int => {
                    f32::from_bits(u32::from_le(*(*info).vptr.cast::<u32>())).to_bits()
                        == hts_sys::bcf_float_missing
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
        if vlen == hts_sys::BCF_VL_A as c_int || vlen == hts_sys::BCF_VL_R as c_int {
            let (inc, ntop, new_ndat) = if vlen == hts_sys::BCF_VL_A as c_int {
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
            if type_ == hts_sys::BCF_HT_INT as c_int {
                let ptr = dat.cast::<i32>();
                for jj in 0..ntop {
                    if *ptr.add(jj as usize) == hts_sys::bcf_int32_vector_end {
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
            } else if type_ == hts_sys::BCF_HT_REAL as c_int {
                let ptr = dat.cast::<f32>();
                for jj in 0..ntop {
                    if (*ptr.add(jj as usize)).to_bits() == hts_sys::bcf_float_vector_end {
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
            let mut l_ori = -1;
            let mut l_new = 0;
            if type_ == hts_sys::BCF_HT_INT as c_int {
                let ptr = dat.cast::<i32>();
                for jj in 0..n_r_ori {
                    for k in 0..=jj {
                        l_ori += 1;
                        if *ptr.add(l_ori as usize) == hts_sys::bcf_int32_vector_end {
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
            } else if type_ == hts_sys::BCF_HT_REAL as c_int {
                let ptr = dat.cast::<f32>();
                for jj in 0..n_r_ori {
                    for k in 0..=jj {
                        l_ori += 1;
                        if (*ptr.add(l_ori as usize)).to_bits() == hts_sys::bcf_float_vector_end {
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
        if *map.add(i as usize) != i {
            break;
        }
        i += 1;
    }
    if i < n_r_ori {
        mdat = mdat_bytes / 4;
        let mut nret = vcf::bcf_get_format_values(
            header,
            line,
            c"GT".as_ptr(),
            (&mut dat as *mut *mut u8).cast::<*mut c_void>(),
            &mut mdat,
            hts_sys::BCF_HT_INT as c_int,
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
                    if v == hts_sys::bcf_int32_vector_end {
                        break;
                    }
                    let al = (v >> 1) - 1;
                    if !(al < n_r_ori && *map.add(al as usize) >= -1) {
                        err!();
                    }
                    *ptr.add(jj as usize) = if *map.add(al as usize) < 0 {
                        0
                    } else {
                        ((*map.add(al as usize) + 1) << 1) | (v & 1)
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
                hts_sys::BCF_HT_INT as c_int,
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
        (&mut laa as *mut *mut c_int).cast::<*mut c_void>(),
        &mut laa_size,
        hts_sys::BCF_HT_INT as c_int,
    );
    if num_laa < -1 && num_laa != -3 {
        err!();
    }
    if num_laa > 0 {
        let num_laa_vals = num_laa / n_sample;
        laa_map_stride = num_laa_vals + 1;
        let mut max_k = 0;
        laa_map = libc::malloc(size_of::<c_int>() * laa_map_stride as usize * n_sample as usize)
            .cast::<c_int>();
        if laa_map.is_null() {
            err!();
        }
        lr_orig = libc::malloc(size_of::<c_int>() * n_sample as usize).cast::<c_int>();
        if lr_orig.is_null() {
            err!();
        }
        let mut laa_changed = 0;
        for sample in 0..n_sample {
            let sample_laa = laa.add((sample * num_laa_vals) as usize);
            let sample_laa_map = laa_map.add((sample * laa_map_stride) as usize);
            *sample_laa_map = 0;
            let mut k = 0;
            let mut jj = 0;
            while jj < num_laa_vals {
                let val = *sample_laa.add(jj as usize);
                if val == hts_sys::bcf_int32_vector_end || val == hts_sys::bcf_int32_missing {
                    break;
                }
                let allele = if val > 0 && val < n_r_ori { val } else { 0 };
                if allele == 0 || *map.add(allele as usize) < 0 {
                    *sample_laa_map.add(jj as usize + 1) = -1;
                    laa_changed = 1;
                    jj += 1;
                    continue;
                }
                if allele != *map.add(allele as usize) {
                    laa_changed = 1;
                }
                *sample_laa.add(k as usize) = *map.add(allele as usize);
                k += 1;
                *sample_laa_map.add(jj as usize + 1) = k;
                jj += 1;
            }
            *lr_orig.add(sample as usize) = jj + 1;
            if max_k < k {
                max_k = k;
            }
            while jj < num_laa_vals {
                *sample_laa_map.add(jj as usize + 1) = -1;
                jj += 1;
            }
            while k < num_laa_vals {
                *sample_laa.add(k as usize) = if k > 0 {
                    hts_sys::bcf_int32_vector_end
                } else {
                    hts_sys::bcf_int32_missing
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
                        *laa.add(sample as usize) = hts_sys::bcf_int32_missing;
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
                hts_sys::BCF_HT_INT as c_int,
            ) < 0
            {
                err!();
            }
        }
    }

    for fmt_i in 0..(*line).n_fmt() {
        let fmt = (*line).d.fmt.add(fmt_i as usize);
        let fmt_id = (*fmt).id as usize;
        let id = (*(*header).id[hts_sys::BCF_DT_ID as usize].add(fmt_id)).key;
        let vlen = ((*(*(*header).id[hts_sys::BCF_DT_ID as usize].add(fmt_id)).val).info
            [hts_sys::BCF_HL_FMT as usize]
            >> 8
            & 0xf) as c_int;

        if (vl_a_g_r_la_lg_lr & (1_u32 << vlen)) == 0 {
            continue;
        }
        let is_local = ((vl_la_lg_lr & (1_u32 << vlen)) != 0) as c_int;
        let type_ = ((*(*(*header).id[hts_sys::BCF_DT_ID as usize].add(fmt_id)).val).info
            [hts_sys::BCF_HL_FMT as usize]
            >> 4
            & 0xf) as c_int;
        if type_ == hts_sys::BCF_HT_FLAG as c_int {
            continue;
        }
        let size =
            if type_ == hts_sys::BCF_HT_REAL as c_int || type_ == hts_sys::BCF_HT_INT as c_int {
                4
            } else {
                1
            };

        mdat = mdat_bytes / size;
        let mut nret = vcf::bcf_get_format_values(
            header,
            line,
            id,
            (&mut dat as *mut *mut u8).cast::<*mut c_void>(),
            &mut mdat,
            type_,
        );
        mdat_bytes = mdat * size;
        if nret < 0 {
            err!();
        }
        if nret == 0 {
            continue;
        }

        if type_ == hts_sys::BCF_HT_STR as c_int {
            let width = nret / n_sample;
            str_.l = 0;
            if (vl_a_r_la_lr & (1_u32 << vlen)) != 0 {
                let mut nexp = 0;
                let mut inc = 0;
                match vlen {
                    x if x == hts_sys::BCF_VL_A as c_int => {
                        nexp = n_a_ori;
                        inc = 1;
                    }
                    x if x == hts_sys::BCF_VL_R as c_int => nexp = n_r_ori,
                    x if x == BCF_VL_LA => {
                        inc = 1;
                        if laa_map.is_null() {
                            err!();
                        }
                    }
                    x if x == BCF_VL_LR => {
                        if laa_map.is_null() {
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
                        laa_map.add((sample * laa_map_stride) as usize)
                    } else {
                        map
                    };
                    if is_local != 0 {
                        nexp = *lr_orig.add(sample as usize) - inc;
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
                            kputc(b',' as c_int, &mut str_);
                        }
                        kputsn(ss, ptr.offset_from(ss) as usize, &mut str_);
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
                        kputc(if l == 0 { b'.' } else { 0 } as c_int, &mut str_);
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
                        *lr_orig.add(sample as usize)
                    } else {
                        n_r_ori
                    };
                    let sample_n_g_ori = if is_local != 0 {
                        sample_n_r_ori * (sample_n_r_ori + 1) / 2
                    } else {
                        n_g_ori
                    };
                    let sample_map = if is_local != 0 {
                        laa_map.add((sample * laa_map_stride) as usize)
                    } else {
                        map
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
                        kputc(b'.' as c_int, &mut str_);
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
                                    kputc(b',' as c_int, &mut str_);
                                }
                                kputsn(ss, ptr.offset_from(ss) as usize, &mut str_);
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
                                kputc(b',' as c_int, &mut str_);
                            }
                            kputsn(ss, ptr.offset_from(ss) as usize, &mut str_);
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
                        kputc(0, &mut str_);
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
        if nori == 1 && !(vlen == hts_sys::BCF_VL_A as c_int && nori == n_a_ori) {
            let mut all_missing = 1;
            match (*fmt).type_ {
                x if x == hts_sys::BCF_BT_INT8 as c_int => {
                    for sample in 0..n_sample {
                        let val = *(*fmt).p.add((sample * (*fmt).size) as usize).cast::<i8>();
                        if val as c_int != hts_sys::bcf_int8_missing {
                            all_missing = 0;
                            break;
                        }
                    }
                }
                x if x == hts_sys::BCF_BT_INT16 as c_int => {
                    for sample in 0..n_sample {
                        let val =
                            i16::from_le(*(*fmt).p.add((sample * (*fmt).size) as usize).cast());
                        if val as c_int != hts_sys::bcf_int16_missing {
                            all_missing = 0;
                            break;
                        }
                    }
                }
                x if x == hts_sys::BCF_BT_INT32 as c_int => {
                    for sample in 0..n_sample {
                        let val =
                            i32::from_le(*(*fmt).p.add((sample * (*fmt).size) as usize).cast());
                        if val != hts_sys::bcf_int32_missing {
                            all_missing = 0;
                            break;
                        }
                    }
                }
                x if x == hts_sys::BCF_BT_FLOAT as c_int => {
                    for sample in 0..n_sample {
                        let val =
                            u32::from_le(*(*fmt).p.add((sample * (*fmt).size) as usize).cast());
                        if val != hts_sys::bcf_float_missing {
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
        if (vl_a_r_la_lr & (1_u32 << vlen)) != 0
            || (vlen == hts_sys::BCF_VL_G as c_int && nori == n_r_ori)
        {
            let mut inc = 0;
            let nnew;
            match vlen {
                x if x == hts_sys::BCF_VL_A as c_int => {
                    if nori != n_a_ori {
                        err!();
                    }
                    ndat = n_a_new * n_sample;
                    nnew = n_a_new;
                    inc = 1;
                }
                x if x == hts_sys::BCF_VL_R as c_int => {
                    if nori != n_r_ori {
                        err!();
                    }
                    ndat = n_r_new * n_sample;
                    nnew = n_r_new;
                }
                x if x == hts_sys::BCF_VL_G as c_int => {
                    ndat = n_r_new * n_sample;
                    nnew = n_r_new;
                }
                x if x == BCF_VL_LA => {
                    inc = 1;
                    if laa_map.is_null() {
                        err!();
                    }
                    nnew = nori;
                    ndat = nori * n_sample;
                }
                x if x == BCF_VL_LR => {
                    if laa_map.is_null() {
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

            if type_ == hts_sys::BCF_HT_INT as c_int {
                for sample in 0..n_sample {
                    let ptr_src = dat.cast::<i32>().add((sample * nori) as usize);
                    let ptr_dst = dat.cast::<i32>().add((sample * nnew) as usize);
                    let sample_map = if is_local != 0 {
                        laa_map.add((sample * laa_map_stride) as usize)
                    } else {
                        map
                    };
                    let sample_nori = if is_local != 0 {
                        std::cmp::min(*lr_orig.add(sample as usize) - inc, nori)
                    } else {
                        nori
                    };
                    let mut k_dst = 0;
                    for k_src in 0..sample_nori {
                        if *ptr_src.add(k_src as usize) == hts_sys::bcf_int32_vector_end {
                            break;
                        }
                        if *sample_map.add((k_src + inc) as usize) < 0 {
                            continue;
                        }
                        *ptr_dst.add(k_dst as usize) = *ptr_src.add(k_src as usize);
                        k_dst += 1;
                    }
                    if k_dst == 0 {
                        *ptr_dst.add(k_dst as usize) = hts_sys::bcf_int32_missing;
                        k_dst += 1;
                    }
                    while k_dst < nnew {
                        *ptr_dst.add(k_dst as usize) = hts_sys::bcf_int32_vector_end;
                        k_dst += 1;
                    }
                }
            } else if type_ == hts_sys::BCF_HT_REAL as c_int {
                for sample in 0..n_sample {
                    let ptr_src = dat.cast::<f32>().add((sample * nori) as usize);
                    let ptr_dst = dat.cast::<f32>().add((sample * nnew) as usize);
                    let sample_map = if is_local != 0 {
                        laa_map.add((sample * laa_map_stride) as usize)
                    } else {
                        map
                    };
                    let sample_nori = if is_local != 0 {
                        std::cmp::min(*lr_orig.add(sample as usize) - inc, nori)
                    } else {
                        nori
                    };
                    let mut k_dst = 0;
                    for k_src in 0..sample_nori {
                        if (*ptr_src.add(k_src as usize)).to_bits() == hts_sys::bcf_float_vector_end
                        {
                            break;
                        }
                        if *sample_map.add((k_src + inc) as usize) < 0 {
                            continue;
                        }
                        *ptr_dst.add(k_dst as usize) = *ptr_src.add(k_src as usize);
                        k_dst += 1;
                    }
                    if k_dst == 0 {
                        *ptr_dst.add(k_dst as usize) = f32::from_bits(hts_sys::bcf_float_missing);
                        k_dst += 1;
                    }
                    while k_dst < nnew {
                        *ptr_dst.add(k_dst as usize) =
                            f32::from_bits(hts_sys::bcf_float_vector_end);
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
            if type_ == hts_sys::BCF_HT_INT as c_int {
                for sample in 0..n_sample {
                    let ptr_src = dat.cast::<i32>().add((sample * nori) as usize);
                    let ptr_dst = dat.cast::<i32>().add((sample * n_g_new) as usize);
                    let sample_map = if is_local != 0 {
                        laa_map.add((sample * laa_map_stride) as usize)
                    } else {
                        map
                    };
                    let sample_n_r_ori = if is_local != 0 {
                        *lr_orig.add(sample as usize)
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
                        if *ptr_src.add(k_src as usize) == hts_sys::bcf_int32_vector_end {
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
                        let mut k_src = -1;
                        'outer_int: for ia in 0..sample_n_r_ori {
                            for ib in 0..=ia {
                                k_src += 1;
                                if *ptr_src.add(k_src as usize) == hts_sys::bcf_int32_vector_end {
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
                        *ptr_dst.add(k_dst as usize) = hts_sys::bcf_int32_vector_end;
                        k_dst += 1;
                    }
                }
            } else if type_ == hts_sys::BCF_HT_REAL as c_int {
                for sample in 0..n_sample {
                    let ptr_src = dat.cast::<f32>().add((sample * nori) as usize);
                    let ptr_dst = dat.cast::<f32>().add((sample * n_g_new) as usize);
                    let sample_map = if is_local != 0 {
                        laa_map.add((sample * laa_map_stride) as usize)
                    } else {
                        map
                    };
                    let sample_n_r_ori = if is_local != 0 {
                        *lr_orig.add(sample as usize)
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
                        if (*ptr_src.add(k_src as usize)).to_bits() == hts_sys::bcf_float_vector_end
                        {
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
                        let mut k_src = -1;
                        'outer_real: for ia in 0..sample_n_r_ori {
                            for ib in 0..=ia {
                                k_src += 1;
                                if (*ptr_src.add(k_src as usize)).to_bits()
                                    == hts_sys::bcf_float_vector_end
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
                        *ptr_dst.add(k_dst as usize) =
                            f32::from_bits(hts_sys::bcf_float_vector_end);
                        k_dst += 1;
                    }
                }
            }
        }
        if vcf::bcf_update_format(header, line, id, dat.cast(), ndat, type_) < 0 {
            err!();
        }
    }

    libc::free(str_.s.cast());
    libc::free(map.cast());
    libc::free(laa_map.cast());
    libc::free(lr_orig.cast());
    libc::free(laa.cast());
    libc::free(dat.cast());
    0
}
