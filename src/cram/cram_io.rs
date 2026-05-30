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

use crate::htslib_rs::c_compat::malloc;
use std::ffi::{c_char, c_int, c_void};
use std::io::{Read, Write};

// Stubs without a native equivalent yet still delegate to libhts via these
// link_name externs. The flush/write_eof/read_SAM_hdr externs were removed
// once their native equivalents in src/cram.rs and src/cram_flush_bridge.rs
// took over.
unsafe extern "C" {
    #[link_name = "cram_write_SAM_hdr"]
    fn htslib_cram_write_sam_hdr(fd: *mut hts_sys::cram_fd, hdr: *mut hts_sys::sam_hdr_t) -> c_int;
}

unsafe fn copy_vec_to_malloc(mut out: Vec<u8>, out_size: *mut usize) -> *mut c_char {
    *out_size = out.len();
    let p = malloc(out.len().max(1) as u64).cast::<u8>();
    if p.is_null() {
        return std::ptr::null_mut();
    }
    if !out.is_empty() {
        std::ptr::copy_nonoverlapping(out.as_ptr(), p, out.len());
    }
    out.clear();
    p.cast()
}

// original: libdeflate_deflate (htslib/cram/cram_io.c:1113)
pub unsafe fn cram_cram_io_c_1113_libdeflate_deflate(
    data: *mut c_char,
    size: usize,
    cdata_size: *mut usize,
    level: c_int,
    strat: c_int,
) -> *mut c_char {
    cram_cram_io_c_1222_zlib_mem_deflate(data, size, cdata_size, level, strat)
}

// original: zlib_mem_deflate (htslib/cram/cram_io.c:1222)
pub unsafe fn cram_cram_io_c_1222_zlib_mem_deflate(
    data: *mut c_char,
    size: usize,
    cdata_size: *mut usize,
    level: c_int,
    _strat: c_int,
) -> *mut c_char {
    *cdata_size = 0;
    let input = std::slice::from_raw_parts(data.cast::<u8>(), size);
    let compression = flate2::Compression::new(level.clamp(0, 9) as u32);
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), compression);
    if encoder.write_all(input).is_err() {
        return std::ptr::null_mut();
    }
    match encoder.finish() {
        Ok(compressed) => copy_vec_to_malloc(compressed, cdata_size),
        Err(_) => std::ptr::null_mut(),
    }
}

// original: lzma_mem_deflate (htslib/cram/cram_io.c:1295)
pub unsafe fn cram_cram_io_c_1295_lzma_mem_deflate(
    data: *mut c_char,
    size: usize,
    cdata_size: *mut usize,
    level: c_int,
) -> *mut c_char {
    *cdata_size = 0;
    let input = std::slice::from_raw_parts(data.cast::<u8>(), size);
    let mut encoder = xz2::write::XzEncoder::new(Vec::new(), level as u32);
    if encoder.write_all(input).is_err() {
        return std::ptr::null_mut();
    }
    match encoder.finish() {
        Ok(compressed) => copy_vec_to_malloc(compressed, cdata_size),
        Err(_) => std::ptr::null_mut(),
    }
}

// original: lzma_mem_inflate (htslib/cram/cram_io.c:1313)
pub unsafe fn cram_cram_io_c_1313_lzma_mem_inflate(
    cdata: *mut c_char,
    csize: usize,
    size: *mut usize,
) -> *mut c_char {
    *size = 0;
    let input = std::slice::from_raw_parts(cdata.cast::<u8>(), csize);
    let mut decoder = xz2::read::XzDecoder::new(std::io::Cursor::new(input));
    let mut decoded = Vec::new();
    if decoder.read_to_end(&mut decoded).is_err() {
        return std::ptr::null_mut();
    }
    copy_vec_to_malloc(decoded, size)
}

