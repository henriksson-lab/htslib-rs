// Functions translated from htslib/cram/cram_encode.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

use super::*;

// (all extracted functions, preserving src/cram.rs order)

pub unsafe fn cram_cram_encode_c_70_sub_idx(key: *mut c_char, val: c_char) -> c_int {
    let mut i = 0;
    let mut keyp = key;
    while i < 4 {
        let c = *keyp;
        keyp = keyp.add(1);
        if c == val {
            break;
        }
        i += 1;
    }
    i
}

// Native translation of `cram_encode_compression_header` (htslib/cram/cram_encode.c:83).
// Byte-faithful with the c2rust mirror at src/cram/cram_encode.rs:9437 — every
// branch, value, and condition preserved. Supporting primitives (block helpers,
// itf8_put_blk, khash, sub_idx) are routed to the existing production natives.
pub unsafe fn cram_cram_encode_c_2810_cram_encode_compression_header(
    fd: *mut cram_fd,
    c: *mut cram_container,
    h: *mut cram_block_compression_hdr,
    embed_ref: c_int,
) -> *mut cram_block {
    // Data-series ids used below (htslib/cram/cram_structs.h cram_DS_ID).
    const DS_BF: usize = 15;
    const DS_CF: usize = 16;
    const DS_AP: usize = 17;
    const DS_RG: usize = 18;
    const DS_MQ: usize = 19;
    const DS_NS: usize = 20;
    const DS_MF: usize = 21;
    const DS_TS: usize = 22;
    const DS_NP: usize = 23;
    const DS_NF: usize = 24;
    const DS_RL: usize = 25;
    const DS_FN: usize = 26;
    const DS_FC: usize = 27;
    const DS_FP: usize = 28;
    const DS_DL: usize = 29;
    const DS_BA: usize = 30;
    const DS_BS: usize = 31;
    const DS_TL: usize = 32;
    const DS_RI: usize = 33;
    const DS_RS: usize = 34;
    const DS_PD: usize = 35;
    const DS_HC: usize = 36;
    const DS_BB: usize = 37;
    const DS_QQ: usize = 38;
    const DS_TC: usize = 44;
    const DS_TM: usize = 45;
    const DS_TV: usize = 46;
    const DS_RN_L: usize = 11;
    const DS_QS_L: usize = 12;
    const DS_IN_L: usize = 13;
    const DS_SC_L: usize = 14;
    const DS_TN_L: usize = 39;

    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let hl = h.cast::<cram_block_compression_hdr_layout>();

    let cb = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_COMPRESSION_HEADER, 0);
    let map = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_COMPRESSION_HEADER, 0);
    let mut mc: c_int;
    let mut r: c_int = 0;
    let no_ref: c_int = (*cl).no_ref;
    if cb.is_null() || map.is_null() {
        return std::ptr::null_mut::<cram_block>();
    }
    let cb_l = cb.cast::<cram_block_layout>();
    let map_l = map.cast::<cram_block_layout>();

    if (*fdl).version >> 8 == 1 {
        r |= cram_cram_io_c_620_itf8_put_blk(cb, (*hl).ref_seq_id);
        r |= cram_cram_io_c_620_itf8_put_blk(cb, (*hl).ref_seq_start as i32);
        r |= cram_cram_io_c_620_itf8_put_blk(cb, (*hl).ref_seq_span as i32);
        r |= cram_cram_io_c_620_itf8_put_blk(cb, (*hl).num_records);
        r |= cram_cram_io_c_620_itf8_put_blk(cb, (*hl).num_landmarks);
        let mut i: c_int = 0;
        while i < (*hl).num_landmarks {
            r |= cram_cram_io_c_620_itf8_put_blk(cb, *(*hl).landmark.offset(i as isize));
            i += 1;
        }
    }

    if !(*hl).preservation_map.is_null() {
        // Inline kh_destroy_map: free flags / keys / vals / struct.
        let pm = (*hl).preservation_map.cast::<kh_generic_layout>();
        free((*pm).flags.cast());
        free((*pm).keys.cast());
        free((*pm).vals.cast());
        free(pm.cast());
        (*hl).preservation_map = std::ptr::null_mut();
    }

    if (*cl).num_records > 0 {
        let mut k: u32;
        let mut r_0: c_int = 0;
        // Inline kh_init_map: zero-initialised kh_generic_layout.
        (*hl).preservation_map =
            calloc(1, std::mem::size_of::<kh_generic_layout>() as u64).cast::<c_void>();
        if (*hl).preservation_map.is_null() {
            return std::ptr::null_mut::<cram_block>();
        }
        let pm = (*hl).preservation_map.cast::<kh_generic_layout>();
        k = kh_put_map(pm, c"RN".as_ptr(), &raw mut r_0);
        if -1 == r_0 {
            return std::ptr::null_mut::<cram_block>();
        }
        (*(*pm).vals.cast::<pmap_val>().offset(k as isize)).i =
            ((*fdl).lossy_read_names == 0) as c_int;
        if (*fdl).version >> 8 == 1 {
            k = kh_put_map(pm, c"PI".as_ptr(), &raw mut r_0);
            if -1 == r_0 {
                return std::ptr::null_mut::<cram_block>();
            }
            (*(*pm).vals.cast::<pmap_val>().offset(k as isize)).i = 0;
            k = kh_put_map(pm, c"UI".as_ptr(), &raw mut r_0);
            if -1 == r_0 {
                return std::ptr::null_mut::<cram_block>();
            }
            (*(*pm).vals.cast::<pmap_val>().offset(k as isize)).i = 1;
            k = kh_put_map(pm, c"MI".as_ptr(), &raw mut r_0);
            if -1 == r_0 {
                return std::ptr::null_mut::<cram_block>();
            }
            (*(*pm).vals.cast::<pmap_val>().offset(k as isize)).i = 1;
        } else {
            k = kh_put_map(pm, c"SM".as_ptr(), &raw mut r_0);
            if -1 == r_0 {
                return std::ptr::null_mut::<cram_block>();
            }
            (*(*pm).vals.cast::<pmap_val>().offset(k as isize)).i = 0;
            k = kh_put_map(pm, c"TD".as_ptr(), &raw mut r_0);
            if -1 == r_0 {
                return std::ptr::null_mut::<cram_block>();
            }
            (*(*pm).vals.cast::<pmap_val>().offset(k as isize)).i = 0;
            k = kh_put_map(pm, c"AP".as_ptr(), &raw mut r_0);
            if -1 == r_0 {
                return std::ptr::null_mut::<cram_block>();
            }
            (*(*pm).vals.cast::<pmap_val>().offset(k as isize)).i = (*hl).ap_delta;
            if (*fdl).version >> 8 >= 4 {
                k = kh_put_map(pm, c"QO".as_ptr(), &raw mut r_0);
                if -1 == r_0 {
                    return std::ptr::null_mut::<cram_block>();
                }
                (*(*pm).vals.cast::<pmap_val>().offset(k as isize)).i = (*hl).qs_seq_orient;
            }
            if no_ref != 0 || embed_ref > 0 {
                k = kh_put_map(pm, c"RR".as_ptr(), &raw mut r_0);
                if -1 == r_0 {
                    return std::ptr::null_mut::<cram_block>();
                }
                (*(*pm).vals.cast::<pmap_val>().offset(k as isize)).i = 0;
            }
        }
    }

    mc = 0;
    (*map_l).byte = 0;
    let mut early_fail: bool = false;
    if !(*hl).preservation_map.is_null() {
        let pm = (*hl).preservation_map.cast::<kh_generic_layout>();
        let mut k_0: u32 = 0;
        while k_0 != (*pm).n_buckets {
            // Skip empty / deleted slots: flags bits & 3 != 0
            let flag = *(*pm).flags.add((k_0 >> 4) as usize);
            if ((flag >> ((k_0 & 0xf) << 1)) & 3) != 0 {
                k_0 = k_0.wrapping_add(1);
                continue;
            }
            let key = *(*pm).keys.cast::<*const c_char>().offset(k_0 as isize);
            if cram_cram_io_h_248_block_append(map, key.cast(), 2) < 0 {
                early_fail = true;
                break;
            }
            let tag = ((*key.offset(0) as u8 as c_int) << 8) | (*key.offset(1) as u8 as c_int);
            match tag {
                19785 | 21833 | 20553 | 16720 | 21070 | 21074 | 20815 => {
                    if cram_cram_io_h_261_block_append_char(
                        map,
                        (*(*pm).vals.cast::<pmap_val>().offset(k_0 as isize)).i as c_char,
                    ) < 0
                    {
                        early_fail = true;
                        break;
                    }
                }
                21325 => {
                    let smat: [c_char; 5] = [0x1b, 0x87u8 as c_char, 0x4b, 0x93u8 as c_char, 0x1b];
                    if cram_cram_io_h_248_block_append(map, smat.as_ptr().cast(), 5) < 0 {
                        early_fail = true;
                        break;
                    }
                }
                21572 => {
                    let td_bl = (*hl).td_blk;
                    r |= ((*fdl)
                        .vv
                        .varint_put32_blk
                        .expect("non-null function pointer")(
                        map, (*td_bl).byte as i32
                    ) <= 0) as c_int;
                    if cram_cram_io_h_248_block_append(map, (*td_bl).data.cast(), (*td_bl).byte) < 0
                    {
                        early_fail = true;
                        break;
                    }
                }
                _ => {
                    hts_log_cstr(
                        HTS_LOG_WARNING,
                        c"cram_encode_compression_header".as_ptr(),
                        c"Unknown preservation key".as_ptr(),
                    );
                }
            }
            mc += 1;
            k_0 = k_0.wrapping_add(1);
        }
    }
    if early_fail {
        return std::ptr::null_mut::<cram_block>();
    }

    r |= ((*fdl)
        .vv
        .varint_put32_blk
        .expect("non-null function pointer")(
        cb,
        ((*map_l).byte
            + (*fdl).vv.varint_size.expect("non-null function pointer")(mc as i64) as usize)
            as i32,
    ) <= 0) as c_int;
    r |= ((*fdl)
        .vv
        .varint_put32_blk
        .expect("non-null function pointer")(cb, mc)
        <= 0) as c_int;
    if cram_cram_io_h_248_block_append(cb, (*map_l).data.cast(), (*map_l).byte) < 0 {
        return std::ptr::null_mut::<cram_block>();
    }
    mc = 0;
    (*map_l).byte = 0;

    // Codec encoding-map. For each non-null codec, call its `.store` fn through
    // the codec-base layout. All codec layouts share the same prefix up through
    // `store`, so casting to `cram_codec_external_layout` is safe.
    macro_rules! store_codec {
        ($ds:expr, $name:expr) => {{
            let cd = (*hl).codecs[$ds].cast::<cram_codec_external_layout>();
            if !cd.is_null() {
                let store: CramCodecStoreFn = cram_fn((*cd).store);
                if -1
                    == store(
                        cd.cast::<c_void>(),
                        map,
                        $name.as_ptr() as *mut c_char,
                        (*fdl).version,
                    )
                {
                    return std::ptr::null_mut::<cram_block>();
                }
                mc += 1;
            }
        }};
    }

    store_codec!(DS_BF, c"BF");
    store_codec!(DS_CF, c"CF");
    store_codec!(DS_RL, c"RL");
    store_codec!(DS_AP, c"AP");
    store_codec!(DS_RG, c"RG");
    store_codec!(DS_MF, c"MF");
    store_codec!(DS_NS, c"NS");
    store_codec!(DS_NP, c"NP");
    store_codec!(DS_TS, c"TS");
    store_codec!(DS_NF, c"NF");
    store_codec!(DS_TC, c"TC");
    store_codec!(DS_TN_L, c"TN");
    store_codec!(DS_TL, c"TL");
    store_codec!(DS_FN, c"FN");
    store_codec!(DS_FC, c"FC");
    store_codec!(DS_FP, c"FP");
    store_codec!(DS_BS, c"BS");
    store_codec!(DS_IN_L, c"IN");
    store_codec!(DS_DL, c"DL");
    store_codec!(DS_BA, c"BA");
    store_codec!(DS_BB, c"BB");
    store_codec!(DS_MQ, c"MQ");
    store_codec!(DS_RN_L, c"RN");
    store_codec!(DS_QS_L, c"QS");
    store_codec!(DS_QQ, c"QQ");
    store_codec!(DS_RI, c"RI");
    if (*fdl).version >> 8 != 1 {
        store_codec!(DS_SC_L, c"SC");
        store_codec!(DS_RS, c"RS");
        store_codec!(DS_PD, c"PD");
        store_codec!(DS_HC, c"HC");
    }
    store_codec!(DS_TM, c"TM");
    store_codec!(DS_TV, c"TV");

    r |= ((*fdl)
        .vv
        .varint_put32_blk
        .expect("non-null function pointer")(
        cb,
        ((*map_l).byte
            + (*fdl).vv.varint_size.expect("non-null function pointer")(mc as i64) as usize)
            as i32,
    ) <= 0) as c_int;
    r |= ((*fdl)
        .vv
        .varint_put32_blk
        .expect("non-null function pointer")(cb, mc)
        <= 0) as c_int;
    if cram_cram_io_h_248_block_append(cb, (*map_l).data.cast(), (*map_l).byte) < 0 {
        return std::ptr::null_mut::<cram_block>();
    }
    mc = 0;
    (*map_l).byte = 0;

    if !(*cl).tags_used.is_null() {
        let tu = (*cl).tags_used;
        let mut k_1: u32 = 0;
        while k_1 != (*tu).n_buckets {
            let flag = *(*tu).flags.add((k_1 >> 4) as usize);
            if ((flag >> ((k_1 & 0xf) << 1)) & 3) != 0 {
                k_1 = k_1.wrapping_add(1);
                continue;
            }
            let key_0 = *(*tu).keys.cast::<c_int>().offset(k_1 as isize);
            // tags_used vals are pointers to a tag-map entry whose first field is
            // a `*mut cram_codec` (cram_tag_map). Read the codec pointer via the
            // val layout.
            let val_ptr = *(*tu).vals.cast::<*mut c_void>().offset(k_1 as isize);
            // First sizeof(ptr) bytes are the codec pointer.
            let cd_void = *(val_ptr.cast::<*mut c_void>());
            let cd = cd_void.cast::<cram_codec_external_layout>();
            r |= ((*fdl)
                .vv
                .varint_put32_blk
                .expect("non-null function pointer")(map, key_0)
                <= 0) as c_int;
            let store: CramCodecStoreFn = cram_fn((*cd).store);
            if -1
                == store(
                    cd.cast::<c_void>(),
                    map,
                    std::ptr::null_mut::<c_char>(),
                    (*fdl).version,
                )
            {
                return std::ptr::null_mut::<cram_block>();
            }
            mc += 1;
            k_1 = k_1.wrapping_add(1);
        }
    }

    r |= ((*fdl)
        .vv
        .varint_put32_blk
        .expect("non-null function pointer")(
        cb,
        ((*map_l).byte
            + (*fdl).vv.varint_size.expect("non-null function pointer")(mc as i64) as usize)
            as i32,
    ) <= 0) as c_int;
    r |= ((*fdl)
        .vv
        .varint_put32_blk
        .expect("non-null function pointer")(cb, mc)
        <= 0) as c_int;
    if cram_cram_io_h_248_block_append(cb, (*map_l).data.cast(), (*map_l).byte) < 0 {
        return std::ptr::null_mut::<cram_block>();
    }

    hts_log_cstr(
        HTS_LOG_INFO,
        c"cram_encode_compression_header".as_ptr(),
        c"Wrote compression block header".as_ptr(),
    );
    (*cb_l).uncomp_size = (*cb_l).byte as i32;
    (*cb_l).comp_size = (*cb_l).uncomp_size;
    cram_cram_io_c_1565_cram_free_block(map);
    if r >= 0 {
        return cb;
    }
    std::ptr::null_mut::<cram_block>()
}

/// `cram_allocate_block` (htslib/cram/cram_encode.c:1006). Allocates an
/// external block for one codec when needed, dispatching by encoding kind.
/// Returns 0 on success, -1 on allocation failure.
pub unsafe fn cram_cram_encode_c_1006_cram_allocate_block(
    codec: *mut cram_codec,
    s: *mut cram_slice,
    ds_id: c_int,
) -> c_int {
    if codec.is_null() {
        return 0;
    }
    let sl = s.cast::<cram_slice_layout>();
    let cb = codec.cast::<cram_codec_base_layout>();
    let enc = (*cb).codec as c_uint;
    match enc {
        // E_GOLOMB | E_HUFFMAN | E_BETA | E_SUBEXP | E_GOLOMB_RICE | E_GAMMA
        // → output to the slice's CORE block (block[0]).
        2 | 3 | 6 | 7 | 8 | 9 => {
            (*cb).out = *(*sl).block.offset(0);
        }
        // E_CONST_BYTE | E_CONST_INT
        43 | 44 => {
            (*cb).out = std::ptr::null_mut();
        }
        // E_EXTERNAL | E_VARINT_UNSIGNED | E_VARINT_SIGNED
        1 | 41 | 42 => {
            let blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, ds_id);
            *(*sl).block.offset(ds_id as isize) = blk.cast::<cram_block_layout>();
            if blk.is_null() {
                return -1;
            }
            let cx = codec.cast::<cram_codec_external_layout>();
            (*cx).external.content_id = ds_id;
            (*cb).out = blk.cast::<cram_block_layout>();
        }
        // E_BYTE_ARRAY_STOP
        5 => {
            let blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, ds_id);
            *(*sl).block.offset(ds_id as isize) = blk.cast::<cram_block_layout>();
            if blk.is_null() {
                return -1;
            }
            let cbas = codec.cast::<cram_codec_byte_array_stop_layout>();
            (*cbas).byte_array_stop.content_id = ds_id;
            (*cb).out = blk.cast::<cram_block_layout>();
        }
        // E_BYTE_ARRAY_LEN
        4 => {
            let cbal = codec.cast::<cram_codec_byte_array_len_layout>();
            // Recurse into len_codec and val_codec; each uses its own content_id.
            let len_c = (*cbal).byte_array_len.len_codec.cast::<cram_codec>();
            let len_cb = len_c.cast::<cram_codec_external_layout>();
            if cram_cram_encode_c_1006_cram_allocate_block(len_c, s, (*len_cb).external.content_id)
                != 0
            {
                return -1;
            }
            let val_c = (*cbal).byte_array_len.val_codec.cast::<cram_codec>();
            let val_cb = val_c.cast::<cram_codec_external_layout>();
            if cram_cram_encode_c_1006_cram_allocate_block(val_c, s, (*val_cb).external.content_id)
                != 0
            {
                return -1;
            }
        }
        // E_XRLE
        52 => {
            let cxr = codec.cast::<cram_codec_xrle_layout>();
            if cram_cram_encode_c_1006_cram_allocate_block(
                (*cxr).xrle.len_codec.cast::<cram_codec>(),
                s,
                ds_id,
            ) != 0
            {
                return -1;
            }
            if cram_cram_encode_c_1006_cram_allocate_block(
                (*cxr).xrle.lit_codec.cast::<cram_codec>(),
                s,
                ds_id,
            ) != 0
            {
                return -1;
            }
        }
        // E_XPACK
        51 => {
            let cxp = codec.cast::<cram_codec_xpack_layout>();
            if cram_cram_encode_c_1006_cram_allocate_block(
                (*cxp).xpack.sub_codec.cast::<cram_codec>(),
                s,
                ds_id,
            ) != 0
            {
                return -1;
            }
            let outb = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_FILE_HEADER, 0);
            (*cb).out = outb.cast::<cram_block_layout>();
            if outb.is_null() {
                return -1;
            }
        }
        // E_XDELTA
        53 => {
            let cxd = codec.cast::<cram_codec_xdelta_layout>();
            if cram_cram_encode_c_1006_cram_allocate_block(
                (*cxd).xdelta.sub_codec.cast::<cram_codec>(),
                s,
                ds_id,
            ) != 0
            {
                return -1;
            }
            let outb = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_FILE_HEADER, 0);
            (*cb).out = outb.cast::<cram_block_layout>();
            if outb.is_null() {
                return -1;
            }
        }
        _ => {}
    }
    0
}

/// `cram_encode_slice_header` (htslib/cram/cram_encode.c:512). Encodes the
/// per-slice block header into a freshly-allocated MAPPED_SLICE block.
pub unsafe fn cram_cram_encode_c_512_cram_encode_slice_header(
    fd: *mut cram_fd,
    s: *mut cram_slice,
) -> *mut cram_block {
    let fdl = fd.cast::<cram_fd_layout>();
    let sl = s.cast::<cram_slice_layout>();
    let b = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_MAPPED_SLICE, 0);
    if b.is_null() {
        return std::ptr::null_mut::<cram_block>();
    }
    let hdr = (*sl).hdr;
    let buf_sz = (22 + 16 + 5 * (8 + (*hdr).num_blocks)) as usize;
    let buf = malloc(buf_sz as u64).cast::<c_char>();
    let mut cp = buf;
    if buf.is_null() {
        cram_cram_io_c_1565_cram_free_block(b);
        return std::ptr::null_mut::<cram_block>();
    }
    cp = cp.offset(
        ((*fdl).vv.varint_put32s.expect("non-null function pointer"))(
            cp,
            std::ptr::null_mut::<c_char>(),
            (*hdr).ref_seq_id,
        ) as isize,
    );
    if (*fdl).version >> 8 >= 4 {
        cp = cp.offset(
            ((*fdl).vv.varint_put64.expect("non-null function pointer"))(
                cp,
                std::ptr::null_mut::<c_char>(),
                (*hdr).ref_seq_start,
            ) as isize,
        );
        cp = cp.offset(
            ((*fdl).vv.varint_put64.expect("non-null function pointer"))(
                cp,
                std::ptr::null_mut::<c_char>(),
                (*hdr).ref_seq_span,
            ) as isize,
        );
    } else {
        if (*hdr).ref_seq_start < 0 || (*hdr).ref_seq_start > c_int::MAX as i64 {
            hts_log_cstr(
                HTS_LOG_ERROR,
                c"cram_encode_slice_header".as_ptr(),
                c"Reference position too large for CRAM 3".as_ptr(),
            );
            cram_cram_io_c_1565_cram_free_block(b);
            free(buf.cast());
            return std::ptr::null_mut::<cram_block>();
        }
        cp = cp.offset(
            ((*fdl).vv.varint_put32.expect("non-null function pointer"))(
                cp,
                std::ptr::null_mut::<c_char>(),
                (*hdr).ref_seq_start as i32,
            ) as isize,
        );
        cp = cp.offset(
            ((*fdl).vv.varint_put32.expect("non-null function pointer"))(
                cp,
                std::ptr::null_mut::<c_char>(),
                (*hdr).ref_seq_span as i32,
            ) as isize,
        );
    }
    cp = cp.offset(
        ((*fdl).vv.varint_put32.expect("non-null function pointer"))(
            cp,
            std::ptr::null_mut::<c_char>(),
            (*hdr).num_records,
        ) as isize,
    );
    if (*fdl).version >> 8 == 2 {
        cp = cp.offset(
            ((*fdl).vv.varint_put32.expect("non-null function pointer"))(
                cp,
                std::ptr::null_mut::<c_char>(),
                (*hdr).record_counter as i32,
            ) as isize,
        );
    } else if (*fdl).version >> 8 >= 3 {
        cp = cp.offset(
            ((*fdl).vv.varint_put64.expect("non-null function pointer"))(
                cp,
                std::ptr::null_mut::<c_char>(),
                (*hdr).record_counter,
            ) as isize,
        );
    }
    cp = cp.offset(
        ((*fdl).vv.varint_put32.expect("non-null function pointer"))(
            cp,
            std::ptr::null_mut::<c_char>(),
            (*hdr).num_blocks,
        ) as isize,
    );
    cp = cp.offset(
        ((*fdl).vv.varint_put32.expect("non-null function pointer"))(
            cp,
            std::ptr::null_mut::<c_char>(),
            (*hdr).num_content_ids,
        ) as isize,
    );
    let mut j: c_int = 0;
    while j < (*hdr).num_content_ids {
        cp = cp.offset(
            ((*fdl).vv.varint_put32.expect("non-null function pointer"))(
                cp,
                std::ptr::null_mut::<c_char>(),
                *(*hdr).block_content_ids.offset(j as isize),
            ) as isize,
        );
        j += 1;
    }
    if (*hdr).content_type == CRAM_CONTENT_TYPE_MAPPED_SLICE {
        cp = cp.offset(
            ((*fdl).vv.varint_put32.expect("non-null function pointer"))(
                cp,
                std::ptr::null_mut::<c_char>(),
                (*hdr).ref_base_id,
            ) as isize,
        );
    }
    if (*fdl).version >> 8 != 1 {
        memcpy(cp.cast(), (&raw const (*hdr).md5) as *const c_void, 16);
        cp = cp.offset(16);
    }
    // Assert from the C source: written bytes fit within the buf allocation.
    debug_assert!(cp.offset_from(buf) as i64 <= (22 + 16 + 5 * (8 + (*hdr).num_blocks)) as i64);
    let bl = b.cast::<cram_block_layout>();
    (*bl).data = buf.cast();
    (*bl).uncomp_size = cp.offset_from(buf) as i32;
    (*bl).comp_size = (*bl).uncomp_size;
    b
}

