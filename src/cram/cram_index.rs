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

/*
 * The index is a gzipped tab-delimited text file with one line per slice.
 * The columns are:
 * 1: reference number (0 to N-1, as per BAM ref_id)
 * 2: reference position of 1st read in slice (1..?)
 * 3: number of reads in slice
 * 4: offset of container start (relative to end of SAM header, so 1st
 *    container is offset 0).
 * 5: slice number within container (ie which landmark).
 *
 * In memory, we hold this in a nested containment list. Each list element is
 * a cram_index struct. Each element in turn can contain its own list of
 * cram_index structs.
 *
 * Any start..end range which is entirely contained within another (and
 * earlier as it is sorted) range will be held within it. This ensures that
 * the outer list will never have containments and we can safely do a
 * binary search to find the first range which overlaps any given coordinate.
 */

use super::cram_structs::{
    cram_block_compression_hdr, cram_container, cram_index, cram_slice, varint_vec,
};
use crate::htslib_rs::bgzf::{bgzf_close, bgzf_open, bgzf_write};
use crate::htslib_rs::c_compat::{calloc, free, realloc};
use crate::htslib_rs::cram::{
    cram_block, cram_decode_compression_header, cram_free_container, cram_read_block,
    cram_read_container, cram_seek,
};
use crate::htslib_rs::hfile::htslib_hfile_h_247_hread;
use crate::htslib_rs::hfile::{hclose, hisremote, hopen, htslib_hfile_h_155_htell};
use crate::htslib_rs::hts::{
    cram_fd, hFILE, hts_c_4756_hts_idx_check_local, hts_c_4915_hts_idx_getfn, hts_pos_t, kputs,
    kputsn, kstring_t, BGZF, HTS_FMT_CRAI, HTS_IDX_NOCOOR, HTS_IDX_NONE, HTS_IDX_REST,
    HTS_IDX_START,
};
use std::ffi::{c_char, c_int, c_uint, c_void};
use std::mem;
use std::ptr;

const CRAM_DS_END: usize = 47;

#[repr(C)]
struct cram_range {
    refid: c_int,
    start: hts_pos_t,
    end: hts_pos_t,
}

#[repr(C)]
struct cram_block_layout {
    method: c_int,
    orig_method: c_int,
    content_type: hts_sys::cram_content_type,
}

#[repr(C)]
struct cram_fd_layout {
    fp: *mut hFILE,
    mode: c_int,
    version: c_int,
    file_def: *mut c_void,
    header: *mut hts_sys::sam_hdr_t,
    prefix: *mut c_char,
    record_counter: i64,
    err: c_int,
    ctr: *mut cram_container,
    ctr_mt: *mut cram_container,
    first_base: c_int,
    last_base: c_int,
    refs: *mut hts_sys::refs_t,
    ref_: *mut c_char,
    ref_free: *mut c_char,
    ref_id: c_int,
    ref_start: i64,
    ref_end: i64,
    ref_fn: *mut c_char,
    level: c_int,
    m: [*mut hts_sys::cram_metrics; CRAM_DS_END],
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
    range: cram_range,
    bam_flag_swap: [c_uint; 0x1000],
    cram_flag_swap: [c_uint; 0x1000],
    l1: [u8; 256],
    l2: [u8; 256],
    cram_sub_matrix: [[c_char; 32]; 32],
    index_sz: c_int,
    index: *mut cram_index,
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
    vv: varint_vec,
    ap_delta: c_int,
}

unsafe extern "C" {
    #[link_name = "zlib_mem_inflate"]
    fn htslib_zlib_mem_inflate(cdata: *mut c_char, csize: usize, size: *mut usize) -> *mut c_char;
    #[link_name = "cram_decode_slice"]
    fn htslib_cram_decode_slice(
        fd: *mut cram_fd,
        c: *mut hts_sys::cram_container,
        s: *mut hts_sys::cram_slice,
        hdr: *mut hts_sys::sam_hdr_t,
    ) -> c_int;
    #[link_name = "cram_read_slice"]
    fn htslib_cram_read_slice(fd: *mut cram_fd) -> *mut hts_sys::cram_slice;
    #[link_name = "cram_free_slice"]
    fn htslib_cram_free_slice(s: *mut hts_sys::cram_slice);
    #[link_name = "cram_set_option"]
    fn htslib_cram_set_option(fd: *mut cram_fd, opt: hts_sys::hts_fmt_option, ...) -> c_int;
}

// original: dump_index_ (htslib/cram/cram_index.c:72)
pub unsafe fn cram_cram_index_c_72_dump_index_(e: *mut cram_index, level: c_int) {
    let n = libc::printf(
        c"%*s%d / %d .. %d, ".as_ptr(),
        level * 4,
        c"".as_ptr(),
        (*e).refid,
        (*e).start,
        (*e).end,
    );
    libc::printf(
        c"%*soffset %ld %p %p\n".as_ptr(),
        (50 - n).max(0),
        c"".as_ptr(),
        (*e).offset,
        e,
        (*e).e_next,
    );
    let mut i = 0;
    while i < (*e).nslice {
        cram_cram_index_c_72_dump_index_((*e).e.add(i as usize), level + 1);
        i += 1;
    }
}

