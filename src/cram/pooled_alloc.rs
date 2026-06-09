// Functions translated from htslib/cram/pooled_alloc.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_int, c_void};
use std::mem::size_of;
use std::ptr::NonNull;

use super::*;

#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct PoolStorageChunk([usize; 2]);

struct PoolStorage {
    ptr: NonNull<PoolStorageChunk>,
    chunks: usize,
}

impl PoolStorage {
    fn new(size: usize) -> Option<Self> {
        let chunk_size = size_of::<PoolStorageChunk>();
        let chunks = (size + chunk_size - 1) / chunk_size;
        let mut storage = Vec::<PoolStorageChunk>::new();
        if storage.try_reserve_exact(chunks).is_err() {
            return None;
        }
        storage.resize(chunks, PoolStorageChunk([0; 2]));

        let mut storage = storage.into_boxed_slice();
        let ptr = NonNull::new(storage.as_mut_ptr())?;
        let _ = Box::into_raw(storage);
        Some(Self { ptr, chunks })
    }

    unsafe fn from_raw(ptr: NonNull<u8>, size: usize) -> Option<Self> {
        let ptr = ptr.cast::<PoolStorageChunk>();
        let chunk_size = size_of::<PoolStorageChunk>();
        let chunks = (size + chunk_size - 1) / chunk_size;
        Some(Self { ptr, chunks })
    }

    fn as_ptr(&self) -> NonNull<u8> {
        self.ptr.cast()
    }

    fn into_ptr(self) -> NonNull<u8> {
        let ptr = self.as_ptr();
        std::mem::forget(self);
        ptr
    }

    unsafe fn byte_at(&self, offset: usize) -> Option<NonNull<u8>> {
        NonNull::new(self.ptr.as_ptr().cast::<u8>().add(offset))
    }
}

impl Drop for PoolStorage {
    fn drop(&mut self) {
        unsafe {
            let storage = std::ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), self.chunks);
            drop(Box::from_raw(storage));
        }
    }
}

#[derive(Clone, Copy)]
struct FreeSlot(NonNull<u8>);

impl FreeSlot {
    fn from_raw(ptr: *mut c_void) -> Option<Self> {
        NonNull::new(ptr.cast::<u8>()).map(Self)
    }

    fn from_nonnull(ptr: NonNull<u8>) -> Self {
        Self(ptr)
    }

    unsafe fn next(self) -> Option<Self> {
        Self::from_raw(*(self.0.as_ptr().cast::<*mut c_void>()))
    }

    unsafe fn set_next(self, next: Option<Self>) {
        *(self.0.as_ptr().cast::<*mut c_void>()) =
            next.map_or(std::ptr::null_mut(), FreeSlot::as_void_ptr);
    }

    fn as_void_ptr(self) -> *mut c_void {
        self.0.as_ptr().cast()
    }
}

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

pub unsafe fn pool_create(dsize: usize) -> Box<pool_alloc_t> {
    let mut rounded = (dsize + size_of::<*mut c_void>() - 1) & !(size_of::<*mut c_void>() - 1);
    if rounded < size_of::<*mut c_void>() {
        rounded = size_of::<*mut c_void>();
    }

    Box::new(pool_alloc_t {
        dsize: rounded,
        psize: std::cmp::min(
            POOLED_ALLOC_PSIZE,
            cram_pooled_alloc_c_47_next_power_2((rounded * 1024) as u32) as usize,
        ),
        npools: 0,
        pools: std::ptr::null_mut(),
        free: std::ptr::null_mut(),
        pools_storage: Vec::new(),
    })
}

pub unsafe fn cram_pooled_alloc_c_64_pool_create(dsize: usize) -> *mut pool_alloc_t {
    Box::into_raw(pool_create(dsize))
}

unsafe fn pool_capacity_bytes(p: &pool_alloc_t) -> usize {
    let n = p.psize / p.dsize;
    n * p.dsize
}

unsafe fn pool_storage_from_raw(ptr: *mut c_void, size: usize) -> Option<PoolStorage> {
    PoolStorage::from_raw(NonNull::new(ptr.cast::<u8>())?, size)
}

fn sync_pool_table(p: &mut pool_alloc_t) {
    p.npools = p.pools_storage.len();
    p.pools = p
        .pools_storage
        .first_mut()
        .map_or(std::ptr::null_mut(), |pool| pool as *mut pool_t);
}

pub unsafe fn pool_destroy(p: &mut pool_alloc_t) {
    let size = pool_capacity_bytes(p);
    for pool in p.pools_storage.drain(..) {
        if let Some(storage) = pool_storage_from_raw(pool.pool, size) {
            drop(storage);
        }
    }
    sync_pool_table(p);
}

pub unsafe fn pool_destroy_box(mut p: Box<pool_alloc_t>) {
    pool_destroy(&mut p);
}

pub unsafe fn cram_pooled_alloc_c_84_pool_destroy(p: *mut pool_alloc_t) {
    if p.is_null() {
        return;
    }
    pool_destroy_box(Box::from_raw(p));
}

pub unsafe fn new_pool(p: &mut pool_alloc_t) -> Option<&mut pool_t> {
    let n = p.psize / p.dsize;
    if n == 0 {
        return None;
    }

    let size = n * p.dsize;
    let storage = PoolStorage::new(size)?;

    if p.pools_storage.try_reserve_exact(1).is_err() {
        return None;
    }

    p.pools_storage.push(pool_t {
        pool: storage.into_ptr().as_ptr().cast(),
        used: 0,
    });
    sync_pool_table(p);

    p.pools_storage.last_mut()
}

