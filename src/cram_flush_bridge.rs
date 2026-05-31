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

//! Byte-faithful Rust translations of the CRAM write/flush helpers
//! cram_flush_container2, cram_flush_container, cram_flush_container_mt, and
//! cram_write_eof_block from htslib/cram/cram_io.c. These bridge the
//! production `src/cram.rs` natives (cram_write_container, cram_write_block,
//! cram_index_slice) to the dormant mirror's `cram_encode_container` /
//! `cram_encode_compression_header` translations (src/cram/cram_encode.rs).

use std::ffi::{c_char, c_int, c_uint, c_void};

use crate::htslib_rs::c_compat::free;
use crate::htslib_rs::cram::{
    cram_block, cram_container, cram_cram_io_c_1565_cram_free_block as cram_free_block,
    cram_cram_io_c_3705_cram_free_container as cram_free_container,
    cram_cram_io_c_5431_cram_seek as cram_seek, cram_write_block, cram_write_container,
};
use crate::htslib_rs::hts::{
    cram_fd, hFILE, BGZF, HTS_IDX_NOCOOR, HTS_IDX_NONE, HTS_IDX_REST, HTS_IDX_START,
};
use crate::htslib_rs::sam::{bam1_t, bam_destroy1, sam_hdr_t};

// The mirror module's `cram_encode_container` and `cram_encode_compression_header`
// are byte-faithful translations of the C originals. Their concrete struct
// definitions (`cram_fd`, `cram_container`, `cram_block`, ...) are
// `#[repr(C)]` and field-for-field identical to the C structs that production
// also aliases to `hts_sys::cram_*`, so all the pointer casts below are pure
// type-system noise.
use crate::cram_mirror::cram_encode::{
    cram_encode_compression_header as mirror_cram_encode_compression_header,
    cram_encode_container as mirror_cram_encode_container,
    cram_block_compression_hdr as mirror_cram_block_compression_hdr,
    cram_container as mirror_cram_container,
    cram_fd as mirror_cram_fd,
};
use crate::cram_mirror::cram_index::cram_cram_index_c_695_cram_index_slice;

// hFILE buffer layout (htslib/htslib/hfile.h:21). Used to implement the
// inline `htell(fd->fp)` calls in cram_flush_container2: htell returns
// `fp->offset + (fp->begin - fp->buffer)` (see htslib/hfile_internal.h).
#[repr(C)]
struct HfileLayoutForTell {
    buffer: *mut std::ffi::c_char,
    begin: *mut std::ffi::c_char,
    end: *mut std::ffi::c_char,
    limit: *mut std::ffi::c_char,
    backend: *const c_void,
    offset: libc::off_t,
    flags: std::ffi::c_uint,
    has_errno: c_int,
}

// Local cram_fd prefix view: we only read fp, mode, version through this
// (the trailing fields are large and private to cram.rs). The rest is
// fetched via the small public accessors in `cram::cram_fd_*_get`.
#[repr(C)]
struct CramFdPrefix {
    fp: *mut HfileLayoutForTell,
    mode: c_int,
    version: c_int,
    // Trailing fields are not accessed through this prefix.
}

#[inline]
unsafe fn htell_offset(fp: *mut HfileLayoutForTell) -> libc::off_t {
    // htell(fp) == fp->offset + (fp->begin - fp->buffer)
    (*fp).offset + (*fp).begin.offset_from((*fp).buffer) as libc::off_t
}

// cram_flush_container2 moved into production `src/cram.rs`; this shim
// delegates so callers still routed through the bridge land on the native.
pub unsafe fn cram_cram_io_c_4089_cram_flush_container2(
    fd: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_4089_cram_flush_container2(fd, c)
}

// original: cram_flush_container (htslib/cram/cram_io.c:4143)
//
// Flushes a completely or partially full container: encode (mirror) then
// flush_container2 (native).
pub unsafe fn cram_cram_io_c_4143_cram_flush_container(
    fd: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    // cram_encode_container is the mirror's byte-faithful translation.
    if 0 != mirror_cram_encode_container(fd.cast::<mirror_cram_fd>(), c.cast::<mirror_cram_container>()) {
        return -1;
    }
    cram_cram_io_c_4089_cram_flush_container2(fd, c)
}

// cram_flush_container_mt moved into production `src/cram.rs`; this shim
// delegates.
pub unsafe fn cram_cram_io_c_4275_cram_flush_container_mt(
    fd: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_4275_cram_flush_container_mt(fd, c)
}