/// `cram_encode_slice_read` (htslib/cram/cram_encode.c:573). Encodes one
/// `cram_record` into the slice by dispatching to per-DS codecs. Returns
/// 0 on success, -1 on error.
pub unsafe fn cram_cram_encode_c_573_cram_encode_slice_read(
    fd: *mut cram_fd,
    c: *mut cram_container,
    h: *mut cram_block_compression_hdr,
    s: *mut cram_slice,
    cr: *mut cram_record,
    last_pos: *mut i64,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let hl = h.cast::<cram_block_compression_hdr_layout>();
    let sl = s.cast::<cram_slice_layout>();
    let crl = cr.cast::<cram_record_layout>();
    let mut r: c_int = 0;
    let mut i32v: i32;
    let mut i64v: i64;
    let mut uc: c_uchar;

    // DS_BF: cram_flag_swap[flags & 0xfff]
    i32v = (*fdl).cram_flag_swap[((*crl).flags & 0xfff) as usize] as i32;
    r |= cram_codec_encode(
        (*hl).codecs[DS_ENC_BF as usize],
        s,
        &raw mut i32v as *mut c_char,
        1,
    );

    // DS_CF: cram_flags & CRAM_FLAG_MASK
    i32v = (*crl).cram_flags & CRAM_FLAG_MASK_ENC;
    r |= cram_codec_encode(
        (*hl).codecs[DS_ENC_CF as usize],
        s,
        &raw mut i32v as *mut c_char,
        1,
    );

    // DS_RI: when v>=2 and slice.ref_seq_id == -2 (multi-ref)
    if (*fdl).version >> 8 != 1 && (*(*sl).hdr).ref_seq_id == -2 {
        r |= cram_codec_encode(
            (*hl).codecs[DS_ENC_RI as usize],
            s,
            &raw mut (*crl).ref_id as *mut c_char,
            1,
        );
    }

    // DS_RL: read length
    r |= cram_codec_encode(
        (*hl).codecs[DS_ENC_RL as usize],
        s,
        &raw mut (*crl).len as *mut c_char,
        1,
    );

    // DS_AP: alignment position (delta or absolute, 32 or 64 bit by version).
    if (*cl).pos_sorted != 0 {
        if (*fdl).version >> 8 >= 4 {
            i64v = (*crl).apos - *last_pos;
            r |= cram_codec_encode(
                (*hl).codecs[DS_ENC_AP as usize],
                s,
                &raw mut i64v as *mut c_char,
                1,
            );
        } else {
            i32v = ((*crl).apos - *last_pos) as i32;
            r |= cram_codec_encode(
                (*hl).codecs[DS_ENC_AP as usize],
                s,
                &raw mut i32v as *mut c_char,
                1,
            );
        }
        *last_pos = (*crl).apos;
    } else if (*fdl).version >> 8 >= 4 {
        i64v = (*crl).apos;
        r |= cram_codec_encode(
            (*hl).codecs[DS_ENC_AP as usize],
            s,
            &raw mut i64v as *mut c_char,
            1,
        );
    } else {
        i32v = (*crl).apos as i32;
        r |= cram_codec_encode(
            (*hl).codecs[DS_ENC_AP as usize],
            s,
            &raw mut i32v as *mut c_char,
            1,
        );
    }

    // DS_RG: read group
    r |= cram_codec_encode(
        (*hl).codecs[DS_ENC_RG as usize],
        s,
        &raw mut (*crl).rg as *mut c_char,
        1,
    );

    if (*crl).cram_flags & CRAM_FLAG_DETACHED_ENC != 0 {
        // DS_MF: mate flags
        i32v = (*crl).mate_flags;
        r |= cram_codec_encode(
            (*hl).codecs[DS_ENC_MF as usize],
            s,
            &raw mut i32v as *mut c_char,
            1,
        );
        // DS_NS: mate ref id
        r |= cram_codec_encode(
            (*hl).codecs[DS_ENC_NS as usize],
            s,
            &raw mut (*crl).mate_ref_id as *mut c_char,
            1,
        );
        if (*fdl).version >> 8 >= 4 {
            // DS_NP: mate pos (64-bit)
            r |= cram_codec_encode(
                (*hl).codecs[DS_ENC_NP as usize],
                s,
                &raw mut (*crl).mate_pos as *mut c_char,
                1,
            );
            // DS_TS: tlen (64-bit)
            r |= cram_codec_encode(
                (*hl).codecs[DS_ENC_TS as usize],
                s,
                &raw mut (*crl).tlen as *mut c_char,
                1,
            );
        } else {
            i32v = (*crl).mate_pos as i32;
            r |= cram_codec_encode(
                (*hl).codecs[DS_ENC_NP as usize],
                s,
                &raw mut i32v as *mut c_char,
                1,
            );
            i32v = (*crl).tlen as i32;
            r |= cram_codec_encode(
                (*hl).codecs[DS_ENC_TS as usize],
                s,
                &raw mut i32v as *mut c_char,
                1,
            );
        }
    } else {
        if (*crl).cram_flags & CRAM_FLAG_MATE_DOWNSTREAM_ENC != 0 {
            r |= cram_codec_encode(
                (*hl).codecs[DS_ENC_NF as usize],
                s,
                &raw mut (*crl).mate_line as *mut c_char,
                1,
            );
        }
        if (*crl).cram_flags & CRAM_FLAG_EXPLICIT_TLEN_ENC != 0 && (*fdl).version >> 8 >= 4 {
            r |= cram_codec_encode(
                (*hl).codecs[DS_ENC_TS as usize],
                s,
                &raw mut (*crl).tlen as *mut c_char,
                1,
            );
        }
    }

    if (*fdl).version >> 8 == 1 {
        // DS_TC: ntags as uchar
        uc = (*crl).ntags as c_uchar;
        r |= cram_codec_encode(
            (*hl).codecs[DS_ENC_TC as usize],
            s,
            &raw mut uc as *mut c_char,
            1,
        );
        let mut jj: c_int = 0;
        while jj < (*crl).ntags {
            let mut tn_v: u32 = *(*sl).tn.offset(((*crl).tn_idx + jj) as isize);
            r |= cram_codec_encode(
                (*hl).codecs[DS_TN as usize],
                s,
                &raw mut tn_v as *mut c_char,
                1,
            );
            jj += 1;
        }
    } else {
        // DS_TL: tag-line index
        r |= cram_codec_encode(
            (*hl).codecs[DS_ENC_TL as usize],
            s,
            &raw mut (*crl).tl as *mut c_char,
            1,
        );
    }

    if (*crl).flags & BAM_FUNMAP_ENC == 0 {
        // Mapped: encode features.
        let mut prev_pos: c_int = 0;
        // DS_FN: feature count
        r |= cram_codec_encode(
            (*hl).codecs[DS_ENC_FN as usize],
            s,
            &raw mut (*crl).nfeature as *mut c_char,
            1,
        );
        let mut j_feat: u32 = 0;
        while j_feat < (*crl).nfeature {
            let f = (*sl)
                .features
                .offset((*crl).feature.wrapping_add(j_feat) as isize);
            // Feature uses X-layout for the base prefix (pos, code, base);
            // for variants that overlay len/qual we re-cast.
            let fx = f.cast::<cram_feature_X_layout>();
            uc = (*fx).code as c_uchar;
            r |= cram_codec_encode(
                (*hl).codecs[DS_ENC_FC as usize],
                s,
                &raw mut uc as *mut c_char,
                1,
            );
            i32v = ((*fx).pos - prev_pos) as i32;
            r |= cram_codec_encode(
                (*hl).codecs[DS_ENC_FP as usize],
                s,
                &raw mut i32v as *mut c_char,
                1,
            );
            prev_pos = (*fx).pos;
            match (*fx).code {
                88 => {
                    // 'X' — substitution
                    uc = (*fx).base as c_uchar;
                    r |= cram_codec_encode(
                        (*hl).codecs[DS_ENC_BS as usize],
                        s,
                        &raw mut uc as *mut c_char,
                        1,
                    );
                }
                105 => {
                    // 'i' — insert one base
                    let fi = f.cast::<cram_feature_i_layout>();
                    uc = (*fi).base as c_uchar;
                    r |= cram_codec_encode(
                        (*hl).codecs[DS_ENC_BA as usize],
                        s,
                        &raw mut uc as *mut c_char,
                        1,
                    );
                }
                68 => {
                    // 'D' — deletion
                    let fd_ = f.cast::<cram_feature_D_layout>();
                    i32v = (*fd_).len;
                    r |= cram_codec_encode(
                        (*hl).codecs[DS_ENC_DL as usize],
                        s,
                        &raw mut i32v as *mut c_char,
                        1,
                    );
                }
                66 => {
                    // 'B' — base+qual
                    let fb = f.cast::<cram_feature_B_layout>();
                    uc = (*fb).base as c_uchar;
                    r |= cram_codec_encode(
                        (*hl).codecs[DS_ENC_BA as usize],
                        s,
                        &raw mut uc as *mut c_char,
                        1,
                    );
                }
                98 => {
                    // 'b' — bases block: emit a run from seqs_blk
                    let fbb = f.cast::<cram_feature_b_layout>();
                    let seqs_data = (*(*sl).seqs_blk).data.cast::<c_char>();
                    r |= cram_codec_encode(
                        (*hl).codecs[DS_ENC_BB as usize],
                        s,
                        seqs_data.offset((*fbb).seq_idx as isize),
                        (*fbb).len,
                    );
                }
                83 | 73 | 81 => {
                    // 'S' (soft-clip), 'I' (insert), 'Q' (qual): handled
                    // by the codec's accumulated state — nothing to do here.
                }
                78 => {
                    // 'N' — ref-skip
                    let fn_ = f.cast::<cram_feature_D_layout>(); // (pos, code, len) shape
                    i32v = (*fn_).len;
                    r |= cram_codec_encode(
                        (*hl).codecs[DS_ENC_RS as usize],
                        s,
                        &raw mut i32v as *mut c_char,
                        1,
                    );
                }
                80 => {
                    // 'P' — padding
                    let fp = f.cast::<cram_feature_D_layout>();
                    i32v = (*fp).len;
                    r |= cram_codec_encode(
                        (*hl).codecs[DS_ENC_PD as usize],
                        s,
                        &raw mut i32v as *mut c_char,
                        1,
                    );
                }
                72 => {
                    // 'H' — hard-clip
                    let fh = f.cast::<cram_feature_D_layout>();
                    i32v = (*fh).len;
                    r |= cram_codec_encode(
                        (*hl).codecs[DS_ENC_HC as usize],
                        s,
                        &raw mut i32v as *mut c_char,
                        1,
                    );
                }
                _ => {
                    hts_log_cstr(
                        HTS_LOG_ERROR,
                        c"cram_encode_slice_read".as_ptr(),
                        c"Unhandled feature code".as_ptr(),
                    );
                    return -1;
                }
            }
            j_feat += 1;
        }
        // DS_MQ: mapping quality
        r |= cram_codec_encode(
            (*hl).codecs[DS_ENC_MQ as usize],
            s,
            &raw mut (*crl).mqual as *mut c_char,
            1,
        );
    } else {
        // Unmapped: emit the raw sequence bases via DS_BA.
        let seq = (*(*sl).seqs_blk)
            .data
            .cast::<c_char>()
            .offset((*crl).seq as isize);
        if (*crl).len != 0 {
            r |= cram_codec_encode((*hl).codecs[DS_ENC_BA as usize], s, seq, (*crl).len);
        }
    }

    if r != 0 {
        -1
    } else {
        0
    }
}

/// `cram_compress_slice` (htslib/cram/cram_encode.c:804). Choose a set of
/// compression methods per data series and compress every block of the slice.
/// Returns 0 on success, -1 on failure.
// `method_f` is assigned in branches that are later unconditionally overwritten;
// retained for literal parity with the C source.
#[allow(unused_assignments)]
pub unsafe fn cram_cram_encode_c_804_cram_compress_slice(
    fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let sl = s.cast::<cram_slice_layout>();
    let level = (*fdl).level;
    let mut method: c_int = (1 << CBMI_GZIP) | (1 << CBMI_GZIP_RLE);
    let mut method_f: c_int = method;
    let v31_or_above: c_int = ((*fdl).version > (3 << 8)) as c_int;

    // CORE block: at level>5 with > 500 bytes uncompressed, opportunistically
    // pre-compress with GZIP into the CORE block.
    if level > 5 && (*(*(*sl).block.offset(0))).uncomp_size > 500 {
        cram_cram_io_c_2317_cram_compress_block2(
            fd,
            sl.cast::<c_void>(),
            (*sl).block.offset(0).read().cast::<cram_block>(),
            std::ptr::null_mut::<cram_metrics>(),
            1 << CBMI_GZIP,
            1,
        );
    }
    if (*fdl).use_bz2 != 0 {
        method |= 1 << CBMI_BZIP2;
    }

    let method_rans: c_int = (1 << CBMI_RANS0) | (1 << CBMI_RANS1);
    let mut method_ranspr: c_int = method_rans;
    if (*fdl).use_rans != 0 {
        method_ranspr = (1 << CBMI_RANS_PR0) | (1 << CBMI_RANS_PR1);
        if level > 1 {
            method_ranspr |= (1 << CBMI_RANS_PR64)
                | (1 << CBMI_RANS_PR9)
                | (1 << CBMI_RANS_PR128)
                | (1 << CBMI_RANS_PR193);
        }
        if level > 5 {
            method_ranspr |= (1 << CBMI_RANS_PR129) | (1 << CBMI_RANS_PR192);
        }
    }
    if (*fdl).use_rans != 0 {
        method_f |= if v31_or_above != 0 {
            method_ranspr
        } else {
            method_rans
        };
        method |= if v31_or_above != 0 {
            method_ranspr
        } else {
            method_rans
        };
    }

    let mut method_arith: c_int = 0;
    if (*fdl).use_arith != 0 {
        method_arith = (1 << CBMI_ARITH_PR0) | (1 << CBMI_ARITH_PR1);
        if level > 1 {
            method_arith |= (1 << CBMI_ARITH_PR64)
                | (1 << CBMI_ARITH_PR9)
                | (1 << CBMI_ARITH_PR128)
                | (1 << CBMI_ARITH_PR129)
                | (1 << CBMI_ARITH_PR192)
                | (1 << CBMI_ARITH_PR193);
        }
    }
    if (*fdl).use_arith != 0 && v31_or_above != 0 {
        method_f |= method_arith;
        method |= method_arith;
    }

    if (*fdl).use_lzma != 0 {
        method |= 1 << CBMI_LZMA;
    }

    method_f = method & !((1 << CBMI_GZIP) | (1 << CBMI_BZIP2) | (1 << CBMI_LZMA));

    if level >= 5 {
        method |= 1 << CBMI_GZIP_1;
        method_f = method;
    }
    if level == 1 {
        method &= !(1 << CBMI_GZIP);
        method |= 1 << CBMI_GZIP_1;
        method_f = method;
    }

    let mut qmethod: c_int = method;
    let mut qmethod_f: c_int = method;
    if v31_or_above != 0 && (*fdl).use_fqz != 0 {
        qmethod |= 1 << CBMI_FQZ;
        qmethod_f |= 1 << CBMI_FQZ;
        if (*fdl).level > 4 {
            qmethod |= 1 << CBMI_FQZ_B;
            qmethod_f |= 1 << CBMI_FQZ_B;
        }
        if (*fdl).level > 6 {
            qmethod |= (1 << CBMI_FQZ_C) | (1 << CBMI_FQZ_D);
            qmethod_f |= (1 << CBMI_FQZ_C) | (1 << CBMI_FQZ_D);
        }
    }

    crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).metrics_lock);
    let mut i: c_int = 0;
    while i < DS_ENC_END {
        if !(*cl).stats[i as usize].is_null() && (*(*cl).stats[i as usize]).nvals > 16 {
            (*(*fdl).m[i as usize]).unpackable = 1;
        }
        i += 1;
    }
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).metrics_lock);

    // DS_IN: insertion sequences
    if cram_cram_io_c_2317_cram_compress_block2(
        fd,
        sl.cast::<c_void>(),
        (*sl)
            .block
            .offset(DS_IN as isize)
            .read()
            .cast::<cram_block>(),
        (*fdl).m[DS_IN as usize].cast::<cram_metrics>(),
        method,
        level,
    ) != 0
    {
        return -1;
    }

    if (*fdl).level != 0 {
        if (*fdl).level == 1 {
            if cram_cram_io_c_2317_cram_compress_block2(
                fd,
                sl.cast::<c_void>(),
                (*sl)
                    .block
                    .offset(DS_QS as isize)
                    .read()
                    .cast::<cram_block>(),
                (*fdl).m[DS_QS as usize].cast::<cram_metrics>(),
                qmethod_f,
                1,
            ) != 0
            {
                return -1;
            }
            i = DS_ENC_aux;
            while i <= DS_ENC_aux_oz {
                if !(*(*sl).block.offset(i as isize)).is_null()
                    && cram_cram_io_c_2317_cram_compress_block2(
                        fd,
                        sl.cast::<c_void>(),
                        (*sl).block.offset(i as isize).read().cast::<cram_block>(),
                        (*fdl).m[i as usize].cast::<cram_metrics>(),
                        method,
                        1,
                    ) != 0
                {
                    return -1;
                }
                i += 1;
            }
        } else if (*fdl).level < 3 {
            if cram_cram_io_c_2317_cram_compress_block2(
                fd,
                sl.cast::<c_void>(),
                (*sl)
                    .block
                    .offset(DS_QS as isize)
                    .read()
                    .cast::<cram_block>(),
                (*fdl).m[DS_QS as usize].cast::<cram_metrics>(),
                qmethod,
                1,
            ) != 0
            {
                return -1;
            }
            if cram_cram_io_c_2317_cram_compress_block2(
                fd,
                sl.cast::<c_void>(),
                (*sl)
                    .block
                    .offset(DS_ENC_BA as isize)
                    .read()
                    .cast::<cram_block>(),
                (*fdl).m[DS_ENC_BA as usize].cast::<cram_metrics>(),
                method,
                1,
            ) != 0
            {
                return -1;
            }
            if !(*(*sl).block.offset(DS_ENC_BB as isize)).is_null()
                && cram_cram_io_c_2317_cram_compress_block2(
                    fd,
                    sl.cast::<c_void>(),
                    (*sl)
                        .block
                        .offset(DS_ENC_BB as isize)
                        .read()
                        .cast::<cram_block>(),
                    (*fdl).m[DS_ENC_BB as usize].cast::<cram_metrics>(),
                    method,
                    1,
                ) != 0
            {
                return -1;
            }
            i = DS_ENC_aux;
            while i <= DS_ENC_aux_oz {
                if !(*(*sl).block.offset(i as isize)).is_null()
                    && cram_cram_io_c_2317_cram_compress_block2(
                        fd,
                        sl.cast::<c_void>(),
                        (*sl).block.offset(i as isize).read().cast::<cram_block>(),
                        (*fdl).m[i as usize].cast::<cram_metrics>(),
                        method,
                        level,
                    ) != 0
                {
                    return -1;
                }
                i += 1;
            }
        } else {
            if cram_cram_io_c_2317_cram_compress_block2(
                fd,
                sl.cast::<c_void>(),
                (*sl)
                    .block
                    .offset(DS_QS as isize)
                    .read()
                    .cast::<cram_block>(),
                (*fdl).m[DS_QS as usize].cast::<cram_metrics>(),
                qmethod,
                level,
            ) != 0
            {
                return -1;
            }
            if cram_cram_io_c_2317_cram_compress_block2(
                fd,
                sl.cast::<c_void>(),
                (*sl)
                    .block
                    .offset(DS_ENC_BA as isize)
                    .read()
                    .cast::<cram_block>(),
                (*fdl).m[DS_ENC_BA as usize].cast::<cram_metrics>(),
                method,
                level,
            ) != 0
            {
                return -1;
            }
            if !(*(*sl).block.offset(DS_ENC_BB as isize)).is_null()
                && cram_cram_io_c_2317_cram_compress_block2(
                    fd,
                    sl.cast::<c_void>(),
                    (*sl)
                        .block
                        .offset(DS_ENC_BB as isize)
                        .read()
                        .cast::<cram_block>(),
                    (*fdl).m[DS_ENC_BB as usize].cast::<cram_metrics>(),
                    method,
                    level,
                ) != 0
            {
                return -1;
            }
            i = DS_ENC_aux;
            while i <= DS_ENC_aux_oz {
                if !(*(*sl).block.offset(i as isize)).is_null()
                    && cram_cram_io_c_2317_cram_compress_block2(
                        fd,
                        sl.cast::<c_void>(),
                        (*sl).block.offset(i as isize).read().cast::<cram_block>(),
                        (*fdl).m[i as usize].cast::<cram_metrics>(),
                        method,
                        level,
                    ) != 0
                {
                    return -1;
                }
                i += 1;
            }
        }
    }

    // DS_RN: read names — strip the RANS-family methods (use tok3/toka instead at v3.1+).
    let mut method_rn: c_int = method & !(method_rans | method_ranspr | (1 << CBMI_GZIP_RLE));
    if (*fdl).version > (3 << 8) && (*fdl).use_tok != 0 {
        method_rn |= if (*fdl).use_arith != 0 {
            1 << CBMI_TOKA
        } else {
            1 << CBMI_TOK3
        };
    }
    if cram_cram_io_c_2317_cram_compress_block2(
        fd,
        sl.cast::<c_void>(),
        (*sl)
            .block
            .offset(DS_RN as isize)
            .read()
            .cast::<cram_block>(),
        (*fdl).m[DS_RN as usize].cast::<cram_metrics>(),
        method_rn,
        level,
    ) != 0
    {
        return -1;
    }

    // DS_NS: only if non-null and not aliased to CORE.
    if !(*(*sl).block.offset(DS_ENC_NS as isize)).is_null()
        && *(*sl).block.offset(DS_ENC_NS as isize) != *(*sl).block.offset(0)
        && cram_cram_io_c_2317_cram_compress_block2(
            fd,
            sl.cast::<c_void>(),
            (*sl)
                .block
                .offset(DS_ENC_NS as isize)
                .read()
                .cast::<cram_block>(),
            (*fdl).m[DS_ENC_NS as usize].cast::<cram_metrics>(),
            method,
            level,
        ) != 0
    {
        return -1;
    }

    // Tag aux blocks past DS_END.
    let mut i_0: c_int = DS_ENC_END;
    while i_0 < (*(*sl).hdr).num_blocks {
        if !(*(*sl).block.offset(i_0 as isize)).is_null()
            && *(*sl).block.offset(i_0 as isize) != *(*sl).block.offset(0)
            && (*(*(*sl).block.offset(i_0 as isize))).method == CBMI_RAW
            && cram_cram_io_c_2317_cram_compress_block2(
                fd,
                sl.cast::<c_void>(),
                (*sl).block.offset(i_0 as isize).read().cast::<cram_block>(),
                (*(*(*sl).block.offset(i_0 as isize)))
                    .m
                    .cast::<cram_metrics>(),
                method,
                level,
            ) != 0
        {
            return -1;
        }
        i_0 += 1;
    }

    // Remaining intra-slice blocks 1..DS_END that are still raw.
    let mut i_1: c_int = 1;
    while i_1 < (*(*sl).hdr).num_blocks && i_1 < DS_ENC_END {
        if !(*(*sl).block.offset(i_1 as isize)).is_null()
            && *(*sl).block.offset(i_1 as isize) != *(*sl).block.offset(0)
            && (*(*(*sl).block.offset(i_1 as isize))).method == CBMI_RAW
            && cram_cram_io_c_2317_cram_compress_block2(
                fd,
                sl.cast::<c_void>(),
                (*sl).block.offset(i_1 as isize).read().cast::<cram_block>(),
                (*fdl).m[i_1 as usize].cast::<cram_metrics>(),
                method_f,
                level,
            ) != 0
        {
            return -1;
        }
        i_1 += 1;
    }
    0
}

