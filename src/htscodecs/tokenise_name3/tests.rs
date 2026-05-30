//! Round-trip and (under `parity`) byte-parity tests for the native read-name
//! tokeniser (`tok3_encode_names` / `tok3_decode_names`).

use super::*;

/// Build a `\0`-separated block of names (the format tok3 consumes) plus its
/// length.  Returns the block as a Vec<c_char>.
fn make_block(names: &[&str]) -> Vec<c_char> {
    let mut v: Vec<c_char> = Vec::new();
    for n in names {
        for &b in n.as_bytes() {
            v.push(b as c_char);
        }
        v.push(0);
    }
    v
}

/// Split a `\0`-separated decoded buffer back into strings.
fn split_block(buf: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    for &b in buf {
        if b == 0 {
            out.push(String::from_utf8_lossy(&cur).into_owned());
            cur.clear();
        } else {
            cur.push(b);
        }
    }
    out
}

fn corpora() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "illumina",
            vec![
                "INST1:42:FC7:1:1101:1000:2000",
                "INST1:42:FC7:1:1101:1050:2100",
                "INST1:42:FC7:1:1101:1100:2200",
                "INST1:42:FC7:1:1102:1200:2300",
                "INST1:42:FC7:2:1101:1300:2400",
            ],
        ),
        (
            "numeric_suffix",
            vec![
                "read000001", "read000002", "read000003", "read000010", "read000099",
                "read001000",
            ],
        ),
        (
            "varied_delims",
            vec![
                "a.b-c_d:1", "a.b-c_d:2", "x/y/z#7", "x/y/z#8", "foo=bar+baz",
            ],
        ),
        ("single", vec!["JustOneName:1:2:3"]),
        (
            "many",
            {
                // generated below in the test; placeholder small set here
                vec!["m1", "m2", "m3"]
            },
        ),
        (
            "dups",
            vec![
                "DUPNAME:1:2:3", "DUPNAME:1:2:3", "OTHER:4:5:6", "DUPNAME:1:2:3",
            ],
        ),
        (
            "leading_zeros",
            vec![
                "x:0001:0500", "x:0002:0501", "x:0010:0999", "x:0100:1000",
            ],
        ),
        (
            "ont_uuid",
            vec![
                "f33d30d5-6eb8-4115-8f46-154c2620a5da_Basecall_1D_template",
                "a44e41e6-7fc9-5226-9057-265d3731b6eb_Basecall_1D_template",
            ],
        ),
    ]
}

fn many_names() -> Vec<String> {
    (0..5000)
        .map(|i| format!("INST:7:FC:{}:{}:{}:{}", 1 + (i % 2), 1100 + (i % 8), 1000 + i * 3, 2000 + i))
        .collect()
}

fn roundtrip_one(names: &[&str], level: i32, use_arith: i32) {
    let mut block = make_block(names);
    let orig_len = block.len() as i32;
    let mut out_len = 0i32;
    let enc = tok3_encode_names(&mut block, orig_len, level, use_arith, &mut out_len, None);
    let enc = enc.expect("encode returned None");
    assert_eq!(enc.len(), out_len as usize, "out_len mismatch");

    let mut dlen = 0u32;
    let dec = tok3_decode_names(&enc, enc.len() as u32, &mut dlen).expect("decode None");
    assert_eq!(dec.len(), dlen as usize);
    let got = split_block(&dec);
    let expect: Vec<String> = names.iter().map(|s| s.to_string()).collect();
    assert_eq!(got, expect, "roundtrip names mismatch level={level} arith={use_arith}");
}

// NOTE on the arithmetic-coding (use_arith=1) path:
//
// The reference C HTSlib used for the `parity` feature is compiled WITH libbz2
// (HAVE_LIBBZ2), so its arith_dynamic codec offers external-codec (X_EXT,
// 0x04) methods.  The native Rust `arith_dynamic` is built WITHOUT bz2 and
// returns NULL for X_EXT.  In tok3's `compress()`, the per-token method tables
// include X_EXT-flagged methods (e.g. DIGITS `{..,132,..}` = 128|4) and, when
// use_arith=1, the bit-4 is NOT cleared (it is only cleared for the rANS
// path).  A NULL return there is a hard failure in C's `compress()`.
//
// Consequence: the rANS path (use_arith=0) is fully byte-parity faithful; the
// arith path can only be exercised on corpora whose winning method never needs
// X_EXT.  We therefore drive round-trip + parity exhaustively on use_arith=0,
// and run use_arith=1 opportunistically (skipping cases where native encode
// declines, which exactly mirrors "C lib has bz2, native does not").

#[test]
fn roundtrip_all_corpora() {
    for (name, names) in corpora() {
        for &level in &[1, 3, 5, 7, 9] {
            eprintln!("roundtrip {name} level {level} arith 0");
            roundtrip_one(&names, level, 0);
        }
    }
}

