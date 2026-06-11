/*  simd.c -- SIMD optimised versions of various internal functions.

    Copyright (C) 2024 Genome Research Ltd.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.  */

use crate::htslib_rs::sam;

const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

pub type Nibble2BaseFn = unsafe fn(&[u8], &mut [u8]);

pub static mut htslib_nibble2base: Nibble2BaseFn = sam::nibble2base_default;

// original: cpu_supports_neon (htslib/simd.c:75)
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub unsafe fn simd_c_75_cpu_supports_neon() -> i32 {
    #[cfg(target_arch = "arm")]
    {
        if std::arch::is_arm_feature_detected!("neon") {
            1
        } else {
            0
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            1
        } else {
            0
        }
    }
}

fn nibble2base_default_slice(nib: &[u8], seq: &mut [u8]) {
    for (i, out) in seq.iter_mut().enumerate() {
        let byte = nib[i / 2];
        let code = if i % 2 == 0 { byte >> 4 } else { byte & 0x0f };
        *out = SEQ_NT16_STR[code as usize];
    }
}

/*
 * Convert a nibble encoded BAM sequence to a string of bases.
 *
 * Using SSSE3 instructions, 16 codepoints that hold 2 bases each can be
 * unpacked into 32 indexes from 0-15. Using the pshufb instruction these can
 * be converted to the IUPAC characters.
 * It falls back on the nibble2base_default function for the remainder.
 */
#[target_feature(enable = "ssse3")]
// original: nibble2base_ssse3 (htslib/simd.c:132)
#[cfg(target_arch = "x86_64")]
unsafe fn nibble2base_ssse3_slice(nib: &[u8], seq: &mut [u8]) {
    use std::arch::x86_64::{
        __m128i, _mm_and_si128, _mm_lddqu_si128, _mm_set1_epi8, _mm_shuffle_epi8, _mm_srli_epi64,
        _mm_storeu_si128, _mm_unpackhi_epi8, _mm_unpacklo_epi8,
    };

    let block_in_len = std::mem::size_of::<__m128i>();
    let block_out_len = 2 * block_in_len;
    let full_blocks_len = seq.len() / block_out_len * block_out_len;
    let nuc_lookup_vec = _mm_lddqu_si128(SEQ_NT16_STR.as_ptr().cast::<__m128i>());
    /* Nucleotides are encoded 4-bits per nucleotide and stored in 8-bit bytes
    as follows: |AB|CD|EF|GH|. The 4-bit codes (going from 0-15) can be used
    together with the pshufb instruction as a lookup table. The most efficient
    way is to use bitwise AND and shift to create two vectors. One with all
    the upper codes (|A|C|E|G|) and one with the lower codes (|B|D|F|H|).
    The lookup can then be performed and the resulting vectors can be
    interleaved again using the unpack instructions. */
    for (encoded_chunk, seq_chunk) in nib[..full_blocks_len / 2]
        .chunks_exact(block_in_len)
        .zip(seq[..full_blocks_len].chunks_exact_mut(block_out_len))
    {
        let encoded = _mm_lddqu_si128(encoded_chunk.as_ptr().cast::<__m128i>());
        let mut encoded_upper = _mm_srli_epi64(encoded, 4);
        encoded_upper = _mm_and_si128(encoded_upper, _mm_set1_epi8(15));
        let encoded_lower = _mm_and_si128(encoded, _mm_set1_epi8(15));
        let nucs_upper = _mm_shuffle_epi8(nuc_lookup_vec, encoded_upper);
        let nucs_lower = _mm_shuffle_epi8(nuc_lookup_vec, encoded_lower);
        let first_nucleotides = _mm_unpacklo_epi8(nucs_upper, nucs_lower);
        let second_nucleotides = _mm_unpackhi_epi8(nucs_upper, nucs_lower);
        _mm_storeu_si128(seq_chunk.as_mut_ptr().cast::<__m128i>(), first_nucleotides);
        _mm_storeu_si128(
            seq_chunk[block_in_len..].as_mut_ptr().cast::<__m128i>(),
            second_nucleotides,
        );
    }

    nibble2base_default_slice(&nib[full_blocks_len / 2..], &mut seq[full_blocks_len..]);
}

// original: nibble2base_ssse3 (htslib/simd.c:132)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3")]
pub unsafe fn simd_c_132_nibble2base_ssse3(nib: &[u8], seq: &mut [u8]) {
    unsafe { nibble2base_ssse3_slice(nib, seq) };
}

// original: nibble2base_resolve (htslib/simd.c:164)
#[cfg(target_arch = "x86_64")]
pub unsafe fn simd_c_164_nibble2base_resolve() {
    if std::arch::is_x86_feature_detected!("ssse3") {
        unsafe {
            htslib_nibble2base = simd_c_132_nibble2base_ssse3;
        }
    }
}

