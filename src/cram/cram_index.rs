// Functions translated from htslib/cram/cram_index.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int, CStr};
use std::ptr::NonNull;

use super::*;

unsafe fn fd_layout<'a>(fd: *mut cram_fd) -> Option<&'a cram_fd_layout> {
    fd.cast::<cram_fd_layout>().as_ref()
}

unsafe fn fd_layout_mut<'a>(fd: *mut cram_fd) -> Option<&'a mut cram_fd_layout> {
    fd.cast::<cram_fd_layout>().as_mut()
}

unsafe fn slice_layout_mut<'a>(s: *mut cram_slice) -> Option<&'a mut cram_slice_layout> {
    s.cast::<cram_slice_layout>().as_mut()
}

unsafe fn container_layout_mut<'a>(
    c: *mut cram_container,
) -> Option<&'a mut cram_container_layout> {
    c.cast::<cram_container_layout>().as_mut()
}

unsafe fn range_layout_mut<'a>(r: *mut cram_range_layout) -> Option<&'a mut cram_range_layout> {
    r.as_mut()
}

fn index_buckets(fd: &cram_fd_layout) -> Option<&[cram_index_layout]> {
    let index = NonNull::new(fd.index.cast::<cram_index_layout>())?;
    let len = fd.index_sz.max(0) as usize;
    Some(unsafe { std::slice::from_raw_parts(index.as_ptr(), len) })
}

fn index_buckets_mut(fd: &mut cram_fd_layout) -> Option<&mut [cram_index_layout]> {
    let index = NonNull::new(fd.index.cast::<cram_index_layout>())?;
    let len = fd.index_sz.max(0) as usize;
    Some(unsafe { std::slice::from_raw_parts_mut(index.as_ptr(), len) })
}

unsafe fn index_node_children(node: &cram_index_layout) -> Option<&[cram_index_layout]> {
    if node.e.is_null() || node.nslice <= 0 {
        None
    } else {
        Some(std::slice::from_raw_parts(node.e, node.nslice as usize))
    }
}

unsafe fn index_node_children_mut(
    node: &mut cram_index_layout,
) -> Option<&mut [cram_index_layout]> {
    if node.e.is_null() || node.nslice <= 0 {
        None
    } else {
        Some(std::slice::from_raw_parts_mut(node.e, node.nslice as usize))
    }
}

unsafe fn index_child_ptr(
    node: NonNull<cram_index_layout>,
    index: usize,
) -> *mut cram_index_layout {
    node.as_ref().e.add(index)
}

unsafe fn next_index_node(node: NonNull<cram_index_layout>) -> Option<NonNull<cram_index_layout>> {
    NonNull::new(node.as_ref().e_next)
}

fn kstring_bytes(k: &kstring_t) -> Option<&[u8]> {
    Some(&k.data)
}

struct KStringParser<'a> {
    bytes: &'a [u8],
}

impl<'a> KStringParser<'a> {
    fn from_kstring(k: &'a kstring_t) -> Option<Self> {
        kstring_bytes(k).map(|bytes| Self { bytes })
    }

    fn from_bytes(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn skip_line_tail(&self, pos: &mut usize) {
        while *pos < self.bytes.len() && self.bytes[*pos] != b'\n' {
            *pos += 1;
        }
        if *pos < self.bytes.len() {
            *pos += 1;
        }
    }

    fn parse_i32(&self, pos: &mut usize, val_p: &mut i32) -> c_int {
        let mut out = 0_i64;
        let ret = self.parse_i64(pos, &mut out);
        if ret == 0 {
            *val_p = out as i32;
        }
        ret
    }

    fn parse_i64(&self, pos: &mut usize, val_p: &mut i64) -> c_int {
        let mut sign: i64 = 1;
        let mut val: i64 = 0;
        let mut p = *pos;

        while p < self.bytes.len() && matches!(self.bytes[p], b' ' | b'\t') {
            p += 1;
        }

        if p < self.bytes.len() && self.bytes[p] == b'-' {
            sign = -1;
            p += 1;
        }

        if p >= self.bytes.len() || !self.bytes[p].is_ascii_digit() {
            return -1;
        }

        while p < self.bytes.len() && self.bytes[p].is_ascii_digit() {
            let digit = (self.bytes[p] - b'0') as i64;
            p += 1;
            val = val * 10 + digit;
        }

        *pos = p;
        *val_p = sign * val;
        0
    }
}

unsafe fn take_index_buckets(fd: &mut cram_fd_layout) -> Vec<cram_index_layout> {
    let len = fd.index_sz.max(0) as usize;
    let buckets = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
        fd.index.cast::<cram_index_layout>(),
        len,
    ));
    fd.index = std::ptr::null_mut();
    fd.index_sz = 0;
    buckets.into_vec()
}

