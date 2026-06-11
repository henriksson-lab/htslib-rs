// Functions translated from htslib/cram/cram_codecs.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int, c_void};

use super::*;

unsafe fn raw_block_mut<'a>(block: *mut cram_block) -> Option<&'a mut cram_block> {
    block.as_mut()
}

fn cram_block_layout_mut(block: &mut cram_block) -> &mut cram_block_layout {
    unsafe { &mut *(block as *mut cram_block).cast::<cram_block_layout>() }
}

unsafe fn raw_bytes<'a>(bytes: *const c_char, len: c_int) -> Option<&'a [u8]> {
    if len < 0 || (bytes.is_null() && len != 0) {
        return None;
    }
    if len == 0 {
        return Some(&[]);
    }
    Some(std::slice::from_raw_parts(bytes.cast::<u8>(), len as usize))
}

unsafe fn raw_c_string_bytes<'a>(bytes: *const c_char) -> Option<&'a [u8]> {
    if bytes.is_null() {
        return None;
    }

    let mut len = 0usize;
    while *bytes.add(len) != 0 {
        len += 1;
    }
    Some(std::slice::from_raw_parts(bytes.cast::<u8>(), len))
}

unsafe fn append_bytes_to_block(block: &mut cram_block, bytes: &[u8]) -> c_int {
    cram_cram_io_h_248_block_append(
        (block as *mut cram_block).cast(),
        bytes.as_ptr().cast(),
        bytes.len(),
    )
}

unsafe fn extract_block_bytes<'a>(block: &mut cram_block, len: c_int) -> Option<&'a [u8]> {
    let bytes = cram_cram_codecs_c_319_cram_extract_block(block, len);
    raw_bytes(bytes, len)
}

unsafe fn append_optional_c_prefix(block: &mut cram_block, prefix: *const c_char) -> Option<c_int> {
    let Some(bytes) = raw_c_string_bytes(prefix) else {
        return Some(0);
    };
    if append_bytes_to_block(block, bytes) == 0 {
        Some(bytes.len() as c_int)
    } else {
        None
    }
}

unsafe fn external_codec_mut<'a>(codec: *mut c_void) -> Option<&'a mut cram_codec_external_layout> {
    codec.cast::<cram_codec_external_layout>().as_mut()
}

unsafe fn varint_codec_mut<'a>(codec: *mut c_void) -> Option<&'a mut cram_codec_varint_layout> {
    codec.cast::<cram_codec_varint_layout>().as_mut()
}

unsafe fn const_codec_mut<'a>(codec: *mut c_void) -> Option<&'a mut cram_codec_const_layout> {
    codec.cast::<cram_codec_const_layout>().as_mut()
}

fn cram_cram_codecs_c_73_get_bit_MSB_ref(block: &mut cram_block_layout) -> c_int {
    unsafe {
        if block.byte > block.alloc {
            return -1;
        }

        let val = *block.data.add(block.byte) >> block.bit;
        block.bit -= 1;
        if block.bit == -1 {
            block.bit = 7;
            block.byte += 1;
        }

        (val & 1) as c_int
    }
}

pub unsafe fn cram_cram_codecs_c_73_get_bit_MSB(block: *mut cram_block) -> c_int {
    let Some(block) = block.as_mut() else {
        return -1;
    };
    cram_cram_codecs_c_73_get_bit_MSB_ref(cram_block_layout_mut(block))
}

fn cram_cram_codecs_c_95_get_one_bits_MSB_ref(block: &mut cram_block_layout) -> c_int {
    unsafe {
        let mut n = 0;
        if block.byte >= block.uncomp_size as usize {
            return -1;
        }

        loop {
            let b = *block.data.add(block.byte) >> block.bit;
            block.bit -= 1;
            if block.bit == -1 {
                block.bit = 7;
                block.byte += 1;
                if block.byte == block.uncomp_size as usize && (b & 1) != 0 {
                    return -1;
                }
            }
            n += 1;
            if (b & 1) == 0 {
                break;
            }
        }

        n - 1
    }
}

pub unsafe fn cram_cram_codecs_c_95_get_one_bits_MSB(block: *mut cram_block) -> c_int {
    let Some(block) = block.as_mut() else {
        return -1;
    };
    cram_cram_codecs_c_95_get_one_bits_MSB_ref(cram_block_layout_mut(block))
}

fn cram_cram_codecs_c_113_get_zero_bits_MSB_ref(block: &mut cram_block_layout) -> c_int {
    unsafe {
        let mut n = 0;
        if block.byte >= block.uncomp_size as usize {
            return -1;
        }

        loop {
            let b = *block.data.add(block.byte) >> block.bit;
            block.bit -= 1;
            if block.bit == -1 {
                block.bit = 7;
                block.byte += 1;
                if block.byte == block.uncomp_size as usize && (b & 1) == 0 {
                    return -1;
                }
            }
            n += 1;
            if (b & 1) != 0 {
                break;
            }
        }

        n - 1
    }
}

pub unsafe fn cram_cram_codecs_c_113_get_zero_bits_MSB(block: *mut cram_block) -> c_int {
    let Some(block) = block.as_mut() else {
        return -1;
    };
    cram_cram_codecs_c_113_get_zero_bits_MSB_ref(cram_block_layout_mut(block))
}

fn cram_cram_codecs_c_133_store_bit_MSB_ref(block: &mut cram_block_layout, bit: libc::c_uint) {
    unsafe {
        if block.byte >= block.alloc {
            block.alloc = if block.alloc != 0 {
                block.alloc * 2
            } else {
                1024
            };
            block.data = realloc(block.data.cast(), block.alloc as u64).cast::<u8>();
        }

        if bit != 0 {
            *block.data.add(block.byte) |= 1 << block.bit;
        }

        block.bit -= 1;
        if block.bit == -1 {
            block.bit = 7;
            block.byte += 1;
            *block.data.add(block.byte) = 0;
        }
    }
}

pub unsafe fn cram_cram_codecs_c_133_store_bit_MSB(block: *mut cram_block, bit: libc::c_uint) {
    if let Some(block) = block.as_mut() {
        cram_cram_codecs_c_133_store_bit_MSB_ref(cram_block_layout_mut(block), bit);
    }
}

fn cram_cram_codecs_c_169_get_bits_MSB_ref(block: &mut cram_block_layout, mut nbits: c_int) -> i64 {
    unsafe {
        let mut val = 0u64;

        if nbits <= block.bit + 1 {
            val = ((*block.data.add(block.byte) >> (block.bit - (nbits - 1))) as u16
                & ((1u16 << nbits) - 1)) as u64;
            block.bit -= nbits;
            if block.bit == -1 {
                block.bit = 7;
                block.byte += 1;
            }
            return val as i64;
        }

        while nbits > 0 {
            val <<= 1;
            val |= ((*block.data.add(block.byte) >> block.bit) & 1) as u64;
            block.bit -= 1;
            if block.bit < 0 {
                block.byte += 1;
                block.bit &= 7;
            }
            nbits -= 1;
        }

        val as i64
    }
}

pub unsafe fn cram_cram_codecs_c_169_get_bits_MSB(block: *mut cram_block, nbits: c_int) -> i64 {
    let Some(block) = block.as_mut() else {
        return -1;
    };
    cram_cram_codecs_c_169_get_bits_MSB_ref(cram_block_layout_mut(block), nbits)
}

fn cram_cram_codecs_c_259_store_bits_MSB_ref(
    block: &mut cram_block_layout,
    val: u64,
    mut nbits: c_int,
) -> c_int {
    unsafe {
        if block.byte + 8 >= block.alloc {
            if block.byte != 0 {
                block.alloc *= 2;
                block.data = realloc(block.data.cast(), (block.alloc + 8) as u64).cast::<u8>();
                if block.data.is_null() {
                    return -1;
                }
            } else {
                block.alloc = 1024;
                block.data = realloc(block.data.cast(), (block.alloc + 8) as u64).cast::<u8>();
                if block.data.is_null() {
                    return -1;
                }
                *block.data = 0;
            }
        }

        if nbits <= block.bit + 1 {
            *block.data.add(block.byte) |= (val << (block.bit + 1 - nbits)) as u8;
            block.bit -= nbits;
            if block.bit == -1 {
                block.bit = 7;
                block.byte += 1;
                *block.data.add(block.byte) = 0;
            }
            return 0;
        }

        nbits -= block.bit + 1;
        *block.data.add(block.byte) |= (val >> nbits) as u8;
        block.bit = 7;
        block.byte += 1;
        *block.data.add(block.byte) = 0;

        let mut mask = 1u32 << (nbits - 1);
        loop {
            if (val & mask as u64) != 0 {
                *block.data.add(block.byte) |= 1 << block.bit;
            }
            block.bit -= 1;
            if block.bit == -1 {
                block.bit = 7;
                block.byte += 1;
                *block.data.add(block.byte) = 0;
            }
            mask >>= 1;
            nbits -= 1;
            if nbits == 0 {
                break;
            }
        }

        0
    }
}

pub unsafe fn cram_cram_codecs_c_259_store_bits_MSB(
    block: *mut cram_block,
    val: u64,
    nbits: c_int,
) -> c_int {
    let Some(block) = block.as_mut() else {
        return -1;
    };
    cram_cram_codecs_c_259_store_bits_MSB_ref(cram_block_layout_mut(block), val, nbits)
}

fn cram_cram_codecs_c_152_store_bytes_MSB_ref(block: &mut cram_block_layout, bytes: &[c_char]) {
    unsafe {
        if block.bit != 7 {
            block.bit = 7;
            block.byte += 1;
        }

        while block.byte + bytes.len() >= block.alloc {
            block.alloc = if block.alloc != 0 {
                block.alloc * 2
            } else {
                1024
            };
            block.data = realloc(block.data.cast(), block.alloc as u64).cast::<u8>();
        }

        memcpy(
            block.data.add(block.byte).cast(),
            bytes.as_ptr().cast(),
            bytes.len() as u64,
        );
        block.byte += bytes.len();
    }
}

pub unsafe fn cram_cram_codecs_c_152_store_bytes_MSB(
    block: *mut cram_block,
    bytes: *mut c_char,
    len: c_int,
) {
    if block.is_null() || len < 0 || (bytes.is_null() && len != 0) {
        return;
    }
    let bytes = std::slice::from_raw_parts(bytes, len as usize);
    cram_cram_codecs_c_152_store_bytes_MSB_ref(&mut *block.cast::<cram_block_layout>(), bytes);
}

pub unsafe fn cram_cram_codecs_c_319_cram_extract_block(
    b: *mut cram_block,
    size: c_int,
) -> *mut c_char {
    let b = b.cast::<cram_block_layout>();
    let cp = (*b).data.add((*b).idx as usize).cast::<c_char>();
    (*b).idx += size;
    if (*b).idx > (*b).uncomp_size {
        return std::ptr::null_mut();
    }

    cp
}

pub unsafe fn cram_cram_codecs_c_350_cram_external_decode_int(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }

    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get32.unwrap())(&mut cp, endp, &mut err);
    *(out.cast::<i32>()) = val as i32;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;

    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_370_cram_external_decode_long(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }

    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get64.unwrap())(&mut cp, endp, &mut err);
    *(out.cast::<i64>()) = val;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;

    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_390_cram_external_decode_char(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }

    let Some(b) = raw_block_mut(b) else {
        return if *out_size != 0 { -1 } else { 0 };
    };
    let Some(bytes) = extract_block_bytes(b, *out_size) else {
        return -1;
    };

    if !out.is_null() {
        memcpy(out.cast(), bytes.as_ptr().cast(), bytes.len() as u64);
    }
    0
}

pub unsafe fn cram_cram_codecs_c_410_cram_external_decode_block(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out_: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }

    let Some(b) = raw_block_mut(b) else {
        return if *out_size != 0 { -1 } else { 0 };
    };
    let Some(bytes) = extract_block_bytes(b, *out_size) else {
        return -1;
    };

    let Some(out_block) = raw_block_mut(out_.cast()) else {
        return -1;
    };
    append_bytes_to_block(out_block, bytes)
}

pub unsafe fn cram_cram_codecs_c_433_cram_external_decode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_external_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_439_cram_external_decode_size(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id);
    if b.is_null() {
        return -1;
    }

    (*(b.cast::<cram_block_layout>())).uncomp_size
}

pub unsafe fn cram_cram_codecs_c_450_cram_external_get_block(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> *mut cram_block {
    let c = c.cast::<cram_codec_external_layout>();
    cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id)
}

