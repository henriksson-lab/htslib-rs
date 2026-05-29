//! Translation of `pooled_alloc.h` (htscodecs).
//!
//! Header-only pooled block allocator used by the name tokeniser (tok3).
//! All items are the same size; many of them are needed.
//!
//! Source: htslib/htscodecs/htscodecs/pooled_alloc.h

#![allow(non_camel_case_types)]

use std::os::raw::c_void;

use crate::c_compat;

/// `#define PSIZE 1024*1024`
// pooled_alloc.h:61
pub const PSIZE: usize = 1024 * 1024;

/// ```c
/// typedef struct {
///     void   *pool;
///     size_t  used;
/// } pool_t;
/// ```
// pooled_alloc.h:49
#[derive(Debug, Default)]
#[repr(C)]
pub struct pool_t {
    pub pool: *mut c_void,
    pub used: usize,
}

/// ```c
/// typedef struct {
///     size_t dsize;
///     size_t npools;
///     pool_t *pools;
///     void *free;
/// } pool_alloc_t;
/// ```
// pooled_alloc.h:54
#[derive(Debug)]
#[repr(C)]
pub struct pool_alloc_t {
    pub dsize: usize,
    pub npools: usize,
    pub pools: *mut pool_t,
    pub free: *mut c_void,
}

/// `static pool_alloc_t *pool_create(size_t dsize)`
///
/// Returns a malloc'd `pool_alloc_t` (NULL on failure).
// pooled_alloc.h:63
pub fn pool_create(mut dsize: usize) -> *mut pool_alloc_t {
    let p = unsafe { c_compat::malloc(std::mem::size_of::<pool_alloc_t>() as u64) }
        as *mut pool_alloc_t;
    if p.is_null() {
        return std::ptr::null_mut();
    }

    // Minimum size is a pointer, for free list
    let ptr_size = std::mem::size_of::<*mut c_void>();
    dsize = (dsize + ptr_size - 1) & !(ptr_size - 1);
    if dsize < ptr_size {
        dsize = ptr_size;
    }
    unsafe {
        (*p).dsize = dsize;
        (*p).npools = 0;
        (*p).pools = std::ptr::null_mut();
        (*p).free = std::ptr::null_mut();
    }

    p
}

/// `static pool_t *new_pool(pool_alloc_t *p)`
///
/// Returns a pointer into `p->pools` (which is realloc'd), NULL on failure.
// pooled_alloc.h:82
pub fn new_pool(p: &mut pool_alloc_t) -> *mut pool_t {
    let n = PSIZE / p.dsize;

    let pool = unsafe {
        c_compat::realloc(
            p.pools as *mut c_void,
            ((p.npools + 1) * std::mem::size_of::<pool_t>()) as u64,
        )
    } as *mut pool_t;
    if pool.is_null() {
        return std::ptr::null_mut();
    }
    p.pools = pool;
    let pool = unsafe { p.pools.add(p.npools) };

    unsafe {
        (*pool).pool = c_compat::malloc((n * p.dsize) as u64);
        if (*pool).pool.is_null() {
            return std::ptr::null_mut();
        }
        (*pool).used = 0;
    }

    p.npools += 1;

    pool
}

/// `static void pool_destroy(pool_alloc_t *p)`
// pooled_alloc.h:101
pub fn pool_destroy(p: *mut pool_alloc_t) {
    if p.is_null() {
        return;
    }
    unsafe {
        for i in 0..(*p).npools {
            c_compat::free((*(*p).pools.add(i)).pool);
        }
        c_compat::free((*p).pools as *mut c_void);
        c_compat::free(p as *mut c_void);
    }
}

/// `static void *pool_alloc(pool_alloc_t *p)`
///
/// Returns a pointer to a fixed-size slot from the pool (NULL on failure).
// pooled_alloc.h:111
pub fn pool_alloc(p: &mut pool_alloc_t) -> *mut c_void {
    // Look on free list
    if !p.free.is_null() {
        let ret = p.free;
        p.free = unsafe { *(p.free as *mut *mut c_void) };
        return ret;
    }

    // Look for space in the last pool
    if p.npools != 0 {
        let pool = unsafe { p.pools.add(p.npools - 1) };
        unsafe {
            if (*pool).used + p.dsize < PSIZE {
                let ret = ((*pool).pool as *mut u8).add((*pool).used) as *mut c_void;
                (*pool).used += p.dsize;
                return ret;
            }
        }
    }

    // Need a new pool
    let pool = new_pool(p);
    if pool.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        (*pool).used = p.dsize;
        (*pool).pool
    }
}

// Note: `pool_free` (pooled_alloc.h:140) is commented out in the C source and
// is intentionally not translated.
