// Functions translated from htslib/cram/cram_external.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int, c_void};
use std::ptr::NonNull;

use super::*;

unsafe fn raw_const<'a, T>(ptr: *const T) -> Option<&'a T> {
    unsafe { ptr.as_ref() }
}

unsafe fn raw_mut<'a, T>(ptr: *mut T) -> Option<&'a mut T> {
    unsafe { ptr.as_mut() }
}

unsafe fn raw_slice<'a, T>(ptr: *const T, len: usize) -> Option<&'a [T]> {
    if len == 0 {
        Some(&[])
    } else if ptr.is_null() {
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts(ptr, len) })
    }
}

unsafe fn raw_slice_mut<'a, T>(ptr: *mut T, len: usize) -> Option<&'a mut [T]> {
    if len == 0 {
        Some(&mut [])
    } else if ptr.is_null() {
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts_mut(ptr, len) })
    }
}

fn cid2ds_new_box() -> Box<cram_cid2ds_t> {
    Box::new(cram_cid2ds_t {
        ds: Vec::new(),
        hash: HashMap::new(),
        ds_a: Vec::new(),
    })
}

unsafe fn cid2ds_free_raw(cid2ds: *mut cram_cid2ds_t) {
    if !cid2ds.is_null() {
        drop(Box::from_raw(cid2ds));
    }
}

unsafe fn slice_hdr_layout(hdr: &cram_block_slice_hdr) -> &cram_block_slice_hdr_layout {
    unsafe {
        raw_const((hdr as *const cram_block_slice_hdr).cast::<cram_block_slice_hdr_layout>())
            .expect("reference-derived layout pointer is non-null")
    }
}

unsafe fn block_layout(b: &cram_block) -> &cram_block_layout {
    unsafe {
        raw_const((b as *const cram_block).cast::<cram_block_layout>())
            .expect("reference-derived layout pointer is non-null")
    }
}

unsafe fn block_layout_mut(b: &mut cram_block) -> &mut cram_block_layout {
    unsafe {
        raw_mut((b as *mut cram_block).cast::<cram_block_layout>())
            .expect("reference-derived layout pointer is non-null")
    }
}

unsafe fn container_layout(c: &cram_container) -> &cram_container_layout {
    unsafe {
        raw_const((c as *const cram_container).cast::<cram_container_layout>())
            .expect("reference-derived layout pointer is non-null")
    }
}

unsafe fn container_layout_mut(c: &mut cram_container) -> &mut cram_container_layout {
    unsafe {
        raw_mut((c as *mut cram_container).cast::<cram_container_layout>())
            .expect("reference-derived layout pointer is non-null")
    }
}

unsafe fn cram_fd_layout(fd: &crate::htslib_rs::hts::cram_fd) -> &cram_fd_layout {
    unsafe {
        raw_const((fd as *const crate::htslib_rs::hts::cram_fd).cast::<cram_fd_layout>())
            .expect("reference-derived layout pointer is non-null")
    }
}

unsafe fn cram_fd_layout_mut(fd: &mut crate::htslib_rs::hts::cram_fd) -> &mut cram_fd_layout {
    unsafe {
        raw_mut((fd as *mut crate::htslib_rs::hts::cram_fd).cast::<cram_fd_layout>())
            .expect("reference-derived layout pointer is non-null")
    }
}

pub fn cram_slice_hdr_get_num_blocks(hdr: &cram_block_slice_hdr) -> i32 {
    unsafe { slice_hdr_layout(hdr).num_blocks }
}

pub fn cram_slice_hdr_get_embed_ref_id(h: &cram_block_slice_hdr) -> c_int {
    unsafe { slice_hdr_layout(h).ref_base_id }
}

pub fn cram_slice_hdr_get_coords(
    h: &cram_block_slice_hdr,
    refid: Option<&mut c_int>,
    start: Option<&mut crate::htslib_rs::hts::hts_pos_t>,
    span: Option<&mut crate::htslib_rs::hts::hts_pos_t>,
) {
    let h = unsafe { slice_hdr_layout(h) };
    if let Some(refid) = refid {
        *refid = h.ref_seq_id;
    }
    if let Some(start) = start {
        *start = h.ref_seq_start;
    }
    if let Some(span) = span {
        *span = h.ref_seq_span;
    }
}

pub fn cram_block_get_size(b: &cram_block) -> i32 {
    unsafe { block_layout(b).byte as i32 }
}

pub fn cram_block_get_method(b: &cram_block) -> cram_block_method {
    unsafe { block_layout(b).orig_method }
}

pub fn cram_block_set_size(b: &mut cram_block, size: i32) {
    unsafe { block_layout_mut(b).byte = size as usize };
}

pub fn cram_fd_get_header(
    fd: &crate::htslib_rs::hts::cram_fd,
) -> Option<NonNull<crate::htslib_rs::sam::sam_hdr_t>> {
    unsafe { NonNull::new(cram_fd_layout(fd).header) }
}

pub fn cram_fd_set_header(
    fd: &mut crate::htslib_rs::hts::cram_fd,
    hdr: Option<NonNull<crate::htslib_rs::sam::sam_hdr_t>>,
) {
    unsafe { cram_fd_layout_mut(fd).header = hdr.map_or(std::ptr::null_mut(), NonNull::as_ptr) };
}