pub unsafe fn cram_cram_codecs_c_454_cram_external_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let ks = &mut *ks;
    if kputsn(b"EXTERNAL(id=", 12, ks) < 0
        || kputw((*c).external.content_id, ks) < 0
        || kputsn(b")", 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_459_cram_external_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    codec: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    if size < 1 {
        return std::ptr::null_mut();
    }

    let mut c = Box::new(cram_codec_external_layout {
        codec: 1,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: std::ptr::null_mut(),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        external: cram_external_decoder_layout {
            content_id: 0,
            type_: 0,
        },
    });
    if (version >> 8) >= 4 {
        if codec != 1 {
            return std::ptr::null_mut();
        }
        c.decode = if option == 5 {
            cram_fn_ptr(cram_cram_codecs_c_410_cram_external_decode_block as usize)
        } else if option == 3 || option == 4 {
            cram_fn_ptr(cram_cram_codecs_c_390_cram_external_decode_char as usize)
        } else {
            return std::ptr::null_mut();
        };
    } else if option == 1 {
        c.decode = cram_fn_ptr(cram_cram_codecs_c_350_cram_external_decode_int as usize);
    } else if option == 2 {
        c.decode = cram_fn_ptr(cram_cram_codecs_c_370_cram_external_decode_long as usize);
    } else if option == 4 || option == 3 {
        c.decode = cram_fn_ptr(cram_cram_codecs_c_390_cram_external_decode_char as usize);
    } else {
        c.decode = cram_fn_ptr(cram_cram_codecs_c_410_cram_external_decode_block as usize);
    }
    c.free = cram_fn_ptr(cram_cram_codecs_c_433_cram_external_decode_free as usize);
    c.size = cram_fn_ptr(cram_cram_codecs_c_439_cram_external_decode_size as usize);
    c.get_block = cram_fn_ptr(cram_cram_codecs_c_450_cram_external_get_block as usize);
    c.describe = cram_fn_ptr(cram_cram_codecs_c_454_cram_external_describe as usize);

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    c.external.content_id =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    if cp.offset_from(data) != size as isize {
        return std::ptr::null_mut();
    }
    c.external.type_ = option;
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_523_cram_external_encode_int(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let i32_ = *(in_.cast::<u32>()) as i32;
    if ((*(*c).vv).varint_put32_blk.unwrap())((*c).out.cast(), i32_) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_529_cram_external_encode_sint(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let i32_ = *(in_.cast::<i32>());
    if ((*(*c).vv).varint_put32s_blk.unwrap())((*c).out.cast(), i32_) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_535_cram_external_encode_long(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let i64_ = *(in_.cast::<u64>()) as i64;
    if ((*(*c).vv).varint_put64_blk.unwrap())((*c).out.cast(), i64_) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_541_cram_external_encode_slong(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let i64_ = *(in_.cast::<i64>());
    if ((*(*c).vv).varint_put64s_blk.unwrap())((*c).out.cast(), i64_) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_547_cram_external_encode_char(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let Some(c) = external_codec_mut(c) else {
        return -1;
    };
    let Some(bytes) = raw_bytes(in_, in_size) else {
        return -1;
    };
    let Some(out_block) = raw_block_mut(c.out.cast()) else {
        return -1;
    };
    append_bytes_to_block(out_block, bytes)
}

pub unsafe fn cram_cram_codecs_c_556_cram_external_encode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_external_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_562_cram_external_encode_store(
    c: *mut c_void,
    b: *mut cram_block,
    prefix: *mut c_char,
    _version: c_int,
) -> c_int {
    let Some(c) = external_codec_mut(c) else {
        return -1;
    };
    let Some(b) = raw_block_mut(b) else {
        return -1;
    };
    let mut tmp = [0u8; 99];
    let mut tp = tmp.as_mut_ptr().cast::<c_char>();
    let tpend = tmp.as_mut_ptr().add(tmp.len()).cast::<c_char>();
    let mut len = match append_optional_c_prefix(b, prefix) {
        Some(len) => len,
        None => return -1,
    };
    let mut r = 0;

    tp = tp.add(((*(c.vv)).varint_put32.unwrap())(tp, tpend, c.external.content_id) as usize);
    let b_raw = (b as *mut cram_block).cast();
    let n = ((*(c.vv)).varint_put32_blk.unwrap())(b_raw, c.codec);
    len += n;
    r |= n;
    let n =
        ((*(c.vv)).varint_put32_blk.unwrap())(b_raw, tp.offset_from(tmp.as_ptr().cast()) as i32);
    len += n;
    r |= n;
    let nbytes = tp.offset_from(tmp.as_ptr().cast()) as usize;
    if append_bytes_to_block(b, &tmp[..nbytes]) != 0 {
        return -1;
    }
    len += nbytes as c_int;

    if r > 0 {
        len
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_586_cram_external_encode_init(
    _st: *mut c_void,
    codec: c_int,
    option: c_int,
    dat: *mut c_void,
    version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_external_layout {
        codec: 1,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_556_cram_external_encode_free as usize),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        external: cram_external_decoder_layout {
            content_id: dat as usize as i32,
            type_: option,
        },
    });
    if (version >> 8) >= 4 {
        if codec != 1 || (option != 3 && option != 4) {
            return std::ptr::null_mut();
        }
        c.encode = cram_fn_ptr(cram_cram_codecs_c_547_cram_external_encode_char as usize);
    } else if option == 1 {
        c.encode = cram_fn_ptr(cram_cram_codecs_c_523_cram_external_encode_int as usize);
    } else if option == 2 {
        c.encode = cram_fn_ptr(cram_cram_codecs_c_535_cram_external_encode_long as usize);
    } else if option == 4 || option == 3 {
        c.encode = cram_fn_ptr(cram_cram_codecs_c_547_cram_external_encode_char as usize);
    } else {
        libc::abort();
    }
    c.store = cram_fn_ptr(cram_cram_codecs_c_562_cram_external_encode_store as usize);
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_644_cram_varint_decode_int(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get32.unwrap())(&mut cp, endp, &mut err) + (*c).varint.offset;
    *(out.cast::<i32>()) = val as i32;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;
    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_666_cram_varint_decode_sint(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get32s.unwrap())(&mut cp, endp, &mut err) + (*c).varint.offset;
    *(out.cast::<i32>()) = val as i32;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;
    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_688_cram_varint_decode_long(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get64.unwrap())(&mut cp, endp, &mut err) + (*c).varint.offset;
    *(out.cast::<i64>()) = val;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;
    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_710_cram_varint_decode_slong(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get64s.unwrap())(&mut cp, endp, &mut err) + (*c).varint.offset;
    *(out.cast::<i64>()) = val;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;
    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_732_cram_varint_decode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_varint_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_737_cram_varint_decode_size(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id);
    if b.is_null() {
        return -1;
    }
    (*(b.cast::<cram_block_layout>())).uncomp_size
}

pub unsafe fn cram_cram_codecs_c_748_cram_varint_get_block(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> *mut cram_block {
    let c = c.cast::<cram_codec_varint_layout>();
    cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id)
}

pub unsafe fn cram_cram_codecs_c_752_cram_varint_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let ks = &mut *ks;
    if kputsn(b"VARINT(id=", 10, ks) < 0
        || kputw((*c).varint.content_id, ks) < 0
        || kputsn(b",offset=", 8, ks) < 0
        || kputll((*c).varint.offset, ks) < 0
        || kputsn(b",type=", 6, ks) < 0
        || kputw((*c).varint.type_, ks) < 0
        || kputsn(b")", 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_760_cram_varint_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_varint_layout {
        codec,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: std::ptr::null_mut(),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        varint: cram_varint_decoder_layout {
            content_id: 0,
            offset: 0,
            type_: 0,
        },
    });
    c.decode = match codec {
        41 => {
            if option == 1 || option == 6 {
                cram_fn_ptr(cram_cram_codecs_c_644_cram_varint_decode_int as usize)
            } else if option == 2 || option == 7 {
                cram_fn_ptr(cram_cram_codecs_c_688_cram_varint_decode_long as usize)
            } else {
                return std::ptr::null_mut();
            }
        }
        42 => {
            if option == 1 || option == 6 {
                cram_fn_ptr(cram_cram_codecs_c_666_cram_varint_decode_sint as usize)
            } else if option == 2 || option == 7 {
                cram_fn_ptr(cram_cram_codecs_c_710_cram_varint_decode_slong as usize)
            } else {
                return std::ptr::null_mut();
            }
        }
        _ => {
            return std::ptr::null_mut();
        }
    };
    c.free = cram_fn_ptr(cram_cram_codecs_c_732_cram_varint_decode_free as usize);
    c.size = cram_fn_ptr(cram_cram_codecs_c_737_cram_varint_decode_size as usize);
    c.get_block = cram_fn_ptr(cram_cram_codecs_c_748_cram_varint_get_block as usize);
    c.describe = cram_fn_ptr(cram_cram_codecs_c_752_cram_varint_describe as usize);

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    c.varint.content_id =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    c.varint.offset =
        ((*vv).varint_get64s.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut());
    if cp.offset_from(data) != size as isize {
        return std::ptr::null_mut();
    }
    c.varint.type_ = option;
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_820_cram_varint_encode_int(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let val = *(in_.cast::<u32>()) as i64 - (*c).varint.offset;
    if ((*(*c).vv).varint_put32_blk.unwrap())((*c).out.cast(), val as i32) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_827_cram_varint_encode_sint(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let val = *(in_.cast::<i32>()) as i64 - (*c).varint.offset;
    if ((*(*c).vv).varint_put32s_blk.unwrap())((*c).out.cast(), val as i32) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_834_cram_varint_encode_long(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let val = *(in_.cast::<u64>()) as i64 - (*c).varint.offset;
    if ((*(*c).vv).varint_put64_blk.unwrap())((*c).out.cast(), val) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_841_cram_varint_encode_slong(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let val = *(in_.cast::<i64>()) - (*c).varint.offset;
    if ((*(*c).vv).varint_put64s_blk.unwrap())((*c).out.cast(), val) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_848_cram_varint_encode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_varint_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_854_cram_varint_encode_store(
    c: *mut c_void,
    b: *mut cram_block,
    prefix: *mut c_char,
    _version: c_int,
) -> c_int {
    let Some(c) = varint_codec_mut(c) else {
        return -1;
    };
    let Some(b) = raw_block_mut(b) else {
        return -1;
    };
    let mut tmp = [0u8; 99];
    let mut tp = tmp.as_mut_ptr().cast::<c_char>();
    let mut len = match append_optional_c_prefix(b, prefix) {
        Some(len) => len,
        None => return -1,
    };

    tp = tp.add(
        ((*(c.vv)).varint_put32.unwrap())(tp, std::ptr::null_mut(), c.varint.content_id) as usize,
    );
    tp = tp.add(
        ((*(c.vv)).varint_put64s.unwrap())(tp, std::ptr::null_mut(), c.varint.offset) as usize,
    );
    let b_raw = (b as *mut cram_block).cast();
    len += ((*(c.vv)).varint_put32_blk.unwrap())(b_raw, c.codec);
    len += ((*(c.vv)).varint_put32_blk.unwrap())(b_raw, tp.offset_from(tmp.as_ptr().cast()) as i32);
    let nbytes = tp.offset_from(tmp.as_ptr().cast()) as usize;
    if append_bytes_to_block(b, &tmp[..nbytes]) != 0 {
        return -1;
    }
    len + nbytes as c_int
}

pub unsafe fn cram_cram_codecs_c_878_cram_varint_encode_init(
    st: *mut c_void,
    mut codec: c_int,
    option: c_int,
    dat: *mut c_void,
    _version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_varint_layout {
        codec,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_848_cram_varint_encode_free as usize),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: cram_fn_ptr(cram_cram_codecs_c_854_cram_varint_encode_store as usize),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        varint: cram_varint_decoder_layout {
            content_id: 0,
            offset: 0,
            type_: option,
        },
    });
    if !st.is_null() {
        let st = st.cast::<cram_stats_layout>();
        if (*st).min_val < 0 && (*st).min_val >= -127 && (*st).max_val / -(*st).min_val > 100 {
            c.varint.offset = -(*st).min_val;
            codec = 41;
        } else if (*st).min_val > 0 {
            c.varint.offset = -(*st).min_val;
        }
    }

    c.codec = codec;
    c.encode = match codec {
        41 => {
            if option == 1 {
                cram_fn_ptr(cram_cram_codecs_c_820_cram_varint_encode_int as usize)
            } else {
                cram_fn_ptr(cram_cram_codecs_c_834_cram_varint_encode_long as usize)
            }
        }
        42 => {
            if option == 1 {
                cram_fn_ptr(cram_cram_codecs_c_827_cram_varint_encode_sint as usize)
            } else {
                cram_fn_ptr(cram_cram_codecs_c_841_cram_varint_encode_slong as usize)
            }
        }
        _ => {
            return std::ptr::null_mut();
        }
    };
    c.varint.content_id = dat as usize as i32;
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_932_cram_const_decode_byte(
    _slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    if out.is_null() {
        return 0;
    }
    let c = c.cast::<cram_codec_const_layout>();
    for i in 0..*out_size {
        *out.add(i as usize) = (*c).xconst.val as c_char;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_945_cram_const_decode_int(
    _slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_const_layout>();
    let out_i = out.cast::<i32>();
    for i in 0..*out_size {
        *out_i.add(i as usize) = (*c).xconst.val as i32;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_956_cram_const_decode_long(
    _slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_const_layout>();
    let out_i = out.cast::<i64>();
    for i in 0..*out_size {
        *out_i.add(i as usize) = (*c).xconst.val;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_967_cram_const_decode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_const_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_976_cram_const_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_const_layout>();
    let ks = &mut *ks;
    if kputsn(b"CONST(val=", 10, ks) < 0
        || kputll((*c).xconst.val, ks) < 0
        || kputsn(b")", 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_981_cram_const_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_const_layout {
        codec,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: std::ptr::null_mut(),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        xconst: cram_const_codec_layout { val: 0 },
    });
    c.decode = if codec == 43 && option == 3 {
        cram_fn_ptr(cram_cram_codecs_c_932_cram_const_decode_byte as usize)
    } else if codec == 44 && (option == 1 || option == 6) {
        cram_fn_ptr(cram_cram_codecs_c_945_cram_const_decode_int as usize)
    } else if codec == 44 && (option == 2 || option == 7) {
        cram_fn_ptr(cram_cram_codecs_c_956_cram_const_decode_long as usize)
    } else {
        return std::ptr::null_mut();
    };
    c.free = cram_fn_ptr(cram_cram_codecs_c_967_cram_const_decode_free as usize);
    c.size = cram_fn_ptr(cram_cram_codecs_c_972_cram_const_decode_size as usize);
    c.describe = cram_fn_ptr(cram_cram_codecs_c_976_cram_const_describe as usize);

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    c.xconst.val = ((*vv).varint_get64s.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut());
    if cp.offset_from(data) != size as isize {
        return std::ptr::null_mut();
    }
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_1025_cram_const_encode_store(
    c: *mut c_void,
    b: *mut cram_block,
    prefix: *mut c_char,
    _version: c_int,
) -> c_int {
    let Some(c) = const_codec_mut(c) else {
        return -1;
    };
    let Some(b) = raw_block_mut(b) else {
        return -1;
    };
    let mut tmp = [0u8; 99];
    let mut tp = tmp.as_mut_ptr().cast::<c_char>();
    let mut len = match append_optional_c_prefix(b, prefix) {
        Some(len) => len,
        None => return -1,
    };

    tp =
        tp.add(((*(c.vv)).varint_put64s.unwrap())(tp, std::ptr::null_mut(), c.xconst.val) as usize);
    let b_raw = (b as *mut cram_block).cast();
    len += ((*(c.vv)).varint_put32_blk.unwrap())(b_raw, c.codec);
    len += ((*(c.vv)).varint_put32_blk.unwrap())(b_raw, tp.offset_from(tmp.as_ptr().cast()) as i32);
    let nbytes = tp.offset_from(tmp.as_ptr().cast()) as usize;
    if append_bytes_to_block(b, &tmp[..nbytes]) != 0 {
        return -1;
    }
    len + nbytes as c_int
}

pub unsafe fn cram_cram_codecs_c_1048_cram_const_encode_init(
    st: *mut c_void,
    codec: c_int,
    _option: c_int,
    _dat: *mut c_void,
    _version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let c = Box::new(cram_codec_const_layout {
        codec,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_967_cram_const_decode_free as usize),
        decode: std::ptr::null_mut(),
        encode: cram_fn_ptr(cram_cram_codecs_c_1020_cram_const_encode as usize),
        store: cram_fn_ptr(cram_cram_codecs_c_1025_cram_const_encode_store as usize),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        xconst: cram_const_codec_layout {
            val: (*(st.cast::<cram_stats_layout>())).min_val,
        },
    });
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_1072_cram_beta_decode_long(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let out_i = out.cast::<i64>();
    let n = *out_size;
    if (*c).beta.nbits != 0 {
        if cram_cram_codecs_h_230_cram_not_enough_bits(in_, (*c).beta.nbits * n) != 0 {
            return -1;
        }
        for i in 0..n {
            *out_i.add(i as usize) =
                cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).beta.nbits) - (*c).beta.offset as i64;
        }
    } else {
        for i in 0..n {
            *out_i.add(i as usize) = -((*c).beta.offset as i64);
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1090_cram_beta_decode_int(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let out_i = out.cast::<i32>();
    let n = *out_size;
    if (*c).beta.nbits != 0 {
        if cram_cram_codecs_h_230_cram_not_enough_bits(in_, (*c).beta.nbits * n) != 0 {
            return -1;
        }
        for i in 0..n {
            *out_i.add(i as usize) =
                cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).beta.nbits) as i32 - (*c).beta.offset;
        }
    } else {
        for i in 0..n {
            *out_i.add(i as usize) = -(*c).beta.offset;
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1108_cram_beta_decode_char(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let n = *out_size;
    if (*c).beta.nbits != 0 {
        if cram_cram_codecs_h_230_cram_not_enough_bits(in_, (*c).beta.nbits * n) != 0 {
            return -1;
        }
        if !out.is_null() {
            for i in 0..n {
                *out.add(i as usize) = (cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).beta.nbits)
                    as i32
                    - (*c).beta.offset) as c_char;
            }
        } else {
            for _ in 0..n {
                cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).beta.nbits);
            }
        }
    } else if !out.is_null() {
        for i in 0..n {
            *out.add(i as usize) = (-(*c).beta.offset) as c_char;
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1131_cram_beta_decode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_beta_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_1136_cram_beta_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let ks = &mut *ks;
    if kputsn(b"BETA(offset=", 12, ks) < 0
        || kputw((*c).beta.offset, ks) < 0
        || kputsn(b", nbits=", 8, ks) < 0
        || kputw((*c).beta.nbits, ks) < 0
        || kputsn(b")", 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_1142_cram_beta_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_beta_layout {
        codec: 6,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: std::ptr::null_mut(),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        beta: cram_beta_decoder_layout {
            offset: 0,
            nbits: -1,
        },
    });
    c.decode = if option == 1 || option == 6 {
        cram_fn_ptr(cram_cram_codecs_c_1090_cram_beta_decode_int as usize)
    } else if option == 2 || option == 7 {
        cram_fn_ptr(cram_cram_codecs_c_1072_cram_beta_decode_long as usize)
    } else if option == 4 || option == 3 {
        cram_fn_ptr(cram_cram_codecs_c_1108_cram_beta_decode_char as usize)
    } else {
        return std::ptr::null_mut();
    };
    c.free = cram_fn_ptr(cram_cram_codecs_c_1131_cram_beta_decode_free as usize);
    c.describe = cram_fn_ptr(cram_cram_codecs_c_1136_cram_beta_describe as usize);

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    c.beta.offset =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    if cp < endp {
        c.beta.nbits =
            ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    }
    if cp.offset_from(data) != size as isize || c.beta.nbits < 0 || c.beta.nbits > 32 {
        return std::ptr::null_mut();
    }
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_1183_cram_beta_encode_store(
    c: *mut c_void,
    b: *mut cram_block,
    prefix: *mut c_char,
    _version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let Some(b_ref) = raw_block_mut(b) else {
        return -1;
    };
    let mut len = match append_optional_c_prefix(b_ref, prefix) {
        Some(len) => len,
        None => return -1,
    };
    let mut r = 0;

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;
    let body_len = ((*(*c).vv).varint_size.unwrap())((*c).beta.offset as i64)
        + ((*(*c).vv).varint_size.unwrap())((*c).beta.nbits as i64);
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, body_len);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).beta.offset);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).beta.nbits);
    len += n;
    r |= n;

    if r > 0 {
        len
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_1207_cram_beta_encode_long(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let syms = in_.cast::<i64>();
    let mut r = 0;
    for i in 0..in_size {
        r |= cram_cram_codecs_c_259_store_bits_MSB(
            (*c).out.cast(),
            (*syms.add(i as usize) + (*c).beta.offset as i64) as u64,
            (*c).beta.nbits,
        );
    }
    r
}

pub unsafe fn cram_cram_codecs_c_1219_cram_beta_encode_int(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let syms = in_.cast::<c_int>();
    let mut r = 0;
    for i in 0..in_size {
        r |= cram_cram_codecs_c_259_store_bits_MSB(
            (*c).out.cast(),
            (*syms.add(i as usize) + (*c).beta.offset) as u64,
            (*c).beta.nbits,
        );
    }
    r
}

pub unsafe fn cram_cram_codecs_c_1231_cram_beta_encode_char(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let syms = in_.cast::<u8>();
    let mut r = 0;
    for i in 0..in_size {
        r |= cram_cram_codecs_c_259_store_bits_MSB(
            (*c).out.cast(),
            (*syms.add(i as usize) as i32 + (*c).beta.offset) as u64,
            (*c).beta.nbits,
        );
    }
    r
}

pub unsafe fn cram_cram_codecs_c_1243_cram_beta_encode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_beta_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_1247_cram_beta_encode_init(
    st: *mut c_void,
    _codec: c_int,
    option: c_int,
    dat: *mut c_void,
    _version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_beta_layout {
        codec: 6,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_1243_cram_beta_encode_free as usize),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: cram_fn_ptr(cram_cram_codecs_c_1183_cram_beta_encode_store as usize),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        beta: cram_beta_decoder_layout {
            offset: 0,
            nbits: 0,
        },
    });
    c.encode = if option == 1 || option == 6 {
        cram_fn_ptr(cram_cram_codecs_c_1219_cram_beta_encode_int as usize)
    } else if option == 2 || option == 7 {
        cram_fn_ptr(cram_cram_codecs_c_1207_cram_beta_encode_long as usize)
    } else {
        cram_fn_ptr(cram_cram_codecs_c_1231_cram_beta_encode_char as usize)
    };

    let (min_val, max_val) = if !dat.is_null() {
        let dat = dat.cast::<i64>();
        (*dat, *dat.add(1))
    } else {
        let st = st.cast::<cram_stats_layout>();
        let mut min_val = i32::MAX as i64;
        let mut max_val = i32::MIN as i64;
        for i in 0..1024usize {
            if (*st).freqs[i] == 0 {
                continue;
            }
            if min_val > i as i64 {
                min_val = i as i64;
            }
            max_val = i as i64;
        }
        if let Some(h) = (*st).h.as_deref() {
            for k in 0..h.n_buckets {
                let flag = h.flags[(k >> 4) as usize];
                if ((flag >> ((k & 0xf) << 1)) & 3) != 0 {
                    continue;
                }
                let i = h.keys[k as usize];
                if min_val > i {
                    min_val = i;
                }
                if max_val < i {
                    max_val = i;
                }
            }
        }
        (min_val, max_val)
    };

    if max_val < min_val {
        return std::ptr::null_mut();
    }

    let mut range = max_val - min_val;
    match option {
        6 => {
            if min_val < i32::MIN as i64 || range > i32::MAX as i64 {
                return std::ptr::null_mut();
            }
        }
        1 => {
            if max_val > u32::MAX as i64 || range > u32::MAX as i64 {
                return std::ptr::null_mut();
            }
        }
        _ => {}
    }

    c.beta.offset = (-min_val) as i32;
    let mut len = 0;
    while range != 0 {
        len += 1;
        range >>= 1;
    }
    c.beta.nbits = len;

    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_1344_cram_xpack_decode_long(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    let out_i = out.cast::<i64>();
    let n = *out_size;
    if (*c).xpack.nbits != 0 {
        for i in 0..n {
            let idx = cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).xpack.nbits) as usize;
            *out_i.add(i as usize) = (*c).xpack.rmap[idx] as i64;
        }
    } else {
        for i in 0..n {
            *out_i.add(i as usize) = (*c).xpack.rmap[0] as i64;
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1359_cram_xpack_decode_int(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    let out_i = out.cast::<i32>();
    let n = *out_size;
    if (*c).xpack.nbits != 0 {
        if cram_cram_codecs_h_230_cram_not_enough_bits(in_, (*c).xpack.nbits * n) != 0 {
            return -1;
        }
        for i in 0..n {
            let idx = cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).xpack.nbits) as usize;
            *out_i.add(i as usize) = (*c).xpack.rmap[idx] as i32;
        }
    } else {
        for i in 0..n {
            *out_i.add(i as usize) = (*c).xpack.rmap[0] as i32;
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1377_cram_xpack_decode_expand_char(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> c_int {
    let slice_layout = slice.cast::<cram_slice_layout>();
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    let slot = (512 + (*c_xpack).codec_id) as usize;
    let cached = *(*slice_layout).block_by_id.add(slot);
    if !cached.is_null() {
        return 0;
    }

    let sub_codec = (*c_xpack).xpack.sub_codec;
    let sub_layout = sub_codec.cast::<cram_codec_xpack_layout>();
    let get_block: CramCodecGetBlockFn = cram_fn((*sub_layout).get_block);
    let sub_b = get_block(slice, sub_codec);
    if sub_b.is_null() || (*c_xpack).xpack.nbits == 0 {
        return -1;
    }

    let b = cram_cram_io_c_1388_cram_new_block(
        crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER,
        0,
    );
    if b.is_null() {
        return -1;
    }
    *(*slice_layout).block_by_id.add(slot) = b.cast();
    let sub = sub_b.cast::<cram_block_layout>();
    let out_n = (*sub).uncomp_size * 8 / (*c_xpack).xpack.nbits;
    if cram_cram_io_h_243_block_grow(b, out_n as usize) < 0 {
        return -1;
    }
    let out = b.cast::<cram_block_layout>();
    (*out).uncomp_size = out_n;

    let nsym = 8 / (*c_xpack).xpack.nbits;
    let mut i = 0usize;
    let mut j = 0usize;
    while i < out_n as usize {
        let mut byte = *(*sub).data.add(j);
        j += 1;
        match nsym {
            8 => {
                for _ in 0..8 {
                    if i >= out_n as usize {
                        break;
                    }
                    *(*out).data.add(i) = (*c_xpack).xpack.rmap[(byte & 1) as usize] as u8;
                    byte >>= 1;
                    i += 1;
                }
            }
            4 => {
                for _ in 0..4 {
                    if i >= out_n as usize {
                        break;
                    }
                    *(*out).data.add(i) = (*c_xpack).xpack.rmap[(byte & 3) as usize] as u8;
                    byte >>= 2;
                    i += 1;
                }
            }
            2 => {
                for _ in 0..2 {
                    if i >= out_n as usize {
                        break;
                    }
                    *(*out).data.add(i) = (*c_xpack).xpack.rmap[(byte & 15) as usize] as u8;
                    byte >>= 4;
                    i += 1;
                }
            }
            1 => {
                *(*out).data.add(i) = byte;
                i += 1;
            }
            _ => return -1,
        }
    }

    0
}

pub unsafe fn cram_cram_codecs_c_1408_cram_xpack_decode_char(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    if (*c_xpack).xpack.nval > 1 {
        cram_cram_codecs_c_1377_cram_xpack_decode_expand_char(slice, c);
        let slice_layout = slice.cast::<cram_slice_layout>();
        let b = *(*slice_layout)
            .block_by_id
            .add((512 + (*c_xpack).codec_id) as usize);
        if b.is_null() {
            return -1;
        }
        let block = b.cast::<cram_block_layout>();
        if !out.is_null() {
            memcpy(
                out.cast(),
                (*block).data.add((*block).byte).cast(),
                *out_size as u64,
            );
        }
        (*block).byte += *out_size as usize;
    } else if !out.is_null() {
        std::ptr::write_bytes(out, (*c_xpack).xpack.rmap[0] as u8, *out_size as usize);
    }

    0
}

pub unsafe fn cram_cram_codecs_c_1431_cram_xpack_decode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    if !(*c_xpack).xpack.sub_codec.is_null() {
        let sub = (*c_xpack).xpack.sub_codec.cast::<cram_codec_xpack_layout>();
        if !(*sub).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*sub).free);
            free_fn((*c_xpack).xpack.sub_codec);
        }
    }
    drop(Box::from_raw(c_xpack));
}

pub unsafe fn cram_cram_codecs_c_1443_cram_xpack_decode_size(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> c_int {
    cram_cram_codecs_c_1377_cram_xpack_decode_expand_char(slice, c);
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    let slice_layout = slice.cast::<cram_slice_layout>();
    let b = *(*slice_layout)
        .block_by_id
        .add((512 + (*c_xpack).codec_id) as usize);
    (*(b.cast::<cram_block_layout>())).uncomp_size
}

pub unsafe fn cram_cram_codecs_c_1448_cram_xpack_get_block(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> *mut cram_block {
    cram_cram_codecs_c_1377_cram_xpack_decode_expand_char(slice, c);
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    let slice_layout = slice.cast::<cram_slice_layout>();
    (*(*slice_layout)
        .block_by_id
        .add((512 + (*c_xpack).codec_id) as usize))
    .cast()
}

pub unsafe fn cram_cram_codecs_c_1453_cram_xpack_decode_init(
    hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_xpack_layout {
        codec: 51,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_1431_cram_xpack_decode_free as usize),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: cram_fn_ptr(cram_cram_codecs_c_1443_cram_xpack_decode_size as usize),
        flush: std::ptr::null_mut(),
        get_block: cram_fn_ptr(cram_cram_codecs_c_1448_cram_xpack_get_block as usize),
        describe: std::ptr::null_mut(),
        xpack: cram_xpack_decoder_layout {
            nbits: 0,
            sub_encoding: 0,
            sub_codec_dat: std::ptr::null_mut(),
            sub_codec: std::ptr::null_mut(),
            nval: 0,
            rmap: [0; 256],
            map: [0; 256],
        },
    });

    c.decode = if option == 2 {
        cram_fn_ptr(cram_cram_codecs_c_1344_cram_xpack_decode_long as usize)
    } else if option == 1 {
        cram_fn_ptr(cram_cram_codecs_c_1359_cram_xpack_decode_int as usize)
    } else if option == 4 || option == 3 {
        cram_fn_ptr(cram_cram_codecs_c_1408_cram_xpack_decode_char as usize)
    } else {
        return std::ptr::null_mut();
    };

    let vv_layout = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    c.xpack.nbits =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as i32;
    c.xpack.nval =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as i32;
    if c.xpack.nbits >= 8 || c.xpack.nbits < 0 || c.xpack.nval > 256 || c.xpack.nval < 0 {
        return std::ptr::null_mut();
    }
    for i in 0..c.xpack.nval {
        let v =
            ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
                as u32;
        if v >= 256 {
            return std::ptr::null_mut();
        }
        c.xpack.rmap[i as usize] = v;
    }

    let encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        return std::ptr::null_mut();
    }
    c.xpack.sub_codec =
        cram_cram_codecs_c_3872_cram_decoder_init(hdr, encoding, cp, sub_size, option, version, vv);
    if c.xpack.sub_codec.is_null() {
        return std::ptr::null_mut();
    }
    cp = cp.add(sub_size as usize);

    if cp.offset_from(data) != size as isize
        || c.xpack.nbits < 0
        || c.xpack.nbits > (8 * std::mem::size_of::<i64>()) as i32
    {
        let sub = c.xpack.sub_codec.cast::<cram_codec_xpack_layout>();
        if !(*sub).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*sub).free);
            free_fn(c.xpack.sub_codec);
        }
        c.xpack.sub_codec = std::ptr::null_mut();
        return std::ptr::null_mut();
    }

    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_1581_cram_xpack_encode_long(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    let syms = in_.cast::<i64>();
    let mut r = 0;
    for i in 0..in_size {
        r |= cram_cram_codecs_c_259_store_bits_MSB(
            (*c).out.cast(),
            (*c).xpack.map[*syms.add(i as usize) as usize] as u64,
            (*c).xpack.nbits,
        );
    }
    r
}

pub unsafe fn cram_cram_codecs_c_1592_cram_xpack_encode_int(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    let syms = in_.cast::<c_int>();
    let mut r = 0;
    for i in 0..in_size {
        r |= cram_cram_codecs_c_259_store_bits_MSB(
            (*c).out.cast(),
            (*c).xpack.map[*syms.add(i as usize) as usize] as u64,
            (*c).xpack.nbits,
        );
    }
    r
}

pub unsafe fn cram_cram_codecs_c_1603_cram_xpack_encode_char(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    if cram_cram_io_h_248_block_append((*c).out.cast(), in_.cast(), in_size as usize) == 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_1612_cram_xpack_encode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    if !(*c_xpack).xpack.sub_codec.is_null() {
        let sub = (*c_xpack).xpack.sub_codec.cast::<cram_codec_xpack_layout>();
        if !(*sub).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*sub).free);
            free_fn((*c_xpack).xpack.sub_codec);
        }
    }
    cram_cram_io_c_1565_cram_free_block((*c_xpack).out.cast());
    drop(Box::from_raw(c_xpack));
}

pub unsafe extern "C" fn cram_cram_codecs_c_1515_cram_xpack_encode_flush(c: *mut c_void) -> c_int {
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    let out_block = (*c_xpack).out.cast::<cram_block_layout>();
    let mut meta_len = 0;
    let mut out_len = 0u64;
    let mut out_meta = [0u8; 1024];
    let Some(mut out) = crate::htslib_rs::htscodecs::pack::hts_pack(
        std::slice::from_raw_parts((*out_block).data, (*out_block).byte),
        (*out_block).byte as i64,
        &mut out_meta,
        &mut meta_len,
        &mut out_len,
    ) else {
        return -1;
    };
    let sub_codec = (*c_xpack).xpack.sub_codec;
    let sub_layout = sub_codec.cast::<cram_codec_xpack_layout>();
    let encode: CramCodecEncodeFn = cram_fn((*sub_layout).encode);
    if encode(
        std::ptr::null_mut(),
        sub_codec,
        out.as_mut_ptr().cast(),
        out_len as c_int,
    ) != 0
    {
        return -1;
    }

    let mut r = 0;
    if !(*sub_layout).flush.is_null() {
        let flush: CramCodecFlushFn = cram_fn((*sub_layout).flush);
        r = flush(sub_codec);
    }

    r
}

pub unsafe fn cram_cram_codecs_c_1537_cram_xpack_encode_store(
    c: *mut c_void,
    b: *mut cram_block,
    prefix: *mut c_char,
    version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    let Some(b_ref) = raw_block_mut(b) else {
        return -1;
    };
    let mut len = match append_optional_c_prefix(b_ref, prefix) {
        Some(len) => len,
        None => return -1,
    };
    let mut r = 0;

    let tc = (*c).xpack.sub_codec;
    let tb = cram_cram_io_c_1388_cram_new_block(
        crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER,
        0,
    );
    if tb.is_null() {
        return -1;
    }
    let store: CramCodecStoreFn = cram_fn((*(tc.cast::<cram_codec_xpack_layout>())).store);
    let len2 = store(tc, tb, std::ptr::null_mut(), version);

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;

    let mut len1 = 0;
    for i in 0..(*c).xpack.nval {
        let n = ((*(*c).vv).varint_size.unwrap())((*c).xpack.rmap[i as usize] as i64);
        len1 += n;
        r |= n;
    }
    let body_len = ((*(*c).vv).varint_size.unwrap())((*c).xpack.nbits as i64)
        + ((*(*c).vv).varint_size.unwrap())((*c).xpack.nval as i64)
        + len1
        + len2;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, body_len);
    len += n;
    r |= n;

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).xpack.nbits);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).xpack.nval);
    len += n;
    r |= n;
    for i in 0..(*c).xpack.nval {
        let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).xpack.rmap[i as usize] as i32);
        len += n;
        r |= n;
    }

    if cram_cram_io_h_248_block_append(
        b,
        (*(tb.cast::<cram_block_layout>())).data.cast(),
        (*(tb.cast::<cram_block_layout>())).byte,
    ) != 0
    {
        cram_cram_io_c_1565_cram_free_block(tb);
        return -1;
    }
    cram_cram_io_c_1565_cram_free_block(tb);

    if r > 0 {
        len + len2
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_1623_cram_xpack_encode_init(
    _st: *mut c_void,
    _codec: c_int,
    option: c_int,
    dat: *mut c_void,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let e = dat.cast::<cram_xpack_decoder_layout>();
    let mut c = Box::new(cram_codec_xpack_layout {
        codec: 51,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_1612_cram_xpack_encode_free as usize),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: cram_fn_ptr(cram_cram_codecs_c_1537_cram_xpack_encode_store as usize),
        size: std::ptr::null_mut(),
        flush: cram_fn_ptr(cram_cram_codecs_c_1515_cram_xpack_encode_flush as usize),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        xpack: cram_xpack_decoder_layout {
            nbits: (*e).nbits,
            sub_encoding: (*e).sub_encoding,
            sub_codec_dat: (*e).sub_codec_dat,
            sub_codec: std::ptr::null_mut(),
            nval: (*e).nval,
            rmap: [0; 256],
            map: (*e).map,
        },
    });
    c.encode = if option == 2 {
        cram_fn_ptr(cram_cram_codecs_c_1581_cram_xpack_encode_long as usize)
    } else if option == 1 {
        cram_fn_ptr(cram_cram_codecs_c_1592_cram_xpack_encode_int as usize)
    } else {
        cram_fn_ptr(cram_cram_codecs_c_1603_cram_xpack_encode_char as usize)
    };

    c.xpack.sub_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).sub_encoding,
        None,
        4,
        (*e).sub_codec_dat,
        version,
        vv,
    );
    let mut n = 0;
    for i in 0..256usize {
        if (*e).map[i] != -1 {
            c.xpack.rmap[n as usize] = i as u32;
            n += 1;
        }
    }
    if n != (*e).nval {
        if !c.xpack.sub_codec.is_null() {
            let sub = c.xpack.sub_codec.cast::<cram_codec_xpack_layout>();
            if !(*sub).free.is_null() {
                let free_fn: unsafe fn(*mut c_void) = cram_fn((*sub).free);
                free_fn(c.xpack.sub_codec);
            }
            c.xpack.sub_codec = std::ptr::null_mut();
        }
        return std::ptr::null_mut();
    }

    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_1688_cram_xdelta_decode_int(
    slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xdelta_layout>();
    let out32 = out.cast::<u32>();
    for i in 0..*out_size {
        let mut v = 0u32;
        let mut one = 1;
        let sub = (*c).xdelta.sub_codec;
        let sub_codec = sub.cast::<cram_codec_xdelta_layout>();
        let decode_fn: CramCodecDecodeFn = cram_fn((*sub_codec).decode);
        if decode_fn(slice, sub, in_, (&mut v as *mut u32).cast(), &mut one) < 0 {
            return -1;
        }
        let d = cram_cram_codecs_c_1682_unzigzag32(v) as u32;
        (*c).xdelta.last = d.wrapping_add((*c).xdelta.last as u32) as i64;
        *out32.add(i as usize) = (*c).xdelta.last as u32;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1719_cram_xdelta_decode_block(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out_: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xdelta_layout>();
    let out = out_.cast::<cram_block>();
    let sub = (*c).xdelta.sub_codec;
    let sub_codec = sub.cast::<cram_codec_xdelta_layout>();
    let get_block_fn: CramCodecGetBlockFn = cram_fn((*sub_codec).get_block);
    let b = get_block_fn(slice, sub);
    let w = (*c).xdelta.word_size as c_int;
    let mut npad = (w - *out_size % w) % w;
    let out_sz = *out_size + npad;
    (*c).xdelta.last = 0;

    let mut i = 0;
    while i < out_sz {
        let block = b.cast::<cram_block_layout>();
        let mut cp = (*block).data.add((*block).byte).cast::<c_char>();
        let cp_end = (*block)
            .data
            .add((*block).uncomp_size as usize)
            .cast::<c_char>();
        let mut err = 0;
        let v = ((*(*c).vv).varint_get32.unwrap())(&mut cp, cp_end, &mut err) as u16;
        if err != 0 {
            return -1;
        }
        (*block).byte = cp.offset_from((*block).data.cast::<c_char>()) as usize;

        match w {
            2 => {
                let d = cram_cram_codecs_c_1681_unzigzag16(v) as i64;
                (*c).xdelta.last += d;
                let z = cram_cram_codecs_c_1713_le_int2((*c).xdelta.last as i16);
                if cram_cram_io_h_248_block_append(
                    out,
                    (&z as *const i16).cast(),
                    (2 - npad) as usize,
                ) != 0
                {
                    return -1;
                }
                npad = 0;
            }
            _ => return -1,
        }
        i += w;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1762_cram_xdelta_decode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xdelta = c.cast::<cram_codec_xdelta_layout>();
    if !(*c_xdelta).xdelta.sub_codec.is_null() {
        let sub = (*c_xdelta)
            .xdelta
            .sub_codec
            .cast::<cram_codec_xdelta_layout>();
        if !(*sub).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*sub).free);
            free_fn((*c_xdelta).xdelta.sub_codec);
        }
    }
    drop(Box::from_raw(c_xdelta));
}