/// `cram_encode_slice` (htslib/cram/cram_encode.c:1097). Top-level driver:
/// allocates per-DS blocks, encodes every record via cram_encode_slice_read,
/// transfers staging blocks into the slice, compresses, and writes the
/// slice header. Returns 0 on success, -1 on failure.
pub unsafe fn cram_cram_encode_c_1097_cram_encode_slice(
    fd: *mut cram_fd,
    c: *mut cram_container,
    h: *mut cram_block_compression_hdr,
    s: *mut cram_slice,
    embed_ref: c_int,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let hl = h.cast::<cram_block_compression_hdr_layout>();
    let sl = s.cast::<cram_slice>();
    let sll = s.cast::<cram_slice_layout>();
    let r: c_int = 0;
    let mut last_pos: i64;
    let mut id: c_int;

    (*(*sll).hdr).ref_base_id = if embed_ref > 0 && (*(*sll).hdr).ref_seq_span > 0 {
        DS_ENC_ref
    } else if (*fdl).version >> 8 >= 4 {
        0
    } else {
        -1
    };
    (*(*sll).hdr).record_counter = (*cl).num_records as i64 + (*cl).record_counter;
    (*cl).num_records += (*(*sll).hdr).num_records;

    let ntags: c_int = if !(*cl).tags_used.is_null() {
        (*(*cl).tags_used).n_occupied as c_int
    } else {
        0
    };
    (*sll).block = calloc(
        (DS_ENC_END + ntags * 2) as u64,
        std::mem::size_of::<*mut cram_block_layout>() as u64,
    )
    .cast::<*mut cram_block_layout>();
    (*(*sll).hdr).block_content_ids =
        malloc((DS_ENC_END as u64).wrapping_mul(std::mem::size_of::<i32>() as u64)).cast::<i32>();
    if (*sll).block.is_null() || (*(*sll).hdr).block_content_ids.is_null() {
        return -1;
    }
    let core = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_CORE, 0);
    *(*sll).block.offset(0) = core.cast::<cram_block_layout>();
    if core.is_null() {
        return -1;
    }
    if (*fdl).version >> 8 == 1 {
        let tn_codec = (*hl).codecs[DS_TN as usize].cast::<cram_codec_external_layout>();
        if (*tn_codec).codec as c_uint == E_EXTERNAL_ENC {
            let blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_TN);
            *(*sll).block.offset(DS_TN as isize) = blk.cast::<cram_block_layout>();
            if blk.is_null() {
                return -1;
            }
            (*tn_codec).external.content_id = DS_TN;
        } else {
            *(*sll).block.offset(DS_TN as isize) = *(*sll).block.offset(0);
        }
    }
    if embed_ref > 0 {
        let blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_ENC_ref);
        *(*sll).block.offset(DS_ENC_ref as isize) = blk.cast::<cram_block_layout>();
        if blk.is_null() {
            return -1;
        }
        (*sll).ref_id = DS_ENC_ref;
        let ref_off = ((*(*sll).hdr).ref_seq_start - (*cl).ref_start) as isize;
        if cram_cram_io_h_248_block_append(
            blk,
            (*cl).ref_.offset(ref_off).cast::<c_void>(),
            (*(*sll).hdr).ref_seq_span as usize,
        ) < 0
        {
            return -1;
        }
    }
    id = DS_QS;
    while id < DS_TN {
        if cram_cram_encode_c_1006_cram_allocate_block(
            (*hl).codecs[id as usize].cast::<cram_codec>(),
            s,
            id,
        ) < 0
        {
            return -1;
        }
        id += 1;
    }
    if !(*cl).tags_used.is_null() {
        (*(*sll).hdr).num_blocks = DS_ENC_END;
        let mut n: c_int = 0;
        while n < (*sll).naux_block {
            let dst_idx = (*(*sll).hdr).num_blocks;
            (*(*sll).hdr).num_blocks += 1;
            *(*sll).block.offset(dst_idx as isize) = *(*sll).aux_block.offset(n as isize);
            *(*sll).aux_block.offset(n as isize) = std::ptr::null_mut();
            n += 1;
        }
    }
    last_pos = (*(*sll).hdr).ref_seq_start;
    let mut rec: c_int = 0;
    while rec < (*(*sll).hdr).num_records {
        let cr = (*sll).crecs.offset(rec as isize).cast::<cram_record>();
        if cram_cram_encode_c_573_cram_encode_slice_read(fd, c, h, sl, cr, &raw mut last_pos) == -1
        {
            return -1;
        }
        rec += 1;
    }
    // Finalize CORE block size (account for any trailing bit not yet at byte boundary).
    let core_l = *(*sll).block.offset(0);
    (*core_l).uncomp_size = ((*core_l).byte + if (*core_l).bit < 7 { 1 } else { 0 }) as i32;
    (*core_l).comp_size = (*core_l).uncomp_size;

    // Transfer staging blocks (base/qual/name/soft) over into slice block[].
    if !(*(*sll).block.offset(DS_IN as isize)).is_null() {
        cram_cram_io_c_1565_cram_free_block(
            (*(*sll).block.offset(DS_IN as isize)).cast::<cram_block>(),
        );
    }
    *(*sll).block.offset(DS_IN as isize) = (*sll).base_blk;
    (*sll).base_blk = std::ptr::null_mut();

    if !(*(*sll).block.offset(DS_QS as isize)).is_null() {
        cram_cram_io_c_1565_cram_free_block(
            (*(*sll).block.offset(DS_QS as isize)).cast::<cram_block>(),
        );
    }
    *(*sll).block.offset(DS_QS as isize) = (*sll).qual_blk;
    (*sll).qual_blk = std::ptr::null_mut();

    if !(*(*sll).block.offset(DS_RN as isize)).is_null() {
        cram_cram_io_c_1565_cram_free_block(
            (*(*sll).block.offset(DS_RN as isize)).cast::<cram_block>(),
        );
    }
    *(*sll).block.offset(DS_RN as isize) = (*sll).name_blk;
    (*sll).name_blk = std::ptr::null_mut();

    if !(*(*sll).block.offset(DS_SC as isize)).is_null() {
        cram_cram_io_c_1565_cram_free_block(
            (*(*sll).block.offset(DS_SC as isize)).cast::<cram_block>(),
        );
    }
    *(*sll).block.offset(DS_SC as isize) = (*sll).soft_blk;
    (*sll).soft_blk = std::ptr::null_mut();

    // Flush codecs.
    id = DS_QS;
    while id < DS_TN {
        let cd = (*hl).codecs[id as usize];
        if !cd.is_null() {
            let cv = cd.cast::<cram_codec_external_layout>();
            if !(*cv).flush.is_null() {
                cram_codec_flush(cd);
            }
        }
        id += 1;
    }

    // For non-CORE, non-null blocks past DS_aux, ensure uncomp_size matches accumulated byte count.
    id = DS_ENC_aux;
    while id < (*(*sll).hdr).num_blocks {
        let blk_ptr = *(*sll).block.offset(id as isize);
        if !blk_ptr.is_null() && blk_ptr != *(*sll).block.offset(0) && (*blk_ptr).uncomp_size == 0 {
            (*blk_ptr).uncomp_size = (*blk_ptr).byte as i32;
            (*blk_ptr).comp_size = (*blk_ptr).uncomp_size;
        }
        id += 1;
    }

    if cram_cram_encode_c_804_cram_compress_slice(fd, c, s) == -1 {
        return -1;
    }

    // Compact the block list: drop empty blocks, fill block_content_ids, update num_blocks.
    (*(*sll).hdr).block_content_ids = realloc(
        (*(*sll).hdr).block_content_ids.cast::<c_void>(),
        ((*(*sll).hdr).num_blocks as u64).wrapping_mul(std::mem::size_of::<i32>() as u64),
    )
    .cast::<i32>();
    if (*(*sll).hdr).block_content_ids.is_null() {
        return -1;
    }
    let mut j: c_int = 1;
    let mut i: c_int = j;
    while i < (*(*sll).hdr).num_blocks {
        let bp = *(*sll).block.offset(i as isize);
        if !bp.is_null() && bp != *(*sll).block.offset(0) {
            if (*bp).uncomp_size == 0 {
                cram_cram_io_c_1565_cram_free_block(bp.cast::<cram_block>());
                *(*sll).block.offset(i as isize) = std::ptr::null_mut();
            } else {
                *(*sll).block.offset(j as isize) = bp;
                *(*(*sll).hdr).block_content_ids.offset((j - 1) as isize) = (*bp).content_id;
                j += 1;
            }
        }
        i += 1;
    }
    (*(*sll).hdr).num_content_ids = j - 1;
    (*(*sll).hdr).num_blocks = j;
    (*sll).hdr_block =
        cram_cram_encode_c_512_cram_encode_slice_header(fd, sl).cast::<cram_block_layout>();
    if (*sll).hdr_block.is_null() {
        return -1;
    }
    if r != 0 {
        -1
    } else {
        0
    }
}

/// `extend_ref` (htslib/cram/cram_encode.c:1508). Grows the synthesised
/// reference buffer + per-base histogram so position `pos` is in range.
/// Returns 0 on success, -1 on alloc failure / pos<ref_start, -2 on overflow.
pub unsafe fn cram_cram_encode_c_1508_extend_ref(
    ref_0: *mut *mut c_char,
    hist: *mut *mut [u32; 5],
    pos: i64,
    ref_start: i64,
    ref_end: *mut i64,
    ref_end_alloc: *mut i64,
) -> c_int {
    if *ref_end < pos && pos < *ref_end_alloc {
        *ref_end = pos;
    }
    if pos < ref_start {
        return -1;
    }
    if pos < *ref_end_alloc {
        return 0;
    }
    if pos - ref_start > u32::MAX as i64 {
        return -2;
    }
    let old_end: i64 = if *ref_end_alloc != 0 {
        *ref_end_alloc
    } else {
        ref_start
    };
    let new_end: i64 = ((ref_start + 1000) as f64 + (pos - ref_start) as f64 * 1.5f64) as i64;
    if (new_end - ref_start) as usize
        > (u32::MAX as usize)
            .wrapping_div(std::mem::size_of::<[u32; 5]>())
            .wrapping_div(2)
    {
        return -2;
    }
    let tmp = realloc((*ref_0).cast::<c_void>(), (new_end - ref_start + 1) as u64).cast::<c_char>();
    if tmp.is_null() {
        return -1;
    }
    *ref_0 = tmp;
    let tmp5 = realloc(
        (*hist).cast::<c_void>(),
        ((new_end - ref_start) as u64).wrapping_mul(std::mem::size_of::<[u32; 5]>() as u64),
    )
    .cast::<[u32; 5]>();
    if tmp5.is_null() {
        return -1;
    }
    *hist = tmp5;
    *ref_end_alloc = new_end;
    let old_end_off = old_end - ref_start;
    let new_end_off = new_end - ref_start;
    libc::memset(
        (*ref_0).offset(old_end_off as isize).cast::<c_void>(),
        0,
        (new_end_off - old_end_off) as usize,
    );
    libc::memset(
        (*hist).offset(old_end_off as isize).cast::<c_void>(),
        0,
        ((new_end_off - old_end_off) as usize).wrapping_mul(std::mem::size_of::<[u32; 5]>()),
    );
    if *ref_end < pos {
        *ref_end = pos;
    }
    0
}

/// `cram_add_to_ref_MD` (htslib/cram/cram_encode.c:1557). Uses the BAM MD:Z
/// auxiliary tag and CIGAR to populate the synthesised reference. Returns
/// \>0 on success, 0 if MD was unusable (caller falls back to CIGAR-only),
/// -1 on hard failure.
pub unsafe fn cram_cram_encode_c_1557_cram_add_to_ref_MD(
    b: *mut bam1_t,
    ref_0: *mut *mut c_char,
    hist: *mut *mut [u32; 5],
    ref_start: i64,
    ref_end: *mut i64,
    ref_end_alloc: *mut i64,
    mut md: *const u8,
) -> c_int {
    // The 16-byte IUPAC -> "=ACMGRSVTWYHKDBN" translation table.
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

    let seq: *mut u8 = (*b)
        .data
        .add(((*b).core.n_cigar << 2) as usize)
        .add((*b).core.l_qname as usize);
    let cigar: *mut u32 = (*b).data.add((*b).core.l_qname as usize).cast::<u32>();
    let ncigar: u32 = (*b).core.n_cigar;
    let mut cig_op: u32 = 0;
    let mut cig_len: u32 = 0;
    let mut cig_ind: u32 = 0;
    let rlen: i64 = crate::htslib_rs::sam::bam_cigar2rlen((*b).core.n_cigar as c_int, cigar);
    let rseq_end: i64 = (*b).core.pos
        + if rlen != 0 {
            rlen
        } else {
            (*b).core.l_qseq as i64
        };
    if (*b).core.l_qseq == 0
        && cram_cram_encode_c_1508_extend_ref(
            ref_0,
            hist,
            rseq_end,
            ref_start,
            ref_end,
            ref_end_alloc,
        ) < 0
    {
        return -1;
    }
    let mut iseq: c_int = 0;
    let mut next_op: c_int;
    let mut iref: i64 = (*b).core.pos - ref_start;
    // BAM CIGAR ops to skip when stepping through the seq for MD interp.
    // (M=0,I=1,D=2,N=3,S=4,H=5,P=6,=7,X=8); skip=1 for I/N/S/H/P (consumes seq
    // but not ref, OR consumes neither), 0 for M/D/=/X (consumes ref).
    let mut cig_skip: [c_int; 16] = [0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1];

    while (iseq) < (*b).core.l_qseq && *md != 0 {
        // Cast through libc::isdigit
        if libc::isdigit(*md as c_int) != 0 {
            let mut overflow: c_int = 0;
            let mut len: c_int = crate::htslib_rs::hts::hts_str2uint(
                md.cast::<c_char>(),
                (&raw mut md).cast::<*mut c_char>(),
                31,
                &raw mut overflow,
            ) as c_int;
            if overflow != 0
                || cram_cram_encode_c_1508_extend_ref(
                    ref_0,
                    hist,
                    iref + ref_start + len as i64,
                    ref_start,
                    ref_end,
                    ref_end_alloc,
                ) < 0
            {
                return -1;
            }
            while iseq < (*b).core.l_qseq && len != 0 {
                next_op = cram_cram_encode_c_1476_next_cigar_op(
                    cigar,
                    ncigar,
                    cig_skip.as_mut_ptr(),
                    &raw mut iseq,
                    &raw mut cig_ind,
                    &raw mut cig_op,
                    &raw mut cig_len,
                );
                if next_op < 0 {
                    return -1;
                }
                if next_op != crate::htslib_rs::sam::BAM_CMATCH
                    && next_op != crate::htslib_rs::sam::BAM_CEQUAL
                {
                    hts_log_cstr(
                        HTS_LOG_INFO,
                        c"cram_add_to_ref_MD".as_ptr(),
                        c"MD:Z and CIGAR are incompatible for record".as_ptr(),
                    );
                    return -1;
                }
                cig_len = cig_len.wrapping_add(1);
                loop {
                    cig_len = cig_len.wrapping_sub(1);
                    let fresh = iref;
                    iref += 1;
                    *(*ref_0).offset(fresh as isize) =
                        SEQ_NT16_STR[(*seq.offset((iseq >> 1) as isize) as c_int
                            >> ((!iseq & 1) << 2)
                            & 0xf) as usize] as c_char;
                    iseq += 1;
                    len -= 1;
                    if !(cig_len != 0 && iseq < (*b).core.l_qseq && len != 0) {
                        break;
                    }
                }
            }
            if len > 0 {
                return -1;
            }
        } else if *md as c_int == b'^' as c_int {
            md = md.add(1);
            while libc::isalpha(*md as c_int) != 0 {
                if cram_cram_encode_c_1508_extend_ref(
                    ref_0,
                    hist,
                    iref + ref_start,
                    ref_start,
                    ref_end,
                    ref_end_alloc,
                ) < 0
                {
                    return -1;
                }
                next_op = cram_cram_encode_c_1476_next_cigar_op(
                    cigar,
                    ncigar,
                    cig_skip.as_mut_ptr(),
                    &raw mut iseq,
                    &raw mut cig_ind,
                    &raw mut cig_op,
                    &raw mut cig_len,
                );
                if next_op < 0 {
                    return -1;
                }
                if next_op != crate::htslib_rs::sam::BAM_CDEL {
                    hts_log_cstr(
                        HTS_LOG_INFO,
                        c"cram_add_to_ref_MD".as_ptr(),
                        c"MD:Z and CIGAR are incompatible".as_ptr(),
                    );
                    return -1;
                }
                let fresh_md = md;
                md = md.add(1);
                let fresh_iref = iref;
                iref += 1;
                *(*ref_0).offset(fresh_iref as isize) = (*fresh_md as c_int & !0x20) as c_char;
            }
        } else {
            if cram_cram_encode_c_1508_extend_ref(
                ref_0,
                hist,
                iref + ref_start,
                ref_start,
                ref_end,
                ref_end_alloc,
            ) < 0
            {
                return -1;
            }
            next_op = cram_cram_encode_c_1476_next_cigar_op(
                cigar,
                ncigar,
                cig_skip.as_mut_ptr(),
                &raw mut iseq,
                &raw mut cig_ind,
                &raw mut cig_op,
                &raw mut cig_len,
            );
            if next_op < 0 {
                return -1;
            }
            if next_op != crate::htslib_rs::sam::BAM_CMATCH
                && next_op != crate::htslib_rs::sam::BAM_CDIFF
            {
                hts_log_cstr(
                    HTS_LOG_INFO,
                    c"cram_add_to_ref_MD".as_ptr(),
                    c"MD:Z and CIGAR are incompatible".as_ptr(),
                );
                return -1;
            }
            let fresh_md = md;
            md = md.add(1);
            let fresh_iref = iref;
            iref += 1;
            *(*ref_0).offset(fresh_iref as isize) = (*fresh_md as c_int & !0x20) as c_char;
            iseq += 1;
        }
    }
    1
}

/// `cram_add_to_ref` (htslib/cram/cram_encode.c:1663). CIGAR-only fallback
/// (also consults MD tag where available). Returns >=1 on success, -1 on err.
pub unsafe fn cram_cram_encode_c_1663_cram_add_to_ref(
    b: *mut bam1_t,
    ref_0: *mut *mut c_char,
    hist: *mut *mut [u32; 5],
    ref_start: i64,
    ref_end: *mut i64,
    ref_end_alloc: *mut i64,
) -> c_int {
    let md_tag = [b'M' as c_char, b'D' as c_char, 0];
    let md: *const u8 = bam_aux_get(b, md_tag.as_ptr());
    let ret: c_int = 0;
    if !md.is_null() && *md as c_int == b'Z' as c_int {
        let ret0 = cram_cram_encode_c_1557_cram_add_to_ref_MD(
            b,
            ref_0,
            hist,
            ref_start,
            ref_end,
            ref_end_alloc,
            md.add(1),
        );
        if ret0 > 0 {
            return ret0;
        }
    }
    let cigar: *mut u32 = (*b).data.add((*b).core.l_qname as usize).cast::<u32>();
    let ncigar: u32 = (*b).core.n_cigar;
    let mut i: u32;
    let mut j: u32;
    let mut iseq: i64 = 0;
    let mut iref: i64 = (*b).core.pos - ref_start;
    let seq: *mut u8 = (*b)
        .data
        .add(((*b).core.n_cigar << 2) as usize)
        .add((*b).core.l_qname as usize);
    // BAM 4-bit code -> ACGTN index (0..4). 4 = "N" sink for ambiguous codes.
    const L16: [u8; 16] = [4, 0, 1, 4, 2, 4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4];
    i = 0;
    while i < ncigar {
        match *cigar.add(i as usize) & BAM_CIGAR_MASK {
            // S=4 (soft clip) | I=1 (insertion): consume seq, not ref.
            4 | 1 => {
                iseq += (*cigar.add(i as usize) >> BAM_CIGAR_SHIFT) as i64;
            }
            // M=0 | =7 | X=8: consume both seq and ref.
            0 | 7 | 8 => {
                let len: c_int = (*cigar.add(i as usize) >> BAM_CIGAR_SHIFT) as c_int;
                if cram_cram_encode_c_1508_extend_ref(
                    ref_0,
                    hist,
                    iref + ref_start + len as i64,
                    ref_start,
                    ref_end,
                    ref_end_alloc,
                ) < 0
                {
                    return -1;
                }
                if iseq + len as i64 <= (*b).core.l_qseq as i64 {
                    if ret < 0 {
                        libc::memset(
                            (*ref_0).offset(iref as isize).cast::<c_void>(),
                            0,
                            len as usize,
                        );
                    }
                    j = 0;
                    while j < len as u32 {
                        let base_code =
                            *seq.offset((iseq >> 1) as isize) as c_int >> ((!iseq & 1) << 2) & 0xf;
                        let h = (*(*hist).offset(iref as isize)).as_mut_ptr();
                        let idx = L16[base_code as usize] as usize;
                        *h.add(idx) = (*h.add(idx)).wrapping_add(1);
                        j = j.wrapping_add(1);
                        iref += 1;
                        iseq += 1;
                    }
                } else {
                    iseq += len as i64;
                    iref += len as i64;
                }
            }
            // D=2 | N=3: consume ref, not seq.
            2 | 3 => {
                iref += (*cigar.add(i as usize) >> BAM_CIGAR_SHIFT) as i64;
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    1
}

/// `cram_generate_reference` (htslib/cram/cram_encode.c:1737). Synthesises a
/// reference from the BAM records in a slice, writing the result into the
/// container's `ref_/ref_start/ref_end/ref_free` fields. Returns 0/-1.
pub unsafe fn cram_cram_encode_c_1737_cram_generate_reference(
    c: *mut cram_container,
    s: *mut cram_slice,
    mut r1: c_int,
) -> c_int {
    let cl = c.cast::<cram_container_layout>();
    let sl = s.cast::<cram_slice_layout>();
    let mut ref_buf: *mut c_char = std::ptr::null_mut();
    let mut hist: *mut [u32; 5] = std::ptr::null_mut();
    let ref_start: i64 = (*(*(*cl).bams.offset(r1 as isize))).core.pos;
    let mut ref_end: i64 = 0;
    let mut ref_end_alloc: i64 = 0;
    if ref_start < 0 {
        return -1;
    }
    // Pre-extend up to the last BAM's end position so we don't have to
    // realloc on every record.
    let last_idx = r1 + (*(*sl).hdr).num_records - 1;
    let last_bam = *(*cl).bams.offset(last_idx as isize);
    if cram_cram_encode_c_1508_extend_ref(
        &raw mut ref_buf,
        &raw mut hist,
        (*last_bam).core.pos + (*last_bam).core.l_qseq as i64,
        ref_start,
        &raw mut ref_end,
        &raw mut ref_end_alloc,
    ) < 0
    {
        return -1;
    }
    let mut r2: c_int = 0;
    let mut last_pos: i64 = -1;
    let mut failed = false;
    while r1 < (*cl).curr_c_rec && r2 < (*(*sl).hdr).num_records {
        if (*(*(*cl).bams.offset(r1 as isize))).core.pos < last_pos {
            hts_log_cstr(
                HTS_LOG_ERROR,
                c"cram_generate_reference".as_ptr(),
                c"Cannot build reference with unsorted data".as_ptr(),
            );
            failed = true;
            break;
        }
        last_pos = (*(*(*cl).bams.offset(r1 as isize))).core.pos;
        if cram_cram_encode_c_1663_cram_add_to_ref(
            *(*cl).bams.offset(r1 as isize),
            &raw mut ref_buf,
            &raw mut hist,
            ref_start,
            &raw mut ref_end,
            &raw mut ref_end_alloc,
        ) < 0
        {
            failed = true;
            break;
        }
        r1 += 1;
        r2 += 1;
    }
    if failed {
        free(ref_buf.cast::<c_void>());
        free(hist.cast::<c_void>());
        return -1;
    }
    // Resolve unspecified bases by majority-vote in the histogram.
    let mut i: i64 = 0;
    while i < ref_end - ref_start {
        if *ref_buf.offset(i as isize) == 0 {
            let mut max_v: c_int = 0;
            let mut max_j: c_int = 4;
            let mut j: c_int = 0;
            while j < 4 {
                if (max_v as u32) < (*hist.offset(i as isize))[j as usize] {
                    max_v = (*hist.offset(i as isize))[j as usize] as c_int;
                    max_j = j;
                }
                j += 1;
            }
            *ref_buf.offset(i as isize) = b"ACGTN"[max_j as usize] as c_char;
        }
        i += 1;
    }
    free(hist.cast::<c_void>());
    (*cl).ref_ = ref_buf;
    (*cl).ref_start = ref_start + 1;
    (*cl).ref_end = ref_end + 1;
    (*cl).ref_free = 1;
    0
}

/// `validate_md5` (htslib/cram/cram_encode.c:1798). Compares the MD5 of the
/// loaded reference against the @SQ M5 header tag, if present.
pub unsafe fn cram_cram_encode_c_1798_validate_md5(fd: *mut cram_fd, ref_id: c_int) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    if (*fdl).ignore_md5 != 0 || ref_id < 0 || ref_id >= (*(*fdl).refs).nref {
        return 0;
    }
    let entry = *(*(*fdl).refs).ref_id.offset(ref_id as isize);
    if (*entry).validated_md5 != 0 {
        return 0;
    }
    let hdr = (*fdl).header;
    let hrecs = (*hdr).hrecs;
    let ref_name = (*(*hrecs).ref_.offset(ref_id as isize)).name;
    let sq_tag = [b'S' as c_char, b'Q' as c_char, 0];
    let sn_tag = [b'S' as c_char, b'N' as c_char, 0];
    let ty = crate::htslib_rs::sam::sam_hrecs_find_type_id(
        hrecs,
        sq_tag.as_ptr(),
        sn_tag.as_ptr(),
        ref_name,
    );
    if ty.is_null() {
        return 0;
    }
    let m5_tag = [b'M' as c_char, b'5' as c_char, 0];
    let m5tag =
        crate::htslib_rs::sam::sam_hrecs_find_key(ty, m5_tag.as_ptr(), std::ptr::null_mut());
    if m5tag.is_null() {
        return 0;
    }
    let ref_seq = (*entry).seq;
    let len = (*entry).length;
    let md5 = crate::htslib_rs::md5::hts_md5_init();
    if md5.is_null() {
        return -1;
    }
    let mut buf: [c_uchar; 16] = [0; 16];
    let mut buf2: [c_char; 33] = [0; 33];
    crate::htslib_rs::md5::hts_md5_update(md5, ref_seq.cast::<c_void>(), len as std::ffi::c_ulong);
    crate::htslib_rs::md5::hts_md5_final(buf.as_mut_ptr(), md5);
    crate::htslib_rs::md5::hts_md5_destroy(md5);
    crate::htslib_rs::md5::hts_md5_hex(buf2.as_mut_ptr(), buf.as_ptr());
    // (*m5tag).str_ points at "M5:<hex>"; skip the 3-byte "M5:" prefix.
    if libc::strcmp((*m5tag).str_.add(3), buf2.as_ptr()) != 0 {
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"validate_md5".as_ptr(),
            c"SQ header M5 tag discrepancy for reference".as_ptr(),
        );
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"validate_md5".as_ptr(),
            c"Please use the correct reference, or consider using embed_ref=2".as_ptr(),
        );
        return -1;
    }
    (*entry).validated_md5 = 1;
    0
}

/// `lossy_read_names` (htslib/cram/cram_encode.c:1344). When fd->lossy_read_names
/// is enabled, marks records whose names appear `expected_template_count`
/// times within the slice with CRAM_FLAG_DISCARD_NAME so the decoder can
/// regenerate names from the template grouping. Returns 0 on success, -1
/// on hash failure.
pub unsafe fn cram_cram_encode_c_1344_lossy_read_names(
    fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
    bam_start: c_int,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let sl = s.cast::<cram_slice_layout>();

    // Initialise cram_flags on every record.
    let mut r2: c_int = 0;
    while r2 < (*(*sl).hdr).num_records {
        (*(*sl).crecs.offset(r2 as isize)).cram_flags = 0;
        r2 += 1;
    }
    if (*fdl).lossy_read_names == 0 {
        return 0;
    }

    // Allocate a kh_m_s2u64 (FNV1a-hashed const-char* -> uint64_t).
    let names =
        calloc(1, std::mem::size_of::<kh_m_s2u64_layout>() as u64).cast::<kh_m_s2u64_layout>();
    if names.is_null() {
        return -1;
    }
    let mut ret: c_int = -1;
    let mut hash_fail = false;

    // Pass 1: count name occurrences vs the expected template count.
    let mut r1: c_int = bam_start;
    r2 = 0;
    while r2 < (*(*sl).hdr).num_records {
        let b = *(*cl).bams.offset(r1 as isize);
        let mut n: c_int = 0;
        let e: u64 = cram_cram_encode_c_1301_expected_template_count(b) as u64;
        // Pack u.counts.e (low 32 bits) | u.counts.c (high 32 bits).
        let u_initial: u64 = (e as u32 as u64) | ((1u32 as u64) << 32);
        let k = kh_put_m_s2u64(names, (*b).data.cast::<c_char>(), &raw mut n);
        if n == -1 {
            hash_fail = true;
            break;
        }
        if n == 0 {
            // Existing key: read packed counts, update.
            let cur = *(*names).vals.offset(k as isize);
            let cur_e = cur as u32 as i32;
            let mut cur_c = (cur >> 32) as u32 as i32;
            if cur_e as u64 != e {
                *(*names).vals.offset(k as isize) = 0;
            } else {
                cur_c += 1;
                if cur_e == cur_c {
                    *(*names).vals.offset(k as isize) = u64::MAX;
                } else {
                    *(*names).vals.offset(k as isize) =
                        (cur_e as u32 as u64) | ((cur_c as u32 as u64) << 32);
                }
            }
        } else {
            *(*names).vals.offset(k as isize) = u_initial;
        }
        r1 += 1;
        r2 += 1;
    }

    // Pass 2: any name whose total reached expected -> mark DISCARD_NAME.
    if !hash_fail {
        r1 = bam_start;
        r2 = 0;
        let mut ok = true;
        while r2 < (*(*sl).hdr).num_records {
            let cr = (*sl).crecs.offset(r2 as isize);
            let b = *(*cl).bams.offset(r1 as isize);
            let k = kh_get_m_s2u64(names, (*b).data.cast::<c_char>());
            if k == (*names).n_buckets {
                ok = false;
                break;
            }
            if *(*names).vals.offset(k as isize) == u64::MAX {
                (*cr).cram_flags = CRAM_FLAG_DISCARD_NAME_ENC;
            }
            r1 += 1;
            r2 += 1;
        }
        if ok {
            ret = 0;
        }
    }

    kh_destroy_m_s2u64(names);
    ret
}