pub fn cram_fd_get_version(fd: &crate::htslib_rs::hts::cram_fd) -> c_int {
    unsafe { cram_fd_layout(fd).version }
}

pub fn cram_fd_set_version(fd: &mut crate::htslib_rs::hts::cram_fd, vers: c_int) {
    unsafe { cram_fd_layout_mut(fd).version = vers };
}

pub fn cram_major_vers(fd: &crate::htslib_rs::hts::cram_fd) -> c_int {
    cram_fd_get_version(fd) >> 8
}

pub fn cram_minor_vers(fd: &crate::htslib_rs::hts::cram_fd) -> c_int {
    cram_fd_get_version(fd) & 0xff
}

pub fn cram_fd_get_fp(
    fd: &crate::htslib_rs::hts::cram_fd,
) -> Option<NonNull<crate::htslib_rs::hts::hFILE>> {
    unsafe { NonNull::new(cram_fd_layout(fd).fp.cast()) }
}

pub fn cram_fd_set_fp(
    fd: &mut crate::htslib_rs::hts::cram_fd,
    fp: Option<NonNull<crate::htslib_rs::hts::hFILE>>,
) {
    unsafe { cram_fd_layout_mut(fd).fp = fp.map_or(std::ptr::null_mut(), NonNull::as_ptr).cast() };
}

pub fn cram_container_get_length(c: &cram_container) -> i32 {
    unsafe { container_layout(c).length }
}

pub fn cram_container_set_length(c: &mut cram_container, length: i32) {
    unsafe { container_layout_mut(c).length = length };
}

pub fn cram_container_get_num_blocks(c: &cram_container) -> i32 {
    unsafe { container_layout(c).num_blocks }
}

pub fn cram_container_set_num_blocks(c: &mut cram_container, num_blocks: i32) {
    unsafe { container_layout_mut(c).num_blocks = num_blocks };
}

pub fn cram_container_get_num_records(c: &cram_container) -> i32 {
    unsafe { container_layout(c).num_records }
}

pub fn cram_container_get_num_bases(c: &cram_container) -> i64 {
    unsafe { container_layout(c).num_bases }
}

pub fn cram_container_get_landmarks(c: &mut cram_container) -> &mut [i32] {
    let c = unsafe { container_layout_mut(c) };
    unsafe { raw_slice_mut(c.landmark, c.num_landmarks.max(0) as usize) }.unwrap_or(&mut [])
}

pub fn cram_container_set_landmarks(c: &mut cram_container, landmarks: &mut [i32]) {
    let c = unsafe { container_layout_mut(c) };
    let num_landmarks = landmarks.len() as i32;
    c.num_landmarks = num_landmarks;
    c.landmark = if landmarks.is_empty() {
        std::ptr::null_mut()
    } else {
        landmarks.as_mut_ptr()
    };
}

pub unsafe fn cram_cram_external_c_120_cram_container_is_empty(
    fd: *mut crate::htslib_rs::hts::cram_fd,
) -> c_int {
    unsafe { raw_const(fd) }.map_or(0, |fd| unsafe { cram_fd_layout(fd).empty_container })
}

pub fn cram_container_get_coords(
    c: &cram_container,
    refid: Option<&mut c_int>,
    start: Option<&mut i64>,
    span: Option<&mut i64>,
) {
    let c = unsafe { container_layout(c) };
    if let Some(refid) = refid {
        *refid = c.ref_seq_id;
    }
    if let Some(start) = start {
        *start = c.ref_seq_start;
    }
    if let Some(span) = span {
        *span = c.ref_seq_span;
    }
}

pub fn cram_block_compression_hdr_set_ds(
    ch: &mut cram_block_compression_hdr_layout,
    ds: usize,
    new_rg: c_int,
) -> c_int {
    let Some(&co) = ch.codecs.get(ds) else {
        return -1;
    };
    let Some(co) = NonNull::new(co) else {
        return -1;
    };

    match unsafe { *(co.as_ptr().cast::<c_int>()) } {
        3 => {
            let co = unsafe {
                raw_mut(co.as_ptr().cast::<cram_codec_huffman_layout>())
                    .expect("non-null codec pointer")
            };
            if co.huffman.ncodes != 1 {
                return -1;
            }
            let Some(mut code) = NonNull::new(co.huffman.codes) else {
                return -1;
            };
            unsafe { code.as_mut().symbol = new_rg as i64 };
            0
        }
        6 => {
            let co = unsafe {
                raw_mut(co.as_ptr().cast::<cram_codec_beta_layout>())
                    .expect("non-null codec pointer")
            };
            if co.beta.nbits != 0 {
                return -1;
            }
            co.beta.offset = -new_rg;
            0
        }
        _ => -1,
    }
}

pub unsafe fn cram_cram_external_c_177_cram_block_compression_hdr_set_rg(
    ch: *mut c_void,
    new_rg: c_int,
) -> c_int {
    let Some(ch) = (unsafe { raw_mut(ch.cast::<cram_block_compression_hdr_layout>()) }) else {
        return -1;
    };
    cram_block_compression_hdr_set_ds(ch, 17, new_rg)
}

