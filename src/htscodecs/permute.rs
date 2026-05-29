//! Translation of htscodecs `permute.h` (header-only).
//!
//! Lookup tables for SIMD permutation: shuffle bytes based on input bits
//! (bit N true => keep Nth byte; bit N false => skip Nth byte).

/// ```c
/// #define _ 9
/// ```
/// Placeholder lane value used in the permute tables.
// permute.h:88
pub const UNDERSCORE: u32 = 9;

/// ```c
/// int main(void);
/// ```
/// Table generator, compiled only under `#ifdef MAIN`.
///
/// 1:1 translation of the C `main(void)` in `permute.h`:
///   * Opens `__FILE__` and copies the source through to stdout. The C uses
///     the preprocessor macro `__FILE__` which expands to the path of the
///     source file being compiled. Rust has no equivalent macro that survives
///     into a runtime `fopen()`, so we hard-code the bundled C header path
///     (`htslib/htscodecs/htscodecs/permute.h`) here: when this program is
///     run from the crate root that file is reachable and its byte stream
///     is identical to what the C tool would emit (the bundled header is
///     the original C source).
///   * Emits the decode `permute[256][8]` table (reverse binary bit order).
///   * Emits the encode `permutec[256][8]` table.
///
/// `printf` format strings are kept byte-for-byte identical to the C so the
/// generated output is byte-identical to the C tool's output.
// permute.h:10
pub fn main() -> i32 {
    // SAFETY: libc FFI; pointers are valid for the duration of the calls.
    unsafe {
        // FILE *fp = fopen(__FILE__, "r");
        // Use the bundled C source path; see doc comment above.
        let path = c"htslib/htscodecs/htscodecs/permute.h";
        let fp = libc::fopen(path.as_ptr(), c"r".as_ptr());
        // The C does not check fp for NULL before fgets(); a NULL would
        // crash. We mirror that behaviour but guard the dereference to
        // keep the Rust function panic-free if the file is missing.
        if !fp.is_null() {
            let mut line = [0 as libc::c_char; 8192];
            // while(fgets(line, 8192, fp)) { printf("%s", line); }
            while !libc::fgets(line.as_mut_ptr(), 8192, fp).is_null() {
                libc::printf(c"%s".as_ptr(), line.as_ptr());
            }
            // The C calls `close(fp)` (a typo for `fclose`); fclose is the
            // correct call for a FILE*, so we use it here.
            libc::fclose(fp);
        }
        libc::printf(c"\n".as_ptr());

        // Decode table; distributes N adjacent values across lanes
        libc::printf(c"#define _ 9\n".as_ptr());
        libc::printf(
            c"static uint32_t permute[256][8] = { // reverse binary bit order\n".as_ptr(),
        );
        let mut i: libc::c_int = 0;
        while i < 256 {
            let mut b: libc::c_int = 0;
            let mut v: [libc::c_int; 8] = [0; 8];
            let mut j: libc::c_int = 0;
            while j < 8 {
                if (i & (1 << j)) != 0 {
                    b += 1;
                    v[j as usize] = b;
                }
                j += 1;
            }
            libc::printf(c"  { ".as_ptr());
            let mut j: libc::c_int = 0;
            while j < 8 {
                if v[j as usize] != 0 {
                    libc::printf(c"%d,".as_ptr(), v[j as usize] - 1);
                } else {
                    libc::printf(c"_,".as_ptr());
                }
                j += 1;
            }
            libc::printf(c"},\n".as_ptr());
            i += 1;
        }
        libc::printf(c"};\n\n".as_ptr());

        // Encode table; collapses N values spread across lanes
        libc::printf(
            c"static uint32_t permutec[256][8] = { // reverse binary bit order\n".as_ptr(),
        );
        let mut i: libc::c_int = 0;
        while i < 256 {
            let mut b: libc::c_int = 0;
            let mut v: [libc::c_int; 9] = [0; 9];
            let mut j: libc::c_int = 0;
            while j < 8 {
                if (i & (1 << j)) != 0 {
                    v[b as usize] = j + 1;
                    b += 1;
                }
                j += 1;
            }
            libc::printf(c"  { ".as_ptr());
            let mut j: libc::c_int = b - 8;
            while j < b {
                if j >= 0 && v[j as usize] != 0 {
                    libc::printf(c"%d,".as_ptr(), v[j as usize] - 1);
                } else {
                    libc::printf(c"_,".as_ptr());
                }
                j += 1;
            }
            libc::printf(c"},\n".as_ptr());
            i += 1;
        }
        libc::printf(c"};\n".as_ptr());

        0
    }
}

/// ```c
/// static uint32_t permute[256][8] __attribute__((aligned(32)));
/// ```
/// Decode table; distributes N adjacent values across lanes (reverse binary
/// bit order). Computed at compile time from the same algorithm the C
/// `#ifdef MAIN` program in `permute.h` uses to print the literal table:
/// for each mask `i`, walk `j = 0..8`; if `i & (1 << j)` set then
/// `permute[i][j] = ++b - 1` (0-indexed slot), else `permute[i][j] = _ = 9`.
// permute.h:89
pub static permute: [[u32; 8]; 256] = build_permute();

/// ```c
/// static uint32_t permutec[256][8] __attribute__((aligned(32)));
/// ```
/// Encode table; collapses N values spread across lanes (reverse binary bit
/// order). Right-aligned inverse of `permute`: for each mask `i`, let
/// `b = popcount(i)` and `offset = 8 - b`; lanes `0..offset` are `_ = 9`,
/// lanes `offset..8` hold the indices of `i`'s set bits in increasing order.
// permute.h:348
pub static permutec: [[u32; 8]; 256] = build_permutec();

// const-fn translations of the C `#ifdef MAIN` table generators (permute.h:24-43
// for the decode table, permute.h:46-62 for the encode table). Evaluated once
// at compile time; no runtime cost. Tested in `permute/tests.rs`.
const fn build_permute() -> [[u32; 8]; 256] {
    let mut t = [[UNDERSCORE; 8]; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut b: u32 = 0;
        let mut j = 0usize;
        while j < 8 {
            if (i & (1 << j)) != 0 {
                t[i][j] = b;
                b += 1;
            }
            j += 1;
        }
        i += 1;
    }
    t
}

const fn build_permutec() -> [[u32; 8]; 256] {
    let mut t = [[UNDERSCORE; 8]; 256];
    let mut i = 0usize;
    while i < 256 {
        // Collect set-bit positions of i in increasing order.
        let mut bits = [0u32; 8];
        let mut b: usize = 0;
        let mut j = 0usize;
        while j < 8 {
            if (i & (1 << j)) != 0 {
                bits[b] = j as u32;
                b += 1;
            }
            j += 1;
        }
        // Right-align: lanes 0..(8-b) are the sentinel; lanes (8-b)..8 are bits[0..b].
        let offset = 8 - b;
        let mut k = 0usize;
        while k < b {
            t[i][offset + k] = bits[k];
            k += 1;
        }
        i += 1;
    }
    t
}

#[cfg(test)]
mod tests;