/// `add_read_names` (htslib/cram/cram_encode.c:1437). Populates the slice's
/// name block with the BAM read names (skipping discarded names per
/// `lossy_read_names`), and updates per-record name/name_len + DS_RN stats.
pub unsafe fn cram_cram_encode_c_1437_add_read_names(
    fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
    bam_start: c_int,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let sl = s.cast::<cram_slice_layout>();
    let keep_names: c_int = ((*fdl).lossy_read_names == 0) as c_int;
    let mut r1: c_int = bam_start;
    let mut r2: c_int = 0;
    while r1 < (*cl).curr_c_rec && r2 < (*(*sl).hdr).num_records {
        let cr = (*sl).crecs.offset(r2 as isize);
        let b = *(*cl).bams.offset(r1 as isize);
        (*cr).name = (*(*sl).name_blk).byte as i32;
        if (*cr).cram_flags & CRAM_FLAG_DETACHED_ENC != 0 || keep_names != 0 {
            if (*fdl).version >> 8 >= 4
                && (*cr).cram_flags & CRAM_FLAG_MATE_DOWNSTREAM_ENC != 0
                && (*cr).mate_line != 0
            {
                // Emit a single NUL byte as the placeholder name.
                let nul: u8 = 0;
                if cram_cram_io_h_248_block_append(
                    (*sl).name_blk.cast::<cram_block>(),
                    (&raw const nul).cast::<c_void>(),
                    1,
                ) < 0
                {
                    return -1;
                }
                (*cr).name_len = 1;
            } else {
                let nlen = (*b).core.l_qname as c_int - (*b).core.l_extranul as c_int;
                if cram_cram_io_h_248_block_append(
                    (*sl).name_blk.cast::<cram_block>(),
                    (*b).data.cast::<c_void>(),
                    nlen as usize,
                ) < 0
                {
                    return -1;
                }
                (*cr).name_len = nlen;
            }
        } else {
            (*cr).name_len = 0;
        }
        // Production `cram_stats_add` returns `()`. Matches mirror's behaviour
        // when shrunk to c_int: stats add never fails in the native impl.
        cram_cram_stats_c_52_cram_stats_add(
            (*cl).stats[DS_RN as usize].cast::<c_void>(),
            (*cr).name_len,
        );
        r1 += 1;
        r2 += 1;
    }
    0
}

/// `cram_add_feature` (htslib/cram/cram_encode.c:2578). Appends `f` to the
/// slice's feature vector (grows `features` via realloc, doubling `afeatures`
/// from 0 to 1024 on first insert) and updates the per-record FP delta and
/// FC code stats. Returns 0 on success, -1 on realloc failure.
pub(crate) unsafe fn cram_cram_encode_c_2578_cram_add_feature(
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    f: *mut cram_feature_layout,
) -> c_int {
    let cl = c.cast::<cram_container_layout>();
    let sl = s.cast::<cram_slice_layout>();
    if (*sl).nfeatures >= (*sl).afeatures {
        (*sl).afeatures = if (*sl).afeatures != 0 {
            (*sl).afeatures.wrapping_mul(2)
        } else {
            1024
        };
        (*sl).features = realloc(
            (*sl).features.cast::<c_void>(),
            ((*sl).afeatures as u64)
                .wrapping_mul(std::mem::size_of::<cram_feature_layout>() as u64),
        )
        .cast::<cram_feature_layout>();
        if (*sl).features.is_null() {
            return -1;
        }
    }
    let fx = f.cast::<cram_feature_X_layout>();
    let fresh163 = (*r).nfeature;
    (*r).nfeature = (*r).nfeature.wrapping_add(1);
    if fresh163 == 0 {
        (*r).feature = (*sl).nfeatures;
        cram_cram_stats_c_52_cram_stats_add(
            (*cl).stats[DS_FP_ENC as usize].cast::<c_void>(),
            (*fx).pos,
        );
    } else {
        let prev = (*sl)
            .features
            .offset((*r).feature.wrapping_add((*r).nfeature).wrapping_sub(2) as isize)
            .cast::<cram_feature_X_layout>();
        cram_cram_stats_c_52_cram_stats_add(
            (*cl).stats[DS_FP_ENC as usize].cast::<c_void>(),
            (*fx).pos - (*prev).pos,
        );
    }
    cram_cram_stats_c_52_cram_stats_add(
        (*cl).stats[DS_FC_ENC as usize].cast::<c_void>(),
        (*fx).code,
    );
    let fresh164 = (*sl).nfeatures;
    (*sl).nfeatures = (*sl).nfeatures.wrapping_add(1);
    *(*sl).features.offset(fresh164 as isize) = *f;
    0
}

/// `cram_add_substitution` (htslib/cram/cram_encode.c:2605). Records either an
/// 'X' substitution (when both read base and ref base are known A/C/G/T(/N))
/// or a 'B' (base+qual) feature, then appends it via `cram_add_feature`.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn cram_cram_encode_c_2605_cram_add_substitution(
    fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    pos: c_int,
    base: c_char,
    qual: c_char,
    ref_0: c_char,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let sl = s.cast::<cram_slice_layout>();
    let mut f = cram_feature_layout { fields: [0; 4] };
    let base_idx = (base as c_uchar) as usize;
    let ref_idx = (ref_0 as c_uchar) as usize;
    if ((*fdl).l2[base_idx] as c_int) < 4
        || ((*fdl).l2[base_idx] as c_int) < 5 && ((*fdl).l2[ref_idx] as c_int) < 4
    {
        let fx = (&raw mut f).cast::<cram_feature_X_layout>();
        (*fx).pos = pos + 1;
        (*fx).code = 'X' as c_int;
        (*fx).base = (*fdl).cram_sub_matrix[(ref_0 as c_int & 0x1f) as usize]
            [(base as c_int & 0x1f) as usize] as c_int;
        cram_cram_stats_c_52_cram_stats_add(
            (*c.cast::<cram_container_layout>()).stats[DS_BS_ENC as usize].cast::<c_void>(),
            (*fx).base,
        );
    } else {
        let fb = (&raw mut f).cast::<cram_feature_B_layout>();
        (*fb).pos = pos + 1;
        (*fb).code = 'B' as c_int;
        (*fb).base = base as c_int;
        (*fb).qual = qual as c_int;
        cram_cram_stats_c_52_cram_stats_add(
            (*c.cast::<cram_container_layout>()).stats[DS_BA_ENC as usize].cast::<c_void>(),
            (*fb).base,
        );
        cram_cram_stats_c_52_cram_stats_add(
            (*c.cast::<cram_container_layout>()).stats[DS_QS as usize].cast::<c_void>(),
            (*fb).qual,
        );
        if cram_cram_io_h_261_block_append_char((*sl).qual_blk.cast::<cram_block>(), qual) < 0 {
            return -1;
        }
    }
    cram_cram_encode_c_2578_cram_add_feature(c, s, r, &raw mut f)
}

/// `cram_add_bases` (htslib/cram/cram_encode.c:2632). Builds a 'b' feature
/// referencing a span of `seqs_blk` and appends it.
pub(crate) unsafe fn cram_cram_encode_c_2632_cram_add_bases(
    _fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    pos: c_int,
    len: c_int,
    base: *mut c_char,
) -> c_int {
    let sl = s.cast::<cram_slice_layout>();
    let mut f = cram_feature_layout { fields: [0; 4] };
    let fbb = (&raw mut f).cast::<cram_feature_b_layout>();
    (*fbb).pos = pos + 1;
    (*fbb).code = 'b' as c_int;
    (*fbb).seq_idx = base.offset_from((*(*sl).seqs_blk).data.cast::<c_char>()) as c_int;
    (*fbb).len = len;
    cram_cram_encode_c_2578_cram_add_feature(c, s, r, &raw mut f)
}

/// `cram_add_base` (htslib/cram/cram_encode.c:2645). Records a single 'B'
/// base+qual feature, updating DS_BA/DS_QS stats and qual_blk.
pub(crate) unsafe fn cram_cram_encode_c_2645_cram_add_base(
    _fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    pos: c_int,
    base: c_char,
    qual: c_char,
) -> c_int {
    let sl = s.cast::<cram_slice_layout>();
    let cl = c.cast::<cram_container_layout>();
    let mut f = cram_feature_layout { fields: [0; 4] };
    let fb = (&raw mut f).cast::<cram_feature_B_layout>();
    (*fb).pos = pos + 1;
    (*fb).code = 'B' as c_int;
    (*fb).base = base as c_int;
    (*fb).qual = qual as c_int;
    cram_cram_stats_c_52_cram_stats_add(
        (*cl).stats[DS_BA_ENC as usize].cast::<c_void>(),
        base as c_int,
    );
    cram_cram_stats_c_52_cram_stats_add(
        (*cl).stats[DS_QS as usize].cast::<c_void>(),
        qual as c_int,
    );
    if cram_cram_io_h_261_block_append_char((*sl).qual_blk.cast::<cram_block>(), qual) < 0 {
        return -1;
    }
    cram_cram_encode_c_2578_cram_add_feature(c, s, r, &raw mut f)
}

/// `cram_add_quality` (htslib/cram/cram_encode.c:2662). Records a 'Q' quality
/// feature and appends the qual byte to `qual_blk`.
pub(crate) unsafe fn cram_cram_encode_c_2662_cram_add_quality(
    _fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    pos: c_int,
    qual: c_char,
) -> c_int {
    let sl = s.cast::<cram_slice_layout>();
    let cl = c.cast::<cram_container_layout>();
    let mut f = cram_feature_layout { fields: [0; 4] };
    let fq = (&raw mut f).cast::<cram_feature_Q_layout>();
    (*fq).pos = pos + 1;
    (*fq).code = 'Q' as c_int;
    (*fq).qual = qual as c_int;
    cram_cram_stats_c_52_cram_stats_add(
        (*cl).stats[DS_QS as usize].cast::<c_void>(),
        qual as c_int,
    );
    if cram_cram_io_h_261_block_append_char((*sl).qual_blk.cast::<cram_block>(), qual) < 0 {
        return -1;
    }
    cram_cram_encode_c_2578_cram_add_feature(c, s, r, &raw mut f)
}

/// `cram_add_deletion` (htslib/cram/cram_encode.c:2677). Records a 'D'
/// deletion feature and updates DS_DL.
pub(crate) unsafe fn cram_cram_encode_c_2677_cram_add_deletion(
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    pos: c_int,
    len: c_int,
    _base: *mut c_char,
) -> c_int {
    let cl = c.cast::<cram_container_layout>();
    let mut f = cram_feature_layout { fields: [0; 4] };
    let fd_ = (&raw mut f).cast::<cram_feature_D_layout>();
    (*fd_).pos = pos + 1;
    (*fd_).code = 'D' as c_int;
    (*fd_).len = len;
    cram_cram_stats_c_52_cram_stats_add((*cl).stats[DS_DL_ENC as usize].cast::<c_void>(), len);
    cram_cram_encode_c_2578_cram_add_feature(c, s, r, &raw mut f)
}

/// `cram_add_softclip` (htslib/cram/cram_encode.c:2687). Records an 'S'
/// soft-clip feature; v1 stores bases in `base_blk` with a NUL terminator,
/// v2+ stores them in `soft_blk` (or 'N' fill if `base` is null).
pub(crate) unsafe fn cram_cram_encode_c_2687_cram_add_softclip(
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    pos: c_int,
    len: c_int,
    base: *mut c_char,
    version: c_int,
) -> c_int {
    let sl = s.cast::<cram_slice_layout>();
    let mut f = cram_feature_layout { fields: [0; 4] };
    let fs = (&raw mut f).cast::<cram_feature_S_layout>();
    (*fs).pos = pos + 1;
    (*fs).code = 'S' as c_int;
    (*fs).len = len;
    let ok: bool = match version >> 8 {
        1 => {
            (*fs).seq_idx = (*(*sl).base_blk).byte as c_int;
            cram_cram_io_h_248_block_append(
                (*sl).base_blk.cast::<cram_block>(),
                base.cast::<c_void>(),
                len as usize,
            ) >= 0
                && cram_cram_io_h_261_block_append_char((*sl).base_blk.cast::<cram_block>(), 0) >= 0
        }
        _ => {
            (*fs).seq_idx = (*(*sl).soft_blk).byte as c_int;
            let mut inner_ok = true;
            if !base.is_null() {
                if cram_cram_io_h_248_block_append(
                    (*sl).soft_blk.cast::<cram_block>(),
                    base.cast::<c_void>(),
                    len as usize,
                ) < 0
                {
                    inner_ok = false;
                }
            } else {
                let mut i: c_int = 0;
                while i < len {
                    if cram_cram_io_h_261_block_append_char(
                        (*sl).soft_blk.cast::<cram_block>(),
                        'N' as c_char,
                    ) < 0
                    {
                        inner_ok = false;
                        break;
                    }
                    i += 1;
                }
            }
            inner_ok
                && cram_cram_io_h_261_block_append_char((*sl).soft_blk.cast::<cram_block>(), 0) >= 0
        }
    };
    if ok {
        cram_cram_encode_c_2578_cram_add_feature(c, s, r, &raw mut f)
    } else {
        -1
    }
}

/// `cram_add_hardclip` (htslib/cram/cram_encode.c:2723). Records an 'H'
/// hard-clip feature (uses S-shape layout: pos/code/len) and updates DS_HC.
pub(crate) unsafe fn cram_cram_encode_c_2723_cram_add_hardclip(
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    pos: c_int,
    len: c_int,
    _base: *mut c_char,
) -> c_int {
    let cl = c.cast::<cram_container_layout>();
    let mut f = cram_feature_layout { fields: [0; 4] };
    let fs = (&raw mut f).cast::<cram_feature_S_layout>();
    (*fs).pos = pos + 1;
    (*fs).code = 'H' as c_int;
    (*fs).len = len;
    cram_cram_stats_c_52_cram_stats_add((*cl).stats[DS_HC_ENC as usize].cast::<c_void>(), len);
    cram_cram_encode_c_2578_cram_add_feature(c, s, r, &raw mut f)
}

/// `cram_add_skip` (htslib/cram/cram_encode.c:2733). Records an 'N' ref-skip
/// feature (uses S-shape layout: pos/code/len) and updates DS_RS.
pub(crate) unsafe fn cram_cram_encode_c_2733_cram_add_skip(
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    pos: c_int,
    len: c_int,
    _base: *mut c_char,
) -> c_int {
    let cl = c.cast::<cram_container_layout>();
    let mut f = cram_feature_layout { fields: [0; 4] };
    let fs = (&raw mut f).cast::<cram_feature_S_layout>();
    (*fs).pos = pos + 1;
    (*fs).code = 'N' as c_int;
    (*fs).len = len;
    cram_cram_stats_c_52_cram_stats_add((*cl).stats[DS_RS_ENC as usize].cast::<c_void>(), len);
    cram_cram_encode_c_2578_cram_add_feature(c, s, r, &raw mut f)
}

/// `cram_add_pad` (htslib/cram/cram_encode.c:2743). Records a 'P' padding
/// feature (uses S-shape layout: pos/code/len) and updates DS_PD.
pub(crate) unsafe fn cram_cram_encode_c_2743_cram_add_pad(
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    pos: c_int,
    len: c_int,
    _base: *mut c_char,
) -> c_int {
    let cl = c.cast::<cram_container_layout>();
    let mut f = cram_feature_layout { fields: [0; 4] };
    let fs = (&raw mut f).cast::<cram_feature_S_layout>();
    (*fs).pos = pos + 1;
    (*fs).code = 'P' as c_int;
    (*fs).len = len;
    cram_cram_stats_c_52_cram_stats_add((*cl).stats[DS_PD_ENC as usize].cast::<c_void>(), len);
    cram_cram_encode_c_2578_cram_add_feature(c, s, r, &raw mut f)
}

/// `cram_add_insertion` (htslib/cram/cram_encode.c:2753). Length 1 produces
/// an 'i' single-base insert (DS_BA stats updated). Longer inserts produce
/// an 'I' feature whose bases are appended to `base_blk` (with a NUL
/// terminator), or 'N' fill if `base` is null.
pub(crate) unsafe fn cram_cram_encode_c_2753_cram_add_insertion(
    c: *mut cram_container,
    s: *mut cram_slice,
    r: *mut cram_record_layout,
    pos: c_int,
    len: c_int,
    base: *mut c_char,
) -> c_int {
    let cl = c.cast::<cram_container_layout>();
    let sl = s.cast::<cram_slice_layout>();
    let mut f = cram_feature_layout { fields: [0; 4] };
    // Both the i and the I/S layouts share the (pos, code, ...) prefix; set
    // pos first via the X layout (pos at offset 0).
    {
        let fx = (&raw mut f).cast::<cram_feature_X_layout>();
        (*fx).pos = pos + 1;
    }
    if len == 1 {
        let b: c_char = if !base.is_null() {
            *base
        } else {
            'N' as c_char
        };
        let fi = (&raw mut f).cast::<cram_feature_i_layout>();
        (*fi).code = 'i' as c_int;
        (*fi).base = b as c_int;
        cram_cram_stats_c_52_cram_stats_add(
            (*cl).stats[DS_BA_ENC as usize].cast::<c_void>(),
            b as c_int,
        );
    } else {
        let fs = (&raw mut f).cast::<cram_feature_S_layout>();
        (*fs).code = 'I' as c_int;
        (*fs).len = len;
        (*fs).seq_idx = (*(*sl).base_blk).byte as c_int;
        let mut ok = true;
        if !base.is_null() {
            if cram_cram_io_h_248_block_append(
                (*sl).base_blk.cast::<cram_block>(),
                base.cast::<c_void>(),
                len as usize,
            ) < 0
            {
                ok = false;
            }
        } else {
            let mut i: c_int = 0;
            while i < len {
                if cram_cram_io_h_261_block_append_char(
                    (*sl).base_blk.cast::<cram_block>(),
                    'N' as c_char,
                ) < 0
                {
                    ok = false;
                    break;
                }
                i += 1;
            }
        }
        if ok && cram_cram_io_h_261_block_append_char((*sl).base_blk.cast::<cram_block>(), 0) < 0 {
            ok = false;
        }
        if !ok {
            return -1;
        }
    }
    cram_cram_encode_c_2578_cram_add_feature(c, s, r, &raw mut f)
}

