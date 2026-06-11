type khint_t = u32;

const MAX_ENTRIES: usize = 99_999_999;

#[repr(C)]
pub struct kh_str2int_t {
    n_buckets: khint_t,
    size: khint_t,
    n_occupied: khint_t,
    upper_bound: khint_t,
    flags: *mut khint_t,
    keys: *mut *mut u8,
    vals: *mut i32,
}

unsafe fn kh_isempty(flags: *const khint_t, i: khint_t) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) != 0
}

unsafe fn kh_isdel(flags: *const khint_t, i: khint_t) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 1) != 0
}

unsafe fn kh_iseither(flags: *const khint_t, i: khint_t) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 3) != 0
}

unsafe fn kh_set_isboth_false(flags: *mut khint_t, i: khint_t) {
    *flags.add((i >> 4) as usize) &= !(3 << ((i & 0x0f) << 1));
}

unsafe fn kh_set_isdel_true(flags: *mut khint_t, i: khint_t) {
    *flags.add((i >> 4) as usize) =
        (*flags.add((i >> 4) as usize) & !(2 << ((i & 0x0f) << 1))) | (1 << ((i & 0x0f) << 1));
}

unsafe fn kh_str_hash_func(key: *const u8) -> khint_t {
    crate::htslib_rs::hts::__ac_FNV1a_hash_string(key.cast())
}

unsafe fn kh_str_hash_equal(a: *const u8, b: *const u8) -> bool {
    !a.is_null() && !b.is_null() && libc::strcmp(a.cast(), b.cast()) == 0
}

unsafe fn kh_init_str2int() -> *mut kh_str2int_t {
    Box::into_raw(Box::new(kh_str2int_t {
        n_buckets: 0,
        size: 0,
        n_occupied: 0,
        upper_bound: 0,
        flags: std::ptr::null_mut(),
        keys: std::ptr::null_mut(),
        vals: std::ptr::null_mut(),
    }))
}

unsafe fn kh_destroy_str2int(h: *mut kh_str2int_t) {
    if h.is_null() {
        return;
    }
    drop(Vec::from_raw_parts((*h).flags, 0, 0));
    drop(Vec::from_raw_parts((*h).keys, 0, 0));
    drop(Vec::from_raw_parts((*h).vals, 0, 0));
    drop(Box::from_raw(h));
}

