// Functions translated from htslib/cram/cram_stats.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_int, c_void};
use std::ptr::{self, NonNull};

use super::*;

const CRAM_STATS_SMALL_LIMIT: usize = 1024;
const KH_INITIAL_BUCKETS: u32 = 4;

unsafe fn cram_stats_ref<'a>(st: *const cram_stats_layout) -> Option<&'a cram_stats_layout> {
    NonNull::new(st.cast_mut()).map(|st| st.as_ref())
}

unsafe fn cram_stats_mut<'a>(st: *mut cram_stats_layout) -> Option<&'a mut cram_stats_layout> {
    NonNull::new(st).map(|mut st| st.as_mut())
}

unsafe fn cram_stats_box(st: *mut cram_stats_layout) -> Option<Box<cram_stats_layout>> {
    if st.is_null() {
        None
    } else {
        Some(Box::from_raw(st))
    }
}

unsafe fn cram_fd_ref<'a>(fd: *const cram_fd_layout) -> Option<&'a cram_fd_layout> {
    NonNull::new(fd.cast_mut()).map(|fd| fd.as_ref())
}

fn cram_stats_new() -> cram_stats_layout {
    cram_stats_layout {
        freqs: [0; 1024],
        h: ptr::null_mut(),
        nsamp: 0,
        nvals: 0,
        min_val: 0,
        max_val: 0,
    }
}

fn cram_stats_create() -> Box<cram_stats_layout> {
    Box::new(cram_stats_new())
}