// original: dump_index (htslib/cram/cram_index.c:81)
pub unsafe fn cram_cram_index_c_81_dump_index(fd: *mut cram_fd) {
    let fd = fd.cast::<cram_fd_layout>();
    let mut i = 0;
    while i < (*fd).index_sz {
        cram_cram_index_c_72_dump_index_((*fd).index.add(i as usize), 0);
        i += 1;
    }
}

// original: link_index_ (htslib/cram/cram_index.c:92)
pub unsafe fn cram_cram_index_c_92_link_index_(
    e: *mut cram_index,
    mut e_last: *mut cram_index,
) -> *mut cram_index {
    if !e_last.is_null() {
        (*e_last).e_next = e;
    }

    // We don't want to link in the top-level cram_index with
    // offset=0 and start/end = INT_MIN/INT_MAX.
    if (*e).offset != 0 {
        e_last = e;
    }

    let mut i = 0;
    while i < (*e).nslice {
        e_last = cram_cram_index_c_92_link_index_((*e).e.add(i as usize), e_last);
        i += 1;
    }

    e_last
}

// original: link_index (htslib/cram/cram_index.c:108)
pub unsafe fn cram_cram_index_c_108_link_index(fd: *mut cram_fd) {
    let fd = fd.cast::<cram_fd_layout>();
    let mut e_last: *mut cram_index = ptr::null_mut();

    let mut i = 0;
    while i < (*fd).index_sz {
        e_last = cram_cram_index_c_92_link_index_((*fd).index.add(i as usize), e_last);
        i += 1;
    }

    if !e_last.is_null() {
        (*e_last).e_next = ptr::null_mut();
    }
}

// original: kget_int32 (htslib/cram/cram_index.c:120)
pub unsafe fn cram_cram_index_c_120_kget_int32(
    k: *mut kstring_t,
    pos: *mut usize,
    val_p: *mut i32,
) -> c_int {
    let mut sign = 1i32;
    let mut val = 0i32;
    let mut p = *pos;

    while p < (*k).l && (*(*k).s.add(p) == b' ' as c_char || *(*k).s.add(p) == b'\t' as c_char) {
        p += 1;
    }

    if p < (*k).l && *(*k).s.add(p) == b'-' as c_char {
        sign = -1;
        p += 1;
    }

    if p >= (*k).l || !(*(*k).s.add(p) >= b'0' as c_char && *(*k).s.add(p) <= b'9' as c_char) {
        return -1;
    }

    while p < (*k).l && *(*k).s.add(p) >= b'0' as c_char && *(*k).s.add(p) <= b'9' as c_char {
        let digit = (*(*k).s.add(p) - b'0' as c_char) as i32;
        p += 1;
        val = val * 10 + digit;
    }

    *pos = p;
    *val_p = sign * val;

    0
}

// original: kget_int64 (htslib/cram/cram_index.c:145)
pub unsafe fn cram_cram_index_c_145_kget_int64(
    k: *mut kstring_t,
    pos: *mut usize,
    val_p: *mut i64,
) -> c_int {
    let mut sign = 1i64;
    let mut val = 0i64;
    let mut p = *pos;

    while p < (*k).l && (*(*k).s.add(p) == b' ' as c_char || *(*k).s.add(p) == b'\t' as c_char) {
        p += 1;
    }

    if p < (*k).l && *(*k).s.add(p) == b'-' as c_char {
        sign = -1;
        p += 1;
    }

    if p >= (*k).l || !(*(*k).s.add(p) >= b'0' as c_char && *(*k).s.add(p) <= b'9' as c_char) {
        return -1;
    }

    while p < (*k).l && *(*k).s.add(p) >= b'0' as c_char && *(*k).s.add(p) <= b'9' as c_char {
        let digit = (*(*k).s.add(p) - b'0' as c_char) as i64;
        p += 1;
        val = val * 10 + digit;
    }

    *pos = p;
    *val_p = sign * val;

    0
}