/// `cram_encode_aux` (htslib/cram/cram_encode.c:2788). Encodes the auxiliary
/// tag stream of a single BAM record into the slice/container CRAM aux
/// blocks, populating the per-container `tags_used` map (creating cram_tag_map
/// codecs on first sighting) and the comp-hdr's TD hash. Returns the RG
/// sam_hrec_rg_t pointer derived from the read's RG:Z tag, or NULL on failure
/// / no RG present. Sets *err non-zero on failure.
#[allow(clippy::too_many_arguments)]
pub unsafe fn cram_cram_encode_c_2788_cram_encode_aux(
    fd: *mut cram_fd,
    b: *mut bam1_t,
    c: *mut cram_container,
    s: *mut cram_slice,
    cr: *mut cram_record,
    verbatim_NM: c_int,
    verbatim_MD: c_int,
    NM: c_int,
    MD: *mut kstring_t,
    cf_tag: c_int,
    no_ref: c_int,
    err: *mut c_int,
) -> *mut crate::htslib_rs::sam::sam_hrec_rg_t {
    use crate::htslib_rs::sam::{sam_hrec_rg_t, BAM_FUNMAP};

    const E_NULL: c_int = 0;
    const E_HUFFMAN: c_int = 3;
    const E_BYTE_ARRAY_LEN: c_int = 4;
    const E_BYTE_ARRAY_STOP: c_int = 5;
    const E_EXTERNAL: c_int = 1;
    const E_VARINT_UNSIGNED: c_int = 41;
    const E_CONST_INT: c_int = 44;
    const E_XDELTA: c_int = 53;
    const E_INT: c_int = 1;
    const E_BYTE_ARRAY: c_int = 4;
    const DS_TL_LOCAL: c_int = 32;

    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let sl = s.cast::<cram_slice_layout>();
    let crl = cr.cast::<cram_record_layout>();

    let mut aux: *mut c_char;
    let mut orig: *mut c_char;
    let mut brg: *mut sam_hrec_rg_t = std::ptr::null_mut();
    let mut aux_size: c_int = ((*b).l_data as u32)
        .wrapping_sub((*b).core.n_cigar << 2)
        .wrapping_sub((*b).core.l_qname as u32)
        .wrapping_sub((*b).core.l_qseq as u32)
        .wrapping_sub((((*b).core.l_qseq + 1) >> 1) as u32) as c_int;
    let mut aux_end: *const c_char = cram_cram_encode_c_1246_bam_data_end(b);
    let td_b: *mut cram_block = (*(*cl).comp_hdr).td_blk.cast();
    let TD_blk_size: c_int = (*(td_b.cast::<cram_block_layout>())).byte as c_int;
    let mut new_: c_int = 0;
    let key_ptr: *mut c_char;
    let mut k: u32;

    if !err.is_null() {
        *err = 1;
    }

    aux = (*b)
        .data
        .add(((*b).core.n_cigar << 2) as usize)
        .add((*b).core.l_qname as usize)
        .add((((*b).core.l_qseq + 1) >> 1) as usize)
        .add((*b).core.l_qseq as usize) as *mut c_char;
    orig = aux;

    if cf_tag != 0 && ((*fdl).version >> 8) < 4 {
        aux = malloc((aux_size + 4) as u64).cast::<c_char>();
        if aux.is_null() {
            return std::ptr::null_mut();
        }
        memcpy(aux.cast(), orig.cast(), aux_size as u64);
        let fresh151 = aux_size;
        aux_size += 1;
        *aux.offset(fresh151 as isize) = b'c' as c_char;
        let fresh152 = aux_size;
        aux_size += 1;
        *aux.offset(fresh152 as isize) = b'F' as c_char;
        let fresh153 = aux_size;
        aux_size += 1;
        *aux.offset(fresh153 as isize) = b'C' as c_char;
        let fresh154 = aux_size;
        aux_size += 1;
        *aux.offset(fresh154 as isize) = cf_tag as c_char;
        orig = aux;
        aux_end = aux.add(aux_size as usize);
    }

    let current_block: u64;
    loop {
        if !(aux_end.offset_from(aux) >= 1 && *aux.offset(0) as c_int != 0) {
            current_block = 13391418783698890455;
            break;
        }
        let mut r: c_int = 0;
        if aux.offset_from(orig) >= (aux_size - 3) as isize {
            current_block = 9865445363914956224;
            break;
        }
        // RG:Z
        if *aux.offset(0) as c_int == b'R' as c_int
            && *aux.offset(1) as c_int == b'G' as c_int
            && *aux.offset(2) as c_int == b'Z' as c_int
        {
            let rg: *mut c_char = aux.offset(3);
            aux = rg;
            while aux < aux_end as *mut c_char && {
                let fresh155 = aux;
                aux = aux.offset(1);
                *fresh155 as c_int != 0
            } {}
            if std::ptr::eq(aux, aux_end as *mut c_char)
                && *aux.offset(-1) as c_int != b'\0' as c_int
            {
                hts_log_cstr(
                    HTS_LOG_ERROR,
                    c"cram_encode_aux".as_ptr(),
                    c"Unterminated RG:Z tag".as_ptr(),
                );
                current_block = 9865445363914956224;
                break;
            } else {
                brg = crate::htslib_rs::sam::sam_hrecs_find_rg((*(*fdl).header).hrecs, rg);
                if !brg.is_null() {
                    if (*fdl).version >> 8 < 4 {
                        continue;
                    }
                    if cram_cram_io_h_248_block_append(td_b, c"RG*".as_ptr().cast(), 3) < 0 {
                        current_block = 9865445363914956224;
                        break;
                    } else {
                        continue;
                    }
                } else {
                    hts_log_cstr(
                        HTS_LOG_WARNING,
                        c"cram_encode_aux".as_ptr(),
                        c"Missing @RG header for RG tag".as_ptr(),
                    );
                    aux = rg.offset(-3);
                }
            }
        }
        // MD:Z
        if *aux.offset(0) as c_int == b'M' as c_int
            && *aux.offset(1) as c_int == b'D' as c_int
            && *aux.offset(2) as c_int == b'Z' as c_int
            && (*crl).len != 0
            && no_ref == 0
            && (*crl).flags & BAM_FUNMAP == 0
            && verbatim_MD == 0
            && !MD.is_null()
            && !(*MD).s.is_null()
            && libc::strncasecmp(
                (*MD).s,
                aux.offset(3),
                orig.offset(aux_size as isize).offset_from(aux.offset(3)) as usize,
            ) == 0
        {
            while aux < aux_end as *mut c_char && {
                let fresh156 = aux;
                aux = aux.offset(1);
                *fresh156 as c_int != 0
            } {}
            if std::ptr::eq(aux, aux_end as *mut c_char)
                && *aux.offset(-1) as c_int != b'\0' as c_int
            {
                hts_log_cstr(
                    HTS_LOG_ERROR,
                    c"cram_encode_aux".as_ptr(),
                    c"Unterminated MD:Z tag".as_ptr(),
                );
                current_block = 9865445363914956224;
                break;
            } else {
                if (*fdl).version >> 8 < 4 {
                    continue;
                }
                if cram_cram_io_h_248_block_append(td_b, c"MD*".as_ptr().cast(), 3) < 0 {
                    current_block = 9865445363914956224;
                    break;
                } else {
                    continue;
                }
            }
        }
        // NM:i
        if *aux.offset(0) as c_int == b'N' as c_int
            && *aux.offset(1) as c_int == b'M' as c_int
            && (*crl).len != 0
            && no_ref == 0
            && (*crl).flags & BAM_FUNMAP == 0
            && verbatim_NM == 0
        {
            let NM_: c_int = cram_cram_encode_c_1253_bam_aux2i_end(
                (aux as *mut u8).offset(2),
                aux_end as *mut u8,
            );
            if NM_ == NM {
                match *aux.offset(2) as c_int {
                    65 | 67 | 99 => {
                        aux = aux.offset(4);
                    }
                    83 | 115 => {
                        aux = aux.offset(5);
                    }
                    73 | 105 | 102 => {
                        aux = aux.offset(7);
                    }
                    _ => {
                        hts_log_cstr(
                            HTS_LOG_ERROR,
                            c"cram_encode_aux".as_ptr(),
                            c"Unhandled type code for NM tag".as_ptr(),
                        );
                        current_block = 9865445363914956224;
                        break;
                    }
                }
                if (*fdl).version >> 8 < 4 {
                    continue;
                }
                if cram_cram_io_h_248_block_append(td_b, c"NM*".as_ptr().cast(), 3) < 0 {
                    current_block = 9865445363914956224;
                    break;
                } else {
                    continue;
                }
            }
        }
        if cram_cram_io_h_248_block_append(td_b, aux as *const c_void, 3) < 0 {
            current_block = 9865445363914956224;
            break;
        }
        let key_0: c_int = ((*(aux as *mut c_uchar).offset(0) as c_int) << 16)
            | ((*(aux as *mut c_uchar).offset(1) as c_int) << 8)
            | (*(aux as *mut c_uchar).offset(2) as c_int);
        let tagmap = (*cl).tags_used.cast::<kh_m_tagmap_layout>();
        k = kh_put_m_tagmap(tagmap, key_0 as u32, &raw mut r);
        if -1 == r {
            current_block = 9865445363914956224;
            break;
        }
        if r != 0 {
            *(*tagmap).vals.offset(k as isize) = std::ptr::null_mut();
        }
        let k_global: u32;
        if r == 1 {
            crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).metrics_lock);
            let metrics_h = (*fdl).tags_used.cast::<kh_m_metrics_layout>();
            k_global = kh_put_m_metrics(metrics_h, key_0 as u32, &raw mut r);
            if -1 == r {
                crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).metrics_lock);
                current_block = 9865445363914956224;
                break;
            } else {
                if r >= 1 {
                    *(*metrics_h).vals.offset(k_global as isize) =
                        cram_cram_io_c_2327_cram_new_metrics().cast();
                    if (*(*metrics_h).vals.offset(k_global as isize)).is_null() {
                        kh_del_m_metrics(metrics_h, k_global);
                        crate::htslib_rs::c_compat::pthread_mutex_unlock(
                            &raw mut (*fdl).metrics_lock,
                        );
                        current_block = 9865445363914956224;
                        break;
                    }
                }
                crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).metrics_lock);
                let mut i2: [c_int; 2] = [b'\t' as c_int, key_0];
                let sk: usize = key_0 as usize;
                let m: *mut cram_tag_map_layout =
                    calloc(1, std::mem::size_of::<cram_tag_map_layout>() as u64)
                        .cast::<cram_tag_map_layout>();
                if m.is_null() {
                    current_block = 9865445363914956224;
                    break;
                }
                *(*tagmap).vals.offset(k as isize) = m;
                let c_0: *mut cram_codec;
                match *aux.offset(2) as c_int {
                    90 | 72 => {
                        c_0 = cram_cram_codecs_c_3928_cram_encoder_init(
                            E_BYTE_ARRAY_STOP,
                            std::ptr::null_mut(),
                            E_BYTE_ARRAY,
                            (&raw mut i2 as *mut c_int).cast(),
                            (*fdl).version,
                            (&raw mut (*fdl).vv).cast(),
                        )
                        .cast();
                    }
                    65 | 99 | 67 => {
                        let mut e = cram_byte_array_len_encoder_dat_layout {
                            len_encoding: E_NULL,
                            val_encoding: E_NULL,
                            len_dat: std::ptr::null_mut(),
                            val_dat: std::ptr::null_mut(),
                            len_codec: std::ptr::null_mut(),
                            val_codec: std::ptr::null_mut(),
                        };
                        let mut st = cram_stats_layout {
                            freqs: [0; 1024],
                            h: std::ptr::null_mut(),
                            nsamp: 0,
                            nvals: 0,
                            min_val: 0,
                            max_val: 0,
                        };
                        if (*fdl).version >> 8 <= 3 {
                            e.len_encoding = E_HUFFMAN;
                            e.len_dat = std::ptr::null_mut();
                        } else {
                            e.len_encoding = E_CONST_INT;
                            e.len_dat = std::ptr::null_mut();
                        }
                        libc::memset(
                            (&raw mut st).cast(),
                            0,
                            std::mem::size_of::<cram_stats_layout>(),
                        );
                        cram_cram_stats_c_52_cram_stats_add((&raw mut st).cast(), 1);
                        cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (&raw mut st).cast());
                        e.val_encoding = E_EXTERNAL;
                        e.val_dat = sk as *mut c_void;
                        c_0 = cram_cram_codecs_c_3928_cram_encoder_init(
                            E_BYTE_ARRAY_LEN,
                            (&raw mut st).cast(),
                            E_BYTE_ARRAY,
                            (&raw mut e).cast(),
                            (*fdl).version,
                            (&raw mut (*fdl).vv).cast(),
                        )
                        .cast();
                    }
                    115 | 83 => {
                        let mut e_0 = cram_byte_array_len_encoder_dat_layout {
                            len_encoding: E_NULL,
                            val_encoding: E_NULL,
                            len_dat: std::ptr::null_mut(),
                            val_dat: std::ptr::null_mut(),
                            len_codec: std::ptr::null_mut(),
                            val_codec: std::ptr::null_mut(),
                        };
                        let mut st_0 = cram_stats_layout {
                            freqs: [0; 1024],
                            h: std::ptr::null_mut(),
                            nsamp: 0,
                            nvals: 0,
                            min_val: 0,
                            max_val: 0,
                        };
                        if (*fdl).version >> 8 <= 3 {
                            e_0.len_encoding = E_HUFFMAN;
                            e_0.len_dat = std::ptr::null_mut();
                        } else {
                            e_0.len_encoding = E_CONST_INT;
                            e_0.len_dat = std::ptr::null_mut();
                        }
                        libc::memset(
                            (&raw mut st_0).cast(),
                            0,
                            std::mem::size_of::<cram_stats_layout>(),
                        );
                        cram_cram_stats_c_52_cram_stats_add((&raw mut st_0).cast(), 2);
                        cram_cram_stats_c_134_cram_stats_encoding(
                            fd.cast(),
                            (&raw mut st_0).cast(),
                        );
                        e_0.val_encoding = E_EXTERNAL;
                        e_0.val_dat = sk as *mut c_void;
                        c_0 = cram_cram_codecs_c_3928_cram_encoder_init(
                            E_BYTE_ARRAY_LEN,
                            (&raw mut st_0).cast(),
                            E_BYTE_ARRAY,
                            (&raw mut e_0).cast(),
                            (*fdl).version,
                            (&raw mut (*fdl).vv).cast(),
                        )
                        .cast();
                    }
                    105 | 73 | 102 => {
                        let mut e_1 = cram_byte_array_len_encoder_dat_layout {
                            len_encoding: E_NULL,
                            val_encoding: E_NULL,
                            len_dat: std::ptr::null_mut(),
                            val_dat: std::ptr::null_mut(),
                            len_codec: std::ptr::null_mut(),
                            val_codec: std::ptr::null_mut(),
                        };
                        let mut st_1 = cram_stats_layout {
                            freqs: [0; 1024],
                            h: std::ptr::null_mut(),
                            nsamp: 0,
                            nvals: 0,
                            min_val: 0,
                            max_val: 0,
                        };
                        if (*fdl).version >> 8 <= 3 {
                            e_1.len_encoding = E_HUFFMAN;
                            e_1.len_dat = std::ptr::null_mut();
                        } else {
                            e_1.len_encoding = E_CONST_INT;
                            e_1.len_dat = std::ptr::null_mut();
                        }
                        libc::memset(
                            (&raw mut st_1).cast(),
                            0,
                            std::mem::size_of::<cram_stats_layout>(),
                        );
                        cram_cram_stats_c_52_cram_stats_add((&raw mut st_1).cast(), 4);
                        cram_cram_stats_c_134_cram_stats_encoding(
                            fd.cast(),
                            (&raw mut st_1).cast(),
                        );
                        e_1.val_encoding = E_EXTERNAL;
                        e_1.val_dat = sk as *mut c_void;
                        c_0 = cram_cram_codecs_c_3928_cram_encoder_init(
                            E_BYTE_ARRAY_LEN,
                            (&raw mut st_1).cast(),
                            E_BYTE_ARRAY,
                            (&raw mut e_1).cast(),
                            (*fdl).version,
                            (&raw mut (*fdl).vv).cast(),
                        )
                        .cast();
                    }
                    66 => {
                        let mut e_2 = cram_byte_array_len_encoder_dat_layout {
                            len_encoding: E_NULL,
                            val_encoding: E_NULL,
                            len_dat: std::ptr::null_mut(),
                            val_dat: std::ptr::null_mut(),
                            len_codec: std::ptr::null_mut(),
                            val_codec: std::ptr::null_mut(),
                        };
                        e_2.len_encoding = if (*fdl).version >> 8 >= 4 {
                            E_VARINT_UNSIGNED
                        } else {
                            E_EXTERNAL
                        };
                        e_2.len_dat = sk as *mut c_void;
                        e_2.val_encoding = E_EXTERNAL;
                        e_2.val_dat = sk as *mut c_void;
                        c_0 = cram_cram_codecs_c_3928_cram_encoder_init(
                            E_BYTE_ARRAY_LEN,
                            std::ptr::null_mut(),
                            E_BYTE_ARRAY,
                            (&raw mut e_2).cast(),
                            (*fdl).version,
                            (&raw mut (*fdl).vv).cast(),
                        )
                        .cast();
                    }
                    _ => {
                        hts_log_cstr(
                            HTS_LOG_ERROR,
                            c"cram_encode_aux".as_ptr(),
                            c"Unsupported SAM aux type".as_ptr(),
                        );
                        c_0 = std::ptr::null_mut();
                    }
                }
                if c_0.is_null() {
                    current_block = 9865445363914956224;
                    break;
                }
                (*m).codec = c_0;
                crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).metrics_lock);
                (*m).m = if k_global != 0 {
                    *(*metrics_h).vals.offset(k_global as isize)
                } else {
                    std::ptr::null_mut()
                };
                crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).metrics_lock);
            }
        }
        let tm: *mut cram_tag_map_layout = *(*tagmap).vals.offset(k as isize);
        if tm.is_null() {
            current_block = 9865445363914956224;
            break;
        }
        let codec: *mut cram_codec = (*tm).codec;
        if (*tm).codec.is_null() {
            current_block = 9865445363914956224;
            break;
        }
        match *aux.offset(2) as c_int {
            65 | 67 | 99 => {
                if aux_end.offset_from(aux) < (3 + 1) {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*tm).blk.is_null() {
                    (*tm).blk =
                        cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    let cbal = codec.cast::<cram_codec_byte_array_len_layout>();
                    (*((*cbal)
                        .byte_array_len
                        .val_codec
                        .cast::<cram_codec_base_layout>()))
                    .out = (*tm).blk.cast();
                }
                aux = aux.offset(3);
                if cram_cram_io_h_261_block_append_char((*tm).blk, *aux) < 0 {
                    current_block = 9865445363914956224;
                    break;
                }
                aux = aux.offset(1);
            }
            83 | 115 => {
                if aux_end.offset_from(aux) < (3 + 2) {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*tm).blk.is_null() {
                    (*tm).blk =
                        cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    let cbal = codec.cast::<cram_codec_byte_array_len_layout>();
                    (*((*cbal)
                        .byte_array_len
                        .val_codec
                        .cast::<cram_codec_base_layout>()))
                    .out = (*tm).blk.cast();
                }
                aux = aux.offset(3);
                if cram_cram_io_h_248_block_append((*tm).blk, aux as *const c_void, 2) < 0 {
                    current_block = 9865445363914956224;
                    break;
                }
                aux = aux.offset(2);
            }
            73 | 105 | 102 => {
                if aux_end.offset_from(aux) < (3 + 4) {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*tm).blk.is_null() {
                    (*tm).blk =
                        cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    let cbal = codec.cast::<cram_codec_byte_array_len_layout>();
                    (*((*cbal)
                        .byte_array_len
                        .val_codec
                        .cast::<cram_codec_base_layout>()))
                    .out = (*tm).blk.cast();
                }
                aux = aux.offset(3);
                if cram_cram_io_h_248_block_append((*tm).blk, aux as *const c_void, 4) < 0 {
                    current_block = 9865445363914956224;
                    break;
                }
                aux = aux.offset(4);
            }
            100 => {
                if aux_end.offset_from(aux) < (3 + 8) {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*tm).blk.is_null() {
                    (*tm).blk =
                        cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    let cbal = codec.cast::<cram_codec_byte_array_len_layout>();
                    (*((*cbal)
                        .byte_array_len
                        .val_codec
                        .cast::<cram_codec_base_layout>()))
                    .out = (*tm).blk.cast();
                }
                aux = aux.offset(3);
                if cram_cram_io_h_248_block_append((*tm).blk, aux as *const c_void, 8) < 0 {
                    current_block = 9865445363914956224;
                    break;
                }
                aux = aux.offset(8);
            }
            90 | 72 => {
                if aux_end.offset_from(aux) < 3 {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*tm).blk.is_null() {
                    (*tm).blk =
                        cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    (*(codec.cast::<cram_codec_base_layout>())).out = (*tm).blk.cast();
                }
                aux = aux.offset(3);
                let aux_s: *mut c_char = aux;
                while aux < aux_end as *mut c_char && {
                    let fresh160 = aux;
                    aux = aux.offset(1);
                    *fresh160 as c_int != 0
                } {}
                if std::ptr::eq(aux, aux_end as *mut c_char)
                    && *aux.offset(-1) as c_int != b'\0' as c_int
                {
                    hts_log_cstr(
                        HTS_LOG_ERROR,
                        c"cram_encode_aux".as_ptr(),
                        c"Unterminated Z/H tag".as_ptr(),
                    );
                    current_block = 9865445363914956224;
                    break;
                } else {
                    let encode: CramCodecEncodeFn = cram_fn(
                        (*(codec
                            .cast::<cram_codec_base_layout>()
                            .cast::<cram_codec_external_layout>()))
                        .encode,
                    );
                    if encode(s, codec.cast(), aux_s, aux.offset_from(aux_s) as c_int) < 0 {
                        current_block = 9865445363914956224;
                        break;
                    }
                }
            }
            66 => {
                if aux_end.offset_from(aux) < (4 + 4) {
                    current_block = 9865445363914956224;
                    break;
                }
                let type_0: c_int = *aux.offset(3) as c_int;
                let count: u64 = (*(aux as *mut c_uchar).offset(4) as u64)
                    | ((*(aux as *mut c_uchar).offset(5) as u64) << 8)
                    | ((*(aux as *mut c_uchar).offset(6) as u64) << 16)
                    | ((*(aux as *mut c_uchar).offset(7) as u64) << 24);
                let mut blen: u64;
                if (*tm).blk.is_null() {
                    (*tm).blk =
                        cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    let cbal = codec.cast::<cram_codec_byte_array_len_layout>();
                    let val_codec_ptr = (*cbal).byte_array_len.val_codec;
                    if (*(val_codec_ptr.cast::<cram_codec_base_layout>())).codec == E_XDELTA {
                        (*tm).blk2 = cram_cram_io_c_1388_cram_new_block(
                            CRAM_CONTENT_TYPE_EXTERNAL,
                            key_0 + 128,
                        );
                        if (*tm).blk2.is_null() {
                            current_block = 9865445363914956224;
                            break;
                        }
                        (*((*cbal)
                            .byte_array_len
                            .len_codec
                            .cast::<cram_codec_base_layout>()))
                        .out = (*tm).blk2.cast();
                        let xd = val_codec_ptr.cast::<cram_codec_xdelta_layout>();
                        (*((*xd).xdelta.sub_codec.cast::<cram_codec_base_layout>())).out =
                            (*tm).blk.cast();
                    } else {
                        (*((*cbal)
                            .byte_array_len
                            .len_codec
                            .cast::<cram_codec_base_layout>()))
                        .out = (*tm).blk.cast();
                        (*((*cbal)
                            .byte_array_len
                            .val_codec
                            .cast::<cram_codec_base_layout>()))
                        .out = (*tm).blk.cast();
                    }
                }
                aux = aux.offset(3);
                match type_0 {
                    99 | 67 => {
                        blen = count;
                    }
                    115 | 83 => {
                        blen = 2u64.wrapping_mul(count);
                    }
                    105 | 73 | 102 => {
                        blen = 4u64.wrapping_mul(count);
                    }
                    _ => {
                        hts_log_cstr(
                            HTS_LOG_ERROR,
                            c"cram_encode_aux".as_ptr(),
                            c"Unknown sub-type for aux type B".as_ptr(),
                        );
                        current_block = 9865445363914956224;
                        break;
                    }
                }
                blen = blen.wrapping_add(5);
                if (aux_end.offset_from(aux) as u64) < blen || blen > c_int::MAX as u64 {
                    current_block = 9865445363914956224;
                    break;
                }
                let encode: CramCodecEncodeFn =
                    cram_fn((*(codec.cast::<cram_codec_external_layout>())).encode);
                if encode(s, codec.cast(), aux, blen as c_int) < 0 {
                    current_block = 9865445363914956224;
                    break;
                }
                aux = aux.offset(blen as isize);
            }
            _ => {
                hts_log_cstr(
                    HTS_LOG_ERROR,
                    c"cram_encode_aux".as_ptr(),
                    c"Unknown aux type".as_ptr(),
                );
                current_block = 9865445363914956224;
                break;
            }
        }
        (*((*tm).blk.cast::<cram_block_layout>())).m = (*tm).m;
    }
    if current_block == 13391418783698890455 && cram_cram_io_h_261_block_append_char(td_b, 0) >= 0 {
        key_ptr = (*(td_b.cast::<cram_block_layout>()))
            .data
            .add(TD_blk_size as usize) as *mut c_char;
        let td_hash = (*(*cl).comp_hdr).td_hash.cast::<kh_m_s2i_layout>();
        k = kh_put_m_s2i(td_hash, key_ptr as *const c_char, &raw mut new_);
        if new_ >= 0 {
            let mut td_ok = true;
            if new_ == 0 {
                (*(td_b.cast::<cram_block_layout>())).byte = TD_blk_size as usize;
            } else {
                let pooled_key: *mut c_char = cram_string_alloc_c_153_string_ndup(
                    (*(*cl).comp_hdr).td_keys.cast(),
                    (*(td_b.cast::<cram_block_layout>()))
                        .data
                        .add(TD_blk_size as usize) as *const c_char,
                    (*(td_b.cast::<cram_block_layout>()))
                        .byte
                        .wrapping_sub(TD_blk_size as usize),
                );
                if pooled_key.is_null() {
                    td_ok = false;
                } else {
                    *(*td_hash).keys.offset(k as isize) = pooled_key as *const c_char;
                    *(*td_hash).vals.offset(k as isize) = (*(*cl).comp_hdr).ntl;
                    (*(*cl).comp_hdr).ntl += 1;
                }
            }
            if td_ok {
                (*crl).tl = *(*td_hash).vals.offset(k as isize);
                cram_cram_stats_c_52_cram_stats_add(
                    (*cl).stats[DS_TL_LOCAL as usize].cast(),
                    (*crl).tl,
                );
                if orig
                    != (*b)
                        .data
                        .add(((*b).core.n_cigar << 2) as usize)
                        .add((*b).core.l_qname as usize)
                        .add((((*b).core.l_qseq + 1) >> 1) as usize)
                        .add((*b).core.l_qseq as usize) as *mut c_char
                {
                    free(orig.cast());
                }
                if !err.is_null() {
                    *err = 0;
                }
                return brg;
            }
        }
    }
    if orig
        != (*b)
            .data
            .add(((*b).core.n_cigar << 2) as usize)
            .add((*b).core.l_qname as usize)
            .add((((*b).core.l_qseq + 1) >> 1) as usize)
            .add((*b).core.l_qseq as usize) as *mut c_char
    {
        free(orig.cast());
    }
    std::ptr::null_mut()
}

