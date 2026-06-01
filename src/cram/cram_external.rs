// Functions translated from htslib/cram/cram_external.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int, c_uchar, c_uint, c_ulong, c_void, CStr};

use super::*;

pub unsafe fn cram_cram_external_c_500_cram_slice_hdr_get_num_blocks(
    hdr: *mut cram_block_slice_hdr,
) -> i32 {
    (*(hdr.cast::<cram_block_slice_hdr_layout>())).num_blocks
}

pub unsafe fn cram_cram_external_c_504_cram_slice_hdr_get_embed_ref_id(
    h: *mut cram_block_slice_hdr,
) -> c_int {
    (*(h.cast::<cram_block_slice_hdr_layout>())).ref_base_id
}

pub unsafe fn cram_cram_external_c_508_cram_slice_hdr_get_coords(
    h: *mut cram_block_slice_hdr,
    refid: *mut c_int,
    start: *mut crate::htslib_rs::hts::hts_pos_t,
    span: *mut crate::htslib_rs::hts::hts_pos_t,
) {
    let h = h.cast::<cram_block_slice_hdr_layout>();
    if !refid.is_null() {
        *refid = (*h).ref_seq_id;
    }
    if !start.is_null() {
        *start = (*h).ref_seq_start;
    }
    if !span.is_null() {
        *span = (*h).ref_seq_span;
    }
}

pub unsafe fn cram_cram_external_c_529_cram_block_get_size(b: *mut cram_block) -> i32 {
    (*(b.cast::<cram_block_layout>())).byte as i32
}

pub unsafe fn cram_cram_external_c_530_cram_block_get_method(
    b: *mut cram_block,
) -> cram_block_method {
    (*(b.cast::<cram_block_layout>())).orig_method
}

pub unsafe fn cram_cram_external_c_542_cram_block_set_size(b: *mut cram_block, size: i32) {
    (*(b.cast::<cram_block_layout>())).byte = size as usize;
}

pub unsafe fn cram_cram_external_c_58_cram_fd_get_header(
    fd: *mut crate::htslib_rs::hts::cram_fd,
) -> *mut crate::htslib_rs::sam::sam_hdr_t {
    (*fd.cast::<cram_fd_layout>()).header
}

pub unsafe fn cram_cram_external_c_59_cram_fd_set_header(
    fd: *mut crate::htslib_rs::hts::cram_fd,
    hdr: *mut crate::htslib_rs::sam::sam_hdr_t,
) {
    (*fd.cast::<cram_fd_layout>()).header = hdr;
}

pub unsafe fn cram_cram_external_c_61_cram_fd_get_version(fd: *mut crate::htslib_rs::hts::cram_fd) -> c_int {
    (*fd.cast::<cram_fd_layout>()).version
}

pub unsafe fn cram_cram_external_c_62_cram_fd_set_version(fd: *mut crate::htslib_rs::hts::cram_fd, vers: c_int) {
    (*fd.cast::<cram_fd_layout>()).version = vers;
}

pub unsafe fn cram_cram_external_c_64_cram_major_vers(fd: *mut crate::htslib_rs::hts::cram_fd) -> c_int {
    // CRAM_MAJOR_VERS(v) = (v) >> 8
    (*fd.cast::<cram_fd_layout>()).version >> 8
}

pub unsafe fn cram_cram_external_c_65_cram_minor_vers(fd: *mut crate::htslib_rs::hts::cram_fd) -> c_int {
    // CRAM_MINOR_VERS(v) = (v) & 0xff
    (*fd.cast::<cram_fd_layout>()).version & 0xff
}

pub unsafe fn cram_cram_external_c_67_cram_fd_get_fp(
    fd: *mut crate::htslib_rs::hts::cram_fd,
) -> *mut crate::htslib_rs::hts::hFILE {
    (*fd.cast::<cram_fd_layout>()).fp.cast()
}

