use std::{
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

// original: reglist (htslib/region.c:31)
#[repr(C)]
pub struct region_c_31_reglist {
    n: u32,
    m: u32,
    a: *mut hts_pair_pos_t,
    tid: c_int,
}

// original: reghash_t (htslib/region.c:38)
pub struct region_c_38_reghash_t {
    entries: Vec<region_c_31_reglist>,
}

// original: compare_hts_pair_pos_t (htslib/region.c:41)
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

// original: reg_compact (htslib/region.c:87)
pub unsafe fn region_c_87_reg_compact(h: *mut region_c_38_reghash_t) -> c_int {
    let mut count = 0;

    if h.is_null() {
        return 0;
    }

    for p in (*h).entries.iter_mut() {
        if p.n == 0 {
            continue;
        }

        libc::qsort(
            p.a.cast(),
            p.n as usize,
            std::mem::size_of::<hts_pair_pos_t>(),
            Some(region_c_41_compare_hts_pair_pos_t),
        );

        let mut new_n: u32 = 0;
        let mut j: u32 = 1;
        while j < p.n {
            if (*p.a.add(new_n as usize)).end < (*p.a.add(j as usize)).beg {
                new_n += 1;
                (*p.a.add(new_n as usize)).beg = (*p.a.add(j as usize)).beg;
                (*p.a.add(new_n as usize)).end = (*p.a.add(j as usize)).end;
            } else if (*p.a.add(new_n as usize)).end < (*p.a.add(j as usize)).end {
                (*p.a.add(new_n as usize)).end = (*p.a.add(j as usize)).end;
            }
            j += 1;
        }
        new_n += 1;
        if p.n > new_n {
            let new_a = c_compat::realloc(
                p.a.cast(),
                (new_n as usize * std::mem::size_of::<hts_pair_pos_t>()) as u64,
            )
            .cast::<hts_pair_pos_t>();
            if !new_a.is_null() {
                p.a = new_a;
            }
        }
        p.n = new_n;
        count += 1;
    }

    count
}

// original: reg_insert (htslib/region.c:123)
pub unsafe fn region_c_123_reg_insert(
    h: *mut region_c_38_reghash_t,
    tid: c_int,
    beg: hts_pos_t,
    end: hts_pos_t,
) -> c_int {
    if h.is_null() {
        return -1;
    }

    let mut idx = None;
    for (i, entry) in (*h).entries.iter().enumerate() {
        if entry.tid == tid {
            idx = Some(i);
            break;
        }
    }

    let idx = match idx {
        Some(i) => i,
        None => {
            if (*h).entries.try_reserve(1).is_err() {
                return -1;
            }
            (*h).entries.push(region_c_31_reglist {
                n: 0,
                m: 0,
                a: ptr::null_mut(),
                tid,
            });
            (*h).entries.len() - 1
        }
    };
    let entries = &mut (&mut (*h).entries)[..];
    let p = &mut entries[idx];

    if p.n == p.m {
        let new_m = if p.m != 0 { p.m.wrapping_shl(1) } else { 4 };
        if new_m == 0 {
            return -1;
        }
        let Some(bytes) = (new_m as usize).checked_mul(std::mem::size_of::<hts_pair_pos_t>())
        else {
            return -1;
        };
        let new_a = c_compat::realloc(p.a.cast(), bytes as u64).cast::<hts_pair_pos_t>();
        if new_a.is_null() {
            return -1;
        }
        p.m = new_m;
        p.a = new_a;
    }
    (*p.a.add(p.n as usize)).beg = beg;
    (*p.a.add(p.n as usize)).end = end;
    p.n += 1;

    0
}

fn region_khash_int_order(entries: &[region_c_31_reglist]) -> Vec<usize> {
    fn khash_resize_slots(old: &[Option<c_int>], new_n: usize) -> Vec<Option<c_int>> {
        let old_n = old.len();
        let mut keys = vec![None; new_n];
        for (i, key) in old.iter().copied().enumerate() {
            keys[i] = key;
        }
        let mut old_exists = old.iter().map(Option::is_some).collect::<Vec<_>>();
        let mut new_exists = vec![false; new_n];
        let new_mask = new_n - 1;

        for j in 0..old_n {
            if !old_exists[j] {
                continue;
            }

            let mut key = keys[j].unwrap();
            old_exists[j] = false;
            loop {
                let mut step = 0usize;
                let mut i = (key as u32 as usize) & new_mask;
                while new_exists[i] {
                    step += 1;
                    i = (i + step) & new_mask;
                }
                new_exists[i] = true;

                if i < old_n && old_exists[i] {
                    let old_key = keys[i].unwrap();
                    keys[i] = Some(key);
                    key = old_key;
                    old_exists[i] = false;
                } else {
                    keys[i] = Some(key);
                    break;
                }
            }
        }

        for (i, exists) in new_exists.iter().enumerate() {
            if !exists {
                keys[i] = None;
            }
        }
        keys
    }

    fn rounded_bucket_count(mut n: usize) -> usize {
        if n == 0 {
            return 4;
        }
        n -= 1;
        n |= n >> 1;
        n |= n >> 2;
        n |= n >> 4;
        n |= n >> 8;
        n |= n >> 16;
        if usize::BITS > 32 {
            n |= n >> 32;
        }
        (n + 1).max(4)
    }

    fn upper_bound(n_buckets: usize) -> usize {
        (n_buckets as f64 * 0.77 + 0.5) as usize
    }

    fn insert_slot(table: &mut [Option<c_int>], key: c_int) {
        let mask = table.len() - 1;
        let mut step = 0usize;
        let mut i = (key as u32 as usize) & mask;
        while table[i].is_some() {
            step += 1;
            i = (i + step) & mask;
        }
        table[i] = Some(key);
    }

    let mut table: Vec<Option<c_int>> = Vec::new();
    let mut occupied = 0usize;
    let mut upper = 0usize;

    for (size, entry) in entries.iter().enumerate() {
        if occupied >= upper {
            let new_n = rounded_bucket_count(table.len() + 1);
            table = khash_resize_slots(&table, new_n);
            occupied = size;
            upper = upper_bound(new_n);
        }
        insert_slot(&mut table, entry.tid);
        occupied += 1;
    }

    table
        .into_iter()
        .flatten()
        .filter_map(|tid| entries.iter().position(|entry| entry.tid == tid))
        .collect()
}

// original: reg_destroy (htslib/region.c:159)
pub unsafe fn region_c_159_reg_destroy(h: *mut region_c_38_reghash_t) {
    if h.is_null() {
        return;
    }

    let mut h_box = Box::from_raw(h);
    for entry in h_box.entries.iter_mut() {
        c_compat::free(entry.a.cast());
        entry.a = ptr::null_mut();
    }
}

// original: hts_reglist_create (htslib/region.c:177)
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

    let h = Box::into_raw(Box::new(region_c_38_reghash_t {
        entries: Vec::new(),
    }));
    if h.is_null() {
        return ptr::null_mut();
    }

    let mut l_count = 0;

    for i in 0..argc {
        let arg = *argv.add(i as usize);
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
                region_c_159_reg_destroy(h);
                return ptr::null_mut();
            }
            continue;
        }

        if region_c_123_reg_insert(h, tid, beg, end) != 0 {
            region_c_159_reg_destroy(h);
            return ptr::null_mut();
        }
    }

    *r_count = region_c_87_reg_compact(h);
    if *r_count == 0 {
        region_c_159_reg_destroy(h);
        return ptr::null_mut();
    }

    let h_reglist = c_compat::calloc(*r_count as u64, std::mem::size_of::<hts_reglist_t>() as u64)
        .cast::<hts_reglist_t>();
    if h_reglist.is_null() {
        region_c_159_reg_destroy(h);
        return ptr::null_mut();
    }

    let order = region_khash_int_order(&(*h).entries);
    for entry_idx in order {
        if l_count >= *r_count {
            break;
        }
        let p = &mut (&mut (*h).entries)[entry_idx];
        if p.n == 0 {
            continue;
        }

        (*h_reglist.add(l_count as usize)).tid = p.tid;
        (*h_reglist.add(l_count as usize)).intervals = p.a;
        (*h_reglist.add(l_count as usize)).count = p.n;
        p.a = ptr::null_mut();

        if p.n > 0 {
            (*h_reglist.add(l_count as usize)).min_beg =
                (*(*h_reglist.add(l_count as usize)).intervals).beg;
            (*h_reglist.add(l_count as usize)).max_end = (*(*h_reglist.add(l_count as usize))
                .intervals
                .add((p.n - 1) as usize))
            .end;
        } else {
            (*h_reglist.add(l_count as usize)).min_beg = 0;
            (*h_reglist.add(l_count as usize)).max_end = 0;
        }
        l_count += 1;
    }
    region_c_159_reg_destroy(h);

    h_reglist
}

