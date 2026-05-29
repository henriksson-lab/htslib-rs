//! Translation of htscodecs `rle.c` + `rle.h`.
//!
//! Run length encoding of a byte stream into separate lengths and literals.

use crate::c_compat;

use super::varint::{var_get_u32, var_put_u32};

// rle.c:44
pub const MAGIC: usize = 8;

/// ```c
/// static void rle_find_syms(uint8_t *data, uint64_t data_len,
///                           int64_t *saved, // dim >= 256
///                           uint8_t *rle_syms, int *rle_nsyms);
/// ```
/// Auto-computes `rle_syms` / `rle_nsyms`. `saved` has dimension >= 256.
// rle.c:48
pub fn rle_find_syms(
    data: &[u8],
    data_len: u64,
    saved: &mut [i64],
    rle_syms: &mut [u8],
    rle_nsyms: &mut i32,
) {
    let mut last: i32 = -1;
    let n: i32;
    let mut i: u64;

    if data_len > 256 {
        // 186/450
        // Interleaved buffers to avoid cache collisions
        let mut saved2 = [0i64; 256 + MAGIC];
        let mut saved3 = [0i64; 256 + MAGIC];
        let mut saved4 = [0i64; 256 + MAGIC];
        let len4: u64 = data_len & !3;
        i = 0;
        while i < len4 {
            let idx = i as usize;
            let d1: i64 = ((data[idx] as i32 == last) as i64) << 1;
            let d2: i64 = ((data[idx + 1] == data[idx]) as i64) << 1;
            let d3: i64 = ((data[idx + 2] == data[idx + 1]) as i64) << 1;
            let d4: i64 = ((data[idx + 3] == data[idx + 2]) as i64) << 1;
            last = data[idx + 3] as i32;
            saved[data[idx] as usize] += d1 - 1;
            saved2[data[idx + 1] as usize] += d2 - 1;
            saved3[data[idx + 2] as usize] += d3 - 1;
            saved4[data[idx + 3] as usize] += d4 - 1;
            i += 4;
        }
        while i < data_len {
            let idx = i as usize;
            let d: i64 = ((data[idx] as i32 == last) as i64) << 1;
            saved[data[idx] as usize] += d - 1;
            last = data[idx] as i32;
            i += 1;
        }
        for i in 0..256usize {
            saved[i] += saved2[i] + saved3[i] + saved4[i];
        }
    } else {
        // 163/391
        i = 0;
        while i < data_len {
            let idx = i as usize;
            if data[idx] as i32 == last {
                saved[data[idx] as usize] += 1;
            } else {
                saved[data[idx] as usize] -= 1;
                last = data[idx] as i32;
            }
            i += 1;
        }
    }

    // Map back to a list
    let mut nn: i32 = 0;
    for i in 0..256usize {
        if saved[i] > 0 {
            rle_syms[nn as usize] = i as u8;
            nn += 1;
        }
    }
    n = nn;
    *rle_nsyms = n;
}