// original: cram_compress_by_method (htslib/cram/cram_io.c:1756)
pub unsafe fn cram_cram_io_c_1756_cram_compress_by_method(
    _s: *mut hts_sys::cram_slice,
    in_: *mut c_char,
    in_size: usize,
    _content_id: c_int,
    out_size: *mut usize,
    method: c_int,
    level: c_int,
    strat: c_int,
) -> *mut c_char {
    match method {
        x if x == crate::htslib_rs::cram::CRAM_BLOCK_METHOD_GZIP => {
            cram_cram_io_c_1222_zlib_mem_deflate(in_, in_size, out_size, level, strat)
        }
        x if x == crate::htslib_rs::cram::CRAM_BLOCK_METHOD_LZMA => {
            cram_cram_io_c_1295_lzma_mem_deflate(in_, in_size, out_size, level)
        }
        _ => std::ptr::null_mut(),
    }
}

// original: cram_compress_block3 (htslib/cram/cram_io.c:1912)
pub unsafe fn cram_cram_io_c_1912_cram_compress_block3(
    fd: *mut hts_sys::cram_fd,
    _s: *mut hts_sys::cram_slice,
    b: *mut hts_sys::cram_block,
    metrics: *mut hts_sys::cram_metrics,
    method: c_int,
    level: c_int,
    _recurse: c_int,
) -> c_int {
    // Native cram_compress_block3 is module-private; route via the
    // public cram_compress_block entry (the slice arg is unused here).
    crate::htslib_rs::cram::cram_cram_io_c_2323_cram_compress_block(
        fd.cast(),
        b,
        metrics,
        method,
        level,
    )
}

// original: cram_compress_block2 (htslib/cram/cram_io.c:2316)
pub unsafe fn cram_cram_io_c_2316_cram_compress_block2(
    fd: *mut hts_sys::cram_fd,
    s: *mut hts_sys::cram_slice,
    b: *mut hts_sys::cram_block,
    metrics: *mut hts_sys::cram_metrics,
    method: c_int,
    level: c_int,
) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_2317_cram_compress_block2(
        fd.cast(),
        s.cast(),
        b,
        metrics,
        method,
        level,
    )
}

// original: cram_compress_block (htslib/cram/cram_io.c:2322)
pub unsafe fn cram_cram_io_c_2322_cram_compress_block(
    fd: *mut hts_sys::cram_fd,
    b: *mut hts_sys::cram_block,
    metrics: *mut hts_sys::cram_metrics,
    method: c_int,
    level: c_int,
) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_2323_cram_compress_block(
        fd.cast(),
        b,
        metrics,
        method,
        level,
    )
}

// original: cram_populate_ref (htslib/cram/cram_io.c:2977)
pub unsafe fn cram_cram_io_c_2977_cram_populate_ref(
    fd: *mut hts_sys::cram_fd,
    id: c_int,
    r: *mut c_void,
) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_2977_cram_populate_ref(fd.cast(), id, r)
}

// original: cram_get_ref (htslib/cram/cram_io.c:3409)
pub unsafe fn cram_cram_io_c_3409_cram_get_ref(
    fd: *mut hts_sys::cram_fd,
    id: c_int,
    start: hts_sys::hts_pos_t,
    end: hts_sys::hts_pos_t,
) -> *mut c_char {
    crate::htslib_rs::cram::cram_cram_io_c_3409_cram_get_ref(fd.cast(), id, start, end)
}

// original: cram_new_container (htslib/cram/cram_io.c:3637)
pub unsafe fn cram_cram_io_c_3637_cram_new_container(
    nrec: c_int,
    nslice: c_int,
) -> *mut hts_sys::cram_container {
    crate::htslib_rs::cram::cram_cram_io_c_3639_cram_new_container(nrec, nslice)
}

// original: cram_free_container (htslib/cram/cram_io.c:3703)
pub unsafe fn cram_cram_io_c_3703_cram_free_container(c: *mut hts_sys::cram_container) {
    crate::htslib_rs::cram::cram_cram_io_c_3705_cram_free_container(c)
}

// original: cram_read_container (htslib/cram/cram_io.c:3786)
pub unsafe fn cram_cram_io_c_3786_cram_read_container(
    fd: *mut hts_sys::cram_fd,
) -> *mut hts_sys::cram_container {
    crate::htslib_rs::cram::cram_cram_io_c_3788_cram_read_container(fd)
}

