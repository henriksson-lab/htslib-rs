use crate::htslib_rs::{cram as cram_api, sam};
use std::ffi::{c_char, c_int, c_uint, c_void};

const DS_RI: usize = 32;
const DS_RG: c_int = 17;
const DS_END: usize = 47;
const E_HUFFMAN: c_int = 3;
const E_BETA: c_int = 6;

// original: ds_list (htslib/cram/cram_external.c:304)
#[repr(C)]
pub struct ds_list {
    pub data_series: c_int,
    pub next: c_int,
}

// original: cram_cid2ds_t (htslib/cram/cram_external.c:312)
#[repr(C)]
pub struct cram_cid2ds_t {
    pub ds: *mut ds_list,
    pub ds_size: c_int,
    pub ds_idx: c_int,
    pub hash: *mut c_void,
    pub ds_a: *mut c_int,
}

type VarintGet32Fn = unsafe extern "C" fn(*mut *mut c_char, *const c_char, *mut c_int) -> i64;

#[inline]
fn mirror_cram_fd(fd: *mut c_void) -> *mut crate::htslib_rs::hts::cram_fd {
    fd.cast()
}

#[repr(C)]
struct varint_vec_layout {
    varint_decode32_crc: *mut c_void,
    varint_decode32s_crc: *mut c_void,
    varint_decode64_crc: *mut c_void,
    varint_get32: Option<VarintGet32Fn>,
    varint_get32s: *mut c_void,
    varint_get64: *mut c_void,
    varint_get64s: *mut c_void,
    varint_put32: *mut c_void,
    varint_put32s: *mut c_void,
    varint_put64: *mut c_void,
    varint_put64s: *mut c_void,
    varint_put32_blk: *mut c_void,
    varint_put32s_blk: *mut c_void,
    varint_put64_blk: *mut c_void,
    varint_put64s_blk: *mut c_void,
    varint_size: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_range_layout {
    refid: c_int,
    start: i64,
    end: i64,
}

#[repr(C)]
struct cram_fd_layout {
    fp: *mut c_void,
    mode: c_int,
    version: c_int,
    file_def: *mut c_void,
    header: *mut c_void,
    prefix: *mut c_char,
    record_counter: i64,
    err: c_int,
    ctr: *mut cram_container_layout,
    ctr_mt: *mut cram_container_layout,
    first_base: c_int,
    last_base: c_int,
    refs: *mut c_void,
    ref_: *mut c_char,
    ref_free: *mut c_char,
    ref_id: c_int,
    ref_start: i64,
    ref_end: i64,
    ref_fn: *mut c_char,
    level: c_int,
    m: [*mut c_void; DS_END],
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
    range: cram_range_layout,
    bam_flag_swap: [c_uint; 0x1000],
    cram_flag_swap: [c_uint; 0x1000],
    l1: [u8; 256],
    l2: [u8; 256],
    cram_sub_matrix: [[c_char; 32]; 32],
    index_sz: c_int,
    index: *mut c_void,
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
    idxfp: *mut c_void,
    vv: varint_vec_layout,
    ap_delta: c_int,
}

#[repr(C)]
struct cram_container_layout {
    length: i32,
    ref_seq_id: i32,
    ref_seq_start: i64,
    ref_seq_span: i64,
    record_counter: i64,
    num_bases: i64,
    num_records: i32,
    num_blocks: i32,
    num_landmarks: i32,
    landmark: *mut i32,
    offset: usize,
    comp_hdr: *mut cram_block_compression_hdr_layout,
    comp_hdr_block: *mut c_void,
    slice: *mut cram_slice_layout,
    curr_slice: c_int,
    max_slice: c_int,
}

#[repr(C)]
struct cram_block_layout {
    method: c_int,
    orig_method: c_int,
    content_type: c_int,
    content_id: i32,
    comp_size: i32,
    uncomp_size: i32,
    crc32: u32,
    idx: i32,
    data: *mut u8,
    alloc: usize,
    byte: usize,
    bit: c_int,
    m: *mut c_void,
    crc32_checked: c_int,
    crc_part: u32,
}

#[repr(C)]
struct cram_block_slice_hdr_layout {
    content_type: c_int,
    ref_seq_id: i32,
    ref_seq_start: i64,
    ref_seq_span: i64,
    num_records: i32,
    record_counter: i64,
    num_blocks: i32,
    num_content_ids: i32,
    block_content_ids: *mut i32,
    ref_base_id: i32,
    md5: [u8; 16],
}

#[repr(C)]
struct cram_record_layout {
    s: *mut cram_slice_layout,
    ref_id: i32,
    flags: i32,
    cram_flags: i32,
    len: i32,
    apos: i64,
    rg: i32,
    name: i32,
    name_len: i32,
    mate_line: i32,
    mate_ref_id: i32,
    mate_pos: i64,
    tlen: i64,
    explicit_tlen: i64,
    ntags: i32,
    aux: u32,
    aux_size: u32,
    tn_idx: i32,
    tl: c_int,
    seq: u32,
    qual: u32,
    cigar: u32,
    ncigar: i32,
    aend: i64,
    mqual: i32,
    feature: u32,
    nfeature: u32,
    mate_flags: i32,
}

#[repr(C)]
struct cram_slice_layout {
    hdr: *mut cram_block_slice_hdr_layout,
    hdr_block: *mut c_void,
    block: *mut *mut c_void,
    block_by_id: *mut *mut c_void,
    last_apos: i64,
    max_apos: i64,
    crecs: *mut cram_record_layout,
    cigar: *mut u32,
    cigar_alloc: u32,
    ncigar: u32,
    features: *mut c_void,
    nfeatures: u32,
    afeatures: u32,
    tn: *mut u32,
    ntn: c_int,
    atn: c_int,
    name_blk: *mut c_void,
    seqs_blk: *mut c_void,
    qual_blk: *mut c_void,
    base_blk: *mut c_void,
    soft_blk: *mut c_void,
    aux_blk: *mut c_void,
    pair_keys: *mut c_void,
    pair: [*mut c_void; 2],
    ref_: *mut c_char,
    ref_start: i64,
    ref_end: i64,
    ref_id: c_int,
    naux_block: c_int,
    aux_block: *mut *mut c_void,
    data_series: c_uint,
    decode_md: c_int,
    max_rec: c_int,
    curr_rec: c_int,
    slice_num: c_int,
}

#[repr(C)]
struct cram_block_compression_hdr_layout {
    ref_seq_id: i32,
    ref_seq_start: i64,
    ref_seq_span: i64,
    num_records: i32,
    num_landmarks: i32,
    landmark: *mut i32,
    read_names_included: i32,
    ap_delta: i32,
    substitution_matrix: [[c_char; 4]; 5],
    no_ref: i32,
    qs_seq_orient: i32,
    td_blk: *mut c_void,
    ntl: i32,
    tl: *mut *mut u8,
    td_hash: *mut c_void,
    td_keys: *mut c_void,
    preservation_map: *mut c_void,
    rec_encoding_map: [*mut c_void; 32],
    tag_encoding_map: [*mut c_void; 32],
    codecs: [*mut c_void; DS_END],
    uncomp: *mut c_char,
    uncomp_size: usize,
    uncomp_alloc: usize,
    ncodecs: i32,
}

#[repr(C)]
struct cram_huffman_code_layout {
    symbol: i64,
    p: i32,
    code: i32,
    len: i32,
}

#[repr(C)]
struct cram_huffman_decoder_layout {
    ncodes: c_int,
    codes: *mut cram_huffman_code_layout,
    option: c_int,
}

#[repr(C)]
struct cram_codec_huffman_layout {
    codec: c_int,
    out: *mut c_void,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    huffman: cram_huffman_decoder_layout,
}

#[repr(C)]
struct cram_beta_decoder_layout {
    offset: i32,
    nbits: i32,
}

#[repr(C)]
struct cram_codec_beta_layout {
    codec: c_int,
    out: *mut c_void,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    beta: cram_beta_decoder_layout,
}

unsafe extern "C" {
    fn cram_next_slice(
        fd: *mut c_void,
        cp: *mut *mut c_void,
    ) -> *mut c_void;
    fn cram_to_bam(
        h: *mut c_void,
        fd: *mut c_void,
        s: *mut c_void,
        cr: *mut c_void,
        recno: c_int,
        b: *mut *mut c_void,
    ) -> c_int;
    fn cram_put_bam_seq(fd: *mut c_void, b: *mut c_void) -> c_int;
    fn cram_encode_compression_header(
        fd: *mut c_void,
        c: *mut c_void,
        h: *mut c_void,
        embed_ref: c_int,
    ) -> *mut c_void;
}

// original: cram_block_compression_hdr_set_DS (htslib/cram/cram_external.c:152)
unsafe fn cram_block_compression_hdr_set_ds(
    ch: *mut cram_block_compression_hdr_layout,
    ds: c_int,
    new_rg: c_int,
) -> c_int {
    if ch.is_null() || (*ch).codecs[ds as usize].is_null() {
        return -1;
    }

    let co = (*ch).codecs[ds as usize];
    match *(co.cast::<c_int>()) {
        E_HUFFMAN => {
            let co = co.cast::<cram_codec_huffman_layout>();
            if (*co).huffman.ncodes != 1 {
                return -1;
            }
            (*(*co).huffman.codes).symbol = new_rg as i64;
            0
        }
        E_BETA => {
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

// original: cram_block_compression_hdr_set_rg (htslib/cram/cram_external.c:177)
unsafe fn cram_block_compression_hdr_set_rg(
    ch: *mut cram_block_compression_hdr_layout,
    new_rg: c_int,
) -> c_int {
    cram_block_compression_hdr_set_ds(ch, DS_RG, new_rg)
}

// original: cram_block_compression_hdr_decoder2encoder (htslib/cram/cram_external.c:189)
unsafe fn cram_block_compression_hdr_decoder2encoder(
    fd: *mut c_void,
    ch: *mut cram_block_compression_hdr_layout,
) -> c_int {
    if ch.is_null() {
        return -1;
    }

    for i in 0..DS_END {
        let co = (*ch).codecs[i];
        if co.is_null() {
            continue;
        }
        if cram_api::cram_cram_codecs_c_4031_cram_codec_decoder2encoder(fd.cast(), co) == -1 {
            return -1;
        }
    }

    0
}

unsafe fn cram_slice_hdr_get_num_blocks(h: *mut c_void) -> i32 {
    (*h.cast::<cram_block_slice_hdr_layout>()).num_blocks
}

unsafe fn cram_block_get_data(b: *mut c_void) -> *mut c_char {
    (*b.cast::<cram_block_layout>()).data.cast()
}

unsafe fn cram_block_get_size(b: *mut c_void) -> i32 {
    (*b.cast::<cram_block_layout>()).byte as i32
}

unsafe fn cram_block_get_uncomp_size(b: *mut c_void) -> i32 {
    (*b.cast::<cram_block_layout>()).uncomp_size
}

unsafe fn cram_block_set_size(b: *mut c_void, size: i32) {
    (*b.cast::<cram_block_layout>()).byte = size as usize;
}

unsafe fn cram_block_update_size(b: *mut c_void) {
    let b = b.cast::<cram_block_layout>();
    (*b).comp_size = (*b).byte as i32;
    (*b).uncomp_size = (*b).byte as i32;
}

unsafe fn cram_container_get_landmarks(
    c: *mut c_void,
    num_landmarks: *mut i32,
) -> *mut i32 {
    let c = c.cast::<cram_container_layout>();
    *num_landmarks = (*c).num_landmarks;
    (*c).landmark
}

unsafe fn cram_container_get_length(c: *mut c_void) -> i32 {
    (*c.cast::<cram_container_layout>()).length
}

unsafe fn cram_container_set_length(c: *mut c_void, length: i32) {
    (*c.cast::<cram_container_layout>()).length = length;
}

// original: cram_copy_slice (htslib/cram/cram_external.c:683)
unsafe fn cram_copy_slice(
    in_: *mut c_void,
    out: *mut c_void,
    num_slice: i32,
) -> c_int {
    for _ in 0..num_slice {
        let mut blk = cram_api::cram_read_block(mirror_cram_fd(in_));
        if blk.is_null() {
            return -1;
        }

        let hdr = cram_api::cram_decode_slice_header(mirror_cram_fd(in_), blk);
        if hdr.is_null() {
            cram_api::cram_cram_io_c_1565_cram_free_block(blk);
            return -1;
        }

        if cram_api::cram_write_block(mirror_cram_fd(out), blk) != 0 {
            cram_api::cram_cram_io_c_1565_cram_free_block(blk);
            return -1;
        }
        cram_api::cram_cram_io_c_1565_cram_free_block(blk);

        let num_blocks = cram_slice_hdr_get_num_blocks(hdr.cast());
        for _ in 0..num_blocks {
            blk = cram_api::cram_read_block(mirror_cram_fd(in_));
            if blk.is_null() || cram_api::cram_write_block(mirror_cram_fd(out), blk) != 0 {
                if !blk.is_null() {
                    cram_api::cram_cram_io_c_1565_cram_free_block(blk);
                }
                return -1;
            }
            cram_api::cram_cram_io_c_1565_cram_free_block(blk);
        }
        cram_api::cram_free_slice_header(hdr);
    }

    0
}

// original: cram_skip_container (htslib/cram/cram_external.c:725)
unsafe fn cram_skip_container(
    in_: *mut c_void,
    c: *mut c_void,
) -> c_int {
    let mut blk = cram_api::cram_read_block(mirror_cram_fd(in_));
    if blk.is_null() {
        return -1;
    }
    cram_api::cram_cram_io_c_1565_cram_free_block(blk);

    let c = c.cast::<cram_container_layout>();
    for _ in 0..(*c).num_landmarks {
        blk = cram_api::cram_read_block(mirror_cram_fd(in_));
        if blk.is_null() {
            return -1;
        }
        let hdr = cram_api::cram_decode_slice_header(mirror_cram_fd(in_), blk);
        if hdr.is_null() {
            cram_api::cram_cram_io_c_1565_cram_free_block(blk);
            return -1;
        }
        cram_api::cram_cram_io_c_1565_cram_free_block(blk);

        let num_blocks = cram_slice_hdr_get_num_blocks(hdr.cast());
        for _ in 0..num_blocks {
            blk = cram_api::cram_read_block(mirror_cram_fd(in_));
            if blk.is_null() {
                cram_api::cram_free_slice_header(hdr);
                return -1;
            }
            cram_api::cram_cram_io_c_1565_cram_free_block(blk);
        }
        cram_api::cram_free_slice_header(hdr);
    }

    0
}

// original: cram_filter_container (htslib/cram/cram_external.c:776)
pub unsafe fn cram_cram_external_c_776_cram_filter_container(
    in_: *mut c_void,
    out: *mut c_void,
    c: *mut c_void,
    ref_id: *mut c_int,
) -> c_int {
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
        return cram_skip_container(in_, c);
    }

    let blk = cram_api::cram_read_block(mirror_cram_fd(in_));
    if blk.is_null() {
        return -1;
    }
    (*c_layout).comp_hdr =
        cram_api::cram_decode_compression_header(mirror_cram_fd(in_), blk).cast();
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
            err |= (cram_api::cram_write_container(mirror_cram_fd(out), c.cast()) < 0) as c_int;
            err |= cram_api::cram_write_block(mirror_cram_fd(out), blk);
            return cram_copy_slice(in_, out, (*c_layout).num_landmarks) | -err;
        }
    }

    let rng_copy = (*in_fd).range;
    (*in_fd).range.start = i64::MIN;
    (*in_fd).range.end = i64::MAX;

    let mut b = sam::bam_init1();
    while (*c_layout).curr_slice < (*c_layout).max_slice
        || (!(*c_layout).slice.is_null()
            && (*(*c_layout).slice).curr_rec < (*(*c_layout).slice).max_rec)
    {
        let s = if !(*c_layout).slice.is_null()
            && (*(*c_layout).slice).curr_rec < (*(*c_layout).slice).max_rec
        {
            (*c_layout).slice
        } else if (*c_layout).curr_slice < (*c_layout).max_slice {
            cram_next_slice(in_, &mut c_ptr).cast()
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

        err |= (cram_to_bam(
            (*in_fd).header,
            in_,
            s.cast(),
            cr.cast(),
            {
                let rec = (*s).curr_rec;
                (*s).curr_rec += 1;
                rec
            },
            (&mut b as *mut *mut sam::bam1_t).cast(),
        ) < 0) as c_int;

        if cram_put_bam_seq(out, b.cast()) < 0 {
            err |= 1;
            break;
        }
    }
    sam::bam_destroy1(b);

    if !ref_id.is_null() {
        *ref_id = fixed_ref;
    }

    (*in_fd).range = rng_copy;
    (*in_fd).ctr = std::ptr::null_mut();
    (*in_fd).ctr_mt = std::ptr::null_mut();

    err |= cram_api::cram_flush(mirror_cram_fd(out));
    cram_api::cram_cram_io_c_1565_cram_free_block(blk);

    -err
}

// original: cram_transcode_rg (htslib/cram/cram_external.c:934)
pub unsafe fn cram_cram_external_c_934_cram_transcode_rg(
    in_: *mut c_void,
    out: *mut c_void,
    c: *mut c_void,
    nrg: c_int,
    _in_rg: *mut c_int,
    out_rg: *mut c_int,
) -> c_int {
    let in_fd = in_.cast::<cram_fd_layout>();
    let new_rg = *out_rg;

    if nrg != 1 {
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            c"cram_transcode_rg".as_ptr(),
            c"CRAM transcode supports only a single RG".as_ptr(),
        );
        return -2;
    }

    let o_blk = cram_api::cram_read_block(mirror_cram_fd(in_));
    let old_size = cram_api::cram_block_size(o_blk) as c_int;
    let ch = cram_api::cram_decode_compression_header(mirror_cram_fd(in_), o_blk);
    if cram_block_compression_hdr_set_rg(ch.cast(), new_rg) != 0 {
        return -1;
    }
    if cram_block_compression_hdr_decoder2encoder(in_, ch.cast()) != 0 {
        return -1;
    }
    let n_blk = cram_encode_compression_header(in_, c, ch.cast(), (*in_fd).embed_ref);
    cram_api::cram_free_compression_header(ch);

    let mut cp = cram_block_get_data(o_blk.cast());
    let mut op = cp;
    let endp = cp.add(cram_block_get_uncomp_size(o_blk.cast()) as usize);
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

    cram_block_set_size(n_blk, cram_block_get_size(n_blk) - 2);
    cram_api::cram_cram_external_c_544_cram_block_append(n_blk.cast(), op.cast(), i32_);
    cram_block_update_size(n_blk);

    let new_size = cram_api::cram_block_size(n_blk.cast()) as c_int;

    let mut num_landmarks = 0;
    let landmarks = cram_container_get_landmarks(c, &mut num_landmarks);

    if old_size != new_size {
        let diff = new_size - old_size;

        for j in 0..num_landmarks {
            *landmarks.add(j as usize) += diff;
        }
        cram_container_set_length(c, cram_container_get_length(c) + diff);
    }

    if cram_api::cram_write_container(mirror_cram_fd(out), c.cast()) != 0 {
        return -2;
    }

    cram_api::cram_write_block(mirror_cram_fd(out), n_blk.cast());
    cram_api::cram_cram_io_c_1565_cram_free_block(o_blk);
    cram_api::cram_cram_io_c_1565_cram_free_block(n_blk.cast());

    cram_copy_slice(in_, out, num_landmarks)
}