pub fn cram_block_compression_hdr_decoder2encoder(
    fd: Option<NonNull<c_void>>,
    ch: &mut cram_block_compression_hdr_layout,
) -> c_int {
    let fd = fd.map_or(std::ptr::null_mut(), NonNull::as_ptr);
    for codec in ch.codecs.iter().take(46).filter_map(|&co| NonNull::new(co)) {
        if unsafe { cram_cram_codecs_c_4031_cram_codec_decoder2encoder(fd, codec.as_ptr()) } == -1 {
            return -1;
        }
    }
    0
}

pub fn cram_codec_iter_init(
    hdr: &mut cram_block_compression_hdr_layout,
    iter: &mut cram_codec_iter_layout,
) {
    iter.hdr = hdr;
    iter.curr_map = std::ptr::null_mut();
    iter.idx = 0;
    iter.is_tag = 0;
}

pub fn cram_ds_to_key(ds: c_int) -> c_int {
    match ds {
        10 => 256 * b'R' as c_int + b'N' as c_int,
        11 => 256 * b'Q' as c_int + b'S' as c_int,
        12 => 256 * b'I' as c_int + b'N' as c_int,
        13 => 256 * b'S' as c_int + b'C' as c_int,
        14 => 256 * b'B' as c_int + b'F' as c_int,
        15 => 256 * b'C' as c_int + b'F' as c_int,
        16 => 256 * b'A' as c_int + b'P' as c_int,
        17 => 256 * b'R' as c_int + b'G' as c_int,
        18 => 256 * b'M' as c_int + b'Q' as c_int,
        19 => 256 * b'N' as c_int + b'S' as c_int,
        20 => 256 * b'M' as c_int + b'F' as c_int,
        21 => 256 * b'T' as c_int + b'S' as c_int,
        22 => 256 * b'N' as c_int + b'P' as c_int,
        23 => 256 * b'N' as c_int + b'F' as c_int,
        24 => 256 * b'R' as c_int + b'L' as c_int,
        25 => 256 * b'F' as c_int + b'N' as c_int,
        26 => 256 * b'F' as c_int + b'C' as c_int,
        27 => 256 * b'F' as c_int + b'P' as c_int,
        28 => 256 * b'D' as c_int + b'L' as c_int,
        29 => 256 * b'B' as c_int + b'A' as c_int,
        30 => 256 * b'B' as c_int + b'S' as c_int,
        31 => 256 * b'T' as c_int + b'L' as c_int,
        32 => 256 * b'R' as c_int + b'I' as c_int,
        33 => 256 * b'R' as c_int + b'S' as c_int,
        34 => 256 * b'P' as c_int + b'D' as c_int,
        35 => 256 * b'H' as c_int + b'C' as c_int,
        36 => 256 * b'B' as c_int + b'B' as c_int,
        37 => 256 * b'Q' as c_int + b'Q' as c_int,
        38 => 256 * b'T' as c_int + b'N' as c_int,
        43 => 256 * b'T' as c_int + b'C' as c_int,
        44 => 256 * b'T' as c_int + b'M' as c_int,
        45 => 256 * b'T' as c_int + b'V' as c_int,
        _ => -1,
    }
}

struct CramCodecIter<'a> {
    hdr: &'a cram_block_compression_hdr_layout,
    curr_map: Option<NonNull<cram_map_layout>>,
    idx: c_int,
    is_tag: bool,
}

impl<'a> CramCodecIter<'a> {
    fn new(hdr: &'a cram_block_compression_hdr_layout) -> Self {
        Self {
            hdr,
            curr_map: None,
            idx: 0,
            is_tag: false,
        }
    }

    unsafe fn next(&mut self, key: &mut c_int) -> Option<NonNull<c_void>> {
        if !self.is_tag {
            while let Some(&cc) = self.hdr.codecs.get(self.idx as usize) {
                let ds = self.idx;
                self.idx += 1;
                if let Some(codec) = NonNull::new(cc) {
                    *key = cram_ds_to_key(ds);
                    return Some(codec.cast());
                }
            }

            self.idx = 0;
            self.is_tag = true;
        }

        loop {
            if self.curr_map.is_none() {
                let Some(&map) = self.hdr.tag_encoding_map.get(self.idx as usize) else {
                    break;
                };
                self.curr_map = NonNull::new(map.cast::<cram_map_layout>());
                self.idx += 1;
            }

            let cc = if let Some(curr_map) = self.curr_map {
                unsafe { (*curr_map.as_ptr()).codec }
            } else {
                std::ptr::null_mut()
            };
            if let Some(codec) = NonNull::new(cc) {
                let curr_map = self.curr_map.expect("non-null current map");
                *key = unsafe { (*curr_map.as_ptr()).key };
                self.curr_map = unsafe { NonNull::new((*curr_map.as_ptr()).next) };
                return Some(codec.cast());
            }
            if self.idx >= 32 {
                break;
            }
        }

        None
    }
}