/// ```c
/// uint8_t *hts_rle_encode(uint8_t *data, uint64_t data_len,
///                         uint8_t *run,  uint64_t *run_len,
///                         uint8_t *rle_syms, int *rle_nsyms,
///                         uint8_t *out, uint64_t *out_len);
/// ```
/// `out` may be NULL, in which case it is malloc'd (returned as a `Vec<u8>`)
/// and is the caller's to free; otherwise the supplied buffer is written to.
// rle.c:100 (also rle.h:69)
pub fn hts_rle_encode(
    data: &[u8],
    data_len: u64,
    run: &mut [u8],
    run_len: &mut u64,
    rle_syms: &mut [u8],
    rle_nsyms: &mut i32,
    out: Option<&mut [u8]>,
    out_len: &mut u64,
) -> Option<Vec<u8>> {
    // if (!out) if (!(out = malloc(data_len*2))) return NULL;
    //
    // We model the malloc-on-NULL path by allocating a libc buffer and viewing
    // it as a slice. When `out` is supplied (Some) we write into it instead.
    // The result is always returned as an owned Vec (a copy of the used bytes),
    // matching the byte content of the C output buffer.
    let alloc_ptr: *mut u8;
    let out_slice: &mut [u8];
    if let Some(o) = out {
        alloc_ptr = std::ptr::null_mut();
        out_slice = o;
    } else {
        let p = unsafe { c_compat::malloc(data_len.wrapping_mul(2)) } as *mut u8;
        if p.is_null() {
            return None;
        }
        alloc_ptr = p;
        out_slice =
            unsafe { std::slice::from_raw_parts_mut(p, (data_len.wrapping_mul(2)) as usize) };
    }

    // Two pass:  Firstly compute which symbols are worth using RLE on.
    let mut saved = [0i64; 256 + MAGIC];

    if *rle_nsyms != 0 {
        for i in 0..(*rle_nsyms as usize) {
            saved[rle_syms[i] as usize] = 1;
        }
    } else {
        // Writes back to rle_syms and rle_nsyms
        rle_find_syms(data, data_len, &mut saved, rle_syms, rle_nsyms);
    }

    // 2nd pass: perform RLE itself to out[] and run[] arrays.
    let mut i: u64 = 0;
    let mut j: u64 = 0;
    let mut k: u64 = 0;
    while i < data_len {
        out_slice[k as usize] = data[i as usize];
        k += 1;
        if saved[data[i as usize] as usize] > 0 {
            // int rlen = i;  int last = data[i];
            let rlen_start: i32 = i as i32;
            let last: i32 = data[i as usize] as i32;
            while i < data_len && data[i as usize] as i32 == last {
                i += 1;
            }
            i -= 1;
            let rlen: i32 = i as i32 - rlen_start;

            j += var_put_u32(&mut run[j as usize..], None, rlen as u32) as u64;
        }
        i += 1;
    }

    *run_len = j;
    *out_len = k;

    // Return an owned copy of the literal buffer (full allocation length, as the
    // C function returns the buffer pointer; callers use *out_len for the used
    // portion). We copy [0..out_len) which is the meaningful content.
    let result = out_slice[..k as usize].to_vec();
    if !alloc_ptr.is_null() {
        unsafe { c_compat::free(alloc_ptr as *mut std::ffi::c_void) };
    }
    Some(result)
}

/// ```c
/// uint8_t *hts_rle_decode(uint8_t *lit, uint64_t lit_len,
///                         uint8_t *run, uint64_t run_len,
///                         uint8_t *rle_syms, int rle_nsyms,
///                         uint8_t *out, uint64_t *out_len);
/// ```
/// On input `*out_len` holds the allocated size of `out`; on output the used size.
// rle.c:142 (also rle.h:84)
pub fn hts_rle_decode<'a>(
    lit: &[u8],
    lit_len: u64,
    run: &[u8],
    run_len: u64,
    rle_syms: &[u8],
    rle_nsyms: i32,
    out: &'a mut [u8],
    out_len: &mut u64,
) -> Option<&'a mut [u8]> {
    // uint8_t *run_end = run + run_len;
    let run_end: usize = run_len as usize;

    let mut saved = [0i32; 256];
    for j in 0..rle_nsyms as usize {
        saved[rle_syms[j] as usize] = 1;
    }

    let lit_end: usize = lit_len as usize;
    let out_end: usize = *out_len as usize;

    // run pointer offset
    let mut run_pos: usize = 0;
    let mut lit_pos: usize = 0;
    let mut outp: usize = 0;

    while lit_pos < lit_end {
        if outp >= out_end {
            return None; // goto err
        }

        let b: u8 = lit[lit_pos];
        if saved[b as usize] != 0 {
            let mut rlen: u32 = 0;
            let endp = run_end - run_pos;
            run_pos += var_get_u32(&run[run_pos..], Some(endp), &mut rlen) as usize;
            if rlen != 0 {
                // if (outp + rlen >= out_end) goto err;
                if outp + rlen as usize >= out_end {
                    return None;
                }
                // memset(outp, b, rlen+1);
                for x in 0..(rlen as usize + 1) {
                    out[outp + x] = b;
                }
                outp += rlen as usize + 1;
            } else {
                out[outp] = b;
                outp += 1;
            }
        } else {
            out[outp] = b;
            outp += 1;
        }
        lit_pos += 1;
    }

    *out_len = outp as u64;
    Some(&mut out[..outp])
}

