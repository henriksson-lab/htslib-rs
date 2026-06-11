// Functions translated from htslib/header.c.
// Extracted from src/sam.rs.

use std::ptr::NonNull;

use crate::htslib_rs::hts::{hts_pos_t, kputc, kputsn, ks_free, ks_release, kstring_t};
use crate::htslib_rs::sam::*;

// original: sam_hdr_add_lines (htslib/header.c:1658)
//
// Matches htslib's invariant of routing every header mutation through hrecs:
// fill hrecs from the cached text on first call, parse the incoming lines into
// the hrecs global list, refresh the hashes, rebuild the target arrays, mark
// dirty, and redact the cached text (so the next read/write rebuilds from
// hrecs). The previous text-append fallback was an intentional divergence from
// C that produced verbatim insertion order; closing it brings serialization
// into line with htslib's canonical type-grouped output.
pub unsafe fn sam_hdr_add_lines(h: &mut sam_hdr_t, lines: &[u8]) -> i32 {
    if lines.is_empty() {
        return 0;
    }
    if h.hrecs.is_null() && sam_hdr_fill_hrecs(h) < 0 {
        return -1;
    }
    let hrecs = h.hrecs;
    if sam_hrecs_parse_lines(hrecs, lines.as_ptr(), lines.len()) != 0 {
        return -1;
    }
    // parse_single_line only appends to the global list; refresh ref_/rg_/pg_
    // arrays and the per-type hash tables so subsequent lookups see the new
    // records.
    if sam_hrecs_update_hashes(hrecs) < 0 {
        return -1;
    }
    if rebuild_target_arrays(h) != 0 {
        return -1;
    }
    (*hrecs).dirty = 1;
    redact_header_text(h);
    0
}

// original: sam_hdr_add_line (htslib/header.c:1693)
//
// Pre-validates the type / tag pairs (rejecting embedded tabs, colons in
// keys, and newlines), fills hrecs if not yet populated, then routes through
// sam_hdr_add_line_hrecs. The pre-validation is stricter than C's
// sam_hrecs_vadd (which does fewer checks); we keep it so callers that relied
// on the previous text-mode rejection of bad inputs still see -1.
pub unsafe fn sam_hdr_add_line(
    h: &mut sam_hdr_t,
    type_: &[u8],
    tags: &[(Option<&[u8]>, Option<&[u8]>)],
) -> i32 {
    if type_.len() != 2 {
        return -1;
    }

    if type_ == b"CO" {
        if tags.len() != 1 || tags[0].0.is_none() || tags[0].1.is_some() {
            return -1;
        }
        let comment = tags[0].0.unwrap();
        if comment.contains(&b'\n') {
            return -1;
        }
    } else {
        if !matches!(type_, b"HD" | b"SQ" | b"RG" | b"PG") || tags.is_empty() {
            return -1;
        }
        for &(key, value) in tags {
            let (Some(key), Some(value)) = (key, value) else {
                return -1;
            };
            if key.is_empty()
                || key.contains(&b'\t')
                || key.contains(&b':')
                || key.contains(&b'\n')
                || value.contains(&b'\t')
                || value.contains(&b'\n')
            {
                return -1;
            }
        }
    }

    if h.hrecs.is_null() && sam_hdr_fill_hrecs(h) < 0 {
        return -1;
    }
    // Adapt to the pointer-based callee: build NUL-terminated owned copies of
    // type_ and each tag key/value (callees read them via CStr::from_ptr), then
    // a parallel pointer slice. Backing buffers are collected first so that no
    // reallocation invalidates the pointers taken afterwards.
    let mut type_c = type_.to_vec();
    type_c.push(0);
    let backing: Vec<(Option<Vec<u8>>, Option<Vec<u8>>)> = tags
        .iter()
        .map(|&(k, v)| {
            (
                k.map(|b| [b, &[0u8]].concat()),
                v.map(|b| [b, &[0u8]].concat()),
            )
        })
        .collect();
    let ptr_tags: Vec<(*const u8, *const u8)> = backing
        .iter()
        .map(|(k, v)| {
            (
                k.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
                v.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
            )
        })
        .collect();
    sam_hdr_add_line_hrecs(h, type_c.as_ptr(), &ptr_tags)
}

// original: sam_hdr_update_line (htslib/header.c:1909)
pub unsafe fn sam_hdr_update_line(
    h: &mut sam_hdr_t,
    type_: &[u8],
    id: Option<(&[u8], &[u8])>,
    tags: &[(Option<&[u8]>, Option<&[u8]>)],
) -> i32 {
    if h.hrecs.is_null() && sam_hdr_fill_hrecs(h) < 0 {
        return -1;
    }

    let hrecs = h.hrecs;
    let Some(hrecs_ref) = hrecs.as_mut() else {
        return -1;
    };
    let Some(mut ty) = sam_hrecs_find_type_id(hrecs_ref, type_, id) else {
        return -1;
    };
    let ty_ref = ty.as_mut();

    // Adapt to the pointer-based callees: NUL-terminated owned backing buffers
    // for each tag key/value, then a parallel pointer slice.
    let backing: Vec<(Option<Vec<u8>>, Option<Vec<u8>>)> = tags
        .iter()
        .map(|&(k, v)| {
            (
                k.map(|b| [b, &[0u8]].concat()),
                v.map(|b| [b, &[0u8]].concat()),
            )
        })
        .collect();
    let ptr_tags: Vec<(*const u8, *const u8)> = backing
        .iter()
        .map(|(k, v)| {
            (
                k.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
                v.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
            )
        })
        .collect();

    match check_for_name_update(hrecs, ty_ref, &ptr_tags) {
        SamHrecNameUpdate::Clash => return -1,
        SamHrecNameUpdate::Changed
            if header_h_58_TYPEKEY(type_.as_ptr()) == header_h_58_TYPEKEY(b"PG".as_ptr()) =>
        {
            return -1;
        }
        _ => {}
    }

    if sam_hrecs_update_pairs(hrecs, ty_ref, &ptr_tags) < 0 {
        return -1;
    }

    if sam_hrecs_update_hashes(hrecs) < 0 {
        return -1;
    }
    if ty_ref.type_ == header_h_58_TYPEKEY(b"SQ".as_ptr()) && rebuild_target_arrays(h) < 0 {
        return -1;
    }
    0
}

