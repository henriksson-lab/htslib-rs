use std::ffi::c_void;

use super::hts::{
    hts_name2id_f, hts_pair_pos_t, hts_parse_region, hts_pos_t, HTS_IDX_NOCOOR, HTS_IDX_START,
    HTS_PARSE_THOUSANDS_SEP,
};

// original: reglist (htslib/region.c:31)
pub struct region_c_31_reglist {
    tid: i32,
    intervals: Vec<hts_pair_pos_t>,
}

/// Owned, idiomatic equivalent of the C `hts_reglist_t` entry: the region name
/// and interval list are carried as owned Rust rather than raw `malloc`ed
/// pointers plus a separate count.
pub struct region_reglist_entry {
    pub reg: Option<Vec<u8>>,
    pub intervals: Vec<hts_pair_pos_t>,
    pub tid: i32,
    pub min_beg: hts_pos_t,
    pub max_end: hts_pos_t,
}

// original: reghash_t (htslib/region.c:38)
pub struct region_c_38_reghash_t {
    entries: Vec<region_c_31_reglist>,
}

fn reghash_new() -> region_c_38_reghash_t {
    region_c_38_reghash_t {
        entries: Vec::new(),
    }
}

fn region_compare_hts_pair_pos(a: &hts_pair_pos_t, b: &hts_pair_pos_t) -> i32 {
    if a.beg < b.beg {
        return -1;
    }
    if a.beg > b.beg {
        return 1;
    }
    if a.end < b.end {
        return -1;
    }
    if a.end > b.end {
        return 1;
    }
    0
}

// original: compare_hts_pair_pos_t (htslib/region.c:41)
pub fn region_c_41_compare_hts_pair_pos_t(a: &hts_pair_pos_t, b: &hts_pair_pos_t) -> i32 {
    region_compare_hts_pair_pos(a, b)
}

// original: reg_compact (htslib/region.c:87)
pub fn region_c_87_reg_compact(h: &mut region_c_38_reghash_t) -> i32 {
    let mut count = 0;

    for p in h.entries.iter_mut() {
        if p.intervals.is_empty() {
            continue;
        }

        p.intervals
            .sort_by(|a, b| a.beg.cmp(&b.beg).then_with(|| a.end.cmp(&b.end)));

        let mut compacted = 0usize;
        for j in 1..p.intervals.len() {
            if p.intervals[compacted].end < p.intervals[j].beg {
                compacted += 1;
                p.intervals[compacted] = p.intervals[j];
            } else if p.intervals[compacted].end < p.intervals[j].end {
                p.intervals[compacted].end = p.intervals[j].end;
            }
        }
        p.intervals.truncate(compacted + 1);
        p.intervals.shrink_to_fit();
        count += 1;
    }

    count
}

// original: reg_insert (htslib/region.c:123)
pub fn region_c_123_reg_insert(
    h: &mut region_c_38_reghash_t,
    tid: i32,
    beg: hts_pos_t,
    end: hts_pos_t,
) -> i32 {
    let mut idx = None;
    for (i, entry) in h.entries.iter().enumerate() {
        if entry.tid == tid {
            idx = Some(i);
            break;
        }
    }

    let idx = match idx {
        Some(i) => i,
        None => {
            if h.entries.try_reserve(1).is_err() {
                return -1;
            }
            h.entries.push(region_c_31_reglist {
                tid,
                intervals: Vec::new(),
            });
            h.entries.len() - 1
        }
    };
    let p = &mut h.entries[idx];

    if p.intervals.try_reserve(1).is_err() {
        return -1;
    }
    p.intervals.push(hts_pair_pos_t { beg, end });

    0
}