// original: cram_index_load (htslib/cram/cram_index.c:176)
pub unsafe fn cram_cram_index_c_176_cram_index_load(
    fd: *mut cram_fd,
    fn_: *const c_char,
    mut fn_idx: *const c_char,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let mut tfn_idx: *mut c_char = ptr::null_mut();
    let mut buf = [0 as c_char; 65536];
    let mut kstr: kstring_t = mem::zeroed();
    let mut fp: *mut hFILE;
    let mut idx: *mut cram_index;
    let mut idx_stack: *mut *mut cram_index = ptr::null_mut();
    let mut idx_stack_alloc = 0;
    let mut idx_stack_ptr = 0;
    let mut pos = 0usize;

    macro_rules! fail {
        () => {{
            free(kstr.s.cast());
            free(idx_stack.cast());
            free(tfn_idx.cast());
            cram_cram_index_c_373_cram_index_free(fd); // Also sets fd->index = NULL
            return -1;
        }};
    }

    /* Check if already loaded */
    if !(*fdl).index.is_null() {
        return 0;
    }

    (*fdl).index_sz = 1;
    (*fdl).index = calloc(1, mem::size_of::<cram_index>() as u64).cast();
    if (*fdl).index.is_null() {
        return -1;
    }

    idx = (*fdl).index;
    (*idx).refid = -1;
    (*idx).start = c_int::MIN;
    (*idx).end = c_int::MAX;

    idx_stack_alloc += 1;
    idx_stack = calloc(
        idx_stack_alloc as u64,
        mem::size_of::<*mut cram_index>() as u64,
    )
    .cast();
    if idx_stack.is_null() {
        fail!();
    }

    *idx_stack.add(idx_stack_ptr as usize) = idx;

    // Support pathX.cram##idx##pathY.crai
    let fn_delim = libc::strstr(fn_, crate::htslib_rs::cram::HTS_IDX_DELIM.as_ptr().cast());
    if !fn_delim.is_null() && fn_idx.is_null() {
        fn_idx = fn_delim.add(libc::strlen(crate::htslib_rs::cram::HTS_IDX_DELIM.as_ptr().cast()));
    }

    if fn_idx.is_null() {
        if hts_c_4756_hts_idx_check_local(fn_, HTS_FMT_CRAI, &mut tfn_idx) == 0
            && hisremote(fn_) != 0
        {
            tfn_idx = hts_c_4915_hts_idx_getfn(fn_, c".crai".as_ptr());
        }

        if tfn_idx.is_null() {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Could not retrieve index file for '%s'\n".as_ptr(),
                fn_,
            );
            fail!();
        }
        fn_idx = tfn_idx;
    }

    fp = hopen(fn_idx, c"r".as_ptr());
    if fp.is_null() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Could not open index file '%s'\n".as_ptr(),
            fn_idx,
        );
        fail!();
    }

    // Load the file into memory
    loop {
        let len = htslib_hfile_h_247_hread(fp, buf.as_mut_ptr().cast(), buf.len());
        if len <= 0 {
            if len < 0 || kstr.l < 2 {
                fail!();
            }
            break;
        }
        if kputsn(buf.as_ptr(), len as usize, &mut kstr) < 0 {
            fail!();
        }
    }

    if hclose(fp) < 0 {
        fail!();
    }

    // Uncompress if required
    if *kstr.s == 31 && *kstr.s.add(1) as u8 == 139 {
        let mut l = 0usize;
        let s = htslib_zlib_mem_inflate(kstr.s, kstr.l, &mut l);
        if s.is_null() {
            fail!();
        }

        free(kstr.s.cast());
        kstr.s = s;
        kstr.l = l;
        kstr.m = l; // conservative estimate of the size allocated
        if kputsn(c"".as_ptr(), 0, &mut kstr) < 0 {
            fail!();
        }
    }

    // Parse it line at a time
    while pos < kstr.l {
        let mut e: cram_index = mem::zeroed();

        /* 1.1 layout */
        if cram_cram_index_c_120_kget_int32(&mut kstr, &mut pos, &mut e.refid) == -1
            || cram_cram_index_c_120_kget_int32(&mut kstr, &mut pos, &mut e.start) == -1
            || cram_cram_index_c_120_kget_int32(&mut kstr, &mut pos, &mut e.end) == -1
            || cram_cram_index_c_145_kget_int64(&mut kstr, &mut pos, &mut e.offset) == -1
            || cram_cram_index_c_120_kget_int32(&mut kstr, &mut pos, &mut e.slice) == -1
            || cram_cram_index_c_120_kget_int32(&mut kstr, &mut pos, &mut e.len) == -1
        {
            fail!();
        }

        e.end += e.start - 1;

        if e.refid < -1 {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Malformed index file, refid %d\n".as_ptr(),
                e.refid,
            );
            fail!();
        }

        if e.refid != (*idx).refid {
            if (*fdl).index_sz < e.refid + 2 {
                let new_sz = e.refid + 2;
                let index_end = (*fdl).index_sz as usize * mem::size_of::<cram_index>();
                let new_idx = realloc(
                    (*fdl).index.cast(),
                    (new_sz as usize * mem::size_of::<cram_index>()) as u64,
                )
                .cast::<cram_index>();
                if new_idx.is_null() {
                    fail!();
                }

                (*fdl).index = new_idx;
                (*fdl).index_sz = new_sz;
                libc::memset(
                    ((*fdl).index.cast::<c_char>()).add(index_end).cast(),
                    0,
                    (*fdl).index_sz as usize * mem::size_of::<cram_index>() - index_end,
                );
            }
            idx = (*fdl).index.add((e.refid + 1) as usize);
            (*idx).refid = e.refid;
            (*idx).start = c_int::MIN;
            (*idx).end = c_int::MAX;
            (*idx).nslice = 0;
            (*idx).nalloc = 0;
            (*idx).e = ptr::null_mut();
            idx_stack_ptr = 0;
            *idx_stack.add(idx_stack_ptr as usize) = idx;
        }

        while !(e.start >= (*idx).start && e.end <= (*idx).end)
            || ((*idx).start == 0 && (*idx).refid == -1)
        {
            idx_stack_ptr -= 1;
            idx = *idx_stack.add(idx_stack_ptr as usize);
        }

        // Now contains, so append
        if (*idx).nslice + 1 >= (*idx).nalloc {
            (*idx).nalloc = if (*idx).nalloc != 0 {
                (*idx).nalloc * 2
            } else {
                16
            };
            let new_e = realloc(
                (*idx).e.cast(),
                ((*idx).nalloc as usize * mem::size_of::<cram_index>()) as u64,
            )
            .cast::<cram_index>();
            if new_e.is_null() {
                fail!();
            }

            (*idx).e = new_e;
        }

        e.nalloc = 0;
        e.nslice = 0;
        e.e = ptr::null_mut();
        let ep = (*idx).e.add((*idx).nslice as usize);
        (*idx).nslice += 1;
        *ep = e;
        idx = ep;

        idx_stack_ptr += 1;
        if idx_stack_ptr >= idx_stack_alloc {
            idx_stack_alloc *= 2;
            let new_stack = realloc(
                idx_stack.cast(),
                (idx_stack_alloc as usize * mem::size_of::<*mut cram_index>()) as u64,
            )
            .cast::<*mut cram_index>();
            if new_stack.is_null() {
                fail!();
            }
            idx_stack = new_stack;
        }
        *idx_stack.add(idx_stack_ptr as usize) = idx;

        while pos < kstr.l && *kstr.s.add(pos) != b'\n' as c_char {
            pos += 1;
        }
        pos += 1;
    }

    free(idx_stack.cast());
    free(kstr.s.cast());
    free(tfn_idx.cast());

    // Convert NCList to linear linked list
    cram_cram_index_c_108_link_index(fd);

    //dump_index(fd);

    0
}