/// ```c
/// uint8_t *rle_encode(uint8_t *data, uint64_t data_len,
///                     uint8_t *run,  uint64_t *run_len,
///                     uint8_t *rle_syms, int *rle_nsyms,
///                     uint8_t *out, uint64_t *out_len);
/// ```
/// Deprecated interface; forwards to `hts_rle_encode`.
// rle.c:192
pub fn rle_encode(
    data: &[u8],
    data_len: u64,
    run: &mut [u8],
    run_len: &mut u64,
    rle_syms: &mut [u8],
    rle_nsyms: &mut i32,
    out: Option<&mut [u8]>,
    out_len: &mut u64,
) -> Option<Vec<u8>> {
    hts_rle_encode(
        data, data_len, run, run_len, rle_syms, rle_nsyms, out, out_len,
    )
}

/// ```c
/// uint8_t *rle_decode(uint8_t *lit, uint64_t lit_len,
///                     uint8_t *run, uint64_t run_len,
///                     uint8_t *rle_syms, int rle_nsyms,
///                     uint8_t *out, uint64_t *out_len);
/// ```
/// Deprecated interface; forwards to `hts_rle_decode`.
// rle.c:201
pub fn rle_decode<'a>(
    lit: &[u8],
    lit_len: u64,
    run: &[u8],
    run_len: u64,
    rle_syms: &[u8],
    rle_nsyms: i32,
    out: &'a mut [u8],
    out_len: &mut u64,
) -> Option<&'a mut [u8]> {
    hts_rle_decode(lit, lit_len, run, run_len, rle_syms, rle_nsyms, out, out_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Native round-trip: encode then decode recovers the input.
    fn roundtrip(data: &[u8]) {
        let data_len = data.len() as u64;
        let mut run = vec![0u8; data.len().max(1)];
        let mut run_len: u64 = 0;
        let mut rle_syms = [0u8; 256];
        let mut rle_nsyms: i32 = 0;
        let mut out_len: u64 = 0;

        let lit = hts_rle_encode(
            data,
            data_len,
            &mut run,
            &mut run_len,
            &mut rle_syms,
            &mut rle_nsyms,
            None,
            &mut out_len,
        )
        .expect("encode");

        // Decode back.
        let mut decoded = vec![0u8; data.len() + 16];
        let mut dlen: u64 = decoded.len() as u64;
        let res = hts_rle_decode(
            &lit,
            out_len,
            &run[..run_len as usize],
            run_len,
            &rle_syms[..rle_nsyms as usize],
            rle_nsyms,
            &mut decoded,
            &mut dlen,
        );
        assert!(res.is_some(), "decode returned None");
        assert_eq!(dlen, data_len, "decoded length mismatch");
        assert_eq!(&decoded[..dlen as usize], data, "decoded content mismatch");
    }

    fn corpus() -> Vec<(&'static str, Vec<u8>)> {
        let mut v: Vec<(&'static str, Vec<u8>)> = Vec::new();
        v.push(("empty", vec![]));
        v.push(("single", vec![42]));
        v.push(("no_runs", (0..200u32).map(|x| (x % 251) as u8).collect()));
        v.push(("long_run", vec![7u8; 5000]));
        v.push(("two_runs", {
            let mut d = vec![1u8; 1000];
            d.extend(vec![2u8; 1000]);
            d
        }));
        v.push(("mixed", {
            let mut d = Vec::new();
            for i in 0..1000u32 {
                if i % 3 == 0 {
                    d.extend(vec![(i % 7) as u8; (i % 50) as usize + 1]);
                } else {
                    d.push((i * 13) as u8);
                }
            }
            d
        }));
        v.push(("small_runs", b"aaabbbcccaaa".to_vec()));
        v.push(("all_same_small", vec![3u8; 100]));
        v
    }

    #[test]
    fn native_roundtrip() {
        for (name, data) in corpus() {
            // Skip empty: encode is fine but decode of empty is trivial.
            println!("roundtrip {name}");
            roundtrip(&data);
        }
    }

    // -----------------------------------------------------------------------
    // Byte-parity against libhts C `hts_rle_encode` / `hts_rle_decode`.
    // -----------------------------------------------------------------------
    #[cfg(feature = "parity")]
    mod parity {
        use super::*;
        use std::os::raw::c_int;

        extern "C" {
            #[link_name = "hts_rle_encode"]
            fn c_hts_rle_encode(
                data: *mut u8,
                data_len: u64,
                run: *mut u8,
                run_len: *mut u64,
                rle_syms: *mut u8,
                rle_nsyms: *mut c_int,
                out: *mut u8,
                out_len: *mut u64,
            ) -> *mut u8;
            #[link_name = "hts_rle_decode"]
            fn c_hts_rle_decode(
                lit: *mut u8,
                lit_len: u64,
                run: *mut u8,
                run_len: u64,
                rle_syms: *mut u8,
                rle_nsyms: c_int,
                out: *mut u8,
                out_len: *mut u64,
            ) -> *mut u8;
        }

        struct CEnc {
            lit: Vec<u8>,
            run: Vec<u8>,
            syms: Vec<u8>,
            nsyms: i32,
        }

        fn c_encode(data: &[u8]) -> CEnc {
            let mut buf = data.to_vec();
            let mut run = vec![0u8; data.len().max(1)];
            let mut run_len: u64 = 0;
            let mut syms = vec![0u8; 256];
            let mut nsyms: c_int = 0;
            let mut out = vec![0u8; data.len() * 2 + 2];
            let mut out_len: u64 = 0;
            let p = unsafe {
                c_hts_rle_encode(
                    buf.as_mut_ptr(),
                    data.len() as u64,
                    run.as_mut_ptr(),
                    &mut run_len,
                    syms.as_mut_ptr(),
                    &mut nsyms,
                    out.as_mut_ptr(),
                    &mut out_len,
                )
            };
            assert!(!p.is_null(), "C encode null");
            let lit = unsafe { std::slice::from_raw_parts(p, out_len as usize) }.to_vec();
            CEnc {
                lit,
                run: run[..run_len as usize].to_vec(),
                syms: syms[..nsyms as usize].to_vec(),
                nsyms,
            }
        }

        fn native_encode(data: &[u8]) -> CEnc {
            let mut run = vec![0u8; data.len().max(1)];
            let mut run_len: u64 = 0;
            let mut syms = [0u8; 256];
            let mut nsyms: i32 = 0;
            let mut out_len: u64 = 0;
            let lit = hts_rle_encode(
                data,
                data.len() as u64,
                &mut run,
                &mut run_len,
                &mut syms,
                &mut nsyms,
                None,
                &mut out_len,
            )
            .expect("native encode");
            CEnc {
                lit,
                run: run[..run_len as usize].to_vec(),
                syms: syms[..nsyms as usize].to_vec(),
                nsyms,
            }
        }

        fn c_decode(e: &CEnc, expect_len: usize) -> Vec<u8> {
            let mut lit = e.lit.clone();
            let mut run = e.run.clone();
            let mut syms = e.syms.clone();
            let mut out = vec![0u8; expect_len + 16];
            let mut out_len: u64 = out.len() as u64;
            let p = unsafe {
                c_hts_rle_decode(
                    lit.as_mut_ptr(),
                    lit.len() as u64,
                    run.as_mut_ptr(),
                    run.len() as u64,
                    syms.as_mut_ptr(),
                    e.nsyms as c_int,
                    out.as_mut_ptr(),
                    &mut out_len,
                )
            };
            assert!(!p.is_null(), "C decode null");
            out[..out_len as usize].to_vec()
        }

        fn native_decode(e: &CEnc, expect_len: usize) -> Vec<u8> {
            let mut out = vec![0u8; expect_len + 16];
            let mut out_len: u64 = out.len() as u64;
            let res = hts_rle_decode(
                &e.lit,
                e.lit.len() as u64,
                &e.run,
                e.run.len() as u64,
                &e.syms,
                e.nsyms,
                &mut out,
                &mut out_len,
            );
            assert!(res.is_some(), "native decode None");
            out[..out_len as usize].to_vec()
        }

        #[test]
        fn encode_byte_identical() {
            for (name, data) in corpus() {
                let n = native_encode(&data);
                let c = c_encode(&data);
                assert_eq!(n.nsyms, c.nsyms, "nsyms mismatch {name}");
                assert_eq!(n.syms, c.syms, "syms mismatch {name}");
                assert_eq!(n.lit, c.lit, "lit mismatch {name}");
                assert_eq!(n.run, c.run, "run mismatch {name}");
            }
        }

        #[test]
        fn cross_decode() {
            for (name, data) in corpus() {
                if data.is_empty() {
                    continue;
                }
                // native encode -> C decode
                let n = native_encode(&data);
                let d_c = c_decode(&n, data.len());
                assert_eq!(d_c, data, "C decode of native output mismatch {name}");

                // C encode -> native decode
                let c = c_encode(&data);
                let d_n = native_decode(&c, data.len());
                assert_eq!(d_n, data, "native decode of C output mismatch {name}");
            }
        }
    }
}
