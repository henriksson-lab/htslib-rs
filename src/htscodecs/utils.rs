//! Translation of htscodecs `utils.c` + `utils.h`.
//!
//! Thread-local storage pool helpers and histogram / transpose utilities.

use crate::c_compat;
use core::ffi::c_void;
use std::cell::RefCell;

// ----------------------------------------------------------------------------
// utils.h: macros and constants
// ----------------------------------------------------------------------------

// utils.h:140
pub const MAGIC: usize = 8;

/// ```c
/// #define likely(x)      __builtin_expect((x), 1)
/// ```
// utils.h:45/48/52
#[inline]
pub fn likely(x: bool) -> bool {
    x
}

/// ```c
/// #define unlikely(x)     __builtin_expect((x), 0)
/// ```
// utils.h:50/53
#[inline]
pub fn unlikely(x: bool) -> bool {
    x
}

// ----------------------------------------------------------------------------
// utils.c: thread-local storage pool
// ----------------------------------------------------------------------------

/// ```c
/// #define MAX_TLS_BUFS 10
/// ```
// utils.c:69
pub const MAX_TLS_BUFS: usize = 10;

/// ```c
/// typedef struct {
///     void   *bufs[MAX_TLS_BUFS];
///     size_t sizes[MAX_TLS_BUFS];
///     int     used[MAX_TLS_BUFS];
/// } tls_pool;
/// ```
// utils.c:70
#[repr(C)]
pub struct tls_pool {
    pub bufs: [*mut c_void; MAX_TLS_BUFS],
    pub sizes: [usize; MAX_TLS_BUFS],
    pub used: [i32; MAX_TLS_BUFS],
}

impl tls_pool {
    const fn new() -> Self {
        tls_pool {
            bufs: [core::ptr::null_mut(); MAX_TLS_BUFS],
            sizes: [0; MAX_TLS_BUFS],
            used: [0; MAX_TLS_BUFS],
        }
    }
}

// In C the per-thread pool lives behind a pthread_key whose destructor
// (`htscodecs_tls_free_all`) runs when the thread exits.  The faithful Rust
// equivalent is a `thread_local!` holding the same `tls_pool`; the cell's own
// `Drop` is the destructor.  We wrap the pool in a small newtype so a `Drop`
// impl mirrors `htscodecs_tls_free_all`.
struct TlsCell(tls_pool);

impl Drop for TlsCell {
    fn drop(&mut self) {
        // utils.c:83 htscodecs_tls_free_all
        htscodecs_tls_free_all(&mut self.0 as *mut tls_pool as *mut c_void);
        // Null the released pointers so they can't be seen/freed again.
        for b in self.0.bufs.iter_mut() {
            *b = core::ptr::null_mut();
        }
    }
}

thread_local! {
    static RANS_TLS: RefCell<TlsCell> = const { RefCell::new(TlsCell(tls_pool::new())) };
}

/// ```c
/// static void htscodecs_tls_free_all(void *ptr) {
///     tls_pool *tls = (tls_pool *)ptr;
///     if (!tls)
///         return;
///     int i;
///     for (i = 0; i < MAX_TLS_BUFS; i++) {
///         if (tls->used[i]) {
///             fprintf(stderr, "Closing thread while TLS data is in use\n");
///         }
///         free(tls->bufs[i]);
///     }
///     free(tls);
/// }
/// ```
/// Frees all local storage for a thread (pthread_key destructor).
///
/// Note: unlike the C version we do *not* `free(tls)` itself: the `tls_pool`
/// here is owned by the `thread_local!` cell, not separately heap-allocated, so
/// freeing the buffers is the analogous step.
// utils.c:83
pub fn htscodecs_tls_free_all(ptr: *mut c_void) {
    let tls = ptr as *mut tls_pool;
    if tls.is_null() {
        return;
    }
    let tls = unsafe { &mut *tls };

    for i in 0..MAX_TLS_BUFS {
        if tls.used[i] != 0 {
            eprintln!("Closing thread while TLS data is in use");
        }
        unsafe { c_compat::free(tls.bufs[i]) };
    }
    // (No `free(tls)`: the pool is owned by the thread_local cell.)
}