// original: cram_index_free_recurse (htslib/cram/cram_index.c:363)
pub unsafe fn cram_cram_index_c_363_cram_index_free_recurse(e: *mut cram_index) {
    if !(*e).e.is_null() {
        let mut i = 0;
        while i < (*e).nslice {
            cram_cram_index_c_363_cram_index_free_recurse((*e).e.add(i as usize));
            i += 1;
        }
        free((*e).e.cast());
    }
}

// original: cram_index_free (htslib/cram/cram_index.c:373)
pub unsafe fn cram_cram_index_c_373_cram_index_free(fd: *mut cram_fd) {
    let fd = fd.cast::<cram_fd_layout>();

    if (*fd).index.is_null() {
        return;
    }

    let mut i = 0;
    while i < (*fd).index_sz {
        cram_cram_index_c_363_cram_index_free_recurse((*fd).index.add(i as usize));
        i += 1;
    }
    free((*fd).index.cast());

    (*fd).index = ptr::null_mut();
}

// original: cram_index_query (htslib/cram/cram_index.c:404)
pub unsafe fn cram_cram_index_c_404_cram_index_query(
    fd: *mut cram_fd,
    mut refid: c_int,
    mut pos: hts_pos_t,
    mut from: *mut cram_index,
) -> *mut cram_index {
    let fdl = fd.cast::<cram_fd_layout>();
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut e: *mut cram_index;

    if !from.is_null() {
        // Continue from a previous search.
        // We switch to just scanning the linked list, as the nested
        // lists are typically short.
        if refid == HTS_IDX_NOCOOR {
            refid = -1;
        }

        e = (*from).e_next;
        if !e.is_null() && (*e).refid == refid && (*e).start as hts_pos_t <= pos {
            return e;
        } else {
            return ptr::null_mut();
        }
    }

    match refid {
        HTS_IDX_NONE | HTS_IDX_REST => {
            // fail, or already there, dealt with elsewhere.
            return ptr::null_mut();
        }
        -1 | HTS_IDX_NOCOOR => {
            refid = -1;
            pos = 0;
        }
        HTS_IDX_START => {
            let mut min_idx = i64::MAX;
            i = 0;
            j = -1;
            while i < (*fdl).index_sz {
                if !(*(*fdl).index.add(i as usize)).e.is_null()
                    && (*(*(*fdl).index.add(i as usize)).e).offset < min_idx
                {
                    min_idx = (*(*(*fdl).index.add(i as usize)).e).offset;
                    j = i;
                }
                i += 1;
            }
            if j < 0 {
                return ptr::null_mut();
            }
            return (*(*fdl).index.add(j as usize)).e;
        }
        _ => {
            if refid < HTS_IDX_NONE || refid + 1 >= (*fdl).index_sz {
                return ptr::null_mut();
            }
        }
    }

    from = (*fdl).index.add((refid + 1) as usize);

    // Ref with nothing aligned against it.
    if (*from).e.is_null() {
        return ptr::null_mut();
    }

    // This sequence is covered by the index, so binary search to find
    // the optimal starting block.
    i = 0;
    j = (*(*fdl).index.add((refid + 1) as usize)).nslice - 1;
    k = j / 2;
    while k != i {
        if (*(*from).e.add(k as usize)).refid > refid {
            j = k;
            k = (j - i) / 2 + i;
            continue;
        }

        if (*(*from).e.add(k as usize)).refid < refid {
            i = k;
            k = (j - i) / 2 + i;
            continue;
        }

        if (*(*from).e.add(k as usize)).start as hts_pos_t >= pos {
            j = k;
            k = (j - i) / 2 + i;
            continue;
        }

        if ((*(*from).e.add(k as usize)).start as hts_pos_t) < pos {
            i = k;
            k = (j - i) / 2 + i;
            continue;
        }
        k = (j - i) / 2 + i;
    }
    // i==j or i==j-1. Check if j is better.
    if j >= 0
        && ((*(*from).e.add(j as usize)).start as hts_pos_t) < pos
        && (*(*from).e.add(j as usize)).refid == refid
    {
        i = j;
    }

    /* The above found *a* bin overlapping, but not necessarily the first */
    while i > 0 && (*(*from).e.add((i - 1) as usize)).end as hts_pos_t >= pos {
        i -= 1;
    }

    /* We may be one bin before the optimum, so check */
    while i + 1 < (*from).nslice
        && ((*(*from).e.add(i as usize)).refid < refid
            || ((*(*from).e.add(i as usize)).end as hts_pos_t) < pos)
    {
        i += 1;
    }

    e = (*from).e.add(i as usize);

    e
}