fn install_index_buckets(fd: &mut cram_fd_layout, buckets: Vec<cram_index_layout>) {
    let mut buckets = buckets.into_boxed_slice();
    fd.index_sz = buckets.len() as c_int;
    fd.index = buckets.as_mut_ptr().cast();
    std::mem::forget(buckets);
}

unsafe fn take_index_children(node: &mut cram_index_layout) -> Vec<cram_index_layout> {
    let len = node.nslice.max(0) as usize;
    let cap = node.nalloc.max(0) as usize;
    if node.e.is_null() {
        Vec::new()
    } else {
        let ptr = node.e;
        node.e = std::ptr::null_mut();
        node.nslice = 0;
        node.nalloc = 0;
        Vec::from_raw_parts(ptr, len, cap)
    }
}

fn install_index_children(node: &mut cram_index_layout, mut children: Vec<cram_index_layout>) {
    node.nslice = children.len() as c_int;
    node.nalloc = children.capacity() as c_int;
    node.e = if children.capacity() == 0 {
        std::ptr::null_mut()
    } else {
        children.as_mut_ptr()
    };
    std::mem::forget(children);
}

// original: cram_seek_to_refpos (htslib/cram/cram_index.c:573)
//
// Walks the CRAI for (refid, pos), seeks the underlying file to the container
// containing it, and snapshots the new range on the fd. Returns:
//   0  on success
//  -1  on a general failure (seek failed)
//  -2  when no overlapping slice exists (most commonly: empty chromosome)
pub unsafe fn cram_seek_to_refpos(fd: &mut cram_fd_layout, r: &mut cram_range_layout) -> c_int {
    use crate::htslib_rs::hts::{HTS_IDX_NOCOOR, HTS_IDX_NONE, HTS_IDX_REST, HTS_IDX_START};
    let mut ret: c_int = 0;

    if r.refid == HTS_IDX_NONE {
        ret = -2;
    } else {
        let e = cram_index_query(fd, r.refid, r.start, None);
        if let Some(e) = e {
            if 0 != cram_seek(
                (fd as *mut cram_fd_layout).cast(),
                e.as_ref().offset as libc::off_t,
                libc::SEEK_SET,
            ) {
                ret = -1;
            }
        } else {
            ret = -2;
        }
    }

    if ret != 0 {
        crate::htslib_rs::c_compat::pthread_mutex_lock(&mut fd.range_lock);
        fd.range = *r;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut fd.range_lock);
        return ret;
    }

    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut fd.range_lock);
    fd.range = *r;
    if r.refid == HTS_IDX_NOCOOR {
        fd.range.refid = -1;
        fd.range.start = 0;
    } else if r.refid == HTS_IDX_START || r.refid == HTS_IDX_REST {
        fd.range.refid = -2;
    }
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut fd.range_lock);

    if !fd.ctr.is_null() {
        cram_cram_io_c_3705_cram_free_container(fd.ctr.cast());
        if !fd.ctr_mt.is_null() && fd.ctr_mt != fd.ctr {
            cram_cram_io_c_3705_cram_free_container(fd.ctr_mt.cast());
        }
        fd.ctr = std::ptr::null_mut();
        fd.ctr_mt = std::ptr::null_mut();
        fd.ooc = 0;
        fd.eof = 0;
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
pub unsafe fn cram_index_build_multiref(
    sl: &mut cram_slice_layout,
    fp: &mut crate::htslib_rs::hts::BGZF,
    cpos: libc::off_t,
    landmark: i32,
    sz: c_int,
) -> c_int {
    let mut ref_: i32 = -2;
    let mut ref_start: i64 = 0;
    let mut ref_end: i64 = c_int::MIN as i64;

    let mut last_ref: i32 = -9;
    let mut last_pos: i64 = -9;
    let num_records = (*sl.hdr).num_records;
    let mut i: i32 = 0;
    while i < num_records {
        let rec = sl.crecs.add(i as usize);
        if (*rec).ref_id == last_ref && (*rec).apos < last_pos {
            eprintln!("CRAM file is not sorted by chromosome / position");
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
            let line = format!(
                "{}\t{}\t{}\t{}\t{}\t{}\n",
                ref_,
                ref_start,
                ref_end - ref_start + 1,
                cpos,
                landmark,
                sz,
            );
            if crate::htslib_rs::bgzf::bgzf_write(fp, line.as_ptr().cast(), line.len()) < 0 {
                return -4;
            }
        }

        ref_ = (*rec).ref_id;
        ref_start = (*rec).apos;
        ref_end = (*rec).aend;
        i += 1;
    }

    if ref_ != -2 {
        let line = format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            ref_,
            ref_start,
            ref_end - ref_start + 1,
            cpos,
            landmark,
            sz,
        );
        if crate::htslib_rs::bgzf::bgzf_write(fp, line.as_ptr().cast(), line.len()) < 0 {
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
pub unsafe fn cram_index_slice(
    fd: *mut cram_fd,
    c: *mut cram_container,
    sl: &mut cram_slice_layout,
    fp: &mut crate::htslib_rs::hts::BGZF,
    cpos: libc::off_t,
    spos: libc::off_t,
    sz: libc::off_t,
) -> c_int {
    if sz > c_int::MAX as libc::off_t {
        eprintln!("CRAM slice is too big ({} bytes)", sz);
        return -1;
    }

    if (*sl.hdr).ref_seq_id == -2 {
        cram_index_build_multiref(sl, fp, cpos, spos as i32, sz as i32)
    } else {
        let line = format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            (*sl.hdr).ref_seq_id,
            (*sl.hdr).ref_seq_start,
            (*sl.hdr).ref_seq_span,
            cpos,
            spos as c_int,
            sz as c_int,
        );
        if crate::htslib_rs::bgzf::bgzf_write(fp, line.as_ptr().cast(), line.len()) >= 0 {
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
pub unsafe fn cram_index_query(
    fd: &cram_fd_layout,
    mut refid: c_int,
    mut pos: crate::htslib_rs::hts::hts_pos_t,
    from: Option<NonNull<cram_index_layout>>,
) -> Option<NonNull<cram_index_layout>> {
    use crate::htslib_rs::hts::{HTS_IDX_NOCOOR, HTS_IDX_NONE, HTS_IDX_REST, HTS_IDX_START};

    if let Some(from) = from {
        // Continuation search down the e_next linked list.
        if refid == HTS_IDX_NOCOOR {
            refid = -1;
        }
        if let Some(e) = next_index_node(from).filter(|e| {
            let e = e.as_ref();
            e.refid == refid && (e.start as crate::htslib_rs::hts::hts_pos_t) <= pos
        }) {
            return Some(e);
        }
        return None;
    }

    let Some(index) = NonNull::new(fd.index.cast::<cram_index_layout>()) else {
        return None;
    };

    match refid {
        HTS_IDX_NONE | HTS_IDX_REST => return None,
        v if v == -1 || v == HTS_IDX_NOCOOR => {
            refid = -1;
            pos = 0;
        }
        HTS_IDX_START => {
            // Find the ref-bucket with the smallest first-entry offset.
            let Some(buckets) = index_buckets(fd) else {
                return None;
            };
            let mut best: Option<NonNull<cram_index_layout>> = None;
            for bucket in buckets {
                let Some(first) = NonNull::new(bucket.e) else {
                    continue;
                };
                if best
                    .map(|best| first.as_ref().offset < best.as_ref().offset)
                    .unwrap_or(true)
                {
                    best = Some(first);
                }
            }
            return best;
        }
        _ => {
            if refid < HTS_IDX_NONE || refid + 1 >= fd.index_sz {
                return None;
            }
        }
    }

    let from = NonNull::new(index.as_ptr().add((refid + 1) as usize))
        .expect("index bucket pointer derived from non-null index");
    let Some(entries) = index_node_children(from.as_ref()) else {
        return None;
    };

    // Binary search to find an overlapping bin.
    let mut i: c_int = 0;
    let mut j: c_int = entries.len() as c_int - 1;
    let mut k: c_int = j / 2;
    while k != i {
        let entry = &entries[k as usize];
        if entry.refid > refid {
            j = k;
            k = (j - i) / 2 + i;
            continue;
        }
        if entry.refid < refid {
            i = k;
            k = (j - i) / 2 + i;
            continue;
        }
        if (entry.start as crate::htslib_rs::hts::hts_pos_t) >= pos {
            j = k;
            k = (j - i) / 2 + i;
            continue;
        }
        if (entry.start as crate::htslib_rs::hts::hts_pos_t) < pos {
            i = k;
            k = (j - i) / 2 + i;
            continue;
        }
        k = (j - i) / 2 + i;
    }
    if j >= 0 {
        let entry = &entries[j as usize];
        if (entry.start as crate::htslib_rs::hts::hts_pos_t) < pos && entry.refid == refid {
            i = j;
        }
    }

    // Move backward to the first overlapping bin.
    while i > 0 && (entries[(i - 1) as usize].end as crate::htslib_rs::hts::hts_pos_t) >= pos {
        i -= 1;
    }

    // And forward if our candidate doesn't cover pos.
    while i + 1 < entries.len() as c_int
        && (entries[i as usize].refid < refid
            || (entries[i as usize].end as crate::htslib_rs::hts::hts_pos_t) < pos)
    {
        i += 1;
    }

    NonNull::new(index_child_ptr(from, i as usize))
}

// original: cram_index_free_recurse (htslib/cram/cram_index.c:364)
pub unsafe fn cram_index_free_recurse(node: &mut cram_index_layout) {
    let child_array = node.e;
    if let Some(children) = index_node_children_mut(node) {
        for child in children {
            cram_index_free_recurse(child);
        }
        drop(Vec::from_raw_parts(
            child_array,
            node.nslice as usize,
            node.nalloc as usize,
        ));
    }
}

// original: cram_index_free (htslib/cram/cram_index.c:374)
//
// Walks the top-level CRAI tree (one entry per reference) and recursively
// frees each subtree, then releases the index array and clears the fd field.
pub unsafe fn cram_index_free(fd: &mut cram_fd_layout) {
    let Some(index) = NonNull::new(fd.index.cast::<cram_index_layout>()) else {
        return;
    };
    let mut buckets = take_index_buckets(fd);
    for bucket in &mut buckets {
        cram_index_free_recurse(bucket);
    }
}

unsafe fn link_index_node(
    mut e: NonNull<cram_index_layout>,
    e_last: Option<NonNull<cram_index_layout>>,
) -> Option<NonNull<cram_index_layout>> {
    if let Some(mut e_last) = e_last {
        e_last.as_mut().e_next = e.as_ptr();
    }

    // We don't want to link in the top-level cram_index with
    // offset=0 and start/end = INT_MIN/INT_MAX.
    let mut e_last = if e.as_ref().offset != 0 {
        Some(e)
    } else {
        e_last
    };

    if let Some(children) = index_node_children_mut(e.as_mut()) {
        for child in children {
            e_last = link_index_node(NonNull::from(child), e_last);
        }
    }

    e_last
}

// original: link_index_ (htslib/cram/cram_index.c:93)
//
// Threads a linked list (`e_next`) through the nested containment list of
// `cram_index` entries. The top-level dummy entry with `offset == 0` is
// skipped so it never becomes a `prev` link. Recursive, byte-faithful.
pub unsafe fn cram_cram_index_c_92_link_index_(
    e: *mut cram_index_layout,
    e_last: *mut cram_index_layout,
) -> *mut cram_index_layout {
    let Some(e) = NonNull::new(e) else {
        return e_last;
    };
    link_index_node(e, NonNull::new(e_last))
        .map(NonNull::as_ptr)
        .unwrap_or_else(std::ptr::null_mut)
}

// original: link_index (htslib/cram/cram_index.c:109)
//
// Drives `link_index_` across every ref-bucket so the entries can be walked
// in file order without descending the nested tree on each step.
pub unsafe fn link_index(fd: &mut cram_fd_layout) {
    let Some(buckets) = index_buckets_mut(fd) else {
        return;
    };
    let mut e_last: Option<NonNull<cram_index_layout>> = None;

    for bucket in buckets {
        e_last = link_index_node(NonNull::from(bucket), e_last);
    }

    if let Some(mut e_last) = e_last {
        e_last.as_mut().e_next = std::ptr::null_mut();
    }
}

// original: kget_int32 (htslib/cram/cram_index.c:121)
//
// Parses a signed 32-bit decimal integer from the kstring starting at
// `*pos`, skipping leading spaces/tabs. Returns 0 on success (and advances
// `*pos`), -1 if no digit is found.
pub unsafe fn kget_int32(k: &kstring_t, pos: &mut usize, val_p: &mut i32) -> c_int {
    let Some(parser) = KStringParser::from_kstring(k) else {
        return -1;
    };
    parser.parse_i32(pos, val_p)
}

// original: kget_int64 (htslib/cram/cram_index.c:146)
//
// Same as `kget_int32` but reads into an `i64`.
pub unsafe fn kget_int64(k: &kstring_t, pos: &mut usize, val_p: &mut i64) -> c_int {
    let Some(parser) = KStringParser::from_kstring(k) else {
        return -1;
    };
    parser.parse_i64(pos, val_p)
}

struct ResolvedCraiIndexFilename(NonNull<c_char>);

impl ResolvedCraiIndexFilename {
    fn new(ptr: *mut c_char) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr()
    }
}

impl Drop for ResolvedCraiIndexFilename {
    fn drop(&mut self) {
        unsafe {
            free(self.0.as_ptr().cast());
        }
    }
}

struct OwnedCraiIndexFilename {
    bytes: Vec<u8>,
}

impl OwnedCraiIndexFilename {
    unsafe fn from_base(fn_base: *const c_char) -> Option<Self> {
        let fn_base = fn_base.as_ref()?;
        let fn_base = CStr::from_ptr(fn_base);
        let mut bytes = Vec::with_capacity(fn_base.to_bytes().len() + b".crai".len() + 1);
        bytes.extend_from_slice(fn_base.to_bytes());
        bytes.extend_from_slice(b".crai");
        bytes.push(0);
        Some(Self { bytes })
    }

    fn as_ptr(&self) -> *const c_char {
        self.bytes.as_ptr().cast()
    }
}

struct MallocOwnedIndexBytes {
    ptr: NonNull<u8>,
    len: usize,
}

impl MallocOwnedIndexBytes {
    fn new(ptr: *mut c_char, len: usize) -> Option<Self> {
        NonNull::new(ptr.cast::<u8>()).map(|ptr| Self { ptr, len })
    }

    unsafe fn as_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
    }
}

impl Drop for MallocOwnedIndexBytes {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr.as_ptr().cast());
        }
    }
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
    let Some(fdl) = fd_layout_mut(fd) else {
        return -1;
    };
    let mut buf = [0_u8; 65536];
    let mut index_data: Vec<u8> = Vec::new();
    let mut idx: NonNull<cram_index_layout>;
    let mut idx_stack: Vec<NonNull<cram_index_layout>> = Vec::new();
    let mut pos: usize = 0;

    macro_rules! fail {
        () => {{
            cram_index_free(fdl); // Also sets fd->index = NULL
            return -1;
        }};
    }

    /* Check if already loaded */
    if !fdl.index.is_null() {
        return 0;
    }

    let mut index_buckets: Vec<cram_index_layout> = Vec::new();
    if index_buckets.try_reserve_exact(1).is_err() {
        return -1;
    }
    index_buckets.resize_with(1, || std::mem::zeroed());
    install_index_buckets(fdl, index_buckets);

    idx = match NonNull::new(fdl.index.cast::<cram_index_layout>()) {
        Some(idx) => idx,
        None => return -1,
    };
    idx.as_mut().refid = -1;
    idx.as_mut().start = c_int::MIN;
    idx.as_mut().end = c_int::MAX;

    if idx_stack.try_reserve(1).is_err() {
        fail!();
    }
    idx_stack.push(idx);

    // Support pathX.cram##idx##pathY.crai
    let fn_bytes = CStr::from_ptr(fn_).to_bytes();
    let needle: &[u8] = &HTS_IDX_DELIM[..HTS_IDX_DELIM.len() - 1];
    let fn_delim = fn_bytes
        .windows(needle.len())
        .position(|w| w == needle);
    if let (Some(off), true) = (fn_delim, fn_idx.is_null()) {
        fn_idx = fn_.add(off + needle.len());
    }

    let _resolved_idx_filename = if fn_idx.is_null() {
        let mut resolved_idx: *mut c_char = std::ptr::null_mut();
        if crate::htslib_rs::hts::hts_c_4756_hts_idx_check_local(
            fn_,
            crate::htslib_rs::hts::HTS_FMT_CRAI,
            &mut resolved_idx,
        ) == 0
            && hisremote(fn_) != 0
        {
            resolved_idx = crate::htslib_rs::hts::hts_c_4915_hts_idx_getfn(fn_, c".crai".as_ptr());
        }

        let Some(resolved_idx_filename) = ResolvedCraiIndexFilename::new(resolved_idx) else {
            eprintln!(
                "Could not retrieve index file for '{}'",
                String::from_utf8_lossy(CStr::from_ptr(fn_).to_bytes())
            );
            fail!();
        };
        fn_idx = resolved_idx_filename.as_ptr();
        Some(resolved_idx_filename)
    } else {
        None
    };

    let fp = hopen(fn_idx, c"r".as_ptr());
    if fp.is_null() {
        eprintln!(
            "Could not open index file '{}'",
            String::from_utf8_lossy(CStr::from_ptr(fn_idx).to_bytes())
        );
        fail!();
    }

    // Load the file into memory
    loop {
        let len = htslib_hfile_h_247_hread(fp, buf.as_mut_ptr().cast(), buf.len());
        if len <= 0 {
            if len < 0 || index_data.len() < 2 {
                fail!();
            }
            break;
        }
        if index_data.try_reserve(len as usize).is_err() {
            fail!();
        }
        index_data.extend_from_slice(&buf[..len as usize]);
    }

    if hclose(fp) < 0 {
        fail!();
    }

    // Uncompress if required
    if index_data[0] == 31 && index_data[1] as u8 == 139 {
        let mut l: usize = 0;
        let s = cram_cram_io_c_1157_zlib_mem_inflate(
            index_data.as_mut_ptr().cast(),
            index_data.len(),
            &mut l,
        );
        if s.is_null() {
            fail!();
        }

        let Some(inflated) = MallocOwnedIndexBytes::new(s, l) else {
            fail!();
        };
        let mut inflated_data = Vec::new();
        if inflated_data.try_reserve_exact(l).is_err() {
            fail!();
        }
        inflated_data.extend_from_slice(inflated.as_bytes());
        index_data = inflated_data;
    }

    let parser = KStringParser::from_bytes(&index_data);

    // Parse it line at a time
    while pos < parser.len() {
        let mut e: cram_index_layout = std::mem::zeroed();

        /* 1.1 layout */
        if parser.parse_i32(&mut pos, &mut e.refid) == -1
            || parser.parse_i32(&mut pos, &mut e.start) == -1
            || parser.parse_i32(&mut pos, &mut e.end) == -1
            || parser.parse_i64(&mut pos, &mut e.offset) == -1
            || parser.parse_i32(&mut pos, &mut e.slice) == -1
            || parser.parse_i32(&mut pos, &mut e.len) == -1
        {
            fail!();
        }

        e.end += e.start - 1;

        if e.refid < -1 {
            eprintln!("Malformed index file, refid {}", e.refid);
            fail!();
        }

        if e.refid != idx.as_ref().refid {
            if fdl.index_sz < e.refid + 2 {
                let new_sz = e.refid + 2;
                let mut buckets = take_index_buckets(fdl);
                if buckets
                    .try_reserve_exact((new_sz as usize).saturating_sub(buckets.len()))
                    .is_err()
                {
                    install_index_buckets(fdl, buckets);
                    fail!();
                }
                buckets.resize_with(new_sz as usize, || std::mem::zeroed());
                install_index_buckets(fdl, buckets);
            }
            idx = NonNull::new(
                fdl.index
                    .cast::<cram_index_layout>()
                    .add((e.refid + 1) as usize),
            )
            .expect("index bucket pointer derived from non-null index");
            idx.as_mut().refid = e.refid;
            idx.as_mut().start = c_int::MIN;
            idx.as_mut().end = c_int::MAX;
            idx.as_mut().nslice = 0;
            idx.as_mut().nalloc = 0;
            idx.as_mut().e = std::ptr::null_mut();
            idx_stack.clear();
            idx_stack.push(idx);
        }

        while !(e.start >= idx.as_ref().start && e.end <= idx.as_ref().end)
            || (idx.as_ref().start == 0 && idx.as_ref().refid == -1)
        {
            idx_stack.pop();
            let Some(parent) = idx_stack.last().copied() else {
                fail!();
            };
            idx = parent;
        }

        // Now contains, so append
        if idx.as_ref().nslice + 1 >= idx.as_ref().nalloc {
            let old_nalloc = idx.as_ref().nalloc.max(0) as usize;
            let new_nalloc = if old_nalloc != 0 {
                let Some(new_nalloc) = old_nalloc.checked_mul(2) else {
                    fail!();
                };
                if new_nalloc > c_int::MAX as usize {
                    fail!();
                }
                new_nalloc
            } else {
                16
            };
            let mut children = take_index_children(idx.as_mut());

            if children
                .try_reserve_exact(new_nalloc.saturating_sub(children.capacity()))
                .is_err()
            {
                install_index_children(idx.as_mut(), children);
                fail!();
            }

            install_index_children(idx.as_mut(), children);
        }

        e.nalloc = 0;
        e.nslice = 0;
        e.e = std::ptr::null_mut();
        let ep = {
            let parent = idx.as_mut();
            let ep = NonNull::new(parent.e.add(parent.nslice as usize))
                .expect("child pointer derived from non-null child array");
            parent.nslice += 1;
            *ep.as_ptr() = e;
            ep
        };
        idx = ep;

        if idx_stack.len() == idx_stack.capacity() && idx_stack.try_reserve(1).is_err() {
            fail!();
        }
        idx_stack.push(idx);

        parser.skip_line_tail(&mut pos);
    }

    // Convert NCList to linear linked list
    link_index(fdl);

    //dump_index(fd);

    0
}

