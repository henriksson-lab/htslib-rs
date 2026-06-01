// Functions translated from htslib/cram/cram_stats.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_int, c_void};

use super::*;

pub unsafe fn cram_cram_stats_c_48_cram_stats_create() -> *mut c_void {
    calloc(1, std::mem::size_of::<cram_stats_layout>() as u64)
}

pub unsafe fn cram_cram_stats_c_52_cram_stats_add(st: *mut c_void, val: c_int) {
    let st = st.cast::<cram_stats_layout>();
    (*st).nsamp += 1;

    if val >= 0 && val < 1024 {
        (*st).freqs[val as usize] += 1;
        return;
    }

    if (*st).h.is_null() {
        let h = calloc(1, std::mem::size_of::<kh_m_i2i_layout>() as u64).cast::<kh_m_i2i_layout>();
        if h.is_null() {
            return;
        }
        let n_buckets = 4u32;
        let n_flags = if n_buckets < 16 { 1 } else { n_buckets >> 4 };
        (*h).flags = malloc(n_flags as u64 * std::mem::size_of::<u32>() as u64).cast::<u32>();
        (*h).keys = malloc(n_buckets as u64 * std::mem::size_of::<i64>() as u64).cast::<i64>();
        (*h).vals = malloc(n_buckets as u64 * std::mem::size_of::<c_int>() as u64).cast::<c_int>();
        if (*h).flags.is_null() || (*h).keys.is_null() || (*h).vals.is_null() {
            free((*h).flags.cast());
            free((*h).keys.cast());
            free((*h).vals.cast());
            free(h.cast());
            return;
        }
        for i in 0..n_flags {
            *(*h).flags.add(i as usize) = 0xaaaa_aaaa;
        }
        (*h).n_buckets = n_buckets;
        (*h).upper_bound = (n_buckets as f64 * 0.77) as u32;
        (*st).h = h.cast();
    }

    let mut h = (*st).h.cast::<kh_m_i2i_layout>();
    if (*h).n_buckets != 0 {
        let key = val as u64;
        let hash = ((key >> 33) ^ key ^ (key << 11)) as u32;
        let mask = (*h).n_buckets - 1;
        let mut k = hash & mask;
        let last = k;
        let mut step = 0u32;
        loop {
            let flag = *(*h).flags.add((k >> 4) as usize);
            if ((flag >> ((k & 0x0f) << 1)) & 2) != 0 {
                break;
            }
            if ((flag >> ((k & 0x0f) << 1)) & 1) == 0 && *(*h).keys.add(k as usize) == val as i64 {
                *(*h).vals.add(k as usize) += 1;
                return;
            }
            step += 1;
            k = (k + step) & mask;
            if k == last {
                break;
            }
        }
    }

    if (*h).n_occupied >= (*h).upper_bound {
        let old_h = h;
        let old_n = (*old_h).n_buckets;
        let new_n = if old_n == 0 { 4 } else { old_n << 1 };
        let new_flags_n = if new_n < 16 { 1 } else { new_n >> 4 };
        let new_h =
            calloc(1, std::mem::size_of::<kh_m_i2i_layout>() as u64).cast::<kh_m_i2i_layout>();
        if new_h.is_null() {
            return;
        }
        (*new_h).flags =
            malloc(new_flags_n as u64 * std::mem::size_of::<u32>() as u64).cast::<u32>();
        (*new_h).keys = malloc(new_n as u64 * std::mem::size_of::<i64>() as u64).cast::<i64>();
        (*new_h).vals = malloc(new_n as u64 * std::mem::size_of::<c_int>() as u64).cast::<c_int>();
        if (*new_h).flags.is_null() || (*new_h).keys.is_null() || (*new_h).vals.is_null() {
            free((*new_h).flags.cast());
            free((*new_h).keys.cast());
            free((*new_h).vals.cast());
            free(new_h.cast());
            return;
        }
        for i in 0..new_flags_n {
            *(*new_h).flags.add(i as usize) = 0xaaaa_aaaa;
        }
        (*new_h).n_buckets = new_n;
        (*new_h).upper_bound = (new_n as f64 * 0.77 + 0.5) as u32;

        for k in 0..old_n {
            let flag = *(*old_h).flags.add((k >> 4) as usize);
            if ((flag >> ((k & 0x0f) << 1)) & 3) != 0 {
                continue;
            }
            let key = *(*old_h).keys.add(k as usize);
            let hash_key = key as u64;
            let hash = ((hash_key >> 33) ^ hash_key ^ (hash_key << 11)) as u32;
            let mask = new_n - 1;
            let mut i = hash & mask;
            let mut step = 0u32;
            loop {
                let new_flag = *(*new_h).flags.add((i >> 4) as usize);
                if ((new_flag >> ((i & 0x0f) << 1)) & 2) != 0 {
                    break;
                }
                step += 1;
                i = (i + step) & mask;
            }
            *(*new_h).keys.add(i as usize) = key;
            *(*new_h).vals.add(i as usize) = *(*old_h).vals.add(k as usize);
            *(*new_h).flags.add((i >> 4) as usize) &= !(3 << ((i & 0x0f) << 1));
            (*new_h).size += 1;
            (*new_h).n_occupied += 1;
        }
        free((*old_h).flags.cast());
        free((*old_h).keys.cast());
        free((*old_h).vals.cast());
        free(old_h.cast());
        (*st).h = new_h.cast();
        h = new_h;
    }

    let key = val as u64;
    let hash = ((key >> 33) ^ key ^ (key << 11)) as u32;
    let mask = (*h).n_buckets - 1;
    let mut x = (*h).n_buckets;
    let mut site = (*h).n_buckets;
    let mut i = hash & mask;
    let flag = *(*h).flags.add((i >> 4) as usize);
    if ((flag >> ((i & 0x0f) << 1)) & 2) != 0 {
        x = i;
    } else {
        let last = i;
        let mut step = 0u32;
        while {
            let flag = *(*h).flags.add((i >> 4) as usize);
            ((flag >> ((i & 0x0f) << 1)) & 2) == 0
                && (((flag >> ((i & 0x0f) << 1)) & 1) != 0
                    || *(*h).keys.add(i as usize) != val as i64)
        } {
            let flag = *(*h).flags.add((i >> 4) as usize);
            if ((flag >> ((i & 0x0f) << 1)) & 1) != 0 {
                site = i;
            }
            step += 1;
            i = (i + step) & mask;
            if i == last {
                x = site;
                break;
            }
        }
        if x == (*h).n_buckets {
            let flag = *(*h).flags.add((i >> 4) as usize);
            if ((flag >> ((i & 0x0f) << 1)) & 2) != 0 && site != (*h).n_buckets {
                x = site;
            } else {
                x = i;
            }
        }
    }

    let flag = *(*h).flags.add((x >> 4) as usize);
    if ((flag >> ((x & 0x0f) << 1)) & 2) != 0 {
        *(*h).keys.add(x as usize) = val as i64;
        *(*h).vals.add(x as usize) = 1;
        *(*h).flags.add((x >> 4) as usize) &= !(3 << ((x & 0x0f) << 1));
        (*h).size += 1;
        (*h).n_occupied += 1;
    } else if ((flag >> ((x & 0x0f) << 1)) & 1) != 0 {
        *(*h).keys.add(x as usize) = val as i64;
        *(*h).vals.add(x as usize) = 1;
        *(*h).flags.add((x >> 4) as usize) &= !(3 << ((x & 0x0f) << 1));
        (*h).size += 1;
    }
}