// original: cram_index_last (htslib/cram/cram_index.c:503)
pub unsafe fn cram_cram_index_c_503_cram_index_last(
    fd: *mut cram_fd,
    refid: c_int,
    mut from: *mut cram_index,
) -> *mut cram_index {
    let fd = fd.cast::<cram_fd_layout>();

    if refid + 1 < 0 || refid + 1 >= (*fd).index_sz {
        return ptr::null_mut();
    }

    if from.is_null() {
        from = (*fd).index.add((refid + 1) as usize);
    }

    // Ref with nothing aligned against it.
    if (*from).e.is_null() {
        return ptr::null_mut();
    }

    let slice = (*(*fd).index.add((refid + 1) as usize)).nslice - 1;

    // e is the last entry in the nested containment list, but it may
    // contain further slices within it.
    let mut e = (*from).e.add(slice as usize);
    while !(*e).e_next.is_null() {
        e = (*e).e_next;
    }

    e
}

// original: cram_index_query_last (htslib/cram/cram_index.c:531)
pub unsafe fn cram_cram_index_c_531_cram_index_query_last(
    fd: *mut cram_fd,
    refid: c_int,
    end: hts_pos_t,
) -> *mut cram_index {
    let mut e: *mut cram_index = ptr::null_mut();
    let mut prev_e: *mut cram_index;
    loop {
        prev_e = e;
        e = cram_cram_index_c_404_cram_index_query(fd, refid, end, prev_e);
        if e.is_null() {
            break;
        }
    }

    if prev_e.is_null() {
        return ptr::null_mut();
    }
    e = prev_e;

    // Note: offset of e and e->e_next may be the same if we're using a
    // multi-ref container where a single container generates multiple
    // index entries.
    //
    // We need to keep iterating until offset differs in order to find
    // the genuine file offset for the end of container.
    loop {
        prev_e = e;
        e = (*e).e_next;
        if e.is_null() || (*e).offset != (*prev_e).offset {
            break;
        }
    }

    prev_e
}

// original: cram_seek_to_refpos (htslib/cram/cram_index.c:572)
pub unsafe fn cram_cram_index_c_572_cram_seek_to_refpos(
    fd: *mut cram_fd,
    r: *mut cram_range,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let mut ret = 0;
    let e: *mut cram_index;

    if (*r).refid == HTS_IDX_NONE {
        ret = -2;
        libc::pthread_mutex_lock(&mut (*fdl).range_lock);
        (*fdl).range = ptr::read(r);
        libc::pthread_mutex_unlock(&mut (*fdl).range_lock);
        return ret;
    }

    // Ideally use an index, so see if we have one.
    e = cram_cram_index_c_404_cram_index_query(fd, (*r).refid, (*r).start, ptr::null_mut());
    if !e.is_null() {
        if cram_seek(fd, (*e).offset, libc::SEEK_SET) != 0 {
            ret = -1;
            libc::pthread_mutex_lock(&mut (*fdl).range_lock);
            (*fdl).range = ptr::read(r);
            libc::pthread_mutex_unlock(&mut (*fdl).range_lock);
            return ret;
        }
    } else {
        // Absent from index, but this most likely means it simply has no data.
        ret = -2;
        libc::pthread_mutex_lock(&mut (*fdl).range_lock);
        (*fdl).range = ptr::read(r);
        libc::pthread_mutex_unlock(&mut (*fdl).range_lock);
        return ret;
    }

    libc::pthread_mutex_lock(&mut (*fdl).range_lock);
    (*fdl).range = ptr::read(r);
    if (*r).refid == HTS_IDX_NOCOOR {
        (*fdl).range.refid = -1;
        (*fdl).range.start = 0;
    } else if (*r).refid == HTS_IDX_START || (*r).refid == HTS_IDX_REST {
        (*fdl).range.refid = -2; // special case in cram_next_slice
    }
    libc::pthread_mutex_unlock(&mut (*fdl).range_lock);

    if !(*fdl).ctr.is_null() {
        cram_free_container((*fdl).ctr.cast());
        if !(*fdl).ctr_mt.is_null() && (*fdl).ctr_mt != (*fdl).ctr {
            cram_free_container((*fdl).ctr_mt.cast());
        }
        (*fdl).ctr = ptr::null_mut();
        (*fdl).ctr_mt = ptr::null_mut();
        (*fdl).ooc = 0;
        (*fdl).eof = 0;
    }

    0
}