pub unsafe fn cram_cram_codecs_c_1771_cram_xdelta_decode_size(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> c_int {
    cram_cram_codecs_c_1705_cram_xdelta_decode_expand_char(slice, c);
    let slice = slice.cast::<cram_slice_layout>();
    let c = c.cast::<cram_codec_xdelta_layout>();
    let b = *(*slice).block_by_id.add((512 + (*c).codec_id) as usize);
    (*(b.cast::<cram_block_layout>())).uncomp_size
}

pub unsafe fn cram_cram_codecs_c_1776_cram_xdelta_get_block(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> *mut cram_block {
    cram_cram_codecs_c_1705_cram_xdelta_decode_expand_char(slice, c);
    let slice = slice.cast::<cram_slice_layout>();
    let c = c.cast::<cram_codec_xdelta_layout>();
    (*(*slice).block_by_id.add((512 + (*c).codec_id) as usize)).cast()
}

pub unsafe fn cram_cram_codecs_c_1781_cram_xdelta_decode_init(
    hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    mut option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_xdelta_layout {
        codec: 53,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: std::ptr::null_mut(),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        xdelta: cram_xdelta_decoder_layout {
            last: 0,
            word_size: 0,
            sub_encoding: 0,
            sub_codec_dat: std::ptr::null_mut(),
            sub_codec: std::ptr::null_mut(),
        },
    });
    c.decode = if option == 2 {
        cram_fn_ptr(cram_cram_codecs_c_1684_cram_xdelta_decode_long as usize)
    } else if option == 1 {
        cram_fn_ptr(cram_cram_codecs_c_1688_cram_xdelta_decode_int as usize)
    } else if option == 4 || option == 3 {
        cram_fn_ptr(cram_cram_codecs_c_1709_cram_xdelta_decode_char as usize)
    } else if option == 5 {
        option = 4;
        cram_fn_ptr(cram_cram_codecs_c_1719_cram_xdelta_decode_block as usize)
    } else {
        return std::ptr::null_mut();
    };
    c.free = cram_fn_ptr(cram_cram_codecs_c_1762_cram_xdelta_decode_free as usize);
    c.size = cram_fn_ptr(cram_cram_codecs_c_1771_cram_xdelta_decode_size as usize);
    c.get_block = cram_fn_ptr(cram_cram_codecs_c_1776_cram_xdelta_get_block as usize);

    let vv_layout = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    c.xdelta.word_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as u8;

    let encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        return std::ptr::null_mut();
    }
    c.xdelta.sub_codec =
        cram_cram_codecs_c_3872_cram_decoder_init(hdr, encoding, cp, sub_size, option, version, vv);
    if c.xdelta.sub_codec.is_null() {
        return std::ptr::null_mut();
    }
    cp = cp.add(sub_size as usize);

    if cp.offset_from(data) != size as isize {
        let sub = c.xdelta.sub_codec.cast::<cram_codec_xdelta_layout>();
        if !(*sub).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*sub).free);
            free_fn(c.xdelta.sub_codec);
        }
        c.xdelta.sub_codec = std::ptr::null_mut();
        return std::ptr::null_mut();
    }

    Box::into_raw(c).cast()
}