unsafe fn kh_resize_str2int(h: *mut kh_str2int_t, mut new_n_buckets: khint_t) -> i32 {
    if new_n_buckets < 4 {
        new_n_buckets = 4;
    }
    new_n_buckets = kroundup_size_t(new_n_buckets as usize) as khint_t;
    if new_n_buckets < (*h).size {
        return 0;
    }

    let n_flags = if new_n_buckets < 16 {
        1
    } else {
        new_n_buckets >> 4
    };
    let mut new_flags_vec: Vec<khint_t> = vec![0xaaaa_aaaa; n_flags as usize];
    let new_flags = new_flags_vec.as_mut_ptr();
    std::mem::forget(new_flags_vec);

    if (*h).n_buckets < new_n_buckets {
        // On the first resize the table is empty (n_buckets == 0) and keys/vals
        // are NULL: C's realloc(NULL, ..) allocates fresh, but Vec::from_raw_parts
        // requires a non-null pointer, so allocate new Vecs in that case.
        if (*h).keys.is_null() {
            let mut keys_vec: Vec<*mut u8> = vec![std::ptr::null_mut(); new_n_buckets as usize];
            (*h).keys = keys_vec.as_mut_ptr();
            std::mem::forget(keys_vec);
            let mut vals_vec: Vec<i32> = vec![0; new_n_buckets as usize];
            (*h).vals = vals_vec.as_mut_ptr();
            std::mem::forget(vals_vec);
        } else {
            let mut keys_vec = Vec::from_raw_parts((*h).keys, (*h).n_buckets as usize, (*h).n_buckets as usize);
            keys_vec.resize(new_n_buckets as usize, std::ptr::null_mut());
            (*h).keys = keys_vec.as_mut_ptr();
            std::mem::forget(keys_vec);
            let mut vals_vec = Vec::from_raw_parts((*h).vals, (*h).n_buckets as usize, (*h).n_buckets as usize);
            vals_vec.resize(new_n_buckets as usize, 0);
            (*h).vals = vals_vec.as_mut_ptr();
            std::mem::forget(vals_vec);
        }
    }

    let old_n_buckets = (*h).n_buckets;
    let old_flags = (*h).flags;
    let mut j = 0;
    while j != old_n_buckets {
        if !kh_iseither(old_flags, j) {
            let mut key = *(*h).keys.add(j as usize);
            let mut val = *(*h).vals.add(j as usize);
            kh_set_isdel_true(old_flags, j);
            loop {
                let mut new_mask = new_n_buckets - 1;
                let mut k = kh_str_hash_func(key);
                let mut i = k & new_mask;
                let mut step = 0;
                while !kh_isempty(new_flags, i) {
                    step += 1;
                    i = (i + step) & new_mask;
                }
                kh_set_isboth_false(new_flags, i);
                if i < old_n_buckets && !kh_iseither(old_flags, i) {
                    std::ptr::swap((*h).keys.add(i as usize), &mut key);
                    std::ptr::swap((*h).vals.add(i as usize), &mut val);
                    kh_set_isdel_true(old_flags, i);
                } else {
                    *(*h).keys.add(i as usize) = key;
                    *(*h).vals.add(i as usize) = val;
                    break;
                }
                new_mask = new_n_buckets - 1;
                k &= new_mask;
            }
        }
        j += 1;
    }

    if (*h).n_buckets > new_n_buckets {
        let mut keys_vec = Vec::from_raw_parts((*h).keys, (*h).n_buckets as usize, (*h).n_buckets as usize);
        keys_vec.truncate(new_n_buckets as usize);
        keys_vec.shrink_to_fit();
        (*h).keys = keys_vec.as_mut_ptr();
        std::mem::forget(keys_vec);
        let mut vals_vec = Vec::from_raw_parts((*h).vals, (*h).n_buckets as usize, (*h).n_buckets as usize);
        vals_vec.truncate(new_n_buckets as usize);
        vals_vec.shrink_to_fit();
        (*h).vals = vals_vec.as_mut_ptr();
        std::mem::forget(vals_vec);
    }

    // On the first resize the old flags pointer is NULL; only reclaim a real one.
    if !(*h).flags.is_null() {
        drop(Vec::from_raw_parts((*h).flags, 0, 0));
    }
    (*h).flags = new_flags;
    (*h).n_buckets = new_n_buckets;
    (*h).n_occupied = (*h).size;
    (*h).upper_bound = (new_n_buckets as f64 * 0.77 + 0.5) as khint_t;
    if (*h).upper_bound == new_n_buckets {
        (*h).upper_bound -= 1;
    }
    0
}

unsafe fn kh_put_str2int(h: *mut kh_str2int_t, key: *mut u8, ret: *mut i32) -> khint_t {
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > (*h).size << 1 {
            if kh_resize_str2int(h, (*h).n_buckets - 1) < 0 {
                *ret = -1;
                return (*h).n_buckets;
            }
        } else if kh_resize_str2int(h, (*h).n_buckets + 1) < 0 {
            *ret = -1;
            return (*h).n_buckets;
        }
    }

    let mask = (*h).n_buckets - 1;
    let mut i = kh_str_hash_func(key) & mask;
    let mut site = (*h).n_buckets;
    let x = if kh_isempty((*h).flags, i) {
        i
    } else {
        let last = i;
        let mut found = (*h).n_buckets;
        let mut step = 0;
        while !kh_isempty((*h).flags, i)
            && (kh_isdel((*h).flags, i) || !kh_str_hash_equal(*(*h).keys.add(i as usize), key))
        {
            if kh_isdel((*h).flags, i) {
                site = i;
            }
            step += 1;
            i = (i + step) & mask;
            if i == last {
                found = site;
                break;
            }
        }
        if found == (*h).n_buckets {
            if kh_isempty((*h).flags, i) && site != (*h).n_buckets {
                site
            } else {
                i
            }
        } else {
            found
        }
    };

    if kh_isempty((*h).flags, x) {
        *(*h).keys.add(x as usize) = key;
        kh_set_isboth_false((*h).flags, x);
        (*h).size += 1;
        (*h).n_occupied += 1;
        *ret = 1;
    } else if kh_isdel((*h).flags, x) {
        *(*h).keys.add(x as usize) = key;
        kh_set_isboth_false((*h).flags, x);
        (*h).size += 1;
        *ret = 2;
    } else {
        *ret = 0;
    }
    x
}