pub unsafe fn cram_cram_stats_c_80_cram_stats_del(st: *mut c_void, val: c_int) {
    let st = st.cast::<cram_stats_layout>();
    (*st).nsamp -= 1;

    if val >= 0 && val < 1024 {
        (*st).freqs[val as usize] -= 1;
        debug_assert!((*st).freqs[val as usize] >= 0);
        return;
    }

    if !(*st).h.is_null() {
        let h = (*st).h.cast::<kh_m_i2i_layout>();
        if (*h).n_buckets != 0 {
            let key = val as u64;
            let hash = ((key >> 33) ^ key ^ (key << 11)) as u32;
            let mask = (*h).n_buckets - 1;
            let mut k = hash & mask;
            let last = k;
            let mut step = 0u32;
            loop {
                let flag = *(*h).flags.add((k >> 4) as usize);
                if ((flag >> ((k & 0x0f) << 1)) & 2) != 0 {
                    break;
                }
                if ((flag >> ((k & 0x0f) << 1)) & 1) == 0
                    && *(*h).keys.add(k as usize) == val as i64
                {
                    *(*h).vals.add(k as usize) -= 1;
                    if *(*h).vals.add(k as usize) == 0 {
                        *(*h).flags.add((k >> 4) as usize) |= 1 << ((k & 0x0f) << 1);
                        (*h).size -= 1;
                    }
                    return;
                }
                step += 1;
                k = (k + step) & mask;
                if k == last {
                    break;
                }
            }
        }
    }

    (*st).nsamp += 1;
}