pub unsafe extern "C" fn cram_cram_codecs_c_1835_cram_xdelta_encode_flush(c: *mut c_void) -> c_int {
    let c = c.cast::<cram_codec_xdelta_layout>();
    let b = cram_cram_io_c_1388_cram_new_block(
        crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER,
        0,
    );
    if b.is_null() {
        return -1;
    }
    let out = (*c).out.cast::<cram_block_layout>();
    let mut r = -1;

    match (*c).xdelta.word_size {
        2 => {
            let n = (*out).byte / 2;
            let mut dat = (*out).data.cast::<u8>();
            let mut last = 0u16;
            if n * 2 < (*out).byte {
                last = *dat as u16;
                ((*(*c).vv).varint_put32_blk.unwrap())(
                    b,
                    cram_cram_codecs_c_1677_zigzag16(last as i16) as i32,
                );
                dat = dat.add(1);
            }
            let dat16 = dat.cast::<u16>();
            for i in 0..n {
                let v = std::ptr::read_unaligned(dat16.add(i));
                let d = v.wrapping_sub(last);
                last = v;
                ((*(*c).vv).varint_put32_blk.unwrap())(
                    b,
                    cram_cram_codecs_c_1677_zigzag16(d as i16) as i32,
                );
            }
        }
        4 => {
            let n = (*out).byte / 4;
            let dat = (*out).data.cast::<u32>();
            let mut last = 0u32;
            for i in 0..n {
                let v = std::ptr::read_unaligned(dat.add(i));
                let d = v.wrapping_sub(last);
                last = v;
                ((*(*c).vv).varint_put32_blk.unwrap())(
                    b,
                    cram_cram_codecs_c_1678_zigzag32(d as i32) as i32,
                );
            }
        }
        1 => {
            let n = (*out).byte;
            let dat = (*out).data;
            let mut last = 0u8;
            for i in 0..n {
                let v = *dat.add(i);
                let d = v.wrapping_sub(last);
                last = v;
                ((*(*c).vv).varint_put32_blk.unwrap())(
                    b,
                    cram_cram_codecs_c_1676_zigzag8(d as i8) as i32,
                );
            }
        }
        _ => {
            cram_cram_io_c_1565_cram_free_block(b);
            return -1;
        }
    }

    let sub_codec = (*c).xdelta.sub_codec;
    let sub_layout = sub_codec.cast::<cram_codec_xdelta_layout>();
    let encode: CramCodecEncodeFn = cram_fn((*sub_layout).encode);
    let b_layout = b.cast::<cram_block_layout>();
    if encode(
        std::ptr::null_mut(),
        sub_codec,
        (*b_layout).data.cast(),
        (*b_layout).byte as c_int,
    ) == 0
    {
        r = 0;
    }

    cram_cram_io_c_1565_cram_free_block(b);
    r
}

pub unsafe fn cram_cram_codecs_c_1930_cram_xdelta_encode_store(
    c: *mut c_void,
    b: *mut cram_block,
    prefix: *mut c_char,
    version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xdelta_layout>();
    let Some(b_ref) = raw_block_mut(b) else {
        return -1;
    };
    let mut len = match append_optional_c_prefix(b_ref, prefix) {
        Some(len) => len,
        None => return -1,
    };
    let mut r = 0;

    let tc = (*c).xdelta.sub_codec;
    let tb = cram_cram_io_c_1388_cram_new_block(
        crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER,
        0,
    );
    if tb.is_null() {
        return -1;
    }
    let store: CramCodecStoreFn = cram_fn((*(tc.cast::<cram_codec_xdelta_layout>())).store);
    let len2 = store(tc, tb, std::ptr::null_mut(), version);

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(
        b,
        ((*(*c).vv).varint_size.unwrap())((*c).xdelta.word_size as i64) + len2,
    );
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).xdelta.word_size as i32);
    len += n;
    r |= n;

    let tb_layout = tb.cast::<cram_block_layout>();
    if cram_cram_io_h_248_block_append(b, (*tb_layout).data.cast(), (*tb_layout).byte) != 0 {
        cram_cram_io_c_1565_cram_free_block(tb);
        return -1;
    }
    cram_cram_io_c_1565_cram_free_block(tb);

    if r > 0 {
        len + len2
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_1976_cram_xdelta_encode_char(
    slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xdelta_layout>();
    if in_size < 0 {
        return -1;
    }
    let Some(dat_len) = (in_size as usize).checked_mul(5) else {
        return -1;
    };
    let mut dat = Vec::<c_char>::new();
    if dat.try_reserve_exact(dat_len).is_err() {
        return -1;
    }
    dat.resize(dat_len, 0);
    let dat_ptr = dat.as_mut_ptr();
    let mut cp = dat_ptr;
    let cp_end = dat_ptr.add(dat_len);
    (*c).xdelta.last = 0;

    if (*c).xdelta.word_size == 2 {
        let part = in_size % 2;
        if part != 0 {
            let z = *in_ as i16;
            (*c).xdelta.last = cram_cram_codecs_c_1713_le_int2(z) as i64;
            cp = cp.add(((*(*c).vv).varint_put32.unwrap())(
                cp,
                cp_end,
                cram_cram_codecs_c_1677_zigzag16((*c).xdelta.last as i16) as i32,
            ) as usize);
        }
        let in16 = in_.add(part as usize).cast::<i16>();
        for i in 0..(in_size / 2) {
            let v = cram_cram_codecs_c_1713_le_int2(std::ptr::read_unaligned(in16.add(i as usize)));
            let d = (v as i64 - (*c).xdelta.last) as i16;
            (*c).xdelta.last = v as i64;
            cp = cp.add(((*(*c).vv).varint_put32.unwrap())(
                cp,
                cp_end,
                cram_cram_codecs_c_1677_zigzag16(d) as i32,
            ) as usize);
        }
    }

    let sub_codec = (*c).xdelta.sub_codec;
    let sub_layout = sub_codec.cast::<cram_codec_xdelta_layout>();
    let encode: CramCodecEncodeFn = cram_fn((*sub_layout).encode);
    if encode(slice, sub_codec, dat_ptr, cp.offset_from(dat_ptr) as c_int) != 0 {
        return -1;
    }

    0
}

pub unsafe fn cram_cram_codecs_c_2011_cram_xdelta_encode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xdelta = c.cast::<cram_codec_xdelta_layout>();
    if !(*c_xdelta).xdelta.sub_codec.is_null() {
        let sub = (*c_xdelta)
            .xdelta
            .sub_codec
            .cast::<cram_codec_xdelta_layout>();
        if !(*sub).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*sub).free);
            free_fn((*c_xdelta).xdelta.sub_codec);
        }
    }
    cram_cram_io_c_1565_cram_free_block((*c_xdelta).out.cast());
    drop(Box::from_raw(c_xdelta));
}

pub unsafe fn cram_cram_codecs_c_2022_cram_xdelta_encode_init(
    _st: *mut c_void,
    _codec: c_int,
    option: c_int,
    dat: *mut c_void,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_xdelta_layout {
        codec: 53,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_2011_cram_xdelta_encode_free as usize),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: cram_fn_ptr(cram_cram_codecs_c_1930_cram_xdelta_encode_store as usize),
        size: std::ptr::null_mut(),
        flush: cram_fn_ptr(cram_cram_codecs_c_1835_cram_xdelta_encode_flush as usize),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        xdelta: cram_xdelta_decoder_layout {
            last: 0,
            word_size: 0,
            sub_encoding: 0,
            sub_codec_dat: std::ptr::null_mut(),
            sub_codec: std::ptr::null_mut(),
        },
    });
    c.encode = if option == 2 {
        cram_fn_ptr(cram_cram_codecs_c_1966_cram_xdelta_encode_long as usize)
    } else if option == 1 {
        cram_fn_ptr(cram_cram_codecs_c_1971_cram_xdelta_encode_int as usize)
    } else {
        cram_fn_ptr(cram_cram_codecs_c_1976_cram_xdelta_encode_char as usize)
    };

    let e = dat.cast::<cram_xdelta_decoder_layout>();
    c.xdelta.word_size = (*e).word_size;
    c.xdelta.sub_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).sub_encoding,
        None,
        4,
        (*e).sub_codec_dat,
        version,
        vv,
    );
    if c.xdelta.sub_codec.is_null() {
        return std::ptr::null_mut();
    }

    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_2074_cram_xrle_decode_expand_char(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> c_int {
    let slice_layout = slice.cast::<cram_slice_layout>();
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    let cache_index = (512 + (*c_xrle).codec_id) as usize;
    let slot = (*slice_layout).block_by_id.add(cache_index);
    if !(*slot).is_null() {
        return 0;
    }

    let b = cram_cram_io_c_1388_cram_new_block(0, 0);
    *slot = b.cast();
    if b.is_null() {
        return -1;
    }

    let lit_codec = (*c_xrle).xrle.lit_codec;
    let lit_get_block: CramCodecGetBlockFn =
        cram_fn((*(lit_codec.cast::<cram_codec_xrle_layout>())).get_block);
    let lit_b = lit_get_block(slice, lit_codec);
    if lit_b.is_null() {
        return -1;
    }
    let lit_layout = lit_b.cast::<cram_block_layout>();
    let lit_dat = (*lit_layout).data;
    let lit_sz = (*lit_layout).uncomp_size as u64;

    let len_codec = (*c_xrle).xrle.len_codec;
    let len_size_fn: CramCodecSizeFn =
        cram_fn((*(len_codec.cast::<cram_codec_xrle_layout>())).size);
    let len_sz = len_size_fn(slice, len_codec) as usize;
    let len_get_block: CramCodecGetBlockFn =
        cram_fn((*(len_codec.cast::<cram_codec_xrle_layout>())).get_block);
    let len_b = len_get_block(slice, len_codec);
    if len_b.is_null() {
        return -1;
    }
    let len_layout = len_b.cast::<cram_block_layout>();
    let len_dat = (*len_layout).data;

    let mut rle_syms = [0u8; 256];
    let mut rle_nsyms = 0;
    for i in 0..256usize {
        if (*c_xrle).xrle.rep_score[i] > 0 {
            rle_syms[rle_nsyms] = i as u8;
            rle_nsyms += 1;
        }
    }

    let mut cp = len_dat;
    let endp = len_dat.add(len_sz);
    let mut out_sz = 0u64;
    let mut shift = 0u32;
    if cp >= endp {
        out_sz = 0;
    } else {
        loop {
            let ch = *cp;
            cp = cp.add(1);
            out_sz |= ((ch & 0x7f) as u64) << shift;
            shift += 7;
            if (ch & 0x80) == 0 || cp >= endp {
                break;
            }
        }
    }
    let nb = cp.offset_from(len_dat) as usize;

    let b_layout = b.cast::<cram_block_layout>();
    (*b_layout).data = malloc(out_sz).cast();
    if (*b_layout).data.is_null() {
        return -1;
    }
    crate::htslib_rs::htscodecs::rle::hts_rle_decode_raw_slices(
        std::slice::from_raw_parts(lit_dat, lit_sz as usize),
        std::slice::from_raw_parts(len_dat.add(nb), len_sz - nb),
        &rle_syms[..rle_nsyms],
        rle_nsyms as c_int,
        std::slice::from_raw_parts_mut((*b_layout).data, out_sz as usize),
        &mut out_sz,
    );
    (*b_layout).uncomp_size = out_sz as i32;
    0
}

pub unsafe extern "C" fn cram_cram_codecs_c_2115_cram_xrle_decode_size(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> c_int {
    cram_cram_codecs_c_2074_cram_xrle_decode_expand_char(slice, c);
    let slice_layout = slice.cast::<cram_slice_layout>();
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    let b = *(*slice_layout)
        .block_by_id
        .add((512 + (*c_xrle).codec_id) as usize);
    (*(b.cast::<cram_block_layout>())).uncomp_size
}

pub unsafe extern "C" fn cram_cram_codecs_c_2120_cram_xrle_get_block(
    slice: *mut cram_slice,
    c: *mut c_void,
) -> *mut cram_block {
    cram_cram_codecs_c_2074_cram_xrle_decode_expand_char(slice, c);
    let slice_layout = slice.cast::<cram_slice_layout>();
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    (*(*slice_layout)
        .block_by_id
        .add((512 + (*c_xrle).codec_id) as usize))
    .cast()
}

pub unsafe extern "C" fn cram_cram_codecs_c_2125_cram_xrle_decode_char(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let n = *out_size;
    cram_cram_codecs_c_2074_cram_xrle_decode_expand_char(slice, c);
    let slice_layout = slice.cast::<cram_slice_layout>();
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    let b = *(*slice_layout)
        .block_by_id
        .add((512 + (*c_xrle).codec_id) as usize);
    let b_layout = b.cast::<cram_block_layout>();
    if !out.is_null() {
        memcpy(
            out.cast(),
            (*b_layout).data.add((*b_layout).idx as usize).cast(),
            n as u64,
        );
    }
    (*b_layout).idx += n;
    0
}

pub unsafe fn cram_cram_codecs_c_2172_cram_xrle_decode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    if !(*c_xrle).xrle.len_codec.is_null() {
        let len = (*c_xrle).xrle.len_codec.cast::<cram_codec_xrle_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn((*c_xrle).xrle.len_codec);
        }
    }
    if !(*c_xrle).xrle.lit_codec.is_null() {
        let lit = (*c_xrle).xrle.lit_codec.cast::<cram_codec_xrle_layout>();
        if !(*lit).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*lit).free);
            free_fn((*c_xrle).xrle.lit_codec);
        }
    }
    drop(Box::from_raw(c_xrle));
}

pub unsafe fn cram_cram_codecs_c_2184_cram_xrle_decode_init(
    hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_xrle_layout {
        codec: 52,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: std::ptr::null_mut(),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        xrle: cram_xrle_decoder_layout {
            len_encoding: 0,
            lit_encoding: 0,
            len_dat: std::ptr::null_mut(),
            lit_dat: std::ptr::null_mut(),
            len_codec: std::ptr::null_mut(),
            lit_codec: std::ptr::null_mut(),
            cur_len: 0,
            cur_lit: -1,
            rep_score: [0; 256],
            to_flush: std::ptr::null_mut(),
            to_flush_size: 0,
        },
    });

    c.decode = if option == 2 {
        cram_fn_ptr(cram_cram_codecs_c_2063_cram_xrle_decode_long as usize)
    } else if option == 1 {
        cram_fn_ptr(cram_cram_codecs_c_2068_cram_xrle_decode_int as usize)
    } else if option == 4 || option == 3 {
        cram_fn_ptr(cram_cram_codecs_c_2125_cram_xrle_decode_char as usize)
    } else {
        return std::ptr::null_mut();
    };
    c.free = cram_fn_ptr(cram_cram_codecs_c_2172_cram_xrle_decode_free as usize);
    c.size = cram_fn_ptr(cram_cram_codecs_c_2115_cram_xrle_decode_size as usize);
    c.get_block = cram_fn_ptr(cram_cram_codecs_c_2120_cram_xrle_get_block as usize);

    let vv_layout = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    let mut err = 0;

    let nrle = ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
    for _ in 0..nrle.min(256) {
        let j = ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
        if (0..256).contains(&j) {
            c.xrle.rep_score[j as usize] = 1;
        }
    }

    c.xrle.len_encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        return std::ptr::null_mut();
    }
    c.xrle.len_codec = cram_cram_codecs_c_3872_cram_decoder_init(
        hdr,
        c.xrle.len_encoding,
        cp,
        sub_size,
        1,
        version,
        vv,
    );
    if c.xrle.len_codec.is_null() {
        return std::ptr::null_mut();
    }
    cp = cp.add(sub_size as usize);

    c.xrle.lit_encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        let len = c.xrle.len_codec.cast::<cram_codec_xrle_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn(c.xrle.len_codec);
        }
        c.xrle.len_codec = std::ptr::null_mut();
        return std::ptr::null_mut();
    }
    c.xrle.lit_codec = cram_cram_codecs_c_3872_cram_decoder_init(
        hdr,
        c.xrle.lit_encoding,
        cp,
        sub_size,
        option,
        version,
        vv,
    );
    if c.xrle.lit_codec.is_null() {
        let len = c.xrle.len_codec.cast::<cram_codec_xrle_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn(c.xrle.len_codec);
        }
        c.xrle.len_codec = std::ptr::null_mut();
        return std::ptr::null_mut();
    }

    if err != 0 {
        let len = c.xrle.len_codec.cast::<cram_codec_xrle_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn(c.xrle.len_codec);
        }
        c.xrle.len_codec = std::ptr::null_mut();
        let lit = c.xrle.lit_codec.cast::<cram_codec_xrle_layout>();
        if !(*lit).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*lit).free);
            free_fn(c.xrle.lit_codec);
        }
        c.xrle.lit_codec = std::ptr::null_mut();
        return std::ptr::null_mut();
    }

    Box::into_raw(c).cast()
}