/// ```c
/// static void htscodecs_tls_init(void) {
///     pthread_key_create(&rans_key, htscodecs_tls_free_all);
/// }
/// ```
///
/// In C this is run once via `pthread_once` to install the key + destructor.
/// In the Rust translation the per-thread pool and its destructor are provided
/// directly by `thread_local!`/`Drop`, so initialisation is implicit and this
/// function is a no-op kept for 1:1 naming.
// utils.c:103
pub fn htscodecs_tls_init() {}

/// ```c
/// void *htscodecs_tls_alloc(size_t size) {
///     ...
///     tls_pool *tls = pthread_getspecific(rans_key);
///     ...
///     int avail = -1;
///     for (i = 0; i < MAX_TLS_BUFS; i++) {
///         if (!tls->used[i]) {
///             if (size <= tls->sizes[i]) {
///                 tls->used[i] = 1;
///                 return tls->bufs[i];
///             } else if (avail == -1) {
///                 avail = i;
///             }
///         }
///     }
///     if (i == MAX_TLS_BUFS && avail == -1) { ... return NULL; }
///     if (tls->bufs[avail]) free(tls->bufs[avail]);
///     if (!(tls->bufs[avail] = calloc(1, size))) return NULL;
///     tls->sizes[avail] = size;
///     tls->used[avail] = 1;
///     return tls->bufs[avail];
/// }
/// ```
// utils.c:119 (also utils.h:63)
pub fn htscodecs_tls_alloc(size: usize) -> *mut c_void {
    htscodecs_tls_init();

    RANS_TLS.with(|cell| {
        let mut cell = cell.borrow_mut();
        let tls = &mut cell.0;

        // Query pool for size
        let mut avail: i32 = -1;
        for i in 0..MAX_TLS_BUFS {
            if tls.used[i] == 0 {
                if size <= tls.sizes[i] {
                    tls.used[i] = 1;
                    return tls.bufs[i];
                } else if avail == -1 {
                    avail = i as i32;
                }
            }
        }

        if avail == -1 {
            // Shouldn't happen given our very limited use of this function
            eprintln!("Error: out of rans_tls_alloc slots");
            return core::ptr::null_mut();
        }

        let avail = avail as usize;
        if !tls.bufs[avail].is_null() {
            unsafe { c_compat::free(tls.bufs[avail]) };
        }
        let p = unsafe { c_compat::calloc(1, size as u64) };
        if p.is_null() {
            // C leaves tls->bufs[avail] dangling here; we null it so a later
            // alloc / free_all won't double-free.
            tls.bufs[avail] = core::ptr::null_mut();
            tls.sizes[avail] = 0;
            return core::ptr::null_mut();
        }
        tls.bufs[avail] = p;
        tls.sizes[avail] = size;
        tls.used[avail] = 1;

        tls.bufs[avail]
    })
}

/// ```c
/// void *htscodecs_tls_calloc(size_t nmemb, size_t size) {
///     void *ptr = htscodecs_tls_alloc(nmemb * size);
///     if (ptr)
///         memset(ptr, 0, nmemb * size);
///     return ptr;
/// }
/// ```
// utils.c:173 (also utils.h:64)
pub fn htscodecs_tls_calloc(nmemb: usize, size: usize) -> *mut c_void {
    let ptr = htscodecs_tls_alloc(nmemb * size);
    if !ptr.is_null() {
        unsafe {
            core::ptr::write_bytes(ptr as *mut u8, 0, nmemb * size);
        }
    }
    ptr
}

