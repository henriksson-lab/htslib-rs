/*
Copyright (c) 2013-2020, 2023-2024 Genome Research Ltd.
Author: James Bonfield <jkb@sanger.ac.uk>

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

   1. Redistributions of source code must retain the above copyright notice,
this list of conditions and the following disclaimer.

   2. Redistributions in binary form must reproduce the above copyright notice,
this list of conditions and the following disclaimer in the documentation
and/or other materials provided with the distribution.

   3. Neither the names Genome Research Ltd and Wellcome Trust Sanger
Institute nor the names of its contributors may be used to endorse or promote
products derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY GENOME RESEARCH LTD AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL GENOME RESEARCH LTD OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

//! Native, non-variadic shims for the CRAM option setter
//! `cram_set_option(cram_fd*, opt, ...)` from `htslib/cram/cram_io.c:5674`.
//!
//! Rust does not support stable C-style variadic *definitions*, so we cannot
//! translate `cram_set_option` directly. The C body packages its single
//! trailing argument into a `va_list` and calls `cram_set_voption` — which
//! is already natively translated in `crate::htslib_rs::cram` as
//! `cram_cram_io_c_5692_cram_set_voption`.
//!
//! Every production call site of `cram_set_option` in `src/hts.rs` passes
//! exactly one trailing argument (either a `c_int` or a `*mut`-shaped
//! pointer), matching the call-shape used by the C HTSlib code paths that
//! route here. We therefore provide two thin shims — `cram_set_option_int`
//! and `cram_set_option_ptr` — each of which synthesises a single-word
//! SysV-AMD64 `__va_list_tag` and dispatches into the native
//! `cram_set_voption`.
//!
//! The synthetic-`va_list` layout follows the established pattern at
//! `src/hts.rs:5988-6011` (hopen / crypt4gh "parent" key) and
//! `src/hfile_s3.rs:2779-2798` (`hfile_s3_hopen_vargs`): the first 6
//! pointer-sized GP arguments go in `reg_save_area` (matched by
//! `gp_offset = 0`), with `fp_offset = 48` to skip the XMM save area, and
//! anything beyond goes in `overflow_arg_area`. The native `cram_set_voption`
//! reads its variadic argument via `cram_voption_va_arg_word`
//! (`src/cram.rs:1474-1492`), which honours this same layout, so a single
//! word in `reg_save_area[0]` is exactly what it expects for the single
//! `va_arg(args, int)` / `va_arg(args, T*)` consumed per supported option.

use std::ffi::{c_int, c_void};
use std::ptr::NonNull;

use crate::htslib_rs::c_compat::__va_list_tag;
use crate::htslib_rs::cram::cram_cram_io_c_5692_cram_set_voption;
use crate::htslib_rs::hts::{cram_fd, hts_fmt_option};

/// Build a synthetic SysV-AMD64 `__va_list_tag` carrying `words` as the
/// successive pointer-sized varargs, then invoke the native
/// `cram_set_voption`. The buffers backing the va_list must outlive the
/// callee, so we keep them on the stack here and pass `&mut` to keep them
/// alive for the duration of the call.
//
// Pattern follows `src/hts.rs:5988-6011` and `src/hfile_s3.rs:2779-2798`.
#[inline]
unsafe fn dispatch_voption(fd: &mut cram_fd, opt: hts_fmt_option, words: &[usize]) -> c_int {
    // The SysV AMD64 ABI reserves space for 6 general-purpose register args
    // in `reg_save_area`; anything beyond spills into `overflow_arg_area`.
    let mut reg_save = [0usize; 6];
    let mut overflow = vec![0usize; words.len().saturating_sub(reg_save.len())];
    for (i, word) in words.iter().copied().enumerate() {
        if i < reg_save.len() {
            reg_save[i] = word;
        } else {
            overflow[i - reg_save.len()] = word;
        }
    }
    let mut va = __va_list_tag {
        gp_offset: 0,
        fp_offset: 48,
        overflow_arg_area: overflow.as_mut_ptr().cast(),
        reg_save_area: reg_save.as_mut_ptr().cast(),
    };
    cram_cram_io_c_5692_cram_set_voption(fd as *mut cram_fd, opt, &mut va)
}

// htslib/cram/cram_io.c:5674
//
// Native shim for `cram_set_option(cram_fd*, opt, int)`. Matches the call
// shape used by every integer-typed CRAM option that hts.rs sets via the
// variadic setter (decode_md flag, compression level, nthreads, etc.).
// The C body just packages its `int` arg into a va_list and calls
// `cram_set_voption`; we do the same with a synthetic SysV va_list.
///
/// # Safety
///
/// `fd` must be a valid CRAM file handle accepted by
/// `cram_set_voption`. `opt` must name an option whose variadic argument is
/// an integer, and the native callee must not retain the synthetic va_list
/// beyond this call.
pub unsafe fn cram_set_option_int(fd: &mut cram_fd, opt: hts_fmt_option, val: c_int) -> c_int {
    // `int` widens to `usize` via a zero-extending cast through the unsigned
    // 32-bit value, matching how the SysV ABI promotes int → 8-byte slot in
    // the GP save area (which is what `cram_voption_va_arg_word` reads).
    let word = (val as u32) as usize;
    dispatch_voption(fd, opt, &[word])
}

// htslib/cram/cram_io.c:5674
//
// Native shim for `cram_set_option(cram_fd*, opt, void*)`. Matches the call
// shape used by every pointer-typed CRAM option that hts.rs sets via the
// variadic setter (CRAM_OPT_REFERENCE, CRAM_OPT_THREAD_POOL, and the generic
// `hts_set_opt_ptr` fall-through to CRAM). Mirrors the integer shim — same
// SysV layout, just a pointer-sized word.
///
/// # Safety
///
/// `fd` must be a valid CRAM file handle accepted by
/// `cram_set_voption`. `opt` must name an option whose variadic argument is a
/// pointer, `val` must satisfy that option's pointer validity and lifetime
/// requirements, and the native callee must not retain the synthetic va_list
/// beyond this call.
pub unsafe fn cram_set_option_ptr(
    fd: &mut cram_fd,
    opt: hts_fmt_option,
    val: Option<NonNull<c_void>>,
) -> c_int {
    let word = val.map_or(0, |val| val.as_ptr() as usize);
    dispatch_voption(fd, opt, &[word])
}