pub unsafe extern "C" fn cram_cram_codecs_c_2257_cram_xrle_encode_flush(c: *mut c_void) -> c_int {
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    let mut rle_syms = [0u8; 256];
    let mut rle_nsyms = 0;
    for i in 0..256usize {
        if (*c_xrle).xrle.rep_score[i] > 0 {
            rle_syms[rle_nsyms] = i as u8;
            rle_nsyms += 1;
        }
    }

    if (*c_xrle).xrle.to_flush.is_null() {
        let out = (*c_xrle).out.cast::<cram_block_layout>();
        (*c_xrle).xrle.to_flush = (*out).data.cast();
        (*c_xrle).xrle.to_flush_size = (*out).byte;
    }

    let Some(out_len_cap) = (*c_xrle).xrle.to_flush_size.checked_add(8) else {
        return -1;
    };
    let mut out_len = Vec::<u8>::new();
    if out_len.try_reserve_exact(out_len_cap).is_err() {
        return -1;
    }
    out_len.resize(out_len_cap, 0);
    let out_len_ptr = out_len.as_mut_ptr();

    let mut v = (*c_xrle).xrle.to_flush_size as u64;
    let mut nb = 0usize;
    loop {
        *out_len_ptr.add(nb) = ((v & 0x7f) as u8) + if v >= 0x80 { 0x80 } else { 0 };
        nb += 1;
        v >>= 7;
        if v == 0 {
            break;
        }
    }

    let mut out_len_size = 0u64;
    let mut out_lit_size = 0u64;
    let mut rle_nsyms_i = rle_nsyms as c_int;
    let out_lit_vec = crate::htslib_rs::htscodecs::rle::hts_rle_encode_raw_slices(
        std::slice::from_raw_parts(
            (*c_xrle).xrle.to_flush.cast(),
            (*c_xrle).xrle.to_flush_size,
        ),
        std::slice::from_raw_parts_mut(out_len_ptr.add(nb), out_len_cap - nb),
        &mut out_len_size,
        &mut rle_syms[..rle_nsyms],
        &mut rle_nsyms_i,
        None,
        &mut out_lit_size,
    );
    let out_lit: *mut u8 = match out_lit_vec {
        Some(mut v) => {
            let p = v.as_mut_ptr();
            std::mem::forget(v);
            p
        }
        None => std::ptr::null_mut(),
    };
    out_len_size += nb as u64;

    let len_codec = (*c_xrle).xrle.len_codec;
    let len_encode: CramCodecEncodeFn =
        cram_fn((*(len_codec.cast::<cram_codec_xrle_layout>())).encode);
    if len_encode(
        std::ptr::null_mut(),
        len_codec,
        out_len_ptr.cast(),
        out_len_size as c_int,
    ) != 0
    {
        free(out_lit.cast());
        return -1;
    }

    let lit_codec = (*c_xrle).xrle.lit_codec;
    let lit_encode: CramCodecEncodeFn =
        cram_fn((*(lit_codec.cast::<cram_codec_xrle_layout>())).encode);
    if lit_encode(
        std::ptr::null_mut(),
        lit_codec,
        out_lit.cast(),
        out_lit_size as c_int,
    ) != 0
    {
        free(out_lit.cast());
        return -1;
    }

    free(out_lit.cast());
    0
}

pub unsafe extern "C" fn cram_cram_codecs_c_2303_cram_xrle_encode_store(
    c: *mut c_void,
    b: *mut cram_block,
    prefix: *mut c_char,
    version: c_int,
) -> c_int {
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    let Some(b_ref) = raw_block_mut(b) else {
        return -1;
    };
    let mut len = match append_optional_c_prefix(b_ref, prefix) {
        Some(len) => len,
        None => return -1,
    };
    let mut r = 0;

    let b_rle = cram_cram_io_c_1388_cram_new_block(0, 0);
    if b_rle.is_null() {
        return -1;
    }
    let mut nrle = 0;
    let mut len1 = 0;
    for i in 0..256i32 {
        if (*c_xrle).xrle.rep_score[i as usize] > 0 {
            nrle += 1;
            let n = ((*(*c_xrle).vv).varint_put32_blk.unwrap())(b_rle, i);
            len1 += n;
            r |= n;
        }
    }

    let tc = (*c_xrle).xrle.len_codec;
    let b_len = cram_cram_io_c_1388_cram_new_block(0, 0);
    if b_len.is_null() {
        cram_cram_io_c_1565_cram_free_block(b_rle);
        return -1;
    }
    let store: CramCodecStoreFn = cram_fn((*(tc.cast::<cram_codec_xrle_layout>())).store);
    let len2 = store(tc, b_len, std::ptr::null_mut(), version);

    let tc = (*c_xrle).xrle.lit_codec;
    let b_lit = cram_cram_io_c_1388_cram_new_block(0, 0);
    if b_lit.is_null() {
        cram_cram_io_c_1565_cram_free_block(b_rle);
        cram_cram_io_c_1565_cram_free_block(b_len);
        return -1;
    }
    let store: CramCodecStoreFn = cram_fn((*(tc.cast::<cram_codec_xrle_layout>())).store);
    let len3 = store(tc, b_lit, std::ptr::null_mut(), version);

    let n = ((*(*c_xrle).vv).varint_put32_blk.unwrap())(b, (*c_xrle).codec);
    len += n;
    r |= n;
    let n = ((*(*c_xrle).vv).varint_put32_blk.unwrap())(
        b,
        len1 + len2 + len3 + ((*(*c_xrle).vv).varint_size.unwrap())(nrle as i64),
    );
    len += n;
    r |= n;
    let n = ((*(*c_xrle).vv).varint_put32_blk.unwrap())(b, nrle);
    len += n;
    r |= n;

    let b_rle_layout = b_rle.cast::<cram_block_layout>();
    if cram_cram_io_h_248_block_append(b, (*b_rle_layout).data.cast(), (*b_rle_layout).byte) != 0 {
        cram_cram_io_c_1565_cram_free_block(b_rle);
        cram_cram_io_c_1565_cram_free_block(b_len);
        cram_cram_io_c_1565_cram_free_block(b_lit);
        return -1;
    }
    let b_len_layout = b_len.cast::<cram_block_layout>();
    if cram_cram_io_h_248_block_append(b, (*b_len_layout).data.cast(), (*b_len_layout).byte) != 0 {
        cram_cram_io_c_1565_cram_free_block(b_rle);
        cram_cram_io_c_1565_cram_free_block(b_len);
        cram_cram_io_c_1565_cram_free_block(b_lit);
        return -1;
    }
    let b_lit_layout = b_lit.cast::<cram_block_layout>();
    if cram_cram_io_h_248_block_append(b, (*b_lit_layout).data.cast(), (*b_lit_layout).byte) != 0 {
        cram_cram_io_c_1565_cram_free_block(b_rle);
        cram_cram_io_c_1565_cram_free_block(b_len);
        cram_cram_io_c_1565_cram_free_block(b_lit);
        return -1;
    }

    cram_cram_io_c_1565_cram_free_block(b_rle);
    cram_cram_io_c_1565_cram_free_block(b_len);
    cram_cram_io_c_1565_cram_free_block(b_lit);

    if r > 0 {
        len + len1 + len2 + len3
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_2371_cram_xrle_encode_char(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xrle_layout>();
    if !(*c).xrle.to_flush.is_null() {
        if (*c).out.is_null() {
            (*c).out = cram_cram_io_c_1388_cram_new_block(0, 0).cast();
            if (*c).out.is_null() {
                return -1;
            }
        }
        if cram_cram_io_h_248_block_append(
            (*c).out.cast(),
            (*c).xrle.to_flush.cast(),
            (*c).xrle.to_flush_size,
        ) != 0
        {
            return -1;
        }
        (*c).xrle.to_flush = std::ptr::null_mut();
        (*c).xrle.to_flush_size = 0;
    }

    if !(*c).out.is_null() && (*((*c).out.cast::<cram_block_layout>())).byte > 0 {
        if cram_cram_io_h_248_block_append((*c).out.cast(), in_.cast(), in_size as usize) != 0 {
            return -1;
        }
        return 0;
    }

    (*c).xrle.to_flush = in_;
    (*c).xrle.to_flush_size = in_size as usize;
    0
}

pub unsafe fn cram_cram_codecs_c_2396_cram_xrle_encode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    if !(*c_xrle).xrle.len_codec.is_null() {
        let len = (*c_xrle).xrle.len_codec.cast::<cram_codec_xrle_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn((*c_xrle).xrle.len_codec);
        }
    }
    if !(*c_xrle).xrle.lit_codec.is_null() {
        let lit = (*c_xrle).xrle.lit_codec.cast::<cram_codec_xrle_layout>();
        if !(*lit).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*lit).free);
            free_fn((*c_xrle).xrle.lit_codec);
        }
    }
    cram_cram_io_c_1565_cram_free_block((*c_xrle).out.cast());
    drop(Box::from_raw(c_xrle));
}

