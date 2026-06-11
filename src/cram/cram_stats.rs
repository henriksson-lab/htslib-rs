// Functions translated from htslib/cram/cram_stats.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use super::*;

const CRAM_STATS_SMALL_LIMIT: usize = 1024;
const KH_INITIAL_BUCKETS: u32 = 4;

fn cram_stats_new() -> cram_stats_layout {
    cram_stats_layout {
        freqs: [0; 1024],
        h: None,
        nsamp: 0,
        nvals: 0,
        min_val: 0,
        max_val: 0,
    }
}

fn cram_stats_create() -> Box<cram_stats_layout> {
    Box::new(cram_stats_new())
}

pub fn cram_cram_stats_c_48_cram_stats_create() -> Box<cram_stats_layout> {
    cram_stats_create()
}

fn kh_flags_len(n_buckets: u32) -> u32 {
    if n_buckets < 16 {
        1
    } else {
        n_buckets >> 4
    }
}

fn kh_hash(key: i64) -> u32 {
    let key = key as u64;
    ((key >> 33) ^ key ^ (key << 11)) as u32
}

fn kh_flag_at(flags: &[u32], k: u32) -> u32 {
    (flags[(k >> 4) as usize] >> ((k & 0x0f) << 1)) & 3
}

fn kh_mark_occupied(flags: &mut [u32], k: u32) {
    flags[(k >> 4) as usize] &= !(3 << ((k & 0x0f) << 1));
}

fn kh_mark_deleted(flags: &mut [u32], k: u32) {
    flags[(k >> 4) as usize] |= 1 << ((k & 0x0f) << 1);
}

fn kh_alloc(n_buckets: u32) -> Box<kh_m_i2i_layout> {
    let flags_n = kh_flags_len(n_buckets);

    let upper_bound = if n_buckets == KH_INITIAL_BUCKETS {
        (n_buckets as f64 * 0.77) as u32
    } else {
        (n_buckets as f64 * 0.77 + 0.5) as u32
    };

    Box::new(kh_m_i2i_layout {
        n_buckets,
        size: 0,
        n_occupied: 0,
        upper_bound,
        flags: vec![0xaaaa_aaaau32; flags_n as usize],
        keys: vec![0i64; n_buckets as usize],
        vals: vec![0i32; n_buckets as usize],
    })
}

fn stats_ensure_hash(st: &mut cram_stats_layout) -> &mut kh_m_i2i_layout {
    if st.h.is_none() {
        st.h = Some(kh_alloc(KH_INITIAL_BUCKETS));
    }
    st.h.as_mut().unwrap()
}

fn kh_find_occupied(h: &kh_m_i2i_layout, key: i64) -> Option<u32> {
    if h.n_buckets == 0 {
        return None;
    }

    let mask = h.n_buckets - 1;
    let mut k = kh_hash(key) & mask;
    let last = k;
    let mut step = 0u32;

    loop {
        let flag = kh_flag_at(&h.flags, k);
        if (flag & 2) != 0 {
            return None;
        }
        if (flag & 1) == 0 && h.keys[k as usize] == key {
            return Some(k);
        }
        step += 1;
        k = (k + step) & mask;
        if k == last {
            return None;
        }
    }
}

fn kh_insert_absent(h: &mut kh_m_i2i_layout, key: i64, val: i32) {
    let mask = h.n_buckets - 1;
    let mut x = h.n_buckets;
    let mut site = h.n_buckets;
    let mut i = kh_hash(key) & mask;
    let flag = kh_flag_at(&h.flags, i);

    if (flag & 2) != 0 {
        x = i;
    } else {
        let last = i;
        let mut step = 0u32;
        while {
            let flag = kh_flag_at(&h.flags, i);
            (flag & 2) == 0 && ((flag & 1) != 0 || h.keys[i as usize] != key)
        } {
            let flag = kh_flag_at(&h.flags, i);
            if (flag & 1) != 0 {
                site = i;
            }
            step += 1;
            i = (i + step) & mask;
            if i == last {
                x = site;
                break;
            }
        }
        if x == h.n_buckets {
            let flag = kh_flag_at(&h.flags, i);
            if (flag & 2) != 0 && site != h.n_buckets {
                x = site;
            } else {
                x = i;
            }
        }
    }

    let flag = kh_flag_at(&h.flags, x);
    if (flag & 2) != 0 {
        h.keys[x as usize] = key;
        h.vals[x as usize] = val;
        kh_mark_occupied(&mut h.flags, x);
        h.size += 1;
        h.n_occupied += 1;
    } else if (flag & 1) != 0 {
        h.keys[x as usize] = key;
        h.vals[x as usize] = val;
        kh_mark_occupied(&mut h.flags, x);
        h.size += 1;
    }
}

