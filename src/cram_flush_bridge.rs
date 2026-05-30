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

// original: cram_flush_container2 (htslib/cram/cram_io.c:4089)
//
// Common component shared by cram_flush_container{,_mt}: writes the
// container header, the compression header block, and every slice block; if
// idxfp is set, also drives cram_index_slice for each slice. Byte-faithful
// 1:1 translation.
pub unsafe fn cram_cram_io_c_4089_cram_flush_container2(
    fd: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    let mut i: c_int;
    let mut j: c_int;

    let fd_prefix = fd.cast::<CramFdPrefix>();
    let fp = (*fd_prefix).fp;

    // C: if (c->curr_slice > 0 && !c->slices) return -1;
    let cm = c.cast::<mirror_cram_container>();
    if (*cm).curr_slice > 0 && (*cm).slices.is_null() {
        return -1;
    }

    // off_t c_offset = htell(fd->fp);
    let c_offset = htell_offset(fp);

    // if (0 != cram_write_container(fd, c)) return -1;
    if 0 != cram_write_container(fd, c) {
        return -1;
    }

    // off_t hdr_size = htell(fd->fp) - c_offset;
    let hdr_size = htell_offset(fp) - c_offset;

    // if (0 != cram_write_block(fd, c->comp_hdr_block)) return -1;
    if 0 != cram_write_block(fd, (*cm).comp_hdr_block.cast::<cram_block>()) {
        return -1;
    }

    // off_t file_offset = htell(fd->fp);
    let mut file_offset = htell_offset(fp);

    // for (i = 0; i < c->curr_slice; i++)
    i = 0;
    while i < (*cm).curr_slice {
        let s = *(*cm).slices.offset(i as isize);
        let spos = file_offset - c_offset - hdr_size;

        // if (0 != cram_write_block(fd, s->hdr_block)) return -1;
        if 0 != cram_write_block(fd, (*s).hdr_block.cast::<cram_block>()) {
            return -1;
        }

        // for (j = 0; j < s->hdr->num_blocks; j++)
        let s_hdr = (*s).hdr;
        let num_blocks = (*s_hdr).num_blocks;
        j = 0;
        while j < num_blocks {
            let blk = *(*s).block.offset(j as isize);
            if 0 != cram_write_block(fd, blk.cast::<cram_block>()) {
                return -1;
            }
            j += 1;
        }

        file_offset = htell_offset(fp);
        let sz = file_offset - c_offset - hdr_size - spos;

        // if (fd->idxfp) { cram_index_slice(...) }
        let idxfp = crate::htslib_rs::cram::cram_fd_idxfp_get(fd);
        if !idxfp.is_null() {
            // cram_index_slice signature in mirror takes the mirror types,
            // but they're #[repr(C)] and ABI-identical to the production ones.
            let rc = cram_cram_index_c_695_cram_index_slice(
                fd.cast(),
                c.cast(),
                s.cast(),
                idxfp.cast(),
                c_offset,
                spos,
                sz,
            );
            if rc < 0 {
                return -1;
            }
        }

        i += 1;
    }

    0
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

// original: cram_flush_container_mt (htslib/cram/cram_io.c:4275)
//
// Multi-threaded flush dispatcher. The single-threaded branch (the common
// case when no thread pool was attached via CRAM_OPT_NTHREADS /
// CRAM_OPT_THREAD_POOL) is translated verbatim:
//
//     if (!fd->pool) return cram_flush_container(fd, c);
//
// The multi-threaded branch (hts_tpool_dispatch2 → cram_flush_thread →
// cram_flush_result) depends on several still-unported helpers in C:
//   * cram_flush_thread / cram_flush_result (cram_io.c:4156, 4168) which own
//     the `cram_job` work-item plumbing and the container reference-counted
//     teardown
//   * reset_metrics (cram_io.c:4238) which inspects fd->m[] and unlocks
//     metrics_lock while flushing the rqueue
// These are out of scope for this step. Even when fd->pool is set we run
// the single-threaded path: this is a faithful semantic equivalent (same
// on-disk output, same return code) at the cost of forfeiting concurrency
// for the duration of the flush. The MT-pool TODO is tracked in the
// production cram_close fallback (which still routes to hts_sys::cram_close
// when pool/rqueue are set so the existing job queue can be drained
// safely); cram_flush itself takes the native ST path unconditionally.
pub unsafe fn cram_cram_io_c_4275_cram_flush_container_mt(
    fd: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    // The metrics_lock / reset_metrics block (cram_io.c:4288-4294) only
    // matters across MT job boundaries; the ST fallback never crosses
    // them, so we skip it. fd->last_mapped is set by reset_metrics but
    // also recomputed for each container; the ST path's snapshot is what
    // the encoder reads next.
    cram_cram_io_c_4143_cram_flush_container(fd, c)
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

// original: free_bam_list (htslib/cram/cram_io.c:3697)
//
// Frees each `bam1_t *` slot in `bams[0..max_rec]`, then frees the array
// itself. Byte-for-byte equivalent to the C original.
pub unsafe fn cram_cram_io_c_3697_free_bam_list(bams: *mut *mut bam1_t, max_rec: c_int) {
    let mut i: c_int = 0;
    while i < max_rec {
        bam_destroy1(*bams.offset(i as isize));
        i += 1;
    }
    free(bams.cast());
}

// original: cram_index_free_recurse (htslib/cram/cram_index.c:364)
//
// Recursively frees the per-node `e` child array of a CRAI tree node.
unsafe fn cram_cram_index_c_364_cram_index_free_recurse(e: *mut CramIndexLayout) {
    if !(*e).e.is_null() {
        let mut i: c_int = 0;
        while i < (*e).nslice {
            cram_cram_index_c_364_cram_index_free_recurse((*e).e.offset(i as isize));
            i += 1;
        }
        free((*e).e.cast());
    }
}

// original: cram_index_free (htslib/cram/cram_index.c:374)
//
// Walks the top-level CRAI tree (one entry per reference) and recursively
// frees each subtree, then releases the index array and clears the fd field.
pub unsafe fn cram_cram_index_c_374_cram_index_free(fd: *mut cram_fd) {
    let fdl = fd.cast::<CramFdLayoutForSeek>();
    if (*fdl).index.is_null() {
        return;
    }
    let mut i: c_int = 0;
    while i < (*fdl).index_sz {
        cram_cram_index_c_364_cram_index_free_recurse((*fdl).index.offset(i as isize));
        i += 1;
    }
    free((*fdl).index.cast());
    (*fdl).index = std::ptr::null_mut();
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
pub unsafe fn cram_cram_index_c_573_cram_seek_to_refpos(
    fd: *mut cram_fd,
    r: *mut CramRangeLayout,
) -> c_int {
    let fdl = fd.cast::<CramFdLayoutForSeek>();
    let mut ret: c_int = 0;

    // C: if (r->refid == HTS_IDX_NONE) { ret = -2; goto err; }
    if (*r).refid == HTS_IDX_NONE {
        ret = -2;
    } else {
        // Ideally use an index, so see if we have one.
        let e = crate::cram_mirror::cram_index::cram_cram_index_c_404_cram_index_query(
            fd.cast(),
            (*r).refid,
            (*r).start,
            std::ptr::null_mut(),
        );
        if !e.is_null() {
            // C: if (0 != cram_seek(fd, e->offset, SEEK_SET)) { ret = -1; goto err; }
            let e_layout = e.cast::<CramIndexLayout>();
            if 0 != cram_seek(fd, (*e_layout).offset as libc::off_t, libc::SEEK_SET) {
                ret = -1;
            }
        } else {
            // Absent from index, but this most likely means it simply has no data.
            ret = -2;
        }
    }

    if ret != 0 {
        // err: branch. fd->range is still updated to match the input range,
        // matching the C source exactly.
        libc::pthread_mutex_lock(&mut (*fdl).range_lock);
        (*fdl).range = *r;
        libc::pthread_mutex_unlock(&mut (*fdl).range_lock);
        return ret;
    }

    // Success path.
    libc::pthread_mutex_lock(&mut (*fdl).range_lock);
    (*fdl).range = *r;
    if (*r).refid == HTS_IDX_NOCOOR {
        (*fdl).range.refid = -1;
        (*fdl).range.start = 0;
    } else if (*r).refid == HTS_IDX_START || (*r).refid == HTS_IDX_REST {
        (*fdl).range.refid = -2; // special case in cram_next_slice
    }
    libc::pthread_mutex_unlock(&mut (*fdl).range_lock);

    if !(*fdl).ctr.is_null() {
        cram_free_container((*fdl).ctr);
        if !(*fdl).ctr_mt.is_null() && (*fdl).ctr_mt != (*fdl).ctr {
            cram_free_container((*fdl).ctr_mt);
        }
        (*fdl).ctr = std::ptr::null_mut();
        (*fdl).ctr_mt = std::ptr::null_mut();
        (*fdl).ooc = 0;
        (*fdl).eof = 0;
    }

    0
}