pub unsafe fn cram_cram_codecs_c_2409_cram_xrle_encode_init(
    _st: *mut c_void,
    _codec: c_int,
    option: c_int,
    dat: *mut c_void,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_xrle_layout {
        codec: 52,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_2396_cram_xrle_encode_free as usize),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: cram_fn_ptr(cram_cram_codecs_c_2303_cram_xrle_encode_store as usize),
        size: std::ptr::null_mut(),
        flush: cram_fn_ptr(cram_cram_codecs_c_2257_cram_xrle_encode_flush as usize),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        xrle: cram_xrle_decoder_layout {
            len_encoding: 0,
            lit_encoding: 0,
            len_dat: std::ptr::null_mut(),
            lit_dat: std::ptr::null_mut(),
            len_codec: std::ptr::null_mut(),
            lit_codec: std::ptr::null_mut(),
            cur_len: -1,
            cur_lit: -1,
            rep_score: [0; 256],
            to_flush: std::ptr::null_mut(),
            to_flush_size: 0,
        },
    });
    c.encode = if option == 2 {
        cram_fn_ptr(cram_cram_codecs_c_2359_cram_xrle_encode_long as usize)
    } else if option == 1 {
        cram_fn_ptr(cram_cram_codecs_c_2365_cram_xrle_encode_int as usize)
    } else {
        cram_fn_ptr(cram_cram_codecs_c_2371_cram_xrle_encode_char as usize)
    };

    let e = dat.cast::<cram_xrle_decoder_layout>();
    c.xrle.len_encoding = (*e).len_encoding;
    c.xrle.lit_encoding = (*e).lit_encoding;
    c.xrle.len_dat = (*e).len_dat;
    c.xrle.lit_dat = (*e).lit_dat;
    c.xrle.len_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).len_encoding,
        None,
        3,
        (*e).len_dat,
        version,
        vv,
    );
    if c.xrle.len_codec.is_null() {
        return std::ptr::null_mut();
    }
    c.xrle.lit_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).lit_encoding,
        None,
        3,
        (*e).lit_dat,
        version,
        vv,
    );
    if c.xrle.lit_codec.is_null() {
        let len = c.xrle.len_codec.cast::<cram_codec_xrle_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn(c.xrle.len_codec);
        }
        c.xrle.len_codec = std::ptr::null_mut();
        return std::ptr::null_mut();
    }
    c.xrle.rep_score = (*e).rep_score;

    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_2452_cram_subexp_decode(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_subexp_layout>();
    let out_i = out.cast::<i32>();
    let k = (*c).subexp.k;
    for count in 0..*out_size {
        let i = cram_cram_codecs_c_95_get_one_bits_MSB(in_);
        if i < 0
            || cram_cram_codecs_h_230_cram_not_enough_bits(in_, if i > 0 { i + k - 1 } else { k })
                != 0
        {
            return -1;
        }
        let val = if i != 0 {
            let tail = i + k - 1;
            let bits = if tail != 0 {
                cram_cram_codecs_c_169_get_bits_MSB(in_, tail) as i32
            } else {
                0
            };
            bits + (1 << tail)
        } else if k != 0 {
            cram_cram_codecs_c_169_get_bits_MSB(in_, k) as i32
        } else {
            0
        };
        *out_i.add(count as usize) = val - (*c).subexp.offset;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2496_cram_subexp_decode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_subexp_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_2501_cram_subexp_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_subexp_layout>();
    let ks = &mut *ks;
    if kputsn(b"SUBEXP(offset=", 14, ks) < 0
        || kputw((*c).subexp.offset, ks) < 0
        || kputsn(b",k=", 3, ks) < 0
        || kputw((*c).subexp.k, ks) < 0
        || kputsn(b")", 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_2508_cram_subexp_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    if option != 1 {
        return std::ptr::null_mut();
    }
    let mut c = Box::new(cram_codec_subexp_layout {
        codec: 7,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_2496_cram_subexp_decode_free as usize),
        decode: cram_fn_ptr(cram_cram_codecs_c_2452_cram_subexp_decode as usize),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: cram_fn_ptr(cram_cram_codecs_c_2501_cram_subexp_describe as usize),
        subexp: cram_subexp_decoder_layout { offset: 0, k: -1 },
    });

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    c.subexp.offset =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    c.subexp.k =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    if cp.offset_from(data) != size as isize || c.subexp.k < 0 {
        return std::ptr::null_mut();
    }
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_2546_cram_gamma_decode(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_gamma_layout>();
    let out_i = out.cast::<i32>();
    for i in 0..*out_size {
        let mut nz = cram_cram_codecs_c_113_get_zero_bits_MSB(in_);
        if cram_cram_codecs_h_230_cram_not_enough_bits(in_, nz) != 0 {
            return -1;
        }
        let mut val = 1;
        while nz > 0 {
            val <<= 1;
            val |= cram_cram_codecs_c_73_get_bit_MSB(in_);
            nz -= 1;
        }
        *out_i.add(i as usize) = val - (*c).gamma.offset;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2570_cram_gamma_decode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_gamma_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_2575_cram_gamma_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_gamma_layout>();
    let ks = &mut *ks;
    if kputsn(b"GAMMA(offset=", 13, ks) < 0
        || kputw((*c).gamma.offset, ks) < 0
        || kputsn(b")", 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_2580_cram_gamma_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    if option != 1 || size < 1 {
        return std::ptr::null_mut();
    }
    let mut c = Box::new(cram_codec_gamma_layout {
        codec: 9,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_2570_cram_gamma_decode_free as usize),
        decode: cram_fn_ptr(cram_cram_codecs_c_2546_cram_gamma_decode as usize),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: cram_fn_ptr(cram_cram_codecs_c_2575_cram_gamma_describe as usize),
        gamma: cram_gamma_decoder_layout { offset: 0 },
    });

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    c.gamma.offset =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    if cp.offset_from(data) != size as isize {
        return std::ptr::null_mut();
    }
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_2622_code_sort(vp1: *const c_void, vp2: *const c_void) -> c_int {
    let c1 = vp1.cast::<cram_huffman_code_layout>();
    let c2 = vp2.cast::<cram_huffman_code_layout>();
    if (*c1).len != (*c2).len {
        (*c1).len - (*c2).len
    } else if (*c1).symbol < (*c2).symbol {
        -1
    } else if (*c1).symbol > (*c2).symbol {
        1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_2632_cram_huffman_decode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c = c.cast::<cram_codec_huffman_layout>();
    if !(*c).huffman.codes.is_null() {
        let len = (*c).huffman.ncodes.max(0) as usize;
        drop(Vec::from_raw_parts((*c).huffman.codes, len, len));
    }
    drop(Box::from_raw(c));
}

pub unsafe fn cram_cram_codecs_c_2795_cram_huffman_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let ks = &mut *ks;
    let mut r = 0;
    r |= (kputsn(b"HUFFMAN(codes={", 15, ks) < 0) as c_int;
    for n in 0..(*c).huffman.ncodes {
        if n != 0 {
            r |= (kputsn(b",", 1, ks) < 0) as c_int;
        }
        r |= (kputll((*(*c).huffman.codes.add(n as usize)).symbol, ks) < 0) as c_int;
    }
    r |= (kputsn(b"},lengths={", 11, ks) < 0) as c_int;
    for n in 0..(*c).huffman.ncodes {
        if n != 0 {
            r |= (kputsn(b",", 1, ks) < 0) as c_int;
        }
        r |= (kputw((*(*c).huffman.codes.add(n as usize)).len, ks) < 0) as c_int;
    }
    r |= (kputsn(b"})", 2, ks) < 0) as c_int;
    r
}

pub unsafe fn cram_cram_codecs_c_2814_cram_huffman_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    if option == 5 {
        return std::ptr::null_mut();
    }

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let data_end = data.add(size as usize);
    let mut err = 0;
    let ncodes64 = ((*vv).varint_get32.unwrap())(&mut cp, data_end.cast_const(), &mut err);
    if ncodes64 < 0 {
        return std::ptr::null_mut();
    }
    let ncodes = ncodes64 as c_int;
    if ncodes as usize >= usize::MAX / std::mem::size_of::<cram_huffman_code_layout>() {
        *__errno_location() = ENOMEM;
        return std::ptr::null_mut();
    }

    let mut h = Box::new(cram_codec_huffman_layout {
        codec: 3,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_2632_cram_huffman_decode_free as usize),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        huffman: cram_huffman_decoder_layout {
            codes: std::ptr::null_mut(),
            ncodes,
            option,
        },
    });

    let mut codes = if ncodes != 0 {
        let mut v = Vec::<cram_huffman_code_layout>::new();
        if v.try_reserve_exact(ncodes as usize).is_err() {
            return std::ptr::null_mut();
        }
        v.resize(
            ncodes as usize,
            cram_huffman_code_layout {
                symbol: 0,
                p: 0,
                code: 0,
                len: 0,
            },
        );
        v
    } else {
        Vec::new()
    };
    h.huffman.codes = codes.as_mut_ptr();

    if option == 2 {
        for i in 0..ncodes {
            codes[i as usize].symbol =
                ((*vv).varint_get64.unwrap())(&mut cp, data_end.cast_const(), &mut err);
        }
    } else if option == 1 || option == 3 {
        for i in 0..ncodes {
            codes[i as usize].symbol =
                ((*vv).varint_get32.unwrap())(&mut cp, data_end.cast_const(), &mut err);
        }
    } else {
        return std::ptr::null_mut();
    }
    if err != 0 {
        return std::ptr::null_mut();
    }

    let n_lens = ((*vv).varint_get32.unwrap())(&mut cp, data_end.cast_const(), &mut err) as c_int;
    if n_lens != ncodes {
        return std::ptr::null_mut();
    }

    if ncodes == 0 {
        h.decode = cram_fn_ptr(cram_cram_codecs_c_2641_cram_huffman_decode_null as usize);
        h.huffman.codes = std::ptr::null_mut();
        return Box::into_raw(h).cast();
    }

    let mut max_len = 0;
    for i in 0..ncodes {
        let len = ((*vv).varint_get32.unwrap())(&mut cp, data_end.cast_const(), &mut err) as i32;
        codes[i as usize].len = len;
        if err != 0 || len < 0 {
            return std::ptr::null_mut();
        }
        if max_len < len {
            max_len = len;
        }
    }
    if err != 0 || cp.offset_from(data) != size as isize || max_len >= ncodes {
        return std::ptr::null_mut();
    }
    if max_len > 31 {
        return std::ptr::null_mut();
    }

    codes.sort_by(|a, b| {
        if a.len != b.len {
            a.len.cmp(&b.len)
        } else {
            a.symbol.cmp(&b.symbol)
        }
    });

    let mut val = -1;
    let mut last_len = 0;
    let mut max_val = 0u32;
    for i in 0..ncodes {
        val += 1;
        if val as u32 > max_val {
            return std::ptr::null_mut();
        }
        if codes[i as usize].len > last_len {
            val <<= codes[i as usize].len - last_len;
            last_len = codes[i as usize].len;
            max_val = (1u32 << last_len) - 1;
        }
        codes[i as usize].code = val;
    }

    last_len = 0;
    let mut j = 0;
    for i in 0..ncodes {
        if codes[i as usize].len > last_len {
            j = codes[i as usize].code - i;
            last_len = codes[i as usize].len;
        }
        codes[i as usize].p = j;
    }

    if option == 3 || option == 4 {
        h.decode = if codes[0].len == 0 {
            cram_fn_ptr(cram_cram_codecs_c_2646_cram_huffman_decode_char0 as usize)
        } else {
            cram_fn_ptr(cram_cram_codecs_c_2660_cram_huffman_decode_char as usize)
        };
    } else if option == 2 || option == 7 {
        h.decode = if codes[0].len == 0 {
            cram_fn_ptr(cram_cram_codecs_c_2745_cram_huffman_decode_long0 as usize)
        } else {
            cram_fn_ptr(cram_cram_codecs_c_2758_cram_huffman_decode_long as usize)
        };
    } else if option == 1 || option == 6 || option == 3 {
        h.decode = if codes[0].len == 0 {
            cram_fn_ptr(cram_cram_codecs_c_2695_cram_huffman_decode_int0 as usize)
        } else {
            cram_fn_ptr(cram_cram_codecs_c_2708_cram_huffman_decode_int as usize)
        };
    } else {
        return std::ptr::null_mut();
    }
    h.describe = cram_fn_ptr(cram_cram_codecs_c_2795_cram_huffman_describe as usize);
    h.huffman.codes = codes.as_mut_ptr();
    std::mem::forget(codes);

    Box::into_raw(h).cast()
}

pub unsafe fn cram_cram_codecs_c_2646_cram_huffman_decode_char0(
    _slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    if out.is_null() {
        return 0;
    }
    let c = c.cast::<cram_codec_huffman_layout>();
    let symbol = (*(*c).huffman.codes).symbol as c_char;
    for i in 0..*out_size {
        *out.add(i as usize) = symbol;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2660_cram_huffman_decode_char(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let ncodes = (*c).huffman.ncodes;
    let codes = (*c).huffman.codes;
    for i in 0..*out_size {
        let mut idx = 0;
        let mut val = 0;
        let mut len = 0;
        let mut last_len = 0;
        loop {
            let mut dlen = (*codes.add(idx as usize)).len - last_len;
            if cram_cram_codecs_h_230_cram_not_enough_bits(in_, dlen) != 0 {
                return -1;
            }
            last_len = {
                len += dlen;
                len
            };
            while dlen != 0 {
                val <<= 1;
                val |= cram_cram_codecs_c_73_get_bit_MSB(in_);
                dlen -= 1;
            }
            idx = val - (*codes.add(idx as usize)).p;
            if idx >= ncodes || idx < 0 {
                return -1;
            }
            if (*codes.add(idx as usize)).code == val && (*codes.add(idx as usize)).len == len {
                if !out.is_null() {
                    *out.add(i as usize) = (*codes.add(idx as usize)).symbol as c_char;
                }
                break;
            }
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2695_cram_huffman_decode_int0(
    _slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let out_i = out.cast::<i32>();
    let symbol = (*(*c).huffman.codes).symbol as i32;
    for i in 0..*out_size {
        *out_i.add(i as usize) = symbol;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2708_cram_huffman_decode_int(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let ncodes = (*c).huffman.ncodes;
    let codes = (*c).huffman.codes;
    let out_i = out.cast::<i32>();
    for i in 0..*out_size {
        let mut idx = 0;
        let mut val = 0;
        let mut len = 0;
        let mut last_len = 0;
        loop {
            let mut dlen = (*codes.add(idx as usize)).len - last_len;
            if cram_cram_codecs_h_230_cram_not_enough_bits(in_, dlen) != 0 {
                return -1;
            }
            last_len = {
                len += dlen;
                len
            };
            while dlen != 0 {
                val <<= 1;
                val |= cram_cram_codecs_c_73_get_bit_MSB(in_);
                dlen -= 1;
            }
            idx = val - (*codes.add(idx as usize)).p;
            if idx >= ncodes || idx < 0 {
                return -1;
            }
            if (*codes.add(idx as usize)).code == val && (*codes.add(idx as usize)).len == len {
                *out_i.add(i as usize) = (*codes.add(idx as usize)).symbol as i32;
                break;
            }
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2745_cram_huffman_decode_long0(
    _slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let out_i = out.cast::<i64>();
    let symbol = (*(*c).huffman.codes).symbol;
    for i in 0..*out_size {
        *out_i.add(i as usize) = symbol;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2758_cram_huffman_decode_long(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let ncodes = (*c).huffman.ncodes;
    let codes = (*c).huffman.codes;
    let out_i = out.cast::<i64>();
    for i in 0..*out_size {
        let mut idx = 0;
        let mut val = 0;
        let mut len = 0;
        let mut last_len = 0;
        loop {
            let mut dlen = (*codes.add(idx as usize)).len - last_len;
            if cram_cram_codecs_h_230_cram_not_enough_bits(in_, dlen) != 0 {
                return -1;
            }
            last_len = {
                len += dlen;
                len
            };
            while dlen != 0 {
                val <<= 1;
                val |= cram_cram_codecs_c_73_get_bit_MSB(in_);
                dlen -= 1;
            }
            idx = val - (*codes.add(idx as usize)).p;
            if idx >= ncodes || idx < 0 {
                return -1;
            }
            if (*codes.add(idx as usize)).code == val && (*codes.add(idx as usize)).len == len {
                *out_i.add(i as usize) = (*codes.add(idx as usize)).symbol;
                break;
            }
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2994_cram_huffman_encode_char(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    mut in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_encoder_layout>();
    let mut syms = in_.cast::<u8>();
    let mut r = 0;
    while in_size != 0 {
        let sym = *syms as c_int;
        syms = syms.add(1);
        let i = if (-1..128).contains(&sym) {
            (*c).huffman.val2code[(sym + 1) as usize]
        } else {
            let mut i = 0;
            while i < (*c).huffman.nvals {
                if (*(*c).huffman.codes.add(i as usize)).symbol == sym as i64 {
                    break;
                }
                i += 1;
            }
            if i == (*c).huffman.nvals {
                return -1;
            }
            i
        };
        let code = (*(*c).huffman.codes.add(i as usize)).code;
        let len = (*(*c).huffman.codes.add(i as usize)).len;
        r |= cram_cram_codecs_c_259_store_bits_MSB((*c).out.cast(), code as u64, len);
        in_size -= 1;
    }
    r
}

pub unsafe fn cram_cram_codecs_c_3030_cram_huffman_encode_int(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    mut in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_encoder_layout>();
    let mut syms = in_.cast::<c_int>();
    let mut r = 0;
    while in_size != 0 {
        let sym = *syms;
        syms = syms.add(1);
        let i = if (-1..128).contains(&sym) {
            (*c).huffman.val2code[(sym + 1) as usize]
        } else {
            let mut i = 0;
            while i < (*c).huffman.nvals {
                if (*(*c).huffman.codes.add(i as usize)).symbol == sym as i64 {
                    break;
                }
                i += 1;
            }
            if i == (*c).huffman.nvals {
                return -1;
            }
            i
        };
        let code = (*(*c).huffman.codes.add(i as usize)).code;
        let len = (*(*c).huffman.codes.add(i as usize)).len;
        r |= cram_cram_codecs_c_259_store_bits_MSB((*c).out.cast(), code as u64, len);
        in_size -= 1;
    }
    r
}

pub unsafe fn cram_cram_codecs_c_3067_cram_huffman_encode_long(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    mut in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_encoder_layout>();
    let mut syms = in_.cast::<i64>();
    let mut r = 0;
    while in_size != 0 {
        let sym64 = *syms;
        syms = syms.add(1);
        let i = if (-1..128).contains(&sym64) {
            (*c).huffman.val2code[(sym64 + 1) as usize]
        } else {
            let mut i = 0;
            while i < (*c).huffman.nvals {
                if (*(*c).huffman.codes.add(i as usize)).symbol == sym64 {
                    break;
                }
                i += 1;
            }
            if i == (*c).huffman.nvals {
                return -1;
            }
            i
        };
        let code = (*(*c).huffman.codes.add(i as usize)).code;
        let len = (*(*c).huffman.codes.add(i as usize)).len;
        r |= cram_cram_codecs_c_259_store_bits_MSB((*c).out.cast(), code as u64, len);
        in_size -= 1;
    }
    r
}

pub unsafe fn cram_cram_codecs_c_3099_cram_huffman_encode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c = c.cast::<cram_codec_huffman_encoder_layout>();
    if !(*c).huffman.codes.is_null() {
        let len = (*c).huffman.nvals.max(0) as usize;
        drop(Vec::from_raw_parts((*c).huffman.codes, len, len));
    }
    drop(Box::from_raw(c));
}

pub unsafe fn cram_cram_codecs_c_3112_cram_huffman_encode_store(
    c: *mut c_void,
    b: *mut cram_block,
    prefix: *mut c_char,
    _version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_encoder_layout>();
    let codes = (*c).huffman.codes;
    let tmp_len = 6usize
        .saturating_mul((*c).huffman.nvals as usize)
        .saturating_add(16);
    let mut tmp = Vec::<c_char>::new();
    if tmp.try_reserve_exact(tmp_len).is_err() {
        return -1;
    }
    tmp.resize(tmp_len, 0);
    let tmp_ptr = tmp.as_mut_ptr();
    let mut tp = tmp_ptr;
    let tpend = tmp_ptr.add(tmp_len);
    let Some(b_ref) = raw_block_mut(b) else {
        return -1;
    };
    let mut len = match append_optional_c_prefix(b_ref, prefix) {
        Some(len) => len,
        None => return -1,
    };
    let mut r = 0;

    tp = tp.add(((*(*c).vv).varint_put32.unwrap())(tp, tpend, (*c).huffman.nvals) as usize);
    if (*c).huffman.option == 2 {
        for i in 0..(*c).huffman.nvals {
            tp = tp.add(((*(*c).vv).varint_put64.unwrap())(
                tp,
                tpend,
                (*codes.add(i as usize)).symbol,
            ) as usize);
        }
    } else if (*c).huffman.option == 7 {
        for i in 0..(*c).huffman.nvals {
            tp = tp.add(((*(*c).vv).varint_put64s.unwrap())(
                tp,
                tpend,
                (*codes.add(i as usize)).symbol,
            ) as usize);
        }
    } else if (*c).huffman.option == 1 || (*c).huffman.option == 3 {
        for i in 0..(*c).huffman.nvals {
            tp = tp.add(((*(*c).vv).varint_put32.unwrap())(
                tp,
                tpend,
                (*codes.add(i as usize)).symbol as i32,
            ) as usize);
        }
    } else if (*c).huffman.option == 6 {
        for i in 0..(*c).huffman.nvals {
            tp = tp.add(((*(*c).vv).varint_put32s.unwrap())(
                tp,
                tpend,
                (*codes.add(i as usize)).symbol as i32,
            ) as usize);
        }
    } else {
        return -1;
    }

    tp = tp.add(((*(*c).vv).varint_put32.unwrap())(tp, tpend, (*c).huffman.nvals) as usize);
    for i in 0..(*c).huffman.nvals {
        tp = tp.add(
            ((*(*c).vv).varint_put32.unwrap())(tp, tpend, (*codes.add(i as usize)).len) as usize,
        );
    }

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;
    let payload_len = tp.offset_from(tmp_ptr) as c_int;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, payload_len);
    len += n;
    r |= n;
    if cram_cram_io_h_248_block_append(b, tmp_ptr.cast(), payload_len as usize) != 0 {
        return -1;
    }
    len += payload_len;

    if r > 0 {
        len
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_3176_cram_huffman_encode_init(
    st: *mut c_void,
    _codec: c_int,
    option: c_int,
    _dat: *mut c_void,
    _version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let mut vals = Vec::<c_int>::new();
    let mut freqs = Vec::<c_int>::new();
    let mut max_val = 0i32;
    let mut min_val = i32::MAX;

    let mut c = Box::new(cram_codec_huffman_encoder_layout {
        codec: 3,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: std::ptr::null_mut(),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        huffman: cram_huffman_encoder_layout {
            codes: std::ptr::null_mut(),
            nvals: 0,
            val2code: [0; 129],
            option: 0,
        },
    });

    let st = st.cast::<cram_stats_layout>();
    for i in 0..1024usize {
        if (*st).freqs[i] == 0 {
            continue;
        }
        if vals.try_reserve_exact(1).is_err() || freqs.try_reserve_exact(1).is_err() {
            return std::ptr::null_mut();
        }
        vals.push(i as c_int);
        freqs.push((*st).freqs[i]);
        if max_val < i as i32 {
            max_val = i as i32;
        }
        if min_val > i as i32 {
            min_val = i as i32;
        }
    }

    if let Some(h) = (*st).h.as_deref() {
        let i_after_stat_loop = 1024i32;
        for k in 0..h.n_buckets {
            let flag = h.flags[(k >> 4) as usize];
            if ((flag >> ((k & 0xf) << 1)) & 3) != 0 {
                continue;
            }
            if vals.try_reserve_exact(1).is_err() || freqs.try_reserve_exact(1).is_err() {
                return std::ptr::null_mut();
            }
            vals.push(h.keys[k as usize] as c_int);
            freqs.push(h.vals[k as usize]);
            if max_val < i_after_stat_loop {
                max_val = i_after_stat_loop;
            }
            if min_val > i_after_stat_loop {
                min_val = i_after_stat_loop;
            }
        }
    }

    let mut nvals = vals.len();
    if nvals == 0 {
        return std::ptr::null_mut();
    }

    if freqs.try_reserve_exact(nvals).is_err() {
        return std::ptr::null_mut();
    }
    freqs.resize(2 * nvals, 0);
    let mut lens = Vec::<c_int>::new();
    if lens.try_reserve_exact(2 * nvals).is_err() {
        return std::ptr::null_mut();
    }
    lens.resize(2 * nvals, 0);

    let mut heap_nvals = nvals;
    loop {
        let mut low1 = i32::MAX;
        let mut low2 = i32::MAX;
        let mut ind1 = 0usize;
        let mut ind2 = 0usize;
        for i in 0..heap_nvals {
            let f = freqs[i];
            if f < 0 {
                continue;
            }
            if low1 > f {
                low2 = low1;
                ind2 = ind1;
                low1 = f;
                ind1 = i;
            } else if low2 > f {
                low2 = f;
                ind2 = i;
            }
        }
        if low2 == i32::MAX {
            break;
        }
        freqs[heap_nvals] = low1 + low2;
        lens[ind1] = heap_nvals as c_int;
        lens[ind2] = heap_nvals as c_int;
        freqs[ind1] *= -1;
        freqs[ind2] *= -1;
        heap_nvals += 1;
    }
    nvals = heap_nvals / 2 + 1;

    for i in 0..nvals {
        let mut code_len = 0;
        let mut k = lens[i];
        while k != 0 {
            code_len += 1;
            k = lens[k as usize];
        }
        lens[i] = code_len;
        freqs[i] *= -1;
    }

    let mut codes = Vec::<cram_huffman_code_layout>::new();
    if codes.try_reserve_exact(nvals).is_err() {
        return std::ptr::null_mut();
    }
    for i in 0..nvals {
        codes.push(cram_huffman_code_layout {
            symbol: vals[i] as i64,
            p: 0,
            code: 0,
            len: lens[i],
        });
    }

    codes.sort_by(|a, b| {
        if a.len != b.len {
            a.len.cmp(&b.len)
        } else {
            a.symbol.cmp(&b.symbol)
        }
    });

    let mut code = 0;
    let mut len = codes[0].len;
    for i in 0..nvals {
        while len != codes[i].len {
            code <<= 1;
            len += 1;
        }
        codes[i].code = code;
        code += 1;

        let symbol = codes[i].symbol;
        if (-1..128).contains(&symbol) {
            c.huffman.val2code[(symbol + 1) as usize] = i as c_int;
        }
    }

    c.huffman.codes = codes.as_mut_ptr();
    c.huffman.nvals = nvals as c_int;
    c.huffman.option = option;
    c.free = cram_fn_ptr(cram_cram_codecs_c_3099_cram_huffman_encode_free as usize);
    c.encode = if option == 3 || option == 4 {
        if codes[0].len == 0 {
            cram_fn_ptr(cram_cram_codecs_c_2989_cram_huffman_encode_char0 as usize)
        } else {
            cram_fn_ptr(cram_cram_codecs_c_2994_cram_huffman_encode_char as usize)
        }
    } else if option == 1 || option == 6 {
        if codes[0].len == 0 {
            cram_fn_ptr(cram_cram_codecs_c_3025_cram_huffman_encode_int0 as usize)
        } else {
            cram_fn_ptr(cram_cram_codecs_c_3030_cram_huffman_encode_int as usize)
        }
    } else if option == 2 || option == 7 {
        if codes[0].len == 0 {
            cram_fn_ptr(cram_cram_codecs_c_3062_cram_huffman_encode_long0 as usize)
        } else {
            cram_fn_ptr(cram_cram_codecs_c_3067_cram_huffman_encode_long as usize)
        }
    } else {
        return std::ptr::null_mut();
    };
    c.store = cram_fn_ptr(cram_cram_codecs_c_3112_cram_huffman_encode_store as usize);
    std::mem::forget(codes);

    let _ = (max_val, min_val);
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_3371_cram_byte_array_len_decode(
    slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_len_layout>();
    let mut len = 0i32;
    let mut one = 1;
    let len_codec = (*c).byte_array_len.len_codec;
    let val_codec = (*c).byte_array_len.val_codec;
    let len_decode: CramCodecDecodeFn =
        cram_fn((*(len_codec.cast::<cram_codec_byte_array_len_layout>())).decode);
    let mut r = len_decode(
        slice,
        len_codec,
        in_,
        (&mut len as *mut i32).cast(),
        &mut one,
    );

    let val_layout = val_codec.cast::<cram_codec_external_layout>();
    let val_is_external_block =
        !val_codec.is_null() && (*val_layout).codec == 1 && (*val_layout).external.type_ == 5;
    if len < 0 || (len > *out_size && !val_is_external_block) {
        return -1;
    }

    if r == 0 && !val_codec.is_null() {
        let val_decode: CramCodecDecodeFn =
            cram_fn((*(val_codec.cast::<cram_codec_byte_array_len_layout>())).decode);
        r = val_decode(slice, val_codec, in_, out, &mut len);
    } else {
        return -1;
    }
    *out_size = len;
    r
}

pub unsafe fn cram_cram_codecs_c_3400_cram_byte_array_len_decode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_ba = c.cast::<cram_codec_byte_array_len_layout>();
    if !(*c_ba).byte_array_len.len_codec.is_null() {
        let len = (*c_ba)
            .byte_array_len
            .len_codec
            .cast::<cram_codec_byte_array_len_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn((*c_ba).byte_array_len.len_codec);
        }
    }
    if !(*c_ba).byte_array_len.val_codec.is_null() {
        let val = (*c_ba)
            .byte_array_len
            .val_codec
            .cast::<cram_codec_byte_array_len_layout>();
        if !(*val).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*val).free);
            free_fn((*c_ba).byte_array_len.val_codec);
        }
    }
    drop(Box::from_raw(c_ba));
}

pub unsafe fn cram_cram_codecs_c_3412_cram_byte_array_len_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_len_layout>();
    let mut r = 0;
    r |= (kputsn(b"BYTE_ARRAY_LEN(len_codec={", 26, &mut *ks) < 0) as c_int;
    let len_codec = (*c).byte_array_len.len_codec;
    if !(*(len_codec.cast::<cram_codec_byte_array_len_layout>()))
        .describe
        .is_null()
    {
        let describe: CramCodecDescribeFn =
            cram_fn((*(len_codec.cast::<cram_codec_byte_array_len_layout>())).describe);
        r |= describe(len_codec, ks);
    } else {
        r |= (kputsn(b"?", 1, &mut *ks) < 0) as c_int;
    }
    r |= (kputsn(b"},val_codec={", 13, &mut *ks) < 0) as c_int;
    let val_codec = (*c).byte_array_len.val_codec;
    if !(*(val_codec.cast::<cram_codec_byte_array_len_layout>()))
        .describe
        .is_null()
    {
        let describe: CramCodecDescribeFn =
            cram_fn((*(val_codec.cast::<cram_codec_byte_array_len_layout>())).describe);
        r |= describe(val_codec, ks);
    } else {
        r |= (kputsn(b"?", 1, &mut *ks) < 0) as c_int;
    }
    r |= (kputsn(b"}", 1, &mut *ks) < 0) as c_int;
    r
}

pub unsafe fn cram_cram_codecs_c_3428_cram_byte_array_len_decode_init(
    hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let mut c = Box::new(cram_codec_byte_array_len_layout {
        codec: 4,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_3400_cram_byte_array_len_decode_free as usize),
        decode: cram_fn_ptr(cram_cram_codecs_c_3371_cram_byte_array_len_decode as usize),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: cram_fn_ptr(cram_cram_codecs_c_3412_cram_byte_array_len_describe as usize),
        byte_array_len: cram_byte_array_len_decoder_layout {
            len_codec: std::ptr::null_mut(),
            val_codec: std::ptr::null_mut(),
        },
    });

    let vv_layout = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);

    let encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        return std::ptr::null_mut();
    }
    c.byte_array_len.len_codec =
        cram_cram_codecs_c_3872_cram_decoder_init(hdr, encoding, cp, sub_size, 1, version, vv);
    if c.byte_array_len.len_codec.is_null() {
        return std::ptr::null_mut();
    }
    cp = cp.add(sub_size as usize);

    let encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        let len = c
            .byte_array_len
            .len_codec
            .cast::<cram_codec_byte_array_len_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn(c.byte_array_len.len_codec);
        }
        c.byte_array_len.len_codec = std::ptr::null_mut();
        return std::ptr::null_mut();
    }
    c.byte_array_len.val_codec =
        cram_cram_codecs_c_3872_cram_decoder_init(hdr, encoding, cp, sub_size, option, version, vv);
    if c.byte_array_len.val_codec.is_null() {
        let len = c
            .byte_array_len
            .len_codec
            .cast::<cram_codec_byte_array_len_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn(c.byte_array_len.len_codec);
        }
        c.byte_array_len.len_codec = std::ptr::null_mut();
        return std::ptr::null_mut();
    }
    cp = cp.add(sub_size as usize);

    if cp.offset_from(data) != size as isize {
        let len = c
            .byte_array_len
            .len_codec
            .cast::<cram_codec_byte_array_len_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn(c.byte_array_len.len_codec);
        }
        c.byte_array_len.len_codec = std::ptr::null_mut();
        let val = c
            .byte_array_len
            .val_codec
            .cast::<cram_codec_byte_array_len_layout>();
        if !(*val).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*val).free);
            free_fn(c.byte_array_len.val_codec);
        }
        c.byte_array_len.val_codec = std::ptr::null_mut();
        return std::ptr::null_mut();
    }

    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_3479_cram_byte_array_len_encode(
    slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_len_layout>();
    let mut i32_ = in_size;
    let len_codec = (*c).byte_array_len.len_codec;
    let val_codec = (*c).byte_array_len.val_codec;
    let len_encode: CramCodecEncodeFn =
        cram_fn((*(len_codec.cast::<cram_codec_byte_array_len_layout>())).encode);
    let val_encode: CramCodecEncodeFn =
        cram_fn((*(val_codec.cast::<cram_codec_byte_array_len_layout>())).encode);
    let mut r = 0;
    r |= len_encode(slice, len_codec, (&mut i32_ as *mut i32).cast(), 1);
    r |= val_encode(slice, val_codec, in_, in_size);
    r
}

pub unsafe fn cram_cram_codecs_c_3493_cram_byte_array_len_encode_free(c: *mut c_void) {
    cram_cram_codecs_c_3400_cram_byte_array_len_decode_free(c);
}

pub unsafe fn cram_cram_codecs_c_3506_cram_byte_array_len_encode_store(
    c: *mut c_void,
    b: *mut cram_block,
    prefix: *mut c_char,
    version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_len_layout>();
    let Some(b_ref) = raw_block_mut(b) else {
        return -1;
    };
    let mut len = match append_optional_c_prefix(b_ref, prefix) {
        Some(len) => len,
        None => return -1,
    };
    let mut r = 0;

    let tc = (*c).byte_array_len.len_codec;
    let b_len = cram_cram_io_c_1388_cram_new_block(
        crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER,
        0,
    );
    if b_len.is_null() {
        return -1;
    }
    let store: CramCodecStoreFn = cram_fn((*(tc.cast::<cram_codec_byte_array_len_layout>())).store);
    let len2 = store(tc, b_len, std::ptr::null_mut(), version);
    if len2 < 0 {
        cram_cram_io_c_1565_cram_free_block(b_len);
        return -1;
    }

    let tc = (*c).byte_array_len.val_codec;
    let b_val = cram_cram_io_c_1388_cram_new_block(
        crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER,
        0,
    );
    if b_val.is_null() {
        cram_cram_io_c_1565_cram_free_block(b_len);
        return -1;
    }
    let store: CramCodecStoreFn = cram_fn((*(tc.cast::<cram_codec_byte_array_len_layout>())).store);
    let len3 = store(tc, b_val, std::ptr::null_mut(), version);
    if len3 < 0 {
        cram_cram_io_c_1565_cram_free_block(b_len);
        cram_cram_io_c_1565_cram_free_block(b_val);
        return -1;
    }

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, len2 + len3);
    len += n;
    r |= n;
    if cram_cram_io_h_248_block_append(
        b,
        (*(b_len.cast::<cram_block_layout>())).data.cast(),
        (*(b_len.cast::<cram_block_layout>())).byte,
    ) != 0
        || cram_cram_io_h_248_block_append(
            b,
            (*(b_val.cast::<cram_block_layout>())).data.cast(),
            (*(b_val.cast::<cram_block_layout>())).byte,
        ) != 0
    {
        cram_cram_io_c_1565_cram_free_block(b_len);
        cram_cram_io_c_1565_cram_free_block(b_val);
        return -1;
    }

    cram_cram_io_c_1565_cram_free_block(b_len);
    cram_cram_io_c_1565_cram_free_block(b_val);

    if r > 0 {
        len + len2 + len3
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_3547_cram_byte_array_len_encode_init(
    st: *mut c_void,
    _codec: c_int,
    _option: c_int,
    dat: *mut c_void,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let e = dat.cast::<cram_byte_array_len_encoder_dat_layout>();
    let mut c = Box::new(cram_codec_byte_array_len_layout {
        codec: 4,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_3493_cram_byte_array_len_encode_free as usize),
        decode: std::ptr::null_mut(),
        encode: cram_fn_ptr(cram_cram_codecs_c_3479_cram_byte_array_len_encode as usize),
        store: cram_fn_ptr(cram_cram_codecs_c_3506_cram_byte_array_len_encode_store as usize),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        byte_array_len: cram_byte_array_len_decoder_layout {
            len_codec: std::ptr::null_mut(),
            val_codec: std::ptr::null_mut(),
        },
    });

    c.byte_array_len.len_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).len_encoding,
        st.cast::<cram_stats_layout>().as_mut(),
        1,
        (*e).len_dat,
        version,
        vv,
    );
    c.byte_array_len.val_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).val_encoding,
        None,
        4,
        (*e).val_dat,
        version,
        vv,
    );
    if c.byte_array_len.len_codec.is_null() {
        return std::ptr::null_mut();
    }
    if c.byte_array_len.val_codec.is_null() {
        let len = c
            .byte_array_len
            .len_codec
            .cast::<cram_codec_byte_array_len_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = cram_fn((*len).free);
            free_fn(c.byte_array_len.len_codec);
        }
        c.byte_array_len.len_codec = std::ptr::null_mut();
        return std::ptr::null_mut();
    }

    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_3586_cram_byte_array_stop_decode_char(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    mut out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_stop_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).byte_array_stop.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let b = b.cast::<cram_block_layout>();
    if (*b).idx >= (*b).uncomp_size {
        return -1;
    }
    let mut term = (*b).uncomp_size - (*b).idx;
    let mut cp = (*b).data.add((*b).idx as usize);
    let start_idx = (*b).idx;
    if !out.is_null() {
        if term > *out_size {
            term = *out_size;
        }
        loop {
            term -= 1;
            if term < 0 || *cp == (*c).byte_array_stop.stop {
                break;
            }
            *out = *cp as c_char;
            out = out.add(1);
            cp = cp.add(1);
        }
    } else {
        loop {
            term -= 1;
            if term < 0 || *cp == (*c).byte_array_stop.stop {
                break;
            }
            cp = cp.add(1);
        }
    }
    if cp >= (*b).data.add((*b).uncomp_size as usize) || *cp != (*c).byte_array_stop.stop {
        return -1;
    }
    *out_size = cp.offset_from((*b).data.add(start_idx as usize)) as c_int;
    (*b).idx = cp.offset_from((*b).data) as i32 + 1;
    0
}