pub unsafe fn sam_hdr_find_line_id(
    h: &mut sam_hdr_t,
    type_: &[u8],
    id_key: &[u8],
    id_val: &[u8],
    ks: &mut kstring_t,
) -> i32 {
    // hrecs-backed headers: sync the serialized text from hrecs, then reuse the
    // text-backed lookup below. We must not delegate to hts_sys: the C library
    // cannot index a Rust-built hrecs hash table, and for a header with no
    // hrecs it would build a C-owned hrecs into our struct that our allocator
    // would later free incorrectly.
    if !h.hrecs.is_null() && sam_hdr_rebuild(h) < 0 {
        return -2;
    }

    let Some(line) = sam_hdr_text_find_line_id(h, type_.as_ptr(), id_key, id_val) else {
        return -1;
    };
    ks.data.clear();
    if kputsn(line, line.len(), ks) < 0 {
        return -2;
    }
    0
}

pub unsafe fn sam_hdr_find_line_pos(
    h: &mut sam_hdr_t,
    type_: &[u8],
    pos: i32,
    ks: &mut kstring_t,
) -> i32 {
    // hrecs-backed: sync text from hrecs, then use the text lookup (see
    // sam_hdr_find_line_id for why we do not delegate to hts_sys).
    if !h.hrecs.is_null() && sam_hdr_rebuild(h) < 0 {
        return -2;
    }

    let Some(line) = sam_hdr_text_find_line_pos(h, type_.as_ptr(), pos) else {
        return -1;
    };
    ks.data.clear();
    if kputsn(line, line.len(), ks) < 0 {
        return -2;
    }
    0
}

pub unsafe fn sam_hdr_remove_line_id(
    h: &mut sam_hdr_t,
    type_: &[u8],
    id: Option<(&[u8], &[u8])>,
) -> i32 {
    if !h.hrecs.is_null() {
        // Adapt to the pointer-based callee: NUL-terminate type_ and the id
        // key/value (callee reads them via CStr::from_ptr).
        let type_c = [type_, &[0u8]].concat();
        let (key_c, value_c) = match id {
            Some((key, value)) => (
                Some([key, &[0u8]].concat()),
                Some([value, &[0u8]].concat()),
            ),
            None => (None, None),
        };
        return header_c_1784_sam_hdr_remove_line_id_hrecs(
            h,
            type_c.as_ptr(),
            key_c.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
            value_c.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
        );
    }
    if type_ == b"PG" {
        return -1;
    }
    let Some((id_key, id_value)) = id else {
        return -1;
    };
    if h.text.is_null() || h.l_text == 0 {
        return 0;
    }

    if type_.len() < 2 {
        return -1;
    }
    let type0 = type_[0];
    let type1 = type_[1];
    sam_hdr_text_remove_line(h, type0, type1, 0, |line, _seen| {
        sam_hdr_text_find_tag_value(line, id_key) == Some(id_value)
    })
}

pub unsafe fn sam_hdr_remove_line_pos(
    h: &mut sam_hdr_t,
    type_: &[u8],
    position: i32,
) -> i32 {
    if position < 0 {
        return -1;
    }
    if !h.hrecs.is_null() {
        let type_c = [type_, &[0u8]].concat();
        return header_c_1823_sam_hdr_remove_line_pos_hrecs(h, type_c.as_ptr(), position);
    }
    if type_ == b"PG" {
        return -1;
    }
    if h.text.is_null() || h.l_text == 0 {
        return -1;
    }

    if type_.len() < 2 {
        return -1;
    }
    let type0 = type_[0];
    let type1 = type_[1];
    sam_hdr_text_remove_line(h, type0, type1, -1, |_line, seen| seen == position)
}

pub unsafe fn sam_hdr_remove_except(
    h: &mut sam_hdr_t,
    type_: &[u8],
    id: Option<(&[u8], &[u8])>,
) -> i32 {
    if !h.hrecs.is_null() {
        let type_c = [type_, &[0u8]].concat();
        let (key_c, value_c) = match id {
            Some((key, value)) => (
                Some([key, &[0u8]].concat()),
                Some([value, &[0u8]].concat()),
            ),
            None => (None, None),
        };
        return header_c_2015_sam_hdr_remove_except_hrecs(
            h,
            type_c.as_ptr(),
            key_c.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
            value_c.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
        );
    }
    if type_.len() < 2 {
        return -1;
    }
    let type0 = type_[0];
    let type1 = type_[1];
    if (type0 == b'P' && type1 == b'G') || (type0 == b'C' && type1 == b'O') {
        return -1;
    }
    if h.text.is_null() || h.l_text == 0 {
        return 0;
    }

    let keep_id = id;
    let mut kept_exception = false;
    sam_hdr_text_remove_line(h, type0, type1, 0, |line, _seen| {
        if let Some((key, value)) = keep_id {
            if !kept_exception && sam_hdr_text_find_tag_value(line, key) == Some(value) {
                kept_exception = true;
                return false;
            }
        }
        true
    })
}

pub unsafe fn sam_hdr_remove_lines(
    h: &mut sam_hdr_t,
    type_: &[u8],
    id: Option<&[u8]>,
    rh: Option<NonNull<()>>,
) -> i32 {
    if !h.hrecs.is_null() {
        let type_c = [type_, &[0u8]].concat();
        let id_c = id.map(|b| [b, &[0u8]].concat());
        return header_c_2071_sam_hdr_remove_lines_hrecs(
            h,
            type_c.as_ptr(),
            id_c.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
            rh.map_or(std::ptr::null_mut(), NonNull::as_ptr),
        );
    }
    if h.text.is_null() || h.l_text == 0 {
        return 0;
    }

    if type_.len() < 2 {
        return -1;
    }
    let type0 = type_[0];
    let type1 = type_[1];
    let id_key = id;
    sam_hdr_text_remove_line(h, type0, type1, 0, |line, _seen| {
        let Some(key) = id_key else {
            return true;
        };
        let keep = sam_hdr_text_find_tag_value(line, key).is_some_and(|value| {
            let Some(rh) = rh else {
                return false;
            };
            let value_c = [value, &[0u8]].concat();
            khash_str2int_has_key(rh.as_ptr(), value_c.as_ptr()) != 0
        });
        !keep
    })
}

