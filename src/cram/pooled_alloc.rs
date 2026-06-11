// Functions translated from htslib/cram/pooled_alloc.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use super::*;

pub fn cram_pooled_alloc_c_47_next_power_2(mut v: u32) -> i32 {
    v = v.wrapping_sub(1);
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v = v.wrapping_add(1);
    v as i32
}

/// Creates a pool.
///
/// Pool allocations are approx minimum of 1024*dsize or PSIZE.
/// (Assumes we're not trying to use pools for >= 2Gb or more)
pub fn cram_pooled_alloc_c_64_pool_create(mut dsize: usize) -> Box<pool_alloc_t> {
    // Minimum size is a pointer, for free list
    let ptr_size = std::mem::size_of::<usize>();
    dsize = (dsize + ptr_size - 1) & !(ptr_size - 1);
    if dsize < ptr_size {
        dsize = ptr_size;
    }

    Box::new(pool_alloc_t {
        dsize,
        psize: std::cmp::min(
            POOLED_ALLOC_PSIZE,
            cram_pooled_alloc_c_47_next_power_2((dsize * 1024) as u32) as usize,
        ),
        pools: Vec::new(),
        free: Vec::new(),
    })
}

/// Dropping the owned allocator releases every pool's backing store.
pub fn cram_pooled_alloc_c_84_pool_destroy(p: Box<pool_alloc_t>) {
    drop(p);
}

/// Adds a new owned backing store and returns its pool index, or `None` on
/// allocation failure.
pub fn cram_pooled_alloc_c_96_new_pool(p: &mut pool_alloc_t) -> Option<usize> {
    let n = p.psize / p.dsize;
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

/// Returns the id of a fixed-size slot from the pool (`None` on failure).
pub fn cram_pooled_alloc_c_115_pool_alloc(p: &mut pool_alloc_t) -> Option<SlotId> {
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
            .is_some_and(|used| used < p.psize)
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
    let pool_idx = cram_pooled_alloc_c_96_new_pool(p)?;
    p.pools[pool_idx].used = dsize;
    Some(SlotId {
        pool: pool_idx,
        offset: 0,
    })
}

/// Returns a slot to the free list. Replaces the C intrusive linked list that
/// threaded a "next" pointer through the first machine word of each free slot.
pub fn cram_pooled_alloc_c_144_pool_free(p: &mut pool_alloc_t, id: SlotId) {
    p.free.push(id);
}

// Pre-existing public aliases kept for out-of-file callers (e.g. bgzf.rs).
// They forward to the audit-anchor implementations above.

pub fn pool_create(dsize: usize) -> Box<pool_alloc_t> {
    cram_pooled_alloc_c_64_pool_create(dsize)
}

pub fn pool_destroy_box(p: Box<pool_alloc_t>) {
    cram_pooled_alloc_c_84_pool_destroy(p);
}

pub fn pool_alloc(p: &mut pool_alloc_t) -> Option<SlotId> {
    cram_pooled_alloc_c_115_pool_alloc(p)
}

pub fn pool_free(p: &mut pool_alloc_t, id: SlotId) {
    cram_pooled_alloc_c_144_pool_free(p, id);
}

#[repr(C)]
struct pooled_alloc_test_xyz {
    x: i32,
    y: i32,
    z: i32,
}

pub fn cram_pooled_alloc_c_167_main() -> i32 {
    let mut p = cram_pooled_alloc_c_64_pool_create(std::mem::size_of::<pooled_alloc_test_xyz>());

    let np = 10000usize;
    let mut items = Vec::<SlotId>::new();
    if items.try_reserve_exact(np).is_err() {
        cram_pooled_alloc_c_84_pool_destroy(p);
        return 1;
    }

    for i in 0..np {
        let Some(id) = cram_pooled_alloc_c_115_pool_alloc(&mut p) else {
            cram_pooled_alloc_c_84_pool_destroy(p);
            return 1;
        };
        let item = &mut p.pools[id.pool].pool[id.offset..id.offset + std::mem::size_of::<pooled_alloc_test_xyz>()];
        item[0..4].copy_from_slice(&(i as i32).to_ne_bytes());
        item[4..8].copy_from_slice(&(i as i32 + 1).to_ne_bytes());
        item[8..12].copy_from_slice(&(i as i32 + 2).to_ne_bytes());
        items.push(id);
    }

    for (i, &id) in items.iter().enumerate() {
        if i % 3 != 0 {
            cram_pooled_alloc_c_144_pool_free(&mut p, id);
        }
    }

    for i in 0..np {
        let Some(id) = cram_pooled_alloc_c_115_pool_alloc(&mut p) else {
            cram_pooled_alloc_c_84_pool_destroy(p);
            return 1;
        };
        let item = &mut p.pools[id.pool].pool[id.offset..id.offset + std::mem::size_of::<pooled_alloc_test_xyz>()];
        item[0..4].copy_from_slice(&(1_000_000 + i as i32).to_ne_bytes());
        item[4..8].copy_from_slice(&(1_000_000 + i as i32 + 1).to_ne_bytes());
        item[8..12].copy_from_slice(&(1_000_000 + i as i32 + 2).to_ne_bytes());
    }

    for id in items {
        cram_pooled_alloc_c_144_pool_free(&mut p, id);
    }

    cram_pooled_alloc_c_84_pool_destroy(p);
    0
}