unsafe fn kh_get_str2int(h: *const kh_str2int_t, key: *const u8) -> khint_t {
    if (*h).n_buckets != 0 {
        let mask = (*h).n_buckets - 1;
        let k = kh_str_hash_func(key);
        let mut i = k & mask;
        let last = i;
        let mut step = 0;
        while !kh_isempty((*h).flags, i)
            && (kh_isdel((*h).flags, i) || !kh_str_hash_equal(*(*h).keys.add(i as usize), key))
        {
            step += 1;
            i = (i + step) & mask;
            if i == last {
                return (*h).n_buckets;
            }
        }
        return if kh_iseither((*h).flags, i) {
            (*h).n_buckets
        } else {
            i
        };
    }
    0
}

unsafe fn kh_del_str2int(h: *mut kh_str2int_t, x: khint_t) {
    if x != (*h).n_buckets && !kh_iseither((*h).flags, x) {
        kh_set_isdel_true((*h).flags, x);
        (*h).size -= 1;
    }
}

unsafe fn kh_stats_str2int(
    h: *mut kh_str2int_t,
    empty: *mut khint_t,
    deleted: *mut khint_t,
    hist_size: *mut khint_t,
    hist_out: *mut *mut khint_t,
) -> i32 {
    let mut hist_vec: Vec<khint_t> = vec![0; 1];
    let mut hist = hist_vec.as_mut_ptr();
    std::mem::forget(hist_vec);
    let mut dist_max = 0;
    let mask = (*h).n_buckets - 1;
    *empty = 0;
    *deleted = 0;
    *hist_size = 0;
    let mut i = 0;
    while i < (*h).n_buckets {
        if kh_isempty((*h).flags, i) {
            *empty += 1;
            i += 1;
            continue;
        }
        if kh_isdel((*h).flags, i) {
            *deleted += 1;
            i += 1;
            continue;
        }
        let mut k = kh_str_hash_func(*(*h).keys.add(i as usize)) & ((*h).n_buckets - 1);
        let mut dist = 0;
        let mut step = 0;
        while k != i {
            dist += 1;
            step += 1;
            k = (k + step) & mask;
        }
        if dist_max <= dist {
            let old_len = dist_max as usize + 1;
            let mut hist_vec = Vec::from_raw_parts(hist, old_len, old_len);
            hist_vec.resize(dist as usize + 1, 0);
            hist = hist_vec.as_mut_ptr();
            std::mem::forget(hist_vec);
            dist_max = dist;
        }
        *hist.add(dist as usize) += 1;
        i += 1;
    }
    *hist_out = hist;
    *hist_size = dist_max + 1;
    0
}

fn kroundup_size_t(mut x: usize) -> usize {
    x = x.wrapping_sub(1);
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    if std::mem::size_of::<usize>() >= 8 {
        x |= x >> 32;
    }
    x.wrapping_add(1)
}

// original: write_stats_str2int (htslib/test/test_khash.c:50)
pub unsafe fn test_test_khash_c_50_write_stats_str2int(h: *mut kh_str2int_t) {
    let mut empty = 0;
    let mut deleted = 0;
    let mut hist_size = 0;
    let mut hist: *mut khint_t = std::ptr::null_mut();

    if kh_stats_str2int(h, &mut empty, &mut deleted, &mut hist_size, &mut hist) == 0 {
        println!("n_buckets = {}", (*h).n_buckets);
        println!("empty     = {}", empty);
        println!("deleted   = {}", deleted);
        let mut i = 0;
        while i < hist_size {
            println!("dist[ {:8} ] = {}", i, *hist.add(i as usize));
            i += 1;
        }
        drop(Vec::from_raw_parts(hist, hist_size as usize, hist_size as usize));
    }
}

// original: make_keys (htslib/test/test_khash.c:66)
pub unsafe fn test_test_khash_c_66_make_keys(num: usize, kl: usize) -> *mut u8 {
    if num > MAX_ENTRIES {
        return std::ptr::null_mut();
    }
    let mut keys_vec: Vec<u8> = vec![0; kl * num];
    let keys = keys_vec.as_mut_ptr();
    std::mem::forget(keys_vec);
    let mut i = 0;
    while i < num {
        let s = format!("test{}", i);
        if s.len() + 1 > kl {
            drop(Vec::from_raw_parts(keys, kl * num, kl * num));
            return std::ptr::null_mut();
        }
        let dst = std::slice::from_raw_parts_mut(keys.add(kl * i), s.len() + 1);
        dst[..s.len()].copy_from_slice(s.as_bytes());
        dst[s.len()] = 0;
        i += 1;
    }

    keys
}