// original: cram_write_eof_block (htslib/cram/cram_io.c:5474)
//
// Writes the empty-container EOF marker for CRAM v2+. Byte-faithful
// translation. For v1 (which lacks the EOF marker concept) it's a no-op
// returning 0, matching the C original.
pub unsafe fn cram_cram_io_c_5474_cram_write_eof_block(fd: *mut cram_fd) -> c_int {
    let fd_prefix = fd.cast::<CramFdPrefix>();
    let major = (*fd_prefix).version >> 8;
    if major >= 2 {
        // cram_container c; memset(&c, 0, sizeof(c));
        let mut c: mirror_cram_container = std::mem::zeroed();
        c.ref_seq_id = -1;
        c.ref_seq_start = 0x454f46; // "EOF"
        c.ref_seq_span = 0;
        c.record_counter = 0;
        c.num_bases = 0;
        c.num_blocks = 1;
        let mut land: [i32; 1] = [0];
        c.landmark = land.as_mut_ptr();
        // C source intentionally leaves c.num_landmarks at the memset(0) value
        // (htslib/cram/cram_io.c:5486-5496). The container encoder then writes
        // varint(num_landmarks)=0 and the landmark[] loop is empty, matching
        // the bytewise template at cram_io.c:5532-5535 ("0f 00 00 00 ...
        // 00 01 00 05 bd d9 4f"). Do NOT set num_landmarks=1 here, even though
        // the landmark array has one cell allocated.

        // cram_block_compression_hdr ch; memset(&ch, 0, sizeof(ch));
        let mut ch: mirror_cram_block_compression_hdr = std::mem::zeroed();

        // c.comp_hdr_block = cram_encode_compression_header(fd, &c, &ch, 0);
        c.comp_hdr_block = mirror_cram_encode_compression_header(
            fd.cast::<mirror_cram_fd>(),
            &mut c as *mut mirror_cram_container,
            &mut ch as *mut mirror_cram_block_compression_hdr,
            0,
        );

        // c.length = c.comp_hdr_block->byte    // Landmark[0]
        //          + 5                          // block struct
        //          + 4*(CRAM_MAJOR_VERS(fd->version) >= 3); // CRC
        let comp_hdr_byte = (*c.comp_hdr_block).byte as i32;
        c.length = comp_hdr_byte + 5 + 4 * (if major >= 3 { 1 } else { 0 });

        let comp_hdr_blk_prod: *mut cram_block = c.comp_hdr_block.cast();
        if cram_write_container(fd, (&mut c as *mut mirror_cram_container).cast::<cram_container>()) < 0
            || cram_write_block(fd, comp_hdr_blk_prod) < 0
        {
            // cram_close(fd) + cram_free_block(c.comp_hdr_block) on failure.
            // The C source calls cram_close from inside cram_write_eof_block
            // (cram_io.c:5516); we replicate that even though it produces a
            // double-close when cram_write_eof_block was itself reached
            // through cram_close (the production cram_close path does both).
            // The C source is awkward here; we mirror it byte-faithfully.
            crate::htslib_rs::cram::cram_close(fd);
            cram_free_block(comp_hdr_blk_prod);
            return -1;
        }

        // if (ch.preservation_map) kh_destroy(map, ch.preservation_map);
        if !ch.preservation_map.is_null() {
            // khash bucket arrays + table; cram_encode_compression_header
            // builds this with kh_init(map) + kh_put(map, ...). The minimum
            // teardown is freeing the bucket arrays (flags/keys/vals) and
            // the table itself, matching kh_destroy(map, ...).
            let h = ch.preservation_map.cast::<KhashGenericLayout>();
            if !(*h).flags.is_null() {
                free((*h).flags.cast());
            }
            if !(*h).keys.is_null() {
                free((*h).keys.cast());
            }
            if !(*h).vals.is_null() {
                free((*h).vals.cast());
            }
            free(h.cast());
        }
        cram_free_block(comp_hdr_blk_prod);
    }

    0
}

// khash table prefix matching kh_generic_layout in src/cram.rs.
#[repr(C)]
struct KhashGenericLayout {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    flags: *mut u32,
    keys: *mut c_void,
    vals: *mut c_void,
}

// ---------------------------------------------------------------------------
// Support types for cram_seek_to_refpos / cram_index_free / free_bam_list
// ---------------------------------------------------------------------------

// original: cram_range (htslib/cram/cram_structs.h:737)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CramRangeLayout {
    pub refid: c_int,
    pub start: i64,
    pub end: i64,
}

// original: cram_index (htslib/cram/cram_structs.h:720)
#[repr(C)]
struct CramIndexLayout {
    nslice: c_int,
    nalloc: c_int,
    e: *mut CramIndexLayout,
    refid: c_int,
    start: c_int,
    end: c_int,
    nseq: c_int,
    slice: c_int,
    len: c_int,
    offset: i64,
    e_next: *mut CramIndexLayout,
}