// original: cram_container_size (htslib/cram/cram_io.c:3945)
pub unsafe fn cram_cram_io_c_3945_cram_container_size(c: *mut hts_sys::cram_container) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_3947_cram_container_size(c)
}

// original: cram_store_container (htslib/cram/cram_io.c:3958)
pub unsafe fn cram_cram_io_c_3958_cram_store_container(
    fd: *mut hts_sys::cram_fd,
    c: *mut hts_sys::cram_container,
    dat: *mut c_char,
    size: *mut c_int,
) -> c_int {
    crate::htslib_rs::cram::cram_store_container(fd.cast(), c, dat, size)
}

// original: cram_write_container (htslib/cram/cram_io.c:4021)
pub unsafe fn cram_cram_io_c_4021_cram_write_container(
    fd: *mut hts_sys::cram_fd,
    h: *mut hts_sys::cram_container,
) -> c_int {
    crate::htslib_rs::cram::cram_write_container(fd.cast(), h)
}

// original: cram_flush_container2 (htslib/cram/cram_io.c:4087)
pub unsafe fn cram_cram_io_c_4087_cram_flush_container2(
    fd: *mut hts_sys::cram_fd,
    c: *mut hts_sys::cram_container,
) -> c_int {
    crate::cram_flush_bridge::cram_cram_io_c_4089_cram_flush_container2(fd.cast(), c)
}

// original: cram_flush_container (htslib/cram/cram_io.c:4141)
pub unsafe fn cram_cram_io_c_4141_cram_flush_container(
    fd: *mut hts_sys::cram_fd,
    c: *mut hts_sys::cram_container,
) -> c_int {
    crate::cram_flush_bridge::cram_cram_io_c_4143_cram_flush_container(fd.cast(), c)
}

#[repr(C)]
pub struct cram_job {
    pub fd: *mut hts_sys::cram_fd,
    pub c: *mut hts_sys::cram_container,
}

// original: cram_flush_thread (htslib/cram/cram_io.c:4154)
pub unsafe extern "C" fn cram_cram_io_c_4154_cram_flush_thread(arg: *mut c_void) -> *mut c_void {
    arg
}

// original: cram_flush_result (htslib/cram/cram_io.c:4166)
pub unsafe fn cram_cram_io_c_4166_cram_flush_result(_fd: *mut hts_sys::cram_fd) -> c_int {
    0
}

// original: cram_flush_container_mt (htslib/cram/cram_io.c:4273)
pub unsafe fn cram_cram_io_c_4273_cram_flush_container_mt(
    fd: *mut hts_sys::cram_fd,
    c: *mut hts_sys::cram_container,
) -> c_int {
    crate::cram_flush_bridge::cram_cram_io_c_4275_cram_flush_container_mt(fd.cast(), c)
}

// original: cram_free_slice_header (htslib/cram/cram_io.c:4409)
// Native body lives in src/cram.rs as cram_cram_io_c_4409_cram_free_slice_header.
pub unsafe fn cram_cram_io_c_4407_cram_free_slice_header(hdr: *mut hts_sys::cram_block_slice_hdr) {
    crate::htslib_rs::cram::cram_cram_io_c_4409_cram_free_slice_header(hdr)
}

// original: cram_free_slice (htslib/cram/cram_io.c:4421)
// Native body lives in src/cram.rs as cram_cram_io_c_4421_cram_free_slice.
pub unsafe fn cram_cram_io_c_4419_cram_free_slice(s: *mut hts_sys::cram_slice) {
    crate::htslib_rs::cram::cram_cram_io_c_4421_cram_free_slice(s)
}

// original: cram_new_slice (htslib/cram/cram_io.c:4506)
// Native body lives in src/cram.rs as cram_cram_io_c_4506_cram_new_slice.
pub unsafe fn cram_cram_io_c_4504_cram_new_slice(
    type_: hts_sys::cram_content_type,
    nrecs: c_int,
) -> *mut hts_sys::cram_slice {
    crate::htslib_rs::cram::cram_cram_io_c_4506_cram_new_slice(type_, nrecs)
}