/// ```c
/// void htscodecs_tls_free(void *ptr) {
///     if (!ptr)
///         return;
///     tls_pool *tls = pthread_getspecific(rans_key);
///     int i;
///     for (i = 0; i < MAX_TLS_BUFS; i++) {
///         if (tls->bufs[i] == ptr)
///             break;
///     }
///     if (i == MAX_TLS_BUFS) { ...not allocated... return; }
///     if (!tls->used[i]) { ...freed twice... return; }
///     tls->used[i] = 0;
/// }
/// ```
// utils.c:183 (also utils.h:65)
pub fn htscodecs_tls_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }

    RANS_TLS.with(|cell| {
        let mut cell = cell.borrow_mut();
        let tls = &mut cell.0;

        let mut i = MAX_TLS_BUFS;
        for k in 0..MAX_TLS_BUFS {
            if tls.bufs[k] == ptr {
                i = k;
                break;
            }
        }
        if i == MAX_TLS_BUFS {
            eprintln!(
                "Attempt to htscodecs_tls_free a buffer not allocated with htscodecs_tls_alloc"
            );
            return;
        }
        if tls.used[i] == 0 {
            eprintln!("Attempt to htscodecs_tls_free a buffer twice");
            return;
        }
        tls.used[i] = 0;
    });
}

// ----------------------------------------------------------------------------
// utils.h: inline functions
// ----------------------------------------------------------------------------

/// ```c
/// static inline double fast_log(double a) {
///   union { double d; long long x; } u = { a };
///   return (u.x - 4606921278410026770) * 1.539095918623324e-16;
/// }
/// ```
/// Fast approximate log base 2.
// utils.h:69
#[inline]
pub fn fast_log(a: f64) -> f64 {
    let x = a.to_bits() as i64;
    (x - 4606921278410026770i64) as f64 * 1.539095918623324e-16
}

/// ```c
/// static inline void unstripe(unsigned char *out, unsigned char *outN,
///                             unsigned int ulen, unsigned int N,
///                             unsigned int idxN[256]);
/// ```
/// Data transpose by N; common to rANS4x16 and arith_dynamic decoders.
// utils.h:79
pub fn unstripe(out: &mut [u8], outN: &[u8], ulen: u32, N: u32, idxN: &mut [u32; 256]) {
    let ulen = ulen as usize;
    let n = N as usize;
    let mut j: usize = 0;

    if ulen >= n {
        match N {
            4 => {
                const LLN: usize = 16;
                if ulen >= 4 * LLN {
                    while j < ulen - 4 * LLN {
                        for l in 0..LLN {
                            for (k, idx) in idxN.iter().take(4).enumerate() {
                                out[j + k + l * 4] = outN[*idx as usize + l];
                            }
                        }
                        for idx in idxN.iter_mut().take(4) {
                            *idx += LLN as u32;
                        }
                        j += 4 * LLN;
                    }
                }
                while j < ulen - 4 {
                    for idx in idxN.iter_mut().take(4) {
                        out[j] = outN[*idx as usize];
                        *idx += 1;
                        j += 1;
                    }
                }
            }
            2 => {
                const LLN: usize = 4;
                if ulen >= 2 * LLN {
                    while j < ulen - 2 * LLN {
                        let mut l: usize = 0;
                        while l < LLN {
                            for idx in idxN.iter().take(2) {
                                out[j] = outN[*idx as usize + l];
                                j += 1;
                            }
                            l += 1;
                        }
                        for idx in idxN.iter_mut().take(2) {
                            *idx += l as u32;
                        }
                    }
                }
                while j < ulen - 2 {
                    for idx in idxN.iter_mut().take(2) {
                        out[j] = outN[*idx as usize];
                        *idx += 1;
                        j += 1;
                    }
                }
            }
            _ => {
                // General case, around 25% slower overall decode
                while j < ulen - n {
                    for k in 0..n {
                        out[j] = outN[idxN[k] as usize];
                        idxN[k] += 1;
                        j += 1;
                    }
                }
            }
        }
    }

    let mut k: usize = 0;
    while j < ulen {
        out[j] = outN[idxN[k] as usize];
        idxN[k] += 1;
        j += 1;
        k += 1;
    }
}

