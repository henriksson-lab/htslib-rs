// Functions translated from htslib/cram/cram_index.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int};

use super::*;

// original: cram_seek_to_refpos (htslib/cram/cram_index.c:573)
//
// Walks the CRAI for (refid, pos), seeks the underlying file to the container
// containing it, and snapshots the new range on the fd. Returns:
//   0  on success
//  -1  on a general failure (seek failed)
//  -2  when no overlapping slice exists (most commonly: empty chromosome)
pub unsafe fn cram_cram_index_c_573_cram_seek_to_refpos(
    fd: *mut cram_fd,
    r: *mut cram_range_layout,
) -> c_int {
    use crate::htslib_rs::hts::{HTS_IDX_NOCOOR, HTS_IDX_NONE, HTS_IDX_REST, HTS_IDX_START};
    let fdl = fd.cast::<cram_fd_layout>();
    let mut ret: c_int = 0;

    if (*r).refid == HTS_IDX_NONE {
        ret = -2;
    } else {
        let e = cram_cram_index_c_404_cram_index_query(
            fd,
            (*r).refid,
            (*r).start,
            std::ptr::null_mut(),
        );
        if !e.is_null() {
            if 0 != cram_seek(fd, (*e).offset as libc::off_t, libc::SEEK_SET) {
                ret = -1;
            }
        } else {
            ret = -2;
        }
    }

    if ret != 0 {
        crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).range_lock);
        (*fdl).range = *r;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).range_lock);
        return ret;
    }

    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).range_lock);
    (*fdl).range = *r;
    if (*r).refid == HTS_IDX_NOCOOR {
        (*fdl).range.refid = -1;
        (*fdl).range.start = 0;
    } else if (*r).refid == HTS_IDX_START || (*r).refid == HTS_IDX_REST {
        (*fdl).range.refid = -2;
    }
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).range_lock);

    if !(*fdl).ctr.is_null() {
        cram_cram_io_c_3705_cram_free_container((*fdl).ctr.cast());
        if !(*fdl).ctr_mt.is_null() && (*fdl).ctr_mt != (*fdl).ctr {
            cram_cram_io_c_3705_cram_free_container((*fdl).ctr_mt.cast());
        }
        (*fdl).ctr = std::ptr::null_mut();
        (*fdl).ctr_mt = std::ptr::null_mut();
        (*fdl).ooc = 0;
        (*fdl).eof = 0;
    }

    0
}

// original: cram_index_build_multiref (htslib/cram/cram_index.c:632)
//
// Used in write mode only by `cram_index_slice` below (we never call this on
// the read side from production). The C source's read-side path calls
// `cram_decode_slice` to populate (*s).crecs first; in write mode the slice
// already has its crecs filled by the encoder, so the decode is skipped.
//
// Emits one CRAI line per ref/pos run within the multiref slice.
pub unsafe fn cram_cram_index_c_632_cram_index_build_multiref(
    _fd: *mut cram_fd,
    _c: *mut cram_container,
    s: *mut cram_slice,
    fp: *mut crate::htslib_rs::hts::BGZF,
    cpos: libc::off_t,
    landmark: i32,
    sz: c_int,
) -> c_int {
    let sl = s.cast::<cram_slice_layout>();
    let mut ref_: i32 = -2;
    let mut ref_start: i64 = 0;
    let mut ref_end: i64 = c_int::MIN as i64;
    let mut buf = [0 as c_char; 1024];

    let mut last_ref: i32 = -9;
    let mut last_pos: i64 = -9;
    let num_records = (*(*sl).hdr).num_records;
    let mut i: i32 = 0;
    while i < num_records {
        let rec = (*sl).crecs.add(i as usize);
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
                cpos,
                landmark,
                sz,
            );
            if crate::htslib_rs::bgzf::bgzf_write(
                fp,
                buf.as_ptr().cast(),
                libc::strlen(buf.as_ptr()),
            ) < 0
            {
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
            cpos,
            landmark,
            sz,
        );
        if crate::htslib_rs::bgzf::bgzf_write(fp, buf.as_ptr().cast(), libc::strlen(buf.as_ptr()))
            < 0
        {
            return -4;
        }
    }

    0
}

