use crate::htslib_mini_rs::{cram::cram_block, hts::kstring_t};
use std::ffi::{c_char, c_int, c_uchar, c_void};

use super::cram_structs::{cram_encoding, cram_slice, cram_stats, varint_vec};

pub type cram_external_type = c_int;

pub const E_INT: cram_external_type = 1;
pub const E_LONG: cram_external_type = 2;
pub const E_BYTE: cram_external_type = 3;
pub const E_BYTE_ARRAY: cram_external_type = 4;
pub const E_BYTE_ARRAY_BLOCK: cram_external_type = 5;
pub const E_SINT: cram_external_type = 6;
pub const E_SLONG: cram_external_type = 7;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_huffman_code {
    pub symbol: i64,
    pub p: i32,
    pub code: i32,
    pub len: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_huffman_decoder {
    pub ncodes: c_int,
    pub codes: *mut cram_huffman_code,
    pub option: c_int,
}

pub const MAX_HUFF: usize = 128;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_huffman_encoder {
    pub codes: *mut cram_huffman_code,
    pub nvals: c_int,
    pub val2code: [c_int; MAX_HUFF + 1],
    pub option: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_beta_decoder {
    pub offset: i32,
    pub nbits: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_xpack_decoder {
    pub nbits: i32,
    pub sub_encoding: cram_encoding,
    pub sub_codec_dat: *mut c_void,
    pub sub_codec: *mut cram_codec,
    pub nval: c_int,
    pub rmap: [u32; 256],
    pub map: [c_int; 256],
}

pub type cram_xpack_encoder = cram_xpack_decoder;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_xrle_decoder {
    pub len_encoding: cram_encoding,
    pub lit_encoding: cram_encoding,
    pub len_dat: *mut c_void,
    pub lit_dat: *mut c_void,
    pub len_codec: *mut cram_codec,
    pub lit_codec: *mut cram_codec,
    pub cur_len: c_int,
    pub cur_lit: c_int,
    pub rep_score: [c_int; 256],
    pub to_flush: *mut c_char,
    pub to_flush_size: usize,
}

pub type cram_xrle_encoder = cram_xrle_decoder;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_xdelta_decoder {
    pub last: i64,
    pub word_size: u8,
    pub sub_encoding: cram_encoding,
    pub sub_codec_dat: *mut c_void,
    pub sub_codec: *mut cram_codec,
}

pub type cram_xdelta_encoder = cram_xdelta_decoder;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_gamma_decoder {
    pub offset: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_subexp_decoder {
    pub offset: i32,
    pub k: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_external_decoder {
    pub content_id: i32,
    pub type_0: cram_external_type,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_varint_decoder {
    pub content_id: i32,
    pub offset: i64,
    pub type_0: cram_external_type,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_byte_array_len_decoder {
    pub len_codec: *mut cram_codec,
    pub val_codec: *mut cram_codec,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_byte_array_stop_decoder {
    pub stop: c_uchar,
    pub content_id: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_byte_array_len_encoder {
    pub len_encoding: cram_encoding,
    pub val_encoding: cram_encoding,
    pub len_dat: *mut c_void,
    pub val_dat: *mut c_void,
    pub len_codec: *mut cram_codec,
    pub val_codec: *mut cram_codec,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_const_codec {
    pub val: i64,
}

// original: cram_codec (htslib/cram/cram_codecs.h:163)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct cram_codec {
    pub codec: cram_encoding,
    pub out: *mut cram_block,
    pub vv: *mut varint_vec,
    pub codec_id: c_int,
    pub free: Option<unsafe extern "C" fn(codec: *mut cram_codec)>,
    pub decode: Option<
        unsafe extern "C" fn(
            slice: *mut cram_slice,
            codec: *mut cram_codec,
            in_: *mut cram_block,
            out: *mut c_char,
            out_size: *mut c_int,
        ) -> c_int,
    >,
    pub encode: Option<
        unsafe extern "C" fn(
            slice: *mut cram_slice,
            codec: *mut cram_codec,
            in_: *mut c_char,
            in_size: c_int,
        ) -> c_int,
    >,
    pub store: Option<
        unsafe extern "C" fn(
            codec: *mut cram_codec,
            b: *mut cram_block,
            prefix: *mut c_char,
            version: c_int,
        ) -> c_int,
    >,
    pub size: Option<unsafe extern "C" fn(slice: *mut cram_slice, codec: *mut cram_codec) -> c_int>,
    pub flush: Option<unsafe extern "C" fn(codec: *mut cram_codec) -> c_int>,
    pub get_block: Option<
        unsafe extern "C" fn(slice: *mut cram_slice, codec: *mut cram_codec) -> *mut cram_block,
    >,
    pub describe: Option<unsafe extern "C" fn(codec: *mut cram_codec, ks: *mut kstring_t) -> c_int>,
    pub u: cram_codec_u,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union cram_codec_u {
    pub huffman: cram_huffman_decoder,
    pub external: cram_external_decoder,
    pub beta: cram_beta_decoder,
    pub gamma: cram_gamma_decoder,
    pub subexp: cram_subexp_decoder,
    pub byte_array_len: cram_byte_array_len_decoder,
    pub byte_array_stop: cram_byte_array_stop_decoder,
    pub xpack: cram_xpack_decoder,
    pub xrle: cram_xrle_decoder,
    pub xdelta: cram_xdelta_decoder,
    pub xconst: cram_const_codec,
    pub varint: cram_varint_decoder,
    pub e_huffman: cram_huffman_encoder,
    pub e_external: cram_external_decoder,
    pub e_byte_array_stop: cram_byte_array_stop_decoder,
    pub e_byte_array_len: cram_byte_array_len_encoder,
    pub e_beta: cram_beta_decoder,
    pub e_xpack: cram_xpack_decoder,
    pub e_xrle: cram_xrle_decoder,
    pub e_xdelta: cram_xdelta_decoder,
    pub e_xconst: cram_const_codec,
    pub e_varint: cram_varint_decoder,
}

unsafe extern "C" {
    pub fn cram_encoding2str(t: cram_encoding) -> *const c_char;
    pub fn cram_decoder_init(
        hdr: *mut super::cram_structs::cram_block_compression_hdr,
        codec: cram_encoding,
        data: *mut c_char,
        size: c_int,
        option: cram_external_type,
        version: c_int,
        vv: *mut varint_vec,
    ) -> *mut cram_codec;
    pub fn cram_encoder_init(
        codec: cram_encoding,
        st: *mut cram_stats,
        option: cram_external_type,
        dat: *mut c_void,
        version: c_int,
        vv: *mut varint_vec,
    ) -> *mut cram_codec;
}