// original: cram_index_build_multiref (htslib/cram/cram_index.c:632)
pub unsafe fn cram_cram_index_c_632_cram_index_build_multiref(
    fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
    fp: *mut BGZF,
    cpos: libc::off_t,
    landmark: i32,
    sz: c_int,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let mut ref_ = -2;
    let mut ref_start = 0i64;
    let mut ref_end: i64;
    let mut buf = [0 as c_char; 1024];

    if (*fdl).mode != b'w' as c_int {
        if htslib_cram_decode_slice(fd, c.cast(), s.cast(), (*fdl).header) != 0 {
            return -1;
        }
    }

    ref_end = c_int::MIN as i64;

    let mut last_ref = -9;
    let mut last_pos = -9;
    let mut i = 0;
    while i < (*(*s).hdr).num_records {
        let rec = (*s).crecs.add(i as usize);
        if (*rec).ref_id == last_ref && (*rec).apos < last_pos {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"CRAM file is not sorted by chromosome / position\n".as_ptr(),
            );
            return -2;
        }
        last_ref = (*rec).ref_id;
        last_pos = (*rec).apos;

        if (*rec).ref_id == ref_ {
            if ref_end < (*rec).aend {
                ref_end = (*rec).aend;
            }
            i += 1;
            continue;
        }

        if ref_ != -2 {
            libc::snprintf(
                buf.as_mut_ptr(),
                buf.len(),
                c"%d\t%ld\t%ld\t%ld\t%d\t%d\n".as_ptr(),
                ref_,
                ref_start,
                ref_end - ref_start + 1,
                cpos as i64,
                landmark,
                sz,
            );
            if bgzf_write(fp, buf.as_ptr().cast(), libc::strlen(buf.as_ptr())) < 0 {
                return -4;
            }
        }

        ref_ = (*rec).ref_id;
        ref_start = (*rec).apos;
        ref_end = (*rec).aend;
        i += 1;
    }

    if ref_ != -2 {
        libc::snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"%d\t%ld\t%ld\t%ld\t%d\t%d\n".as_ptr(),
            ref_,
            ref_start,
            ref_end - ref_start + 1,
            cpos as i64,
            landmark,
            sz,
        );
        if bgzf_write(fp, buf.as_ptr().cast(), libc::strlen(buf.as_ptr())) < 0 {
            return -4;
        }
    }

    0
}

// original: cram_index_slice (htslib/cram/cram_index.c:695)
pub unsafe fn cram_cram_index_c_695_cram_index_slice(
    fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
    fp: *mut BGZF,
    cpos: libc::off_t,
    spos: libc::off_t,
    sz: libc::off_t,
) -> c_int {
    let mut buf = [0 as c_char; 1024];

    if sz > c_int::MAX as libc::off_t {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"CRAM slice is too big (%ld bytes)\n".as_ptr(),
            sz as i64,
        );
        return -1;
    }

    if (*(*s).hdr).ref_seq_id == -2 {
        cram_cram_index_c_632_cram_index_build_multiref(fd, c, s, fp, cpos, spos as i32, sz as i32)
    } else {
        libc::snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"%d\t%ld\t%ld\t%ld\t%d\t%d\n".as_ptr(),
            (*(*s).hdr).ref_seq_id,
            (*(*s).hdr).ref_seq_start,
            (*(*s).hdr).ref_seq_span,
            cpos as i64,
            spos as c_int,
            sz as c_int,
        );
        if bgzf_write(fp, buf.as_ptr().cast(), libc::strlen(buf.as_ptr())) >= 0 {
            0
        } else {
            -4
        }
    }
}

// original: cram_index_container (htslib/cram/cram_index.c:727)
pub unsafe fn cram_cram_index_c_727_cram_index_container(
    fd: *mut cram_fd,
    c: *mut cram_container,
    fp: *mut BGZF,
    cpos: libc::off_t,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let mut j = 0;

    // 2.0 format
    while j < (*c).num_landmarks {
        let spos = htslib_hfile_h_155_htell((*fdl).fp);
        if spos - cpos - (*c).offset as libc::off_t != *(*c).landmark.add(j as usize) as libc::off_t
        {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"CRAM slice offset %ld does not match landmark %d in container header (%d)\n"
                    .as_ptr(),
                (spos - cpos - (*c).offset as libc::off_t) as i64,
                j,
                *(*c).landmark.add(j as usize),
            );
            return -1;
        }

        let s = htslib_cram_read_slice(fd).cast::<cram_slice>();
        if s.is_null() {
            return -1;
        }

        let sz = htslib_hfile_h_155_htell((*fdl).fp) - spos;
        let ret = cram_cram_index_c_695_cram_index_slice(
            fd,
            c,
            s,
            fp,
            cpos,
            *(*c).landmark.add(j as usize) as libc::off_t,
            sz,
        );

        htslib_cram_free_slice(s.cast());

        if ret < 0 {
            return ret;
        }

        j += 1;
    }

    0
}

