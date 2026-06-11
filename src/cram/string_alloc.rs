// Functions translated from htslib/cram/string_alloc.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use super::*;

/// A descriptor for a single slab: its remaining-used byte count.
///
/// In the original C this paired a raw `str_` pointer with a `used` length; the
/// pointer is now implicit (the slab is owned by `StringPool::slabs` at the
/// matching index), so only the `used` count remains.
pub struct cram_string_alloc_string_t {
    pub used: usize,
}

/// An arena allocator that hands out byte strings carved from owned slabs.
pub struct StringPool {
    pub max_length: usize,
    /// Owned slabs of bytes; strings are sub-slices of these.
    slabs: Vec<Vec<u8>>,
    /// `used[i]` is the number of bytes consumed from `slabs[i]`.
    used: Vec<usize>,
}

impl StringPool {
    fn new(max_length: usize) -> Self {
        Self {
            max_length,
            slabs: Vec::new(),
            used: Vec::new(),
        }
    }

    /// Number of slabs allocated (the C `nstrings`).
    pub fn nstrings(&self) -> usize {
        self.slabs.len()
    }

    /// The `used` byte count of slab `index` (the C descriptor's `used` field).
    pub fn used_of(&self, index: usize) -> Option<usize> {
        self.used.get(index).copied()
    }
}

fn new_slab(pool: &mut StringPool) -> Option<usize> {
    let mut slab = Vec::new();
    if slab.try_reserve_exact(pool.max_length).is_err() {
        return None;
    }
    slab.resize(pool.max_length, 0);

    if pool.slabs.try_reserve_exact(1).is_err() {
        return None;
    }
    if pool.used.try_reserve_exact(1).is_err() {
        return None;
    }

    let index = pool.slabs.len();
    pool.slabs.push(slab);
    pool.used.push(0);

    Some(index)
}

fn string_alloc(pool: &mut StringPool, length: usize) -> Option<&mut [u8]> {
    if length == 0 {
        return None;
    }

    if let Some(index) = pool.slabs.len().checked_sub(1) {
        let max_length = pool.max_length;
        let used = pool.used[index];
        let end = used.checked_add(length)?;

        if end < max_length {
            pool.used[index] = end;
            return pool.slabs.get_mut(index).map(|slab| &mut slab[used..end]);
        }
    }

    if length > pool.max_length {
        pool.max_length = length;
    }

    let index = new_slab(pool)?;
    pool.used[index] = length;
    pool.slabs.get_mut(index).map(|slab| &mut slab[..length])
}

fn string_ndup<'a>(pool: &'a mut StringPool, instr: &[u8]) -> Option<&'a mut [u8]> {
    let out = string_alloc(pool, instr.len().checked_add(1)?)?;

    let (body, terminator) = out.split_at_mut(instr.len());
    body.copy_from_slice(instr);
    terminator[0] = 0;

    Some(out)
}

pub fn cram_string_alloc_c_55_string_pool_create(mut max_length: usize) -> Box<StringPool> {
    if max_length < CRAM_STRING_ALLOC_MIN_STR_SIZE {
        max_length = CRAM_STRING_ALLOC_MIN_STR_SIZE;
    }

    Box::new(StringPool::new(max_length))
}

/// Create a fresh slab and return its index within the pool. The original C
/// returned a pointer to the slab's descriptor; with owned slabs the index is
/// the idiomatic handle (callers read `pool.used_of(index)` for the descriptor
/// `used` count).
pub fn cram_string_alloc_c_75_new_string_pool(pool: &mut StringPool) -> Option<usize> {
    new_slab(pool)
}

pub fn cram_string_alloc_c_103_string_pool_destroy(pool: Box<StringPool>) {
    drop(pool);
}

pub fn cram_string_alloc_c_117_string_alloc(
    pool: &mut StringPool,
    length: usize,
) -> Option<&mut [u8]> {
    if length == 0 {
        return None;
    }

    string_alloc(pool, length)
}

pub fn cram_string_alloc_c_149_string_dup<'a>(
    pool: &'a mut StringPool,
    instr: &[u8],
) -> Option<&'a mut [u8]> {
    string_ndup(pool, instr)
}

pub fn cram_string_alloc_c_153_string_ndup<'a>(
    pool: &'a mut StringPool,
    instr: &[u8],
    len: usize,
) -> Option<&'a mut [u8]> {
    let instr = instr.get(..len)?;
    string_ndup(pool, instr)
}
