use std::ffi::{c_char, c_int, CStr};

type khint_t = u32;

const MAX_ENTRIES: usize = 99_999_999;

#[repr(C)]
pub struct kh_str2int_t {
    n_buckets: khint_t,
    size: khint_t,
    n_occupied: khint_t,
    upper_bound: khint_t,
    flags: *mut khint_t,
    keys: *mut *mut c_char,
    vals: *mut c_int,
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

unsafe fn kh_str_hash_func(key: *const c_char) -> khint_t {
    crate::htslib_rs::hts::__ac_FNV1a_hash_string(key)
}

unsafe fn kh_str_hash_equal(a: *const c_char, b: *const c_char) -> bool {
    !a.is_null() && !b.is_null() && CStr::from_ptr(a) == CStr::from_ptr(b)
}

unsafe fn kh_init_str2int() -> *mut kh_str2int_t {
    libc::calloc(1, std::mem::size_of::<kh_str2int_t>()).cast::<kh_str2int_t>()
}

unsafe fn kh_destroy_str2int(h: *mut kh_str2int_t) {
    if h.is_null() {
        return;
    }
    libc::free((*h).flags.cast());
    libc::free((*h).keys.cast());
    libc::free((*h).vals.cast());
    libc::free(h.cast());
}

unsafe fn kh_resize_str2int(h: *mut kh_str2int_t, mut new_n_buckets: khint_t) -> c_int {
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
    let new_flags =
        libc::malloc(n_flags as usize * std::mem::size_of::<khint_t>()).cast::<khint_t>();
    if new_flags.is_null() {
        return -1;
    }
    libc::memset(
        new_flags.cast(),
        0xaa,
        n_flags as usize * std::mem::size_of::<khint_t>(),
    );

    if (*h).n_buckets < new_n_buckets {
        let new_keys = libc::realloc(
            (*h).keys.cast(),
            new_n_buckets as usize * std::mem::size_of::<*mut c_char>(),
        )
        .cast::<*mut c_char>();
        if new_keys.is_null() {
            libc::free(new_flags.cast());
            return -1;
        }
        (*h).keys = new_keys;
        let new_vals = libc::realloc(
            (*h).vals.cast(),
            new_n_buckets as usize * std::mem::size_of::<c_int>(),
        )
        .cast::<c_int>();
        if new_vals.is_null() {
            libc::free(new_flags.cast());
            return -1;
        }
        (*h).vals = new_vals;
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
                    std::mem::swap(&mut *(*h).keys.add(i as usize), &mut key);
                    std::mem::swap(&mut *(*h).vals.add(i as usize), &mut val);
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
        let new_keys = libc::realloc(
            (*h).keys.cast(),
            new_n_buckets as usize * std::mem::size_of::<*mut c_char>(),
        )
        .cast::<*mut c_char>();
        if new_keys.is_null() {
            libc::free(new_flags.cast());
            return -1;
        }
        (*h).keys = new_keys;
        let new_vals = libc::realloc(
            (*h).vals.cast(),
            new_n_buckets as usize * std::mem::size_of::<c_int>(),
        )
        .cast::<c_int>();
        if new_vals.is_null() {
            libc::free(new_flags.cast());
            return -1;
        }
        (*h).vals = new_vals;
    }

    libc::free((*h).flags.cast());
    (*h).flags = new_flags;
    (*h).n_buckets = new_n_buckets;
    (*h).n_occupied = (*h).size;
    (*h).upper_bound = (new_n_buckets as f64 * 0.77 + 0.5) as khint_t;
    if (*h).upper_bound == new_n_buckets {
        (*h).upper_bound -= 1;
    }
    0
}

unsafe fn kh_put_str2int(h: *mut kh_str2int_t, key: *mut c_char, ret: *mut c_int) -> khint_t {
    let x;
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
    x = if kh_isempty((*h).flags, i) {
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

unsafe fn kh_get_str2int(h: *const kh_str2int_t, key: *const c_char) -> khint_t {
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
) -> c_int {
    let mut hist = libc::calloc(1, std::mem::size_of::<khint_t>()).cast::<khint_t>();
    let mut dist_max = 0;
    let mask = (*h).n_buckets - 1;
    *empty = 0;
    *deleted = 0;
    *hist_size = 0;
    if hist.is_null() {
        return -1;
    }
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
            let new_hist = libc::realloc(
                hist.cast(),
                (dist as usize + 1) * std::mem::size_of::<khint_t>(),
            )
            .cast::<khint_t>();
            if new_hist.is_null() {
                libc::free(hist.cast());
                return -1;
            }
            let mut k = dist_max + 1;
            while k <= dist {
                *new_hist.add(k as usize) = 0;
                k += 1;
            }
            hist = new_hist;
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
        libc::printf(c"n_buckets = %u\n".as_ptr(), (*h).n_buckets);
        libc::printf(c"empty     = %u\n".as_ptr(), empty);
        libc::printf(c"deleted   = %u\n".as_ptr(), deleted);
        let mut i = 0;
        while i < hist_size {
            libc::printf(c"dist[ %8u ] = %u\n".as_ptr(), i, *hist.add(i as usize));
            i += 1;
        }
        libc::free(hist.cast());
    }
}

// original: make_keys (htslib/test/test_khash.c:66)
pub unsafe fn test_test_khash_c_66_make_keys(num: usize, kl: usize) -> *mut c_char {
    if num > MAX_ENTRIES {
        return std::ptr::null_mut();
    }
    let keys = libc::malloc(kl * num).cast::<c_char>();
    if keys.is_null() {
        libc::perror(std::ptr::null());
        return std::ptr::null_mut();
    }
    let mut i = 0;
    while i < num {
        if libc::snprintf(keys.add(kl * i), kl, c"test%zu".as_ptr(), i) as usize >= kl {
            libc::free(keys.cast());
            return std::ptr::null_mut();
        }
        i += 1;
    }

    keys
}

// original: add_str2int_entry (htslib/test/test_khash.c:86)
pub unsafe fn test_test_khash_c_86_add_str2int_entry(
    h: *mut kh_str2int_t,
    key: *mut c_char,
    val: khint_t,
) -> c_int {
    let mut ret = 0;
    let k = kh_put_str2int(h, key, &mut ret);

    if ret != 1 && ret != 2 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Unexpected return from kh_put(%s) : %d\n".as_ptr(),
            key,
            ret,
        );
        return -1;
    }
    *(*h).vals.add(k as usize) = val as c_int;
    0
}

