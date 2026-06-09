// Functions translated from htslib/cram/string_alloc.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::c_char;
use std::ptr::NonNull;

use super::*;

#[repr(C)]
struct StringPool {
    header: cram_string_alloc_t,
    descriptors: Vec<cram_string_alloc_string_t>,
    slabs: Vec<Box<[u8]>>,
    used: Vec<usize>,
}

impl StringPool {
    fn new(max_length: usize) -> Self {
        let mut pool = Self {
            header: cram_string_alloc_t {
                max_length,
                nstrings: 0,
                max_strings: 0,
                strings: std::ptr::null_mut(),
            },
            descriptors: Vec::new(),
            slabs: Vec::new(),
            used: Vec::new(),
        };
        pool.sync_header();
        pool
    }

    fn sync_header(&mut self) {
        self.descriptors.clear();
        self.descriptors
            .extend(self.slabs.iter_mut().zip(&self.used).map(|(slab, &used)| {
                cram_string_alloc_string_t {
                    str_: slab.as_mut_ptr().cast(),
                    used,
                }
            }));

        self.header.nstrings = self.descriptors.len();
        self.header.max_strings = self.descriptors.capacity();
        self.header.strings = self
            .descriptors
            .first_mut()
            .map_or(std::ptr::null_mut(), |str_| str_ as *mut _);
    }
}

unsafe fn string_pool_mut<'a>(a_str: *mut cram_string_alloc_t) -> Option<&'a mut StringPool> {
    NonNull::new(a_str.cast::<StringPool>()).map(|mut pool| pool.as_mut())
}

unsafe fn string_pool_from_header(a_str: NonNull<cram_string_alloc_t>) -> Box<StringPool> {
    Box::from_raw(a_str.as_ptr().cast::<StringPool>())
}

fn reserve_descriptor_slot(pool: &mut StringPool) -> bool {
    if pool.descriptors.len() != pool.descriptors.capacity() {
        return true;
    }

    let new_max = (pool.descriptors.capacity() | (pool.descriptors.capacity() >> 2)) + 1;
    pool.descriptors
        .try_reserve_exact(new_max.saturating_sub(pool.descriptors.capacity()))
        .is_ok()
}

fn new_slab(pool: &mut StringPool) -> Option<usize> {
    if !reserve_descriptor_slot(pool) {
        pool.sync_header();
        return None;
    }

    let mut slab = Vec::new();
    if slab.try_reserve_exact(pool.header.max_length).is_err() {
        pool.sync_header();
        return None;
    }
    slab.resize(pool.header.max_length, 0);
    let slab = slab.into_boxed_slice();

    if pool.slabs.try_reserve_exact(1).is_err() {
        pool.sync_header();
        return None;
    }
    if pool.used.try_reserve_exact(1).is_err() {
        pool.sync_header();
        return None;
    }

    let index = pool.slabs.len();
    pool.slabs.push(slab);
    pool.used.push(0);

    pool.sync_header();
    Some(index)
}

fn new_string_pool(pool: &mut StringPool) -> Option<&mut cram_string_alloc_string_t> {
    let index = new_slab(pool)?;
    pool.descriptors.get_mut(index)
}

fn string_alloc(pool: &mut StringPool, length: usize) -> Option<&mut [u8]> {
    if length == 0 {
        return None;
    }

    if let Some(index) = pool.slabs.len().checked_sub(1) {
        let max_length = pool.header.max_length;
        let used = pool.used[index];
        let end = used.checked_add(length)?;

        if end < max_length {
            pool.used[index] = end;
            pool.sync_header();
            return pool.slabs.get_mut(index).map(|slab| &mut slab[used..end]);
        }
    }

    if length > pool.header.max_length {
        pool.header.max_length = length;
    }

    let index = new_slab(pool)?;
    pool.used[index] = length;
    pool.sync_header();
    pool.slabs.get_mut(index).map(|slab| &mut slab[..length])
}

fn string_ndup<'a>(pool: &'a mut StringPool, instr: &[u8]) -> Option<&'a mut [u8]> {
    let out = string_alloc(pool, instr.len().checked_add(1)?)?;

    {
        let (body, terminator) = out.split_at_mut(instr.len());
        body.copy_from_slice(instr);
        terminator[0] = 0;
    }

    Some(out)
}

unsafe fn nul_terminated_bytes<'a>(ptr: *const c_char) -> Option<&'a [u8]> {
    if ptr.is_null() {
        return None;
    }

    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len = len.checked_add(1)?;
    }
    Some(std::slice::from_raw_parts(ptr.cast::<u8>(), len))
}

pub unsafe fn cram_string_alloc_c_55_string_pool_create(
    mut max_length: usize,
) -> *mut cram_string_alloc_t {
    if max_length < CRAM_STRING_ALLOC_MIN_STR_SIZE {
        max_length = CRAM_STRING_ALLOC_MIN_STR_SIZE;
    }

    Box::into_raw(Box::new(StringPool::new(max_length))).cast::<cram_string_alloc_t>()
}

pub unsafe fn cram_string_alloc_c_75_new_string_pool(
    a_str: *mut cram_string_alloc_t,
) -> *mut cram_string_alloc_string_t {
    let Some(pool) = string_pool_mut(a_str) else {
        return std::ptr::null_mut();
    };

    new_string_pool(pool)
        .map(|str_| str_ as *mut cram_string_alloc_string_t)
        .unwrap_or(std::ptr::null_mut())
}

pub unsafe fn cram_string_alloc_c_103_string_pool_destroy(a_str: *mut cram_string_alloc_t) {
    let Some(a_str) = NonNull::new(a_str) else {
        return;
    };

    drop(string_pool_from_header(a_str));
}

pub unsafe fn cram_string_alloc_c_117_string_alloc(
    a_str: *mut cram_string_alloc_t,
    length: usize,
) -> *mut c_char {
    if length == 0 {
        return std::ptr::null_mut();
    }

    let Some(pool) = string_pool_mut(a_str) else {
        return std::ptr::null_mut();
    };

    string_alloc(pool, length)
        .map(|bytes: &mut [u8]| bytes.as_mut_ptr().cast())
        .unwrap_or(std::ptr::null_mut())
}

pub unsafe fn cram_string_alloc_c_149_string_dup(
    a_str: *mut cram_string_alloc_t,
    instr: *const c_char,
) -> *mut c_char {
    let Some(instr) = nul_terminated_bytes(instr) else {
        return std::ptr::null_mut();
    };

    let Some(pool) = string_pool_mut(a_str) else {
        return std::ptr::null_mut();
    };

    string_ndup_ref(pool, instr)
}

pub unsafe fn cram_string_alloc_c_153_string_ndup(
    a_str: *mut cram_string_alloc_t,
    instr: *const c_char,
    len: usize,
) -> *mut c_char {
    let Some(instr) =
        (!instr.is_null()).then(|| std::slice::from_raw_parts(instr.cast::<u8>(), len))
    else {
        return std::ptr::null_mut();
    };

    let Some(pool) = string_pool_mut(a_str) else {
        return std::ptr::null_mut();
    };

    string_ndup_ref(pool, instr)
}

fn string_ndup_ref(pool: &mut StringPool, instr: &[u8]) -> *mut c_char {
    string_ndup(pool, instr)
        .map(|bytes: &mut [u8]| bytes.as_mut_ptr().cast())
        .unwrap_or(std::ptr::null_mut())
}