pub unsafe fn sam_hdr_count_lines(h: &mut sam_hdr_t, type_: &[u8]) -> i32 {
    if type_.len() < 2 {
        return -1;
    }
    // hrecs-backed: sync text from hrecs, then count in the text (see
    // sam_hdr_find_line_id for why we do not delegate to hts_sys).
    if !h.hrecs.is_null() && sam_hdr_rebuild(h) < 0 {
        return -1;
    }
    if h.text.is_null() || h.l_text == 0 {
        return 0;
    }

    let type0 = type_[0];
    let type1 = type_[1];
    let text = std::slice::from_raw_parts(h.text, h.l_text);
    let mut count = 0;
    let mut start = 0usize;
    while start < text.len() {
        let rel_end = text[start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(text.len() - start);
        let line = &text[start..start + rel_end];
        if line.len() >= 3 && line[0] == b'@' && line[1] == type0 && line[2] == type1 {
            count += 1;
        }
        start += rel_end + usize::from(start + rel_end < text.len());
    }
    count
}

// original: sam_hdr_add_pg (htslib/header.c:2614)
//
// Pre-validates the supplied tag pairs (rejecting embedded tabs/colons/
// newlines), fills hrecs if not yet populated, then routes through
// sam_hdr_add_pg_hrecs. The pre-validation is kept (stricter than C) so
// callers see the same -1 they used to get from the dropped text-mode path.
pub unsafe fn sam_hdr_add_pg(
    h: &mut sam_hdr_t,
    name: &[u8],
    tags: &[(Option<&[u8]>, Option<&[u8]>)],
) -> i32 {
    for &(key, value) in tags {
        let (Some(key_b), Some(value_b)) = (key, value) else {
            return -1;
        };
        if key_b.is_empty()
            || key_b.contains(&b'\t')
            || key_b.contains(&b':')
            || value_b.contains(&b'\t')
            || value_b.contains(&b'\n')
        {
            return -1;
        }
    }
    if h.hrecs.is_null() && sam_hdr_fill_hrecs(h) < 0 {
        return -1;
    }
    // Adapt to the pointer-based callee: NUL-terminate name and each tag
    // key/value, then build a parallel pointer slice. Backing buffers are
    // collected first so reallocation cannot invalidate the pointers.
    let name_c = [name, &[0u8]].concat();
    let backing: Vec<(Option<Vec<u8>>, Option<Vec<u8>>)> = tags
        .iter()
        .map(|&(k, v)| {
            (
                k.map(|b| [b, &[0u8]].concat()),
                v.map(|b| [b, &[0u8]].concat()),
            )
        })
        .collect();
    let ptr_tags: Vec<(*const u8, *const u8)> = backing
        .iter()
        .map(|(k, v)| {
            (
                k.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
                v.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
            )
        })
        .collect();
    sam_hdr_add_pg_hrecs(h, name_c.as_ptr(), &ptr_tags)
}

pub unsafe fn sam_hdr_line_index(bh: &mut sam_hdr_t, type_: &[u8], key: &[u8]) -> i32 {
    if type_.len() < 2 {
        return -1;
    }
    // hrecs-backed: sync text from hrecs, then index in the text (see
    // sam_hdr_find_line_id for why we do not delegate to hts_sys).
    if !bh.hrecs.is_null() && sam_hdr_rebuild(bh) < 0 {
        return -1;
    }
    if bh.text.is_null() || bh.l_text == 0 {
        return -1;
    }

    let Some(name_key) = sam_hdr_text_name_key_for_type(type_.as_ptr()) else {
        return -1;
    };
    let needle = key;
    let type0 = type_[0];
    let type1 = type_[1];
    let text = std::slice::from_raw_parts(bh.text, bh.l_text);
    let mut seen = 0;
    let mut start = 0usize;
    while start < text.len() {
        let rel_end = text[start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(text.len() - start);
        let end = start + rel_end;
        let line = text[start..end]
            .strip_suffix(b"\r")
            .unwrap_or(&text[start..end]);
        if sam_hdr_text_line_has_type(line, type0, type1) {
            if sam_hdr_text_find_tag_value(line, name_key) == Some(needle) {
                return seen;
            }
            seen += 1;
        }
        start = end + usize::from(end < text.len());
    }
    -1
}

pub unsafe fn sam_hdr_line_name(
    bh: &mut sam_hdr_t,
    type_: &[u8],
    pos: i32,
) -> *const u8 {
    if pos < 0 {
        return std::ptr::null();
    }
    if type_.len() < 2 {
        return std::ptr::null();
    }
    // hrecs-backed: sync text from hrecs, then resolve from the text/targets
    // (see sam_hdr_find_line_id for why we do not delegate to hts_sys).
    if !bh.hrecs.is_null() && sam_hdr_rebuild(bh) < 0 {
        return std::ptr::null();
    }
    if type_ == b"SQ" && pos < bh.n_targets {
        return (*bh.target_name.add(pos as usize)).cast::<u8>().cast_const();
    }

    let Some(name_key) = sam_hdr_text_name_key_for_type(type_.as_ptr()) else {
        return std::ptr::null();
    };
    let Some(line) = sam_hdr_text_find_line_pos(bh, type_.as_ptr(), pos) else {
        return std::ptr::null();
    };
    let Some(value) = sam_hdr_text_find_tag_value(line, name_key) else {
        return std::ptr::null();
    };
    let mut scratch = match sam_hdr_text_scratch().lock() {
        Ok(guard) => guard,
        Err(_) => return std::ptr::null(),
    };
    let buf = scratch
        .entry((bh as *mut sam_hdr_t).cast_const() as usize)
        .or_default();
    buf.clear();
    buf.extend_from_slice(value);
    buf.push(0);
    buf.as_ptr().cast()
}

pub unsafe fn sam_hdr_find_tag_id(
    h: &mut sam_hdr_t,
    type_: &[u8],
    id: Option<(&[u8], &[u8])>,
    key: &[u8],
    ks: &mut kstring_t,
) -> i32 {
    // hrecs-backed: sync text from hrecs, then use the text lookup (see
    // sam_hdr_find_line_id for why we do not delegate to hts_sys).
    if !h.hrecs.is_null() && sam_hdr_rebuild(h) < 0 {
        return -2;
    }

    let line = if let Some((id_key, id_value)) = id {
        sam_hdr_text_find_line_id(h, type_.as_ptr(), id_key, id_value)
    } else {
        sam_hdr_text_find_line_pos(h, type_.as_ptr(), 0)
    };
    let Some(line) = line else {
        return -1;
    };
    let Some(value) = sam_hdr_text_find_tag_value(line, key) else {
        return -1;
    };
    ks.data.clear();
    if kputsn(value, value.len(), ks) < 0 {
        return -2;
    }
    0
}

pub unsafe fn sam_hdr_find_tag_pos(
    h: &mut sam_hdr_t,
    type_: &[u8],
    pos: i32,
    key: &[u8],
    ks: &mut kstring_t,
) -> i32 {
    // hrecs-backed: sync text from hrecs, then use the text lookup (see
    // sam_hdr_find_line_id for why we do not delegate to hts_sys).
    if !h.hrecs.is_null() && sam_hdr_rebuild(h) < 0 {
        return -2;
    }

    let Some(line) = sam_hdr_text_find_line_pos(h, type_.as_ptr(), pos) else {
        return -1;
    };
    let Some(value) = sam_hdr_text_find_tag_value(line, key) else {
        return -1;
    };
    ks.data.clear();
    if kputsn(value, value.len(), ks) < 0 {
        return -2;
    }
    0
}

pub unsafe fn sam_hdr_remove_tag_id(
    h: &mut sam_hdr_t,
    type_: &[u8],
    id: Option<(&[u8], &[u8])>,
    key: &[u8],
) -> i32 {
    if type_.len() < 2 {
        return -1;
    }
    if !h.hrecs.is_null() {
        // The hrecs path mirrors htslib's `sam_hdr_remove_tag_id`, which
        // returns 1 when a tag was actually removed and 0 when there was no
        // matching tag. Normalize to our public contract (matching the
        // retired text-mode entrypoint): 0 on real removal, -1 on
        // not-found/error, so callers and tests see a single behaviour.
        let type_c = [type_, &[0u8]].concat();
        let key_c = [key, &[0u8]].concat();
        let id_key_c = id.map(|(k, _)| [k, &[0u8]].concat());
        let id_value_c = id.map(|(_, v)| [v, &[0u8]].concat());
        let ret = header_c_2346_sam_hdr_remove_tag_id_hrecs(
            h,
            type_c.as_ptr(),
            id_key_c.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
            id_value_c.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
            key_c.as_ptr(),
        );
        return if ret > 0 { 0 } else { -1 };
    }
    if h.text.is_null() || h.l_text == 0 {
        return -1;
    }

    let type0 = type_[0];
    let type1 = type_[1];
    sam_hdr_text_remove_tag_id(h, type0, type1, id, key)
}

pub unsafe fn sam_hdr_pg_id(h: &mut sam_hdr_t, name: &[u8]) -> *const u8 {
    if !h.hrecs.is_null() {
        // The pointer-based callee reads `name` via strlen/kh_get and may return
        // the same pointer it was given (unique-name case). Stage a
        // NUL-terminated copy in the per-header scratch buffer so the returned
        // pointer stays valid after this call (the C contract keeps the result
        // live until the next sam_hdr_pg_id on the same header).
        let mut scratch = match sam_hdr_text_scratch().lock() {
            Ok(guard) => guard,
            Err(_) => return std::ptr::null(),
        };
        let buf = scratch
            .entry((h as *mut sam_hdr_t).cast_const() as usize)
            .or_default();
        buf.clear();
        buf.extend_from_slice(name);
        buf.push(0);
        let name_ptr = buf.as_ptr();
        drop(scratch);
        return header_c_2562_sam_hdr_pg_id_hrecs(h, name_ptr);
    }

    if !sam_hdr_text_pg_id_exists(h, name) {
        return name.as_ptr();
    }

    let mut scratch = match sam_hdr_text_scratch().lock() {
        Ok(guard) => guard,
        Err(_) => return std::ptr::null(),
    };
    let buf = scratch
        .entry((h as *mut sam_hdr_t).cast_const() as usize)
        .or_default();
    for n in 1..=i32::MAX {
        buf.clear();
        buf.extend_from_slice(name);
        buf.push(b'.');
        buf.extend_from_slice(n.to_string().as_bytes());
        if !sam_hdr_text_pg_id_exists(h, buf) {
            buf.push(0);
            return buf.as_ptr();
        }
    }
    std::ptr::null()
}

pub unsafe fn sam_hdr_length(h: &mut sam_hdr_t) -> usize {
    if !h.hrecs.is_null() {
        return if sam_hdr_rebuild(h) == 0 {
            h.l_text
        } else {
            usize::MAX
        };
    }
    h.l_text
}

pub unsafe fn sam_hdr_str(h: &mut sam_hdr_t) -> *const u8 {
    if !h.hrecs.is_null() {
        // Production-built hrecs is always Rust-marked (sam_hdr_fill_hrecs
        // marks it; cram_dopen's libhts-built header is dup'd in sam_hdr_read
        // via sam_hdr_dup which rebuilds text-only, leaving hrecs null on
        // the returned copy). If a caller somehow constructed a header with
        // unmarked hrecs, fall back to returning the cached text — better
        // than calling libhts on an opaque hrecs we don't own.
        return if sam_hdr_has_rust_hrecs(h) {
            if sam_hdr_rebuild(h) == 0 {
                h.text.cast_const()
            } else {
                std::ptr::null()
            }
        } else {
            h.text.cast_const()
        };
    }
    h.text.cast_const()
}

pub unsafe fn sam_hdr_nref(h: &sam_hdr_t) -> i32 {
    if !h.hrecs.is_null() {
        return (*h.hrecs).nref;
    }
    h.n_targets
}

// original: sam_hrecs_new (htslib/header.c:2732)
pub unsafe fn sam_hrecs_new() -> *mut sam_hrecs_t {
    sam_hrecs_new_box().map_or(std::ptr::null_mut(), Box::into_raw)
}

pub unsafe fn sam_hrecs_new_box() -> Option<Box<sam_hrecs_t>> {
    let mut hrecs = Box::new(sam_hrecs_t {
        h: std::ptr::null_mut(),
        first_line: std::ptr::null_mut(),
        str_pool: std::ptr::null_mut(),
        type_pool: std::ptr::null_mut(),
        tag_pool: std::ptr::null_mut(),
        nref: 0,
        ref_sz: 0,
        ref_: std::ptr::null_mut(),
        ref_hash: std::ptr::null_mut(),
        nrg: 0,
        rg_sz: 0,
        rg: std::ptr::null_mut(),
        rg_hash: std::ptr::null_mut(),
        npg: 0,
        pg_sz: 0,
        npg_end: 0,
        npg_end_alloc: 0,
        pg: std::ptr::null_mut(),
        pg_hash: std::ptr::null_mut(),
        pg_end: std::ptr::null_mut(),
        ID_buf: std::ptr::null_mut(),
        ID_buf_sz: 0,
        ID_cnt: 0,
        dirty: 0,
        refs_changed: 0,
        pgs_changed: 0,
        type_count: 0,
        type_order: std::ptr::null_mut(),
    });

    hrecs.ID_cnt = 1;
    hrecs.refs_changed = -1;
    hrecs.ref_hash = khash_str2int_init();
    hrecs.rg_hash = khash_str2int_init();
    hrecs.pg_hash = khash_str2int_init();
    let hrecs_ptr = (&mut *hrecs) as *mut sam_hrecs_t;
    if hrecs.ref_hash.is_null()
        || hrecs.rg_hash.is_null()
        || hrecs.pg_hash.is_null()
        || sam_hrecs_init_type_order(hrecs_ptr, std::ptr::null_mut()) < 0
    {
        sam_hrecs_free_box(hrecs);
        return None;
    }

    Some(hrecs)
}

// original: sam_hrecs_free (htslib/header.c:2812)
pub unsafe fn sam_hrecs_free(hrecs: *mut sam_hrecs_t) {
    if hrecs.is_null() {
        return;
    }
    sam_hrecs_free_box(Box::from_raw(hrecs));
}

pub unsafe fn sam_hrecs_free_box(mut hrecs: Box<sam_hrecs_t>) {
    sam_hrecs_free_contents(&mut hrecs);
}

pub unsafe fn sam_hrecs_free_contents(hrecs: &mut sam_hrecs_t) {
    while !hrecs.first_line.is_null() {
        let line = hrecs.first_line.cast::<sam_hrec_type_t>();
        let _ = sam_hrecs_remove_line(hrecs, line);
    }
    sam_hrecs_free_ref_altname_hash_keys(hrecs);
    khash_str2int_destroy(hrecs.ref_hash);
    sam_hrecs_free_ref_altname_hash_keys(hrecs);
    khash_str2int_destroy(hrecs.rg_hash);
    khash_str2int_destroy(hrecs.pg_hash);
    // ref_/rg/pg/pg_end/type_order/ID_buf are owned buffers that drop with the
    // struct; their previous C free() calls are removed.
}

// original: sam_hrecs_find_key (htslib/header.c:3009)
pub unsafe fn sam_hrecs_find_key(
    type_: &mut sam_hrec_type_t,
    key: &[u8],
) -> (
    Option<NonNull<sam_hrec_tag_t>>,
    Option<NonNull<sam_hrec_tag_t>>,
) {
    let mut previous = None;
    let mut tag = NonNull::new(type_.tag);
    while let Some(current) = tag {
        if sam_hrec_tag_matches_key(current.as_ptr(), key.as_ptr()) {
            return (Some(current), previous);
        }
        previous = Some(current);
        tag = NonNull::new((*current.as_ptr()).next);
    }
    (None, previous)
}

// original: sam_hrecs_find_type_id (htslib/header.c:2865)
pub unsafe fn sam_hrecs_find_type_id(
    hrecs: &mut sam_hrecs_t,
    type_: &[u8],
    id: Option<(&[u8], &[u8])>,
) -> Option<NonNull<sam_hrec_type_t>> {
    let type_key = header_h_58_TYPEKEY(type_.as_ptr());
    // NUL-terminated copy of id_value for the pointer-based hash/cstr callees
    // (kh_get_m_s2i and cstr_eq expect C strings).
    let id_value_c = id.map(|(_, v)| [v, &[0u8]].concat());
    let id_value_ptr = id_value_c
        .as_ref()
        .map_or(std::ptr::null(), |b| b.as_ptr());
    if let Some((id_key, _id_value)) = id {
        if type_[0] == b'S'
            && type_[1] == b'Q'
            && id_key[0] == b'S'
            && id_key[1] == b'N'
            && !hrecs.ref_hash.is_null()
        {
            let hash = hrecs.ref_hash.cast::<khash_m_s2i_t>();
            let k = kh_get_m_s2i(hash, id_value_ptr);
            if k != (*hash).n_buckets {
                let idx = *(*hash).vals.add(k as usize);
                if idx >= 0 && idx < hrecs.nref && !hrecs.ref_.is_null() {
                    return NonNull::new((*hrecs.ref_.add(idx as usize)).ty.cast());
                }
            }
        }
        if type_[0] == b'R'
            && type_[1] == b'G'
            && id_key[0] == b'I'
            && id_key[1] == b'D'
            && !hrecs.rg_hash.is_null()
        {
            let hash = hrecs.rg_hash.cast::<khash_m_s2i_t>();
            let k = kh_get_m_s2i(hash, id_value_ptr);
            if k != (*hash).n_buckets {
                let idx = *(*hash).vals.add(k as usize);
                if idx >= 0 && idx < hrecs.nrg && !hrecs.rg.is_null() {
                    return NonNull::new(
                        (*hrecs.rg.cast::<sam_hrec_rg_t>().add(idx as usize))
                            .ty
                            .cast(),
                    );
                }
            }
        }
        if type_[0] == b'P'
            && type_[1] == b'G'
            && id_key[0] == b'I'
            && id_key[1] == b'D'
            && !hrecs.pg_hash.is_null()
        {
            let hash = hrecs.pg_hash.cast::<khash_m_s2i_t>();
            let k = kh_get_m_s2i(hash, id_value_ptr);
            if k != (*hash).n_buckets {
                let idx = *(*hash).vals.add(k as usize);
                if idx >= 0 && idx < hrecs.npg && !hrecs.pg.is_null() {
                    return NonNull::new(
                        (*hrecs.pg.cast::<sam_hrec_pg_t>().add(idx as usize))
                            .ty
                            .cast(),
                    );
                }
            }
        }
    }

    let mut found = None;
    sam_hrecs_walk_global(hrecs, |line| {
        if (*line).type_ != type_key {
            return true;
        }
        let Some((id_key, _id_value)) = id else {
            found = NonNull::new(line);
            return false;
        };
        let (tag, _) = sam_hrecs_find_key(&mut *line, id_key);
        if let Some(tag) = tag {
            if !(*tag.as_ptr()).str_.is_null()
                && (*tag.as_ptr()).len >= 3
                && cstr_eq((*tag.as_ptr()).str_.add(3), id_value_ptr)
            {
                found = NonNull::new(line);
                return false;
            }
        }
        true
    });
    found
}

// original: sam_hrecs_remove_key (htslib/header.c:3030)
pub unsafe fn sam_hrecs_remove_key(
    hrecs: &mut sam_hrecs_t,
    type_: &mut sam_hrec_type_t,
    key: &[u8],
) -> i32 {
    let (tag, prev) = sam_hrecs_find_key(type_, key);
    let Some(tag) = tag else {
        return 0;
    };
    let tag_ptr = tag.as_ptr();

    // htslib/header.c:3042-3051: when removing an AN tag from an SQ line the
    // referenced alt-names must also be dropped from the global ref_hash so
    // that subsequent `sam_hdr_name2tid` lookups on those altnames return -1.
    // Faithful to v1.23.
    if type_.type_ == header_h_58_TYPEKEY(b"SQ".as_ptr())
        && !(*tag_ptr).str_.is_null()
        && (*tag_ptr).len >= 3
        && *(*tag_ptr).str_ as u8 == b'A'
        && *(*tag_ptr).str_.add(1) as u8 == b'N'
    {
        let (sn_tag, _) = sam_hrecs_find_key(type_, b"SN");
        if let Some(sn_tag) = sn_tag {
            if !(*sn_tag.as_ptr()).str_.is_null() && (*sn_tag.as_ptr()).len >= 3 {
                let ref_hash = hrecs.ref_hash.cast::<khash_m_s2i_t>();
                if !ref_hash.is_null() {
                    let k = kh_get_m_s2i(ref_hash, (*sn_tag.as_ptr()).str_.add(3));
                    if k != (*ref_hash).n_buckets {
                        let idx = *(*ref_hash).vals.add(k as usize);
                        sam_hrecs_remove_ref_altnames(hrecs, idx, (*tag_ptr).str_.add(3));
                    }
                }
            }
        }
    }

    if let Some(prev) = prev {
        (*prev.as_ptr()).next = (*tag_ptr).next;
    } else {
        type_.tag = (*tag_ptr).next;
    }
    hrecs.dirty = 1;
    1
}

// original: sam_hrecs_rebuild_text (htslib/header.c:2376)
pub unsafe fn sam_hrecs_rebuild_text(hrecs: &sam_hrecs_t, ks: &mut kstring_t) -> i32 {
    ks.data.clear();

    if hrecs.first_line.is_null() {
        return if kputsn(b"", 0, ks) >= 0 {
            0
        } else {
            -1
        };
    }

    let first = hrecs.first_line.cast::<sam_hrec_type_t>();
    let mut line = first;
    loop {
        if build_header_line(line, ks) < 0 || kputc(b'\n' as i32, ks) < 0 {
            return -1;
        }
        let next = (*line).global_next;
        if next.is_null() || next == first {
            break;
        }
        line = next;
    }
    0
}

// original: sam_hdr_rebuild (htslib/header.c:1604)
pub unsafe fn sam_hdr_rebuild(bh: &mut sam_hdr_t) -> i32 {
    let hrecs = bh.hrecs;
    if hrecs.is_null() {
        return if bh.text.is_null() || bh.l_text == 0 { -1 } else { 0 };
    }
    let hrecs_ref = &mut *hrecs;

    if hrecs_ref.refs_changed >= 0 && rebuild_target_arrays(bh) < 0 {
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            b"sam_hdr_rebuild",
            b"Header target array rebuild has failed",
        );
        return -1;
    }

    /* If header text wasn't changed or header is empty, don't rebuild it. */
    if hrecs_ref.dirty == 0 {
        return 0;
    }

    if hrecs_ref.pgs_changed != 0 && sam_hdr_link_pg(bh) < 0 {
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            b"sam_hdr_rebuild",
            b"Linking @PG lines has failed",
        );
        return -1;
    }

    let mut ks = kstring_t { data: Vec::new() };
    if sam_hrecs_rebuild_text(hrecs_ref, &mut ks) != 0 {
        ks_free(&mut ks);
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            b"sam_hdr_rebuild",
            b"Header text rebuild has failed",
        );
        return -1;
    }

    hrecs_ref.dirty = 0;

    /* Sync */
    // bh.text is a C-style NUL-terminated buffer owned as a forgotten Vec of
    // length l_text + 1 (matching sam.rs's text ownership convention). Free the
    // previous buffer, then take the rebuilt bytes, append the NUL terminator,
    // and store the raw pointer.
    let old_len = bh.l_text;
    if !bh.text.is_null() {
        drop(Vec::from_raw_parts(bh.text, old_len + 1, old_len + 1));
    }
    let mut text_vec = ks_release(&mut ks);
    bh.l_text = text_vec.len();
    text_vec.push(0);
    bh.text = text_vec.as_mut_ptr();
    std::mem::forget(text_vec);
    0
}