// original: add_str2int_entry (htslib/test/test_khash.c:86)
pub unsafe fn test_test_khash_c_86_add_str2int_entry(
    h: *mut kh_str2int_t,
    key: *mut u8,
    val: khint_t,
) -> i32 {
    let mut ret = 0;
    let k = kh_put_str2int(h, key, &mut ret);

    if ret != 1 && ret != 2 {
        eprintln!(
            "Unexpected return from kh_put({}) : {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(key.cast()).to_bytes()),
            ret,
        );
        return -1;
    }
    *(*h).vals.add(k as usize) = val as i32;
    0
}

// original: check_str2int_entry (htslib/test/test_khash.c:98)
pub unsafe fn test_test_khash_c_98_check_str2int_entry(
    h: *mut kh_str2int_t,
    key: *mut u8,
    val: khint_t,
    is_deleted: u8,
) -> i32 {
    let k = kh_get_str2int(h, key);
    if is_deleted != 0 {
        if k >= (*h).n_buckets {
            return 0;
        }
        eprintln!(
            "Found deleted entry {} in hash table",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(key.cast()).to_bytes()),
        );
        return -1;
    }

    if k >= (*h).n_buckets {
        eprintln!(
            "Couldn't find {} in hash table",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(key.cast()).to_bytes()),
        );
        return -1;
    }
    if libc::strcmp((*(*h).keys.add(k as usize)).cast(), key.cast()) != 0 {
        eprintln!(
            "Wrong key in hash table, expected {} got {}",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(key.cast()).to_bytes()),
            String::from_utf8_lossy(std::ffi::CStr::from_ptr((*(*h).keys.add(k as usize)).cast()).to_bytes()),
        );
        return -1;
    }
    if *(*h).vals.add(k as usize) != val as i32 {
        eprintln!(
            "Wrong value in hash table, expected {} got {}",
            val,
            *(*h).vals.add(k as usize),
        );
        return -1;
    }
    0
}

// original: del_str2int_entry (htslib/test/test_khash.c:127)
pub unsafe fn test_test_khash_c_127_del_str2int_entry(
    h: *mut kh_str2int_t,
    key: *mut u8,
) -> i32 {
    let k = kh_get_str2int(h, key);
    if k >= (*h).n_buckets {
        eprintln!(
            "Couldn't find {} to delete from hash table",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(key.cast()).to_bytes()),
        );
        return -1;
    }
    kh_del_str2int(h, k);
    0
}