pub unsafe fn cram_cram_codecs_c_3626_cram_byte_array_stop_decode_block(
    slice: *mut cram_slice,
    c: *mut c_void,
    _in: *mut cram_block,
    out_: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_stop_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).byte_array_stop.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let b = b.cast::<cram_block_layout>();
    if (*b).idx >= (*b).uncomp_size {
        return -1;
    }
    let mut cp = (*b).data.add((*b).idx as usize);
    let cp_end = (*b).data.add((*b).uncomp_size as usize);
    let stop = if (*b).orig_method == 8 {
        0
    } else {
        (*c).byte_array_stop.stop
    };
    let cp_start = cp;
    while cp != cp_end && *cp != stop {
        cp = cp.add(1);
    }
    if cram_cram_io_h_248_block_append(
        out_.cast(),
        cp_start.cast(),
        cp.offset_from(cp_start) as usize,
    ) != 0
    {
        return -1;
    }
    *out_size = cp.offset_from((*b).data.add((*b).idx as usize)) as c_int;
    (*b).idx = cp.offset_from((*b).data) as i32 + 1;
    0
}

pub unsafe fn cram_cram_codecs_c_3669_cram_byte_array_stop_decode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_byte_array_stop_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_3675_cram_byte_array_stop_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_stop_layout>();
    let ks = &mut *ks;
    if kputsn(b"BYTE_ARRAY_STOP(stop=", 21, ks) < 0
        || kputw((*c).byte_array_stop.stop as c_int, ks) < 0
        || kputsn(b",id=", 4, ks) < 0
        || kputw((*c).byte_array_stop.content_id, ks) < 0
        || kputsn(b")", 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_3682_cram_byte_array_stop_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data.cast::<u8>();
    let min_size = if (version >> 8) == 1 { 5 } else { 2 };
    if size < min_size {
        return std::ptr::null_mut();
    }

    let mut c = Box::new(cram_codec_byte_array_stop_layout {
        codec: 5,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_3669_cram_byte_array_stop_decode_free as usize),
        decode: std::ptr::null_mut(),
        encode: std::ptr::null_mut(),
        store: std::ptr::null_mut(),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: cram_fn_ptr(cram_cram_codecs_c_3675_cram_byte_array_stop_describe as usize),
        byte_array_stop: cram_byte_array_stop_decoder_layout {
            stop: 0,
            content_id: 0,
        },
    });
    c.decode = match option {
        5 => cram_fn_ptr(cram_cram_codecs_c_3626_cram_byte_array_stop_decode_block as usize),
        4 => cram_fn_ptr(cram_cram_codecs_c_3586_cram_byte_array_stop_decode_char as usize),
        _ => {
            return std::ptr::null_mut();
        }
    };

    c.byte_array_stop.stop = *cp;
    cp = cp.add(1);
    if (version >> 8) == 1 {
        c.byte_array_stop.content_id = *cp.add(0) as i32
            + ((*cp.add(1) as i32) << 8)
            + ((*cp.add(2) as i32) << 16)
            + ((*cp.add(3) as u32) << 24) as i32;
        cp = cp.add(4);
    } else {
        let mut err = 0;
        let mut c_cp = cp.cast::<c_char>();
        let endp = data.add(size as usize);
        c.byte_array_stop.content_id =
            ((*vv).varint_get32.unwrap())(&mut c_cp, endp.cast_const(), &mut err) as i32;
        cp = c_cp.cast::<u8>();
        if err != 0 {
            return std::ptr::null_mut();
        }
    }

    if cp.cast::<c_char>().offset_from(data) != size as isize {
        return std::ptr::null_mut();
    }

    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_3733_cram_byte_array_stop_encode(
    _slice: *mut cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_stop_layout>();
    if cram_cram_io_h_248_block_append((*c).out.cast(), in_.cast(), in_size as usize) != 0 {
        return -1;
    }
    cram_cram_io_h_261_block_append_char((*c).out.cast(), (*c).byte_array_stop.stop as c_char)
}

pub unsafe fn cram_cram_codecs_c_3743_cram_byte_array_stop_encode_free(c: *mut c_void) {
    if !c.is_null() {
        drop(Box::from_raw(c.cast::<cram_codec_byte_array_stop_layout>()));
    }
}