pub fn cram_codec_iter_next(
    iter: &mut cram_codec_iter_layout,
    key: &mut c_int,
) -> Option<NonNull<c_void>> {
    let hdr = NonNull::new(iter.hdr)?;
    let hdr = unsafe { hdr.as_ref() };

    if iter.is_tag == 0 {
        while let Some(&cc) = hdr.codecs.get(iter.idx as usize) {
            let ds = iter.idx;
            iter.idx += 1;
            if let Some(codec) = NonNull::new(cc) {
                *key = cram_ds_to_key(ds);
                return Some(codec);
            }
        }

        iter.idx = 0;
        iter.is_tag = 1;
    }

    loop {
        if iter.curr_map.is_null() {
            let Some(&map) = hdr.tag_encoding_map.get(iter.idx as usize) else {
                break;
            };
            iter.curr_map = map.cast::<cram_map_layout>();
            iter.idx += 1;
        }

        let Some(curr_map) = NonNull::new(iter.curr_map) else {
            if iter.idx >= 32 {
                break;
            }
            continue;
        };
        let curr_map_ref = unsafe { curr_map.as_ref() };
        if let Some(codec) = NonNull::new(curr_map_ref.codec) {
            *key = curr_map_ref.key;
            iter.curr_map = curr_map_ref.next;
            return Some(codec);
        }
        if iter.idx >= 32 {
            break;
        }
    }

    None
}

pub unsafe fn cram_cram_external_c_320_cram_cid2ds_free(cid2ds: *mut cram_cid2ds_t) {
    cid2ds_free_raw(cid2ds);
}

unsafe fn cram_update_cid2ds_map(
    hdr: &cram_block_compression_hdr_layout,
    c2d: &mut cram_cid2ds_t,
) {
    let mut citer = CramCodecIter::new(hdr);
    let mut key = 0;
    while let Some(codec) = unsafe { citer.next(&mut key) } {
        let mut bnum = [-2; 2];
        unsafe { cram_codec_get_content_ids(codec.cast(), &mut bnum) };
        for block_id in bnum {
            if block_id <= -2 {
                continue;
            }

            if let Some(head_ref) = c2d.hash.get_mut(&block_id) {
                let mut dsi = *head_ref;
                while dsi >= 0 {
                    let ds = c2d.ds[dsi as usize];
                    if ds.data_series == key {
                        break;
                    }
                    dsi = ds.next;
                }

                if dsi == -1 {
                    let new_idx = c2d.ds.len() as c_int;
                    c2d.ds.push(cram_ds_list {
                        data_series: key,
                        next: *head_ref,
                    });
                    *head_ref = new_idx;
                }
            } else {
                let new_idx = c2d.ds.len() as c_int;
                c2d.ds.push(cram_ds_list {
                    data_series: key,
                    next: -1,
                });
                c2d.hash.insert(block_id, new_idx);
            }
        }
    }
}

pub unsafe fn cram_cram_external_c_342_cram_update_cid2ds_map(
    hdr: *mut cram_block_compression_hdr,
    cid2ds: *mut cram_cid2ds_t,
) -> *mut cram_cid2ds_t {
    let Some(hdr) = (unsafe { raw_const(hdr.cast::<cram_block_compression_hdr_layout>()) }) else {
        return cid2ds;
    };

    if let Some(mut c2d) = NonNull::new(cid2ds) {
        unsafe { cram_update_cid2ds_map(hdr, c2d.as_mut()) };
        cid2ds
    } else {
        let mut c2d = cid2ds_new_box();
        unsafe { cram_update_cid2ds_map(hdr, &mut c2d) };
        Box::into_raw(c2d)
    }
}

pub fn cram_cid2ds_query(
    c2d: Option<&mut cram_cid2ds_t>,
    content_id: c_int,
    n: &mut c_int,
) -> Option<NonNull<c_int>> {
    *n = 0;
    let Some(c2d) = c2d else {
        return None;
    };

    let Some(mut dsi) = c2d.hash.get(&content_id).copied() else {
        return None;
    };

    c2d.ds_a.clear();
    while dsi >= 0 {
        let ds = c2d.ds[dsi as usize];
        c2d.ds_a.push(ds.data_series);
        dsi = ds.next;
    }

    *n = c2d.ds_a.len() as c_int;
    NonNull::new(c2d.ds_a.as_mut_ptr())
}

pub unsafe fn cram_describe_encodings(
    hdr: &cram_block_compression_hdr_layout,
    ks: &mut kstring_t,
) -> c_int {
    let mut citer = CramCodecIter::new(hdr);
    let mut r = 0;
    let mut key = 0;
    while let Some(codec) = unsafe { citer.next(&mut key) } {
        let mut key_s: Vec<u8> = Vec::with_capacity(3);
        if (key >> 16) != 0 {
            key_s.push((key >> 16) as u8);
        }
        key_s.push(((key >> 8) & 0xff) as u8);
        key_s.push((key & 0xff) as u8);

        r |= (kputc(b'\t' as c_int, ks) < 0) as c_int;
        r |= (kputsn(&key_s, key_s.len(), ks) < 0) as c_int;
        r |= (kputc(b'\t' as c_int, ks) < 0) as c_int;
        r |= (unsafe {
            cram_cram_codecs_c_4185_cram_codec_describe(codec.as_ptr(), ks as *mut kstring_t)
        } < 0) as c_int;
        r |= (kputc(b'\n' as c_int, ks) < 0) as c_int;
    }

    if r != 0 {
        -1
    } else {
        0
    }
}