pub unsafe fn cram_cram_external_c_68_cram_fd_set_fp(
    fd: *mut crate::htslib_rs::hts::cram_fd,
    fp: *mut crate::htslib_rs::hts::hFILE,
) {
    (*fd.cast::<cram_fd_layout>()).fp = fp.cast();
}

pub unsafe fn cram_cram_external_c_75_cram_container_get_length(
    c: *mut cram_container,
) -> i32 {
    (*c.cast::<cram_container_layout>()).length
}

pub unsafe fn cram_cram_external_c_79_cram_container_set_length(
    c: *mut cram_container,
    length: i32,
) {
    (*c.cast::<cram_container_layout>()).length = length;
}

pub unsafe fn cram_cram_external_c_84_cram_container_get_num_blocks(
    c: *mut cram_container,
) -> i32 {
    (*c.cast::<cram_container_layout>()).num_blocks
}

pub unsafe fn cram_cram_external_c_88_cram_container_set_num_blocks(
    c: *mut cram_container,
    num_blocks: i32,
) {
    (*c.cast::<cram_container_layout>()).num_blocks = num_blocks;
}

pub unsafe fn cram_cram_external_c_92_cram_container_get_num_records(
    c: *mut cram_container,
) -> i32 {
    (*c.cast::<cram_container_layout>()).num_records
}

pub unsafe fn cram_cram_external_c_96_cram_container_get_num_bases(
    c: *mut cram_container,
) -> i64 {
    (*c.cast::<cram_container_layout>()).num_bases
}

pub unsafe fn cram_cram_external_c_104_cram_container_get_landmarks(
    c: *mut cram_container,
    num_landmarks: *mut i32,
) -> *mut i32 {
    let c = c.cast::<cram_container_layout>();
    *num_landmarks = (*c).num_landmarks;
    (*c).landmark
}

pub unsafe fn cram_cram_external_c_112_cram_container_set_landmarks(
    c: *mut cram_container,
    num_landmarks: i32,
    landmarks: *mut i32,
) {
    let c = c.cast::<cram_container_layout>();
    (*c).num_landmarks = num_landmarks;
    (*c).landmark = landmarks;
}

pub unsafe fn cram_cram_external_c_120_cram_container_is_empty(fd: *mut crate::htslib_rs::hts::cram_fd) -> c_int {
    (*fd.cast::<cram_fd_layout>()).empty_container
}

pub unsafe fn cram_cram_external_c_124_cram_container_get_coords(
    c: *mut cram_container,
    refid: *mut c_int,
    start: *mut i64,
    span: *mut i64,
) {
    let c = c.cast::<cram_container_layout>();
    if !refid.is_null() {
        *refid = (*c).ref_seq_id;
    }
    if !start.is_null() {
        *start = (*c).ref_seq_start;
    }
    if !span.is_null() {
        *span = (*c).ref_seq_span;
    }
}

pub unsafe fn cram_cram_external_c_152_cram_block_compression_hdr_set_DS(
    ch: *mut c_void,
    ds: c_int,
    new_rg: c_int,
) -> c_int {
    if ch.is_null() {
        return -1;
    }
    let ch = ch.cast::<cram_block_compression_hdr_layout>();
    if (*ch).codecs[ds as usize].is_null() {
        return -1;
    }

    let co = (*ch).codecs[ds as usize];
    match *(co.cast::<c_int>()) {
        3 => {
            let co = co.cast::<cram_codec_huffman_layout>();
            if (*co).huffman.ncodes != 1 {
                return -1;
            }
            (*(*co).huffman.codes).symbol = new_rg as i64;
            0
        }
        6 => {
            let co = co.cast::<cram_codec_beta_layout>();
            if (*co).beta.nbits != 0 {
                return -1;
            }
            (*co).beta.offset = -new_rg;
            0
        }
        _ => -1,
    }
}