/// ```c
/// static inline
/// int hist8(unsigned char *in, unsigned int in_size, uint32_t F0[256]);
/// ```
/// Order-0 histogram construction. Returns 0 on success, -1 on alloc failure.
// utils.h:146
pub fn hist8(r#in: &[u8], in_size: u32, F0: &mut [u32; 256]) -> i32 {
    let in_size = in_size as usize;

    if in_size > 500000 {
        let f0p = htscodecs_tls_calloc((65536 + 37) * 3, core::mem::size_of::<u32>());
        if f0p.is_null() {
            return -1;
        }
        // f0 / f1 / f2 are the three contiguous (65536+37)-element regions.
        let base = f0p as *mut u32;
        let f0 = unsafe { core::slice::from_raw_parts_mut(base, 65536 + 37) };
        let f1 = unsafe { core::slice::from_raw_parts_mut(base.add(65536 + 37), 65536 + 37) };
        let f2 = unsafe { core::slice::from_raw_parts_mut(base.add((65536 + 37) * 2), 65536 + 37) };

        let i8 = in_size & !15;
        let mut i = 0;
        while i < i8 {
            // memcpy(i16a, in+i, 8): 4 little-endian u16s
            let i16a = read_u16x4(&r#in[i..i + 8]);
            f0[i16a[0] as usize] += 1;
            f1[i16a[1] as usize] += 1;
            f2[i16a[2] as usize] += 1;
            f0[i16a[3] as usize] += 1;

            let i16b = read_u16x4(&r#in[i + 8..i + 16]);
            f1[i16b[0] as usize] += 1;
            f0[i16b[1] as usize] += 1;
            f1[i16b[2] as usize] += 1;
            f2[i16b[3] as usize] += 1;

            i += 16;
        }

        while i < in_size {
            F0[r#in[i] as usize] += 1;
            i += 1;
        }

        for i in 0..65536usize {
            let s = f0[i] + f1[i] + f2[i];
            F0[i & 0xff] += s;
            F0[i >> 8] += s;
        }
        htscodecs_tls_free(f0p);
    } else {
        let mut F1 = [0u32; 256 + MAGIC];
        let mut F2 = [0u32; 256 + MAGIC];
        let mut F3 = [0u32; 256 + MAGIC];
        let i8 = in_size & !7;

        let mut i = 0;
        while i < i8 {
            F0[r#in[i] as usize] += 1;
            F1[r#in[i + 1] as usize] += 1;
            F2[r#in[i + 2] as usize] += 1;
            F3[r#in[i + 3] as usize] += 1;
            F0[r#in[i + 4] as usize] += 1;
            F1[r#in[i + 5] as usize] += 1;
            F2[r#in[i + 6] as usize] += 1;
            F3[r#in[i + 7] as usize] += 1;
            i += 8;
        }

        while i < in_size {
            F0[r#in[i] as usize] += 1;
            i += 1;
        }

        for i in 0..256usize {
            F0[i] += F1[i] + F2[i] + F3[i];
        }
    }

    0
}

/// Helper mirroring `memcpy(i16, in, 8)` to read four little-endian `u16`.
#[inline]
fn read_u16x4(b: &[u8]) -> [u16; 4] {
    [
        u16::from_le_bytes([b[0], b[1]]),
        u16::from_le_bytes([b[2], b[3]]),
        u16::from_le_bytes([b[4], b[5]]),
        u16::from_le_bytes([b[6], b[7]]),
    ]
}

/// ```c
/// static inline
/// double hist8e(unsigned char *in, unsigned int in_size, uint32_t F0[256]);
/// ```
/// hist8 with a crude entropy (bits/byte) estimator.
// utils.h:206
pub fn hist8e(r#in: &[u8], in_size: u32, F0: &mut [u32; 256]) -> f64 {
    let in_size = in_size as usize;
    let mut F1 = [0u32; 256 + MAGIC];
    let mut F2 = [0u32; 256 + MAGIC];
    let mut F3 = [0u32; 256 + MAGIC];
    let mut F4 = [0u32; 256 + MAGIC];
    let mut F5 = [0u32; 256 + MAGIC];
    let mut F6 = [0u32; 256 + MAGIC];
    let mut F7 = [0u32; 256 + MAGIC];

    // __GNUC__ path: in_size_r2 = log2(1/in_size)
    let mut e: f64 = 0.0;
    let in_size_r2 = (1.0f64 / in_size as f64).ln() / 2.0f64.ln();

    let i8 = in_size & !7;
    let mut i = 0;
    while i < i8 {
        F0[r#in[i] as usize] += 1;
        F1[r#in[i + 1] as usize] += 1;
        F2[r#in[i + 2] as usize] += 1;
        F3[r#in[i + 3] as usize] += 1;
        F4[r#in[i + 4] as usize] += 1;
        F5[r#in[i + 5] as usize] += 1;
        F6[r#in[i + 6] as usize] += 1;
        F7[r#in[i + 7] as usize] += 1;
        i += 8;
    }
    while i < in_size {
        F0[r#in[i] as usize] += 1;
        i += 1;
    }

    for i in 0..256usize {
        F0[i] += F1[i] + F2[i] + F3[i] + F4[i] + F5[i] + F6[i] + F7[i];
        // __GNUC__ path: e -= F0[i] * (32 - __builtin_clz(F0[i]|1) + in_size_r2)
        let clz = (F0[i] | 1).leading_zeros() as i32;
        e -= F0[i] as f64 * ((32 - clz) as f64 + in_size_r2);
    }

    e / in_size as f64
}

/// ```c
/// static inline
/// void present8(unsigned char *in, unsigned int in_size, uint32_t F0[256]);
/// ```
/// A variant of hist8 that marks symbol presence rather than frequency.
// utils.h:251
pub fn present8(r#in: &[u8], in_size: u32, F0: &mut [u32; 256]) {
    let in_size = in_size as usize;
    let mut F1 = [0u32; 256 + MAGIC];
    let mut F2 = [0u32; 256 + MAGIC];
    let mut F3 = [0u32; 256 + MAGIC];
    let mut F4 = [0u32; 256 + MAGIC];
    let mut F5 = [0u32; 256 + MAGIC];
    let mut F6 = [0u32; 256 + MAGIC];
    let mut F7 = [0u32; 256 + MAGIC];

    let i8 = in_size & !7;
    let mut i = 0;
    while i < i8 {
        F0[r#in[i] as usize] = 1;
        F1[r#in[i + 1] as usize] = 1;
        F2[r#in[i + 2] as usize] = 1;
        F3[r#in[i + 3] as usize] = 1;
        F4[r#in[i + 4] as usize] = 1;
        F5[r#in[i + 5] as usize] = 1;
        F6[r#in[i + 6] as usize] = 1;
        F7[r#in[i + 7] as usize] = 1;
        i += 8;
    }
    while i < in_size {
        F0[r#in[i] as usize] = 1;
        i += 1;
    }

    for i in 0..256usize {
        F0[i] += F1[i] + F2[i] + F3[i] + F4[i] + F5[i] + F6[i] + F7[i];
    }
}

/// ```c
/// static inline
/// int hist1_4(unsigned char *in, unsigned int in_size,
///             uint32_t F0[256][256], uint32_t *T0);
/// ```
/// Order-1 histogram construction. Returns 0 on success, -1 on alloc failure.
// utils.h:280
pub fn hist1_4(r#in: &[u8], in_size: u32, F0: &mut [[u32; 256]; 256], T0: &mut [u32]) -> i32 {
    let in_size = in_size as usize;
    let mut l: u8;

    // cc[5] = {0}; memcpy(cc, in, 4) refreshes cc[0..4], cc[4] chains across.
    let mut cc: [u8; 5] = [0; 5];
    // Cursor into `in`, mirroring C's `in`/`in_end` pointers.  The C loop
    // condition `in < in_end-8` corresponds to `pos + 8 < in_size`; when
    // in_size <= 8 the C pointer `in_end-8` is before `in`, so the loop never
    // runs.
    let mut pos: usize = 0;

    if in_size > 500000 {
        // uint32_t (*F1)[259] = calloc(256, sizeof *F1)
        const STRIDE: usize = 259;
        let f1p = htscodecs_tls_calloc(256, STRIDE * core::mem::size_of::<u32>());
        if f1p.is_null() {
            return -1;
        }
        let f1base = f1p as *mut u32;
        // F1[a][b] == f1base[a*259 + b]
        macro_rules! f1 {
            ($a:expr, $b:expr) => {
                unsafe { &mut *f1base.add(($a) * STRIDE + ($b)) }
            };
        }

        while pos + 8 < in_size {
            cc[0] = r#in[pos];
            cc[1] = r#in[pos + 1];
            cc[2] = r#in[pos + 2];
            cc[3] = r#in[pos + 3];
            pos += 4;
            F0[cc[4] as usize][cc[0] as usize] += 1;
            *f1!(cc[0] as usize, cc[1] as usize) += 1;
            F0[cc[1] as usize][cc[2] as usize] += 1;
            *f1!(cc[2] as usize, cc[3] as usize) += 1;
            cc[4] = cc[3];

            cc[0] = r#in[pos];
            cc[1] = r#in[pos + 1];
            cc[2] = r#in[pos + 2];
            cc[3] = r#in[pos + 3];
            pos += 4;
            F0[cc[4] as usize][cc[0] as usize] += 1;
            *f1!(cc[0] as usize, cc[1] as usize) += 1;
            F0[cc[1] as usize][cc[2] as usize] += 1;
            *f1!(cc[2] as usize, cc[3] as usize) += 1;
            cc[4] = cc[3];
        }
        l = cc[3];

        while pos < in_size {
            let c = r#in[pos];
            pos += 1;
            F0[l as usize][c as usize] += 1;
            l = c;
        }
        T0[l as usize] += 1;

        for i in 0..256usize {
            let mut tt: i32 = 0;
            for (j, val) in F0[i].iter_mut().enumerate() {
                *val += *f1!(i, j);
                tt += *val as i32;
            }
            T0[i] += tt as u32;
        }
        htscodecs_tls_free(f1p);
    } else {
        while pos + 8 < in_size {
            cc[0] = r#in[pos];
            cc[1] = r#in[pos + 1];
            cc[2] = r#in[pos + 2];
            cc[3] = r#in[pos + 3];
            pos += 4;
            F0[cc[4] as usize][cc[0] as usize] += 1;
            F0[cc[0] as usize][cc[1] as usize] += 1;
            F0[cc[1] as usize][cc[2] as usize] += 1;
            F0[cc[2] as usize][cc[3] as usize] += 1;
            cc[4] = cc[3];

            cc[0] = r#in[pos];
            cc[1] = r#in[pos + 1];
            cc[2] = r#in[pos + 2];
            cc[3] = r#in[pos + 3];
            pos += 4;
            F0[cc[4] as usize][cc[0] as usize] += 1;
            F0[cc[0] as usize][cc[1] as usize] += 1;
            F0[cc[1] as usize][cc[2] as usize] += 1;
            F0[cc[2] as usize][cc[3] as usize] += 1;
            cc[4] = cc[3];
        }
        l = cc[3];

        while pos < in_size {
            let c = r#in[pos];
            pos += 1;
            F0[l as usize][c as usize] += 1;
            l = c;
        }
        T0[l as usize] += 1;

        for i in 0..256usize {
            let mut tt: i32 = 0;
            for val in F0[i].iter() {
                tt += *val as i32;
            }
            T0[i] += tt as u32;
        }
    }

    0
}

#[cfg(test)]
mod tests;