// original: cram_index_slice (htslib/cram/cram_index.c:695)
//
// Emits one or more CRAI lines for a slice. Simple case (single-ref): one
// line. Multi-ref slice (ref_seq_id == -2): one line per ref/pos run via
// cram_index_build_multiref above.
pub unsafe fn cram_cram_index_c_695_cram_index_slice(
    fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
    fp: *mut crate::htslib_rs::hts::BGZF,
    cpos: libc::off_t,
    spos: libc::off_t,
    sz: libc::off_t,
) -> c_int {
    let mut buf = [0 as c_char; 1024];

    if sz > c_int::MAX as libc::off_t {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"CRAM slice is too big (%ld bytes)\n".as_ptr(),
            sz,
        );
        return -1;
    }

    let sl = s.cast::<cram_slice_layout>();
    if (*(*sl).hdr).ref_seq_id == -2 {
        cram_cram_index_c_632_cram_index_build_multiref(fd, c, s, fp, cpos, spos as i32, sz as i32)
    } else {
        libc::snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"%d\t%ld\t%ld\t%ld\t%d\t%d\n".as_ptr(),
            (*(*sl).hdr).ref_seq_id,
            (*(*sl).hdr).ref_seq_start,
            (*(*sl).hdr).ref_seq_span,
            cpos,
            spos as c_int,
            sz as c_int,
        );
        if crate::htslib_rs::bgzf::bgzf_write(fp, buf.as_ptr().cast(), libc::strlen(buf.as_ptr()))
            >= 0
        {
            0
        } else {
            -4
        }
    }
}

// original: cram_index_query (htslib/cram/cram_index.c:404)
//
// Walks the loaded CRAI to find the first slice index entry overlapping
// (refid, pos). When `from` is non-null this is a continuation search down
// the e_next linked list. Returns NULL when the position is unindexed or
// outside the data; otherwise the matching `cram_index_layout *`.
pub unsafe fn cram_cram_index_c_404_cram_index_query(
    fd: *mut cram_fd,
    mut refid: c_int,
    mut pos: crate::htslib_rs::hts::hts_pos_t,
    mut from: *mut cram_index_layout,
) -> *mut cram_index_layout {
    use crate::htslib_rs::hts::{HTS_IDX_NOCOOR, HTS_IDX_NONE, HTS_IDX_REST, HTS_IDX_START};
    let fdl = fd.cast::<cram_fd_layout>();
    let index = (*fdl).index.cast::<cram_index_layout>();

    if !from.is_null() {
        // Continuation search down the e_next linked list.
        if refid == HTS_IDX_NOCOOR {
            refid = -1;
        }
        let e = (*from).e_next;
        if !e.is_null()
            && (*e).refid == refid
            && ((*e).start as crate::htslib_rs::hts::hts_pos_t) <= pos
        {
            return e;
        }
        return std::ptr::null_mut();
    }

    match refid {
        HTS_IDX_NONE | HTS_IDX_REST => return std::ptr::null_mut(),
        v if v == -1 || v == HTS_IDX_NOCOOR => {
            refid = -1;
            pos = 0;
        }
        HTS_IDX_START => {
            // Find the ref-bucket with the smallest first-entry offset.
            let mut min_idx = i64::MAX;
            let mut i: c_int = 0;
            let mut j: c_int = -1;
            while i < (*fdl).index_sz {
                let bucket = index.add(i as usize);
                if !(*bucket).e.is_null() && (*(*bucket).e).offset < min_idx {
                    min_idx = (*(*bucket).e).offset;
                    j = i;
                }
                i += 1;
            }
            if j < 0 {
                return std::ptr::null_mut();
            }
            return (*index.add(j as usize)).e;
        }
        _ => {
            if refid < HTS_IDX_NONE || refid + 1 >= (*fdl).index_sz {
                return std::ptr::null_mut();
            }
        }
    }

    from = index.add((refid + 1) as usize);

    if (*from).e.is_null() {
        return std::ptr::null_mut();
    }

    // Binary search to find an overlapping bin.
    let mut i: c_int = 0;
    let mut j: c_int = (*index.add((refid + 1) as usize)).nslice - 1;
    let mut k: c_int = j / 2;
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
        if ((*(*from).e.add(k as usize)).start as crate::htslib_rs::hts::hts_pos_t) >= pos {
            j = k;
            k = (j - i) / 2 + i;
            continue;
        }
        if ((*(*from).e.add(k as usize)).start as crate::htslib_rs::hts::hts_pos_t) < pos {
            i = k;
            k = (j - i) / 2 + i;
            continue;
        }
        k = (j - i) / 2 + i;
    }
    if j >= 0
        && ((*(*from).e.add(j as usize)).start as crate::htslib_rs::hts::hts_pos_t) < pos
        && (*(*from).e.add(j as usize)).refid == refid
    {
        i = j;
    }

    // Move backward to the first overlapping bin.
    while i > 0
        && ((*(*from).e.add((i - 1) as usize)).end as crate::htslib_rs::hts::hts_pos_t) >= pos
    {
        i -= 1;
    }

    // And forward if our candidate doesn't cover pos.
    while i + 1 < (*from).nslice
        && ((*(*from).e.add(i as usize)).refid < refid
            || ((*(*from).e.add(i as usize)).end as crate::htslib_rs::hts::hts_pos_t) < pos)
    {
        i += 1;
    }

    (*from).e.add(i as usize)
}