pub unsafe fn cram_cram_external_c_177_cram_block_compression_hdr_set_rg(
    ch: *mut c_void,
    new_rg: c_int,
) -> c_int {
    cram_cram_external_c_152_cram_block_compression_hdr_set_DS(ch, 17, new_rg)
}

pub unsafe fn cram_cram_external_c_189_cram_block_compression_hdr_decoder2encoder(
    fd: *mut c_void,
    ch: *mut c_void,
) -> c_int {
    if ch.is_null() {
        return -1;
    }
    let ch = ch.cast::<cram_block_compression_hdr_layout>();
    for i in 0..46usize {
        let co = (*ch).codecs[i];
        if co.is_null() {
            continue;
        }
        if cram_cram_codecs_c_4031_cram_codec_decoder2encoder(fd, co) == -1 {
            return -1;
        }
    }
    0
}

pub unsafe fn cram_cram_external_c_215_cram_codec_iter_init(hdr: *mut c_void, iter: *mut c_void) {
    let iter = iter.cast::<cram_codec_iter_layout>();
    (*iter).hdr = hdr.cast::<cram_block_compression_hdr_layout>();
    (*iter).curr_map = std::ptr::null_mut();
    (*iter).idx = 0;
    (*iter).is_tag = 0;
}

pub fn cram_cram_external_c_224_cram_ds_to_key(ds: c_int) -> c_int {
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

pub unsafe fn cram_cram_external_c_264_cram_codec_iter_next(
    iter: *mut c_void,
    key: *mut c_int,
) -> *mut c_void {
    let iter = iter.cast::<cram_codec_iter_layout>();
    let hdr = (*iter).hdr;

    if (*iter).is_tag == 0 {
        let mut cc;
        loop {
            cc = (*hdr).codecs[(*iter).idx as usize];
            (*iter).idx += 1;
            if !cc.is_null() || (*iter).idx >= 46 {
                break;
            }
        }
        if !cc.is_null() {
            *key = cram_cram_external_c_224_cram_ds_to_key((*iter).idx - 1);
            return cc;
        }

        (*iter).idx = 0;
        (*iter).is_tag = 1;
    }

    loop {
        if (*iter).curr_map.is_null() {
            (*iter).curr_map =
                (*hdr).tag_encoding_map[(*iter).idx as usize].cast::<cram_map_layout>();
            (*iter).idx += 1;
        }

        let cc = if !(*iter).curr_map.is_null() {
            (*(*iter).curr_map).codec
        } else {
            std::ptr::null_mut()
        };
        if !cc.is_null() {
            *key = (*(*iter).curr_map).key;
            (*iter).curr_map = (*(*iter).curr_map).next;
            return cc;
        }
        if (*iter).idx >= 32 {
            break;
        }
    }

    std::ptr::null_mut()
}

pub unsafe fn cram_cram_external_c_320_cram_cid2ds_free(cid2ds: *mut cram_cid2ds_t) {
    if !cid2ds.is_null() {
        drop(Box::from_raw(cid2ds));
    }
}

pub unsafe fn cram_cram_external_c_342_cram_update_cid2ds_map(
    hdr: *mut cram_block_compression_hdr,
    cid2ds: *mut cram_cid2ds_t,
) -> *mut cram_cid2ds_t {
    let c2d = if cid2ds.is_null() {
        Box::into_raw(Box::new(cram_cid2ds_t {
            ds: Vec::new(),
            hash: HashMap::new(),
            ds_a: Vec::new(),
        }))
    } else {
        cid2ds
    };

    let mut citer = cram_codec_iter_layout {
        hdr: std::ptr::null_mut(),
        curr_map: std::ptr::null_mut(),
        idx: 0,
        is_tag: 0,
    };
    cram_cram_external_c_215_cram_codec_iter_init(
        hdr.cast(),
        (&mut citer as *mut cram_codec_iter_layout).cast(),
    );

    let mut key = 0;
    loop {
        let codec = cram_cram_external_c_264_cram_codec_iter_next(
            (&mut citer as *mut cram_codec_iter_layout).cast(),
            &mut key,
        );
        if codec.is_null() {
            break;
        }

        let mut bnum = [-2; 2];
        cram_cram_external_c_665_cram_codec_get_content_ids(codec, bnum.as_mut_ptr());
        for block_id in bnum {
            if block_id <= -2 {
                continue;
            }

            let c2d_ref = &mut *c2d;
            if let Some(head_ref) = c2d_ref.hash.get_mut(&block_id) {
                let mut dsi = *head_ref;
                while dsi >= 0 {
                    let ds = c2d_ref.ds[dsi as usize];
                    if ds.data_series == key {
                        break;
                    }
                    dsi = ds.next;
                }

                if dsi == -1 {
                    let new_idx = c2d_ref.ds.len() as c_int;
                    c2d_ref.ds.push(cram_ds_list {
                        data_series: key,
                        next: *head_ref,
                    });
                    *head_ref = new_idx;
                }
            } else {
                let new_idx = c2d_ref.ds.len() as c_int;
                c2d_ref.ds.push(cram_ds_list {
                    data_series: key,
                    next: -1,
                });
                c2d_ref.hash.insert(block_id, new_idx);
            }
        }
    }

    c2d
}

pub unsafe fn cram_cram_external_c_443_cram_cid2ds_query(
    c2d: *mut cram_cid2ds_t,
    content_id: c_int,
    n: *mut c_int,
) -> *mut c_int {
    *n = 0;
    if c2d.is_null() {
        return std::ptr::null_mut();
    }

    let c2d = &mut *c2d;
    let Some(mut dsi) = c2d.hash.get(&content_id).copied() else {
        return std::ptr::null_mut();
    };

    c2d.ds_a.clear();
    while dsi >= 0 {
        let ds = c2d.ds[dsi as usize];
        c2d.ds_a.push(ds.data_series);
        dsi = ds.next;
    }

    *n = c2d.ds_a.len() as c_int;
    c2d.ds_a.as_mut_ptr()
}

pub unsafe fn cram_cram_external_c_476_cram_describe_encodings(
    hdr: *mut cram_block_compression_hdr,
    ks: *mut kstring_t,
) -> c_int {
    let mut citer = cram_codec_iter_layout {
        hdr: std::ptr::null_mut(),
        curr_map: std::ptr::null_mut(),
        idx: 0,
        is_tag: 0,
    };
    cram_cram_external_c_215_cram_codec_iter_init(
        hdr.cast(),
        (&mut citer as *mut cram_codec_iter_layout).cast(),
    );

    let mut r = 0;
    let mut key = 0;
    loop {
        let codec = cram_cram_external_c_264_cram_codec_iter_next(
            (&mut citer as *mut cram_codec_iter_layout).cast(),
            &mut key,
        );
        if codec.is_null() {
            break;
        }

        let mut key_s = [0 as c_char; 4];
        let mut key_i = 0usize;
        if (key >> 16) != 0 {
            key_s[key_i] = (key >> 16) as c_char;
            key_i += 1;
        }
        key_s[key_i] = ((key >> 8) & 0xff) as c_char;
        key_i += 1;
        key_s[key_i] = (key & 0xff) as c_char;
        key_i += 1;

        r |= (kputc(b'\t' as c_int, ks) < 0) as c_int;
        r |= (kputsn(key_s.as_ptr(), key_i, ks) < 0) as c_int;
        r |= (kputc(b'\t' as c_int, ks) < 0) as c_int;
        r |= (cram_cram_codecs_c_4185_cram_codec_describe(codec, ks) < 0) as c_int;
        r |= (kputc(b'\n' as c_int, ks) < 0) as c_int;
    }

    if r != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_external_c_522_cram_block_get_content_id(
    b: *mut cram_block,
) -> i32 {
    let b = b.cast::<cram_block_layout>();
    if (*b).content_type == crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE {
        -1
    } else {
        (*b).content_id
    }
}

pub unsafe fn cram_cram_external_c_525_cram_block_get_comp_size(
    b: *mut cram_block,
) -> i32 {
    (*b.cast::<cram_block_layout>()).comp_size
}

pub unsafe fn cram_cram_external_c_526_cram_block_get_uncomp_size(
    b: *mut cram_block,
) -> i32 {
    (*b.cast::<cram_block_layout>()).uncomp_size
}

pub unsafe fn cram_cram_external_c_527_cram_block_get_crc32(b: *mut cram_block) -> i32 {
    (*b.cast::<cram_block_layout>()).crc32 as i32
}

pub unsafe fn cram_cram_external_c_528_cram_block_get_data(
    b: *mut cram_block,
) -> *mut c_void {
    (*b.cast::<cram_block_layout>()).data.cast()
}

pub unsafe fn cram_cram_external_c_533_cram_block_get_content_type(
    b: *mut cram_block,
) -> cram_content_type {
    (*b.cast::<cram_block_layout>()).content_type
}

pub unsafe fn cram_cram_external_c_537_cram_block_set_content_id(
    b: *mut cram_block,
    id: i32,
) {
    (*b.cast::<cram_block_layout>()).content_id = id;
}

pub unsafe fn cram_cram_external_c_538_cram_block_set_comp_size(
    b: *mut cram_block,
    size: i32,
) {
    (*b.cast::<cram_block_layout>()).comp_size = size;
}

pub unsafe fn cram_cram_external_c_539_cram_block_set_uncomp_size(
    b: *mut cram_block,
    size: i32,
) {
    (*b.cast::<cram_block_layout>()).uncomp_size = size;
}

pub unsafe fn cram_cram_external_c_540_cram_block_set_crc32(b: *mut cram_block, crc: i32) {
    (*b.cast::<cram_block_layout>()).crc32 = crc as u32;
}

pub unsafe fn cram_cram_external_c_541_cram_block_set_data(
    b: *mut cram_block,
    data: *mut c_void,
) {
    (*b.cast::<cram_block_layout>()).data = data.cast();
}

pub unsafe fn cram_cram_external_c_544_cram_block_append(
    b: *mut cram_block,
    data: *const c_void,
    size: c_int,
) -> c_int {
    cram_cram_io_h_248_block_append(b, data, size as usize)
}

pub unsafe fn cram_cram_external_c_551_cram_block_update_size(b: *mut cram_block) {
    let b = b.cast::<cram_block_layout>();
    (*b).comp_size = (*b).byte as i32;
    (*b).uncomp_size = (*b).byte as i32;
}

pub unsafe fn cram_cram_external_c_554_cram_block_get_offset(b: *mut cram_block) -> u64 {
    (*b.cast::<cram_block_layout>()).byte as u64
}

pub unsafe fn cram_cram_external_c_555_cram_block_set_offset(
    b: *mut cram_block,
    offset: u64,
) {
    (*b.cast::<cram_block_layout>()).byte = offset as usize;
}

pub unsafe fn cram_cram_external_c_568_cram_expand_method(
    data: *mut u8,
    size: i32,
    mut comp: cram_block_method,
) -> *mut cram_method_details {
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

    let cm =
        calloc(1, std::mem::size_of::<cram_method_details>() as u64).cast::<cram_method_details>();
    if cm.is_null() {
        return std::ptr::null_mut();
    }

    if comp == CRAM_COMP_UNKNOWN {
        if size > 1 && *data == 0x1f && *data.add(1) == 0x8b {
            comp = CRAM_COMP_GZIP;
        } else if size > 3 && *data.add(1) == b'B' && *data.add(2) == b'Z' && *data.add(3) == b'h' {
            comp = CRAM_COMP_BZIP2;
        } else if size > 6
            && *data == 0xfd
            && *data.add(1) == b'7'
            && *data.add(2) == b'z'
            && *data.add(3) == b'X'
            && *data.add(4) == b'Z'
            && *data.add(5) == 0
        {
            comp = CRAM_COMP_LZMA;
        } else {
            comp = CRAM_COMP_UNKNOWN;
        }
    }
    (*cm).method = comp;

    match comp {
        CRAM_COMP_GZIP => {
            if size > 8 {
                (*cm).level = match *data.add(8) {
                    4 => 1,
                    2 => 9,
                    _ => 5,
                };
            }
        }
        CRAM_COMP_BZIP2 => {
            if size > 3 && *data.add(3) >= b'1' && *data.add(3) <= b'9' {
                (*cm).level = (*data.add(3) - b'0') as c_int;
            }
        }
        CRAM_COMP_RANS4X8 => {
            (*cm).nway = 4;
            (*cm).order = if size > 0 && *data == 1 { 1 } else { 0 };
        }
        CRAM_COMP_RANSNX16 => {
            if size > 0 {
                let flags = *data;
                (*cm).order = (flags & 1) as c_int;
                (*cm).nway = if flags & RANS_ORDER_X32 != 0 { 32 } else { 4 };
                (*cm).rle = (flags & RANS_ORDER_RLE != 0) as c_int;
                (*cm).pack = (flags & RANS_ORDER_PACK != 0) as c_int;
                (*cm).cat = (flags & RANS_ORDER_CAT != 0) as c_int;
                (*cm).stripe = (flags & RANS_ORDER_STRIPE != 0) as c_int;
                (*cm).nosz = (flags & RANS_ORDER_NOSZ != 0) as c_int;
            }
        }
        CRAM_COMP_ARITH => {
            if size > 0 {
                let flags = *data;
                (*cm).order = (flags & 3) as c_int;
                (*cm).rle = (flags & RANS_ORDER_RLE != 0) as c_int;
                (*cm).pack = (flags & RANS_ORDER_PACK != 0) as c_int;
                (*cm).cat = (flags & RANS_ORDER_CAT != 0) as c_int;
                (*cm).stripe = (flags & RANS_ORDER_STRIPE != 0) as c_int;
                (*cm).nosz = (flags & RANS_ORDER_NOSZ != 0) as c_int;
                (*cm).ext = (flags & 4 != 0) as c_int;
            }
        }
        CRAM_COMP_TOK3 => {
            if size > 8 {
                (*cm).level = match *data.add(8) {
                    1 => 11,
                    0 => 1,
                    _ => (*cm).level,
                };
            }
        }
        _ => {}
    }

    cm
}

pub unsafe fn cram_cram_external_c_665_cram_codec_get_content_ids(c: *mut c_void, ids: *mut c_int) {
    *ids = cram_cram_codecs_c_3968_cram_codec_to_id(c, ids.add(1));
}

pub unsafe fn cram_cram_external_c_683_cram_copy_slice(
    in_: *mut cram_fd,
    out: *mut cram_fd,
    num_slice: i32,
) -> c_int {
    for _ in 0..num_slice {
        let mut blk = cram_read_block(in_);
        if blk.is_null() {
            return -1;
        }

        let hdr = cram_decode_slice_header(in_, blk);
        if hdr.is_null() {
            cram_cram_io_c_1565_cram_free_block(blk);
            return -1;
        }

        if cram_write_block(out, blk) != 0 {
            cram_cram_io_c_1565_cram_free_block(blk);
            return -1;
        }
        cram_cram_io_c_1565_cram_free_block(blk);

        let num_blocks = cram_cram_external_c_500_cram_slice_hdr_get_num_blocks(hdr);
        for _ in 0..num_blocks {
            blk = cram_read_block(in_);
            if blk.is_null() || cram_write_block(out, blk) != 0 {
                if !blk.is_null() {
                    cram_cram_io_c_1565_cram_free_block(blk);
                }
                return -1;
            }
            cram_cram_io_c_1565_cram_free_block(blk);
        }
        cram_free_slice_header(hdr);
    }

    0
}

pub unsafe fn cram_cram_external_c_725_cram_skip_container(
    in_: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    let mut blk = cram_read_block(in_);
    if blk.is_null() {
        return -1;
    }
    cram_cram_io_c_1565_cram_free_block(blk);

    let c = c.cast::<cram_container_layout>();
    for _ in 0..(*c).num_landmarks {
        blk = cram_read_block(in_);
        if blk.is_null() {
            return -1;
        }
        let hdr = cram_decode_slice_header(in_, blk);
        if hdr.is_null() {
            cram_cram_io_c_1565_cram_free_block(blk);
            return -1;
        }
        cram_cram_io_c_1565_cram_free_block(blk);

        let num_blocks = cram_cram_external_c_500_cram_slice_hdr_get_num_blocks(hdr);
        for _ in 0..num_blocks {
            blk = cram_read_block(in_);
            if blk.is_null() {
                cram_free_slice_header(hdr);
                return -1;
            }
            cram_cram_io_c_1565_cram_free_block(blk);
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

    let in_fd = in_.cast::<cram_fd_layout>();
    let mut c_ptr = c;
    let c_layout = c.cast::<cram_container_layout>();
    let mut err = 0;
    let mut fixed_ref = -3;

    if !ref_id.is_null() {
        *ref_id = (*c_layout).ref_seq_id;
    }

    let rid = if (*in_fd).range.refid == -2 {
        -1
    } else {
        (*in_fd).range.refid
    };
    if (rid != (*c_layout).ref_seq_id
        || (*in_fd).range.start > (*c_layout).ref_seq_start + (*c_layout).ref_seq_span - 1)
        && (*c_layout).ref_seq_id != -2
    {
        return cram_cram_external_c_725_cram_skip_container(in_, c);
    }

    let blk = cram_read_block(in_);
    if blk.is_null() {
        return -1;
    }
    (*c_layout).comp_hdr =
        cram_decode_compression_header(in_, blk).cast();
    (*in_fd).ctr = c_layout;

    if (*c_layout).ref_seq_id == -2 {
        let ch = (*c_layout).comp_hdr;
        let cd = (*ch).codecs[DS_RI];
        if !cd.is_null()
            && *(cd.cast::<c_int>()) == E_HUFFMAN
            && (*cd.cast::<cram_codec_huffman_layout>()).huffman.ncodes == 1
            && rid == (*(*cd.cast::<cram_codec_huffman_layout>()).huffman.codes).symbol as c_int
            && (*in_fd).range.start <= 1
            && (*in_fd).range.end >= (i64::MAX & ((0xffff_ffff_u64 << 32) as i64))
        {
            if !ref_id.is_null() {
                *ref_id = rid;
            }
            err |= (cram_write_container(out, c) < 0) as c_int;
            err |= cram_write_block(out, blk);
            return cram_cram_external_c_683_cram_copy_slice(in_, out, (*c_layout).num_landmarks) | -err;
        }
    }

    let rng_copy = (*in_fd).range;
    (*in_fd).range.start = i64::MIN;
    (*in_fd).range.end = i64::MAX;

    let mut b = crate::htslib_rs::sam::bam_init1();
    while (*c_layout).curr_slice < (*c_layout).max_slice
        || (!(*c_layout).slice.is_null()
            && (*(*c_layout).slice).curr_rec < (*(*c_layout).slice).max_rec)
    {
        let s = if !(*c_layout).slice.is_null()
            && (*(*c_layout).slice).curr_rec < (*(*c_layout).slice).max_rec
        {
            (*c_layout).slice
        } else if (*c_layout).curr_slice < (*c_layout).max_slice {
            decode_pipeline::cram_next_slice(
                in_.cast(),
                (&mut c_ptr as *mut *mut cram_container).cast(),
            )
            .cast()
        } else {
            break;
        };
        (*c_layout).slice = s;

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
            (*in_fd).header.cast(),
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

    if !ref_id.is_null() {
        *ref_id = fixed_ref;
    }

    (*in_fd).range = rng_copy;
    (*in_fd).ctr = std::ptr::null_mut();
    (*in_fd).ctr_mt = std::ptr::null_mut();

    err |= cram_cram_io_c_5446_cram_flush(out);
    cram_cram_io_c_1565_cram_free_block(blk);

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
    let in_fd = in_.cast::<cram_fd_layout>();
    let new_rg = *out_rg;

    if nrg != 1 {
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"cram_transcode_rg".as_ptr(),
            c"CRAM transcode supports only a single RG".as_ptr(),
        );
        return -2;
    }

    let o_blk = cram_read_block(in_);
    let old_size = cram_block_size(o_blk) as c_int;
    let ch = cram_decode_compression_header(in_, o_blk);
    if cram_cram_external_c_177_cram_block_compression_hdr_set_rg(ch.cast(), new_rg) != 0 {
        return -1;
    }
    if cram_cram_external_c_189_cram_block_compression_hdr_decoder2encoder(in_.cast(), ch.cast()) != 0 {
        return -1;
    }
    let n_blk = cram_cram_encode_c_2810_cram_encode_compression_header(in_, c, ch, (*in_fd).embed_ref);
    cram_free_compression_header(ch);

    let mut cp = cram_cram_external_c_528_cram_block_get_data(o_blk).cast::<c_char>();
    let mut op = cp;
    let endp = cp.add(cram_cram_external_c_526_cram_block_get_uncomp_size(o_blk) as usize);
    let mut err = 0;
    let varint_get32 = (*in_fd)
        .vv
        .varint_get32
        .expect("cram_fd varint_get32 is NULL");

    let mut i32_ = varint_get32(&mut cp, endp, &mut err) as i32;
    cp = cp.add(i32_ as usize);
    i32_ = varint_get32(&mut cp, endp, &mut err) as i32;
    cp = cp.add(i32_ as usize);
    op = cp;
    i32_ = varint_get32(&mut cp, endp, &mut err) as i32;
    i32_ += cp.offset_from(op) as i32;
    if err != 0 {
        return -2;
    }

    cram_cram_external_c_542_cram_block_set_size(n_blk, cram_cram_external_c_529_cram_block_get_size(n_blk) - 2);
    cram_cram_external_c_544_cram_block_append(n_blk, op.cast(), i32_);
    cram_cram_external_c_551_cram_block_update_size(n_blk);

    let new_size = cram_block_size(n_blk) as c_int;

    let mut num_landmarks = 0;
    let landmarks = cram_cram_external_c_104_cram_container_get_landmarks(c, &mut num_landmarks);

    if old_size != new_size {
        let diff = new_size - old_size;

        for j in 0..num_landmarks {
            *landmarks.add(j as usize) += diff;
        }
        cram_cram_external_c_79_cram_container_set_length(
            c,
            cram_cram_external_c_75_cram_container_get_length(c) + diff,
        );
    }

    if cram_write_container(out, c) != 0 {
        return -2;
    }

    cram_write_block(out, n_blk);
    cram_cram_io_c_1565_cram_free_block(o_blk);
    cram_cram_io_c_1565_cram_free_block(n_blk);

    cram_cram_external_c_683_cram_copy_slice(in_, out, num_landmarks)
}

pub unsafe fn cram_cram_external_c_1029_cram_get_refs(fd: *mut htsFile) -> *mut refs_t {
    if (*fd).format.format == HTS_FORMAT_CRAM {
        (*(*fd).fp.cram.cast::<cram_fd_layout>()).refs.cast()
    } else {
        std::ptr::null_mut()
    }
}