pub unsafe fn cram_pooled_alloc_c_96_new_pool(p: *mut pool_alloc_t) -> *mut pool_t {
    NonNull::new(p)
        .and_then(|mut p| new_pool(p.as_mut()))
        .map_or(std::ptr::null_mut(), |pool| pool as *mut pool_t)
}

pub unsafe fn pool_alloc(p: &mut pool_alloc_t) -> Option<NonNull<u8>> {
    if let Some(ret) = FreeSlot::from_raw(p.free) {
        p.free = ret
            .next()
            .map_or(std::ptr::null_mut(), FreeSlot::as_void_ptr);
        return Some(ret.0);
    }

    let capacity_bytes = pool_capacity_bytes(p);
    if let Some(pool) = p
        .npools
        .checked_sub(1)
        .and_then(|last| p.pools_storage.get_mut(last))
    {
        if pool.used + p.dsize < p.psize {
            let Some(storage) = pool_storage_from_raw(pool.pool, capacity_bytes) else {
                return None;
            };
            let Some(ret) = storage.byte_at(pool.used) else {
                return None;
            };
            std::mem::forget(storage);
            pool.used += p.dsize;
            return Some(ret);
        }
    }

    let dsize = p.dsize;
    let Some(pool) = new_pool(p) else {
        return None;
    };
    pool.used = dsize;
    NonNull::new(pool.pool.cast::<u8>())
}

pub unsafe fn cram_pooled_alloc_c_115_pool_alloc(p: *mut pool_alloc_t) -> *mut c_void {
    NonNull::new(p)
        .and_then(|mut p| pool_alloc(p.as_mut()))
        .map_or(std::ptr::null_mut(), |ptr| ptr.as_ptr().cast())
}

pub unsafe fn cram_pooled_alloc_c_144_pool_free(p: *mut pool_alloc_t, ptr: *mut c_void) {
    let Some(mut p) = NonNull::new(p) else {
        return;
    };
    let Some(ptr) = FreeSlot::from_raw(ptr) else {
        return;
    };
    pool_free_slot(p.as_mut(), ptr);
}

unsafe fn pool_free_slot(p: &mut pool_alloc_t, ptr: FreeSlot) {
    ptr.set_next(FreeSlot::from_raw(p.free));
    p.free = ptr.as_void_ptr();
}

pub unsafe fn pool_free(p: &mut pool_alloc_t, ptr: NonNull<u8>) {
    pool_free_slot(p, FreeSlot::from_nonnull(ptr));
}

pub unsafe fn standalone_pool_alloc(p: &pool_alloc_t) -> Option<NonNull<u8>> {
    PoolStorage::new(p.dsize).map(PoolStorage::into_ptr)
}

pub unsafe fn cram_pooled_alloc_c_151_pool_alloc(p: *mut pool_alloc_t) -> *mut c_void {
    NonNull::new(p)
        .and_then(|p| standalone_pool_alloc(p.as_ref()))
        .map_or(std::ptr::null_mut(), |ptr| ptr.as_ptr().cast())
}

pub unsafe fn cram_pooled_alloc_c_155_pool_free(p: *mut pool_alloc_t, ptr: *mut c_void) {
    let Some(p) = NonNull::new(p) else {
        return;
    };
    if let Some(storage) = pool_storage_from_raw(ptr, p.as_ref().dsize) {
        drop(storage);
    }
}

pub unsafe fn cram_pooled_alloc_c_167_main() -> c_int {
    let p = cram_pooled_alloc_c_64_pool_create(std::mem::size_of::<pooled_alloc_test_xyz>());
    if p.is_null() {
        return 1;
    }

    let np = 10000usize;
    let mut items = Vec::<*mut pooled_alloc_test_xyz>::new();
    if items.try_reserve_exact(np).is_err() {
        cram_pooled_alloc_c_84_pool_destroy(p);
        return 1;
    }

    for i in 0..np {
        let item = cram_pooled_alloc_c_115_pool_alloc(p).cast::<pooled_alloc_test_xyz>();
        if item.is_null() {
            cram_pooled_alloc_c_84_pool_destroy(p);
            return 1;
        }
        (*item).x = i as c_int;
        (*item).y = i as c_int + 1;
        (*item).z = i as c_int + 2;
        items.push(item);
    }

    for (i, &item) in items.iter().enumerate() {
        if i % 3 != 0 {
            cram_pooled_alloc_c_144_pool_free(p, item.cast());
        }
    }

    for i in 0..np {
        let item = cram_pooled_alloc_c_115_pool_alloc(p).cast::<pooled_alloc_test_xyz>();
        if item.is_null() {
            cram_pooled_alloc_c_84_pool_destroy(p);
            return 1;
        }
        (*item).x = 1_000_000 + i as c_int;
        (*item).y = 1_000_000 + i as c_int + 1;
        (*item).z = 1_000_000 + i as c_int + 2;
    }

    for item in items {
        cram_pooled_alloc_c_144_pool_free(p, item.cast());
    }

    cram_pooled_alloc_c_84_pool_destroy(p);
    0
}
