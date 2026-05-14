use std::{
    collections::BTreeMap,
    ffi::{c_char, c_int, c_void},
    ptr,
};

use super::{
    c_compat,
    hts::{
        hts_name2id_f, hts_pair_pos_t, hts_parse_region, hts_pos_t, hts_reglist_t, HTS_IDX_NOCOOR,
        HTS_IDX_START, HTS_PARSE_THOUSANDS_SEP,
    },
};

pub unsafe extern "C" fn region_c_41_compare_hts_pair_pos_t(
    av: *const c_void,
    bv: *const c_void,
) -> c_int {
    let a = av.cast::<hts_pair_pos_t>();
    let b = bv.cast::<hts_pair_pos_t>();
    if (*a).beg < (*b).beg {
        return -1;
    }
    if (*a).beg > (*b).beg {
        return 1;
    }
    if (*a).end < (*b).end {
        return -1;
    }
    if (*a).end > (*b).end {
        return 1;
    }
    0
}

pub unsafe fn region_c_177_hts_reglist_create(
    argv: *mut *mut c_char,
    argc: c_int,
    r_count: *mut c_int,
    hdr: *mut c_void,
    getid: hts_name2id_f,
) -> *mut hts_reglist_t {
    if argv.is_null() || argc < 1 {
        return ptr::null_mut();
    }

    let mut regions: BTreeMap<c_int, Vec<hts_pair_pos_t>> = BTreeMap::new();
    for i in 0..argc {
        let arg = *argv.add(i as usize);
        if arg.is_null() {
            continue;
        }
        let mut tid = 0;
        let mut beg: hts_pos_t = 0;
        let mut end: hts_pos_t = 0;

        let q = if libc::strcmp(arg, c".".as_ptr()) == 0 {
            tid = HTS_IDX_START;
            beg = 0;
            end = i64::MAX;
            arg.add(1)
        } else if libc::strcmp(arg, c"*".as_ptr()) == 0 {
            tid = HTS_IDX_NOCOOR;
            beg = 0;
            end = i64::MAX;
            arg.add(1)
        } else {
            hts_parse_region(
                arg,
                &mut tid,
                &mut beg,
                &mut end,
                getid,
                hdr,
                HTS_PARSE_THOUSANDS_SEP,
            )
        };

        if q.is_null() {
            if tid < -1 {
                return ptr::null_mut();
            }
            continue;
        }

        regions
            .entry(tid)
            .or_default()
            .push(hts_pair_pos_t { beg, end });
    }

    let mut compacted: Vec<(c_int, Vec<hts_pair_pos_t>)> = Vec::new();
    for (tid, mut intervals) in regions {
        if intervals.is_empty() {
            continue;
        }
        intervals.sort_by(|a, b| match a.beg.cmp(&b.beg) {
            std::cmp::Ordering::Equal => a.end.cmp(&b.end),
            ord => ord,
        });
        let mut new_n = 0usize;
        for j in 1..intervals.len() {
            if intervals[new_n].end < intervals[j].beg {
                new_n += 1;
                intervals[new_n] = intervals[j];
            } else if intervals[new_n].end < intervals[j].end {
                intervals[new_n].end = intervals[j].end;
            }
        }
        intervals.truncate(new_n + 1);
        compacted.push((tid, intervals));
    }

    *r_count = compacted.len() as c_int;
    if *r_count == 0 {
        return ptr::null_mut();
    }

    let h_reglist = c_compat::calloc(*r_count as u64, std::mem::size_of::<hts_reglist_t>() as u64)
        .cast::<hts_reglist_t>();
    if h_reglist.is_null() {
        return ptr::null_mut();
    }

    let mut l_count = 0;
    for (tid, intervals) in compacted {
        let n = intervals.len();
        let bytes = n * std::mem::size_of::<hts_pair_pos_t>();
        let a = c_compat::malloc(bytes as u64).cast::<hts_pair_pos_t>();
        if a.is_null() {
            region_c_266_hts_reglist_free(h_reglist, l_count);
            return ptr::null_mut();
        }
        ptr::copy_nonoverlapping(intervals.as_ptr(), a, n);
        let out = h_reglist.add(l_count as usize);
        (*out).tid = tid;
        (*out).intervals = a;
        (*out).count = n as u32;
        (*out).min_beg = (*a).beg;
        (*out).max_end = (*a.add(n - 1)).end;
        l_count += 1;
    }

    h_reglist
}