// original: test_str2int (htslib/test/test_khash.c:137)
pub unsafe fn test_test_khash_c_137_test_str2int(
    max: usize,
    to_del: usize,
    show_stats: i32,
) -> i32 {
    let kl = 16;
    let mut mask = max;
    let keys = test_test_khash_c_66_make_keys(max, kl);
    let mut flags: *mut u8 = std::ptr::null_mut();
    let mut r = 0x533d_u32;

    if keys.is_null() {
        return -1;
    }

    let h = kh_init_str2int();
    if h.is_null() {
        drop(Vec::from_raw_parts(keys, max * kl, max * kl));
        return -1;
    }

    // Add some entries
    let mut i = 0;
    while i < max {
        if test_test_khash_c_86_add_str2int_entry(h, keys.add(i * kl), i as khint_t) != 0 {
            kh_destroy_str2int(h);
            drop(Vec::from_raw_parts(keys, max * kl, max * kl));
            if !flags.is_null() {
                drop(Vec::from_raw_parts(flags, max, max));
            }
            return -1;
        }
        i += 1;
    }

    // Check they exist
    i = 0;
    while i < max {
        if test_test_khash_c_98_check_str2int_entry(h, keys.add(i * kl), i as khint_t, 0) != 0 {
            kh_destroy_str2int(h);
            drop(Vec::from_raw_parts(keys, max * kl, max * kl));
            if !flags.is_null() {
                drop(Vec::from_raw_parts(flags, max, max));
            }
            return -1;
        }
        i += 1;
    }

    if show_stats != 0 {
        println!("Initial fill:");
        test_test_khash_c_50_write_stats_str2int(h);
    }

    // Delete a random selection
    let mut flags_vec: Vec<u8> = vec![0; max];
    flags = flags_vec.as_mut_ptr();
    std::mem::forget(flags_vec);

    mask = kroundup_size_t(mask);
    mask -= 1;

    // Note that this method may become slow for a high %age removed
    // as it searches for the last available entries.  Despite this, it
    // seems to be acceptable for the number of entries allowed.
    i = 0;
    while i < to_del {
        let mut victim;
        // LFSR, see http://users.ece.cmu.edu/~koopman/lfsr/index.html
        loop {
            r = (r >> 1) ^ ((r & 1).wrapping_mul(0x80000057));
            victim = ((r as usize) & mask).wrapping_sub(1);
            if victim < max && *flags.add(victim) == 0 {
                break;
            }
        }
        if test_test_khash_c_127_del_str2int_entry(h, keys.add(victim * kl)) != 0 {
            kh_destroy_str2int(h);
            drop(Vec::from_raw_parts(keys, max * kl, max * kl));
            if !flags.is_null() {
                drop(Vec::from_raw_parts(flags, max, max));
            }
            return -1;
        }
        *flags.add(victim) = 1;
        i += 1;
    }

    // Check correct entries are present
    i = 0;
    while i < max {
        if test_test_khash_c_98_check_str2int_entry(
            h,
            keys.add(i * kl),
            i as khint_t,
            *flags.add(i),
        ) != 0
        {
            kh_destroy_str2int(h);
            drop(Vec::from_raw_parts(keys, max * kl, max * kl));
            if !flags.is_null() {
                drop(Vec::from_raw_parts(flags, max, max));
            }
            return -1;
        }
        i += 1;
    }

    if show_stats != 0 {
        println!("\nAfter deletion:");
        test_test_khash_c_50_write_stats_str2int(h);
    }

    // Re-insert deleted entries
    i = 0;
    while i < max {
        if *flags.add(i) != 0
            && test_test_khash_c_86_add_str2int_entry(h, keys.add(i * kl), i as khint_t) != 0
        {
            kh_destroy_str2int(h);
            drop(Vec::from_raw_parts(keys, max * kl, max * kl));
            if !flags.is_null() {
                drop(Vec::from_raw_parts(flags, max, max));
            }
            return -1;
        }
        i += 1;
    }

    // Ensure they're all back
    i = 0;
    while i < max {
        if test_test_khash_c_98_check_str2int_entry(h, keys.add(i * kl), i as khint_t, 0) != 0 {
            kh_destroy_str2int(h);
            drop(Vec::from_raw_parts(keys, max * kl, max * kl));
            if !flags.is_null() {
                drop(Vec::from_raw_parts(flags, max, max));
            }
            return -1;
        }
        i += 1;
    }

    if show_stats != 0 {
        println!("\nAfter re-insert:");
        test_test_khash_c_50_write_stats_str2int(h);
    }

    kh_destroy_str2int(h);
    drop(Vec::from_raw_parts(keys, max * kl, max * kl));
    drop(Vec::from_raw_parts(flags, max, max));

    0
}