// original: sam_hdr_fill_hrecs (htslib/header.c:1623)
pub unsafe fn sam_hdr_fill_hrecs(bh: &mut sam_hdr_t) -> i32 {
    if !bh.hrecs.is_null() {
        return 0;
    }
    let Some(mut hrecs) = sam_hrecs_new_box() else {
        return -1;
    };
    hrecs.h = (bh as *mut sam_hdr_t).cast();
    let hrecs_ptr = (&mut *hrecs) as *mut sam_hrecs_t;
    let parse_ret = if !bh.text.is_null() && bh.l_text > 0 {
        sam_hrecs_parse_lines(hrecs_ptr, bh.text.cast_const(), bh.l_text)
    } else {
        0
    };
    if parse_ret < 0
        || (hrecs.first_line.is_null() && bh.n_targets > 0 && {
            bh.hrecs = hrecs_ptr;
            let ret = add_stub_ref_sq_lines(bh);
            bh.hrecs = std::ptr::null_mut();
            ret < 0
        })
        || sam_hrecs_update_hashes(hrecs_ptr) < 0
    {
        sam_hrecs_free_box(hrecs);
        return -1;
    }
    bh.hrecs = hrecs_ptr;
    if bh.n_targets == 0 && hrecs.nref > 0 && rebuild_target_arrays(bh) < 0 {
        bh.hrecs = std::ptr::null_mut();
        sam_hrecs_free_box(hrecs);
        return -1;
    }
    hrecs.dirty = 0;
    let _ = Box::into_raw(hrecs);
    // Mark the header as having a Rust-built hrecs. sam_hdr_write/_str check
    // this to decide whether to use our native rebuild path or delegate to
    // hts_sys (when hrecs was populated by a C-side mutator and our walkers
    // would mis-interpret C pool-allocated records).
    sam_hdr_mark_rust_hrecs(bh);
    0
}

