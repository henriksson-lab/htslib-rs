//! Tests for `permute.rs`. Compiled only under `#[cfg(test)]`.
//!
//! The C header `htslib/htscodecs/htscodecs/permute.h` defines two static
//! tables, `permute[256][8]` and `permutec[256][8]`, plus a `#ifdef MAIN`
//! debug program that PRINTS those tables as C source. The Rust translation
//! mirrors the printer (`super::main`) AND now populates the statics
//! `super::permute` / `super::permutec` at compile time via the const-fn
//! generators in `permute.rs` (the same algorithm as the C `#ifdef MAIN`).
//!
//! These tests COMPUTE the tables here, in test code, using the exact C
//! algorithm from `permute.h:10-65`, and pin down their values. The test
//! `super_permute_static_matches_test_local_builder` cross-checks the
//! populated statics against the test-local builder, so any drift in the
//! const-fn generator is caught immediately.
//!
//! Inverse invariant (pinned, see `permute.h:46-62`):
//!   Let `b = popcount(i)`. For every input-bit position `j` with bit `j`
//!   of `i` set, let `k` be the 0-based output lane chosen by the decode
//!   table, i.e. `permute[i][j] == k`. The encode table records the
//!   inverse mapping RIGHT-aligned into its 8 lanes: the `b` live entries
//!   occupy slots `8-b..8`, with the first `8-b` slots filled with the
//!   sentinel `_`. Concretely, if `j` is the `k`-th set bit of `i`
//!   (0-indexed), then
//!       permute[i][j]      == k          (decode: source bit j -> lane k)
//!       permutec[i][8-b+k] == j          (encode: lane (8-b+k)  <- bit j)
//!   The `permutec` print loop in `permute.h:54` iterates `j = b-8; j < b`
//!   and emits `_` whenever `j < 0`, producing the right-aligned layout.

use super::*;

/// Sentinel value used by the C `#define _ 9` macro to mark skipped lanes.
const UNDERSCORE_LOCAL: u32 = 9;

/// Reproduces the C decode-table builder from `permute.h:24-40`.
///
/// For each `i` in `0..256` and each input-bit position `j` in `0..8`:
///   * if bit `j` of `i` is set, the byte at input position `j` is kept,
///     and the table records the 0-based output lane `k` it lands in
///     (`k = (number of set bits with index <= j) - 1`).
///   * if bit `j` of `i` is cleared, the slot holds the sentinel `9`.
fn build_permute() -> [[u32; 8]; 256] {
    let mut out = [[UNDERSCORE_LOCAL; 8]; 256];
    for i in 0..256usize {
        let mut b: u32 = 0;
        for j in 0..8 {
            if (i & (1 << j)) != 0 {
                // The C does `v[j] = ++b` then prints `v[j]-1`, i.e. the
                // 0-based output lane index.
                out[i][j] = b;
                b += 1;
            }
        }
    }
    out
}

