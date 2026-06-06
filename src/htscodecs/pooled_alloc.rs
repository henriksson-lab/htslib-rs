//! Translation of `pooled_alloc.h` (htscodecs).
//!
//! Header-only pooled block allocator used by the name tokeniser (tok3).
//! All items are the same size; many of them are needed.
//!
//! Source: htslib/htscodecs/htscodecs/pooled_alloc.h

#![allow(non_camel_case_types)]

use std::os::raw::c_void;
use std::ptr::NonNull;

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
pub struct pool_t {
    pub pool: Vec<u8>,
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
pub struct pool_alloc_t {
    pub dsize: usize,
    pub pools: Vec<pool_t>,
    pub free: Option<NonNull<c_void>>,
}

/// `static pool_alloc_t *pool_create(size_t dsize)`
///
/// Returns an owned allocator behind a raw pointer for translated callers.
// pooled_alloc.h:63
pub fn pool_create(mut dsize: usize) -> *mut pool_alloc_t {
    // Minimum size is a pointer, for free list
    let ptr_size = std::mem::size_of::<*mut c_void>();
    dsize = (dsize + ptr_size - 1) & !(ptr_size - 1);
    if dsize < ptr_size {
        dsize = ptr_size;
    }

    Box::into_raw(Box::new(pool_alloc_t {
        dsize,
        pools: Vec::new(),
        free: None,
    }))
}

/// `static pool_t *new_pool(pool_alloc_t *p)`
///
/// Adds a new owned backing store and returns it, or `None` on allocation failure.
// pooled_alloc.h:82
pub fn new_pool(p: &mut pool_alloc_t) -> Option<&mut pool_t> {
    let n = PSIZE / p.dsize;
    if n == 0 {
        return None;
    }
    let size = n.checked_mul(p.dsize)?;

    let mut storage = Vec::new();
    if storage.try_reserve_exact(size).is_err() {
        return None;
    }
    storage.resize(size, 0);

    if p.pools.try_reserve_exact(1).is_err() {
        return None;
    }

    p.pools.push(pool_t {
        pool: storage,
        used: 0,
    });

    p.pools.last_mut()
}

/// `static void pool_destroy(pool_alloc_t *p)`
// pooled_alloc.h:101
pub(crate) fn pool_destroy(p: *mut pool_alloc_t) {
    if p.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(p));
    }
}

/// `static void *pool_alloc(pool_alloc_t *p)`
///
/// Returns a pointer to a fixed-size slot from the pool (NULL on failure).
// pooled_alloc.h:111
pub fn pool_alloc(p: &mut pool_alloc_t) -> *mut c_void {
    // Look on free list
    if let Some(ret) = p.free {
        let next = unsafe { *(ret.as_ptr() as *mut *mut c_void) };
        p.free = NonNull::new(next);
        return ret.as_ptr();
    }

    // Look for space in the last pool
    if let Some(pool) = p.pools.last_mut() {
        if pool
            .used
            .checked_add(p.dsize)
            .is_some_and(|used| used < PSIZE)
        {
            let ret = unsafe { pool.pool.as_mut_ptr().add(pool.used) }.cast::<c_void>();
            pool.used += p.dsize;
            return ret;
        }
    }

    // Need a new pool
    let dsize = p.dsize;
    let Some(pool) = new_pool(p) else {
        return std::ptr::null_mut();
    };

    pool.used = dsize;
    pool.pool.as_mut_ptr().cast::<c_void>()
}

// Note: `pool_free` (pooled_alloc.h:140) is commented out in the C source and
// is intentionally not translated.