// Full cram_fd layout, matching cram_fd_layout in src/cram.rs (which mirrors
// htslib/cram/cram_structs.h:760-..). The trailing fields after the ones we
// access here are inert to our translations, but we include them so the struct
// size is faithful, which is required for any callee that allocates or moves
// these structs. We only ever cast borrowed pointers, never construct one.
const CRAM_DS_END: usize = 47;

#[repr(C)]
struct CramMetricsOpaque {
    _private: [u8; 0],
}

#[repr(C)]
struct CramFdLayoutForSeek {
    fp: *mut hFILE,
    mode: c_int,
    version: c_int,
    file_def: *mut c_void,
    header: *mut sam_hdr_t,
    prefix: *mut c_char,
    record_counter: i64,
    err: c_int,
    ctr: *mut cram_container,
    ctr_mt: *mut cram_container,
    first_base: c_int,
    last_base: c_int,
    refs: *mut c_void,
    ref_: *mut c_char,
    ref_free: *mut c_char,
    ref_id: c_int,
    ref_start: i64,
    ref_end: i64,
    ref_fn: *mut c_char,
    level: c_int,
    m: [*mut CramMetricsOpaque; CRAM_DS_END],
    tags_used: *mut c_void,
    decode_md: c_int,
    seqs_per_slice: c_int,
    bases_per_slice: c_int,
    slices_per_container: c_int,
    embed_ref: c_int,
    no_ref: c_int,
    no_ref_counter: c_int,
    ignore_md5: c_int,
    use_bz2: c_int,
    use_rans: c_int,
    use_lzma: c_int,
    use_fqz: c_int,
    use_tok: c_int,
    use_arith: c_int,
    shared_ref: c_int,
    required_fields: c_uint,
    store_md: c_int,
    store_nm: c_int,
    range: CramRangeLayout,
    bam_flag_swap: [c_uint; 0x1000],
    cram_flag_swap: [c_uint; 0x1000],
    l1: [u8; 256],
    l2: [u8; 256],
    cram_sub_matrix: [[c_char; 32]; 32],
    index_sz: c_int,
    index: *mut CramIndexLayout,
    first_container: libc::off_t,
    curr_position: libc::off_t,
    eof: c_int,
    last_slice: c_int,
    last_ri_count: c_int,
    multi_seq: c_int,
    multi_seq_user: c_int,
    unsorted: c_int,
    last_mapped: c_int,
    empty_container: c_int,
    own_pool: c_int,
    pool: *mut c_void,
    rqueue: *mut c_void,
    metrics_lock: libc::pthread_mutex_t,
    ref_lock: libc::pthread_mutex_t,
    range_lock: libc::pthread_mutex_t,
    bl: *mut c_void,
    bam_list_lock: libc::pthread_mutex_t,
    job_pending: *mut c_void,
    ooc: c_int,
    lossy_read_names: c_int,
    tlen_approx: c_int,
    tlen_zero: c_int,
    idxfp: *mut BGZF,
    // varint_vec table follows; we never access it through this layout, so it
    // is intentionally omitted. We only cast borrowed pointers and read/write
    // the fields above; the tail is reached only through other paths that use
    // the production cram_fd_layout in src/cram.rs.
}

// free_bam_list / cram_index_free / cram_index_free_recurse moved into
// production `src/cram.rs`. These shims delegate so anything still calling
// the bridge path lands on the canonical native.
pub unsafe fn cram_cram_io_c_3697_free_bam_list(bams: *mut *mut bam1_t, max_rec: c_int) {
    crate::htslib_rs::cram::cram_cram_io_c_3697_free_bam_list(bams, max_rec);
}

pub unsafe fn cram_cram_index_c_374_cram_index_free(fd: *mut cram_fd) {
    crate::htslib_rs::cram::cram_cram_index_c_374_cram_index_free(fd);
}

// original: cram_seek_to_refpos (htslib/cram/cram_index.c:573)
//
// Walks the CRAI for (refid, pos), seeks the underlying file to the container
// containing it, and snapshots the new range on the fd. Returns:
//   0  on success
//  -1  on a general failure (seek failed)
//  -2  when no overlapping slice exists (most commonly: empty chromosome)
// Byte-faithful 1:1 translation; on err the fd->range snapshot is still
// updated to match the input range (matching the C source's deliberate
// "identical behaviour to the previous code" comment at cram_index.c:614).
// cram_seek_to_refpos moved into production `src/cram.rs`. This shim
// delegates so anything still calling the bridge path lands on the native.
pub unsafe fn cram_cram_index_c_573_cram_seek_to_refpos(
    fd: *mut cram_fd,
    r: *mut CramRangeLayout,
) -> c_int {
    crate::htslib_rs::cram::cram_cram_index_c_573_cram_seek_to_refpos(fd, r.cast())
}
