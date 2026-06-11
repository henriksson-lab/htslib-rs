//! Translation of `pooled_alloc.h` (htscodecs).
//!
//! Header-only pooled block allocator used by the name tokeniser (tok3).
//! All items are the same size; many of them are needed.
//!
//! Source: htslib/htscodecs/htscodecs/pooled_alloc.h

#![allow(non_camel_case_types)]

/// `#define PSIZE 1024*1024`
// pooled_alloc.h:61
pub const PSIZE: usize = 1024 * 1024;

/// Identifies a fixed-size slot owned by a [`pool_alloc_t`].
///
/// In the C source a slot was handed out as a raw `void *` pointing inside a
/// pool's backing store. Here a slot is named by which pool it lives in and the
/// byte offset within that pool. The slot bytes themselves can then be reached
/// through [`pool_alloc_t::slot`] / [`pool_alloc_t::slot_mut`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotId {
    pub pool: usize,
    pub offset: usize,
}

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
#[derive(Debug, Default)]
pub struct pool_alloc_t {
    pub dsize: usize,
    pub pools: Vec<pool_t>,
    /// Free list of previously handed-out slots, used as a LIFO stack.
    ///
    /// Replaces the C intrusive linked list that threaded a `void *` "next"
    /// pointer through the first machine word of each free slot.
    ///
    /// To reach a slot's bytes, index `pools[id.pool].pool[id.offset..id.offset + dsize]`.
    pub free: Vec<SlotId>,
}

/// `static pool_alloc_t *pool_create(size_t dsize)`
///
/// Returns an owned allocator.
// pooled_alloc.h:63
pub fn pool_create(mut dsize: usize) -> pool_alloc_t {
    // Minimum size is a pointer, for free list
    let ptr_size = std::mem::size_of::<usize>();
    dsize = (dsize + ptr_size - 1) & !(ptr_size - 1);
    if dsize < ptr_size {
        dsize = ptr_size;
    }

    pool_alloc_t {
        dsize,
        pools: Vec::new(),
        free: Vec::new(),
    }
}

/// `static pool_t *new_pool(pool_alloc_t *p)`
///
/// Adds a new owned backing store and returns its pool index, or `None` on
/// allocation failure.
// pooled_alloc.h:82
pub fn new_pool(p: &mut pool_alloc_t) -> Option<usize> {
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

    Some(p.pools.len() - 1)
}

/// `static void pool_destroy(pool_alloc_t *p)`
// pooled_alloc.h:101
pub(crate) fn pool_destroy(p: pool_alloc_t) {
    // Dropping the owned allocator releases every pool's backing store.
    drop(p);
}

/// `static void *pool_alloc(pool_alloc_t *p)`
///
/// Returns the id of a fixed-size slot from the pool (`None` on failure).
// pooled_alloc.h:111
pub fn pool_alloc(p: &mut pool_alloc_t) -> Option<SlotId> {
    // Look on free list
    if let Some(id) = p.free.pop() {
        return Some(id);
    }

    // Look for space in the last pool
    if let Some(pool_idx) = p.pools.len().checked_sub(1) {
        let pool = &mut p.pools[pool_idx];
        if pool
            .used
            .checked_add(p.dsize)
            .is_some_and(|used| used < PSIZE)
        {
            let offset = pool.used;
            pool.used += p.dsize;
            return Some(SlotId {
                pool: pool_idx,
                offset,
            });
        }
    }

    // Need a new pool
    let dsize = p.dsize;
    let pool_idx = new_pool(p)?;
    p.pools[pool_idx].used = dsize;
    Some(SlotId {
        pool: pool_idx,
        offset: 0,
    })
}

// Note: `pool_free` (pooled_alloc.h:140) is commented out in the C source and
// is intentionally not translated. Were it enabled, it would push the freed
// slot's id onto `p.free`.