/// `process_one_read` (htslib/cram/cram_encode.c:3389). Per-record CRAM
/// encoder: walks the BAM's CIGAR + sequence + quality, populating the slice
/// feature list via the `cram_add_*` builders, runs `cram_encode_aux` for the
/// aux tags, then resolves the mate-pair relationship via `s->pair[sec]`.
///
/// Mirrors the upstream control flow exactly (every `cram_stats_add` failure
/// goto, every detached-fallback condition). Note that the production
/// `cram_cram_stats_c_52_cram_stats_add` returns `()`, not `c_int`, so the
/// upstream `< 0` failure branches collapse — `cram_stats_add` cannot fail
/// without ENOMEM, which the native variant treats as a silent miss exactly
/// like the C code's allocator failure path.
#[allow(clippy::too_many_arguments)]
pub unsafe fn cram_cram_encode_c_3389_process_one_read(
    fd: *mut cram_fd,
    c: *mut cram_container,
    s: *mut cram_slice,
    cr: *mut cram_record,
    b: *mut bam1_t,
    rnum: c_int,
    mut MD: *mut kstring_t,
    embed_ref: c_int,
    no_ref: c_int,
) -> c_int {
    use crate::htslib_rs::hts::{kputc, kputc_, kputsn, kputuw};
    use crate::htslib_rs::sam::{
        sam_hrec_rg_t, sam_hrecs_find_rg, BAM_CDEL, BAM_CDIFF, BAM_CEQUAL, BAM_CHARD_CLIP,
        BAM_CINS, BAM_CMATCH, BAM_CPAD, BAM_CREF_SKIP, BAM_CSOFT_CLIP, BAM_FMREVERSE, BAM_FMUNMAP,
        BAM_FSUPPLEMENTARY,
    };

    // Local DS_* constants (cram_DS_ID, cram_structs.h). Production
    // `cram_stats_add` and `cram_stats_del` take a `*mut c_void` stat slot.
    const DS_RI: usize = 33;
    const DS_BF: usize = 15;
    const DS_AP: usize = 17;
    const DS_RG: usize = 18;
    const DS_MQ: usize = 19;
    const DS_NS: usize = 20;
    const DS_MF: usize = 21;
    const DS_TS: usize = 22;
    const DS_NP: usize = 23;
    const DS_NF: usize = 24;
    const DS_RL: usize = 25;
    const DS_FN: usize = 26;
    const DS_BA: usize = 30;
    const DS_CF: usize = 16;

    // CRAM cram_flags / mate flags (cram_structs.h).
    const CRAM_FLAG_PRESERVE_QUAL_SCORES: i32 = 1 << 0;
    const CRAM_FLAG_DETACHED: i32 = 1 << 1;
    const CRAM_FLAG_MATE_DOWNSTREAM: i32 = 1 << 2;
    const CRAM_FLAG_NO_SEQ: i32 = 1 << 3;
    const CRAM_FLAG_EXPLICIT_TLEN: i32 = 1 << 4;
    const CRAM_FLAG_MASK: i32 = (1 << 5) - 1;
    const CRAM_FLAG_DISCARD_NAME: i32 = i32::MIN; // 1 << 31, signed
    const CRAM_FLAG_STATS_ADDED: i32 = 1 << 30;
    const CRAM_M_REVERSE: i32 = 1;
    const CRAM_M_UNMAP: i32 = 2;

    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let sl = s.cast::<cram_slice_layout>();
    let crl = cr.cast::<cram_record_layout>();

    let mut verbatim_NM: c_int = (*fdl).store_nm;
    let mut verbatim_MD: c_int = (*fdl).store_md;

    (*crl).flags = (*b).core.flag as i32;
    (*crl).len = (*b).core.l_qseq;

    // MD aux tag check — when missing, the caller's MD kstring is unused for
    // this record so the upstream code sets it to NULL locally; we mirror by
    // overwriting the local `MD` parameter.
    let md = bam_aux_get(b, c"MD".as_ptr());
    if md.is_null() {
        MD = std::ptr::null_mut();
    } else {
        (*MD).l = 0;
    }

    let mut cf_tag: c_int = 0;
    if embed_ref == 2 {
        cf_tag = if !MD.is_null() { 0 } else { 1 };
        cf_tag |= if !bam_aux_get(b, c"NM".as_ptr()).is_null() {
            0
        } else {
            2
        };
    }

    let ref_0: *mut c_char = if !(*cl).ref_.is_null() {
        (*cl).ref_.offset(-(((*cl).ref_start - 1) as isize))
    } else {
        std::ptr::null_mut()
    };
    (*crl).ref_id = (*b).core.tid;
    cram_cram_stats_c_52_cram_stats_add((*cl).stats[DS_RI].cast::<c_void>(), (*crl).ref_id);
    cram_cram_stats_c_52_cram_stats_add(
        (*cl).stats[DS_BF].cast::<c_void>(),
        (*fdl).cram_flag_swap[((*crl).flags & 0xfff) as usize] as c_int,
    );

    // Non reference based encoding means storing the bases verbatim as
    // features, which in turn means every base also has a quality already
    // stored.
    if no_ref == 0 || ((*fdl).version >> 8) >= 3 {
        (*crl).cram_flags |= CRAM_FLAG_PRESERVE_QUAL_SCORES;
    }

    if (*crl).len <= 0 && ((*fdl).version >> 8) >= 3 {
        (*crl).cram_flags |= CRAM_FLAG_NO_SEQ;
    }

    (*cl).num_bases += (*crl).len as i64;
    (*crl).apos = (*b).core.pos + 1;
    if (*crl).apos < 0 || (*crl).apos > i64::MAX / 2 {
        return -1;
    }
    if (*cl).pos_sorted != 0 {
        if (*crl).apos < (*sl).last_apos && (*fdl).ap_delta == 0 {
            (*cl).pos_sorted = 0;
        } else {
            cram_cram_stats_c_52_cram_stats_add(
                (*cl).stats[DS_AP].cast::<c_void>(),
                ((*crl).apos - (*sl).last_apos) as c_int,
            );
            (*sl).last_apos = (*crl).apos;
        }
    }
    (*cl).max_apos += ((*crl).apos > (*cl).max_apos) as i64 * ((*crl).apos - (*cl).max_apos);

    // s->seqs_blk: BAM-nibble decoded sequence; s->qual_blk: raw quals.
    let seqs_blk = (*sl).seqs_blk.cast::<cram_block_layout>();
    let qual_blk = (*sl).qual_blk.cast::<cram_block_layout>();
    (*crl).seq = (*seqs_blk).byte as u32;
    (*crl).qual = (*qual_blk).byte as u32;
    if cram_cram_io_h_243_block_grow(
        (*sl).seqs_blk.cast::<cram_block>(),
        ((*crl).len + 1) as usize,
    ) < 0
    {
        return -1;
    }
    if cram_cram_io_h_243_block_grow((*sl).qual_blk.cast::<cram_block>(), (*crl).len as usize) < 0 {
        return -1;
    }

    // BLOCK_END(seqs_blk) — write decoded bases then advance byte cursor.
    let seq: *mut c_char = (*seqs_blk).data.add((*seqs_blk).byte) as *mut c_char;
    *seq = 0;
    crate::htslib_rs::sam::nibble2base_default(
        (*b).data
            .add(((*b).core.n_cigar << 2) as usize)
            .add((*b).core.l_qname as usize),
        seq,
        (*crl).len,
    );
    (*seqs_blk).byte += (*crl).len as usize;

    // qual = bam_qual(b)
    let qual: *mut c_char = (*b)
        .data
        .add(((*b).core.n_cigar << 2) as usize)
        .add((*b).core.l_qname as usize)
        .add((((*b).core.l_qseq + 1) >> 1) as usize) as *mut c_char;

    let fake_qual: c_int;
    let mut NM: c_int = 0;

    // Mapped vs unmapped split
    if (*crl).flags & BAM_FUNMAP == 0 {
        let mut apos: i64 = (*crl).apos - 1;
        let mut spos: i64 = 0;
        let mut MD_last: i64 = apos; // last position of edit in MD tag

        if apos < 0 {
            hts_log_cstr(
                HTS_LOG_ERROR,
                c"process_one_read".as_ptr(),
                c"Mapped read with position <= 0 is disallowed".as_ptr(),
            );
            return -1;
        }

        (*crl).cigar = (*sl).ncigar;
        (*crl).ncigar = (*b).core.n_cigar as i32;
        while (*crl).cigar.wrapping_add((*crl).ncigar as u32) >= (*sl).cigar_alloc {
            (*sl).cigar_alloc = if (*sl).cigar_alloc != 0 {
                (*sl).cigar_alloc.wrapping_mul(2)
            } else {
                1024
            };
            (*sl).cigar = realloc(
                (*sl).cigar.cast::<c_void>(),
                (*sl).cigar_alloc as u64 * std::mem::size_of::<u32>() as u64,
            )
            .cast::<u32>();
            if (*sl).cigar.is_null() {
                return -1;
            }
        }

        let cig_to: *mut u32 = (*sl).cigar;
        let cig_from: *mut u32 = (*b).data.add((*b).core.l_qname as usize).cast::<u32>();

        (*crl).feature = 0;
        (*crl).nfeature = 0;
        let mut i: c_int = 0;
        while i < (*crl).ncigar {
            let cig_op: c_int = (*cig_from.add(i as usize) & BAM_CIGAR_MASK) as c_int;
            let cig_len: u32 = *cig_from.add(i as usize) >> BAM_CIGAR_SHIFT;
            *cig_to.add(((*crl).cigar as usize) + i as usize) = *cig_from.add(i as usize);

            let mut l: c_int;
            // Don't trust = and X ops to be correct: collapse to CMATCH path.
            if cig_op == BAM_CMATCH || cig_op == BAM_CEQUAL || cig_op == BAM_CDIFF {
                l = 0;
                if no_ref == 0 && (*crl).len != 0 {
                    let end_ix: c_int = if (cig_len as i64) + apos < (*cl).ref_end {
                        cig_len as c_int
                    } else {
                        ((*cl).ref_end - apos) as c_int
                    };
                    let sp = seq.offset(spos as isize);
                    let rp = ref_0.offset(apos as isize);
                    let qp = qual.offset(spos as isize);
                    if end_ix > (*crl).len {
                        hts_log_cstr(
                            HTS_LOG_ERROR,
                            c"process_one_read".as_ptr(),
                            c"CIGAR and query sequence are of different length".as_ptr(),
                        );
                        return -1;
                    }
                    l = 0;
                    while l < end_ix {
                        if *rp.offset(l as isize) == b'N' as c_char
                            && *sp.offset(l as isize) == b'N' as c_char
                        {
                            verbatim_NM = 1;
                            verbatim_MD = 1;
                        }
                        if *rp.offset(l as isize) != *sp.offset(l as isize) {
                            if !MD.is_null() && !ref_0.is_null() {
                                if kputuw((apos + l as i64 - MD_last) as u32, MD) < 0 {
                                    return -1;
                                }
                                if kputc(*rp.offset(l as isize) as c_int, MD) < 0 {
                                    return -1;
                                }
                                MD_last = apos + l as i64 + 1;
                            }
                            NM += 1;
                            if *sp.offset(l as isize) == 0 {
                                break;
                            }
                            // C source has a `0 && ...` guard; that branch is
                            // dead code in upstream, so we match the else.
                            if cram_cram_encode_c_2605_cram_add_substitution(
                                fd,
                                c,
                                s,
                                crl,
                                (spos + l as i64) as c_int,
                                *sp.offset(l as isize),
                                *qp.offset(l as isize),
                                *rp.offset(l as isize),
                            ) != 0
                            {
                                return -1;
                            }
                        }
                        l += 1;
                    }
                    spos += l as i64;
                    apos += l as i64;
                }

                if (l as u32) < cig_len && (*crl).len != 0 {
                    if no_ref != 0 {
                        if ((*fdl).version >> 8) == 3 {
                            if cram_cram_encode_c_2632_cram_add_bases(
                                fd,
                                c,
                                s,
                                crl,
                                spos as c_int,
                                (cig_len as c_int) - l,
                                seq.offset(spos as isize),
                            ) != 0
                            {
                                return -1;
                            }
                            spos += (cig_len as i64) - l as i64;
                        } else {
                            while (l as u32) < cig_len && *seq.offset(spos as isize) != 0 {
                                if cram_cram_encode_c_2645_cram_add_base(
                                    fd,
                                    c,
                                    s,
                                    crl,
                                    spos as c_int,
                                    *seq.offset(spos as isize),
                                    *qual.offset(spos as isize),
                                ) != 0
                                {
                                    return -1;
                                }
                                l += 1;
                                spos += 1;
                            }
                        }
                    } else {
                        // off end of sequence or non-ref based output
                        verbatim_NM = 1;
                        verbatim_MD = 1;
                        while (l as u32) < cig_len && *seq.offset(spos as isize) != 0 {
                            if cram_cram_encode_c_2645_cram_add_base(
                                fd,
                                c,
                                s,
                                crl,
                                spos as c_int,
                                *seq.offset(spos as isize),
                                *qual.offset(spos as isize),
                            ) != 0
                            {
                                return -1;
                            }
                            l += 1;
                            spos += 1;
                        }
                    }
                    apos += cig_len as i64;
                } else if (*crl).len == 0 {
                    // Seq "*"
                    verbatim_NM = 1;
                    verbatim_MD = 1;
                    apos += cig_len as i64;
                    spos += cig_len as i64;
                }
            } else if cig_op == BAM_CDEL {
                if !MD.is_null() && !ref_0.is_null() {
                    if kputuw((apos - MD_last) as u32, MD) < 0 {
                        return -1;
                    }
                    if apos < (*cl).ref_end {
                        if kputc_(b'^' as c_int, MD) < 0 {
                            return -1;
                        }
                        let span = if (*cl).ref_end - apos < cig_len as i64 {
                            (*cl).ref_end - apos
                        } else {
                            cig_len as i64
                        };
                        if kputsn(ref_0.offset(apos as isize), span as usize, MD) < 0 {
                            return -1;
                        }
                    }
                }
                NM += cig_len as c_int;

                if cram_cram_encode_c_2677_cram_add_deletion(
                    c,
                    s,
                    crl,
                    spos as c_int,
                    cig_len as c_int,
                    seq.offset(spos as isize),
                ) != 0
                {
                    return -1;
                }
                apos += cig_len as i64;
                MD_last = apos;
            } else if cig_op == BAM_CREF_SKIP {
                if cram_cram_encode_c_2733_cram_add_skip(
                    c,
                    s,
                    crl,
                    spos as c_int,
                    cig_len as c_int,
                    seq.offset(spos as isize),
                ) != 0
                {
                    return -1;
                }
                apos += cig_len as i64;
                MD_last += cig_len as i64;
            } else if cig_op == BAM_CINS {
                if cram_cram_encode_c_2753_cram_add_insertion(
                    c,
                    s,
                    crl,
                    spos as c_int,
                    cig_len as c_int,
                    if (*crl).len != 0 {
                        seq.offset(spos as isize)
                    } else {
                        std::ptr::null_mut()
                    },
                ) != 0
                {
                    return -1;
                }
                if no_ref != 0 && (*crl).len != 0 {
                    let mut ll: u32 = 0;
                    while ll < cig_len {
                        cram_cram_encode_c_2662_cram_add_quality(
                            fd,
                            c,
                            s,
                            crl,
                            spos as c_int,
                            *qual.offset(spos as isize),
                        );
                        ll += 1;
                        spos += 1;
                    }
                } else {
                    spos += cig_len as i64;
                }
                NM += cig_len as c_int;
            } else if cig_op == BAM_CSOFT_CLIP {
                if cram_cram_encode_c_2687_cram_add_softclip(
                    c,
                    s,
                    crl,
                    spos as c_int,
                    cig_len as c_int,
                    if (*crl).len != 0 {
                        seq.offset(spos as isize)
                    } else {
                        std::ptr::null_mut()
                    },
                    (*fdl).version,
                ) != 0
                {
                    return -1;
                }
                if no_ref != 0 && (*crl).cram_flags & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0 {
                    if (*crl).len != 0 {
                        let mut ll: u32 = 0;
                        while ll < cig_len {
                            cram_cram_encode_c_2662_cram_add_quality(
                                fd,
                                c,
                                s,
                                crl,
                                spos as c_int,
                                *qual.offset(spos as isize),
                            );
                            ll += 1;
                            spos += 1;
                        }
                    } else {
                        let mut ll: u32 = 0;
                        while ll < cig_len {
                            cram_cram_encode_c_2662_cram_add_quality(
                                fd,
                                c,
                                s,
                                crl,
                                spos as c_int,
                                -1_i8 as c_char,
                            );
                            ll += 1;
                            spos += 1;
                        }
                    }
                } else {
                    spos += cig_len as i64;
                }
            } else if cig_op == BAM_CHARD_CLIP {
                if cram_cram_encode_c_2723_cram_add_hardclip(
                    c,
                    s,
                    crl,
                    spos as c_int,
                    cig_len as c_int,
                    seq.offset(spos as isize),
                ) != 0
                {
                    return -1;
                }
            } else if cig_op == BAM_CPAD {
                if cram_cram_encode_c_2743_cram_add_pad(
                    c,
                    s,
                    crl,
                    spos as c_int,
                    cig_len as c_int,
                    seq.offset(spos as isize),
                ) != 0
                {
                    return -1;
                }
            } else {
                hts_log_cstr(
                    HTS_LOG_ERROR,
                    c"process_one_read".as_ptr(),
                    c"Unknown CIGAR op code".as_ptr(),
                );
                return -1;
            }
            i += 1;
        }
        if (*crl).len != 0 && spos != (*crl).len as i64 {
            hts_log_cstr(
                HTS_LOG_ERROR,
                c"process_one_read".as_ptr(),
                c"CIGAR and query sequence are of different length".as_ptr(),
            );
            return -1;
        }
        fake_qual = spos as c_int;
        // Protect against negative length refs (fuzz 382922241)
        (*crl).aend = if no_ref != 0 {
            apos
        } else {
            let max_ref = if (*cl).ref_end > 0 { (*cl).ref_end } else { 0 };
            if apos < max_ref {
                apos
            } else {
                max_ref
            }
        };
        cram_cram_stats_c_52_cram_stats_add(
            (*cl).stats[DS_FN].cast::<c_void>(),
            (*crl).nfeature as c_int,
        );

        if !MD.is_null() && !ref_0.is_null() && kputuw((apos - MD_last) as u32, MD) < 0 {
            return -1;
        }
    } else {
        // Unmapped
        (*crl).cram_flags |= CRAM_FLAG_PRESERVE_QUAL_SCORES;
        (*crl).cigar = 0;
        (*crl).ncigar = 0;
        (*crl).nfeature = 0;
        (*crl).aend = if (*crl).apos < (*cl).ref_end {
            (*crl).apos
        } else {
            (*cl).ref_end
        };
        let mut i: c_int = 0;
        while i < (*crl).len {
            cram_cram_stats_c_52_cram_stats_add(
                (*cl).stats[DS_BA].cast::<c_void>(),
                *seq.offset(i as isize) as c_int,
            );
            i += 1;
        }
        fake_qual = 0;
    }

    (*crl).ntags = 0;
    let mut err: c_int = 0;
    let brg: *mut sam_hrec_rg_t = cram_cram_encode_c_2788_cram_encode_aux(
        fd,
        b,
        c,
        s,
        cr,
        verbatim_NM,
        verbatim_MD,
        NM,
        MD,
        cf_tag,
        no_ref,
        &raw mut err,
    );
    if err != 0 {
        return -1;
    }

    // Read group, identified earlier
    if !brg.is_null() {
        (*crl).rg = (*brg).id;
    } else if ((*fdl).version >> 8) == 1 {
        let brg2 = sam_hrecs_find_rg((*(*fdl).header).hrecs, c"UNKNOWN".as_ptr());
        if brg2.is_null() {
            return -1;
        }
        (*crl).rg = (*brg2).id;
    } else {
        (*crl).rg = -1;
    }
    cram_cram_stats_c_52_cram_stats_add((*cl).stats[DS_RG].cast::<c_void>(), (*crl).rg);

    // Append to the qual block now. cram_add_substitution can generate BA/QS
    // events which need to be in the qual block before we append the rest.
    if (*crl).cram_flags & CRAM_FLAG_PRESERVE_QUAL_SCORES != 0 {
        // Special case of seq "*"
        if (*crl).len == 0 {
            (*crl).len = fake_qual;
            if cram_cram_io_h_243_block_grow(
                (*sl).qual_blk.cast::<cram_block>(),
                (*crl).len as usize,
            ) < 0
            {
                return -1;
            }
            let cp = (*qual_blk).data.add((*qual_blk).byte) as *mut c_char;
            libc::memset(cp.cast(), 255, (*crl).len as usize);
        } else {
            if cram_cram_io_h_243_block_grow(
                (*sl).qual_blk.cast::<cram_block>(),
                (*crl).len as usize,
            ) < 0
            {
                return -1;
            }
            let cp = (*qual_blk).data.add((*qual_blk).byte) as *mut c_char;
            let from = (*b)
                .data
                .add(((*b).core.n_cigar << 2) as usize)
                .add((*b).core.l_qname as usize)
                .add((((*b).core.l_qseq + 1) >> 1) as usize) as *mut c_char;
            memcpy(cp.cast(), from.cast(), (*crl).len as u64);

            // Store quality in original orientation for better compression.
            if (*cl).qs_seq_orient == 0 && (*crl).flags & BAM_FREVERSE != 0 {
                let mut i: c_int = 0;
                let mut j: c_int = (*crl).len - 1;
                while i < j {
                    let tmp = *cp.offset(i as isize);
                    *cp.offset(i as isize) = *cp.offset(j as isize);
                    *cp.offset(j as isize) = tmp;
                    i += 1;
                    j -= 1;
                }
            }
        }
        (*qual_blk).byte += (*crl).len as usize;
    } else if (*crl).len == 0 {
        (*crl).len = if fake_qual >= 0 {
            fake_qual
        } else {
            ((*crl).aend - (*crl).apos + 1) as c_int
        };
    }

    cram_cram_stats_c_52_cram_stats_add((*cl).stats[DS_RL].cast::<c_void>(), (*crl).len);

    // Now we know apos and aend both — update mate-pair information.
    {
        let mut new_: c_int;
        let k: u32;
        let sec: usize = if (*crl).flags & BAM_FSECONDARY != 0 {
            1
        } else {
            0
        };
        let mut goto_detached = false;

        if (*crl).flags & BAM_FPAIRED != 0 {
            let pair_h = (*sl).pair[sec].cast::<kh_m_s2i_layout>();
            let qname = (*b).data as *const c_char;
            new_ = 0;
            k = kh_put_m_s2i(pair_h, qname, &raw mut new_);
            if new_ == -1 {
                return -1;
            } else if new_ > 0 {
                // bam_name(b) is likely to change, so copy it to a string pool.
                let qname_len =
                    ((*b).core.l_qname as c_int - (*b).core.l_extranul as c_int) as usize;
                let key = cram_string_alloc_c_153_string_ndup((*sl).pair_keys, qname, qname_len);
                if key.is_null() {
                    return -1;
                }
                *(*pair_h).keys.add(k as usize) = key;
                let r1_bit = ((((*crl).flags & BAM_FREAD1) != 0) as c_int) << 30;
                let r2_bit = ((((*crl).flags & BAM_FREAD2) != 0) as c_int) << 31;
                *(*pair_h).vals.add(k as usize) = rnum | r1_bit | r2_bit;
            }
        } else {
            new_ = 1;
            k = 0; // Prevents false-positive warning from gcc -Og
        }

        if new_ == 0 {
            let pair_h = (*sl).pair[sec].cast::<kh_m_s2i_layout>();
            let val0 = *(*pair_h).vals.add(k as usize);
            let p: *mut cram_record_layout = (*sl).crecs.offset((val0 & ((1 << 30) - 1)) as isize);

            let aleft = if (*crl).apos < (*p).apos {
                (*crl).apos
            } else {
                (*p).apos
            };
            let aright = if (*crl).aend > (*p).aend {
                (*crl).aend
            } else {
                (*p).aend
            };
            let sign: i64 = if (*crl).apos < (*p).apos {
                1
            } else if (*crl).apos > (*p).apos {
                -1
            } else if (*crl).flags & BAM_FREAD1 != 0 {
                1
            } else {
                -1
            };

            // Multiple sets of secondary reads means we cannot tell which is
            // which, so we store TLEN etc verbatim.
            let has_r1 = val0 & (1 << 30);
            let has_r2 = (val0 as u32) & (1u32 << 31);
            if (has_r1 != 0 && ((*crl).flags & BAM_FREAD1) != 0)
                || (has_r2 != 0 && ((*crl).flags & BAM_FREAD2) != 0)
            {
                goto_detached = true;
            }

            // This vs p: tlen, matepos, flags. Permit TLEN 0 and/or TLEN +/-
            // a small amount, if appropriate options set.
            if !goto_detached {
                let mate_pos1 = if (*b).core.mpos + 1 > 0 {
                    (*b).core.mpos + 1
                } else {
                    0
                };
                let cond_a = (*fdl).tlen_zero == 0 && mate_pos1 != (*p).apos;
                let cond_b = (*fdl).tlen_zero != 0 && (*b).core.mpos == 0;
                if cond_a && !cond_b {
                    goto_detached = true;
                }
            }

            if !goto_detached
                && (((*b).core.flag as c_int & BAM_FMUNMAP) != 0)
                    != (((*p).flags & BAM_FUNMAP) != 0)
            {
                goto_detached = true;
            }
            if !goto_detached
                && (((*b).core.flag as c_int & BAM_FMREVERSE) != 0)
                    != (((*p).flags & BAM_FREVERSE) != 0)
            {
                goto_detached = true;
            }

            // p vs this
            if !goto_detached
                && (*p).ref_id != (*crl).ref_id
                && !((*fdl).tlen_zero != 0 && (*p).ref_id == -1)
            {
                goto_detached = true;
            }
            if !goto_detached
                && (*p).mate_pos != (*crl).apos
                && !((*fdl).tlen_zero != 0 && (*p).mate_pos == 0)
            {
                goto_detached = true;
            }
            if !goto_detached
                && (((*p).flags & BAM_FMUNMAP) != 0) != (((*p).mate_flags & CRAM_M_UNMAP) != 0)
            {
                goto_detached = true;
            }
            if !goto_detached
                && (((*p).flags & BAM_FMREVERSE) != 0) != (((*p).mate_flags & CRAM_M_REVERSE) != 0)
            {
                goto_detached = true;
            }
            // Supplementary reads are just too ill defined
            if !goto_detached
                && (((*crl).flags & BAM_FSUPPLEMENTARY) != 0
                    || ((*p).flags & BAM_FSUPPLEMENTARY) != 0)
            {
                goto_detached = true;
            }
            // When in lossy name mode, if a read isn't detached we cannot
            // store the name.  The corollary is that when we must store the
            // name, it must be detached (inefficient).
            if !goto_detached
                && (*fdl).lossy_read_names != 0
                && ((*crl).cram_flags & CRAM_FLAG_DISCARD_NAME == 0
                    || ((*p).cram_flags & CRAM_FLAG_DISCARD_NAME) == 0)
            {
                goto_detached = true;
            }

            // Now check TLEN. We do this last as sometimes it's the only
            // thing that differs. In CRAM4 we have a better way of handling
            // this that doesn't break detached status.
            let mut explicit_tlen: c_int = 0;
            if !goto_detached {
                let ins = (*b).core.isize;
                let tlen_approx = (*fdl).tlen_approx as i64;
                let tflag1 = (ins != 0 && (ins - sign * (aright - aleft + 1)).abs() > tlen_approx)
                    || (ins == 0 && (*fdl).tlen_zero == 0);
                let tflag2 = ((*p).tlen != 0
                    && ((*p).tlen - -sign * (aright - aleft + 1)).abs() > tlen_approx)
                    || ((*p).tlen == 0 && (*fdl).tlen_zero == 0);
                if tflag1 || tflag2 {
                    if ((*fdl).version >> 8) >= 4 {
                        explicit_tlen = CRAM_FLAG_EXPLICIT_TLEN;
                    } else {
                        // Still do detached for unmapped data in CRAM4 as
                        // this also impacts RNEXT calculation.
                        goto_detached = true;
                    }
                }
            }

            if !goto_detached {
                // The fields below are unused when encoding this read as it
                // is no longer detached. In theory they may get referred to
                // when processing a 3rd or 4th read in this template, so we
                // set them here just to be sure. They do not need
                // cram_stats_add() calls since they are not emitted.
                (*crl).mate_pos = (*p).apos;
                cram_cram_stats_c_52_cram_stats_add(
                    (*cl).stats[DS_NP].cast::<c_void>(),
                    (*crl).mate_pos as c_int,
                );
                (*crl).tlen = if explicit_tlen != 0 {
                    (*b).core.isize
                } else {
                    sign * (aright - aleft + 1)
                };
                cram_cram_stats_c_52_cram_stats_add(
                    (*cl).stats[DS_TS].cast::<c_void>(),
                    (*crl).tlen as c_int,
                );
                (*crl).mate_flags = (((*p).flags & BAM_FMUNMAP == BAM_FMUNMAP) as i32)
                    * CRAM_M_UNMAP
                    + (((*p).flags & BAM_FMREVERSE == BAM_FMREVERSE) as i32) * CRAM_M_REVERSE;

                // Decrement statistics aggregated earlier
                if (*p).cram_flags & CRAM_FLAG_STATS_ADDED != 0 {
                    cram_cram_stats_c_80_cram_stats_del(
                        (*cl).stats[DS_NP].cast::<c_void>(),
                        (*p).mate_pos as c_int,
                    );
                    cram_cram_stats_c_80_cram_stats_del(
                        (*cl).stats[DS_MF].cast::<c_void>(),
                        (*p).mate_flags,
                    );
                    if (*p).cram_flags & CRAM_FLAG_EXPLICIT_TLEN == 0 && explicit_tlen == 0 {
                        cram_cram_stats_c_80_cram_stats_del(
                            (*cl).stats[DS_TS].cast::<c_void>(),
                            (*p).tlen as c_int,
                        );
                    }
                    cram_cram_stats_c_80_cram_stats_del(
                        (*cl).stats[DS_NS].cast::<c_void>(),
                        (*p).mate_ref_id,
                    );
                }

                // Clear detached from cr flags
                (*crl).cram_flags &= !CRAM_FLAG_DETACHED;
                (*crl).cram_flags |= explicit_tlen;
                cram_cram_stats_c_52_cram_stats_add(
                    (*cl).stats[DS_CF].cast::<c_void>(),
                    (*crl).cram_flags & CRAM_FLAG_MASK,
                );

                // Clear detached from p flags and set downstream
                if (*p).cram_flags & CRAM_FLAG_STATS_ADDED != 0 {
                    cram_cram_stats_c_80_cram_stats_del(
                        (*cl).stats[DS_CF].cast::<c_void>(),
                        (*p).cram_flags & CRAM_FLAG_MASK,
                    );
                    (*p).cram_flags &= !CRAM_FLAG_STATS_ADDED;
                }

                (*p).cram_flags &= !CRAM_FLAG_DETACHED;
                (*p).cram_flags |= CRAM_FLAG_MATE_DOWNSTREAM | explicit_tlen;
                cram_cram_stats_c_52_cram_stats_add(
                    (*cl).stats[DS_CF].cast::<c_void>(),
                    (*p).cram_flags & CRAM_FLAG_MASK,
                );

                (*p).mate_line = rnum - ((val0 & ((1 << 30) - 1)) + 1);
                cram_cram_stats_c_52_cram_stats_add(
                    (*cl).stats[DS_NF].cast::<c_void>(),
                    (*p).mate_line,
                );

                let r12_flags = val0 & (3i32 << 30);
                let r1_bit = ((((*crl).flags & BAM_FREAD1) != 0) as c_int) << 30;
                let r2_bit = ((((*crl).flags & BAM_FREAD2) != 0) as c_int) << 31;
                *(*pair_h).vals.add(k as usize) = rnum | r12_flags | r1_bit | r2_bit;
            }
        }

        if new_ != 0 || goto_detached {
            // detached
            (*crl).mate_flags = 0;
            if (*b).core.flag as c_int & BAM_FMUNMAP != 0 {
                (*crl).mate_flags |= CRAM_M_UNMAP;
            }
            if (*b).core.flag as c_int & BAM_FMREVERSE != 0 {
                (*crl).mate_flags |= CRAM_M_REVERSE;
            }

            cram_cram_stats_c_52_cram_stats_add(
                (*cl).stats[DS_MF].cast::<c_void>(),
                (*crl).mate_flags,
            );

            let mp = (*b).core.mpos + 1;
            (*crl).mate_pos = if mp > 0 { mp } else { 0 };
            cram_cram_stats_c_52_cram_stats_add(
                (*cl).stats[DS_NP].cast::<c_void>(),
                (*crl).mate_pos as c_int,
            );

            (*crl).tlen = (*b).core.isize;
            cram_cram_stats_c_52_cram_stats_add(
                (*cl).stats[DS_TS].cast::<c_void>(),
                (*crl).tlen as c_int,
            );

            (*crl).cram_flags |= CRAM_FLAG_DETACHED;
            cram_cram_stats_c_52_cram_stats_add(
                (*cl).stats[DS_CF].cast::<c_void>(),
                (*crl).cram_flags & CRAM_FLAG_MASK,
            );
            cram_cram_stats_c_52_cram_stats_add(
                (*cl).stats[DS_NS].cast::<c_void>(),
                (*b).core.mtid,
            );

            (*crl).cram_flags |= CRAM_FLAG_STATS_ADDED;
        }
    }

    (*crl).mqual = (*b).core.qual as i32;
    cram_cram_stats_c_52_cram_stats_add((*cl).stats[DS_MQ].cast::<c_void>(), (*crl).mqual);

    (*crl).mate_ref_id = (*b).core.mtid;

    if (*b).core.flag as c_int & BAM_FUNMAP == 0 {
        if (*cl).first_base > (*crl).apos {
            (*cl).first_base = (*crl).apos;
        }
        if (*cl).last_base < (*crl).aend {
            (*cl).last_base = (*crl).aend;
        }
    }

    0
}