// original: cram_read_slice (htslib/cram/cram_io.c:4568)
// Native body lives in src/cram.rs as cram_cram_io_c_4568_cram_read_slice.
pub unsafe fn cram_cram_io_c_4566_cram_read_slice(
    fd: *mut hts_sys::cram_fd,
) -> *mut hts_sys::cram_slice {
    crate::htslib_rs::cram::cram_cram_io_c_4568_cram_read_slice(fd)
}

// original: cram_read_SAM_hdr (htslib/cram/cram_io.c:4715)
pub unsafe fn cram_cram_io_c_4715_cram_read_SAM_hdr(
    fd: *mut hts_sys::cram_fd,
) -> *mut hts_sys::sam_hdr_t {
    crate::htslib_rs::cram::cram_cram_io_c_4717_cram_read_SAM_hdr(fd.cast()).cast()
}

// original: cram_write_SAM_hdr (htslib/cram/cram_io.c:4889)
// TODO(P5): no native cram_write_SAM_hdr yet
pub unsafe fn cram_cram_io_c_4889_cram_write_SAM_hdr(
    fd: *mut hts_sys::cram_fd,
    hdr: *mut hts_sys::sam_hdr_t,
) -> c_int {
    htslib_cram_write_sam_hdr(fd, hdr)
}

// original: cram_open (htslib/cram/cram_io.c:5262)
pub unsafe fn cram_cram_io_c_5262_cram_open(
    filename: *const c_char,
    mode: *const c_char,
) -> *mut hts_sys::cram_fd {
    crate::htslib_rs::cram::cram_cram_io_c_5264_cram_open(filename, mode).cast()
}

// original: cram_dopen (htslib/cram/cram_io.c:5287)
pub unsafe fn cram_cram_io_c_5287_cram_dopen(
    fp: *mut hts_sys::hFILE,
    filename: *const c_char,
    mode: *const c_char,
) -> *mut hts_sys::cram_fd {
    crate::htslib_rs::cram::cram_cram_io_c_5289_cram_dopen(fp.cast(), filename, mode).cast()
}

// original: cram_seek (htslib/cram/cram_io.c:5429)
pub unsafe fn cram_cram_io_c_5429_cram_seek(
    fd: *mut hts_sys::cram_fd,
    offset: libc::off_t,
    whence: c_int,
) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_5431_cram_seek(fd.cast(), offset, whence)
}

// original: cram_flush (htslib/cram/cram_io.c:5444)
pub unsafe fn cram_cram_io_c_5444_cram_flush(fd: *mut hts_sys::cram_fd) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_5446_cram_flush(fd.cast())
}

// original: cram_write_eof_block (htslib/cram/cram_io.c:5472)
pub unsafe fn cram_cram_io_c_5472_cram_write_eof_block(fd: *mut hts_sys::cram_fd) -> c_int {
    crate::cram_flush_bridge::cram_cram_io_c_5474_cram_write_eof_block(fd.cast())
}

// original: cram_close (htslib/cram/cram_io.c:5556)
pub unsafe fn cram_cram_io_c_5556_cram_close(fd: *mut hts_sys::cram_fd) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_5558_cram_close(fd.cast())
}

// original: cram_set_option (htslib/cram/cram_io.c:5674)
// Rust has no stable C-style variadic function definitions; use the translated
// va_list entry point below for callers that already have a va_list.

// original: cram_set_voption (htslib/cram/cram_io.c:5692)
pub unsafe fn cram_cram_io_c_5692_cram_set_voption(
    fd: *mut hts_sys::cram_fd,
    opt: hts_sys::hts_fmt_option,
    args: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_5692_cram_set_voption(fd.cast(), opt, args)
}

// original: cram_check_EOF (htslib/cram/cram_io.c:5958)
pub unsafe fn cram_cram_io_c_5958_cram_check_EOF(fd: *mut hts_sys::cram_fd) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_5960_cram_check_eof(fd.cast())
}
