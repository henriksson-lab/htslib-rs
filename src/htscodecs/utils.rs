//! Translation of htscodecs `utils.c` + `utils.h`.
//!
//! Thread-local storage pool helpers and histogram / transpose utilities.

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
pub struct tls_pool {
    slots: [TlsSlot; MAX_TLS_BUFS],
}

struct TlsSlot {
    buf: Option<Vec<u8>>,
    used: bool,
}

impl TlsSlot {
    const fn new() -> Self {
        Self {
            buf: None,
            used: false,
        }
    }

    fn capacity(&self) -> usize {
        self.buf.as_ref().map_or(0, Vec::len)
    }
}

impl tls_pool {
    const fn new() -> Self {
        tls_pool {
            slots: [
                TlsSlot::new(),
                TlsSlot::new(),
                TlsSlot::new(),
                TlsSlot::new(),
                TlsSlot::new(),
                TlsSlot::new(),
                TlsSlot::new(),
                TlsSlot::new(),
                TlsSlot::new(),
                TlsSlot::new(),
            ],
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
        htscodecs_tls_free_pool(&mut self.0);
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
/// freeing the buffers is the analogous step.  The C version took a `void*` that
/// could be NULL; the Rust translation takes an owning `&mut tls_pool` directly.
// utils.c:83
pub fn htscodecs_tls_free_all(tls: &mut tls_pool) {
    htscodecs_tls_free_pool(tls);
}

fn htscodecs_tls_free_pool(tls: &mut tls_pool) {
    for slot in tls.slots.iter_mut() {
        if slot.used {
            eprintln!("Closing thread while TLS data is in use");
        }
        slot.buf = None;
        slot.used = false;
    }
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
/// Reserves a thread-local scratch buffer of at least `size` bytes and returns
/// its slot index (0..MAX_TLS_BUFS).  In the C version this returned the raw
/// `void*` buffer pointer; here we return an `Option<usize>` slot handle (with
/// `None` standing in for the C `NULL` failure path), and the buffer itself is
/// reached through `htscodecs_tls_with`.
// utils.c:119 (also utils.h:63)
pub fn htscodecs_tls_alloc(size: usize) -> Option<usize> {
    htscodecs_tls_init();

    RANS_TLS.with(|cell| {
        let mut cell = cell.borrow_mut();
        let tls = &mut cell.0;

        // Query pool for size
        let mut avail = None;
        for (i, slot) in tls.slots.iter_mut().enumerate() {
            if !slot.used {
                if slot.buf.is_some() && size <= slot.capacity() {
                    slot.used = true;
                    return Some(i);
                } else if avail.is_none() {
                    avail = Some(i);
                }
            }
        }

        let Some(avail) = avail else {
            // Shouldn't happen given our very limited use of this function
            eprintln!("Error: out of rans_tls_alloc slots");
            return None;
        };

        let slot = &mut tls.slots[avail];
        let mut buf = Vec::new();
        if buf.try_reserve_exact(size).is_err() {
            slot.buf = None;
            slot.used = false;
            return None;
        }
        buf.resize(size, 0);
        slot.buf = Some(buf);
        slot.used = true;

        Some(avail)
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
pub fn htscodecs_tls_calloc(nmemb: usize, size: usize) -> Option<usize> {
    let n = nmemb * size;
    let idx = htscodecs_tls_alloc(n)?;
    RANS_TLS.with(|cell| {
        let mut cell = cell.borrow_mut();
        if let Some(buf) = cell.0.slots[idx].buf.as_mut() {
            buf[..n].fill(0);
        }
    });
    Some(idx)
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
/// Releases a thread-local slot previously handed out by
/// `htscodecs_tls_alloc`/`htscodecs_tls_calloc`.  The C version matched the raw
/// `void*` against the pool's stored pointers; here the caller passes back the
/// slot index directly.
// utils.c:183 (also utils.h:65)
pub fn htscodecs_tls_free(idx: usize) {
    RANS_TLS.with(|cell| {
        let mut cell = cell.borrow_mut();
        let tls = &mut cell.0;

        let Some(slot) = tls.slots.get_mut(idx) else {
            eprintln!(
                "Attempt to htscodecs_tls_free a buffer not allocated with htscodecs_tls_alloc"
            );
            return;
        };
        if slot.buf.is_none() {
            eprintln!(
                "Attempt to htscodecs_tls_free a buffer not allocated with htscodecs_tls_alloc"
            );
            return;
        }
        if !slot.used {
            eprintln!("Attempt to htscodecs_tls_free a buffer twice");
            return;
        }
        slot.used = false;
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
        let scratch_len = (65536 + 37) * 3;
        let mut scratch = Vec::new();
        if scratch.try_reserve_exact(scratch_len).is_err() {
            return -1;
        }
        scratch.resize(scratch_len, 0u32);
        let (f0, rest) = scratch.split_at_mut(65536 + 37);
        let (f1, f2) = rest.split_at_mut(65536 + 37);

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
        let mut f1 = Vec::new();
        if f1.try_reserve_exact(256 * STRIDE).is_err() {
            return -1;
        }
        f1.resize(256 * STRIDE, 0u32);

        while pos + 8 < in_size {
            cc[0] = r#in[pos];
            cc[1] = r#in[pos + 1];
            cc[2] = r#in[pos + 2];
            cc[3] = r#in[pos + 3];
            pos += 4;
            F0[cc[4] as usize][cc[0] as usize] += 1;
            f1[cc[0] as usize * STRIDE + cc[1] as usize] += 1;
            F0[cc[1] as usize][cc[2] as usize] += 1;
            f1[cc[2] as usize * STRIDE + cc[3] as usize] += 1;
            cc[4] = cc[3];

            cc[0] = r#in[pos];
            cc[1] = r#in[pos + 1];
            cc[2] = r#in[pos + 2];
            cc[3] = r#in[pos + 3];
            pos += 4;
            F0[cc[4] as usize][cc[0] as usize] += 1;
            f1[cc[0] as usize * STRIDE + cc[1] as usize] += 1;
            F0[cc[1] as usize][cc[2] as usize] += 1;
            f1[cc[2] as usize * STRIDE + cc[3] as usize] += 1;
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
                *val += f1[i * STRIDE + j];
                tt += *val as i32;
            }
            T0[i] += tt as u32;
        }
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