// original: sam_hrecs_find_rg (htslib/header.c:2899)
pub unsafe fn sam_hrecs_find_rg(
    hrecs: &mut sam_hrecs_t,
    id: &[u8],
) -> Option<NonNull<sam_hrec_rg_t>> {
    if hrecs.rg_hash.is_null() {
        return None;
    }
    let hash = hrecs.rg_hash.cast::<khash_m_s2i_t>();
    let id_c = [id, &[0u8]].concat();
    let k = kh_get_m_s2i(hash, id_c.as_ptr());
    if k == (*hash).n_buckets {
        return None;
    }
    let idx = *(*hash).vals.add(k as usize);
    if idx < 0 || idx >= hrecs.nrg || hrecs.rg.is_null() {
        return None;
    }
    NonNull::new(hrecs.rg.cast::<sam_hrec_rg_t>().add(idx as usize))
}

// original: sam_hrecs_sort_order (htslib/header.c:3128)
pub unsafe fn sam_hrecs_sort_order(hrecs: &mut sam_hrecs_t) -> i32 {
    let hd = sam_hrecs_find_first_type(hrecs, header_h_58_TYPEKEY(b"HD".as_ptr()));
    let Some(hd) = hd.as_mut() else {
        return ORDER_UNSORTED;
    };

    match sam_hrec_find_tag_value(hd, b'S', b'O') {
        Some(b"unsorted") => ORDER_UNSORTED,
        Some(b"queryname") => ORDER_NAME,
        Some(b"coordinate") => ORDER_COORD,
        Some(b"unknown") => ORDER_UNKNOWN,
        Some(_) => ORDER_UNKNOWN,
        None => ORDER_UNSORTED,
    }
}