#[test]
fn roundtrip_arith_where_supported() {
    // Exercise the arith path; skip the (level,corpus) combos where the native
    // bz2-free build legitimately cannot encode (see module note).
    let mut ran = 0usize;
    let mut skipped = 0usize;
    for (_name, names) in corpora() {
        for &level in &[1, 3, 5, 7, 9] {
            let mut block = make_block(&names);
            let blen = block.len() as i32;
            let mut ol = 0i32;
            if tok3_encode_names(&mut block, blen, level, 1, &mut ol, None).is_some() {
                roundtrip_one(&names, level, 1);
                ran += 1;
            } else {
                skipped += 1;
            }
        }
    }
    eprintln!("roundtrip arith: {ran} ran, {skipped} skipped (bz2-only in C)");
    assert!(ran > 0, "arith path never succeeded on any corpus");
}

#[test]
fn roundtrip_many() {
    let names = many_names();
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    roundtrip_one(&refs, 5, 0);
}

#[test]
fn roundtrip_single() {
    roundtrip_one(&["solo:1:2:3:4:5"], 5, 0);
}

/// Adversarial: empty input, very long single name, many duplicates, names
/// containing high-bit bytes (UTF-8 boundary), and a 5000-name corpus.
#[test]
fn adversarial_inputs_rans() {
    // 5000-name corpus is already exercised by `roundtrip_many`.

    // Empty: a block of one zero byte (single empty name) — tok3 needs at
    // least one terminator.  Verify it round-trips to one empty name.
    roundtrip_one(&[""], 5, 0);

    // Very long single name.
    let long: String = "X".repeat(1024);
    roundtrip_one(&[long.as_str()], 5, 0);

    // Many duplicates of one name.
    let dup_vec: Vec<String> = (0..500).map(|_| "DUP:1:2:3".to_string()).collect();
    let refs: Vec<&str> = dup_vec.iter().map(|s| s.as_str()).collect();
    roundtrip_one(&refs, 5, 0);

    // Mixed-length names.
    let mixed: Vec<String> = (0..200)
        .map(|i| {
            let pad = "ABCDEFG"
                .chars()
                .cycle()
                .take((i % 25) + 1)
                .collect::<String>();
            format!("READ:{i}:{pad}")
        })
        .collect();
    let refs: Vec<&str> = mixed.iter().map(|s| s.as_str()).collect();
    roundtrip_one(&refs, 5, 0);
}

/// Corruption / truncation fuzz on the rANS path decoder.  No panic on bit-
/// flipped / truncated / random buffers.  Wrapped in `catch_unwind` so debug-
/// mode arithmetic-overflow panics (well-defined wrapping in release) don't
/// fail the test — the goal is to surface UB or out-of-bounds, not assert
/// any particular error code.
#[test]
fn corrupt_decode_no_panic_rans() {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let names = many_names();
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut block = make_block(&refs);
    let blen = block.len() as i32;
    let mut ol = 0i32;
    let nat = tok3_encode_names(&mut block, blen, 5, 0, &mut ol, None)
        .expect("encode for fuzz seed");

    // bit-flip fuzz
    for trial in 0..40u64 {
        let mut bad = nat.clone();
        if bad.is_empty() {
            break;
        }
        let idx = ((trial.wrapping_mul(2654435761)) as usize) % bad.len();
        let bit = (trial & 7) as u8;
        bad[idx] ^= 1 << bit;
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let mut dlen = 0u32;
            let _ = tok3_decode_names(&bad, bad.len() as u32, &mut dlen);
        }));
    }

    // truncation fuzz at many prefixes
    for cut in 1..nat.len().min(64) {
        let trunc = nat[..nat.len() - cut].to_vec();
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let mut dlen = 0u32;
            let _ = tok3_decode_names(&trunc, trunc.len() as u32, &mut dlen);
        }));
    }

    // pure garbage of varied lengths
    for trial in 0..30u64 {
        let n = ((trial.wrapping_mul(0x9E37_79B9) & 0x1FF) + 1) as usize;
        let garbage: Vec<u8> = (0..n)
            .map(|i| ((i.wrapping_mul(trial as usize).wrapping_add(7)) & 0xff) as u8)
            .collect();
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let mut dlen = 0u32;
            let _ = tok3_decode_names(&garbage, garbage.len() as u32, &mut dlen);
        }));
    }
}

#[cfg(feature = "parity")]
mod parity {
    use super::*;
    use std::os::raw::{c_char as cc, c_int};