pub fn cram_block_get_content_id(b: &cram_block) -> i32 {
    let b = unsafe { block_layout(b) };
    if b.content_type == crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE {
        -1
    } else {
        b.content_id
    }
}

pub fn cram_block_get_comp_size(b: &cram_block) -> i32 {
    unsafe { block_layout(b).comp_size }
}

pub fn cram_block_get_uncomp_size(b: &cram_block) -> i32 {
    unsafe { block_layout(b).uncomp_size }
}

pub fn cram_block_get_crc32(b: &cram_block) -> i32 {
    unsafe { block_layout(b).crc32 as i32 }
}

pub fn cram_block_get_data(b: &cram_block) -> Option<NonNull<u8>> {
    unsafe { NonNull::new(block_layout(b).data.cast()) }
}

pub fn cram_block_data(b: &cram_block) -> Option<&[u8]> {
    let b = unsafe { block_layout(b) };
    unsafe { raw_slice(b.data.cast::<u8>().cast_const(), b.uncomp_size.max(0) as usize) }
}

pub fn cram_block_get_content_type(b: &cram_block) -> cram_content_type {
    unsafe { block_layout(b).content_type }
}

pub fn cram_block_set_content_id(b: &mut cram_block, id: i32) {
    unsafe { block_layout_mut(b).content_id = id };
}

pub fn cram_block_set_comp_size(b: &mut cram_block, size: i32) {
    unsafe { block_layout_mut(b).comp_size = size };
}

pub fn cram_block_set_uncomp_size(b: &mut cram_block, size: i32) {
    unsafe { block_layout_mut(b).uncomp_size = size };
}

pub fn cram_block_set_crc32(b: &mut cram_block, crc: i32) {
    unsafe { block_layout_mut(b).crc32 = crc as u32 };
}

pub fn cram_block_set_data(b: &mut cram_block, data: Option<NonNull<u8>>) {
    unsafe { block_layout_mut(b).data = data.map_or(std::ptr::null_mut(), NonNull::as_ptr).cast() };
}

pub fn cram_block_append(b: &mut cram_block, data: &[u8]) -> c_int {
    unsafe { cram_cram_io_h_248_block_append(b, data.as_ptr().cast(), data.len()) }
}

pub fn cram_block_update_size(b: &mut cram_block) {
    let b = unsafe { block_layout_mut(b) };
    b.comp_size = b.byte as i32;
    b.uncomp_size = b.byte as i32;
}

pub fn cram_block_get_offset(b: &cram_block) -> u64 {
    unsafe { block_layout(b).byte as u64 }
}

pub fn cram_block_set_offset(b: &mut cram_block, offset: u64) {
    unsafe { block_layout_mut(b).byte = offset as usize };
}