// original: sam_hrecs_group_order (htslib/header.c:3154)
pub unsafe fn sam_hrecs_group_order(hrecs: &mut sam_hrecs_t) -> i32 {
    let hd = sam_hrecs_find_first_type(hrecs, header_h_58_TYPEKEY(b"HD".as_ptr()));
    let Some(hd) = hd.as_mut() else {
        return ORDER_GO_NONE;
    };

    match sam_hrec_find_tag_value(hd, b'G', b'O') {
        Some(b"query") => ORDER_GO_QUERY,
        Some(b"reference") => ORDER_GO_REFERENCE,
        Some(_) => ORDER_GO_UNKNOWN,
        None => ORDER_GO_NONE,
    }
}

// original: known_stderr (htslib/header.c:780)
pub unsafe fn header_c_780_known_stderr(tool: &[u8], advice: &[u8]) {
    let tool_s = String::from_utf8_lossy(tool);
    // hts_log_cstr now takes &[u8] context/message slices; pass the bytes
    // directly. Trim any trailing NUL on advice (callers pass NUL-terminated
    // literals) so the logged text is clean.
    let msg = format!("SAM file corrupted by embedded {tool_s} error/log message").into_bytes();
    let advice = advice.strip_suffix(b"\0").unwrap_or(advice);
    crate::htslib_rs::hts::hts_log_cstr(
        crate::htslib_rs::hts::HTS_LOG_WARNING,
        b"known_stderr",
        &msg,
    );
    crate::htslib_rs::hts::hts_log_cstr(
        crate::htslib_rs::hts::HTS_LOG_WARNING,
        b"known_stderr",
        advice,
    );
}