// original: cram_index_last (htslib/cram/cram_index.c:504)
//
// Returns the index entry for the last slice on a specific reference,
// descending the `e_next` chain so multi-slice containers resolve to the
// genuine last slice.
pub unsafe fn cram_index_last(
    fd: &cram_fd_layout,
    refid: c_int,
    from: Option<NonNull<cram_index_layout>>,
) -> Option<NonNull<cram_index_layout>> {
    if refid + 1 < 0 || refid + 1 >= fd.index_sz {
        return None;
    }

    let Some(index) = NonNull::new(fd.index.cast::<cram_index_layout>()) else {
        return None;
    };
    let from = from.unwrap_or_else(|| {
        NonNull::new(index.as_ptr().add((refid + 1) as usize))
            .expect("index bucket pointer derived from non-null index")
    });

    // Ref with nothing aligned against it.
    let Some(entries) = index_node_children(from.as_ref()) else {
        return None;
    };

    let slice = entries.len() - 1;

    // e is the last entry in the nested containment list, but it may
    // contain further slices within it.
    let mut e = NonNull::new(index_child_ptr(from, slice))
        .expect("child pointer derived from non-null child array");
    while let Some(next) = next_index_node(e) {
        e = next;
    }

    Some(e)
}

// original: cram_index_query_last (htslib/cram/cram_index.c:532)
//
// Walks the linked list to find the last container overlapping `end`, then
// keeps iterating `e_next` until offset changes to land on the genuine
// file-offset for the end of the container (multi-ref containers may emit
// multiple index entries at the same offset).
pub unsafe fn cram_index_query_last(
    fd: &cram_fd_layout,
    refid: c_int,
    end: crate::htslib_rs::hts::hts_pos_t,
) -> Option<NonNull<cram_index_layout>> {
    let mut e: Option<NonNull<cram_index_layout>> = None;
    let mut prev_e: Option<NonNull<cram_index_layout>>;
    loop {
        prev_e = e;
        e = cram_index_query(fd, refid, end, prev_e);
        if e.is_none() {
            break;
        }
    }

    let Some(mut prev_e) = prev_e else {
        return None;
    };
    e = Some(prev_e);

    // Note: offset of e and e->e_next may be the same if we're using a
    // multi-ref container where a single container generates multiple
    // index entries.
    //
    // We need to keep iterating until offset differs in order to find
    // the genuine file offset for the end of container.
    while let Some(current) = e {
        prev_e = current;
        e = next_index_node(current);
        if e.map(|next| next.as_ref().offset != prev_e.as_ref().offset)
            .unwrap_or(true)
        {
            break;
        }
    }

    Some(prev_e)
}