// original: read_keys (htslib/test/test_khash.c:236)
pub unsafe fn test_test_khash_c_236_read_keys(
    keys_file: *const u8,
    keys_out: *mut *mut u8,
    key_locations_out: *mut *mut *mut u8,
) -> usize {
    let mut in_ = libc::fopen(keys_file.cast(), c"r".as_ptr());
    let mut keys_size = 1_000_000usize;
    let mut keys_used = 0usize;
    let mut nkeys = 0usize;
    let mut fileinfo: libc::stat = std::mem::zeroed();

    if in_.is_null() {
        return 0;
    }

    // Slurp entire file
    if libc::fstat(libc::fileno(in_), &mut fileinfo) < 0 && fileinfo.st_size as usize > keys_size {
        keys_size = fileinfo.st_size as usize;
    }

    let mut keys_vec: Vec<u8> = vec![0; keys_size + 1];
    let mut keys = keys_vec.as_mut_ptr();
    std::mem::forget(keys_vec);

    loop {
        let mut avail = keys_size - keys_used;
        if avail == 0 {
            let new_size = keys_size + 1_000_000;
            let mut grown = Vec::from_raw_parts(keys, keys_size + 1, keys_size + 1);
            grown.resize(new_size + 1, 0);
            keys = grown.as_mut_ptr();
            std::mem::forget(grown);
            keys_size = new_size;
            avail = keys_size - keys_used;
        }
        let got = libc::fread(keys.add(keys_used).cast(), 1, avail, in_);
        keys_used += got;
        if got != avail {
            break;
        }
    }
    *keys.add(keys_used) = 0;

    if libc::ferror(in_) != 0 {
        libc::fclose(in_);
        drop(Vec::from_raw_parts(keys, keys_size + 1, keys_size + 1));
        *keys_out = std::ptr::null_mut();
        *key_locations_out = std::ptr::null_mut();
        return 0;
    }
    if libc::fclose(in_) < 0 {
        drop(Vec::from_raw_parts(keys, keys_size + 1, keys_size + 1));
        *keys_out = std::ptr::null_mut();
        *key_locations_out = std::ptr::null_mut();
        return 0;
    }
    in_ = std::ptr::null_mut();

    // Split by line
    let end = keys.add(keys_used);
    let mut key = keys;
    while !key.is_null() {
        while *key == b'\n' {
            key = key.add(1);
        }
        if key < end {
            nkeys += 1;
        }
        key = libc::memchr(key.cast(), b'\n' as i32, end.offset_from(key) as usize).cast();
    }

    let mut key_locations_vec: Vec<*mut u8> = vec![std::ptr::null_mut(); nkeys];
    let key_locations = key_locations_vec.as_mut_ptr();
    std::mem::forget(key_locations_vec);

    nkeys = 0;
    key = keys;
    while !key.is_null() {
        while *key == b'\n' {
            *key = 0;
            key = key.add(1);
        }
        if key < end {
            *key_locations.add(nkeys) = key;
            nkeys += 1;
        }
        key = libc::memchr(key.cast(), b'\n' as i32, end.offset_from(key) as usize).cast();
    }
    *keys_out = keys;
    *key_locations_out = key_locations;
    nkeys
}

// original: get_time (htslib/test/test_khash.c:312)
pub unsafe fn test_test_khash_c_312_get_time() -> i64 {
    let mut tv: libc::timeval = std::mem::zeroed();
    if libc::gettimeofday(&mut tv, std::ptr::null_mut()) < 0 {
        eprintln!("gettimeofday: {}", std::io::Error::last_os_error());
        return -1;
    }
    tv.tv_sec as i64 * 1_000_000 + tv.tv_usec as i64
}

// original: fmt_time (htslib/test/test_khash.c:330)
pub unsafe fn test_test_khash_c_330_fmt_time(elapsed: i64) -> *mut u8 {
    static mut BUF: [u8; 64] = [0; 64];
    let sec = elapsed / 1_000_000;
    let usec = elapsed % 1_000_000;
    let s = format!("{}.{:06} wall-time seconds", sec, usec);
    let n = s.len().min(63);
    let buf = std::ptr::addr_of_mut!(BUF).cast::<u8>();
    std::slice::from_raw_parts_mut(buf, n).copy_from_slice(&s.as_bytes()[..n]);
    *buf.add(n) = 0;
    buf
}