// original: warn_if_known_stderr (htslib/header.c:788)
pub unsafe fn header_c_788_warn_if_known_stderr(line: &[u8]) {
    if line
        .windows(b"M::bwa_idx_load_from_disk".len())
        .any(|w| w == b"M::bwa_idx_load_from_disk")
    {
        header_c_780_known_stderr(
            b"bwa",
            b"Use `bwa mem -o file.sam ...` or `bwa sampe -f file.sam ...` instead of `bwa ... > file.sam`\0",
        );
    } else if line
        .windows(b"M::mem_pestat".len())
        .any(|w| w == b"M::mem_pestat")
    {
        header_c_780_known_stderr(
            b"bwa",
            b"Use `bwa mem -o file.sam ...` instead of `bwa mem ... > file.sam`\0",
        );
    } else if line
        .windows(b"loaded/built the index".len())
        .any(|w| w == b"loaded/built the index")
    {
        header_c_780_known_stderr(
            b"minimap2",
            b"Use `minimap2 -o file.sam ...` instead of `minimap2 ... > file.sam`\0",
        );
    }
}

// original: valid_sam_header_type (htslib/header.c:1325)
pub unsafe fn valid_sam_header_type(s: &[u8]) -> i32 {
    if s.len() < 4 || s[0] != b'@' {
        return 0;
    }
    match s[1] {
        b'H' => (s[2] == b'D' && s[3] == b'\t') as i32,
        b'S' => (s[2] == b'Q' && s[3] == b'\t') as i32,
        b'R' | b'P' => (s[2] == b'G' && s[3] == b'\t') as i32,
        b'C' => (s[2] == b'O') as i32,
        _ => 0,
    }
}