fn kh_resize(old_h: &kh_m_i2i_layout) -> Box<kh_m_i2i_layout> {
    let old_n = old_h.n_buckets;
    let new_n = if old_n == 0 {
        KH_INITIAL_BUCKETS
    } else {
        old_n << 1
    };
    let mut new_h = kh_alloc(new_n);

    for k in 0..old_n {
        if kh_flag_at(&old_h.flags, k) != 0 {
            continue;
        }
        let key = old_h.keys[k as usize];
        let val = old_h.vals[k as usize];
        kh_insert_absent(&mut new_h, key, val);
    }

    new_h
}

fn cram_stats_add(st: &mut cram_stats_layout, val: i32) {
    st.nsamp += 1;

    if (0..CRAM_STATS_SMALL_LIMIT as i32).contains(&val) {
        st.freqs[val as usize] += 1;
        return;
    }

    let h = stats_ensure_hash(st);

    if let Some(k) = kh_find_occupied(h, val as i64) {
        h.vals[k as usize] += 1;
        return;
    }

    if h.n_occupied >= h.upper_bound {
        let resized = kh_resize(h);
        st.h = Some(resized);
    }

    kh_insert_absent(st.h.as_mut().unwrap(), val as i64, 1);
}

pub fn cram_cram_stats_c_52_cram_stats_add(st: &mut cram_stats_layout, val: i32) {
    cram_stats_add(st, val);
}

fn cram_stats_del(st: &mut cram_stats_layout, val: i32) {
    st.nsamp -= 1;

    if (0..CRAM_STATS_SMALL_LIMIT as i32).contains(&val) {
        st.freqs[val as usize] -= 1;
        debug_assert!(st.freqs[val as usize] >= 0);
        return;
    }

    if let Some(h) = st.h.as_mut() {
        if let Some(k) = kh_find_occupied(h, val as i64) {
            h.vals[k as usize] -= 1;
            if h.vals[k as usize] == 0 {
                kh_mark_deleted(&mut h.flags, k);
                h.size -= 1;
            }
            return;
        }
    }

    st.nsamp += 1;
}

pub fn cram_cram_stats_c_80_cram_stats_del(st: &mut cram_stats_layout, val: i32) {
    cram_stats_del(st, val);
}

fn kh_occupied_entries(h: &kh_m_i2i_layout, mut f: impl FnMut(i64, i32)) {
    for k in 0..h.n_buckets {
        if kh_flag_at(&h.flags, k) != 0 {
            continue;
        }
        f(h.keys[k as usize], h.vals[k as usize]);
    }
}

fn cram_stats_dump(st: &cram_stats_layout) {
    eprintln!("cram_stats:");

    for (i, &freq) in st.freqs.iter().enumerate() {
        if freq == 0 {
            continue;
        }
        eprintln!("\t{}\t{}", i, freq);
    }

    if let Some(h) = st.h.as_ref() {
        kh_occupied_entries(h, |key, val| {
            eprintln!("\t{}\t{}", key, val);
        });
    }
}

pub fn cram_cram_stats_c_105_cram_stats_dump(st: &cram_stats_layout) {
    cram_stats_dump(st);
}

fn cram_stats_encoding(fd: &cram_fd_layout, st: &mut cram_stats_layout) -> i32 {
    let mut nvals = 0i32;
    let mut max_val = 0i32;
    let mut min_val = i32::MAX;
    let mut ntot = 0i32;

    for (i, &freq) in st.freqs.iter().enumerate() {
        if freq == 0 {
            continue;
        }
        ntot += freq;
        max_val = max_val.max(i as i32);
        min_val = min_val.min(i as i32);
        nvals += 1;
    }

    if let Some(h) = st.h.as_ref() {
        kh_occupied_entries(h, |key, val| {
            let i = key as i32;
            ntot += val;
            max_val = max_val.max(i);
            min_val = min_val.min(i);
            nvals += 1;
        });
    }

    st.nvals = nvals;
    st.min_val = min_val as i64;
    st.max_val = max_val as i64;
    debug_assert_eq!(ntot, st.nsamp);

    if fd.version >> 8 >= 4 {
        if nvals == 1 {
            44
        } else if nvals == 0 || min_val < 0 {
            42
        } else {
            41
        }
    } else if nvals <= 1 {
        3
    } else {
        1
    }
}

pub fn cram_cram_stats_c_134_cram_stats_encoding(
    fd: &cram_fd_layout,
    st: &mut cram_stats_layout,
) -> i32 {
    cram_stats_encoding(fd, st)
}

fn cram_stats_free(st: &mut cram_stats_layout) {
    st.h = None;
}

fn cram_stats_destroy(mut st: Box<cram_stats_layout>) {
    cram_stats_free(&mut st);
}

pub fn cram_cram_stats_c_223_cram_stats_free(st: Box<cram_stats_layout>) {
    cram_stats_destroy(st);
}