pub fn cram_expand_method(data: &[u8], mut comp: cram_block_method) -> Box<cram_method_details> {
    const CRAM_COMP_UNKNOWN: cram_block_method = -1;
    const CRAM_COMP_GZIP: cram_block_method = 1;
    const CRAM_COMP_BZIP2: cram_block_method = 2;
    const CRAM_COMP_LZMA: cram_block_method = 3;
    const CRAM_COMP_RANS4X8: cram_block_method = 4;
    const CRAM_COMP_RANSNX16: cram_block_method = 5;
    const CRAM_COMP_ARITH: cram_block_method = 6;
    const CRAM_COMP_TOK3: cram_block_method = 8;
    const RANS_ORDER_X32: u8 = 0x04;
    const RANS_ORDER_STRIPE: u8 = 0x08;
    const RANS_ORDER_NOSZ: u8 = 0x10;
    const RANS_ORDER_CAT: u8 = 0x20;
    const RANS_ORDER_RLE: u8 = 0x40;
    const RANS_ORDER_PACK: u8 = 0x80;

    let mut cm = Box::new(cram_method_details {
        method: 0,
        level: 0,
        order: 0,
        rle: 0,
        pack: 0,
        stripe: 0,
        cat: 0,
        nosz: 0,
        nway: 0,
        ext: 0,
    });

    if comp == CRAM_COMP_UNKNOWN {
        if data.len() > 1 && data[0] == 0x1f && data[1] == 0x8b {
            comp = CRAM_COMP_GZIP;
        } else if data.len() > 3 && data[1] == b'B' && data[2] == b'Z' && data[3] == b'h' {
            comp = CRAM_COMP_BZIP2;
        } else if data.len() > 6
            && data[0] == 0xfd
            && data[1] == b'7'
            && data[2] == b'z'
            && data[3] == b'X'
            && data[4] == b'Z'
            && data[5] == 0
        {
            comp = CRAM_COMP_LZMA;
        } else {
            comp = CRAM_COMP_UNKNOWN;
        }
    }
    cm.method = comp;

    match comp {
        CRAM_COMP_GZIP => {
            if data.len() > 8 {
                cm.level = match data[8] {
                    4 => 1,
                    2 => 9,
                    _ => 5,
                };
            }
        }
        CRAM_COMP_BZIP2 => {
            if data.len() > 3 && data[3] >= b'1' && data[3] <= b'9' {
                cm.level = (data[3] - b'0') as c_int;
            }
        }
        CRAM_COMP_RANS4X8 => {
            cm.nway = 4;
            cm.order = if !data.is_empty() && data[0] == 1 {
                1
            } else {
                0
            };
        }
        CRAM_COMP_RANSNX16 => {
            if !data.is_empty() {
                let flags = data[0];
                cm.order = (flags & 1) as c_int;
                cm.nway = if flags & RANS_ORDER_X32 != 0 { 32 } else { 4 };
                cm.rle = (flags & RANS_ORDER_RLE != 0) as c_int;
                cm.pack = (flags & RANS_ORDER_PACK != 0) as c_int;
                cm.cat = (flags & RANS_ORDER_CAT != 0) as c_int;
                cm.stripe = (flags & RANS_ORDER_STRIPE != 0) as c_int;
                cm.nosz = (flags & RANS_ORDER_NOSZ != 0) as c_int;
            }
        }
        CRAM_COMP_ARITH => {
            if !data.is_empty() {
                let flags = data[0];
                cm.order = (flags & 3) as c_int;
                cm.rle = (flags & RANS_ORDER_RLE != 0) as c_int;
                cm.pack = (flags & RANS_ORDER_PACK != 0) as c_int;
                cm.cat = (flags & RANS_ORDER_CAT != 0) as c_int;
                cm.stripe = (flags & RANS_ORDER_STRIPE != 0) as c_int;
                cm.nosz = (flags & RANS_ORDER_NOSZ != 0) as c_int;
                cm.ext = (flags & 4 != 0) as c_int;
            }
        }
        CRAM_COMP_TOK3 => {
            if data.len() > 8 {
                cm.level = match data[8] {
                    1 => 11,
                    0 => 1,
                    _ => cm.level,
                };
            }
        }
        _ => {}
    }

    cm
}

pub unsafe fn cram_codec_get_content_ids(c: NonNull<c_void>, ids: &mut [c_int; 2]) {
    ids[0] = unsafe { cram_cram_codecs_c_3968_cram_codec_to_id(c.as_ptr(), &mut ids[1]) };
}

pub unsafe fn cram_cram_external_c_683_cram_copy_slice(
    in_: *mut cram_fd,
    out: *mut cram_fd,
    num_slice: i32,
) -> c_int {
    if in_.is_null() || out.is_null() || num_slice < 0 {
        return -1;
    }

    for _ in 0..num_slice {
        let mut blk = cram_read_block(in_);
        if blk.is_null() {
            return -1;
        }

        let hdr = cram_decode_slice_header(in_, blk);
        if hdr.is_null() {
            cram_free_block(blk);
            return -1;
        }

        if cram_write_block(out, blk) != 0 {
            cram_free_slice_header(hdr);
            cram_free_block(blk);
            return -1;
        }
        cram_free_block(blk);

        let Some(hdr_ref) = (unsafe { raw_const(hdr) }) else {
            cram_free_slice_header(hdr);
            return -1;
        };
        let num_blocks = cram_slice_hdr_get_num_blocks(hdr_ref);
        for _ in 0..num_blocks {
            blk = cram_read_block(in_);
            if blk.is_null() || cram_write_block(out, blk) != 0 {
                if !blk.is_null() {
                    cram_free_block(blk);
                }
                return -1;
            }
            cram_free_block(blk);
        }
        cram_free_slice_header(hdr);
    }

    0
}

pub unsafe fn cram_cram_external_c_725_cram_skip_container(
    in_: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    if in_.is_null() {
        return -1;
    }
    let Some(c_ref) = (unsafe { raw_const(c) }) else {
        return -1;
    };
    let c_layout = unsafe { container_layout(c_ref) };

    let mut blk = cram_read_block(in_);
    if blk.is_null() {
        return -1;
    }
    cram_free_block(blk);

    for _ in 0..c_layout.num_landmarks {
        blk = cram_read_block(in_);
        if blk.is_null() {
            return -1;
        }
        let hdr = cram_decode_slice_header(in_, blk);
        if hdr.is_null() {
            cram_free_block(blk);
            return -1;
        }
        cram_free_block(blk);

        let Some(hdr_ref) = (unsafe { raw_const(hdr) }) else {
            cram_free_slice_header(hdr);
            return -1;
        };
        let num_blocks = cram_slice_hdr_get_num_blocks(hdr_ref);
        for _ in 0..num_blocks {
            blk = cram_read_block(in_);
            if blk.is_null() {
                cram_free_slice_header(hdr);
                return -1;
            }
            cram_free_block(blk);
        }
        cram_free_slice_header(hdr);
    }

    0
}