// original: redact_header_text (htslib/header.c:1530)
pub unsafe fn redact_header_text(bh: &mut sam_hdr_t) {
    // bh.text is a forgotten Vec of length l_text + 1; free it (matching sam.rs's
    // text ownership convention) and null the pointer before resetting l_text.
    if !bh.text.is_null() {
        drop(Vec::from_raw_parts(bh.text, bh.l_text + 1, bh.l_text + 1));
        bh.text = std::ptr::null_mut();
    }
    bh.l_text = 0;
}

pub unsafe fn sam_hdr_incr_ref(bh: &mut sam_hdr_t) {
    bh.ref_count = bh.ref_count.wrapping_add(1);
}

pub unsafe fn sam_hdr_name2tid(h: &mut sam_hdr_t, ref_: &[u8]) -> i32 {
    // NUL-terminated copy of ref_ for the pointer-based hash/text callees
    // (kh_get_m_s2i and sam_hdr_text_name2tid expect C strings).
    let ref_c = [ref_, &[0u8]].concat();
    let ref_ptr = ref_c.as_ptr();
    if !h.hrecs.is_null() {
        let hrecs = h.hrecs;
        let ref_hash = (*hrecs).ref_hash.cast::<khash_m_s2i_t>();
        if ref_hash.is_null() {
            return -1;
        }
        let k = kh_get_m_s2i(ref_hash, ref_ptr);
        return if k == (*ref_hash).n_buckets {
            -1
        } else {
            *(*ref_hash).vals.add(k as usize)
        };
    }
    if h.target_name.is_null()
        && h.n_targets == 0
        && h.hrecs.is_null()
        && sam_hdr_fill_targets_from_text(h) < 0
    {
        return -2;
    }
    if !h.target_name.is_null() {
        for tid in 0..h.n_targets {
            let name = *h.target_name.add(tid as usize);
            // target_name entries are NUL-terminated C strings; compare against
            // the byte needle without allocating: every needle byte must match
            // and the next byte in name must be the terminating NUL.
            if !name.is_null()
                && (0..ref_.len()).all(|i| *name.add(i) as u8 == ref_[i])
                && *name.add(ref_.len()) as u8 == 0
            {
                return tid;
            }
        }
    }
    if !h.text.is_null() && h.l_text != 0 {
        let tid = sam_hdr_text_name2tid(h, ref_ptr);
        if tid >= 0 {
            return tid;
        }
    }
    let hrecs = h.hrecs;
    if hrecs.is_null() {
        return -1;
    }
    let ref_hash = (*hrecs).ref_hash.cast::<khash_m_s2i_t>();
    if ref_hash.is_null() {
        -1
    } else {
        let k = kh_get_m_s2i(ref_hash, ref_ptr);
        if k == (*ref_hash).n_buckets {
            -1
        } else {
            *(*ref_hash).vals.add(k as usize)
        }
    }
}

pub unsafe fn sam_hdr_tid2len(h: &sam_hdr_t, tid: i32) -> hts_pos_t {
    if tid < 0 {
        return 0;
    }
    let hrecs = h.hrecs;
    if !hrecs.is_null() && tid < (*hrecs).nref {
        return (*(*hrecs).ref_.add(tid as usize)).len;
    }
    if tid < h.n_targets {
        let len = *h.target_len.add(tid as usize);
        if len < u32::MAX || h.sdict.is_null() {
            return len as hts_pos_t;
        }
        let long_refs = h.sdict.cast::<khash_s2i_t>();
        let k = kh_get_s2i(long_refs, *h.target_name.add(tid as usize));
        return if k == (*long_refs).n_buckets {
            u32::MAX as hts_pos_t
        } else {
            *(*long_refs).vals.add(k as usize) as hts_pos_t
        };
    }
    0
}

pub unsafe fn sam_hdr_tid2name(h: &sam_hdr_t, tid: i32) -> *const u8 {
    if tid < 0 {
        return std::ptr::null();
    }
    let hrecs = h.hrecs;
    if !hrecs.is_null() && tid < (*hrecs).nref {
        return (*(*hrecs).ref_.add(tid as usize)).name.cast::<u8>();
    }
    if tid < h.n_targets {
        return (*h.target_name.add(tid as usize)).cast::<u8>().cast_const();
    }
    std::ptr::null()
}