fn region_khash_int_order(entries: &[region_c_31_reglist]) -> Vec<usize> {
    fn khash_resize_slots(old: &[Option<i32>], new_n: usize) -> Vec<Option<i32>> {
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

    fn insert_slot(table: &mut [Option<i32>], key: i32) {
        let mask = table.len() - 1;
        let mut step = 0usize;
        let mut i = (key as u32 as usize) & mask;
        while table[i].is_some() {
            step += 1;
            i = (i + step) & mask;
        }
        table[i] = Some(key);
    }

    let mut table: Vec<Option<i32>> = Vec::new();
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

// original: hts_reglist_create (htslib/region.c:177)
pub unsafe fn region_c_177_hts_reglist_create(
    argv: Option<&[&[u8]]>,
    r_count: Option<&mut i32>,
    hdr: Option<&mut c_void>,
    getid: hts_name2id_f,
) -> Option<Vec<region_reglist_entry>> {
    let Some(argv) = argv else {
        return None;
    };
    let Some(r_count) = r_count else {
        return None;
    };

    if argv.is_empty() {
        return None;
    };

    // Raw header pointer is only needed at the FFI `hts_parse_region` boundary.
    let hdr = hdr.map_or(std::ptr::null_mut(), |h| h as *mut c_void);

    let mut h = reghash_new();

    for &arg in argv.iter() {
        let mut tid = 0;
        let mut beg: hts_pos_t = 0;
        let mut end: hts_pos_t = 0;

        let parsed = if arg == b"." {
            tid = HTS_IDX_START;
            beg = 0;
            end = i64::MAX;
            true
        } else if arg == b"*" {
            tid = HTS_IDX_NOCOOR;
            beg = 0;
            end = i64::MAX;
            true
        } else if arg.contains(&0) {
            false
        } else {
            let mut region = Vec::with_capacity(arg.len() + 1);
            region.extend_from_slice(arg);
            region.push(0);
            unsafe {
                !hts_parse_region(
                    region.as_ptr().cast(),
                    &mut tid,
                    &mut beg,
                    &mut end,
                    getid,
                    hdr,
                    HTS_PARSE_THOUSANDS_SEP,
                )
                .is_null()
            }
        };

        if !parsed {
            if tid < -1 {
                return None;
            }
            continue;
        }

        if region_c_123_reg_insert(&mut h, tid, beg, end) != 0 {
            return None;
        }
    }

    let compacted_count = region_c_87_reg_compact(&mut h);
    *r_count = compacted_count;
    if compacted_count == 0 {
        return None;
    }
    let Ok(region_count) = usize::try_from(compacted_count) else {
        return None;
    };

    let mut h_reglist: Vec<region_reglist_entry> = Vec::new();
    if h_reglist.try_reserve_exact(region_count).is_err() {
        return None;
    }

    let order = region_khash_int_order(&h.entries);
    for entry_idx in order {
        if h_reglist.len() >= region_count {
            break;
        }
        let p = &mut h.entries[entry_idx];
        if p.intervals.is_empty() {
            continue;
        }

        let intervals = std::mem::take(&mut p.intervals);
        let min_beg = intervals.first().map_or(0, |interval| interval.beg);
        let max_end = intervals.last().map_or(0, |interval| interval.end);
        h_reglist.push(region_reglist_entry {
            reg: None,
            intervals,
            tid: p.tid,
            min_beg,
            max_end,
        });
    }

    Some(h_reglist)
}

// original: hts_reglist_free (htslib/region.c:266)
pub fn region_c_266_hts_reglist_free(reglist: Option<Vec<region_reglist_entry>>) {
    // Owned `Vec`s drop their intervals automatically; nothing to free manually.
    drop(reglist);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reglist_by_tid(
        reglist: &[region_reglist_entry],
    ) -> std::collections::BTreeMap<i32, (u32, hts_pos_t, hts_pos_t, Vec<(hts_pos_t, hts_pos_t)>)>
    {
        let mut by_tid = std::collections::BTreeMap::new();
        for r in reglist {
            by_tid.insert(
                r.tid,
                (
                    r.intervals.len() as u32,
                    r.min_beg,
                    r.max_end,
                    r.intervals
                        .iter()
                        .map(|iv| (iv.beg, iv.end))
                        .collect::<Vec<_>>(),
                ),
            );
        }
        by_tid
    }

    unsafe extern "C" fn test_name2id(_data: *mut c_void, name: *const i8) -> i32 {
        let name = std::ffi::CStr::from_ptr(name).to_bytes();
        if name == b"chr1" {
            0
        } else if name == b"chr2" {
            1
        } else {
            -1
        }
    }

    unsafe extern "C" fn fatal_name2id(data: *mut c_void, name: *const i8) -> i32 {
        if std::ffi::CStr::from_ptr(name).to_bytes() == b"fatal" {
            -2
        } else {
            test_name2id(data, name)
        }
    }

    #[test]
    fn compare_hts_pair_pos_t_orders_begin_then_end() {
        let a = hts_pair_pos_t { beg: 10, end: 20 };
        let b = hts_pair_pos_t { beg: 10, end: 25 };
        let c = hts_pair_pos_t { beg: 11, end: 12 };
        assert_eq!(region_compare_hts_pair_pos(&a, &b), -1);
        assert_eq!(region_compare_hts_pair_pos(&c, &b), 1);
        assert_eq!(region_compare_hts_pair_pos(&a, &a), 0);

        assert_eq!(region_c_41_compare_hts_pair_pos_t(&a, &b), -1);
        assert_eq!(region_c_41_compare_hts_pair_pos_t(&c, &b), 1);
        assert_eq!(region_c_41_compare_hts_pair_pos_t(&a, &a), 0);
    }

    #[test]
    fn hts_reglist_create_rejects_empty_inputs_like_c() {
        unsafe {
            let mut count = 7;
            let empty: [&[u8]; 0] = [];
            assert!(
                region_c_177_hts_reglist_create(Some(&empty), Some(&mut count), None, None,)
                    .is_none()
            );
            region_c_266_hts_reglist_free(None);
        }
    }

    #[test]
    fn hts_reglist_create_rejects_null_argv_even_with_positive_argc() {
        unsafe {
            let mut count = 7;
            assert!(region_c_177_hts_reglist_create(
                None,
                Some(&mut count),
                None,
                Some(test_name2id),
            )
            .is_none());
            assert_eq!(count, 7);
        }
    }

    #[test]
    fn hts_reglist_create_rejects_missing_count_without_touching_inputs() {
        unsafe {
            let args: [&[u8]; 1] = [b"chr1:1-2"];
            let count = 7;
            assert!(
                region_c_177_hts_reglist_create(Some(&args), None, None, Some(test_name2id),)
                    .is_none()
            );
            assert_eq!(count, 7);
        }
    }

    #[test]
    fn reg_compact_sorts_and_merges_adjacent_intervals_like_c() {
        let mut h = reghash_new();
        assert_eq!(region_c_123_reg_insert(&mut h, 0, 30, 40), 0);
        assert_eq!(region_c_123_reg_insert(&mut h, 0, 10, 20), 0);
        assert_eq!(region_c_123_reg_insert(&mut h, 0, 20, 30), 0);
        assert_eq!(region_c_123_reg_insert(&mut h, 0, 45, 50), 0);

        assert_eq!(region_c_87_reg_compact(&mut h), 1);
        let entry = &h.entries[0];
        assert_eq!(entry.intervals.len(), 2);
        let intervals = &entry.intervals;
        assert_eq!((intervals[0].beg, intervals[0].end), (10, 40));
        assert_eq!((intervals[1].beg, intervals[1].end), (45, 50));
    }

    #[test]
    fn reg_compact_merges_duplicate_nested_and_per_tid_intervals() {
        let mut h = reghash_new();
        assert_eq!(region_c_123_reg_insert(&mut h, 1, 30, 40), 0);
        assert_eq!(region_c_123_reg_insert(&mut h, 0, 10, 50), 0);
        assert_eq!(region_c_123_reg_insert(&mut h, 0, 15, 20), 0);
        assert_eq!(region_c_123_reg_insert(&mut h, 0, 10, 50), 0);
        assert_eq!(region_c_123_reg_insert(&mut h, 1, 35, 45), 0);

        assert_eq!(region_c_87_reg_compact(&mut h), 2);
        let entry0 = h.entries.iter().find(|entry| entry.tid == 0).unwrap();
        let intervals0 = &entry0.intervals;
        assert_eq!(entry0.intervals.len(), 1);
        assert_eq!((intervals0[0].beg, intervals0[0].end), (10, 50));

        let entry1 = h.entries.iter().find(|entry| entry.tid == 1).unwrap();
        let intervals1 = &entry1.intervals;
        assert_eq!(entry1.intervals.len(), 1);
        assert_eq!((intervals1[0].beg, intervals1[0].end), (30, 45));
    }

    #[test]
    fn hts_reglist_free_tolerates_missing_context_like_c() {
        region_c_266_hts_reglist_free(None);
    }

    #[test]
    fn hts_reglist_free_tolerates_null_interval_entries() {
        let reglist = vec![region_reglist_entry {
            reg: None,
            intervals: Vec::new(),
            tid: 0,
            min_beg: 0,
            max_end: 0,
        }];
        region_c_266_hts_reglist_free(Some(reglist));
    }

    #[test]
    fn hts_reglist_create_parses_compacts_and_steals_intervals_like_c() {
        unsafe {
            let args: [&[u8]; 5] = [
                b"chr1:10-20",
                b"chr1:18-25",
                b"chr2:1-3",
                b"unknown:1-2",
                b"*",
            ];
            let mut count = 0;
            let reglist = region_c_177_hts_reglist_create(
                Some(&args),
                Some(&mut count),
                None,
                Some(test_name2id),
            );
            let reglist = reglist.expect("reglist");
            assert_eq!(count, 3);

            let by_tid = reglist_by_tid(&reglist);
            assert_eq!(by_tid.get(&0).unwrap(), &(1, 9, 25, vec![(9, 25)]));
            assert_eq!(by_tid.get(&1).unwrap(), &(1, 0, 3, vec![(0, 3)]));
            assert_eq!(
                by_tid.get(&HTS_IDX_NOCOOR).unwrap(),
                &(1, 0, i64::MAX, vec![(0, i64::MAX)])
            );

            region_c_266_hts_reglist_free(Some(reglist));
        }
    }

    #[test]
    fn hts_reglist_create_returns_null_when_all_regions_are_ignored() {
        unsafe {
            let args: [&[u8]; 2] = [b"unknown", b"missing:1-2"];
            let mut count = 9;
            let reglist = region_c_177_hts_reglist_create(
                Some(&args),
                Some(&mut count),
                None,
                Some(test_name2id),
            );
            assert!(reglist.is_none());
            assert_eq!(count, 0);
        }
    }

    #[test]
    fn hts_reglist_create_aborts_on_fatal_name_lookup() {
        unsafe {
            let args: [&[u8]; 2] = [b"chr1:1-2", b"fatal:3-4"];
            let mut count = 9;
            let reglist = region_c_177_hts_reglist_create(
                Some(&args),
                Some(&mut count),
                None,
                Some(fatal_name2id),
            );
            assert!(reglist.is_none());
            assert_eq!(count, 9);
        }
    }

    #[test]
    fn hts_reglist_create_handles_start_marker_and_ignored_unknowns() {
        unsafe {
            let args: [&[u8]; 3] = [b"unknown", b".", b"chr1:5-5"];
            let mut count = 0;
            let reglist = region_c_177_hts_reglist_create(
                Some(&args),
                Some(&mut count),
                None,
                Some(test_name2id),
            );
            let reglist = reglist.expect("reglist");
            assert_eq!(count, 2);

            let by_tid = reglist_by_tid(&reglist);
            assert_eq!(
                by_tid.get(&HTS_IDX_START).unwrap(),
                &(1, 0, i64::MAX, vec![(0, i64::MAX)])
            );
            assert_eq!(by_tid.get(&0).unwrap(), &(1, 4, 5, vec![(4, 5)]));
            region_c_266_hts_reglist_free(Some(reglist));
        }
    }

    #[test]
    fn hts_reglist_create_compacts_duplicate_special_markers() {
        unsafe {
            let args: [&[u8]; 4] = [b"*", b"*", b".", b"."];
            let mut count = 0;
            let reglist = region_c_177_hts_reglist_create(
                Some(&args),
                Some(&mut count),
                None,
                Some(test_name2id),
            );
            let reglist = reglist.expect("reglist");
            assert_eq!(count, 2);

            let by_tid = reglist_by_tid(&reglist);
            assert_eq!(
                by_tid.get(&HTS_IDX_NOCOOR).unwrap(),
                &(1, 0, i64::MAX, vec![(0, i64::MAX)])
            );
            assert_eq!(
                by_tid.get(&HTS_IDX_START).unwrap(),
                &(1, 0, i64::MAX, vec![(0, i64::MAX)])
            );

            region_c_266_hts_reglist_free(Some(reglist));
        }
    }

    #[test]
    fn hts_reglist_create_handles_special_markers_without_name_lookup() {
        unsafe {
            let args: [&[u8]; 2] = [b".", b"*"];
            let mut count = 0;
            let reglist =
                region_c_177_hts_reglist_create(Some(&args), Some(&mut count), None, None);
            let reglist = reglist.expect("reglist");
            assert_eq!(count, 2);

            let mut tids = Vec::new();
            for entry in reglist.iter() {
                tids.push(entry.tid);
            }
            tids.sort();
            assert_eq!(tids, [HTS_IDX_START, HTS_IDX_NOCOOR]);

            region_c_266_hts_reglist_free(Some(reglist));
        }
    }

    #[test]
    fn hts_reglist_create_returns_entries_in_khash_bucket_scan_order() {
        unsafe {
            let args: [&[u8]; 4] = [b"chr2:1-2", b"chr1:1-2", b".", b"*"];
            let mut count = 0;
            let reglist = region_c_177_hts_reglist_create(
                Some(&args),
                Some(&mut count),
                None,
                Some(test_name2id),
            );
            let reglist = reglist.expect("reglist");
            assert_eq!(count, 4);

            let tids = reglist.iter().map(|entry| entry.tid).collect::<Vec<_>>();
            assert_eq!(tids, [0, 1, HTS_IDX_START, HTS_IDX_NOCOOR]);

            region_c_266_hts_reglist_free(Some(reglist));
        }
    }

    #[test]
    fn khash_order_matches_resize_kickout_for_colliding_tids() {
        let entries = [35, -5, 19, 39]
            .into_iter()
            .map(|tid| region_c_31_reglist {
                tid,
                intervals: vec![hts_pair_pos_t { beg: 0, end: 1 }],
            })
            .collect::<Vec<_>>();

        let tids = region_khash_int_order(&entries)
            .into_iter()
            .map(|idx| entries[idx].tid)
            .collect::<Vec<_>>();
        assert_eq!(tids, [-5, 35, 19, 39]);
    }
}