pub unsafe fn cram_cram_stats_c_105_cram_stats_dump(st: *mut c_void) {
    let st = st.cast::<cram_stats_layout>();
    libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"cram_stats:\n".as_ptr());

    for i in 0..1024usize {
        let freq = (*st).freqs[i];
        if freq == 0 {
            continue;
        }
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"\t%d\t%d\n".as_ptr(),
            i as c_int,
            freq,
        );
    }

    if !(*st).h.is_null() {
        let h = (*st).h.cast::<kh_m_i2i_layout>();
        for k in 0..(*h).n_buckets {
            let flag = *(*h).flags.add((k >> 4) as usize);
            if ((flag >> ((k & 0x0f) << 1)) & 3) != 0 {
                continue;
            }
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"\t%lld\t%d\n".as_ptr(),
                *(*h).keys.add(k as usize) as libc::c_longlong,
                *(*h).vals.add(k as usize),
            );
        }
    }
}

pub unsafe fn cram_cram_stats_c_134_cram_stats_encoding(fd: *mut c_void, st: *mut c_void) -> c_int {
    let fd = fd.cast::<cram_fd_layout>();
    let st = st.cast::<cram_stats_layout>();
    let mut nvals = 0i32;
    let mut max_val = 0i32;
    let mut min_val = i32::MAX;
    let mut ntot = 0i32;

    for i in 0..1024usize {
        if (*st).freqs[i] == 0 {
            continue;
        }
        ntot += (*st).freqs[i];
        if max_val < i as i32 {
            max_val = i as i32;
        }
        if min_val > i as i32 {
            min_val = i as i32;
        }
        nvals += 1;
    }

    if !(*st).h.is_null() {
        let h = (*st).h.cast::<kh_m_i2i_layout>();
        for k in 0..(*h).n_buckets {
            let flag = *(*h).flags.add((k >> 4) as usize);
            if ((flag >> ((k & 0x0f) << 1)) & 3) != 0 {
                continue;
            }
            let i = *(*h).keys.add(k as usize) as i32;
            ntot += *(*h).vals.add(k as usize);
            if max_val < i {
                max_val = i;
            }
            if min_val > i {
                min_val = i;
            }
            nvals += 1;
        }
    }

    (*st).nvals = nvals;
    (*st).min_val = min_val as i64;
    (*st).max_val = max_val as i64;
    debug_assert_eq!(ntot, (*st).nsamp);

    if (*fd).version >> 8 >= 4 {
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

pub unsafe fn cram_cram_stats_c_223_cram_stats_free(st: *mut c_void) {
    if st.is_null() {
        return;
    }
    let st_layout = st.cast::<cram_stats_layout>();
    if !(*st_layout).h.is_null() {
        let h = (*st_layout).h.cast::<kh_m_i2i_layout>();
        free((*h).flags.cast());
        free((*h).keys.cast());
        free((*h).vals.cast());
        free(h.cast());
    }
    free(st);
}