// original: cram_index_build (htslib/cram/cram_index.c:779)
pub unsafe fn cram_cram_index_c_779_cram_index_build(
    fd: *mut cram_fd,
    fn_base: *const c_char,
    mut fn_idx: *const c_char,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let mut fn_idx_str: kstring_t = mem::zeroed();
    let mut last_ref = -9i64;
    let mut last_start = -9i64;

    // Useful for cram_index_build_multiref
    htslib_cram_set_option(
        fd,
        crate::htslib_rs::cram::CRAM_OPT_REQUIRED_FIELDS,
        crate::htslib_rs::cram::SAM_RNAME | crate::htslib_rs::cram::SAM_POS | crate::htslib_rs::cram::SAM_CIGAR,
    );

    if fn_idx.is_null() {
        kputs(fn_base, &mut fn_idx_str);
        kputs(c".crai".as_ptr(), &mut fn_idx_str);
        fn_idx = fn_idx_str.s;
    }

    let fp = bgzf_open(fn_idx, c"wg".as_ptr());
    if fp.is_null() {
        libc::perror(fn_idx);
        free(fn_idx_str.s.cast());
        return -4;
    }

    free(fn_idx_str.s.cast());

    let mut cpos = htslib_hfile_h_155_htell((*fdl).fp);
    loop {
        let c = cram_read_container(fd).cast::<cram_container>();
        if c.is_null() {
            break;
        }
        if (*fdl).err != 0 {
            libc::perror(c"Cram container read".as_ptr());
            return -1;
        }

        let hpos = htslib_hfile_h_155_htell((*fdl).fp);

        (*c).comp_hdr_block = cram_read_block(fd).cast::<cram_block>();
        if (*c).comp_hdr_block.is_null() {
            return -1;
        }
        assert!(
            (*(*c).comp_hdr_block.cast::<cram_block_layout>()).content_type
                == crate::htslib_rs::cram::CRAM_CONTENT_TYPE_COMPRESSION_HEADER
        );

        (*c).comp_hdr = cram_decode_compression_header(fd, (*c).comp_hdr_block.cast())
            .cast::<cram_block_compression_hdr>();
        if (*c).comp_hdr.is_null() {
            return -1;
        }

        if (*c).ref_seq_id as i64 == last_ref && (*c).ref_seq_start < last_start {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"CRAM file is not sorted by chromosome / position\n".as_ptr(),
            );
            return -2;
        }
        last_ref = (*c).ref_seq_id as i64;
        last_start = (*c).ref_seq_start;

        if cram_cram_index_c_727_cram_index_container(fd, c, fp, cpos) < 0 {
            bgzf_close(fp);
            return -1;
        }

        let next_cpos = htslib_hfile_h_155_htell((*fdl).fp);
        if next_cpos != hpos + (*c).length as libc::off_t {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Length %d in container header at offset %lld does not match block lengths (%lld)\n"
                    .as_ptr(),
                (*c).length,
                cpos as libc::c_longlong,
                (next_cpos - hpos) as libc::c_longlong,
            );
            return -1;
        }
        cpos = next_cpos;

        cram_free_container(c.cast());
    }
    if (*fdl).err != 0 {
        bgzf_close(fp);
        return -1;
    }

    if bgzf_close(fp) >= 0 {
        0
    } else {
        -4
    }
}

// original: cram_num_containers_between_ (htslib/cram/cram_index.c:851)
pub unsafe fn cram_cram_index_c_851_cram_num_containers_between_(
    e: *mut cram_index,
    last_pos: *mut i64,
    nct: i64,
    cstart: libc::off_t,
    cend: libc::off_t,
    first: *mut i64,
    last: *mut i64,
) -> i64 {
    let mut nc = 0i64;

    if (*e).offset != 0 {
        if (*e).offset != *last_pos {
            if (*e).offset >= cstart && (cend == 0 || (*e).offset <= cend) {
                if !first.is_null() && *first < 0 {
                    *first = nct;
                }
                if !last.is_null() {
                    *last = nct;
                }
            }
            nc += 1;
        }
        // else a new multi-ref in same container
        *last_pos = (*e).offset;
    }

    let mut i = 0;
    while i < (*e).nslice {
        nc += cram_cram_index_c_851_cram_num_containers_between_(
            (*e).e.add(i as usize),
            last_pos,
            nc + nct,
            cstart,
            cend,
            first,
            last,
        );
        i += 1;
    }

    nc
}

// original: cram_num_containers_between (htslib/cram/cram_index.c:889)
pub unsafe fn cram_cram_index_c_889_cram_num_containers_between(
    fd: *mut cram_fd,
    cstart: libc::off_t,
    cend: libc::off_t,
    first: *mut i64,
    last: *mut i64,
) -> i64 {
    let fd = fd.cast::<cram_fd_layout>();
    let mut nc = 0i64;
    let mut last_pos = -99i64;
    let mut l_first = -1i64;
    let mut l_last = -1i64;

    let mut i = 0;
    while i < (*fd).index_sz {
        let j = if i + 1 == (*fd).index_sz { 0 } else { i + 1 }; // maps "*" to end
        nc += cram_cram_index_c_851_cram_num_containers_between_(
            (*fd).index.add(j as usize),
            &mut last_pos,
            nc,
            cstart,
            cend,
            &mut l_first,
            &mut l_last,
        );
        i += 1;
    }

    if !first.is_null() {
        *first = l_first;
    }
    if !last.is_null() {
        *last = l_last;
    }

    l_last - l_first + 1
}

