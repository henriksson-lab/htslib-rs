// Functions translated from htslib/cram/pooled_alloc.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_int, c_void};

use super::*;

pub unsafe fn cram_pooled_alloc_c_47_next_power_2(mut v: u32) -> c_int {
    v = v.wrapping_sub(1);
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v = v.wrapping_add(1);
    v as c_int
}

pub unsafe fn cram_pooled_alloc_c_64_pool_create(dsize: usize) -> *mut pool_alloc_t {
    let p = malloc(std::mem::size_of::<pool_alloc_t>() as u64).cast::<pool_alloc_t>();
    if p.is_null() {
        return std::ptr::null_mut();
    }

    let mut rounded = (dsize + std::mem::size_of::<*mut c_void>() - 1)
        & !(std::mem::size_of::<*mut c_void>() - 1);
    if rounded < std::mem::size_of::<*mut c_void>() {
        rounded = std::mem::size_of::<*mut c_void>();
    }
    (*p).dsize = rounded;
    (*p).psize = std::cmp::min(
        POOLED_ALLOC_PSIZE,
        cram_pooled_alloc_c_47_next_power_2(((*p).dsize * 1024) as u32) as usize,
    );
    (*p).npools = 0;
    (*p).pools = std::ptr::null_mut();
    (*p).free = std::ptr::null_mut();

    p
}

pub unsafe fn cram_pooled_alloc_c_84_pool_destroy(p: *mut pool_alloc_t) {
    for i in 0..(*p).npools {
        free((*(*p).pools.add(i)).pool);
    }
    free((*p).pools.cast());
    free(p.cast());
}

pub unsafe fn cram_pooled_alloc_c_96_new_pool(p: *mut pool_alloc_t) -> *mut pool_t {
    let n = (*p).psize / (*p).dsize;
    let pools = realloc(
        (*p).pools.cast(),
        ((*p).npools + 1) as u64 * std::mem::size_of::<pool_t>() as u64,
    )
    .cast::<pool_t>();
    if pools.is_null() {
        return std::ptr::null_mut();
    }
    (*p).pools = pools;
    let pool = (*p).pools.add((*p).npools);

    (*pool).pool = malloc((n * (*p).dsize) as u64);
    if (*pool).pool.is_null() {
        return std::ptr::null_mut();
    }
    (*pool).used = 0;
    (*p).npools += 1;

    pool
}

pub unsafe fn cram_pooled_alloc_c_115_pool_alloc(p: *mut pool_alloc_t) -> *mut c_void {
    if !(*p).free.is_null() {
        let ret = (*p).free;
        (*p).free = *(ret.cast::<*mut c_void>());
        return ret;
    }

    if (*p).npools != 0 {
        let pool = (*p).pools.add((*p).npools - 1);
        if (*pool).used + (*p).dsize < (*p).psize {
            let ret = (*pool).pool.cast::<u8>().add((*pool).used).cast::<c_void>();
            (*pool).used += (*p).dsize;
            return ret;
        }
    }

    let pool = cram_pooled_alloc_c_96_new_pool(p);
    if pool.is_null() {
        return std::ptr::null_mut();
    }
    (*pool).used = (*p).dsize;
    (*pool).pool
}

pub unsafe fn cram_pooled_alloc_c_144_pool_free(p: *mut pool_alloc_t, ptr: *mut c_void) {
    *(ptr.cast::<*mut c_void>()) = (*p).free;
    (*p).free = ptr;
}

pub unsafe fn cram_pooled_alloc_c_151_pool_alloc(p: *mut pool_alloc_t) -> *mut c_void {
    malloc((*p).dsize as u64)
}

pub unsafe fn cram_pooled_alloc_c_155_pool_free(_p: *mut pool_alloc_t, ptr: *mut c_void) {
    free(ptr);
}

pub unsafe fn cram_pooled_alloc_c_167_main() -> c_int {
    let p = cram_pooled_alloc_c_64_pool_create(std::mem::size_of::<pooled_alloc_test_xyz>());
    if p.is_null() {
        return 1;
    }

    let np = 10000usize;
    let items = malloc((np * std::mem::size_of::<*mut pooled_alloc_test_xyz>()) as u64)
        .cast::<*mut pooled_alloc_test_xyz>();
    if items.is_null() {
        cram_pooled_alloc_c_84_pool_destroy(p);
        return 1;
    }

    for i in 0..np {
        let item = cram_pooled_alloc_c_115_pool_alloc(p).cast::<pooled_alloc_test_xyz>();
        if item.is_null() {
            free(items.cast());
            cram_pooled_alloc_c_84_pool_destroy(p);
            return 1;
        }
        (*item).x = i as c_int;
        (*item).y = i as c_int + 1;
        (*item).z = i as c_int + 2;
        *items.add(i) = item;
    }

    for i in 0..np {
        let item = *items.add(i);
        if i % 3 != 0 {
            cram_pooled_alloc_c_144_pool_free(p, item.cast());
        }
    }

    for i in 0..np {
        let item = cram_pooled_alloc_c_115_pool_alloc(p).cast::<pooled_alloc_test_xyz>();
        if item.is_null() {
            free(items.cast());
            cram_pooled_alloc_c_84_pool_destroy(p);
            return 1;
        }
        (*item).x = 1_000_000 + i as c_int;
        (*item).y = 1_000_000 + i as c_int + 1;
        (*item).z = 1_000_000 + i as c_int + 2;
    }

    for i in 0..np {
        cram_pooled_alloc_c_144_pool_free(p, (*items.add(i)).cast());
    }

    free(items.cast());
    cram_pooled_alloc_c_84_pool_destroy(p);
    0
}