// original: cram_index_free_recurse (htslib/cram/cram_index.c:364)
pub unsafe fn cram_cram_index_c_364_cram_index_free_recurse(e: *mut cram_index_layout) {
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
    let fdl = fd.cast::<cram_fd_layout>();
    if (*fdl).index.is_null() {
        return;
    }
    let index = (*fdl).index.cast::<cram_index_layout>();
    let mut i: c_int = 0;
    while i < (*fdl).index_sz {
        cram_cram_index_c_364_cram_index_free_recurse(index.offset(i as isize));
        i += 1;
    }
    free(index.cast());
    (*fdl).index = std::ptr::null_mut();
}

// original: link_index_ (htslib/cram/cram_index.c:93)
//
// Threads a linked list (`e_next`) through the nested containment list of
// `cram_index` entries. The top-level dummy entry with `offset == 0` is
// skipped so it never becomes a `prev` link. Recursive, byte-faithful.
pub unsafe fn cram_cram_index_c_92_link_index_(
    e: *mut cram_index_layout,
    mut e_last: *mut cram_index_layout,
) -> *mut cram_index_layout {
    if !e_last.is_null() {
        (*e_last).e_next = e;
    }

    // We don't want to link in the top-level cram_index with
    // offset=0 and start/end = INT_MIN/INT_MAX.
    if (*e).offset != 0 {
        e_last = e;
    }

    let mut i: c_int = 0;
    while i < (*e).nslice {
        e_last = cram_cram_index_c_92_link_index_((*e).e.add(i as usize), e_last);
        i += 1;
    }

    e_last
}

// original: link_index (htslib/cram/cram_index.c:109)
//
// Drives `link_index_` across every ref-bucket so the entries can be walked
// in file order without descending the nested tree on each step.
pub unsafe fn cram_cram_index_c_108_link_index(fd: *mut cram_fd) {
    let fdl = fd.cast::<cram_fd_layout>();
    let index = (*fdl).index.cast::<cram_index_layout>();
    let mut e_last: *mut cram_index_layout = std::ptr::null_mut();

    let mut i: c_int = 0;
    while i < (*fdl).index_sz {
        e_last = cram_cram_index_c_92_link_index_(index.add(i as usize), e_last);
        i += 1;
    }

    if !e_last.is_null() {
        (*e_last).e_next = std::ptr::null_mut();
    }
}