// original: benchmark (htslib/test/test_khash.c:344)
pub unsafe fn test_test_khash_c_344_benchmark(keys_file: *const u8) -> i32 {
    let kl = 16;
    let mut max = 50_000_000usize;
    let mut keys: *mut u8 = std::ptr::null_mut();
    let mut key_locations: *mut *mut u8 = std::ptr::null_mut();

    if !keys_file.is_null() {
        max = test_test_khash_c_236_read_keys(keys_file, &mut keys, &mut key_locations);
    } else {
        keys = test_test_khash_c_66_make_keys(max, kl);
    }

    if keys.is_null() {
        return -1;
    }

    let h = kh_init_str2int();
    if h.is_null() {
        drop(Vec::from_raw_parts(keys, max * kl, max * kl));
        return -1;
    }

    let mut start = test_test_khash_c_312_get_time();
    if start < 0 {
        kh_destroy_str2int(h);
        drop(Vec::from_raw_parts(keys, max * kl, max * kl));
        return -1;
    }

    let mut i = 0usize;
    if !keys_file.is_null() {
        while i < max {
            let mut ret = 0;
            let k = kh_put_str2int(h, *key_locations.add(i), &mut ret);
            if ret < 0 {
                eprintln!(
                    "Unexpected return from kh_put({}) : {}",
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr((*key_locations.add(i)).cast()).to_bytes()),
                    ret,
                );
                kh_destroy_str2int(h);
                drop(Vec::from_raw_parts(keys, max * kl, max * kl));
                return -1;
            }
            *(*h).vals.add(k as usize) = i as i32;
            i += 1;
        }
    } else {
        while i < max {
            let mut ret = 0;
            let k = kh_put_str2int(h, keys.add(i * kl), &mut ret);
            if ret <= 0 {
                eprintln!(
                    "Unexpected return from kh_put({}) : {}",
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr(keys.add(i * kl).cast()).to_bytes()),
                    ret,
                );
                kh_destroy_str2int(h);
                drop(Vec::from_raw_parts(keys, max * kl, max * kl));
                return -1;
            }
            *(*h).vals.add(k as usize) = i as i32;
            i += 1;
        }
    }

    let mut end = test_test_khash_c_312_get_time();
    if end < 0 {
        kh_destroy_str2int(h);
        drop(Vec::from_raw_parts(keys, max * kl, max * kl));
        return -1;
    }

    println!(
        "Insert {} {}",
        max,
        String::from_utf8_lossy(std::ffi::CStr::from_ptr(test_test_khash_c_330_fmt_time(end - start).cast()).to_bytes()),
    );

    start = test_test_khash_c_312_get_time();
    if start < 0 {
        kh_destroy_str2int(h);
        drop(Vec::from_raw_parts(keys, max * kl, max * kl));
        return -1;
    }

    i = 0;
    if !keys_file.is_null() {
        while i < max {
            let k = kh_get_str2int(h, *key_locations.add(i));
            if k >= (*h).n_buckets {
                eprintln!(
                    "Couldn't find {} in hash table",
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr((*key_locations.add(i)).cast()).to_bytes()),
                );
                kh_destroy_str2int(h);
                drop(Vec::from_raw_parts(keys, max * kl, max * kl));
                return -1;
            }
            i += 1;
        }
    } else {
        while i < max {
            let k = kh_get_str2int(h, keys.add(i * kl));
            if k >= (*h).n_buckets {
                eprintln!(
                    "Couldn't find {} in hash table",
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr(keys.add(i * kl).cast()).to_bytes()),
                );
                kh_destroy_str2int(h);
                drop(Vec::from_raw_parts(keys, max * kl, max * kl));
                return -1;
            }
            i += 1;
        }
    }

    end = test_test_khash_c_312_get_time();
    if end < 0 {
        kh_destroy_str2int(h);
        drop(Vec::from_raw_parts(keys, max * kl, max * kl));
        return -1;
    }

    println!(
        "Lookup {} {}",
        max,
        String::from_utf8_lossy(std::ffi::CStr::from_ptr(test_test_khash_c_330_fmt_time(end - start).cast()).to_bytes()),
    );

    test_test_khash_c_50_write_stats_str2int(h);

    kh_destroy_str2int(h);
    drop(Vec::from_raw_parts(keys, max * kl, max * kl));
    if !key_locations.is_null() {
        drop(Vec::from_raw_parts(key_locations, max, max));
    }

    0
}

// original: show_usage (htslib/test/test_khash.c:437)
pub unsafe fn test_test_khash_c_437_show_usage(to_stderr: bool, prog: *const u8) {
    let prog = String::from_utf8_lossy(std::ffi::CStr::from_ptr(prog.cast()).to_bytes()).into_owned();
    let lines = [
        format!("Usage : {} [-t <test>] [-i <file>]", prog),
        " Options:".to_string(),
        "  -t <TEST>   Test to run (str2int, benchmark)".to_string(),
        "  -i <FILE>   Optional input file for benchmark".to_string(),
        "  -n <INT>    Number of items to add".to_string(),
        "  -f <FRAC>   Fraction to delete and re-insert".to_string(),
        "  -d          Dump hash table stats".to_string(),
        "  -h          Show this help".to_string(),
    ];
    for line in lines {
        if to_stderr {
            eprintln!("{}", line);
        } else {
            println!("{}", line);
        }
    }
}