// original: nibble2base_neon (htslib/simd.c:173)
#[cfg(all(
    any(target_arch = "arm", target_arch = "aarch64"),
    target_feature = "neon"
))]
unsafe fn nibble2base_neon_slice(nib: &[u8], seq0: &mut [u8]) {
    #[cfg(target_arch = "aarch64")]
    {
        use std::arch::aarch64::{
            uint8x16x2_t, vandq_u8, vdupq_n_u8, vld1q_u8, vqtbl1q_u8, vshrq_n_u8, vst1q_u8_x2,
            vzipq_u8,
        };

        let low_nibbles_mask = vdupq_n_u8(0x0f);
        let nuc_lookup_vec = vld1q_u8(SEQ_NT16_STR.as_ptr());
        let full_blocks_len = seq0.len() / 32 * 32;

        for (encoded_chunk, seq_chunk) in nib[..full_blocks_len / 2]
            .chunks_exact(16)
            .zip(seq0[..full_blocks_len].chunks_exact_mut(32))
        {
            let encoded = vld1q_u8(encoded_chunk.as_ptr());

            /* Translate the high and low nibbles to nucleotide letters separately,
            then interleave them back together via vzipq for writing. */
            let high_nibbles = vshrq_n_u8(encoded, 4);
            let low_nibbles = vandq_u8(encoded, low_nibbles_mask);

            let high_nucleotides = vqtbl1q_u8(nuc_lookup_vec, high_nibbles);
            let low_nucleotides = vqtbl1q_u8(nuc_lookup_vec, low_nibbles);

            let nucleotides: uint8x16x2_t = vzipq_u8(high_nucleotides, low_nucleotides);
            vst1q_u8_x2(seq_chunk.as_mut_ptr().cast::<u8>(), nucleotides);
        }

        nibble2base_default_slice(&nib[full_blocks_len / 2..], &mut seq0[full_blocks_len..]);
    }

    #[cfg(target_arch = "arm")]
    {
        use std::arch::arm::{
            uint8x16x2_t, uint8x8x2_t, vandq_u8, vcombine_u8, vdupq_n_u8, vget_high_u8,
            vget_low_u8, vld1q_u8, vshrq_n_u8, vst2q_u8, vtbl2_u8,
        };

        let low_nibbles_mask = vdupq_n_u8(0x0f);
        let nuc_lookup_vec = vld1q_u8(SEQ_NT16_STR.as_ptr());
        let nuc_lookup_vec2 =
            uint8x8x2_t(vget_low_u8(nuc_lookup_vec), vget_high_u8(nuc_lookup_vec));
        let full_blocks_len = seq0.len() / 32 * 32;

        for (encoded_chunk, seq_chunk) in nib[..full_blocks_len / 2]
            .chunks_exact(16)
            .zip(seq0[..full_blocks_len].chunks_exact_mut(32))
        {
            let encoded = vld1q_u8(encoded_chunk.as_ptr());

            /* Translate the high and low nibbles to nucleotide letters separately,
            then interleave them back together via vzipq for writing. */
            let high_nibbles = vshrq_n_u8(encoded, 4);
            let low_nibbles = vandq_u8(encoded, low_nibbles_mask);

            let high_low = vtbl2_u8(nuc_lookup_vec2, vget_low_u8(high_nibbles));
            let high_high = vtbl2_u8(nuc_lookup_vec2, vget_high_u8(high_nibbles));
            let high_nucleotides = vcombine_u8(high_low, high_high);

            let low_low = vtbl2_u8(nuc_lookup_vec2, vget_low_u8(low_nibbles));
            let low_high = vtbl2_u8(nuc_lookup_vec2, vget_high_u8(low_nibbles));
            let low_nucleotides = vcombine_u8(low_low, low_high);

            // Avoid vst1q_u8_x2 as GCC erroneously omits it on 32-bit ARM
            let nucleotides = uint8x16x2_t(high_nucleotides, low_nucleotides);
            vst2q_u8(seq_chunk.as_mut_ptr().cast::<u8>(), nucleotides);
        }

        nibble2base_default_slice(&nib[full_blocks_len / 2..], &mut seq0[full_blocks_len..]);
    }
}

// original: nibble2base_neon (htslib/simd.c:173)
#[cfg(all(
    any(target_arch = "arm", target_arch = "aarch64"),
    target_feature = "neon"
))]
pub unsafe fn simd_c_173_nibble2base_neon(nib: &[u8], seq0: &mut [u8]) {
    unsafe { nibble2base_neon_slice(nib, seq0) };
}

// original: nibble2base_resolve (htslib/simd.c:220)
#[cfg(all(
    any(target_arch = "arm", target_arch = "aarch64"),
    target_feature = "neon"
))]
pub unsafe fn simd_c_220_nibble2base_resolve() {
    if unsafe { simd_c_75_cpu_supports_neon() } != 0 {
        unsafe {
            htslib_nibble2base = simd_c_173_nibble2base_neon;
        }
    }
}

pub static htslib_simd: &[u8] = b"SIMD functions present: nibble2base.\0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simd_lookup_table_matches_bam_nibble_codes() {
        assert_eq!(SEQ_NT16_STR, b"=ACMGRSVTWYHKDBN");
        assert_eq!(SEQ_NT16_STR[0], b'=');
        assert_eq!(SEQ_NT16_STR[15], b'N');
        assert_eq!(htslib_simd, b"SIMD functions present: nibble2base.\0");
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn ssse3_nibble2base_matches_lookup_for_block_and_tail_lengths() {
        if !std::arch::is_x86_feature_detected!("ssse3") {
            return;
        }

        unsafe {
            let packed = [
                0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba,
                0xdc, 0xfe, 0x11, 0x22, 0x33, 0x44,
            ];
            let len = 39;
            let mut seq = [0u8; 39];
            let mut expected = [0u8; 39];

            for i in 0..len {
                let byte = packed[i / 2];
                let nibble = if i % 2 == 0 { byte >> 4 } else { byte & 0x0f };
                expected[i] = SEQ_NT16_STR[nibble as usize];
            }

            simd_c_132_nibble2base_ssse3(&packed[..len.div_ceil(2)], &mut seq[..len]);
            assert_eq!(seq[..len], expected[..len]);
        }
    }
}