pub unsafe fn cram_cram_codecs_c_3749_cram_byte_array_stop_encode_store(
    c: *mut c_void,
    b: *mut cram_block,
    prefix: *mut c_char,
    version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_stop_layout>();
    let Some(b_ref) = raw_block_mut(b) else {
        return -1;
    };
    let len = match append_optional_c_prefix(b_ref, prefix) {
        Some(len) => len,
        None => return -1,
    };

    let mut buf = [0 as c_char; 20];
    let mut cp = buf.as_mut_ptr();
    let endp = buf.as_mut_ptr().add(20);
    let vv = (*c).vv;
    cp = cp.add(((*vv).varint_put32.unwrap())(cp, endp, (*c).codec) as usize);
    if (version >> 8) == 1 {
        cp = cp.add(((*vv).varint_put32.unwrap())(cp, endp, 5) as usize);
        *cp = (*c).byte_array_stop.stop as c_char;
        cp = cp.add(1);
        *cp = (*c).byte_array_stop.content_id as c_char;
        cp = cp.add(1);
        *cp = ((*c).byte_array_stop.content_id >> 8) as c_char;
        cp = cp.add(1);
        *cp = ((*c).byte_array_stop.content_id >> 16) as c_char;
        cp = cp.add(1);
        *cp = ((*c).byte_array_stop.content_id >> 24) as c_char;
        cp = cp.add(1);
    } else {
        cp = cp.add(((*vv).varint_put32.unwrap())(
            cp,
            endp,
            1 + ((*vv).varint_size.unwrap())((*c).byte_array_stop.content_id as i64),
        ) as usize);
        *cp = (*c).byte_array_stop.stop as c_char;
        cp = cp.add(1);
        cp = cp
            .add(((*vv).varint_put32.unwrap())(cp, endp, (*c).byte_array_stop.content_id) as usize);
    }

    let n = cp.offset_from(buf.as_ptr()) as usize;
    if cram_cram_io_h_248_block_append(b, buf.as_ptr().cast(), n) != 0 {
        return -1;
    }
    len + n as c_int
}

pub unsafe fn cram_cram_codecs_c_3785_cram_byte_array_stop_encode_init(
    _st: *mut c_void,
    _codec: c_int,
    _option: c_int,
    dat: *mut c_void,
    _version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let dat = dat.cast::<c_int>();
    let c = Box::new(cram_codec_byte_array_stop_layout {
        codec: 5,
        out: std::ptr::null_mut(),
        vv: std::ptr::null_mut(),
        codec_id: 0,
        free: cram_fn_ptr(cram_cram_codecs_c_3743_cram_byte_array_stop_encode_free as usize),
        decode: std::ptr::null_mut(),
        encode: cram_fn_ptr(cram_cram_codecs_c_3733_cram_byte_array_stop_encode as usize),
        store: cram_fn_ptr(cram_cram_codecs_c_3749_cram_byte_array_stop_encode_store as usize),
        size: std::ptr::null_mut(),
        flush: std::ptr::null_mut(),
        get_block: std::ptr::null_mut(),
        describe: std::ptr::null_mut(),
        byte_array_stop: cram_byte_array_stop_decoder_layout {
            stop: *dat as u8,
            content_id: *dat.add(1),
        },
    });
    Box::into_raw(c).cast()
}

pub unsafe fn cram_cram_codecs_c_3872_cram_decoder_init(
    hdr: *mut c_void,
    codec: c_int,
    data: *mut c_char,
    size: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let init: Option<CramCodecDecodeInitFn> = match codec {
        1 => Some(cram_cram_codecs_c_459_cram_external_decode_init),
        3 => Some(cram_cram_codecs_c_2814_cram_huffman_decode_init),
        4 => Some(cram_cram_codecs_c_3428_cram_byte_array_len_decode_init),
        5 => Some(cram_cram_codecs_c_3682_cram_byte_array_stop_decode_init),
        6 => Some(cram_cram_codecs_c_1142_cram_beta_decode_init),
        7 => Some(cram_cram_codecs_c_2508_cram_subexp_decode_init),
        9 => Some(cram_cram_codecs_c_2580_cram_gamma_decode_init),
        41 | 42 => Some(cram_cram_codecs_c_760_cram_varint_decode_init),
        43 | 44 => Some(cram_cram_codecs_c_981_cram_const_decode_init),
        51 => Some(cram_cram_codecs_c_1453_cram_xpack_decode_init),
        52 => Some(cram_cram_codecs_c_2184_cram_xrle_decode_init),
        53 => Some(cram_cram_codecs_c_1781_cram_xdelta_decode_init),
        _ => None,
    };

    if let Some(init) = init {
        let r = init(hdr, data, size, codec, option, version, vv);
        if !r.is_null() {
            let hdr_layout = hdr.cast::<cram_block_compression_hdr_layout>();
            (*(r.cast::<cram_codec_external_layout>())).vv = vv.cast::<varint_vec_layout>();
            (*(r.cast::<cram_codec_external_layout>())).codec_id = (*hdr_layout).ncodecs;
            (*hdr_layout).ncodecs += 1;
        }
        r
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn cram_cram_codecs_c_3928_cram_encoder_init(
    mut codec: c_int,
    st: Option<&mut cram_stats_layout>,
    option: c_int,
    dat: *mut c_void,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let st: *mut c_void = match st {
        Some(s) => {
            if s.nvals == 0 {
                return std::ptr::null_mut();
            }
            (s as *mut cram_stats_layout).cast::<c_void>()
        }
        None => std::ptr::null_mut(),
    };

    if option == 3 || option == 4 || option == 5 {
        if codec == 41 || codec == 42 {
            codec = 1;
        } else if codec == 44 {
            codec = 43;
        }
    }

    let init: Option<CramCodecEncodeInitFn> = match codec {
        1 => Some(cram_cram_codecs_c_586_cram_external_encode_init),
        3 => Some(cram_cram_codecs_c_3176_cram_huffman_encode_init),
        4 => Some(cram_cram_codecs_c_3547_cram_byte_array_len_encode_init),
        5 => Some(cram_cram_codecs_c_3785_cram_byte_array_stop_encode_init),
        6 => Some(cram_cram_codecs_c_1247_cram_beta_encode_init),
        41 | 42 => Some(cram_cram_codecs_c_878_cram_varint_encode_init),
        43 | 44 => Some(cram_cram_codecs_c_1048_cram_const_encode_init),
        51 => Some(cram_cram_codecs_c_1623_cram_xpack_encode_init),
        52 => Some(cram_cram_codecs_c_2409_cram_xrle_encode_init),
        53 => Some(cram_cram_codecs_c_2022_cram_xdelta_encode_init),
        _ => None,
    };

    if let Some(init) = init {
        let r = init(st, codec, option, dat, version, vv);
        if r.is_null() {
            return std::ptr::null_mut();
        }
        (*(r.cast::<cram_codec_external_layout>())).out = std::ptr::null_mut();
        (*(r.cast::<cram_codec_external_layout>())).vv = vv.cast::<varint_vec_layout>();
        r
    } else {
        libc::abort();
    }
}

pub unsafe fn cram_cram_codecs_c_3968_cram_codec_to_id(c: *mut c_void, id2: *mut c_int) -> c_int {
    let codec = (*(c.cast::<cram_codec_external_layout>())).codec;
    let mut bnum2 = -2;
    let bnum1 = match codec {
        43 | 44 => -2,
        3 => {
            let c = c.cast::<cram_codec_huffman_layout>();
            if (*c).huffman.ncodes == 1 {
                -2
            } else {
                -1
            }
        }
        2 | 6 | 7 | 8 | 9 => -1,
        1 | 41 | 42 => {
            (*(c.cast::<cram_codec_external_layout>()))
                .external
                .content_id
        }
        4 => {
            let c = c.cast::<cram_codec_byte_array_len_layout>();
            let len_codec = (*c).byte_array_len.len_codec;
            let val_codec = (*c).byte_array_len.val_codec;
            bnum2 = cram_cram_codecs_c_3968_cram_codec_to_id(val_codec, std::ptr::null_mut());
            cram_cram_codecs_c_3968_cram_codec_to_id(len_codec, std::ptr::null_mut())
        }
        5 => {
            (*(c.cast::<cram_codec_byte_array_stop_layout>()))
                .byte_array_stop
                .content_id
        }
        0 => -2,
        _ => -1,
    };
    if !id2.is_null() {
        *id2 = bnum2;
    }
    bnum1
}

pub unsafe fn cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
    _fd: *mut c_void,
    c: *mut c_void,
) -> c_int {
    let base = c.cast::<cram_codec_external_layout>();
    match (*base).codec {
        43 | 44 => {
            (*base).store = cram_fn_ptr(cram_cram_codecs_c_1025_cram_const_encode_store as usize);
            0
        }
        1 => {
            (*base).free = cram_fn_ptr(cram_cram_codecs_c_556_cram_external_encode_free as usize);
            (*base).store = cram_fn_ptr(cram_cram_codecs_c_562_cram_external_encode_store as usize);
            (*base).encode = if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_350_cram_external_decode_int as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_523_cram_external_encode_int as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_370_cram_external_decode_long as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_535_cram_external_encode_long as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_390_cram_external_decode_char as usize)
                || (*base).decode
                    == cram_fn_ptr(cram_cram_codecs_c_410_cram_external_decode_block as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_547_cram_external_encode_char as usize)
            } else {
                return -1;
            };
            0
        }
        41 | 42 => {
            (*base).free = cram_fn_ptr(cram_cram_codecs_c_848_cram_varint_encode_free as usize);
            (*base).store = cram_fn_ptr(cram_cram_codecs_c_854_cram_varint_encode_store as usize);
            (*base).encode = if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_644_cram_varint_decode_int as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_820_cram_varint_encode_int as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_666_cram_varint_decode_sint as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_827_cram_varint_encode_sint as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_688_cram_varint_decode_long as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_834_cram_varint_encode_long as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_710_cram_varint_decode_slong as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_841_cram_varint_encode_slong as usize)
            } else {
                return -1;
            };
            0
        }
        6 => {
            (*base).free = cram_fn_ptr(cram_cram_codecs_c_1243_cram_beta_encode_free as usize);
            (*base).store = cram_fn_ptr(cram_cram_codecs_c_1183_cram_beta_encode_store as usize);
            (*base).encode = if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_1090_cram_beta_decode_int as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_1219_cram_beta_encode_int as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_1072_cram_beta_decode_long as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_1207_cram_beta_encode_long as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_1108_cram_beta_decode_char as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_1231_cram_beta_encode_char as usize)
            } else {
                return -1;
            };
            0
        }
        3 => {
            let dec = c.cast::<cram_codec_huffman_layout>();
            let enc = c.cast::<cram_codec_huffman_encoder_layout>();
            (*enc).codec = 3;
            (*enc).vv = (*dec).vv;
            (*enc).out = (*dec).out;
            (*enc).codec_id = (*dec).codec_id;
            (*enc).free = cram_fn_ptr(cram_cram_codecs_c_3099_cram_huffman_encode_free as usize);
            (*enc).store = cram_fn_ptr(cram_cram_codecs_c_3112_cram_huffman_encode_store as usize);
            let codes = (*dec).huffman.codes;
            let nvals = (*dec).huffman.ncodes;
            let option = (*dec).huffman.option;
            (*enc).huffman.codes = codes;
            (*enc).huffman.nvals = nvals;
            (*enc).huffman.val2code = [0; 129];
            (*enc).huffman.option = option;
            for j in 0..nvals {
                let sym = (*codes.add(j as usize)).symbol as i32;
                if (-1..128).contains(&sym) {
                    (*enc).huffman.val2code[(sym + 1) as usize] = j;
                }
            }
            (*enc).encode = if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_2646_cram_huffman_decode_char0 as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_2989_cram_huffman_encode_char0 as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_2660_cram_huffman_decode_char as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_2994_cram_huffman_encode_char as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_2695_cram_huffman_decode_int0 as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_3025_cram_huffman_encode_int0 as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_2708_cram_huffman_decode_int as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_3030_cram_huffman_encode_int as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_2745_cram_huffman_decode_long0 as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_3062_cram_huffman_encode_long0 as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_2758_cram_huffman_decode_long as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_3067_cram_huffman_encode_long as usize)
            } else {
                return -1;
            };
            0
        }
        51 => {
            (*base).free = cram_fn_ptr(cram_cram_codecs_c_1612_cram_xpack_encode_free as usize);
            (*base).store = cram_fn_ptr(cram_cram_codecs_c_1537_cram_xpack_encode_store as usize);
            (*base).encode = if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_1344_cram_xpack_decode_long as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_1581_cram_xpack_encode_long as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_1359_cram_xpack_decode_int as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_1592_cram_xpack_encode_int as usize)
            } else if (*base).decode
                == cram_fn_ptr(cram_cram_codecs_c_1408_cram_xpack_decode_char as usize)
            {
                cram_fn_ptr(cram_cram_codecs_c_1603_cram_xpack_encode_char as usize)
            } else {
                return -1;
            };
            let xpack = c.cast::<cram_codec_xpack_layout>();
            if cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                std::ptr::null_mut(),
                (*xpack).xpack.sub_codec,
            ) == -1
            {
                return -1;
            }
            0
        }
        4 => {
            (*base).free =
                cram_fn_ptr(cram_cram_codecs_c_3493_cram_byte_array_len_encode_free as usize);
            (*base).store =
                cram_fn_ptr(cram_cram_codecs_c_3506_cram_byte_array_len_encode_store as usize);
            (*base).encode =
                cram_fn_ptr(cram_cram_codecs_c_3479_cram_byte_array_len_encode as usize);
            let bal = c.cast::<cram_codec_byte_array_len_layout>();
            if cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                std::ptr::null_mut(),
                (*bal).byte_array_len.len_codec,
            ) == -1
                || cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                    std::ptr::null_mut(),
                    (*bal).byte_array_len.val_codec,
                ) == -1
            {
                return -1;
            }
            0
        }
        5 => {
            (*base).free =
                cram_fn_ptr(cram_cram_codecs_c_3743_cram_byte_array_stop_encode_free as usize);
            (*base).store =
                cram_fn_ptr(cram_cram_codecs_c_3749_cram_byte_array_stop_encode_store as usize);
            (*base).encode =
                cram_fn_ptr(cram_cram_codecs_c_3733_cram_byte_array_stop_encode as usize);
            0
        }
        _ => -1,
    }
}

pub unsafe fn cram_cram_codecs_c_4185_cram_codec_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    if !c.is_null()
        && !(*(c.cast::<cram_codec_external_layout>()))
            .describe
            .is_null()
    {
        let describe: CramCodecDescribeFn =
            cram_fn((*(c.cast::<cram_codec_external_layout>())).describe);
        describe(c, ks)
    } else if kputsn(b"?", 1, &mut *ks) < 0 {
        -1
    } else {
        0
    }
}
pub fn cram_cram_codecs_c_972_cram_const_decode_size(
    _slice: *mut cram_slice,
    _c: *mut c_void,
) -> c_int {
    0
}

pub fn cram_cram_codecs_c_1020_cram_const_encode(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    0
}

pub fn cram_cram_codecs_c_1676_zigzag8(x: i8) -> u8 {
    ((x.wrapping_shl(1)) ^ (x >> 7)) as u8
}

pub fn cram_cram_codecs_c_1677_zigzag16(x: i16) -> u16 {
    ((x.wrapping_shl(1)) ^ (x >> 15)) as u16
}

pub fn cram_cram_codecs_c_1678_zigzag32(x: i32) -> u32 {
    ((x.wrapping_shl(1)) ^ (x >> 31)) as u32
}

pub fn cram_cram_codecs_c_1681_unzigzag16(x: u16) -> i16 {
    (((x >> 1) as i32) ^ -((x & 1) as i32)) as i16
}

pub fn cram_cram_codecs_c_1682_unzigzag32(x: u32) -> i32 {
    ((x >> 1) ^ 0u32.wrapping_sub(x & 1)) as i32
}

pub fn cram_cram_codecs_c_1684_cram_xdelta_decode_long(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut cram_block,
    _out: *mut c_char,
    _out_size: *mut c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_1705_cram_xdelta_decode_expand_char(
    _slice: *mut cram_slice,
    _c: *mut c_void,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_1709_cram_xdelta_decode_char(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut cram_block,
    _out: *mut c_char,
    _out_size: *mut c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_1713_le_int2(i: i16) -> i16 {
    i16::from_ne_bytes(i.to_le_bytes())
}

pub fn cram_cram_codecs_c_1966_cram_xdelta_encode_long(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_1971_cram_xdelta_encode_int(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_2063_cram_xrle_decode_long(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut cram_block,
    _out: *mut c_char,
    _out_size: *mut c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_2068_cram_xrle_decode_int(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut cram_block,
    _out: *mut c_char,
    _out_size: *mut c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_2359_cram_xrle_encode_long(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_2365_cram_xrle_encode_int(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_2641_cram_huffman_decode_null(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut cram_block,
    _out: *mut c_char,
    _out_size: *mut c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_2989_cram_huffman_encode_char0(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    0
}

pub fn cram_cram_codecs_c_3025_cram_huffman_encode_int0(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    0
}

pub fn cram_cram_codecs_c_3062_cram_huffman_encode_long0(
    _slice: *mut cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    0
}

pub fn cram_cram_codecs_c_3811_cram_encoding2str(t: c_int) -> *mut c_char {
    let s: &'static [u8] = match t {
        0 => b"NULL\0",
        1 => b"EXTERNAL\0",
        2 => b"GOLOMB\0",
        3 => b"HUFFMAN\0",
        4 => b"BYTE_ARRAY_LEN\0",
        5 => b"BYTE_ARRAY_STOP\0",
        6 => b"BETA\0",
        7 => b"SUBEXP\0",
        8 => b"GOLOMB_RICE\0",
        9 => b"GAMMA\0",
        41 => b"VARINT_UNSIGNED\0",
        42 => b"VARINT_SIGNED\0",
        43 => b"CONST_BYTE\0",
        44 => b"CONST_INT\0",
        _ => b"?\0",
    };
    s.as_ptr().cast::<c_char>().cast_mut()
}