// original: cram_filter_container (htslib/cram/cram_external.c:776)
pub unsafe fn cram_cram_external_c_776_cram_filter_container(
    in_: *mut cram_fd,
    out: *mut cram_fd,
    c: *mut cram_container,
    ref_id: *mut c_int,
) -> c_int {
    const E_HUFFMAN: c_int = 3;
    const DS_RI: usize = 32;

    let (Some(in_fd), Some(c_ref)) = (unsafe { raw_mut(in_) }, unsafe { raw_mut(c) }) else {
        return -1;
    };
    let in_fd = unsafe { cram_fd_layout_mut(in_fd) };
    let c_layout = unsafe { container_layout_mut(c_ref) };
    let mut c_ptr = c;
    let mut err = 0;
    let mut fixed_ref = -3;

    if let Some(ref_id) = unsafe { ref_id.as_mut() } {
        *ref_id = c_layout.ref_seq_id;
    }

    let rid = if in_fd.range.refid == -2 {
        -1
    } else {
        in_fd.range.refid
    };
    if (rid != c_layout.ref_seq_id
        || in_fd.range.start > c_layout.ref_seq_start + c_layout.ref_seq_span - 1)
        && c_layout.ref_seq_id != -2
    {
        return cram_cram_external_c_725_cram_skip_container(in_, c);
    }

    let blk = cram_read_block(in_);
    if blk.is_null() {
        return -1;
    }
    c_layout.comp_hdr = cram_decode_compression_header(in_, blk).cast();
    in_fd.ctr = c_layout;

    if c_layout.ref_seq_id == -2 {
        let ch = c_layout.comp_hdr;
        let Some(ch) = (unsafe { raw_const(ch) }) else {
            cram_free_block(blk);
            return -1;
        };
        let cd = ch.codecs[DS_RI];
        if !cd.is_null()
            && *(cd.cast::<c_int>()) == E_HUFFMAN
            && (*cd.cast::<cram_codec_huffman_layout>()).huffman.ncodes == 1
            && NonNull::new((*cd.cast::<cram_codec_huffman_layout>()).huffman.codes)
                .is_some_and(|codes| rid == (*codes.as_ptr()).symbol as c_int)
            && in_fd.range.start <= 1
            && in_fd.range.end >= (i64::MAX & ((0xffff_ffff_u64 << 32) as i64))
        {
            if let Some(ref_id) = unsafe { ref_id.as_mut() } {
                *ref_id = rid;
            }
            err |= (cram_write_container(out, c) < 0) as c_int;
            err |= cram_write_block(out, blk);
            return cram_cram_external_c_683_cram_copy_slice(in_, out, c_layout.num_landmarks)
                | -err;
        }
    }

    let rng_copy = in_fd.range;
    in_fd.range.start = i64::MIN;
    in_fd.range.end = i64::MAX;

    let mut b = crate::htslib_rs::sam::bam_init1();
    while c_layout.curr_slice < c_layout.max_slice
        || (!c_layout.slice.is_null() && (*c_layout.slice).curr_rec < (*c_layout.slice).max_rec)
    {
        let s = if !c_layout.slice.is_null()
            && (*c_layout.slice).curr_rec < (*c_layout.slice).max_rec
        {
            c_layout.slice
        } else if c_layout.curr_slice < c_layout.max_slice {
            decode_pipeline::cram_next_slice(
                in_.cast(),
                (&mut c_ptr as *mut *mut cram_container).cast(),
            )
            .cast()
        } else {
            break;
        };
        c_layout.slice = s;

        let cr = (*s).crecs.add((*s).curr_rec as usize);
        if fixed_ref == -3 {
            fixed_ref = (*cr).ref_id;
        } else if fixed_ref != (*cr).ref_id {
            fixed_ref = -2;
        }

        if rng_copy.refid != (*cr).ref_id {
            if rng_copy.refid == -2 {
                if (*cr).ref_id > -1 {
                    (*s).curr_rec += 1;
                    continue;
                }
            } else if rng_copy.refid > (*cr).ref_id || rng_copy.refid == -1 {
                (*s).curr_rec += 1;
                continue;
            } else {
                break;
            }
        }

        if (*cr).aend < rng_copy.start {
            (*s).curr_rec += 1;
            continue;
        }
        if (*cr).apos > rng_copy.end {
            break;
        }

        err |= (decode_pipeline::cram_to_bam(
            in_fd.header.cast(),
            in_.cast(),
            s.cast(),
            cr.cast(),
            {
                let rec = (*s).curr_rec;
                (*s).curr_rec += 1;
                rec
            },
            (&mut b as *mut *mut bam1_t).cast(),
        ) < 0) as c_int;

        if cram_cram_encode_c_4049_cram_put_bam_seq(out, b) < 0 {
            err |= 1;
            break;
        }
    }
    bam_destroy1(b);

    if let Some(ref_id) = unsafe { ref_id.as_mut() } {
        *ref_id = fixed_ref;
    }

    in_fd.range = rng_copy;
    in_fd.ctr = std::ptr::null_mut();
    in_fd.ctr_mt = std::ptr::null_mut();

    err |= cram_cram_io_c_5446_cram_flush(out);
    cram_free_block(blk);

    -err
}