pub unsafe fn cram_cram_stats_c_48_cram_stats_create() -> *mut c_void {
    Box::into_raw(cram_stats_create()).cast()
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

unsafe fn kh_flags(h: &kh_m_i2i_layout) -> &[u32] {
    std::slice::from_raw_parts(h.flags, kh_flags_len(h.n_buckets) as usize)
}

unsafe fn kh_flags_mut(h: &mut kh_m_i2i_layout) -> &mut [u32] {
    std::slice::from_raw_parts_mut(h.flags, kh_flags_len(h.n_buckets) as usize)
}

unsafe fn kh_keys(h: &kh_m_i2i_layout) -> &[i64] {
    std::slice::from_raw_parts(h.keys, h.n_buckets as usize)
}

unsafe fn kh_keys_mut(h: &mut kh_m_i2i_layout) -> &mut [i64] {
    std::slice::from_raw_parts_mut(h.keys, h.n_buckets as usize)
}

unsafe fn kh_vals(h: &kh_m_i2i_layout) -> &[c_int] {
    std::slice::from_raw_parts(h.vals, h.n_buckets as usize)
}

unsafe fn kh_vals_mut(h: &mut kh_m_i2i_layout) -> &mut [c_int] {
    std::slice::from_raw_parts_mut(h.vals, h.n_buckets as usize)
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

unsafe fn kh_alloc(n_buckets: u32) -> Option<NonNull<kh_m_i2i_layout>> {
    let flags_n = kh_flags_len(n_buckets);
    let flags = vec![0xaaaa_aaaau32; flags_n as usize].into_boxed_slice();
    let keys = vec![0i64; n_buckets as usize].into_boxed_slice();
    let vals = vec![0 as c_int; n_buckets as usize].into_boxed_slice();

    let upper_bound = if n_buckets == KH_INITIAL_BUCKETS {
        (n_buckets as f64 * 0.77) as u32
    } else {
        (n_buckets as f64 * 0.77 + 0.5) as u32
    };

    Some(NonNull::from(Box::leak(Box::new(kh_m_i2i_layout {
        n_buckets,
        size: 0,
        n_occupied: 0,
        upper_bound,
        flags: Box::leak(flags).as_mut_ptr(),
        keys: Box::leak(keys).as_mut_ptr(),
        vals: Box::leak(vals).as_mut_ptr(),
    }))))
}

unsafe fn kh_free(mut h: NonNull<kh_m_i2i_layout>) {
    let h_ref = h.as_mut();
    if !h_ref.flags.is_null() {
        drop(Box::from_raw(ptr::slice_from_raw_parts_mut(
            h_ref.flags,
            kh_flags_len(h_ref.n_buckets) as usize,
        )));
    }
    if !h_ref.keys.is_null() {
        drop(Box::from_raw(ptr::slice_from_raw_parts_mut(
            h_ref.keys,
            h_ref.n_buckets as usize,
        )));
    }
    if !h_ref.vals.is_null() {
        drop(Box::from_raw(ptr::slice_from_raw_parts_mut(
            h_ref.vals,
            h_ref.n_buckets as usize,
        )));
    }
    drop(Box::from_raw(h.as_ptr()));
}

fn stats_hash_mut(st: &mut cram_stats_layout) -> Option<NonNull<kh_m_i2i_layout>> {
    NonNull::new(st.h.cast::<kh_m_i2i_layout>())
}

fn stats_hash_ref(st: &cram_stats_layout) -> Option<NonNull<kh_m_i2i_layout>> {
    NonNull::new(st.h.cast::<kh_m_i2i_layout>())
}

unsafe fn stats_ensure_hash(st: &mut cram_stats_layout) -> Option<NonNull<kh_m_i2i_layout>> {
    if let Some(h) = stats_hash_mut(st) {
        return Some(h);
    }

    let h = kh_alloc(KH_INITIAL_BUCKETS)?;
    st.h = h.as_ptr().cast();
    Some(h)
}

unsafe fn kh_find_occupied(h: &kh_m_i2i_layout, key: i64) -> Option<u32> {
    if h.n_buckets == 0 {
        return None;
    }

    let flags = kh_flags(h);
    let keys = kh_keys(h);
    let mask = h.n_buckets - 1;
    let mut k = kh_hash(key) & mask;
    let last = k;
    let mut step = 0u32;

    loop {
        let flag = kh_flag_at(flags, k);
        if (flag & 2) != 0 {
            return None;
        }
        if (flag & 1) == 0 && keys[k as usize] == key {
            return Some(k);
        }
        step += 1;
        k = (k + step) & mask;
        if k == last {
            return None;
        }
    }
}

unsafe fn kh_insert_absent(h: &mut kh_m_i2i_layout, key: i64, val: c_int) {
    let mask = h.n_buckets - 1;
    let mut x = h.n_buckets;
    let mut site = h.n_buckets;
    let mut i = kh_hash(key) & mask;
    let flags = kh_flags(h);
    let keys = kh_keys(h);
    let flag = kh_flag_at(flags, i);

    if (flag & 2) != 0 {
        x = i;
    } else {
        let last = i;
        let mut step = 0u32;
        while {
            let flag = kh_flag_at(flags, i);
            (flag & 2) == 0 && ((flag & 1) != 0 || keys[i as usize] != key)
        } {
            let flag = kh_flag_at(flags, i);
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
            let flag = kh_flag_at(flags, i);
            if (flag & 2) != 0 && site != h.n_buckets {
                x = site;
            } else {
                x = i;
            }
        }
    }

    let flag = kh_flag_at(kh_flags(h), x);
    if (flag & 2) != 0 {
        kh_keys_mut(h)[x as usize] = key;
        kh_vals_mut(h)[x as usize] = val;
        kh_mark_occupied(kh_flags_mut(h), x);
        h.size += 1;
        h.n_occupied += 1;
    } else if (flag & 1) != 0 {
        kh_keys_mut(h)[x as usize] = key;
        kh_vals_mut(h)[x as usize] = val;
        kh_mark_occupied(kh_flags_mut(h), x);
        h.size += 1;
    }
}

unsafe fn kh_resize(h: NonNull<kh_m_i2i_layout>) -> Option<NonNull<kh_m_i2i_layout>> {
    let old_h = h.as_ref();
    let old_n = old_h.n_buckets;
    let new_n = if old_n == 0 {
        KH_INITIAL_BUCKETS
    } else {
        old_n << 1
    };
    let mut new_h = kh_alloc(new_n)?;

    for k in 0..old_n {
        if kh_flag_at(kh_flags(old_h), k) != 0 {
            continue;
        }
        let key = kh_keys(old_h)[k as usize];
        let val = kh_vals(old_h)[k as usize];
        kh_insert_absent(new_h.as_mut(), key, val);
    }

    kh_free(h);
    Some(new_h)
}

unsafe fn cram_stats_add(st: &mut cram_stats_layout, val: c_int) {
    st.nsamp += 1;

    if (0..CRAM_STATS_SMALL_LIMIT as c_int).contains(&val) {
        st.freqs[val as usize] += 1;
        return;
    }

    let Some(mut h) = stats_ensure_hash(st) else {
        return;
    };

    if let Some(k) = kh_find_occupied(h.as_ref(), val as i64) {
        kh_vals_mut(h.as_mut())[k as usize] += 1;
        return;
    }

    if h.as_ref().n_occupied >= h.as_ref().upper_bound {
        let Some(new_h) = kh_resize(h) else {
            return;
        };
        st.h = new_h.as_ptr().cast();
        h = new_h;
    }

    kh_insert_absent(h.as_mut(), val as i64, 1);
}

pub unsafe fn cram_cram_stats_c_52_cram_stats_add(st: *mut c_void, val: c_int) {
    let Some(st) = cram_stats_mut(st.cast()) else {
        return;
    };
    cram_stats_add(st, val);
}

unsafe fn cram_stats_del(st: &mut cram_stats_layout, val: c_int) {
    st.nsamp -= 1;

    if (0..CRAM_STATS_SMALL_LIMIT as c_int).contains(&val) {
        st.freqs[val as usize] -= 1;
        debug_assert!(st.freqs[val as usize] >= 0);
        return;
    }

    if let Some(mut h) = stats_hash_mut(st) {
        if let Some(k) = kh_find_occupied(h.as_ref(), val as i64) {
            let vals = kh_vals_mut(h.as_mut());
            vals[k as usize] -= 1;
            if vals[k as usize] == 0 {
                kh_mark_deleted(kh_flags_mut(h.as_mut()), k);
                h.as_mut().size -= 1;
            }
            return;
        }
    }

    st.nsamp += 1;
}

pub unsafe fn cram_cram_stats_c_80_cram_stats_del(st: *mut c_void, val: c_int) {
    let Some(st) = cram_stats_mut(st.cast()) else {
        return;
    };
    cram_stats_del(st, val);
}

unsafe fn kh_occupied_entries(h: &kh_m_i2i_layout, mut f: impl FnMut(i64, c_int)) {
    for k in 0..h.n_buckets {
        if kh_flag_at(kh_flags(h), k) != 0 {
            continue;
        }
        f(kh_keys(h)[k as usize], kh_vals(h)[k as usize]);
    }
}

unsafe fn cram_stats_dump(st: &cram_stats_layout) {
    eprintln!("cram_stats:");

    for (i, &freq) in st.freqs.iter().enumerate() {
        if freq == 0 {
            continue;
        }
        eprintln!("\t{}\t{}", i, freq);
    }

    if let Some(h) = stats_hash_ref(st) {
        kh_occupied_entries(h.as_ref(), |key, val| {
            eprintln!("\t{}\t{}", key, val);
        });
    }
}

pub unsafe fn cram_cram_stats_c_105_cram_stats_dump(st: *mut c_void) {
    let Some(st) = cram_stats_ref(st.cast()) else {
        return;
    };
    cram_stats_dump(st);
}

unsafe fn cram_stats_encoding(fd: &cram_fd_layout, st: &mut cram_stats_layout) -> c_int {
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

    if let Some(h) = stats_hash_ref(st) {
        kh_occupied_entries(h.as_ref(), |key, val| {
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

pub unsafe fn cram_cram_stats_c_134_cram_stats_encoding(fd: *mut c_void, st: *mut c_void) -> c_int {
    let Some(fd) = cram_fd_ref(fd.cast()) else {
        return 0;
    };
    let Some(st) = cram_stats_mut(st.cast()) else {
        return 0;
    };
    cram_stats_encoding(fd, st)
}

unsafe fn cram_stats_free(st: &mut cram_stats_layout) {
    if let Some(h) = stats_hash_mut(st) {
        kh_free(h);
        st.h = std::ptr::null_mut();
    }
}

unsafe fn cram_stats_destroy(mut st: Box<cram_stats_layout>) {
    cram_stats_free(&mut st);
}

pub unsafe fn cram_cram_stats_c_223_cram_stats_free(st: *mut c_void) {
    let Some(st) = cram_stats_box(st.cast()) else {
        return;
    };
    cram_stats_destroy(st);
}