// original: check_str2int_entry (htslib/test/test_khash.c:98)
pub unsafe fn test_test_khash_c_98_check_str2int_entry(
    h: *mut kh_str2int_t,
    key: *mut c_char,
    val: khint_t,
    is_deleted: u8,
) -> c_int {
    let k = kh_get_str2int(h, key);
    if is_deleted != 0 {
        if k < (*h).n_buckets {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Found deleted entry %s in hash table\n".as_ptr(),
                key,
            );
            return -1;
        } else {
            return 0;
        }
    }

    if k >= (*h).n_buckets {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Couldn't find %s in hash table\n".as_ptr(),
            key,
        );
        return -1;
    }
    if libc::strcmp(*(*h).keys.add(k as usize), key) != 0 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Wrong key in hash table, expected %s got %s\n".as_ptr(),
            key,
            *(*h).keys.add(k as usize),
        );
        return -1;
    }
    if *(*h).vals.add(k as usize) != val as c_int {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Wrong value in hash table, expected %u got %u\n".as_ptr(),
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
    key: *mut c_char,
) -> c_int {
    let k = kh_get_str2int(h, key);
    if k >= (*h).n_buckets {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Couldn't find %s to delete from hash table\n".as_ptr(),
            key,
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
    show_stats: c_int,
) -> c_int {
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
        libc::perror(std::ptr::null());
        libc::free(keys.cast());
        return -1;
    }

    // Add some entries
    let mut i = 0;
    while i < max {
        if test_test_khash_c_86_add_str2int_entry(h, keys.add(i * kl), i as khint_t) != 0 {
            kh_destroy_str2int(h);
            libc::free(keys.cast());
            libc::free(flags.cast());
            return -1;
        }
        i += 1;
    }

    // Check they exist
    i = 0;
    while i < max {
        if test_test_khash_c_98_check_str2int_entry(h, keys.add(i * kl), i as khint_t, 0) != 0 {
            kh_destroy_str2int(h);
            libc::free(keys.cast());
            libc::free(flags.cast());
            return -1;
        }
        i += 1;
    }

    if show_stats != 0 {
        libc::printf(c"Initial fill:\n".as_ptr());
        test_test_khash_c_50_write_stats_str2int(h);
    }

    // Delete a random selection
    flags = libc::calloc(max, std::mem::size_of::<u8>()).cast::<u8>();
    if flags.is_null() {
        libc::perror(c"".as_ptr());
        kh_destroy_str2int(h);
        libc::free(keys.cast());
        return -1;
    }

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
            libc::free(keys.cast());
            libc::free(flags.cast());
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
            libc::free(keys.cast());
            libc::free(flags.cast());
            return -1;
        }
        i += 1;
    }

    if show_stats != 0 {
        libc::printf(c"\nAfter deletion:\n".as_ptr());
        test_test_khash_c_50_write_stats_str2int(h);
    }

    // Re-insert deleted entries
    i = 0;
    while i < max {
        if *flags.add(i) != 0
            && test_test_khash_c_86_add_str2int_entry(h, keys.add(i * kl), i as khint_t) != 0
        {
            kh_destroy_str2int(h);
            libc::free(keys.cast());
            libc::free(flags.cast());
            return -1;
        }
        i += 1;
    }

    // Ensure they're all back
    i = 0;
    while i < max {
        if test_test_khash_c_98_check_str2int_entry(h, keys.add(i * kl), i as khint_t, 0) != 0 {
            kh_destroy_str2int(h);
            libc::free(keys.cast());
            libc::free(flags.cast());
            return -1;
        }
        i += 1;
    }

    if show_stats != 0 {
        libc::printf(c"\nAfter re-insert:\n".as_ptr());
        test_test_khash_c_50_write_stats_str2int(h);
    }

    kh_destroy_str2int(h);
    libc::free(keys.cast());
    libc::free(flags.cast());

    0
}