// original: main (htslib/test/test_khash.c:448)
pub unsafe fn test_test_khash_c_448_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut res = libc::EXIT_SUCCESS;
    let mut test: *mut u8 = std::ptr::null_mut();
    let mut input_file: *mut u8 = std::ptr::null_mut();
    let mut max = 1000usize;
    let mut del_frac = 0.25f64;
    let mut show_stats = 0;

    loop {
        let opt = libc::getopt(argc, argv.cast(), c"df:hi:n:t:".as_ptr());
        if opt == -1 {
            break;
        }
        match opt {
            c if c == b'd' as i32 => show_stats = 1,
            c if c == b'f' as i32 => {
                del_frac = libc::strtod(optarg.cast(), std::ptr::null_mut());
                if !(0.0..=1.0).contains(&del_frac) {
                    eprintln!("Error: -d must be between 0.0 and 1.0");
                    return libc::EXIT_FAILURE;
                }
            }
            c if c == b'h' as i32 => {
                test_test_khash_c_437_show_usage(false, (*argv).cast_const());
                return libc::EXIT_SUCCESS;
            }
            c if c == b'i' as i32 => input_file = optarg.cast(),
            c if c == b'n' as i32 => {
                max = libc::strtoul(optarg.cast(), std::ptr::null_mut(), 0) as usize;
                if !(1..=MAX_ENTRIES).contains(&max) {
                    eprintln!("Error: -n must be between 1 and {}", MAX_ENTRIES);
                    return libc::EXIT_FAILURE;
                }
            }
            c if c == b't' as i32 => test = optarg.cast(),
            _ => {
                test_test_khash_c_437_show_usage(true, (*argv).cast_const());
                return libc::EXIT_FAILURE;
            }
        }
    }

    if (test.is_null() || libc::strcmp(test.cast(), c"str2int".as_ptr()) == 0)
        && test_test_khash_c_137_test_str2int(max, (max as f64 * del_frac) as usize, show_stats)
            != 0
    {
        res = libc::EXIT_FAILURE;
    }

    if !test.is_null()
        && libc::strcmp(test.cast(), c"benchmark".as_ptr()) == 0
        && test_test_khash_c_344_benchmark(input_file.cast_const()) != 0
    {
        res = libc::EXIT_FAILURE;
    }

    res
}

unsafe extern "C" {
    static mut optarg: *mut u8;
    static mut optind: i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    unsafe fn run_main(args: &[Vec<u8>]) -> i32 {
        // NOTE: callers must already hold `ORIGINAL_MAIN_LOCK` (see
        // src/test/mod.rs) — this manipulates the libc getopt globals.
        let mut argv = args
            .iter()
            .map(|arg| arg.as_ptr().cast_mut())
            .collect::<Vec<*mut u8>>();
        optind = 0;
        test_test_khash_c_448_main(argv.len() as i32, argv.as_mut_ptr())
    }

    #[test]
    fn original_test_khash_main_runs_minimal_str2int_path() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            let argv = [
                b"test_khash\0".to_vec(),
                b"-t\0".to_vec(),
                b"str2int\0".to_vec(),
                b"-n\0".to_vec(),
                b"1\0".to_vec(),
                b"-f\0".to_vec(),
                b"0.0\0".to_vec(),
            ];
            assert_eq!(run_main(&argv), libc::EXIT_SUCCESS);
        }
    }

    #[test]
    fn original_test_khash_main_rejects_invalid_item_count() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            let argv = [
                b"test_khash\0".to_vec(),
                b"-t\0".to_vec(),
                b"str2int\0".to_vec(),
                b"-n\0".to_vec(),
                b"0\0".to_vec(),
            ];
            assert_eq!(run_main(&argv), libc::EXIT_FAILURE);
        }
    }

    #[test]
    fn original_test_khash_str2int_delete_reinsert_path_is_bounded() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            assert_eq!(test_test_khash_c_137_test_str2int(128, 37, 0), 0);
        }
    }

    #[test]
    fn original_test_khash_benchmark_path_accepts_file_backed_keys() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let path = std::env::temp_dir().join(format!(
            "htslib_rs_test_khash_keys_{}.txt",
            std::process::id()
        ));
        fs::write(
            &path,
            b"alpha\nbeta\ngamma\nalpha\ndelta\nbucket-collision-candidate\n",
        )
        .unwrap();
        let mut path_c = path.to_string_lossy().as_bytes().to_vec();
        path_c.push(0);

        unsafe {
            assert_eq!(test_test_khash_c_344_benchmark(path_c.as_ptr()), 0);
        }

        fs::remove_file(path).unwrap();
    }
}