/// `cram_encode_container` (htslib/cram/cram_encode.c:1850). The top-level
/// CRAM container encoder. Walks every record of every slice through
/// `process_one_read`, computes per-slice MD5s, runs the codec-init cascade
/// for every data series, then drives `cram_encode_slice` and finally
/// `cram_encode_compression_header`. Returns 0 on success, -1 on failure.
///
/// Preserves the upstream control flow exactly: embed-ref retry loop,
/// multi-ref slice switching, lossy-read-names pre-pass, aux block stashing
/// into each slice via the tags_used khash, MD5 calculation, the full codec
/// init cascade with conditional nesting for v1/v3/v4, slice landmark
/// computation, and the reference-counting decrement at the end.
pub unsafe fn cram_cram_encode_c_1850_cram_encode_container(
    fd: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    // Local DS_* constants (cram_DS_ID, cram_structs.h).
    const DS_RN: usize = 11;
    const DS_QS: usize = 12;
    const DS_IN: usize = 13;
    const DS_SC: usize = 14;
    const DS_BF: usize = 15;
    const DS_CF: usize = 16;
    const DS_AP: usize = 17;
    const DS_RG: usize = 18;
    const DS_MQ: usize = 19;
    const DS_NS: usize = 20;
    const DS_MF: usize = 21;
    const DS_TS: usize = 22;
    const DS_NP: usize = 23;
    const DS_NF: usize = 24;
    const DS_RL: usize = 25;
    const DS_FN: usize = 26;
    const DS_FC: usize = 27;
    const DS_FP: usize = 28;
    const DS_DL: usize = 29;
    const DS_BA: usize = 30;
    const DS_BS: usize = 31;
    const DS_TL: usize = 32;
    const DS_RI: usize = 33;
    const DS_RS: usize = 34;
    const DS_PD: usize = 35;
    const DS_HC: usize = 36;
    const DS_BB: usize = 37;
    const DS_TN: usize = 39;
    const DS_BB_len: usize = 42;
    const DS_SC_len: usize = 41;
    const DS_TC: usize = 44;

    // Codec encoding constants (cram_encoding enum, cram_structs.h).
    const E_INT: c_int = 1;
    const E_LONG: c_int = 2;
    const E_BYTE: c_int = 3;
    const E_BYTE_ARRAY: c_int = 4;
    const E_EXTERNAL: c_int = 1;
    const E_BYTE_ARRAY_LEN: c_int = 4;
    const E_BYTE_ARRAY_STOP: c_int = 5;
    const E_BETA: c_int = 6;
    const E_VARINT_UNSIGNED: c_int = 41;
    const E_VARINT_SIGNED: c_int = 42;

    // CRAM substitution matrix (htslib/cram/cram_encode.c CRAM_SUBST_MATRIX).
    const CRAM_SUBST_MATRIX: [u8; 20] = [
        b'C', b'G', b'T', b'N', b'A', b'G', b'T', b'N', b'A', b'C', b'T', b'N', b'A', b'C', b'G',
        b'N', b'A', b'C', b'G', b'T',
    ];

    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let h: *mut cram_block_compression_hdr = (*cl).comp_hdr.cast();
    let hl = h.cast::<cram_block_compression_hdr_layout>();
    let c_hdr: *mut cram_block;
    let multi_ref: c_int;
    let mut nref: c_int;
    let mut embed_ref: c_int;
    let mut no_ref: c_int;

    if (*cl).bams.is_null() {
        return -1;
    }
    if ((*fdl).version >> 8) == 1 {
        return -1;
    }

    // Don't try embed ref if we repeatedly fail.
    crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).ref_lock);
    let mut failed_embed: c_int = ((*fdl).no_ref_counter >= 5) as c_int;
    if failed_embed == 0 && (*cl).embed_ref == -2 && (*cl).ref_id >= 0 {
        hts_log_cstr(
            HTS_LOG_WARNING,
            c"cram_encode_container".as_ptr(),
            c"Retrying embed_ref=2 mode".as_ptr(),
        );
        (*cl).no_ref = 0;
        (*fdl).no_ref = 0;
        (*cl).embed_ref = 2;
        (*fdl).embed_ref = 2;
    } else if failed_embed != 0 && (*cl).embed_ref == -2 {
        hts_log_cstr(
            HTS_LOG_WARNING,
            c"cram_encode_container".as_ptr(),
            c"Keeping non-ref mode from now on".as_ptr(),
        );
        (*cl).embed_ref = 0;
        (*fdl).embed_ref = 0;
    }
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).ref_lock);

    // Outer `restart:` label is mirrored as a labeled loop. On failure paths
    // that should `goto err` we return -1 immediately. Paths that `continue`
    // the restart loop use `continue 'restart`.
    'restart: loop {
        crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).ref_lock);
        nref = (*(*fdl).refs).nref;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).ref_lock);
        embed_ref = (*cl).embed_ref;
        no_ref = (*cl).no_ref;

        // Fetch reference sequence (when not in no_ref mode).
        if no_ref == 0 {
            if (*cl).bams.is_null() || (*cl).curr_c_rec == 0 || (*(*cl).bams.offset(0)).is_null() {
                return -1;
            }
            let b: *mut bam1_t = *(*cl).bams.offset(0);
            let mut do_auto_ref = false;
            if embed_ref <= 1 {
                let ref_0 = cram_cram_io_c_3409_cram_get_ref(fd, (*b).core.tid, 1, 0);
                if ref_0.is_null() && (*b).core.tid >= 0 {
                    if (*cl).pos_sorted == 0 {
                        hts_log_cstr(
                            HTS_LOG_WARNING,
                            c"cram_encode_container".as_ptr(),
                            c"Failed to load reference".as_ptr(),
                        );
                        hts_log_cstr(
                            HTS_LOG_WARNING,
                            c"cram_encode_container".as_ptr(),
                            c"Switching to non-ref mode".as_ptr(),
                        );
                        crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).ref_lock);
                        (*fdl).embed_ref = 0;
                        (*cl).embed_ref = 0;
                        (*fdl).no_ref = 1;
                        (*cl).no_ref = 1;
                        crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).ref_lock);
                        continue 'restart;
                    }
                    if (*cl).multi_seq != 0 || embed_ref == 0 {
                        hts_log_cstr(
                            HTS_LOG_ERROR,
                            c"cram_encode_container".as_ptr(),
                            c"Failed to load reference".as_ptr(),
                        );
                        return -1;
                    }
                    hts_log_cstr(
                        HTS_LOG_WARNING,
                        c"cram_encode_container".as_ptr(),
                        c"Failed to load reference".as_ptr(),
                    );
                    hts_log_cstr(
                        HTS_LOG_WARNING,
                        c"cram_encode_container".as_ptr(),
                        c"Enabling embed_ref=2 mode to auto-generate reference".as_ptr(),
                    );
                    if embed_ref <= 0 {
                        hts_log_cstr(
                            HTS_LOG_WARNING,
                            c"cram_encode_container".as_ptr(),
                            c"NOTE: the CRAM file will be bigger than using an external reference"
                                .as_ptr(),
                        );
                    }
                    crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).ref_lock);
                    (*fdl).embed_ref = 2;
                    (*cl).embed_ref = 2;
                    embed_ref = 2;
                    crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).ref_lock);
                    do_auto_ref = true;
                } else {
                    if !ref_0.is_null()
                        && cram_cram_encode_c_1798_validate_md5(fd, (*cl).ref_seq_id) < 0
                    {
                        return -1;
                    }
                    (*cl).ref_id = (*b).core.tid;
                    if (*cl).ref_id >= 0 {
                        (*cl).ref_seq_id = (*cl).ref_id;
                        let entry = *(*(*fdl).refs).ref_id.offset((*cl).ref_seq_id as isize);
                        (*cl).ref_ = (*entry).seq;
                        (*cl).ref_start = 1;
                        (*cl).ref_end = (*entry).length;
                    }
                }
            } else {
                do_auto_ref = true;
            }
            if do_auto_ref {
                // auto_ref label: embed_ref=2 path. Allocate a NULL ref now,
                // it will be filled in by cram_generate_reference per-slice.
                (*cl).ref_id = (*b).core.tid;
                if (*cl).ref_id >= 0 {
                    (*cl).ref_ = std::ptr::null_mut();
                    (*cl).ref_free = 1;
                } else {
                    embed_ref = 0;
                    (*cl).no_ref = 1;
                    no_ref = 1;
                }
            }
            (*cl).ref_seq_id = (*cl).ref_id;
        } else {
            (*cl).ref_id = (**(*cl).bams.offset(0)).core.tid;
            cram_cram_io_c_3183_cram_ref_incr((*fdl).refs.cast(), (*cl).ref_id);
            (*cl).ref_seq_id = (*cl).ref_id;
        }

        if no_ref == 0 && !(*cl).refs_used.is_null() {
            let mut i: c_int = 0;
            while i < nref {
                if *(*cl).refs_used.offset(i as isize) != 0 {
                    if !cram_cram_io_c_3409_cram_get_ref(fd, i, 1, 0).is_null() {
                        if cram_cram_encode_c_1798_validate_md5(fd, i) < 0 {
                            return -1;
                        }
                    } else {
                        hts_log_cstr(
                            HTS_LOG_WARNING,
                            c"cram_encode_container".as_ptr(),
                            c"Failed to find reference, switching to non-ref mode".as_ptr(),
                        );
                        (*cl).no_ref = 1;
                        no_ref = 1;
                    }
                }
                i += 1;
            }
        }

        // Turn bams into cram_records and gather basic stats.
        let mut r1: c_int = 0;
        let mut sn: c_int = 0;
        let mut continue_restart = false;
        while r1 < (*cl).curr_c_rec {
            let sl_layout: *mut cram_slice_layout = *(*cl).slices.offset(sn as isize);
            let s: *mut cram_slice = sl_layout.cast::<cram_slice>();
            let mut first_base: i64 = i64::MAX;
            let mut last_base: i64 = i64::MIN;
            let r1_start: c_int = r1;
            // assert(sn < c->curr_slice);
            debug_assert!(sn < (*cl).curr_slice);

            // Discover which read names *may* be safely removed.
            if cram_cram_encode_c_1344_lossy_read_names(fd, c, s, r1_start) != 0 {
                return -1;
            }

            // MD kstring (reused across records, freed per-slice).
            let mut md_ks: kstring_t = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };

            // Embed consensus / MD-generated ref.
            if embed_ref == 2 {
                if (*cl).ref_id < 0 || cram_cram_encode_c_1737_cram_generate_reference(c, s, r1) < 0
                {
                    if sn > 0 {
                        hts_log_cstr(
                            HTS_LOG_ERROR,
                            c"cram_encode_container".as_ptr(),
                            c"Failed to build reference, switching to non-ref mode".as_ptr(),
                        );
                        return -1;
                    } else {
                        hts_log_cstr(
                            HTS_LOG_WARNING,
                            c"cram_encode_container".as_ptr(),
                            c"Failed to build reference, switching to non-ref mode".as_ptr(),
                        );
                    }
                    crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).ref_lock);
                    (*fdl).embed_ref = -2;
                    (*cl).embed_ref = -2;
                    (*fdl).no_ref = 1;
                    (*cl).no_ref = 1;
                    (*fdl).no_ref_counter += 1;
                    crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).ref_lock);
                    failed_embed = 1;
                    let _ = failed_embed;
                    free(md_ks.s.cast());
                    continue_restart = true;
                    break;
                } else {
                    crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).ref_lock);
                    (*fdl).no_ref_counter -= ((*fdl).no_ref_counter > 0) as c_int;
                    crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).ref_lock);
                }
                let entry = *(*(*fdl).refs).ref_id.offset((*cl).ref_id as isize);
                let rlen: i64 = if (*entry).ln_length > (*entry).length {
                    (*entry).ln_length
                } else {
                    (*entry).length
                };
                if (*cl).ref_end > rlen && rlen != 0 {
                    (*cl).ref_end = rlen;
                }
            }

            // Iterate through records.
            let mut r2: c_int = 0;
            while r1 < (*cl).curr_c_rec && r2 < (*(*sl_layout).hdr).num_records {
                let cr: *mut cram_record = (*sl_layout).crecs.offset(r2 as isize).cast();
                let crl = cr.cast::<cram_record_layout>();
                let b: *mut bam1_t = *(*cl).bams.offset(r1 as isize);

                // Multi-ref: switch reference per seq.
                if (*cl).multi_seq != 0
                    && no_ref == 0
                    && (*b).core.tid != (*cl).ref_seq_id
                    && (*b).core.tid >= 0
                {
                    if (*cl).ref_seq_id >= 0 {
                        cram_cram_io_c_3213_cram_ref_decr((*fdl).refs.cast(), (*cl).ref_seq_id);
                    }
                    if cram_cram_io_c_3409_cram_get_ref(fd, (*b).core.tid, 1, 0).is_null() {
                        hts_log_cstr(
                            HTS_LOG_ERROR,
                            c"cram_encode_container".as_ptr(),
                            c"Failed to load reference".as_ptr(),
                        );
                        free(md_ks.s.cast());
                        return -1;
                    }
                    if cram_cram_encode_c_1798_validate_md5(fd, (*b).core.tid) < 0 {
                        return -1;
                    }
                    (*cl).ref_seq_id = (*b).core.tid;
                    let entry = *(*(*fdl).refs).ref_id.offset((*cl).ref_seq_id as isize);
                    if (*entry).seq.is_null() {
                        return -1;
                    }
                    (*cl).ref_ = (*entry).seq;
                    (*cl).ref_start = 1;
                    (*cl).ref_end = (*entry).length;
                }

                if cram_cram_encode_c_3389_process_one_read(
                    fd,
                    c,
                    s,
                    cr,
                    b,
                    r2,
                    &raw mut md_ks,
                    embed_ref,
                    no_ref,
                ) != 0
                {
                    free(md_ks.s.cast());
                    return -1;
                }

                if first_base > (*crl).apos {
                    first_base = (*crl).apos;
                }
                if last_base < (*crl).aend {
                    last_base = (*crl).aend;
                }

                r1 += 1;
                r2 += 1;
            }

            free(md_ks.s.cast());

            // Post-pass: now that all records are processed and any TLEN
            // detached fixups settled, add the read names to the slice.
            if cram_cram_encode_c_1437_add_read_names(fd, c, s, r1_start) < 0 {
                return -1;
            }

            // Slice header refs/spans.
            let hdr_blk = (*sl_layout).hdr;
            if (*cl).multi_seq != 0 {
                (*hdr_blk).ref_seq_id = -2;
                (*hdr_blk).ref_seq_start = 0;
                (*hdr_blk).ref_seq_span = 0;
            } else if (*cl).ref_id == -1 && (*fdl).version >= 0x301 {
                (*hdr_blk).ref_seq_id = -1;
                (*hdr_blk).ref_seq_start = 0;
                (*hdr_blk).ref_seq_span = 0;
            } else {
                (*hdr_blk).ref_seq_id = (*cl).ref_id;
                (*hdr_blk).ref_seq_start = first_base;
                let span = last_base - first_base + 1;
                (*hdr_blk).ref_seq_span = if span > 0 { span } else { 0 };
            }
            (*hdr_blk).num_records = r2;

            // Stash aux blocks from tags_used into this slice's aux_block[].
            if !(*cl).tags_used.is_null() && (*(*cl).tags_used).n_occupied != 0 {
                let ntags: c_int = (*(*cl).tags_used).n_occupied as c_int;
                (*sl_layout).aux_block = calloc(
                    (ntags as u64).wrapping_mul(2),
                    std::mem::size_of::<*mut cram_block_layout>() as u64,
                )
                .cast::<*mut cram_block_layout>();
                if (*sl_layout).aux_block.is_null() {
                    return -1;
                }
                (*sl_layout).naux_block = 0;
                let tagmap = (*cl).tags_used.cast::<kh_m_tagmap_layout>();
                let n_buckets = (*tagmap).n_buckets;
                let mut k: u32 = 0;
                while k != n_buckets {
                    let flag_word = *(*tagmap).flags.add((k >> 4) as usize);
                    let exists = ((flag_word >> ((k & 0xf) << 1)) & 3) == 0;
                    if exists {
                        let tm = *(*tagmap).vals.offset(k as isize);
                        if tm.is_null() {
                            return -1;
                        }
                        if !(*tm).blk.is_null() {
                            let idx = (*sl_layout).naux_block;
                            (*sl_layout).naux_block += 1;
                            *(*sl_layout).aux_block.offset(idx as isize) =
                                (*tm).blk.cast::<cram_block_layout>();
                            (*tm).blk = std::ptr::null_mut();
                            if !(*tm).blk2.is_null() {
                                let idx2 = (*sl_layout).naux_block;
                                (*sl_layout).naux_block += 1;
                                *(*sl_layout).aux_block.offset(idx2 as isize) =
                                    (*tm).blk2.cast::<cram_block_layout>();
                                (*tm).blk2 = std::ptr::null_mut();
                            }
                        }
                    }
                    k = k.wrapping_add(1);
                }
                debug_assert!(
                    (*sl_layout).naux_block as u32
                        <= 2u32.wrapping_mul((*(*cl).tags_used).n_occupied)
                );
            }

            sn += 1;
        }
        if continue_restart {
            continue 'restart;
        }

        if (*cl).multi_seq != 0 && no_ref == 0 && (*cl).ref_seq_id >= 0 {
            cram_cram_io_c_3213_cram_ref_decr((*fdl).refs.cast(), (*cl).ref_seq_id);
        }

        // Link our bams[] array onto the spare bam list for reuse.
        let spares =
            malloc(std::mem::size_of::<spare_bams_layout>() as u64).cast::<spare_bams_layout>();
        if spares.is_null() {
            return -1;
        }
        crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).bam_list_lock);
        (*spares).bams = (*cl).bams;
        (*spares).next = (*fdl).bl.cast::<spare_bams_layout>();
        (*fdl).bl = spares.cast::<c_void>();
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).bam_list_lock);
        (*cl).bams = std::ptr::null_mut();

        // Detect if a multi-seq container.
        cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_RI].cast());
        multi_ref = ((*(*cl).stats[DS_RI]).nvals > 1) as c_int;
        crate::htslib_rs::c_compat::pthread_mutex_lock(&raw mut (*fdl).metrics_lock);
        (*fdl).last_ri_count = (*(*cl).stats[DS_RI]).nvals;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&raw mut (*fdl).metrics_lock);

        if multi_ref != 0 {
            hts_log_cstr(
                HTS_LOG_INFO,
                c"cram_encode_container".as_ptr(),
                c"Multi-ref container".as_ptr(),
            );
            (*cl).ref_seq_id = -2;
            (*cl).ref_seq_start = 0;
            (*cl).ref_seq_span = 0;
        }

        // Compute MD5s.
        no_ref = (*cl).no_ref;
        let is_v4: c_int = if (*fdl).version >> 8 >= 4 { 1 } else { 0 };
        {
            let mut i: c_int = 0;
            while i < (*cl).curr_slice {
                let s = *(*cl).slices.offset(i as isize);
                if ((*fdl).version >> 8) != 1 {
                    let hdr_blk = (*s).hdr;
                    if (*hdr_blk).ref_seq_id >= 0 && (*cl).multi_seq == 0 && no_ref == 0 {
                        let md5 = crate::htslib_rs::md5::hts_md5_init();
                        if md5.is_null() {
                            return -1;
                        }
                        let off = ((*hdr_blk).ref_seq_start - (*cl).ref_start) as isize;
                        crate::htslib_rs::md5::hts_md5_update(
                            md5,
                            (*cl).ref_.offset(off).cast::<c_void>(),
                            (*hdr_blk).ref_seq_span as std::ffi::c_ulong,
                        );
                        crate::htslib_rs::md5::hts_md5_final(
                            (&raw mut (*hdr_blk).md5).cast::<c_uchar>(),
                            md5,
                        );
                        crate::htslib_rs::md5::hts_md5_destroy(md5);
                    } else {
                        libc::memset((&raw mut (*hdr_blk).md5).cast::<c_void>(), 0, 16);
                    }
                }
                i += 1;
            }
        }

        (*cl).num_records = 0;
        (*cl).num_blocks = 1; // cram_block_compression_hdr
        (*cl).length = 0;

        let vv_ptr = (&raw mut (*fdl).vv).cast::<c_void>();
        let version = (*fdl).version;

        // === DS_BF ===
        (*hl).codecs[DS_BF] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_BF].cast()),
            (*cl).stats[DS_BF].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_BF]).nvals != 0 && (*hl).codecs[DS_BF].is_null() {
            return -1;
        }

        // === DS_CF ===
        (*hl).codecs[DS_CF] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_CF].cast()),
            (*cl).stats[DS_CF].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_CF]).nvals != 0 && (*hl).codecs[DS_CF].is_null() {
            return -1;
        }

        // === DS_AP ===
        if (*cl).pos_sorted != 0 || (version >> 8) >= 4 {
            if (*cl).pos_sorted != 0 {
                (*hl).codecs[DS_AP] = cram_cram_codecs_c_3928_cram_encoder_init(
                    cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_AP].cast()),
                    (*cl).stats[DS_AP].cast(),
                    if is_v4 != 0 { E_LONG } else { E_INT },
                    std::ptr::null_mut(),
                    version,
                    vv_ptr,
                );
            } else {
                (*hl).codecs[DS_AP] = cram_cram_codecs_c_3928_cram_encoder_init(
                    if is_v4 != 0 {
                        E_VARINT_SIGNED
                    } else {
                        E_EXTERNAL
                    },
                    std::ptr::null_mut(),
                    if is_v4 != 0 { E_LONG } else { E_INT },
                    std::ptr::null_mut(),
                    version,
                    vv_ptr,
                );
            }
        } else {
            // Removed BETA in v4.0.
            let mut p: [i64; 2] = [0, (*cl).max_apos];
            (*hl).codecs[DS_AP] = cram_cram_codecs_c_3928_cram_encoder_init(
                E_BETA,
                std::ptr::null_mut(),
                if is_v4 != 0 { E_LONG } else { E_INT },
                (&raw mut p).cast::<c_void>(),
                version,
                vv_ptr,
            );
        }
        if (*hl).codecs[DS_AP].is_null() {
            return -1;
        }

        // === DS_RG ===
        (*hl).codecs[DS_RG] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_RG].cast()),
            (*cl).stats[DS_RG].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_RG]).nvals != 0 && (*hl).codecs[DS_RG].is_null() {
            return -1;
        }

        // === DS_MQ ===
        (*hl).codecs[DS_MQ] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_MQ].cast()),
            (*cl).stats[DS_MQ].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_MQ]).nvals != 0 && (*hl).codecs[DS_MQ].is_null() {
            return -1;
        }

        // === DS_NS ===
        (*hl).codecs[DS_NS] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_NS].cast()),
            (*cl).stats[DS_NS].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_NS]).nvals != 0 && (*hl).codecs[DS_NS].is_null() {
            return -1;
        }

        // === DS_MF ===
        (*hl).codecs[DS_MF] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_MF].cast()),
            (*cl).stats[DS_MF].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_MF]).nvals != 0 && (*hl).codecs[DS_MF].is_null() {
            return -1;
        }

        // === DS_TS ===
        (*hl).codecs[DS_TS] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_TS].cast()),
            (*cl).stats[DS_TS].cast(),
            if is_v4 != 0 { E_LONG } else { E_INT },
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_TS]).nvals != 0 && (*hl).codecs[DS_TS].is_null() {
            return -1;
        }

        // === DS_NP ===
        (*hl).codecs[DS_NP] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_NP].cast()),
            (*cl).stats[DS_NP].cast(),
            if is_v4 != 0 { E_LONG } else { E_INT },
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_NP]).nvals != 0 && (*hl).codecs[DS_NP].is_null() {
            return -1;
        }

        // === DS_NF ===
        (*hl).codecs[DS_NF] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_NF].cast()),
            (*cl).stats[DS_NF].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_NF]).nvals != 0 && (*hl).codecs[DS_NF].is_null() {
            return -1;
        }

        // === DS_RL ===
        (*hl).codecs[DS_RL] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_RL].cast()),
            (*cl).stats[DS_RL].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_RL]).nvals != 0 && (*hl).codecs[DS_RL].is_null() {
            return -1;
        }

        // === DS_FN ===
        (*hl).codecs[DS_FN] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_FN].cast()),
            (*cl).stats[DS_FN].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_FN]).nvals != 0 && (*hl).codecs[DS_FN].is_null() {
            return -1;
        }

        // === DS_FC ===
        (*hl).codecs[DS_FC] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_FC].cast()),
            (*cl).stats[DS_FC].cast(),
            E_BYTE,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_FC]).nvals != 0 && (*hl).codecs[DS_FC].is_null() {
            return -1;
        }

        // === DS_FP ===
        (*hl).codecs[DS_FP] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_FP].cast()),
            (*cl).stats[DS_FP].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_FP]).nvals != 0 && (*hl).codecs[DS_FP].is_null() {
            return -1;
        }

        // === DS_DL ===
        (*hl).codecs[DS_DL] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_DL].cast()),
            (*cl).stats[DS_DL].cast(),
            E_INT,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_DL]).nvals != 0 && (*hl).codecs[DS_DL].is_null() {
            return -1;
        }

        // === DS_BA ===
        (*hl).codecs[DS_BA] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_BA].cast()),
            (*cl).stats[DS_BA].cast(),
            E_BYTE,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_BA]).nvals != 0 && (*hl).codecs[DS_BA].is_null() {
            return -1;
        }

        // === DS_BB (v3+ only) ===
        if (version >> 8) >= 3 {
            let mut e = cram_byte_array_len_encoder_dat_layout {
                len_encoding: if (version >> 8) >= 4 {
                    E_VARINT_UNSIGNED
                } else {
                    E_EXTERNAL
                },
                val_encoding: E_EXTERNAL,
                len_dat: cram_data_series_id_ptr(DS_BB_len),
                val_dat: cram_data_series_id_ptr(DS_BB),
                len_codec: std::ptr::null_mut(),
                val_codec: std::ptr::null_mut(),
            };
            (*hl).codecs[DS_BB] = cram_cram_codecs_c_3928_cram_encoder_init(
                E_BYTE_ARRAY_LEN,
                std::ptr::null_mut(),
                E_BYTE_ARRAY,
                (&raw mut e).cast::<c_void>(),
                version,
                vv_ptr,
            );
            if (*hl).codecs[DS_BB].is_null() {
                return -1;
            }
        } else {
            (*hl).codecs[DS_BB] = std::ptr::null_mut();
        }

        // === DS_BS ===
        (*hl).codecs[DS_BS] = cram_cram_codecs_c_3928_cram_encoder_init(
            cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_BS].cast()),
            (*cl).stats[DS_BS].cast(),
            E_BYTE,
            std::ptr::null_mut(),
            version,
            vv_ptr,
        );
        if (*(*cl).stats[DS_BS]).nvals != 0 && (*hl).codecs[DS_BS].is_null() {
            return -1;
        }

        // === v1 vs v2/3/4 branching for TC/TN vs TL/RI/RS/PD/HC/SC ===
        if (version >> 8) == 1 {
            (*hl).codecs[DS_TL] = std::ptr::null_mut();
            (*hl).codecs[DS_RI] = std::ptr::null_mut();
            (*hl).codecs[DS_RS] = std::ptr::null_mut();
            (*hl).codecs[DS_PD] = std::ptr::null_mut();
            (*hl).codecs[DS_HC] = std::ptr::null_mut();
            (*hl).codecs[DS_SC] = std::ptr::null_mut();

            (*hl).codecs[DS_TC] = cram_cram_codecs_c_3928_cram_encoder_init(
                cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_TC].cast()),
                (*cl).stats[DS_TC].cast(),
                E_BYTE,
                std::ptr::null_mut(),
                version,
                vv_ptr,
            );
            if (*(*cl).stats[DS_TC]).nvals != 0 && (*hl).codecs[DS_TC].is_null() {
                return -1;
            }

            (*hl).codecs[DS_TN] = cram_cram_codecs_c_3928_cram_encoder_init(
                cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_TN].cast()),
                (*cl).stats[DS_TN].cast(),
                E_INT,
                std::ptr::null_mut(),
                version,
                vv_ptr,
            );
            if (*(*cl).stats[DS_TN]).nvals != 0 && (*hl).codecs[DS_TN].is_null() {
                return -1;
            }
        } else {
            (*hl).codecs[DS_TC] = std::ptr::null_mut();
            (*hl).codecs[DS_TN] = std::ptr::null_mut();

            // === DS_TL ===
            (*hl).codecs[DS_TL] = cram_cram_codecs_c_3928_cram_encoder_init(
                cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_TL].cast()),
                (*cl).stats[DS_TL].cast(),
                E_INT,
                std::ptr::null_mut(),
                version,
                vv_ptr,
            );
            if (*(*cl).stats[DS_TL]).nvals != 0 && (*hl).codecs[DS_TL].is_null() {
                return -1;
            }

            // === DS_RI ===
            (*hl).codecs[DS_RI] = cram_cram_codecs_c_3928_cram_encoder_init(
                cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_RI].cast()),
                (*cl).stats[DS_RI].cast(),
                E_INT,
                std::ptr::null_mut(),
                version,
                vv_ptr,
            );
            if (*(*cl).stats[DS_RI]).nvals != 0 && (*hl).codecs[DS_RI].is_null() {
                return -1;
            }

            // === DS_RS ===
            (*hl).codecs[DS_RS] = cram_cram_codecs_c_3928_cram_encoder_init(
                cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_RS].cast()),
                (*cl).stats[DS_RS].cast(),
                E_INT,
                std::ptr::null_mut(),
                version,
                vv_ptr,
            );
            if (*(*cl).stats[DS_RS]).nvals != 0 && (*hl).codecs[DS_RS].is_null() {
                return -1;
            }

            // === DS_PD ===
            (*hl).codecs[DS_PD] = cram_cram_codecs_c_3928_cram_encoder_init(
                cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_PD].cast()),
                (*cl).stats[DS_PD].cast(),
                E_INT,
                std::ptr::null_mut(),
                version,
                vv_ptr,
            );
            if (*(*cl).stats[DS_PD]).nvals != 0 && (*hl).codecs[DS_PD].is_null() {
                return -1;
            }

            // === DS_HC ===
            (*hl).codecs[DS_HC] = cram_cram_codecs_c_3928_cram_encoder_init(
                cram_cram_stats_c_134_cram_stats_encoding(fd.cast(), (*cl).stats[DS_HC].cast()),
                (*cl).stats[DS_HC].cast(),
                E_INT,
                std::ptr::null_mut(),
                version,
                vv_ptr,
            );
            if (*(*cl).stats[DS_HC]).nvals != 0 && (*hl).codecs[DS_HC].is_null() {
                return -1;
            }

            // === DS_SC ===
            // The C code uses `if (1)` followed by an unreachable `else` —
            // mirror just the live branch.
            let mut i2: [c_int; 2] = [0, DS_SC as c_int];
            (*hl).codecs[DS_SC] = cram_cram_codecs_c_3928_cram_encoder_init(
                E_BYTE_ARRAY_STOP,
                std::ptr::null_mut(),
                E_BYTE_ARRAY,
                (&raw mut i2).cast::<c_void>(),
                version,
                vv_ptr,
            );
            let _ = DS_SC_len; // silence unused warning when v1-only field unused
            if (*hl).codecs[DS_SC].is_null() {
                return -1;
            }
        }

        // === DS_IN ===
        {
            let mut i2: [c_int; 2] = [0, DS_IN as c_int];
            (*hl).codecs[DS_IN] = cram_cram_codecs_c_3928_cram_encoder_init(
                E_BYTE_ARRAY_STOP,
                std::ptr::null_mut(),
                E_BYTE_ARRAY,
                (&raw mut i2).cast::<c_void>(),
                version,
                vv_ptr,
            );
            if (*hl).codecs[DS_IN].is_null() {
                return -1;
            }
        }

        // === DS_QS ===
        (*hl).codecs[DS_QS] = cram_cram_codecs_c_3928_cram_encoder_init(
            E_EXTERNAL,
            std::ptr::null_mut(),
            E_BYTE,
            cram_data_series_id_ptr(DS_QS),
            version,
            vv_ptr,
        );
        if (*hl).codecs[DS_QS].is_null() {
            return -1;
        }

        // === DS_RN ===
        {
            let mut i2: [c_int; 2] = [0, DS_RN as c_int];
            (*hl).codecs[DS_RN] = cram_cram_codecs_c_3928_cram_encoder_init(
                E_BYTE_ARRAY_STOP,
                std::ptr::null_mut(),
                E_BYTE_ARRAY,
                (&raw mut i2).cast::<c_void>(),
                version,
                vv_ptr,
            );
            if (*hl).codecs[DS_RN].is_null() {
                return -1;
            }
        }

        // Encode slices.
        {
            let mut i: c_int = 0;
            while i < (*cl).curr_slice {
                hts_log_cstr(
                    HTS_LOG_INFO,
                    c"cram_encode_container".as_ptr(),
                    c"Encode slice".as_ptr(),
                );
                let sl_layout = *(*cl).slices.offset(i as isize);
                let s: *mut cram_slice = sl_layout.cast::<cram_slice>();
                let local_embed_ref: c_int =
                    if embed_ref > 0 && (*(*sl_layout).hdr).ref_seq_id != -1 {
                        1
                    } else {
                        0
                    };
                if cram_cram_encode_c_1097_cram_encode_slice(fd, c, h, s, local_embed_ref) != 0 {
                    return -1;
                }
                i += 1;
            }
        }

        // Create compression header.
        {
            (*hl).ref_seq_id = (*cl).ref_seq_id;
            (*hl).ref_seq_start = (*cl).ref_seq_start;
            (*hl).ref_seq_span = (*cl).ref_seq_span;
            (*hl).num_records = (*cl).num_records;
            (*hl).qs_seq_orient = (*cl).qs_seq_orient;
            // ap_delta = pos_sorted (slight misnomer in C).
            (*hl).ap_delta = (*cl).pos_sorted;
            libc::memcpy(
                (&raw mut (*hl).substitution_matrix).cast::<c_void>(),
                CRAM_SUBST_MATRIX.as_ptr().cast::<c_void>(),
                20,
            );
            c_hdr = cram_cram_encode_c_2810_cram_encode_compression_header(fd, c, h, embed_ref);
            if c_hdr.is_null() {
                return -1;
            }
        }

        // Compute landmarks.
        (*cl).num_landmarks = (*cl).curr_slice;
        (*cl).landmark =
            malloc((std::mem::size_of::<i32>() as u64).wrapping_mul((*cl).num_landmarks as u64))
                .cast::<i32>();
        if (*cl).landmark.is_null() {
            return -1;
        }

        // Compute slice_offset: simulate writing the first block.
        let c_hdr_l = c_hdr.cast::<cram_block_layout>();
        let varint_size = (*fdl).vv.varint_size.unwrap();
        let mut slice_offset: c_int = if (*c_hdr_l).method == 0 {
            (*c_hdr_l).uncomp_size
        } else {
            (*c_hdr_l).comp_size
        };
        let v3plus_extra: c_int = if (version >> 8) >= 3 { 4 } else { 0 };
        slice_offset += 2
            + v3plus_extra
            + varint_size((*c_hdr_l).content_id as i64)
            + varint_size((*c_hdr_l).comp_size as i64)
            + varint_size((*c_hdr_l).uncomp_size as i64);

        let first_slice = *(*cl).slices.offset(0);
        (*cl).ref_seq_id = (*(*first_slice).hdr).ref_seq_id;
        if (*cl).ref_seq_id == -1 && (*fdl).version >= 0x301 {
            (*cl).ref_seq_start = 0;
            (*cl).ref_seq_span = 0;
        } else {
            (*cl).ref_seq_start = (*(*first_slice).hdr).ref_seq_start;
            (*cl).ref_seq_span = (*(*first_slice).hdr).ref_seq_span;
        }

        {
            let mut i: c_int = 0;
            while i < (*cl).curr_slice {
                let sl_layout: *mut cram_slice_layout = *(*cl).slices.offset(i as isize);

                (*cl).num_blocks += (*(*sl_layout).hdr).num_blocks + 1; // slice header
                *(*cl).landmark.offset(i as isize) = slice_offset;

                if (*(*sl_layout).hdr).ref_seq_start + (*(*sl_layout).hdr).ref_seq_span
                    > (*cl).ref_seq_start + (*cl).ref_seq_span
                {
                    (*cl).ref_seq_span = (*(*sl_layout).hdr).ref_seq_start
                        + (*(*sl_layout).hdr).ref_seq_span
                        - (*cl).ref_seq_start;
                }

                let hb = (*sl_layout).hdr_block;
                slice_offset += if (*hb).method == 0 {
                    (*hb).uncomp_size
                } else {
                    (*hb).comp_size
                };
                slice_offset += 2
                    + v3plus_extra
                    + varint_size((*hb).content_id as i64)
                    + varint_size((*hb).comp_size as i64)
                    + varint_size((*hb).uncomp_size as i64);

                let mut j: c_int = 0;
                while j < (*(*sl_layout).hdr).num_blocks {
                    let b = *(*sl_layout).block.offset(j as isize);
                    slice_offset += 2
                        + v3plus_extra
                        + varint_size((*b).content_id as i64)
                        + varint_size((*b).comp_size as i64)
                        + varint_size((*b).uncomp_size as i64);
                    slice_offset += if (*b).method == 0 {
                        (*b).uncomp_size
                    } else {
                        (*b).comp_size
                    };
                    j += 1;
                }
                i += 1;
            }
        }
        (*cl).length += slice_offset; // just past the final slice

        (*cl).comp_hdr_block = c_hdr.cast::<cram_block_layout>();

        if (*cl).ref_seq_id >= 0 {
            if (*cl).ref_free != 0 {
                free((*cl).ref_.cast());
                (*cl).ref_ = std::ptr::null_mut();
            } else {
                cram_cram_io_c_3213_cram_ref_decr((*fdl).refs.cast(), (*cl).ref_seq_id);
            }
        }

        // Release the ref-bumped entries we cached up-front for unsorted patterns.
        if no_ref == 0 && !(*cl).refs_used.is_null() {
            let mut i: c_int = 0;
            while i < (*(*fdl).refs).nref {
                if *(*cl).refs_used.offset(i as isize) != 0 {
                    cram_cram_io_c_3213_cram_ref_decr((*fdl).refs.cast(), i);
                }
                i += 1;
            }
        }

        return 0;
    }
}