// original: cram_transcode_rg (htslib/cram/cram_external.c:934)
pub unsafe fn cram_cram_external_c_934_cram_transcode_rg(
    in_: *mut cram_fd,
    out: *mut cram_fd,
    c: *mut cram_container,
    nrg: c_int,
    _in_rg: *mut c_int,
    out_rg: *mut c_int,
) -> c_int {
    let (Some(in_fd), Some(new_rg)) = (unsafe { raw_const(in_) }, unsafe { raw_const(out_rg) })
    else {
        return -1;
    };
    let in_fd = unsafe { cram_fd_layout(in_fd) };
    let new_rg = *new_rg;

    if nrg != 1 {
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"cram_transcode_rg".as_ptr(),
            c"CRAM transcode supports only a single RG".as_ptr(),
        );
        return -2;
    }

    let o_blk = cram_read_block(in_);
    if o_blk.is_null() {
        return -1;
    }
    let old_size = cram_block_size(o_blk) as c_int;
    let ch = cram_decode_compression_header(in_, o_blk);
    if ch.is_null() {
        cram_free_block(o_blk);
        return -1;
    }
    if cram_cram_external_c_177_cram_block_compression_hdr_set_rg(ch.cast(), new_rg) != 0 {
        cram_free_compression_header(ch);
        cram_free_block(o_blk);
        return -1;
    }
    if {
        let fd = NonNull::new(in_.cast::<c_void>());
        match unsafe { raw_mut(ch.cast::<cram_block_compression_hdr_layout>()) } {
            Some(ch) => cram_block_compression_hdr_decoder2encoder(fd, ch),
            None => -1,
        }
    } != 0
    {
        cram_free_compression_header(ch);
        cram_free_block(o_blk);
        return -1;
    }
    let n_blk = cram_cram_encode_c_2810_cram_encode_compression_header(in_, c, ch, in_fd.embed_ref);
    cram_free_compression_header(ch);
    if n_blk.is_null() {
        cram_free_block(o_blk);
        return -1;
    }

    let Some(o_blk_ref) = (unsafe { raw_const(o_blk) }) else {
        cram_free_block(o_blk);
        cram_free_block(n_blk);
        return -1;
    };
    let Some(data) = cram_block_data(o_blk_ref) else {
        cram_free_block(o_blk);
        cram_free_block(n_blk);
        return -1;
    };
    let mut cp = data.as_ptr().cast_mut().cast::<c_char>();
    let endp = cp.add(data.len());
    let mut err = 0;
    let varint_get32 = in_fd.vv.varint_get32.expect("cram_fd varint_get32 is NULL");

    let mut i32_ = varint_get32(&mut cp, endp, &mut err) as i32;
    cp = cp.add(i32_ as usize);
    i32_ = varint_get32(&mut cp, endp, &mut err) as i32;
    cp = cp.add(i32_ as usize);
    let op = cp;
    i32_ = varint_get32(&mut cp, endp, &mut err) as i32;
    i32_ += cp.offset_from(op) as i32;
    if err != 0 {
        cram_free_block(o_blk);
        cram_free_block(n_blk);
        return -2;
    }

    let Some(n_blk_mut) = (unsafe { raw_mut(n_blk) }) else {
        cram_free_block(o_blk);
        cram_free_block(n_blk);
        return -1;
    };
    cram_block_set_size(n_blk_mut, cram_block_get_size(n_blk_mut) - 2);
    {
        let data = unsafe { raw_slice(op.cast::<u8>(), i32_.max(0) as usize) };
        if let Some(data) = data {
            cram_block_append(n_blk_mut, data);
        }
    }
    let n_blk_layout = unsafe { block_layout_mut(n_blk_mut) };
    n_blk_layout.comp_size = n_blk_layout.byte as i32;
    n_blk_layout.uncomp_size = n_blk_layout.byte as i32;

    let new_size = cram_block_size(n_blk) as c_int;

    let Some(c_ref) = (unsafe { raw_mut(c) }) else {
        cram_free_block(o_blk);
        cram_free_block(n_blk);
        return -1;
    };
    let landmarks = cram_container_get_landmarks(c_ref);
    let num_landmarks = landmarks.len() as c_int;

    if old_size != new_size {
        let diff = new_size - old_size;

        for landmark in landmarks.iter_mut() {
            *landmark += diff;
        }
        let c_layout = unsafe { container_layout_mut(c_ref) };
        c_layout.length += diff;
    }

    if cram_write_container(out, c) != 0 {
        return -2;
    }

    cram_write_block(out, n_blk);
    cram_free_block(o_blk);
    cram_free_block(n_blk);

    cram_cram_external_c_683_cram_copy_slice(in_, out, num_landmarks)
}

pub unsafe fn cram_cram_external_c_1029_cram_get_refs(fd: *mut htsFile) -> *mut refs_t {
    let Some(fd) = (unsafe { raw_const(fd) }) else {
        return std::ptr::null_mut();
    };
    if fd.format.format == HTS_FORMAT_CRAM {
        unsafe { raw_const(fd.fp.cram) }.map_or(std::ptr::null_mut(), |cram| unsafe {
            cram_fd_layout(cram).refs.cast()
        })
    } else {
        std::ptr::null_mut()
    }
}