// original: hts_reglist_free (htslib/region.c:266)
pub unsafe fn region_c_266_hts_reglist_free(reglist: *mut hts_reglist_t, count: c_int) {
    if !reglist.is_null() {
        for i in 0..count {
            if !(*reglist.add(i as usize)).intervals.is_null() {
                c_compat::free((*reglist.add(i as usize)).intervals.cast());
            }
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

    unsafe extern "C" fn fatal_name2id(data: *mut c_void, name: *const c_char) -> c_int {
        if libc::strcmp(name, c"fatal".as_ptr()) == 0 {
            -2
        } else {
            test_name2id(data, name)
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
    fn hts_reglist_create_rejects_null_argv_even_with_positive_argc() {
        unsafe {
            let mut count = 7;
            assert!(region_c_177_hts_reglist_create(
                std::ptr::null_mut(),
                1,
                &mut count,
                std::ptr::null_mut(),
                Some(test_name2id),
            )
            .is_null());
            assert_eq!(count, 7);
        }
    }

    #[test]
    fn hts_reglist_create_rejects_negative_argc_without_touching_count() {
        unsafe {
            let mut args = [c"chr1:1-2".as_ptr().cast_mut()];
            let mut count = 7;
            assert!(region_c_177_hts_reglist_create(
                args.as_mut_ptr(),
                -1,
                &mut count,
                std::ptr::null_mut(),
                Some(test_name2id),
            )
            .is_null());
            assert_eq!(count, 7);
        }
    }

    #[test]
    fn reg_compact_sorts_and_merges_adjacent_intervals_like_c() {
        unsafe {
            let mut h = Box::new(region_c_38_reghash_t {
                entries: Vec::new(),
            });
            assert_eq!(region_c_123_reg_insert(&mut *h, 0, 30, 40), 0);
            assert_eq!(region_c_123_reg_insert(&mut *h, 0, 10, 20), 0);
            assert_eq!(region_c_123_reg_insert(&mut *h, 0, 20, 30), 0);
            assert_eq!(region_c_123_reg_insert(&mut *h, 0, 45, 50), 0);

            assert_eq!(region_c_87_reg_compact(&mut *h), 1);
            let entry = &h.entries[0];
            assert_eq!(entry.n, 2);
            let intervals = std::slice::from_raw_parts(entry.a, entry.n as usize);
            assert_eq!((intervals[0].beg, intervals[0].end), (10, 40));
            assert_eq!((intervals[1].beg, intervals[1].end), (45, 50));

            region_c_159_reg_destroy(Box::into_raw(h));
        }
    }

    #[test]
    fn reg_compact_merges_duplicate_nested_and_per_tid_intervals() {
        unsafe {
            let mut h = Box::new(region_c_38_reghash_t {
                entries: Vec::new(),
            });
            assert_eq!(region_c_123_reg_insert(&mut *h, 1, 30, 40), 0);
            assert_eq!(region_c_123_reg_insert(&mut *h, 0, 10, 50), 0);
            assert_eq!(region_c_123_reg_insert(&mut *h, 0, 15, 20), 0);
            assert_eq!(region_c_123_reg_insert(&mut *h, 0, 10, 50), 0);
            assert_eq!(region_c_123_reg_insert(&mut *h, 1, 35, 45), 0);

            assert_eq!(region_c_87_reg_compact(&mut *h), 2);
            let entry0 = h.entries.iter().find(|entry| entry.tid == 0).unwrap();
            let intervals0 = std::slice::from_raw_parts(entry0.a, entry0.n as usize);
            assert_eq!(entry0.n, 1);
            assert_eq!((intervals0[0].beg, intervals0[0].end), (10, 50));

            let entry1 = h.entries.iter().find(|entry| entry.tid == 1).unwrap();
            let intervals1 = std::slice::from_raw_parts(entry1.a, entry1.n as usize);
            assert_eq!(entry1.n, 1);
            assert_eq!((intervals1[0].beg, intervals1[0].end), (30, 45));

            region_c_159_reg_destroy(Box::into_raw(h));
        }
    }

    #[test]
    fn reg_helpers_tolerate_null_contexts_like_c() {
        unsafe {
            assert_eq!(region_c_87_reg_compact(std::ptr::null_mut()), 0);
            assert_eq!(region_c_123_reg_insert(std::ptr::null_mut(), 0, 1, 2), -1);
            region_c_159_reg_destroy(std::ptr::null_mut());
            region_c_266_hts_reglist_free(std::ptr::null_mut(), 3);
        }
    }

    #[test]
    fn hts_reglist_free_tolerates_null_interval_entries() {
        unsafe {
            let reglist = c_compat::calloc(1, std::mem::size_of::<hts_reglist_t>() as u64)
                .cast::<hts_reglist_t>();
            assert!(!reglist.is_null());
            region_c_266_hts_reglist_free(reglist, 1);
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

    #[test]
    fn hts_reglist_create_returns_null_when_all_regions_are_ignored() {
        unsafe {
            let mut args = [
                c"unknown".as_ptr().cast_mut(),
                c"missing:1-2".as_ptr().cast_mut(),
            ];
            let mut count = 9;
            let reglist = region_c_177_hts_reglist_create(
                args.as_mut_ptr(),
                args.len() as c_int,
                &mut count,
                std::ptr::null_mut(),
                Some(test_name2id),
            );
            assert!(reglist.is_null());
            assert_eq!(count, 0);
        }
    }

    #[test]
    fn hts_reglist_create_aborts_on_fatal_name_lookup() {
        unsafe {
            let mut args = [
                c"chr1:1-2".as_ptr().cast_mut(),
                c"fatal:3-4".as_ptr().cast_mut(),
            ];
            let mut count = 9;
            let reglist = region_c_177_hts_reglist_create(
                args.as_mut_ptr(),
                args.len() as c_int,
                &mut count,
                std::ptr::null_mut(),
                Some(fatal_name2id),
            );
            assert!(reglist.is_null());
            assert_eq!(count, 9);
        }
    }

    #[test]
    fn hts_reglist_create_handles_start_marker_and_ignored_unknowns() {
        unsafe {
            let mut args = [
                c"unknown".as_ptr().cast_mut(),
                c".".as_ptr().cast_mut(),
                c"chr1:5-5".as_ptr().cast_mut(),
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
            assert_eq!(count, 2);

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
            assert_eq!(
                by_tid.get(&HTS_IDX_START).unwrap(),
                &(1, 0, i64::MAX, vec![(0, i64::MAX)])
            );
            assert_eq!(by_tid.get(&0).unwrap(), &(1, 4, 5, vec![(4, 5)]));
            region_c_266_hts_reglist_free(reglist, count);
        }
    }

    #[test]
    fn hts_reglist_create_compacts_duplicate_special_markers() {
        unsafe {
            let mut args = [
                c"*".as_ptr().cast_mut(),
                c"*".as_ptr().cast_mut(),
                c".".as_ptr().cast_mut(),
                c".".as_ptr().cast_mut(),
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
            assert_eq!(count, 2);

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
            assert_eq!(
                by_tid.get(&HTS_IDX_NOCOOR).unwrap(),
                &(1, 0, i64::MAX, vec![(0, i64::MAX)])
            );
            assert_eq!(
                by_tid.get(&HTS_IDX_START).unwrap(),
                &(1, 0, i64::MAX, vec![(0, i64::MAX)])
            );

            region_c_266_hts_reglist_free(reglist, count);
        }
    }

    #[test]
    fn hts_reglist_create_handles_special_markers_without_name_lookup() {
        unsafe {
            let mut args = [c".".as_ptr().cast_mut(), c"*".as_ptr().cast_mut()];
            let mut count = 0;
            let reglist = region_c_177_hts_reglist_create(
                args.as_mut_ptr(),
                args.len() as c_int,
                &mut count,
                std::ptr::null_mut(),
                None,
            );
            assert!(!reglist.is_null());
            assert_eq!(count, 2);

            let mut tids = Vec::new();
            for i in 0..count {
                tids.push((*reglist.add(i as usize)).tid);
            }
            tids.sort();
            assert_eq!(tids, [HTS_IDX_START, HTS_IDX_NOCOOR]);

            region_c_266_hts_reglist_free(reglist, count);
        }
    }

    #[test]
    fn hts_reglist_create_returns_entries_in_khash_bucket_scan_order() {
        unsafe {
            let mut args = [
                c"chr2:1-2".as_ptr().cast_mut(),
                c"chr1:1-2".as_ptr().cast_mut(),
                c".".as_ptr().cast_mut(),
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
            assert_eq!(count, 4);

            let tids = (0..count)
                .map(|i| (*reglist.add(i as usize)).tid)
                .collect::<Vec<_>>();
            assert_eq!(tids, [0, 1, HTS_IDX_START, HTS_IDX_NOCOOR]);

            region_c_266_hts_reglist_free(reglist, count);
        }
    }

    #[test]
    fn khash_order_matches_resize_kickout_for_colliding_tids() {
        let entries = [35, -5, 19, 39]
            .into_iter()
            .map(|tid| region_c_31_reglist {
                n: 1,
                m: 1,
                a: std::ptr::null_mut(),
                tid,
            })
            .collect::<Vec<_>>();

        let tids = region_khash_int_order(&entries)
            .into_iter()
            .map(|idx| entries[idx].tid)
            .collect::<Vec<_>>();
        assert_eq!(tids, [-5, 35, 19, 39]);
    }
}