// original: kget_int32 (htslib/cram/cram_index.c:121)
//
// Parses a signed 32-bit decimal integer from the kstring starting at
// `*pos`, skipping leading spaces/tabs. Returns 0 on success (and advances
// `*pos`), -1 if no digit is found.
pub unsafe fn cram_cram_index_c_120_kget_int32(
    k: *mut kstring_t,
    pos: *mut usize,
    val_p: *mut i32,
) -> c_int {
    let mut sign: i32 = 1;
    let mut val: i32 = 0;
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

// original: kget_int64 (htslib/cram/cram_index.c:146)
//
// Same as `kget_int32` but reads into an `i64`.
pub unsafe fn cram_cram_index_c_145_kget_int64(
    k: *mut kstring_t,
    pos: *mut usize,
    val_p: *mut i64,
) -> c_int {
    let mut sign: i64 = 1;
    let mut val: i64 = 0;
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

// original: cram_index_load (htslib/cram/cram_index.c:177)
//
// Loads a CRAM `.crai` index into memory. Handles `path.cram##idx##path.crai`
// notation and falls back to remote / local index resolution via
// `hts_idx_check_local` + `hts_idx_getfn`. Returns 0 on success, -1 on
// failure (also frees any half-built index on failure).
pub unsafe fn cram_cram_index_c_176_cram_index_load(
    fd: *mut cram_fd,
    fn_: *const c_char,
    mut fn_idx: *const c_char,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let mut tfn_idx: *mut c_char = std::ptr::null_mut();
    let mut buf = [0 as c_char; 65536];
    let mut kstr: kstring_t = std::mem::zeroed();
    let mut idx: *mut cram_index_layout;
    let mut idx_stack: *mut *mut cram_index_layout;
    let mut idx_stack_alloc: c_int = 0;
    let mut idx_stack_ptr: c_int = 0;
    let mut pos: usize = 0;

    macro_rules! fail {
        () => {{
            free(kstr.s.cast());
            free(idx_stack.cast());
            free(tfn_idx.cast());
            cram_cram_index_c_374_cram_index_free(fd); // Also sets fd->index = NULL
            return -1;
        }};
    }

    /* Check if already loaded */
    if !(*fdl).index.is_null() {
        return 0;
    }

    (*fdl).index_sz = 1;
    (*fdl).index = calloc(1, std::mem::size_of::<cram_index_layout>() as u64).cast();
    if (*fdl).index.is_null() {
        return -1;
    }

    idx = (*fdl).index.cast::<cram_index_layout>();
    (*idx).refid = -1;
    (*idx).start = c_int::MIN;
    (*idx).end = c_int::MAX;

    idx_stack_alloc += 1;
    idx_stack = calloc(
        idx_stack_alloc as u64,
        std::mem::size_of::<*mut cram_index_layout>() as u64,
    )
    .cast();
    if idx_stack.is_null() {
        fail!();
    }

    *idx_stack.add(idx_stack_ptr as usize) = idx;

    // Support pathX.cram##idx##pathY.crai
    let fn_delim = libc::strstr(fn_, HTS_IDX_DELIM.as_ptr().cast());
    if !fn_delim.is_null() && fn_idx.is_null() {
        fn_idx = fn_delim.add(libc::strlen(HTS_IDX_DELIM.as_ptr().cast()));
    }

    if fn_idx.is_null() {
        if crate::htslib_rs::hts::hts_c_4756_hts_idx_check_local(
            fn_,
            crate::htslib_rs::hts::HTS_FMT_CRAI,
            &mut tfn_idx,
        ) == 0
            && hisremote(fn_) != 0
        {
            tfn_idx = crate::htslib_rs::hts::hts_c_4915_hts_idx_getfn(fn_, c".crai".as_ptr());
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

    let fp = hopen(fn_idx, c"r".as_ptr());
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
        let mut l: usize = 0;
        let s = cram_cram_io_c_1157_zlib_mem_inflate(kstr.s, kstr.l, &mut l);
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
        let mut e: cram_index_layout = std::mem::zeroed();

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
                let index_end = (*fdl).index_sz as usize * std::mem::size_of::<cram_index_layout>();
                let new_idx = realloc(
                    (*fdl).index.cast(),
                    (new_sz as usize * std::mem::size_of::<cram_index_layout>()) as u64,
                )
                .cast::<cram_index_layout>();
                if new_idx.is_null() {
                    fail!();
                }

                (*fdl).index = new_idx.cast();
                (*fdl).index_sz = new_sz;
                libc::memset(
                    (new_idx.cast::<c_char>()).add(index_end).cast(),
                    0,
                    (*fdl).index_sz as usize * std::mem::size_of::<cram_index_layout>() - index_end,
                );
            }
            idx = (*fdl)
                .index
                .cast::<cram_index_layout>()
                .add((e.refid + 1) as usize);
            (*idx).refid = e.refid;
            (*idx).start = c_int::MIN;
            (*idx).end = c_int::MAX;
            (*idx).nslice = 0;
            (*idx).nalloc = 0;
            (*idx).e = std::ptr::null_mut();
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
                ((*idx).nalloc as usize * std::mem::size_of::<cram_index_layout>()) as u64,
            )
            .cast::<cram_index_layout>();
            if new_e.is_null() {
                fail!();
            }

            (*idx).e = new_e;
        }

        e.nalloc = 0;
        e.nslice = 0;
        e.e = std::ptr::null_mut();
        let ep = (*idx).e.add((*idx).nslice as usize);
        (*idx).nslice += 1;
        *ep = e;
        idx = ep;

        idx_stack_ptr += 1;
        if idx_stack_ptr >= idx_stack_alloc {
            idx_stack_alloc *= 2;
            let new_stack = realloc(
                idx_stack.cast(),
                (idx_stack_alloc as usize * std::mem::size_of::<*mut cram_index_layout>()) as u64,
            )
            .cast::<*mut cram_index_layout>();
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

// original: cram_index_last (htslib/cram/cram_index.c:504)
//
// Returns the index entry for the last slice on a specific reference,
// descending the `e_next` chain so multi-slice containers resolve to the
// genuine last slice.
pub unsafe fn cram_cram_index_c_503_cram_index_last(
    fd: *mut cram_fd,
    refid: c_int,
    mut from: *mut cram_index_layout,
) -> *mut cram_index_layout {
    let fdl = fd.cast::<cram_fd_layout>();

    if refid + 1 < 0 || refid + 1 >= (*fdl).index_sz {
        return std::ptr::null_mut();
    }

    let index = (*fdl).index.cast::<cram_index_layout>();
    if from.is_null() {
        from = index.add((refid + 1) as usize);
    }

    // Ref with nothing aligned against it.
    if (*from).e.is_null() {
        return std::ptr::null_mut();
    }

    let slice = (*index.add((refid + 1) as usize)).nslice - 1;

    // e is the last entry in the nested containment list, but it may
    // contain further slices within it.
    let mut e = (*from).e.add(slice as usize);
    while !(*e).e_next.is_null() {
        e = (*e).e_next;
    }

    e
}

// original: cram_index_query_last (htslib/cram/cram_index.c:532)
//
// Walks the linked list to find the last container overlapping `end`, then
// keeps iterating `e_next` until offset changes to land on the genuine
// file-offset for the end of the container (multi-ref containers may emit
// multiple index entries at the same offset).
pub unsafe fn cram_cram_index_c_531_cram_index_query_last(
    fd: *mut cram_fd,
    refid: c_int,
    end: crate::htslib_rs::hts::hts_pos_t,
) -> *mut cram_index_layout {
    let mut e: *mut cram_index_layout = std::ptr::null_mut();
    let mut prev_e: *mut cram_index_layout;
    loop {
        prev_e = e;
        e = cram_cram_index_c_404_cram_index_query(fd, refid, end, prev_e);
        if e.is_null() {
            break;
        }
    }

    if prev_e.is_null() {
        return std::ptr::null_mut();
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

// original: cram_index_container (htslib/cram/cram_index.c:729)
//
// Walks all landmarks in a container, asserts the on-disk landmark offsets
// match the in-memory ones, reads each slice and indexes it via
// `cram_index_slice`, then frees the slice. Returns 0 on success or the
// error code from the failing step.
pub unsafe fn cram_cram_index_c_727_cram_index_container(
    fd: *mut cram_fd,
    c: *mut cram_container,
    fp: *mut crate::htslib_rs::hts::BGZF,
    cpos: libc::off_t,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let mut j: c_int = 0;

    // 2.0 format
    while j < (*cl).num_landmarks {
        let spos = crate::htslib_rs::hfile::htslib_hfile_h_155_htell((*fdl).fp);
        if spos - cpos - (*cl).offset as libc::off_t
            != *(*cl).landmark.add(j as usize) as libc::off_t
        {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"CRAM slice offset %ld does not match landmark %d in container header (%d)\n"
                    .as_ptr(),
                (spos - cpos - (*cl).offset as libc::off_t) as i64,
                j,
                *(*cl).landmark.add(j as usize),
            );
            return -1;
        }

        let s = cram_cram_io_c_4568_cram_read_slice(fd);
        if s.is_null() {
            return -1;
        }

        let sz = crate::htslib_rs::hfile::htslib_hfile_h_155_htell((*fdl).fp) - spos;
        let ret = cram_cram_index_c_695_cram_index_slice(
            fd,
            c,
            s,
            fp,
            cpos,
            *(*cl).landmark.add(j as usize) as libc::off_t,
            sz,
        );

        cram_cram_io_c_4421_cram_free_slice(s);

        if ret < 0 {
            return ret;
        }

        j += 1;
    }

    0
}

// original: cram_index_build (htslib/cram/cram_index.c:780)
//
// Builds a `.crai` index for an open CRAM file by walking every container
// and emitting one CRAI line per slice via `cram_index_container` ->
// `cram_index_slice`. Sets CRAM_OPT_REQUIRED_FIELDS = RNAME|POS|CIGAR
// up front (needed for multi-ref slice decoding inside multiref builder).
pub unsafe fn cram_cram_index_c_779_cram_index_build(
    fd: *mut cram_fd,
    fn_base: *const c_char,
    mut fn_idx: *const c_char,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let mut fn_idx_str: kstring_t = std::mem::zeroed();
    let mut last_ref: i64 = -9;
    let mut last_start: i64 = -9;

    // Useful for cram_index_build_multiref. The C source goes through
    // `cram_set_option(fd, CRAM_OPT_REQUIRED_FIELDS, RNAME|POS|CIGAR)`,
    // which is a thin variadic wrapper around `cram_set_voption`. We
    // construct the va_list with one int argument and dispatch the same
    // way (byte-faithful to the SysV amd64 va_list ABI).
    {
        let mut reg_save = [0usize; 6];
        let overflow = [0usize; 8];
        reg_save[0] = (SAM_RNAME | SAM_POS | SAM_CIGAR) as usize;
        let mut args = crate::htslib_rs::c_compat::__va_list_tag {
            gp_offset: 0,
            fp_offset: 48,
            overflow_arg_area: overflow.as_ptr() as *mut _,
            reg_save_area: reg_save.as_mut_ptr().cast(),
        };
        cram_cram_io_c_5692_cram_set_voption(fd, CRAM_OPT_REQUIRED_FIELDS, &mut args);
    }

    if fn_idx.is_null() {
        crate::htslib_rs::hts::kputs(fn_base, &mut fn_idx_str);
        crate::htslib_rs::hts::kputs(c".crai".as_ptr(), &mut fn_idx_str);
        fn_idx = fn_idx_str.s;
    }

    let fp = bgzf_open(fn_idx, c"wg".as_ptr());
    if fp.is_null() {
        libc::perror(fn_idx);
        free(fn_idx_str.s.cast());
        return -4;
    }

    free(fn_idx_str.s.cast());

    let mut cpos = crate::htslib_rs::hfile::htslib_hfile_h_155_htell((*fdl).fp);
    loop {
        let c = cram_read_container(fd);
        if c.is_null() {
            break;
        }
        if (*fdl).err != 0 {
            libc::perror(c"Cram container read".as_ptr());
            return -1;
        }

        let hpos = crate::htslib_rs::hfile::htslib_hfile_h_155_htell((*fdl).fp);

        let cl = c.cast::<cram_container_layout>();
        (*cl).comp_hdr_block = cram_read_block(fd).cast();
        if (*cl).comp_hdr_block.is_null() {
            return -1;
        }
        assert!((*(*cl).comp_hdr_block).content_type == CRAM_CONTENT_TYPE_COMPRESSION_HEADER);

        (*cl).comp_hdr = cram_decode_compression_header(fd, (*cl).comp_hdr_block.cast()).cast();
        if (*cl).comp_hdr.is_null() {
            return -1;
        }

        if (*cl).ref_seq_id as i64 == last_ref && (*cl).ref_seq_start < last_start {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"CRAM file is not sorted by chromosome / position\n".as_ptr(),
            );
            return -2;
        }
        last_ref = (*cl).ref_seq_id as i64;
        last_start = (*cl).ref_seq_start;

        if cram_cram_index_c_727_cram_index_container(fd, c, fp, cpos) < 0 {
            bgzf_close(fp);
            return -1;
        }

        let next_cpos = crate::htslib_rs::hfile::htslib_hfile_h_155_htell((*fdl).fp);
        if next_cpos != hpos + (*cl).length as libc::off_t {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Length %d in container header at offset %lld does not match block lengths (%lld)\n"
                    .as_ptr(),
                (*cl).length,
                cpos as libc::c_longlong,
                (next_cpos - hpos) as libc::c_longlong,
            );
            return -1;
        }
        cpos = next_cpos;

        cram_cram_io_c_3705_cram_free_container(c);
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