// original: read_keys (htslib/test/test_khash.c:236)
pub unsafe fn test_test_khash_c_236_read_keys(
    keys_file: *const c_char,
    keys_out: *mut *mut c_char,
    key_locations_out: *mut *mut *mut c_char,
) -> usize {
    let mut in_ = libc::fopen(keys_file, c"r".as_ptr());
    let mut keys_size = 1_000_000usize;
    let mut keys_used = 0usize;
    let mut nkeys = 0usize;
    let mut fileinfo: libc::stat = std::mem::zeroed();

    if in_.is_null() {
        return 0;
    }

    // Slurp entire file
    if libc::fstat(libc::fileno(in_), &mut fileinfo) < 0 {
        if fileinfo.st_size as usize > keys_size {
            keys_size = fileinfo.st_size as usize;
        }
    }

    let mut keys = libc::malloc(keys_size + 1).cast::<c_char>();
    if keys.is_null() {
        if !in_.is_null() {
            libc::fclose(in_);
        }
        *keys_out = std::ptr::null_mut();
        *key_locations_out = std::ptr::null_mut();
        return 0;
    }

    loop {
        let mut avail = keys_size - keys_used;
        if avail == 0 {
            let new_size = keys_size + 1_000_000;
            let new_keys = libc::realloc(keys.cast(), new_size + 1).cast::<c_char>();
            if new_keys.is_null() {
                libc::fclose(in_);
                libc::free(keys.cast());
                *keys_out = std::ptr::null_mut();
                *key_locations_out = std::ptr::null_mut();
                return 0;
            }
            keys = new_keys;
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
        libc::free(keys.cast());
        *keys_out = std::ptr::null_mut();
        *key_locations_out = std::ptr::null_mut();
        return 0;
    }
    if libc::fclose(in_) < 0 {
        libc::free(keys.cast());
        *keys_out = std::ptr::null_mut();
        *key_locations_out = std::ptr::null_mut();
        return 0;
    }
    in_ = std::ptr::null_mut();

    // Split by line
    let end = keys.add(keys_used);
    let mut key = keys;
    while !key.is_null() {
        while *key == b'\n' as c_char {
            key = key.add(1);
        }
        if key < end {
            nkeys += 1;
        }
        key = libc::memchr(key.cast(), b'\n' as c_int, end.offset_from(key) as usize).cast();
    }

    let key_locations =
        libc::malloc(nkeys * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
    if key_locations.is_null() {
        if !in_.is_null() {
            libc::fclose(in_);
        }
        libc::free(keys.cast());
        *keys_out = std::ptr::null_mut();
        *key_locations_out = std::ptr::null_mut();
        return 0;
    }

    nkeys = 0;
    key = keys;
    while !key.is_null() {
        while *key == b'\n' as c_char {
            *key = 0;
            key = key.add(1);
        }
        if key < end {
            *key_locations.add(nkeys) = key;
            nkeys += 1;
        }
        key = libc::memchr(key.cast(), b'\n' as c_int, end.offset_from(key) as usize).cast();
    }
    *keys_out = keys;
    *key_locations_out = key_locations;
    nkeys
}

// original: get_time (htslib/test/test_khash.c:312)
pub unsafe fn test_test_khash_c_312_get_time() -> i64 {
    let mut tv: libc::timeval = std::mem::zeroed();
    if libc::gettimeofday(&mut tv, std::ptr::null_mut()) < 0 {
        libc::perror(c"gettimeofday".as_ptr());
        return -1;
    }
    tv.tv_sec as i64 * 1_000_000 + tv.tv_usec as i64
}

// original: fmt_time (htslib/test/test_khash.c:330)
pub unsafe fn test_test_khash_c_330_fmt_time(elapsed: i64) -> *mut c_char {
    static mut BUF: [c_char; 64] = [0; 64];
    let sec = elapsed / 1_000_000;
    let usec = elapsed % 1_000_000;
    libc::snprintf(
        std::ptr::addr_of_mut!(BUF).cast::<c_char>(),
        64,
        c"%lld.%06lld wall-time seconds".as_ptr(),
        sec,
        usec,
    );
    std::ptr::addr_of_mut!(BUF).cast::<c_char>()
}

// original: benchmark (htslib/test/test_khash.c:344)
pub unsafe fn test_test_khash_c_344_benchmark(keys_file: *const c_char) -> c_int {
    let kl = 16;
    let mut max = 50_000_000usize;
    let mut keys: *mut c_char = std::ptr::null_mut();
    let mut key_locations: *mut *mut c_char = std::ptr::null_mut();

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
        libc::free(keys.cast());
        return -1;
    }

    let mut start = test_test_khash_c_312_get_time();
    if start < 0 {
        kh_destroy_str2int(h);
        libc::free(keys.cast());
        return -1;
    }

    let mut i = 0usize;
    if !keys_file.is_null() {
        while i < max {
            let mut ret = 0;
            let k = kh_put_str2int(h, *key_locations.add(i), &mut ret);
            if ret < 0 {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Unexpected return from kh_put(%s) : %d\n".as_ptr(),
                    *key_locations.add(i),
                    ret,
                );
                kh_destroy_str2int(h);
                libc::free(keys.cast());
                return -1;
            }
            *(*h).vals.add(k as usize) = i as c_int;
            i += 1;
        }
    } else {
        while i < max {
            let mut ret = 0;
            let k = kh_put_str2int(h, keys.add(i * kl), &mut ret);
            if ret <= 0 {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Unexpected return from kh_put(%s) : %d\n".as_ptr(),
                    keys.add(i * kl),
                    ret,
                );
                kh_destroy_str2int(h);
                libc::free(keys.cast());
                return -1;
            }
            *(*h).vals.add(k as usize) = i as c_int;
            i += 1;
        }
    }

    let mut end = test_test_khash_c_312_get_time();
    if end < 0 {
        kh_destroy_str2int(h);
        libc::free(keys.cast());
        return -1;
    }

    libc::printf(
        c"Insert %zu %s\n".as_ptr(),
        max,
        test_test_khash_c_330_fmt_time(end - start),
    );

    start = test_test_khash_c_312_get_time();
    if start < 0 {
        kh_destroy_str2int(h);
        libc::free(keys.cast());
        return -1;
    }

    i = 0;
    if !keys_file.is_null() {
        while i < max {
            let k = kh_get_str2int(h, *key_locations.add(i));
            if k >= (*h).n_buckets {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Couldn't find %s in hash table\n".as_ptr(),
                    *key_locations.add(i),
                );
                kh_destroy_str2int(h);
                libc::free(keys.cast());
                return -1;
            }
            i += 1;
        }
    } else {
        while i < max {
            let k = kh_get_str2int(h, keys.add(i * kl));
            if k >= (*h).n_buckets {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Couldn't find %s in hash table\n".as_ptr(),
                    keys.add(i * kl),
                );
                kh_destroy_str2int(h);
                libc::free(keys.cast());
                return -1;
            }
            i += 1;
        }
    }

    end = test_test_khash_c_312_get_time();
    if end < 0 {
        kh_destroy_str2int(h);
        libc::free(keys.cast());
        return -1;
    }

    libc::printf(
        c"Lookup %zu %s\n".as_ptr(),
        max,
        test_test_khash_c_330_fmt_time(end - start),
    );

    test_test_khash_c_50_write_stats_str2int(h);

    kh_destroy_str2int(h);
    libc::free(keys.cast());
    libc::free(key_locations.cast());

    0
}