/// Reproduces the C encode-table builder from `permute.h:45-62`.
///
/// The encode table is RIGHT-aligned: with `b = popcount(i)`, the live
/// entries occupy slots `8-b..8` and the first `8-b` slots are sentinels.
/// Slot `8-b+k` (for `k` in `0..b`) holds the 0-based input bit position
/// of the `k`-th set bit of `i`.
fn build_permutec() -> [[u32; 8]; 256] {
    let mut out = [[UNDERSCORE_LOCAL; 8]; 256];
    for i in 0..256usize {
        // Collect the 1-based bit positions of the set bits, in order.
        let mut v = [0i32; 9];
        let mut b: i32 = 0;
        for j in 0..8i32 {
            if (i & (1 << j)) != 0 {
                v[b as usize] = j + 1;
                b += 1;
            }
        }
        // Mirror the C print loop: `for (j = b-8; j < b; j++)`. That is
        // exactly 8 iterations covering slot index `j - (b-8)` = `0..8`.
        let mut slot = 0usize;
        let mut j = b - 8;
        while j < b {
            if j >= 0 && v[j as usize] != 0 {
                out[i][slot] = (v[j as usize] - 1) as u32;
            }
            slot += 1;
            j += 1;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// 1. Table-shape invariants
// ---------------------------------------------------------------------------

#[test]
fn permute_row_shape_matches_popcount() {
    let p = build_permute();
    for i in 0..256usize {
        let pop = (i as u8).count_ones();
        let live = p[i].iter().filter(|&&x| x != UNDERSCORE_LOCAL).count() as u32;
        assert_eq!(
            live, pop,
            "permute[{i}] has {live} live entries, expected popcount={pop}"
        );
        // All live entries must be in 0..popcount (the C prints v[j]-1, so
        // the output lane indices are 0-based and unique within the row).
        let max_live = p[i]
            .iter()
            .copied()
            .filter(|&x| x != UNDERSCORE_LOCAL)
            .max();
        if pop == 0 {
            assert!(max_live.is_none(), "permute[{i}] should be all sentinels");
        } else {
            assert_eq!(
                max_live,
                Some(pop - 1),
                "permute[{i}] max live should be popcount-1={}",
                pop - 1
            );
        }
        // Live entries must be a permutation of 0..pop (no duplicates).
        let mut live_vals: Vec<u32> =
            p[i].iter().copied().filter(|&x| x != UNDERSCORE_LOCAL).collect();
        live_vals.sort_unstable();
        let expected: Vec<u32> = (0..pop).collect();
        assert_eq!(
            live_vals, expected,
            "permute[{i}] live entries should be exactly 0..{pop}"
        );
    }
}

#[test]
fn permutec_row_shape_matches_popcount() {
    let pc = build_permutec();
    for i in 0..256usize {
        let pop = (i as u8).count_ones();
        let live = pc[i]
            .iter()
            .filter(|&&x| x != UNDERSCORE_LOCAL)
            .count() as u32;
        assert_eq!(
            live, pop,
            "permutec[{i}] has {live} live entries, expected popcount={pop}"
        );
        // permutec is right-aligned: the first 8-pop slots are sentinels.
        let leading_sentinels = (8 - pop) as usize;
        for slot in 0..leading_sentinels {
            assert_eq!(
                pc[i][slot], UNDERSCORE_LOCAL,
                "permutec[{i}][{slot}] should be sentinel (right-aligned)"
            );
        }
        // The trailing slots are 0-based input bit positions; all must
        // be in 0..8 and strictly increasing (set-bit indices in order).
        let mut prev: i32 = -1;
        for slot in leading_sentinels..8 {
            let v = pc[i][slot];
            assert!(v < 8, "permutec[{i}][{slot}]={v} out of range");
            assert!(
                (v as i32) > prev,
                "permutec[{i}][{slot}]={v} not strictly increasing after {prev}"
            );
            prev = v as i32;
        }
    }
}

#[test]
fn sentinel_value_is_nine() {
    // Pin the sentinel value. The C source says `#define _ 9`.
    assert_eq!(UNDERSCORE_LOCAL, 9);
    assert_eq!(super::UNDERSCORE, 9);
}

// ---------------------------------------------------------------------------
// 2. Specific golden rows (lock in exact values)
// ---------------------------------------------------------------------------

#[test]
fn permute_golden_rows() {
    let p = build_permute();
    let _ = UNDERSCORE_LOCAL;
    // row 0 (no bits set): all sentinels (permute.h:90)
    assert_eq!(p[0], [9, 9, 9, 9, 9, 9, 9, 9]);
    // row 255 (all bits set): identity (permute.h:345)
    assert_eq!(p[255], [0, 1, 2, 3, 4, 5, 6, 7]);
    // row 1 (bit 0): one byte goes to lane 0 (permute.h:91)
    assert_eq!(p[1], [0, 9, 9, 9, 9, 9, 9, 9]);
    // row 0x80 = 128 (bit 7): one byte goes to lane 0 (permute.h:218)
    assert_eq!(p[0x80], [9, 9, 9, 9, 9, 9, 9, 0]);
    // row 17 = 0b00010001 (bits 0 and 4): two bytes -> lanes 0,1 (permute.h:106)
    assert_eq!(p[17], [0, 9, 9, 9, 1, 9, 9, 9]);
    // row 170 = 0b10101010 (bits 1,3,5,7) (permute.h:260)
    assert_eq!(p[170], [9, 0, 9, 1, 9, 2, 9, 3]);
    // A handful more mixed patterns, copied byte-for-byte from permute.h:
    // row 2 = 0b00000010 (bit 1)  -> permute.h:92  { _,0,_,_,_,_,_,_, }
    assert_eq!(p[2], [9, 0, 9, 9, 9, 9, 9, 9]);
    // row 3 = 0b00000011 (bits 0,1) -> permute.h:93  { 0,1,_,_,_,_,_,_, }
    assert_eq!(p[3], [0, 1, 9, 9, 9, 9, 9, 9]);
    // row 15 = 0b00001111 (bits 0..3) -> permute.h:105  { 0,1,2,3,_,_,_,_, }
    assert_eq!(p[15], [0, 1, 2, 3, 9, 9, 9, 9]);
    // row 85 = 0b01010101 (bits 0,2,4,6) -> permute.h:175  { 0,_,1,_,2,_,3,_, }
    assert_eq!(p[85], [0, 9, 1, 9, 2, 9, 3, 9]);
    // row 240 = 0b11110000 (bits 4..7) -> permute.h:330  { _,_,_,_,0,1,2,3, }
    assert_eq!(p[240], [9, 9, 9, 9, 0, 1, 2, 3]);
    // row 254 = 0b11111110 (bits 1..7) -> permute.h:344  { _,0,1,2,3,4,5,6, }
    assert_eq!(p[254], [9, 0, 1, 2, 3, 4, 5, 6]);
}

#[test]
fn permutec_golden_rows() {
    let pc = build_permutec();
    // row 0 -> permute.h:349  { _,_,_,_,_,_,_,_, }
    assert_eq!(pc[0], [9, 9, 9, 9, 9, 9, 9, 9]);
    // row 1 -> permute.h:350  { _,_,_,_,_,_,_,0, }
    assert_eq!(pc[1], [9, 9, 9, 9, 9, 9, 9, 0]);
    // row 0x80 = 128 -> permute.h:477  { _,_,_,_,_,_,_,7, }
    assert_eq!(pc[0x80], [9, 9, 9, 9, 9, 9, 9, 7]);
    // row 17 = 0b00010001 -> permute.h:366  { _,_,_,_,_,_,0,4, }
    assert_eq!(pc[17], [9, 9, 9, 9, 9, 9, 0, 4]);
    // row 170 = 0b10101010 -> permute.h:519  { _,_,_,_,1,3,5,7, }
    assert_eq!(pc[170], [9, 9, 9, 9, 1, 3, 5, 7]);
    // row 255 -> permute.h:604  { 0,1,2,3,4,5,6,7, }
    assert_eq!(pc[255], [0, 1, 2, 3, 4, 5, 6, 7]);
    // row 2 -> permute.h:351  { _,_,_,_,_,_,_,1, }
    assert_eq!(pc[2], [9, 9, 9, 9, 9, 9, 9, 1]);
    // row 3 -> permute.h:352  { _,_,_,_,_,_,0,1, }
    assert_eq!(pc[3], [9, 9, 9, 9, 9, 9, 0, 1]);
    // row 15 -> permute.h:364  { _,_,_,_,0,1,2,3, }
    assert_eq!(pc[15], [9, 9, 9, 9, 0, 1, 2, 3]);
    // row 85 = 0b01010101 -> permute.h:380  { _,_,_,0,2,4,6,_, }? No, let me trust the table.
    // popcount(85)=4, leading_sentinels=4, then live bit positions 0,2,4,6.
    assert_eq!(pc[85], [9, 9, 9, 9, 0, 2, 4, 6]);
    // row 240 = 0b11110000 -> popcount=4, live positions 4,5,6,7.
    assert_eq!(pc[240], [9, 9, 9, 9, 4, 5, 6, 7]);
    // row 254 = 0b11111110 -> popcount=7, leading=1, live positions 1..7.
    assert_eq!(pc[254], [9, 1, 2, 3, 4, 5, 6, 7]);
}

// ---------------------------------------------------------------------------
// 3. Permutec inverse relation
// ---------------------------------------------------------------------------

#[test]
fn permutec_is_right_aligned_inverse_of_permute() {
    // Pinned invariant (see permute.h:46-62 and the header docs above):
    //   for every i, every set bit j of i with permute[i][j] == k:
    //     permutec[i][ 8 - popcount(i) + k ] == j
    let p = build_permute();
    let pc = build_permutec();
    for i in 0..256usize {
        let pop = (i as u8).count_ones() as usize;
        let offset = 8 - pop;
        for j in 0..8usize {
            if (i & (1 << j)) != 0 {
                let k = p[i][j] as usize;
                assert!(
                    k < pop,
                    "permute[{i}][{j}]={k} should be in 0..popcount({pop})"
                );
                assert_eq!(
                    pc[i][offset + k], j as u32,
                    "inverse mismatch: permute[{i}][{j}]={k} but \
                     permutec[{i}][{}]={} (expected {j})",
                    offset + k,
                    pc[i][offset + k]
                );
            } else {
                // Bit j is clear -> permute slot is the sentinel.
                assert_eq!(
                    p[i][j], UNDERSCORE_LOCAL,
                    "permute[{i}][{j}] should be sentinel for cleared bit"
                );
            }
        }
        // Every live slot of permutec corresponds to some set bit of i.
        for slot in offset..8 {
            let bit = pc[i][slot] as usize;
            assert!(
                bit < 8 && (i & (1 << bit)) != 0,
                "permutec[{i}][{slot}]={bit} should reference a set bit of {i}"
            );
            // And the round-trip back through permute must land at this slot.
            let k = p[i][bit] as usize;
            assert_eq!(
                offset + k,
                slot,
                "permutec[{i}][{slot}] -> bit {bit} -> permute lane {k}, \
                 expected slot {slot}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 4. End-to-end byte shuffle simulation
// ---------------------------------------------------------------------------

/// Apply the decode permute table: given 8 input bytes and a mask `i`,
/// produce the kept bytes in their lane order. Output length = popcount(i).
fn apply_permute(p: &[[u32; 8]; 256], i: usize, input: &[u8; 8]) -> Vec<u8> {
    let pop = (i as u8).count_ones() as usize;
    let mut out = vec![0u8; pop];
    for j in 0..8 {
        if p[i][j] != UNDERSCORE_LOCAL {
            out[p[i][j] as usize] = input[j];
        }
    }
    out
}

/// Apply the encode permutec table to "unshuffle": given the packed output
/// (length popcount(i), conceptually right-aligned into 8 lanes), return
/// the 8-byte slot-positioned vector with cleared slots filled with `fill`.
fn apply_permutec(
    pc: &[[u32; 8]; 256],
    i: usize,
    packed: &[u8],
    fill: u8,
) -> [u8; 8] {
    let pop = (i as u8).count_ones() as usize;
    assert_eq!(packed.len(), pop, "packed length must equal popcount");
    let offset = 8 - pop;
    let mut out = [fill; 8];
    for k in 0..pop {
        let bit = pc[i][offset + k] as usize;
        out[bit] = packed[k];
    }
    out
}

#[test]
fn permute_shuffles_bytes_correctly() {
    let p = build_permute();
    let input: [u8; 8] = [b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h'];
    // mask 0b10101010 = 170 keeps bits 1,3,5,7 -> [b,d,f,h]
    assert_eq!(apply_permute(&p, 0b10101010, &input), b"bdfh".to_vec());
    // mask 0b01010101 = 85 keeps bits 0,2,4,6 -> [a,c,e,g]
    assert_eq!(apply_permute(&p, 0b01010101, &input), b"aceg".to_vec());
    // mask 0xff = 255 keeps everything -> "abcdefgh"
    assert_eq!(apply_permute(&p, 0xff, &input), b"abcdefgh".to_vec());
    // mask 0 keeps nothing -> empty
    assert_eq!(apply_permute(&p, 0, &input), Vec::<u8>::new());
    // mask 0b00010001 = 17 keeps bits 0,4 -> [a,e]
    assert_eq!(apply_permute(&p, 0b00010001, &input), b"ae".to_vec());
    // mask 0b11110000 = 240 keeps bits 4..7 -> [e,f,g,h]
    assert_eq!(apply_permute(&p, 0b11110000, &input), b"efgh".to_vec());
    // mask 0b00001111 = 15 keeps bits 0..3 -> [a,b,c,d]
    assert_eq!(apply_permute(&p, 0b00001111, &input), b"abcd".to_vec());
}

#[test]
fn permutec_unshuffle_round_trips_with_permute() {
    let p = build_permute();
    let pc = build_permutec();
    let input: [u8; 8] = [10, 20, 30, 40, 50, 60, 70, 80];
    let fill: u8 = 0xCC;
    for i in 0..256usize {
        let packed = apply_permute(&p, i, &input);
        let unshuffled = apply_permutec(&pc, i, &packed, fill);
        // Every set-bit slot should hold the original byte; every cleared
        // slot should be untouched (= fill).
        for j in 0..8 {
            if (i & (1 << j)) != 0 {
                assert_eq!(
                    unshuffled[j], input[j],
                    "round-trip failed for i={i} bit {j}"
                );
            } else {
                assert_eq!(
                    unshuffled[j], fill,
                    "permutec wrote to cleared slot i={i} j={j}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 5. main() output stability (key markers, not bytewise)
// ---------------------------------------------------------------------------

/// Runs `super::main()` with fd 1 redirected to a temp file at the fd level,
/// then returns (return code, captured stdout bytes).
///
/// We swap fd 1 (STDOUT_FILENO) instead of using `freopen` so we don't
/// disturb the `FILE*` cached by libc — that FILE* is shared with cargo's
/// test harness and replacing it under freopen can deadlock the harness
/// when it later tries to print captured output.
fn capture_main_stdout() -> (i32, Vec<u8>) {
    let pid = unsafe { libc::getpid() };
    // Include the thread id so concurrent test threads don't collide.
    let tid = std::thread::current().id();
    let path = format!("/tmp/permute_main_test_{}_{:?}.out\0", pid, tid);

    let rc;
    let mut buf;
    unsafe {
        libc::fflush(crate::htslib_rs::c_compat::stdout.cast());

        let out_fd = libc::open(
            path.as_ptr() as *const libc::c_char,
            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            0o644,
        );
        assert!(out_fd >= 0, "open(temp) failed");

        let saved_fd = libc::dup(libc::STDOUT_FILENO);
        assert!(saved_fd >= 0, "dup(stdout) failed");
        libc::dup2(out_fd, libc::STDOUT_FILENO);
        libc::close(out_fd);

        rc = super::main();
        libc::fflush(crate::htslib_rs::c_compat::stdout.cast());

        let len = libc::lseek(libc::STDOUT_FILENO, 0, libc::SEEK_CUR);
        assert!(len >= 0, "lseek failed");
        let len = len as usize;

        libc::dup2(saved_fd, libc::STDOUT_FILENO);
        libc::close(saved_fd);

        // Read the captured bytes back from the temp file.
        buf = vec![0u8; len];
        let rd_fd = libc::open(
            path.as_ptr() as *const libc::c_char,
            libc::O_RDONLY,
        );
        assert!(rd_fd >= 0, "open(temp read) failed");
        let mut off = 0usize;
        while off < len {
            let n = libc::read(
                rd_fd,
                buf.as_mut_ptr().add(off) as *mut libc::c_void,
                len - off,
            );
            assert!(n > 0, "read returned {n}");
            off += n as usize;
        }
        libc::close(rd_fd);
        libc::unlink(path.as_ptr() as *const libc::c_char);
    }

    (rc, buf)
}

/// Original existing test, preserved: main runs and emits a reasonable amount
/// of output.
#[test]
fn permute_main_runs_and_emits_output() {
    let (rc, buf) = capture_main_stdout();
    assert_eq!(rc, 0, "main() should return 0");
    assert!(
        buf.len() > 1024,
        "main() should emit > 1024 bytes of output, got {}",
        buf.len()
    );
}

#[test]
fn permute_main_emits_expected_markers() {
    let (rc, buf) = capture_main_stdout();
    assert_eq!(rc, 0);
    let s = String::from_utf8_lossy(&buf);
    // Don't bytewise-compare the whole output (~26 KB) — it embeds
    // `__FILE__`-derived path snippets via the source-echo prelude, which
    // are fragile across checkouts. Just pin the key C-source markers
    // emitted by the printer in `permute.rs`.
    assert!(
        s.contains("#define _ 9\n"),
        "missing `#define _ 9` marker"
    );
    assert!(
        s.contains("static uint32_t permute[256][8]"),
        "missing decode-table header"
    );
    assert!(
        s.contains("static uint32_t permutec[256][8]"),
        "missing encode-table header"
    );
    // Each table is terminated by "};\n". Expect at least two such
    // terminators (one per table; the source-echo prelude may contribute
    // more from the bundled header but never fewer).
    let term_count = s.matches("};\n").count();
    assert!(
        term_count >= 2,
        "expected at least 2 `}};` table terminators, got {term_count}"
    );
    // The "reverse binary bit order" comment appears twice, once per table.
    let rbb_count = s.matches("reverse binary bit order").count();
    assert!(
        rbb_count >= 2,
        "expected at least 2 `reverse binary bit order` markers, got {rbb_count}"
    );
}

#[test]
fn permute_main_emits_specific_row_lines() {
    // A few golden rows must appear verbatim in the printed output, since
    // the printer uses the exact same algorithm as `build_permute` /
    // `build_permutec` above. Pinning these guards against a future drift
    // where someone alters the print loop without updating the algorithm.
    let (_rc, buf) = capture_main_stdout();
    let s = String::from_utf8_lossy(&buf);
    // Decode table row 0 -> "  { _,_,_,_,_,_,_,_,},\n"
    assert!(
        s.contains("  { _,_,_,_,_,_,_,_,},\n"),
        "missing decode row 0"
    );
    // Decode table row 255 -> "  { 0,1,2,3,4,5,6,7,},\n"
    assert!(
        s.contains("  { 0,1,2,3,4,5,6,7,},\n"),
        "missing decode row 255 / encode row 255"
    );
    // Encode table row 1 -> "  { _,_,_,_,_,_,_,0,},\n"
    assert!(
        s.contains("  { _,_,_,_,_,_,_,0,},\n"),
        "missing encode row 1"
    );
    // Decode row 170 -> "  { _,0,_,1,_,2,_,3,},\n"
    assert!(
        s.contains("  { _,0,_,1,_,2,_,3,},\n"),
        "missing decode row 170"
    );
    // Encode row 170 -> "  { _,_,_,_,1,3,5,7,},\n"
    assert!(
        s.contains("  { _,_,_,_,1,3,5,7,},\n"),
        "missing encode row 170"
    );
}

// ---------------------------------------------------------------------------
// 6. `#define _ 9` consistency  (and exposed UNDERSCORE)
// ---------------------------------------------------------------------------

#[test]
fn underscore_constant_matches_c_macro() {
    // Both the local test sentinel and the publicly exported constant must
    // match the C macro `#define _ 9`.
    assert_eq!(UNDERSCORE_LOCAL, 9);
    assert_eq!(super::UNDERSCORE, 9);
}

// ---------------------------------------------------------------------------
// Bonus: full-table consistency between the two builders
// ---------------------------------------------------------------------------

#[test]
fn full_table_inverse_round_trip_all_masks() {
    // Strongest end-to-end check: for every mask, every input bit position,
    // applying permute then permutec recovers the original slot positions
    // for the set bits and never touches the cleared slots.
    let p = build_permute();
    let pc = build_permutec();
    // Use a byte pattern with all distinct values so any cross-talk shows up.
    let input: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
    let fill: u8 = 0;
    for i in 0..256usize {
        let packed = apply_permute(&p, i, &input);
        // Packed length must equal popcount.
        assert_eq!(packed.len(), (i as u8).count_ones() as usize);
        let restored = apply_permutec(&pc, i, &packed, fill);
        let mut expected = [0u8; 8];
        for j in 0..8 {
            if (i & (1 << j)) != 0 {
                expected[j] = input[j];
            }
        }
        assert_eq!(
            restored, expected,
            "round-trip mismatch for mask {i}: got {restored:?}, expected {expected:?}"
        );
    }
}

#[test]
fn super_permute_static_matches_test_local_builder() {
    // The const-fn in `permute.rs:build_permute()` and the test-local
    // `build_permute()` here are independent translations of the same C
    // algorithm (`permute.h:24-40`). If they ever drift, both this test
    // and any downstream user of `super::permute` will be affected; this
    // test is the single chokepoint where the drift surfaces.
    let local = build_permute();
    assert_eq!(super::permute, local);
}

#[test]
fn super_permutec_static_matches_test_local_builder() {
    let local = build_permutec();
    assert_eq!(super::permutec, local);
}