// original: cram_num_containers (htslib/cram/cram_index.c:915)
pub unsafe fn cram_cram_index_c_915_cram_num_containers(fd: *mut cram_fd) -> i64 {
    cram_cram_index_c_889_cram_num_containers_between(fd, 0, 0, ptr::null_mut(), ptr::null_mut())
}

// original: cram_container_num2offset_ (htslib/cram/cram_index.c:924)
pub unsafe fn cram_cram_index_c_924_cram_container_num2offset_(
    e: *mut cram_index,
    num: c_int,
    last_pos: *mut i64,
    nc: *mut c_int,
) -> *mut cram_index {
    if (*e).offset != 0 {
        if (*e).offset != *last_pos {
            if *nc == num {
                return e;
            }
            *nc += 1;
        }
        // else a new multi-ref in same container
        *last_pos = (*e).offset;
    }

    let mut i = 0;
    while i < (*e).nslice {
        let tmp = cram_cram_index_c_924_cram_container_num2offset_(
            (*e).e.add(i as usize),
            num,
            last_pos,
            nc,
        );
        if !tmp.is_null() {
            return tmp;
        }
        i += 1;
    }

    ptr::null_mut()
}

// original: cram_container_num2offset (htslib/cram/cram_index.c:948)
pub unsafe fn cram_cram_index_c_948_cram_container_num2offset(
    fd: *mut cram_fd,
    num: i64,
) -> libc::off_t {
    let fd = fd.cast::<cram_fd_layout>();
    let mut nc = 0;
    let mut last_pos = -9i64;
    let mut e: *mut cram_index = ptr::null_mut();

    let mut i = 0;
    while i < (*fd).index_sz {
        let j = if i + 1 == (*fd).index_sz { 0 } else { i + 1 }; // maps "*" to end
        if (*(*fd).index.add(j as usize)).nslice == 0 {
            i += 1;
            continue;
        }
        e = cram_cram_index_c_924_cram_container_num2offset_(
            (*fd).index.add(j as usize),
            num as c_int,
            &mut last_pos,
            &mut nc,
        );
        if !e.is_null() {
            break;
        }
        i += 1;
    }

    if !e.is_null() {
        (*e).offset
    } else {
        -1
    }
}

// original: cram_container_offset2num_ (htslib/cram/cram_index.c:970)
pub unsafe fn cram_cram_index_c_970_cram_container_offset2num_(
    e: *mut cram_index,
    pos: libc::off_t,
    last_pos: *mut i64,
    nc: *mut c_int,
) -> *mut cram_index {
    if (*e).offset != 0 {
        if (*e).offset != *last_pos {
            if (*e).offset >= pos {
                return e;
            }
            *nc += 1;
        }
        // else a new multi-ref in same container
        *last_pos = (*e).offset;
    }

    let mut i = 0;
    while i < (*e).nslice {
        let tmp = cram_cram_index_c_970_cram_container_offset2num_(
            (*e).e.add(i as usize),
            pos,
            last_pos,
            nc,
        );
        if !tmp.is_null() {
            return tmp;
        }
        i += 1;
    }

    ptr::null_mut()
}

// original: cram_container_offset2num (htslib/cram/cram_index.c:994)
pub unsafe fn cram_cram_index_c_994_cram_container_offset2num(
    fd: *mut cram_fd,
    pos: libc::off_t,
) -> i64 {
    let fd = fd.cast::<cram_fd_layout>();
    let mut nc = 0;
    let mut last_pos = -9i64;
    let mut e: *mut cram_index = ptr::null_mut();

    let mut i = 0;
    while i < (*fd).index_sz {
        let j = if i + 1 == (*fd).index_sz { 0 } else { i + 1 }; // maps "*" to end
        if (*(*fd).index.add(j as usize)).nslice == 0 {
            i += 1;
            continue;
        }
        e = cram_cram_index_c_970_cram_container_offset2num_(
            (*fd).index.add(j as usize),
            pos,
            &mut last_pos,
            &mut nc,
        );
        if !e.is_null() {
            break;
        }
        i += 1;
    }

    if !e.is_null() {
        nc as i64
    } else {
        -1
    }
}

// original: cram_index_extents (htslib/cram/cram_index.c:1021)
pub unsafe fn cram_cram_index_c_1021_cram_index_extents(
    fd: *mut cram_fd,
    refid: c_int,
    start: hts_pos_t,
    end: hts_pos_t,
    first: *mut libc::off_t,
    last: *mut libc::off_t,
) -> c_int {
    let mut ci: *mut cram_index;

    if !first.is_null() {
        ci = cram_cram_index_c_404_cram_index_query(fd, refid, start, ptr::null_mut());
        if ci.is_null() {
            return -1;
        }
        *first = (*ci).offset;
    }

    if !last.is_null() {
        ci = cram_cram_index_c_531_cram_index_query_last(fd, refid, end);
        if ci.is_null() {
            return -1;
        }
        *last = (*ci).offset;
    }

    0
}
