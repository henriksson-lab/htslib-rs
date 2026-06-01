// Functions translated from htslib/cram/sam.c (pseudo-file I/O for CRAM
// reference cache). Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_int, c_void};

use super::*;

// original: cram_pseek (htslib/sam.c:1582)
//
// Pseudo-seek used by hts_itr_multi_next on CRAM streams. Mirrors C exactly:
//   - try cram_seek(fd, offset, SEEK_SET)
//   - on failure, retry with `offset - fd->first_container` and SEEK_CUR
//     (handles the case where the iterator hands an absolute file offset and
//     the container chain has already been advanced past it)
//   - on success, stash `offset` into curr_position and tear down the
//     current container so the next decode call starts fresh
//
// Returns 0 on success, -1 on failure. Touches `cram_fd_layout` fields
// directly (curr_position / ctr / ctr_mt / ooc) so it lives in cram.rs where
// the struct layout is in scope.
pub unsafe fn cram_sam_c_1582_cram_pseek(fp: *mut c_void, offset: i64, _whence: c_int) -> c_int {
    let fd = fp.cast::<cram_fd>();
    if fd.is_null() {
        return -1;
    }
    let fdl = fd.cast::<cram_fd_layout>();

    if cram_cram_io_c_5431_cram_seek(fd, offset as libc::off_t, libc::SEEK_SET) != 0
        && cram_cram_io_c_5431_cram_seek(
            fd,
            (offset - (*fdl).first_container as i64) as libc::off_t,
            libc::SEEK_CUR,
        ) != 0
    {
        return -1;
    }

    (*fdl).curr_position = offset as libc::off_t;

    if !(*fdl).ctr.is_null() {
        cram_cram_io_c_3705_cram_free_container((*fdl).ctr.cast());
        if !(*fdl).ctr_mt.is_null() && (*fdl).ctr_mt != (*fdl).ctr {
            cram_cram_io_c_3705_cram_free_container((*fdl).ctr_mt.cast());
        }
        (*fdl).ctr = std::ptr::null_mut();
        (*fdl).ctr_mt = std::ptr::null_mut();
        (*fdl).ooc = 0;
    }

    0
}

// original: cram_ptell (htslib/sam.c:1612)
//
// Pseudo-tell paired with cram_pseek. The CRAM disk cursor is only meaningful
// immediately after a fresh seek; otherwise reads consume records from the
// already-fetched container in memory. So we report fd->curr_position, but
// first nudge it forward if the current slice has been fully consumed.
//
// Touches cram_fd_layout / cram_container_layout / cram_slice_layout fields,
// so the body lives in cram.rs.
pub unsafe fn cram_sam_c_1612_cram_ptell(fp: *mut c_void) -> i64 {
    let fd = fp.cast::<cram_fd>();
    if fd.is_null() {
        return -1;
    }
    let fdl = fd.cast::<cram_fd_layout>();
    let c = (*fdl).ctr;
    if !c.is_null() {
        let s = (*c).slice;
        if !s.is_null() && (*s).max_rec != 0 {
            if ((*c).curr_slice + (*s).curr_rec / (*s).max_rec) >= ((*c).max_slice + 1) {
                (*fdl).curr_position += (*c).offset as libc::off_t + (*c).length as libc::off_t;
            }
        }
    }
    (*fdl).curr_position as i64
}