pub unsafe fn region_c_266_hts_reglist_free(reglist: *mut hts_reglist_t, count: c_int) {
    if !reglist.is_null() {
        for i in 0..count {
            c_compat::free((*reglist.add(i as usize)).intervals.cast());
        }
        c_compat::free(reglist.cast());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn test_name2id(_data: *mut c_void, name: *const c_char) -> c_int {
        if libc::strcmp(name, c"chr1".as_ptr()) == 0 {
            0
        } else if libc::strcmp(name, c"chr2".as_ptr()) == 0 {
            1
        } else {
            -1
        }
    }

    #[test]
    fn compare_hts_pair_pos_t_orders_begin_then_end() {
        unsafe {
            let a = hts_sys::hts_pair_pos_t { beg: 10, end: 20 };
            let b = hts_sys::hts_pair_pos_t { beg: 10, end: 25 };
            let c = hts_sys::hts_pair_pos_t { beg: 11, end: 12 };
            assert_eq!(
                region_c_41_compare_hts_pair_pos_t(
                    (&a as *const hts_sys::hts_pair_pos_t).cast(),
                    (&b as *const hts_sys::hts_pair_pos_t).cast()
                ),
                -1
            );
            assert_eq!(
                region_c_41_compare_hts_pair_pos_t(
                    (&c as *const hts_sys::hts_pair_pos_t).cast(),
                    (&b as *const hts_sys::hts_pair_pos_t).cast()
                ),
                1
            );
            assert_eq!(
                region_c_41_compare_hts_pair_pos_t(
                    (&a as *const hts_sys::hts_pair_pos_t).cast(),
                    (&a as *const hts_sys::hts_pair_pos_t).cast()
                ),
                0
            );
        }
    }

    #[test]
    fn hts_reglist_create_rejects_empty_inputs_like_c() {
        unsafe {
            let mut count = 7;
            assert!(region_c_177_hts_reglist_create(
                std::ptr::null_mut(),
                0,
                &mut count,
                std::ptr::null_mut(),
                None,
            )
            .is_null());
            region_c_266_hts_reglist_free(std::ptr::null_mut(), 0);
        }
    }

    #[test]
    fn hts_reglist_create_parses_compacts_and_steals_intervals_like_c() {
        unsafe {
            let mut args = [
                c"chr1:10-20".as_ptr().cast_mut(),
                c"chr1:18-25".as_ptr().cast_mut(),
                c"chr2:1-3".as_ptr().cast_mut(),
                c"unknown:1-2".as_ptr().cast_mut(),
                c"*".as_ptr().cast_mut(),
            ];
            let mut count = 0;
            let reglist = region_c_177_hts_reglist_create(
                args.as_mut_ptr(),
                args.len() as c_int,
                &mut count,
                std::ptr::null_mut(),
                Some(test_name2id),
            );
            assert!(!reglist.is_null());
            assert_eq!(count, 3);

            let mut by_tid = std::collections::BTreeMap::new();
            for i in 0..count {
                let r = &*reglist.add(i as usize);
                let intervals = std::slice::from_raw_parts(r.intervals, r.count as usize);
                by_tid.insert(
                    r.tid,
                    (
                        r.count,
                        r.min_beg,
                        r.max_end,
                        intervals
                            .iter()
                            .map(|iv| (iv.beg, iv.end))
                            .collect::<Vec<_>>(),
                    ),
                );
            }
            assert_eq!(by_tid.get(&0).unwrap(), &(1, 9, 25, vec![(9, 25)]));
            assert_eq!(by_tid.get(&1).unwrap(), &(1, 0, 3, vec![(0, 3)]));
            assert_eq!(
                by_tid.get(&HTS_IDX_NOCOOR).unwrap(),
                &(1, 0, i64::MAX, vec![(0, i64::MAX)])
            );

            region_c_266_hts_reglist_free(reglist, count);
        }
    }
}