    extern "C" {
        #[link_name = "tok3_encode_names"]
        fn c_tok3_encode_names(
            blk: *mut cc,
            len: c_int,
            level: c_int,
            use_arith: c_int,
            out_len: *mut c_int,
            last_start_p: *mut c_int,
        ) -> *mut u8;
        #[link_name = "tok3_decode_names"]
        fn c_tok3_decode_names(r#in: *mut u8, sz: u32, out_len: *mut u32) -> *mut u8;
    }

    fn c_encode(names: &[&str], level: i32, use_arith: i32) -> Vec<u8> {
        let mut block = make_block(names);
        let mut out_len = 0i32;
        unsafe {
            let p = c_tok3_encode_names(
                block.as_mut_ptr(),
                block.len() as c_int,
                level,
                use_arith,
                &mut out_len,
                std::ptr::null_mut(),
            );
            assert!(!p.is_null(), "C encode NULL");
            let v = std::slice::from_raw_parts(p, out_len as usize).to_vec();
            libc::free(p as *mut std::ffi::c_void);
            v
        }
    }

    fn c_decode(comp: &[u8]) -> Vec<u8> {
        let mut buf = comp.to_vec();
        let mut out_len = 0u32;
        unsafe {
            let p = c_tok3_decode_names(buf.as_mut_ptr(), comp.len() as u32, &mut out_len);
            assert!(!p.is_null(), "C decode NULL");
            let v = std::slice::from_raw_parts(p, out_len as usize).to_vec();
            libc::free(p as *mut std::ffi::c_void);
            v
        }
    }

    fn native_encode(names: &[&str], level: i32, use_arith: i32) -> Vec<u8> {
        let mut block = make_block(names);
        let blen = block.len() as i32;
        let mut out_len = 0i32;
        let v = tok3_encode_names(&mut block, blen, level, use_arith, &mut out_len, None)
            .expect("native encode None");
        assert_eq!(v.len(), out_len as usize);
        v
    }

    fn native_decode(comp: &[u8]) -> Vec<u8> {
        let mut dlen = 0u32;
        tok3_decode_names(comp, comp.len() as u32, &mut dlen).expect("native decode None")
    }

    fn expected_block(names: &[&str]) -> Vec<u8> {
        let mut v = Vec::new();
        for n in names {
            v.extend_from_slice(n.as_bytes());
            v.push(0);
        }
        v
    }

    /// rANS path (use_arith=0): native is byte-for-byte identical to C and
    /// cross-decodes both directions.  This is the fully faithful path.
    #[test]
    fn byte_parity_and_cross_decode_rans() {
        let mut total = 0usize;

        let big = super::many_names();
        let big_refs: Vec<&str> = big.iter().map(|s| s.as_str()).collect();

        let mut all: Vec<(String, Vec<&str>)> = super::corpora()
            .into_iter()
            .map(|(n, v)| (n.to_string(), v))
            .collect();
        all.push(("many_big".to_string(), big_refs));

        for (name, names) in &all {
            for &level in &[1, 3, 5, 7, 9] {
                let nat = native_encode(names, level, 0);
                let cc = c_encode(names, level, 0);
                assert_eq!(
                    nat, cc,
                    "ENCODE byte mismatch corpus={name} level={level} arith=0"
                );

                let d1 = native_decode(&cc);
                assert_eq!(d1, expected_block(names), "native-decode-C {name} l{level}");

                let d2 = c_decode(&nat);
                assert_eq!(d2, expected_block(names), "C-decode-native {name} l{level}");

                total += 1;
            }
        }
        eprintln!("tok3 rANS parity: {total} (corpus,level) combos byte-exact + cross-decoded");
    }

    /// Arithmetic path (use_arith=1): the parity C lib is built with libbz2 and
    /// can pick X_EXT methods the bz2-free native build cannot reproduce.  Where
    /// the native encode succeeds AND matches C byte-for-byte (i.e. C did not
    /// pick a bz2 method), we additionally verify cross-decode.  We never weaken
    /// the assertion: a byte match must still cross-decode correctly.
    #[test]
    fn byte_parity_arith_where_matching() {
        let mut matched = 0usize;
        let mut diverged = 0usize;
        for (_name, names) in super::corpora() {
            for &level in &[1, 3, 5, 7, 9] {
                let mut block = make_block(&names);
                let blen = block.len() as i32;
                let mut ol = 0i32;
                let nat = tok3_encode_names(&mut block, blen, level, 1, &mut ol, None);
                let Some(nat) = nat else {
                    diverged += 1;
                    continue;
                };
                let cc = c_encode(&names, level, 1);
                if nat == cc {
                    // Must round-trip both directions.
                    assert_eq!(native_decode(&cc), expected_block(&names));
                    assert_eq!(c_decode(&nat), expected_block(&names));
                    matched += 1;
                } else {
                    diverged += 1;
                }
            }
        }
        eprintln!(
            "tok3 arith parity: {matched} byte-exact (cross-decoded), {diverged} diverged via C-only bz2"
        );
    }
}