// original: show_usage (htslib/test/test_khash.c:437)
pub unsafe fn test_test_khash_c_437_show_usage(out: *mut libc::FILE, prog: *mut c_char) {
    libc::fprintf(out, c"Usage : %s [-t <test>] [-i <file>]\n".as_ptr(), prog);
    libc::fprintf(out, c" Options:\n".as_ptr());
    libc::fprintf(
        out,
        c"  -t <TEST>   Test to run (str2int, benchmark)\n".as_ptr(),
    );
    libc::fprintf(
        out,
        c"  -i <FILE>   Optional input file for benchmark\n".as_ptr(),
    );
    libc::fprintf(out, c"  -n <INT>    Number of items to add\n".as_ptr());
    libc::fprintf(
        out,
        c"  -f <FRAC>   Fraction to delete and re-insert\n".as_ptr(),
    );
    libc::fprintf(out, c"  -d          Dump hash table stats\n".as_ptr());
    libc::fprintf(out, c"  -h          Show this help\n".as_ptr());
}

// original: main (htslib/test/test_khash.c:448)
pub unsafe fn test_test_khash_c_448_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut res = libc::EXIT_SUCCESS;
    let mut test: *mut c_char = std::ptr::null_mut();
    let mut input_file: *mut c_char = std::ptr::null_mut();
    let mut max = 1000usize;
    let mut del_frac = 0.25f64;
    let mut show_stats = 0;

    loop {
        let opt = libc::getopt(argc, argv, c"df:hi:n:t:".as_ptr());
        if opt == -1 {
            break;
        }
        match opt {
            c if c == b'd' as c_int => show_stats = 1,
            c if c == b'f' as c_int => {
                del_frac = libc::strtod(optarg, std::ptr::null_mut());
                if del_frac < 0.0 || del_frac > 1.0 {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Error: -d must be between 0.0 and 1.0\n".as_ptr(),
                    );
                    return libc::EXIT_FAILURE;
                }
            }
            c if c == b'h' as c_int => {
                test_test_khash_c_437_show_usage(crate::htslib_rs::c_compat::stdout.cast(), *argv);
                return libc::EXIT_SUCCESS;
            }
            c if c == b'i' as c_int => input_file = optarg,
            c if c == b'n' as c_int => {
                max = libc::strtoul(optarg, std::ptr::null_mut(), 0) as usize;
                if max == 0 || max > 99_999_999 {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Error: -n must be between 1 and %u\n".as_ptr(),
                        MAX_ENTRIES as c_int,
                    );
                    return libc::EXIT_FAILURE;
                }
            }
            c if c == b't' as c_int => test = optarg,
            _ => {
                test_test_khash_c_437_show_usage(crate::htslib_rs::c_compat::stderr.cast(), *argv);
                return libc::EXIT_FAILURE;
            }
        }
    }

    if test.is_null() || libc::strcmp(test, c"str2int".as_ptr()) == 0 {
        if test_test_khash_c_137_test_str2int(max, (max as f64 * del_frac) as usize, show_stats)
            != 0
        {
            res = libc::EXIT_FAILURE;
        }
    }

    if !test.is_null() && libc::strcmp(test, c"benchmark".as_ptr()) == 0 {
        if test_test_khash_c_344_benchmark(input_file) != 0 {
            res = libc::EXIT_FAILURE;
        }
    }

    res
}

unsafe extern "C" {
    static mut optarg: *mut c_char;
    static mut optind: c_int;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::fs;

    unsafe fn run_main(args: &[CString]) -> c_int {
        // NOTE: callers must already hold `ORIGINAL_MAIN_LOCK` (see
        // src/test/mod.rs) — this manipulates the libc getopt globals.
        let mut argv = args
            .iter()
            .map(|arg| arg.as_ptr().cast_mut())
            .collect::<Vec<_>>();
        optind = 0;
        test_test_khash_c_448_main(argv.len() as c_int, argv.as_mut_ptr())
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
                CString::new("test_khash").unwrap(),
                CString::new("-t").unwrap(),
                CString::new("str2int").unwrap(),
                CString::new("-n").unwrap(),
                CString::new("1").unwrap(),
                CString::new("-f").unwrap(),
                CString::new("0.0").unwrap(),
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
                CString::new("test_khash").unwrap(),
                CString::new("-t").unwrap(),
                CString::new("str2int").unwrap(),
                CString::new("-n").unwrap(),
                CString::new("0").unwrap(),
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
        let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();

        unsafe {
            assert_eq!(test_test_khash_c_344_benchmark(path_c.as_ptr()), 0);
        }

        fs::remove_file(path).unwrap();
    }
}