pub unsafe fn cram_cram_encode_c_1246_bam_data_end(b: *mut bam1_t) -> *const c_char {
    (*b).data.add((*b).l_data as usize).cast()
}

pub unsafe fn cram_cram_encode_c_1253_bam_aux2i_end(
    mut aux: *const u8,
    aux_end: *const u8,
) -> c_int {
    let type_ = *aux;
    aux = aux.add(1);
    match type_ {
        b'c' => {
            if aux_end.offset_from(aux) < 1 {
                *__errno_location() = EINVAL;
                return 0;
            }
            *(aux.cast::<i8>()) as c_int
        }
        b'C' => {
            if aux_end.offset_from(aux) < 1 {
                *__errno_location() = EINVAL;
                return 0;
            }
            *aux as c_int
        }
        b's' => {
            if aux_end.offset_from(aux) < 2 {
                *__errno_location() = EINVAL;
                return 0;
            }
            i16::from_le_bytes([*aux, *aux.add(1)]) as c_int
        }
        b'S' => {
            if aux_end.offset_from(aux) < 2 {
                *__errno_location() = EINVAL;
                return 0;
            }
            u16::from_le_bytes([*aux, *aux.add(1)]) as c_int
        }
        b'i' => {
            if aux_end.offset_from(aux) < 4 {
                *__errno_location() = EINVAL;
                return 0;
            }
            i32::from_le_bytes([*aux, *aux.add(1), *aux.add(2), *aux.add(3)]) as c_int
        }
        b'I' => {
            if aux_end.offset_from(aux) < 4 {
                *__errno_location() = EINVAL;
                return 0;
            }
            u32::from_le_bytes([*aux, *aux.add(1), *aux.add(2), *aux.add(3)]) as c_int
        }
        _ => {
            *__errno_location() = EINVAL;
            0
        }
    }
}

pub unsafe fn cram_cram_encode_c_1301_expected_template_count(b: *mut bam1_t) -> c_int {
    let mut expected = if ((*b).core.flag as c_int & BAM_FPAIRED) != 0 {
        2
    } else {
        1
    };

    let tc_tag = [b'T' as c_char, b'C' as c_char, 0];
    let tc = bam_aux_get(b, tc_tag.as_ptr());
    if !tc.is_null() {
        let n = cram_cram_encode_c_1253_bam_aux2i_end(
            tc,
            cram_cram_encode_c_1246_bam_data_end(b).cast(),
        );
        if expected < n {
            expected = n;
        }
    }

    let sa_tag = [b'S' as c_char, b'A' as c_char, 0];
    if tc.is_null() && !bam_aux_get(b, sa_tag.as_ptr()).is_null() {
        expected = c_int::MAX;
    }

    expected
}

pub unsafe fn cram_cram_encode_c_1476_next_cigar_op(
    cigar: *mut u32,
    ncigar: u32,
    skip: *mut c_int,
    spos: *mut c_int,
    cig_ind: *mut u32,
    cig_op: *mut u32,
    cig_len: *mut u32,
) -> c_int {
    loop {
        while *cig_len == 0 {
            if *cig_ind < ncigar {
                *cig_op = *cigar.add(*cig_ind as usize) & BAM_CIGAR_MASK;
                *cig_len = *cigar.add(*cig_ind as usize) >> BAM_CIGAR_SHIFT;
                *cig_ind += 1;
            } else {
                return -1;
            }
        }

        if *skip.add(*cig_op as usize) != 0 {
            *spos += (bam_cigar_type(*cig_op as c_int) & 1) * *cig_len as c_int;
            *cig_len = 0;
            continue;
        }

        *cig_len -= 1;
        break;
    }

    *cig_op as c_int
}

// Native cram_put_bam_seq (htslib/cram/cram_encode.c:4049). Appends a BAM
// record to the current CRAM container's slice; allocates the container/
// slice on first call and rolls them over when full via
// cram_next_container_native.
pub unsafe fn cram_cram_encode_c_4049_cram_put_bam_seq(fd: *mut cram_fd, b: *mut bam1_t) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();

    // First call: allocate the initial container and seed from the fd.
    if (*fdl).ctr.is_null() {
        let new_ctr = cram_new_container((*fdl).seqs_per_slice, (*fdl).slices_per_container);
        if new_ctr.is_null() {
            return -1;
        }
        (*fdl).ctr = new_ctr.cast();
        (*(*fdl).ctr).record_counter = (*fdl).record_counter;
        crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).ref_lock);
        (*(*fdl).ctr).no_ref = (*fdl).no_ref;
        (*(*fdl).ctr).embed_ref = (*fdl).embed_ref;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
    }
    let mut c = (*fdl).ctr;
    let embed_ref = (*c).embed_ref;

    let core_tid = (*b).core.tid;
    if (*c).slice.is_null()
        || (*c).curr_rec == (*c).max_rec
        || (core_tid != (*c).curr_ref && (*c).curr_ref >= -1)
        || (*c).s_num_bases.wrapping_add((*c).s_aux_bytes) >= (*fdl).bases_per_slice as u64
    {
        let mut multi_seq: c_int = ((*fdl).multi_seq == 1) as c_int;
        let curr_ref_local: c_int = if !(*c).slice.is_null() {
            (*c).curr_ref
        } else {
            core_tid
        };
        // Multi-seq auto-bump heuristic (htslib/cram/cram_encode.c:4096).
        if (*fdl).multi_seq == -1
            && (*c).curr_rec < (*c).max_rec / 4 + 10
            && (*fdl).last_slice != 0
            && (*fdl).last_slice < (*c).max_rec / 4 + 10
            && embed_ref <= 0
        {
            multi_seq = 1;
        } else if (*fdl).multi_seq == 1 {
            crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).metrics_lock);
            if (*fdl).last_ri_count <= (*c).max_slice && (*fdl).multi_seq_user != 1 {
                multi_seq = 0;
            }
            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).metrics_lock);
        }
        let slice_rec = (*c).slice_rec;
        let curr_rec_snapshot = (*c).curr_rec;
        if ((*fdl).version >> 8) == 1
            || (*c).curr_rec == (*c).max_rec
            || (*fdl).multi_seq != 1
            || (*c).slice.is_null()
            || (*c).s_num_bases.wrapping_add((*c).s_aux_bytes) >= (*fdl).bases_per_slice as u64
        {
            let new_c = cram_next_container_native(fd, b);
            if new_c.is_null() {
                if !(*fdl).ctr.is_null() {
                    (*fdl).ctr_mt = (*fdl).ctr;
                    (*fdl).ctr = std::ptr::null_mut();
                }
                return -1;
            }
            c = new_c;
        }
        if multi_seq == 0 && (*fdl).multi_seq == 1 && (*fdl).multi_seq_user == -1 {
            (*fdl).multi_seq = -1;
        } else if multi_seq != 0 {
            (*fdl).multi_seq = 1;
            (*c).multi_seq = 1;
            (*c).pos_sorted = 0;
            crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).ref_lock);
            if (*fdl).embed_ref > 0 && (*c).curr_rec == 0 && (*c).curr_slice == 0 {
                (*fdl).embed_ref = 0;
                (*c).embed_ref = (*fdl).embed_ref;
                (*fdl).no_ref = 1;
                (*c).no_ref = (*fdl).no_ref;
            }
            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
            if (*c).refs_used.is_null() {
                crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).ref_lock);
                let nref = (*(*fdl).refs.cast::<refs_t_layout>()).nref;
                (*c).refs_used = crate::htslib_rs::c_compat::calloc(
                    nref as u64,
                    std::mem::size_of::<c_int>() as u64,
                )
                .cast();
                crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
                if (*c).refs_used.is_null() {
                    return -1;
                }
            }
        }
        (*fdl).last_slice = curr_rec_snapshot - slice_rec;
        (*c).slice_rec = (*c).curr_rec;
        if core_tid >= 0
            && curr_ref_local >= 0
            && core_tid != curr_ref_local
            && embed_ref <= 0
            && (*fdl).unsorted == 0
            && multi_seq != 0
        {
            if (*c).refs_used.is_null() {
                crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).ref_lock);
                let nref = (*(*fdl).refs.cast::<refs_t_layout>()).nref;
                (*c).refs_used = crate::htslib_rs::c_compat::calloc(
                    nref as u64,
                    std::mem::size_of::<c_int>() as u64,
                )
                .cast();
                crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
                if (*c).refs_used.is_null() {
                    return -1;
                }
            } else if !(*c).refs_used.is_null() && *(*c).refs_used.add(core_tid as usize) != 0 {
                crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).ref_lock);
                (*fdl).unsorted = 1;
                (*fdl).multi_seq = 1;
                crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
            }
        }
        (*c).curr_ref = core_tid;
        if !(*c).refs_used.is_null() && (*c).curr_ref >= 0 {
            *(*c).refs_used.add((*c).curr_ref as usize) += 1;
        }
    }

    if (*c).bams.is_null() {
        crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).bam_list_lock);
        if !(*fdl).bl.is_null() {
            let spare = (*fdl).bl.cast::<spare_bams_layout>();
            (*c).bams = (*spare).bams;
            (*fdl).bl = (*spare).next.cast();
            crate::htslib_rs::c_compat::free(spare.cast());
        } else {
            (*c).bams = crate::htslib_rs::c_compat::calloc(
                (*c).max_c_rec as u64,
                std::mem::size_of::<*mut bam1_t>() as u64,
            )
            .cast();
            if (*c).bams.is_null() {
                crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).bam_list_lock);
                return -1;
            }
        }
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).bam_list_lock);
    }

    let slot = (*c).bams.add((*c).curr_c_rec as usize);
    if !(*slot).is_null() {
        if crate::htslib_rs::sam::bam_copy1(*slot, b).is_null() {
            return -1;
        }
    } else {
        *slot = crate::htslib_rs::sam::bam_dup1(b);
        if (*slot).is_null() {
            return -1;
        }
    }

    let l_qseq = (*b).core.l_qseq;
    if l_qseq != 0 {
        (*c).s_num_bases = (*c).s_num_bases.wrapping_add(l_qseq as u64);
    } else {
        let qlen = crate::htslib_rs::sam::bam_cigar2qlen(
            (*b).core.n_cigar as c_int,
            (*b).data.add((*b).core.l_qname as usize).cast::<u32>(),
        );
        if qlen > 100_000_000 {
            return -1;
        }
        (*c).s_num_bases = (*c).s_num_bases.wrapping_add(qlen as u64);
    }
    (*c).curr_rec += 1;
    (*c).curr_c_rec += 1;
    (*c).s_aux_bytes = (*c).s_aux_bytes.wrapping_add(
        ((*b).l_data as u32)
            .wrapping_sub((*b).core.n_cigar << 2)
            .wrapping_sub((*b).core.l_qname as u32)
            .wrapping_sub((*b).core.l_qseq as u32)
            .wrapping_sub(((*b).core.l_qseq + 1) as u32 >> 1) as u64,
    );
    (*c).n_mapped = (*c)
        .n_mapped
        .wrapping_add(if ((*b).core.flag as c_int) & BAM_FUNMAP != 0 {
            0
        } else {
            1
        });
    (*fdl).record_counter += 1;
    0
}