// original: cram_index_container (htslib/cram/cram_index.c:729)
//
// Walks all landmarks in a container, asserts the on-disk landmark offsets
// match the in-memory ones, reads each slice and indexes it via
// `cram_index_slice`, then frees the slice. Returns 0 on success or the
// error code from the failing step.
pub unsafe fn cram_index_container(
    fd: &mut cram_fd_layout,
    c: &mut cram_container_layout,
    fp: &mut crate::htslib_rs::hts::BGZF,
    cpos: libc::off_t,
) -> c_int {
    let mut j: c_int = 0;

    // 2.0 format
    while j < c.num_landmarks {
        let spos = crate::htslib_rs::hfile::htslib_hfile_h_155_htell(fd.fp);
        if spos - cpos - c.offset as libc::off_t != *c.landmark.add(j as usize) as libc::off_t {
            eprintln!(
                "CRAM slice offset {} does not match landmark {} in container header ({})",
                (spos - cpos - c.offset as libc::off_t) as i64,
                j,
                *c.landmark.add(j as usize),
            );
            return -1;
        }

        let s = cram_cram_io_c_4568_cram_read_slice((fd as *mut cram_fd_layout).cast());
        if s.is_null() {
            return -1;
        }

        let sz = crate::htslib_rs::hfile::htslib_hfile_h_155_htell(fd.fp) - spos;
        let ret = if let Some(s) = s.cast::<cram_slice_layout>().as_mut() {
            cram_index_slice(
                (fd as *mut cram_fd_layout).cast(),
                (c as *mut cram_container_layout).cast(),
                s,
                fp,
                cpos,
                *c.landmark.add(j as usize) as libc::off_t,
                sz,
            )
        } else {
            -1
        };

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
    let Some(fdl) = fd_layout_mut(fd) else {
        return -1;
    };
    let fn_idx_buf: Option<OwnedCraiIndexFilename>;
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
        cram_cram_io_c_5692_cram_set_voption(
            (fdl as *mut cram_fd_layout).cast(),
            CRAM_OPT_REQUIRED_FIELDS,
            &mut args,
        );
    }

    if fn_idx.is_null() {
        let Some(buf) = OwnedCraiIndexFilename::from_base(fn_base) else {
            return -4;
        };
        fn_idx = buf.as_ptr();
        fn_idx_buf = Some(buf);
    } else {
        fn_idx_buf = None;
    }

    let fp = bgzf_open(fn_idx, c"wg".as_ptr());
    if fp.is_null() {
        eprintln!(
            "{}: {}",
            String::from_utf8_lossy(CStr::from_ptr(fn_idx).to_bytes()),
            std::io::Error::last_os_error()
        );
        return -4;
    }

    drop(fn_idx_buf);

    let mut cpos = crate::htslib_rs::hfile::htslib_hfile_h_155_htell(fdl.fp);
    loop {
        let c = cram_read_container((fdl as *mut cram_fd_layout).cast());
        if c.is_null() {
            break;
        }
        if fdl.err != 0 {
            eprintln!("Cram container read: {}", std::io::Error::last_os_error());
            return -1;
        }

        let hpos = crate::htslib_rs::hfile::htslib_hfile_h_155_htell(fdl.fp);

        let cl = c.cast::<cram_container_layout>();
        (*cl).comp_hdr_block = cram_read_block((fdl as *mut cram_fd_layout).cast()).cast();
        if (*cl).comp_hdr_block.is_null() {
            return -1;
        }
        assert!((*(*cl).comp_hdr_block).content_type == CRAM_CONTENT_TYPE_COMPRESSION_HEADER);

        (*cl).comp_hdr = cram_decode_compression_header(
            (fdl as *mut cram_fd_layout).cast(),
            (*cl).comp_hdr_block.cast(),
        )
        .cast();
        if (*cl).comp_hdr.is_null() {
            return -1;
        }

        if (*cl).ref_seq_id as i64 == last_ref && (*cl).ref_seq_start < last_start {
            eprintln!("CRAM file is not sorted by chromosome / position");
            return -2;
        }
        last_ref = (*cl).ref_seq_id as i64;
        last_start = (*cl).ref_seq_start;

        let index_ret = if let (Some(cl), Some(fp)) = (cl.as_mut(), fp.as_mut()) {
            cram_index_container(fdl, cl, fp, cpos)
        } else {
            -1
        };
        if index_ret < 0 {
            bgzf_close(fp);
            return -1;
        }

        let next_cpos = crate::htslib_rs::hfile::htslib_hfile_h_155_htell(fdl.fp);
        if next_cpos != hpos + (*cl).length as libc::off_t {
            eprintln!(
                "Length {} in container header at offset {} does not match block lengths ({})",
                (*cl).length,
                cpos as i64,
                (next_cpos - hpos) as i64,
            );
            return -1;
        }
        cpos = next_cpos;

        cram_cram_io_c_3705_cram_free_container(c);
    }
    if fdl.err != 0 {
        bgzf_close(fp);
        return -1;
    }

    if bgzf_close(fp) >= 0 {
        0
    } else {
        -4
    }
}
