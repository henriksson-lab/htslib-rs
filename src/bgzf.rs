use std::{
    ffi::{c_char, c_int, c_uint, c_ulong, c_void, CStr},
    ptr, slice,
};

use flate2::{Compress, Compression, Decompress, FlushCompress, FlushDecompress, Status};

use super::{
    c_compat,
    hts::{ed_is_big, ed_swap_8, ks_expand, kstring_t, BGZF},
};

const BGZF_MAX_BLOCK_SIZE: usize = 0x10000;
const BGZF_BLOCK_SIZE: usize = 0xff00;
const BLOCK_HEADER_LENGTH: usize = 18;
const BLOCK_FOOTER_LENGTH: usize = 8;

const BGZF_ERR_ZLIB: u32 = 1;
const BGZF_ERR_HEADER: u32 = 2;
const BGZF_ERR_IO: u32 = 4;
const BGZF_ERR_MISUSE: u32 = 8;
const BGZF_ERR_CRC: u32 = 32;

const SEEK_SET: c_int = 0;
const Z_OK: c_int = 0;
const Z_NEED_DICT: c_int = 2;
const Z_ERRNO: c_int = -1;
const Z_STREAM_ERROR: c_int = -2;
const Z_DATA_ERROR: c_int = -3;
const Z_MEM_ERROR: c_int = -4;
const Z_BUF_ERROR: c_int = -5;
const Z_VERSION_ERROR: c_int = -6;
const EOF_BLOCK: [u8; 28] = [
    31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 27, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static mut BGZF_ZERR_BUFFER: [c_char; 32] = [0; 32];

#[repr(C)]
pub struct z_stream_s {
    pub next_in: *const u8,
    pub avail_in: c_uint,
    pub total_in: c_ulong,
    pub next_out: *mut u8,
    pub avail_out: c_uint,
    pub total_out: c_ulong,
    pub msg: *mut c_char,
    pub state: *mut c_void,
    pub zalloc: *mut c_void,
    pub zfree: *mut c_void,
    pub opaque: *mut c_void,
    pub data_type: c_int,
    pub adler: c_ulong,
    pub reserved: c_ulong,
}

#[repr(C)]
pub struct bgzidx1_t {
    pub uaddr: u64,
    pub caddr: u64,
}

#[repr(C)]
pub struct bgzidx_t {
    pub noffs: c_int,
    pub moffs: c_int,
    pub offs: *mut bgzidx1_t,
    pub ublock_addr: u64,
}

fn flag(fp: *const BGZF, bit: u32) -> bool {
    unsafe { ((*fp).bitfields & (1 << bit)) != 0 }
}

unsafe fn set_flag(fp: *mut BGZF, bit: u32, value: bool) {
    if value {
        (*fp).bitfields |= 1 << bit;
    } else {
        (*fp).bitfields &= !(1 << bit);
    }
}

unsafe fn errcode(fp: *const BGZF) -> u32 {
    (*fp).bitfields & 0xffff
}

unsafe fn add_errcode(fp: *mut BGZF, err: u32) {
    (*fp).bitfields |= err & 0xffff;
}

unsafe fn set_compress_level(fp: *mut BGZF, level: c_int) {
    (*fp).bitfields &= !(0x1ff << 20);
    (*fp).bitfields |= ((level as u32) & 0x1ff) << 20;
}

fn unpack_u16(buf: &[u8]) -> u16 {
    u16::from_le_bytes([buf[0], buf[1]])
}

fn unpack_u32(buf: &[u8]) -> u32 {
    u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]])
}

pub unsafe fn packInt16(buffer: *mut u8, value: u16) {
    *buffer.add(0) = value as u8;
    *buffer.add(1) = (value >> 8) as u8;
}

pub unsafe fn unpackInt16(buffer: *const u8) -> c_int {
    *buffer.add(0) as c_int | ((*buffer.add(1) as c_int) << 8)
}

pub unsafe fn packInt32(buffer: *mut u8, value: u32) {
    *buffer.add(0) = value as u8;
    *buffer.add(1) = (value >> 8) as u8;
    *buffer.add(2) = (value >> 16) as u8;
    *buffer.add(3) = (value >> 24) as u8;
}

pub unsafe fn bgzf_zerr(errnum: c_int, zs: *mut z_stream_s) -> *const c_char {
    if !zs.is_null() && !(*zs).msg.is_null() {
        return (*zs).msg;
    }

    match errnum {
        Z_ERRNO => libc::strerror(*c_compat::__errno_location()),
        Z_STREAM_ERROR => c"invalid parameter/compression level, or inconsistent stream state"
            .as_ptr()
            .cast(),
        Z_DATA_ERROR => c"invalid or incomplete IO".as_ptr().cast(),
        Z_MEM_ERROR => c"out of memory".as_ptr().cast(),
        Z_BUF_ERROR => c"progress temporarily not possible, or in() / out() returned an error"
            .as_ptr()
            .cast(),
        Z_VERSION_ERROR => c"zlib version mismatch".as_ptr().cast(),
        Z_NEED_DICT => c"data was compressed using a dictionary".as_ptr().cast(),
        Z_OK | _ => {
            let buffer = std::ptr::addr_of_mut!(BGZF_ZERR_BUFFER).cast::<c_char>();
            libc::snprintf(buffer, 32, c"[%d] unknown".as_ptr(), errnum);
            buffer.cast()
        }
    }
}

pub unsafe fn check_header(header: *const u8) -> c_int {
    if *header.add(0) != 31 || *header.add(1) != 139 || *header.add(2) != 8 {
        return -2;
    }
    if (*header.add(3) & 4) != 0
        && unpackInt16(header.add(10)) == 6
        && *header.add(12) == b'B'
        && *header.add(13) == b'C'
        && unpackInt16(header.add(14)) == 2
    {
        0
    } else {
        -1
    }
}

fn crc32_update(crc: u32, bytes: &[u8]) -> u32 {
    let mut crc = !crc;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn crc32(bytes: &[u8]) -> u32 {
    crc32_update(0, bytes)
}

pub unsafe fn hts_crc32(crc: u32, buf: *const c_void, len: usize) -> u32 {
    if len == 0 {
        return crc32_update(crc, &[]);
    }
    crc32_update(crc, slice::from_raw_parts(buf.cast::<u8>(), len))
}

pub unsafe fn bgzf_c_557_hts_crc32(crc: u32, buf: *const c_void, len: usize) -> u32 {
    hts_crc32(crc, buf, len)
}

pub unsafe fn bgzf_c_620_hts_crc32(crc: u32, buf: *const c_void, len: usize) -> u32 {
    hts_crc32(crc, buf, len)
}

pub unsafe fn bgzf_compress(
    dst: *mut c_void,
    dlen: *mut usize,
    src: *const c_void,
    slen: usize,
    level: c_int,
) -> c_int {
    let src_bytes = slice::from_raw_parts(src.cast::<u8>(), slen);
    let dst_bytes = slice::from_raw_parts_mut(dst.cast::<u8>(), *dlen);

    let mut store_uncompressed = level == 0;
    let mut total_len = 0usize;
    if !store_uncompressed {
        if *dlen < BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH {
            return -1;
        }
        let zlevel = if level < 0 {
            6
        } else {
            level.clamp(0, 9) as u32
        };
        let mut encoder = Compress::new(Compression::new(zlevel), false);
        match encoder.compress(
            src_bytes,
            &mut dst_bytes[BLOCK_HEADER_LENGTH..*dlen - BLOCK_FOOTER_LENGTH],
            FlushCompress::Finish,
        ) {
            Ok(Status::StreamEnd) => {
                let compressed_len = encoder.total_out() as usize;
                if compressed_len == *dlen - BLOCK_HEADER_LENGTH - BLOCK_FOOTER_LENGTH {
                    store_uncompressed = true;
                } else {
                    total_len = compressed_len + BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH;
                }
            }
            _ => return -1,
        }
    }

    if store_uncompressed {
        total_len = slen + 5 + BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH;
        if *dlen < total_len {
            return -1;
        }
        *dst_bytes.as_mut_ptr().add(BLOCK_HEADER_LENGTH) = 1;
        packInt16(
            dst_bytes.as_mut_ptr().add(BLOCK_HEADER_LENGTH + 1),
            slen as u16,
        );
        packInt16(
            dst_bytes.as_mut_ptr().add(BLOCK_HEADER_LENGTH + 3),
            !(slen as u16),
        );
        ptr::copy_nonoverlapping(
            src_bytes.as_ptr(),
            dst_bytes.as_mut_ptr().add(BLOCK_HEADER_LENGTH + 5),
            slen,
        );
    }

    dst_bytes[..BLOCK_HEADER_LENGTH]
        .copy_from_slice(&[31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 0, 0]);
    packInt16(dst_bytes.as_mut_ptr().add(16), (total_len - 1) as u16);
    packInt32(
        dst_bytes.as_mut_ptr().add(total_len - 8),
        hts_crc32(0, src, slen),
    );
    packInt32(dst_bytes.as_mut_ptr().add(total_len - 4), slen as u32);
    *dlen = total_len;
    0
}

pub unsafe fn bgzf_c_561_bgzf_compress(
    dst: *mut c_void,
    dlen: *mut usize,
    src: *const c_void,
    slen: usize,
    level: c_int,
) -> c_int {
    bgzf_compress(dst, dlen, src, slen, level)
}

pub unsafe fn bgzf_c_624_bgzf_compress(
    dst: *mut c_void,
    dlen: *mut usize,
    src: *const c_void,
    slen: usize,
    level: c_int,
) -> c_int {
    bgzf_compress(dst, dlen, src, slen, level)
}

pub unsafe fn bgzf_uncompress(
    dst: *mut u8,
    dlen: *mut usize,
    src: *const u8,
    slen: usize,
    expected_crc: u32,
) -> c_int {
    let mut decoder = Decompress::new(false);
    let output = slice::from_raw_parts_mut(dst, *dlen);
    let input = slice::from_raw_parts(src, slen);
    match decoder.decompress(input, output, FlushDecompress::Finish) {
        Ok(Status::StreamEnd) => {}
        _ => return -1,
    }
    *dlen = decoder.total_out() as usize;
    if hts_crc32(0, dst.cast(), *dlen) != expected_crc {
        return -2;
    }
    0
}

pub unsafe fn bgzf_c_730_bgzf_uncompress(
    dst: *mut u8,
    dlen: *mut usize,
    src: *const u8,
    slen: usize,
    expected_crc: u32,
) -> c_int {
    bgzf_uncompress(dst, dlen, src, slen, expected_crc)
}

pub unsafe fn bgzf_c_762_bgzf_uncompress(
    dst: *mut u8,
    dlen: *mut usize,
    src: *const u8,
    slen: usize,
    expected_crc: u32,
) -> c_int {
    bgzf_uncompress(dst, dlen, src, slen, expected_crc)
}

fn fd_to_ptr(fd: c_int) -> *mut super::hts::hFILE {
    (fd as isize + 1) as usize as *mut super::hts::hFILE
}

fn ptr_to_fd(ptr: *mut super::hts::hFILE) -> c_int {
    (ptr as usize as isize - 1) as c_int
}

fn mode_has(mode: &[u8], needle: u8) -> bool {
    mode.contains(&needle)
}

unsafe fn fd_read(fd: c_int, buffer: *mut c_void, nbytes: usize) -> isize {
    libc::read(fd, buffer, nbytes)
}

unsafe fn fd_write(fd: c_int, buffer: *const c_void, nbytes: usize) -> isize {
    libc::write(fd, buffer, nbytes)
}

unsafe fn fd_seek(fd: c_int, offset: i64, whence: c_int) -> i64 {
    libc::lseek(fd, offset as libc::off_t, whence) as i64
}

unsafe fn fd_tell(fd: c_int) -> i64 {
    fd_seek(fd, 0, libc::SEEK_CUR)
}

fn mode2level(mode: &[u8]) -> c_int {
    if mode_has(mode, b'u') {
        return -2;
    }
    mode.iter()
        .copied()
        .find(u8::is_ascii_digit)
        .map(|b| (b - b'0') as c_int)
        .unwrap_or(-1)
}

unsafe fn bgzf_read_init(fd: c_int) -> *mut BGZF {
    let mut magic = [0u8; BLOCK_HEADER_LENGTH];
    let n = fd_read(fd, magic.as_mut_ptr().cast(), magic.len());
    if n < 0 {
        return ptr::null_mut();
    }
    if fd_seek(fd, 0, SEEK_SET) < 0 {
        return ptr::null_mut();
    }

    let fp = c_compat::calloc(1, std::mem::size_of::<BGZF>() as u64).cast::<BGZF>();
    if fp.is_null() {
        return ptr::null_mut();
    }

    let blocks = c_compat::malloc((2 * BGZF_MAX_BLOCK_SIZE) as u64);
    if blocks.is_null() {
        c_compat::free(fp.cast());
        return ptr::null_mut();
    }

    (*fp).uncompressed_block = blocks;
    (*fp).compressed_block = (blocks.cast::<u8>()).add(BGZF_MAX_BLOCK_SIZE).cast();
    set_flag(fp, 17, false);
    set_flag(
        fp,
        30,
        n == BLOCK_HEADER_LENGTH as isize && magic[0] == 0x1f && magic[1] == 0x8b,
    );
    set_flag(
        fp,
        31,
        flag(fp, 30) && !((magic[3] & 4) != 0 && &magic[12..16] == b"BC\x02\x00"),
    );
    (*fp).fp = fd_to_ptr(fd);
    fp
}

unsafe fn bgzf_write_init(fd: c_int, mode: &[u8]) -> *mut BGZF {
    let fp = c_compat::calloc(1, std::mem::size_of::<BGZF>() as u64).cast::<BGZF>();
    if fp.is_null() {
        return ptr::null_mut();
    }
    set_flag(fp, 17, true);
    (*fp).fp = fd_to_ptr(fd);

    let level = mode2level(mode);
    if level == -2 {
        set_flag(fp, 30, false);
        set_compress_level(fp, level);
        return fp;
    }

    let blocks = c_compat::malloc((2 * BGZF_MAX_BLOCK_SIZE) as u64);
    if blocks.is_null() {
        c_compat::free(fp.cast());
        return ptr::null_mut();
    }
    (*fp).uncompressed_block = blocks;
    (*fp).compressed_block = (blocks.cast::<u8>()).add(BGZF_MAX_BLOCK_SIZE).cast();
    set_flag(fp, 30, true);
    set_compress_level(fp, if level < 0 || level > 9 { 6 } else { level });
    fp
}

unsafe fn bgzf_free_without_hclose(fp: *mut BGZF) {
    if fp.is_null() {
        return;
    }
    c_compat::free((*fp).uncompressed_block);
    c_compat::free(fp.cast());
}

unsafe fn compress_level(fp: *const BGZF) -> c_int {
    let raw = (((*fp).bitfields >> 20) & 0x1ff) as i32;
    if raw & 0x100 != 0 {
        raw | !0x1ff
    } else {
        raw
    }
}

unsafe fn deflate_block(fp: *mut BGZF, block_length: usize) -> c_int {
    let input = slice::from_raw_parts((*fp).uncompressed_block.cast::<u8>(), block_length);
    let output =
        slice::from_raw_parts_mut((*fp).compressed_block.cast::<u8>(), BGZF_MAX_BLOCK_SIZE);
    output[..BLOCK_HEADER_LENGTH]
        .copy_from_slice(&[31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 0, 0]);

    let level = compress_level(fp).clamp(0, 9) as u32;
    let mut encoder = Compress::new(Compression::new(level), false);
    let status = encoder.compress(
        input,
        &mut output[BLOCK_HEADER_LENGTH..BGZF_MAX_BLOCK_SIZE - BLOCK_FOOTER_LENGTH],
        FlushCompress::Finish,
    );
    match status {
        Ok(Status::StreamEnd) => {}
        _ => {
            add_errcode(fp, BGZF_ERR_ZLIB);
            return -1;
        }
    }
    let compressed_len = encoder.total_out() as usize;
    let total_len = BLOCK_HEADER_LENGTH + compressed_len + BLOCK_FOOTER_LENGTH;
    if total_len > BGZF_MAX_BLOCK_SIZE {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    }
    let bsize = (total_len - 1) as u16;
    output[16..18].copy_from_slice(&bsize.to_le_bytes());
    let crc = crc32(input);
    output[BLOCK_HEADER_LENGTH + compressed_len..BLOCK_HEADER_LENGTH + compressed_len + 4]
        .copy_from_slice(&crc.to_le_bytes());
    output[BLOCK_HEADER_LENGTH + compressed_len + 4..total_len]
        .copy_from_slice(&(block_length as u32).to_le_bytes());
    (*fp).block_offset = 0;
    total_len as c_int
}

pub unsafe fn bgzf_flush(fp: *mut BGZF) -> c_int {
    if !flag(fp, 17) {
        return 0;
    }
    if !flag(fp, 30) {
        loop {
            let ret = libc::fsync(ptr_to_fd((*fp).fp));
            let errno = *c_compat::__errno_location();
            if ret < 0 && (errno == libc::EINVAL || errno == libc::ENOTSUP) {
                return 0;
            }
            if !(ret < 0 && errno == libc::EINTR) {
                return ret;
            }
        }
    }
    while (*fp).block_offset > 0 {
        if (*fp).idx_build_otf != 0 {
            bgzf_index_add_block(fp);
            (*(*fp).idx.cast::<bgzidx_t>()).ublock_addr += (*fp).block_offset as u64;
        }
        let block_length = deflate_block(fp, (*fp).block_offset as usize);
        if block_length < 0 {
            return -1;
        }
        let written = fd_write(
            ptr_to_fd((*fp).fp),
            (*fp).compressed_block,
            block_length as usize,
        );
        if written != block_length as isize {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        (*fp).block_address += block_length as i64;
    }
    0
}

unsafe fn bgzf_htell(fp: *mut BGZF) -> i64 {
    if !flag(fp, 30) {
        (*fp).block_address + (*fp).block_offset as i64
    } else {
        fd_tell(ptr_to_fd((*fp).fp))
    }
}

unsafe fn inflate_block(fp: *mut BGZF, block_length: usize) -> c_int {
    if block_length < BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH {
        add_errcode(fp, BGZF_ERR_HEADER);
        return -1;
    }
    let compressed = slice::from_raw_parts((*fp).compressed_block.cast::<u8>(), block_length);
    let payload_end = block_length - BLOCK_FOOTER_LENGTH;
    let expected_crc = unpack_u32(&compressed[payload_end..payload_end + 4]);
    let expected_len = unpack_u32(&compressed[payload_end + 4..payload_end + 8]) as usize;
    let mut written = BGZF_MAX_BLOCK_SIZE;
    let ret = bgzf_uncompress(
        (*fp).uncompressed_block.cast(),
        &mut written,
        (*fp).compressed_block.cast::<u8>().add(BLOCK_HEADER_LENGTH),
        block_length - BLOCK_HEADER_LENGTH,
        expected_crc,
    );
    if ret < 0 {
        add_errcode(
            fp,
            if ret == -2 {
                BGZF_ERR_CRC
            } else {
                BGZF_ERR_ZLIB
            },
        );
        return -1;
    }
    if written != expected_len {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    }
    written as c_int
}

unsafe fn bgzf_read_block(fp: *mut BGZF) -> c_int {
    if errcode(fp) != 0 {
        return -1;
    }
    let mut header = [0u8; BLOCK_HEADER_LENGTH];
    let mut block_address = bgzf_htell(fp);

    if !flag(fp, 30) {
        let count = fd_read(
            ptr_to_fd((*fp).fp),
            (*fp).uncompressed_block,
            BGZF_MAX_BLOCK_SIZE,
        );
        if count < 0 {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        if count == 0 {
            (*fp).block_length = 0;
            return 0;
        }
        if (*fp).block_length != 0 {
            (*fp).block_offset = 0;
        }
        (*fp).block_address = block_address;
        (*fp).block_length = count as c_int;
        return 0;
    }

    if flag(fp, 31) {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    }

    loop {
        let count = fd_read(
            ptr_to_fd((*fp).fp),
            header.as_mut_ptr().cast(),
            header.len(),
        );
        if count == 0 {
            if !flag(fp, 29) && !flag(fp, 18) {
                set_flag(fp, 18, true);
            }
            (*fp).block_length = 0;
            return 0;
        }
        let ret = check_header(header.as_ptr());
        if count != header.len() as isize || ret != 0 {
            add_errcode(fp, BGZF_ERR_HEADER);
            return -1;
        }
        let block_length = unpack_u16(&header[16..18]) as usize + 1;
        if !(BLOCK_HEADER_LENGTH..=BGZF_MAX_BLOCK_SIZE).contains(&block_length) {
            add_errcode(fp, BGZF_ERR_HEADER);
            return -1;
        }
        ptr::copy_nonoverlapping(
            header.as_ptr(),
            (*fp).compressed_block.cast::<u8>(),
            BLOCK_HEADER_LENGTH,
        );
        let remaining = block_length - BLOCK_HEADER_LENGTH;
        let read = fd_read(
            ptr_to_fd((*fp).fp),
            (*fp)
                .compressed_block
                .cast::<u8>()
                .add(BLOCK_HEADER_LENGTH)
                .cast(),
            remaining,
        );
        if read != remaining as isize {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        let count = inflate_block(fp, block_length);
        if count < 0 {
            return -1;
        }
        set_flag(fp, 29, count == 0);
        if count != 0 {
            if (*fp).block_length != 0 {
                (*fp).block_offset = 0;
            }
            (*fp).block_address = block_address;
            (*fp).block_clength = block_length as c_int;
            (*fp).block_length = count;
            return 0;
        }
        block_address = bgzf_htell(fp);
    }
}

pub unsafe fn bgzf_open(path: *const c_char, mode: *const c_char) -> *mut BGZF {
    if path.is_null() || mode.is_null() {
        *c_compat::__errno_location() = c_compat::EINVAL;
        return ptr::null_mut();
    }
    let mode_bytes = CStr::from_ptr(mode).to_bytes();
    if mode_has(mode_bytes, b'r') {
        let fd = libc::open(path, libc::O_RDONLY);
        if fd < 0 {
            return ptr::null_mut();
        }
        let fp = bgzf_read_init(fd);
        if fp.is_null() {
            libc::close(fd);
            return ptr::null_mut();
        }
        if flag(fp, 31) {
            bgzf_free_without_hclose(fp);
            libc::close(fd);
            *c_compat::__errno_location() = c_compat::ENOEXEC;
            return ptr::null_mut();
        }
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else if mode_has(mode_bytes, b'w') || mode_has(mode_bytes, b'a') {
        if mode_has(mode_bytes, b'g') {
            *c_compat::__errno_location() = c_compat::EINVAL;
            return ptr::null_mut();
        }
        let flags = if mode_has(mode_bytes, b'a') {
            libc::O_CREAT | libc::O_WRONLY | libc::O_APPEND
        } else {
            libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC
        };
        let fd = libc::open(path, flags, 0o666);
        if fd < 0 {
            return ptr::null_mut();
        }
        let fp = bgzf_write_init(fd, mode_bytes);
        if fp.is_null() {
            libc::close(fd);
            return ptr::null_mut();
        }
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else {
        *c_compat::__errno_location() = c_compat::EINVAL;
        ptr::null_mut()
    }
}

pub unsafe fn bgzf_dopen(fd: c_int, mode: *const c_char) -> *mut BGZF {
    if mode.is_null() {
        *c_compat::__errno_location() = c_compat::EINVAL;
        return ptr::null_mut();
    }
    let mode_bytes = CStr::from_ptr(mode).to_bytes();
    if mode_has(mode_bytes, b'r') {
        let fp = bgzf_read_init(fd);
        if fp.is_null() {
            return ptr::null_mut();
        }
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else if mode_has(mode_bytes, b'w') || mode_has(mode_bytes, b'a') {
        if mode_has(mode_bytes, b'g') {
            *c_compat::__errno_location() = c_compat::EINVAL;
            return ptr::null_mut();
        }
        let fp = bgzf_write_init(fd, mode_bytes);
        if fp.is_null() {
            return ptr::null_mut();
        }
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else {
        *c_compat::__errno_location() = c_compat::EINVAL;
        ptr::null_mut()
    }
}

pub unsafe fn bgzf_hopen(hfp: *mut super::hts::hFILE, mode: *const c_char) -> *mut BGZF {
    if hfp.is_null() || mode.is_null() {
        *c_compat::__errno_location() = c_compat::EINVAL;
        return ptr::null_mut();
    }
    let mode_bytes = CStr::from_ptr(mode).to_bytes();
    if mode_has(mode_bytes, b'r') {
        let fp = bgzf_read_init(ptr_to_fd(hfp));
        if fp.is_null() {
            return ptr::null_mut();
        }
        (*fp).fp = hfp;
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else if mode_has(mode_bytes, b'w') || mode_has(mode_bytes, b'a') {
        if mode_has(mode_bytes, b'g') {
            *c_compat::__errno_location() = c_compat::EINVAL;
            return ptr::null_mut();
        }
        let fp = bgzf_write_init(ptr_to_fd(hfp), mode_bytes);
        if fp.is_null() {
            return ptr::null_mut();
        }
        (*fp).fp = hfp;
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else {
        *c_compat::__errno_location() = c_compat::EINVAL;
        ptr::null_mut()
    }
}

pub unsafe fn bgzf_close(fp: *mut BGZF) -> c_int {
    if fp.is_null() {
        return -1;
    }
    if flag(fp, 17) && flag(fp, 30) {
        if bgzf_flush(fp) != 0 {
            let _ = libc::close(ptr_to_fd((*fp).fp));
            bgzf_free_without_hclose(fp);
            return -1;
        }
        let block_length = deflate_block(fp, 0);
        if block_length < 0
            || fd_write(
                ptr_to_fd((*fp).fp),
                (*fp).compressed_block,
                block_length as usize,
            ) != block_length as isize
        {
            let _ = libc::close(ptr_to_fd((*fp).fp));
            bgzf_free_without_hclose(fp);
            return -1;
        }
    }
    let had_error = errcode(fp) != 0;
    let ret = libc::close(ptr_to_fd((*fp).fp));
    c_compat::free((*fp).uncompressed_block);
    c_compat::free(fp.cast());
    if ret != 0 || had_error {
        -1
    } else {
        0
    }
}

pub unsafe fn bgzf_read(fp: *mut BGZF, data: *mut c_void, length: usize) -> isize {
    if length == 0 {
        return 0;
    }
    if fp.is_null() || flag(fp, 17) {
        return -1;
    }
    let mut bytes_read = 0usize;
    let mut output = data.cast::<u8>();
    while bytes_read < length {
        let mut available = (*fp).block_length - (*fp).block_offset;
        if available <= 0 {
            if bgzf_read_block(fp) != 0 {
                add_errcode(fp, BGZF_ERR_ZLIB);
                return -1;
            }
            available = (*fp).block_length - (*fp).block_offset;
            if available == 0 {
                if (*fp).block_length == 0 {
                    break;
                }
                (*fp).block_address = bgzf_htell(fp);
                (*fp).block_offset = 0;
                (*fp).block_length = 0;
                continue;
            }
            if available < 0 {
                add_errcode(fp, BGZF_ERR_MISUSE);
                return -1;
            }
        }
        let copy_length = (length - bytes_read).min(available as usize);
        ptr::copy_nonoverlapping(
            (*fp)
                .uncompressed_block
                .cast::<u8>()
                .add((*fp).block_offset as usize),
            output,
            copy_length,
        );
        (*fp).block_offset += copy_length as c_int;
        output = output.add(copy_length);
        bytes_read += copy_length;
        if (*fp).block_offset == (*fp).block_length {
            (*fp).block_address = bgzf_htell(fp);
            (*fp).block_offset = 0;
            (*fp).block_length = 0;
        }
    }
    (*fp).uncompressed_address += bytes_read as i64;
    bytes_read as isize
}

pub unsafe fn bgzf_read_small(fp: *mut BGZF, data: *mut c_void, length: usize) -> isize {
    if (length as isize) < ((*fp).block_length - (*fp).block_offset) as isize {
        ptr::copy_nonoverlapping(
            (*fp)
                .uncompressed_block
                .cast::<u8>()
                .add((*fp).block_offset as usize),
            data.cast::<u8>(),
            length,
        );
        (*fp).block_offset += length as c_int;
        (*fp).uncompressed_address += length as i64;
        length as isize
    } else {
        bgzf_read(fp, data, length)
    }
}

pub unsafe fn bgzf_raw_read(fp: *mut BGZF, data: *mut c_void, length: usize) -> isize {
    let ret = fd_read(ptr_to_fd((*fp).fp), data, length);
    if ret < 0 {
        add_errcode(fp, BGZF_ERR_IO);
    }
    ret
}

pub unsafe fn bgzf_write(fp: *mut BGZF, data: *const c_void, length: usize) -> isize {
    if fp.is_null() || !flag(fp, 17) {
        return -1;
    }
    if length == 0 {
        return 0;
    }
    if !flag(fp, 30) {
        let push = length + (*fp).block_offset as usize;
        (*fp).block_offset = (push % BGZF_MAX_BLOCK_SIZE) as c_int;
        (*fp).block_address += (push - (*fp).block_offset as usize) as i64;
        let written = fd_write(ptr_to_fd((*fp).fp), data, length);
        if written < 0 {
            add_errcode(fp, BGZF_ERR_IO);
        }
        return written;
    }
    let mut written = 0usize;
    while written < length {
        let available = BGZF_BLOCK_SIZE - (*fp).block_offset as usize;
        let copy_length = (length - written).min(available);
        ptr::copy_nonoverlapping(
            data.cast::<u8>().add(written),
            (*fp)
                .uncompressed_block
                .cast::<u8>()
                .add((*fp).block_offset as usize),
            copy_length,
        );
        (*fp).block_offset += copy_length as c_int;
        written += copy_length;
        if (*fp).block_offset as usize == BGZF_BLOCK_SIZE && bgzf_flush(fp) != 0 {
            return -1;
        }
    }
    written as isize
}

pub unsafe fn bgzf_raw_write(fp: *mut BGZF, data: *const c_void, length: usize) -> isize {
    let ret = fd_write(ptr_to_fd((*fp).fp), data, length);
    if ret < 0 {
        add_errcode(fp, BGZF_ERR_IO);
    }
    ret
}

pub unsafe fn bgzf_flush_try(fp: *mut BGZF, size: isize) -> c_int {
    if (*fp).block_offset as isize + size > BGZF_BLOCK_SIZE as isize {
        return bgzf_flush(fp);
    }
    0
}

pub unsafe fn bgzf_block_write(fp: *mut BGZF, data: *const c_void, length: usize) -> isize {
    if fp.is_null() || !flag(fp, 17) {
        return -1;
    }
    if length == 0 {
        return 0;
    }
    if !flag(fp, 30) {
        let push = length + (*fp).block_offset as usize;
        (*fp).block_offset = (push % BGZF_MAX_BLOCK_SIZE) as c_int;
        (*fp).block_address += (push - (*fp).block_offset as usize) as i64;
        let written = fd_write(ptr_to_fd((*fp).fp), data, length);
        if written < 0 {
            add_errcode(fp, BGZF_ERR_IO);
        }
        return written;
    }

    let input = data.cast::<u8>();
    let mut remaining = length;
    let mut consumed = 0usize;
    while remaining > 0 {
        let idx = (*fp).idx.cast::<bgzidx_t>();
        let current_block = (*idx).moffs - (*idx).noffs;
        let ublock_size = if current_block + 1 < (*idx).moffs {
            let cur = (*idx).offs.add(current_block as usize);
            let next = (*idx).offs.add(current_block as usize + 1);
            (*next).uaddr - (*cur).uaddr
        } else {
            BGZF_MAX_BLOCK_SIZE as u64
        };
        let mut copy_length = (ublock_size as c_int - (*fp).block_offset) as usize;
        if copy_length > remaining {
            copy_length = remaining;
        }
        ptr::copy_nonoverlapping(
            input.add(consumed),
            (*fp)
                .uncompressed_block
                .cast::<u8>()
                .add((*fp).block_offset as usize),
            copy_length,
        );
        (*fp).block_offset += copy_length as c_int;
        consumed += copy_length;
        remaining -= copy_length;
        if (*fp).block_offset as u64 == ublock_size {
            if bgzf_flush(fp) != 0 {
                return -1;
            }
            if (*idx).noffs > 0 {
                (*idx).noffs -= 1;
            }
        }
    }
    consumed as isize
}

pub unsafe fn bgzf_set_cache_size(fp: *mut BGZF, cache_size: c_int) {
    if !fp.is_null() && !(*fp).mt.is_null() {
        return;
    }
    if !fp.is_null() && !(*fp).cache.is_null() {
        (*fp).cache_size = cache_size;
    }
}

pub unsafe fn bgzf_write_small(fp: *mut BGZF, data: *const c_void, length: usize) -> isize {
    if flag(fp, 30) && BGZF_BLOCK_SIZE - (*fp).block_offset as usize > length {
        ptr::copy_nonoverlapping(
            data.cast::<u8>(),
            (*fp)
                .uncompressed_block
                .cast::<u8>()
                .add((*fp).block_offset as usize),
            length,
        );
        (*fp).block_offset += length as c_int;
        length as isize
    } else {
        bgzf_write(fp, data, length)
    }
}

pub unsafe fn bgzf_seek(fp: *mut BGZF, pos: i64, whence: c_int) -> i64 {
    if fp.is_null() || flag(fp, 17) || whence != SEEK_SET || flag(fp, 31) {
        if !fp.is_null() {
            add_errcode(fp, BGZF_ERR_MISUSE);
        }
        return -1;
    }
    (*fp).seeked = pos;
    if fd_seek(ptr_to_fd((*fp).fp), pos >> 16, SEEK_SET) < 0 {
        add_errcode(fp, BGZF_ERR_IO);
        return -1;
    }
    (*fp).block_length = 0;
    (*fp).block_address = pos >> 16;
    (*fp).block_offset = (pos & 0xffff) as c_int;
    0
}

pub unsafe fn bgzf_useek(fp: *mut BGZF, uoffset: i64, where_: c_int) -> c_int {
    if fp.is_null() || flag(fp, 17) || where_ != SEEK_SET || flag(fp, 31) {
        if !fp.is_null() {
            add_errcode(fp, BGZF_ERR_MISUSE);
        }
        return -1;
    }
    if uoffset >= (*fp).uncompressed_address - (*fp).block_offset as i64
        && uoffset
            < (*fp).uncompressed_address + (*fp).block_length as i64 - (*fp).block_offset as i64
    {
        (*fp).block_offset += (uoffset - (*fp).uncompressed_address) as c_int;
        (*fp).uncompressed_address = uoffset;
        return 0;
    }
    if !flag(fp, 30) {
        if fd_seek(ptr_to_fd((*fp).fp), uoffset, SEEK_SET) < 0 {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        (*fp).block_length = 0;
        (*fp).block_address = uoffset;
        (*fp).block_offset = 0;
        if bgzf_read_block(fp) < 0 {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        (*fp).uncompressed_address = uoffset;
        return 0;
    }
    add_errcode(fp, BGZF_ERR_IO);
    -1
}

pub unsafe fn bgzf_check_EOF(fp: *mut BGZF) -> c_int {
    if fp.is_null() || !flag(fp, 30) || flag(fp, 31) {
        return 0;
    }
    let fd = ptr_to_fd((*fp).fp);
    let offset = fd_tell(fd);
    if offset < EOF_BLOCK.len() as i64 {
        return 0;
    }
    let mut buf = [0u8; 28];
    if fd_seek(fd, -(EOF_BLOCK.len() as i64), libc::SEEK_END) < 0 {
        return 0;
    }
    let n = fd_read(fd, buf.as_mut_ptr().cast(), buf.len());
    let _ = fd_seek(fd, offset, SEEK_SET);
    if n == buf.len() as isize && buf == EOF_BLOCK {
        1
    } else {
        0
    }
}

pub unsafe fn bgzf_is_bgzf(fn_: *const c_char) -> c_int {
    let fd = libc::open(fn_, libc::O_RDONLY);
    if fd < 0 {
        return 0;
    }
    let mut buf = [0u8; 16];
    let n = fd_read(fd, buf.as_mut_ptr().cast(), buf.len());
    if libc::close(fd) < 0 {
        return 0;
    }
    if n != 16 {
        return 0;
    }
    if check_header(buf.as_ptr()) == 0 {
        1
    } else {
        0
    }
}

pub unsafe fn bgzf_compression(fp: *mut BGZF) -> c_int {
    if !flag(fp, 30) {
        0
    } else if flag(fp, 31) {
        1
    } else {
        2
    }
}

pub unsafe fn bgzf_hfile(fp: *mut BGZF) -> *mut super::hts::hFILE {
    (*fp).fp
}

pub unsafe fn bgzf_utell(fp: *mut BGZF) -> i64 {
    (*fp).uncompressed_address
}

pub unsafe fn bgzf_getc(fp: *mut BGZF) -> c_int {
    if (*fp).block_offset + 1 < (*fp).block_length {
        (*fp).uncompressed_address += 1;
        let c = *(*fp)
            .uncompressed_block
            .cast::<u8>()
            .add((*fp).block_offset as usize);
        (*fp).block_offset += 1;
        return c as c_int;
    }

    if (*fp).block_offset >= (*fp).block_length {
        if bgzf_read_block(fp) != 0 {
            return -2;
        }
        if (*fp).block_length == 0 {
            return -1;
        }
    }
    let c = *(*fp)
        .uncompressed_block
        .cast::<u8>()
        .add((*fp).block_offset as usize);
    (*fp).block_offset += 1;
    if (*fp).block_offset == (*fp).block_length {
        (*fp).block_address = bgzf_htell(fp);
        (*fp).block_offset = 0;
        (*fp).block_length = 0;
    }
    (*fp).uncompressed_address += 1;
    c as c_int
}

pub unsafe fn bgzf_getline(fp: *mut BGZF, delim: c_int, str_: *mut kstring_t) -> c_int {
    let mut state = 0;
    (*str_).l = 0;

    loop {
        if (*fp).block_offset >= (*fp).block_length {
            if bgzf_read_block(fp) != 0 {
                state = -2;
                break;
            }
            if (*fp).block_length == 0 {
                state = -1;
                break;
            }
        }

        let buf = (*fp).uncompressed_block.cast::<u8>();
        let found = libc::memchr(
            buf.add((*fp).block_offset as usize).cast(),
            delim,
            ((*fp).block_length - (*fp).block_offset) as usize,
        );
        let mut l = if found.is_null() {
            (*fp).block_length
        } else {
            found.cast::<u8>().offset_from(buf) as c_int
        };

        if l < (*fp).block_length {
            state = 1;
        }
        l -= (*fp).block_offset;
        if ks_expand(str_, l as usize + 2) < 0 {
            state = -3;
            break;
        }
        ptr::copy_nonoverlapping(
            buf.add((*fp).block_offset as usize),
            (*str_).s.add((*str_).l).cast::<u8>(),
            l as usize,
        );
        (*str_).l += l as usize;
        (*fp).block_offset += l + 1;
        if (*fp).block_offset >= (*fp).block_length {
            (*fp).block_address = bgzf_htell(fp);
            (*fp).block_offset = 0;
            (*fp).block_length = 0;
        }
        if state != 0 {
            break;
        }
    }

    if state < -1 {
        return state;
    }
    if (*str_).l == 0 && state < 0 {
        return state;
    }
    (*fp).uncompressed_address += (*str_).l as i64 + 1;
    if delim == b'\n' as c_int && (*str_).l > 0 && *(*str_).s.add((*str_).l - 1) == b'\r' as c_char
    {
        (*str_).l -= 1;
    }
    *(*str_).s.add((*str_).l) = 0;
    if (*str_).l <= c_int::MAX as usize {
        (*str_).l as c_int
    } else {
        c_int::MAX
    }
}

pub unsafe fn bgzf_thread_pool(
    fp: *mut BGZF,
    pool: *mut hts_sys::hts_tpool,
    qsize: c_int,
) -> c_int {
    hts_sys::bgzf_thread_pool(fp.cast(), pool, qsize)
}

pub unsafe fn bgzf_mt(fp: *mut BGZF, n_threads: c_int, n_sub_blks: c_int) -> c_int {
    hts_sys::bgzf_mt(fp.cast(), n_threads, n_sub_blks)
}

pub unsafe fn bgzf_index_destroy(fp: *mut BGZF) {
    if (*fp).idx.is_null() {
        return;
    }
    let idx = (*fp).idx.cast::<bgzidx_t>();
    c_compat::free((*idx).offs.cast());
    c_compat::free(idx.cast());
    (*fp).idx = ptr::null_mut();
    (*fp).idx_build_otf = 0;
}

pub unsafe fn bgzf_index_build_init(fp: *mut BGZF) -> c_int {
    bgzf_index_destroy(fp);
    (*fp).idx = c_compat::calloc(1, std::mem::size_of::<bgzidx_t>() as u64);
    if (*fp).idx.is_null() {
        return -1;
    }
    (*fp).idx_build_otf = 1;
    0
}

pub unsafe fn bgzf_index_add_block(fp: *mut BGZF) -> c_int {
    let idx = (*fp).idx.cast::<bgzidx_t>();
    (*idx).noffs += 1;
    if (*idx).noffs > (*idx).moffs {
        (*idx).moffs = (*idx).noffs;
        let mut rounded = (*idx).moffs as u32;
        rounded = rounded.wrapping_sub(1);
        rounded |= rounded >> 1;
        rounded |= rounded >> 2;
        rounded |= rounded >> 4;
        rounded |= rounded >> 8;
        rounded |= rounded >> 16;
        rounded = rounded.wrapping_add(1);
        (*idx).moffs = rounded as c_int;
        let tmp = c_compat::realloc(
            (*idx).offs.cast(),
            ((*idx).moffs as usize * std::mem::size_of::<bgzidx1_t>()) as u64,
        )
        .cast::<bgzidx1_t>();
        if tmp.is_null() {
            return -1;
        }
        (*idx).offs = tmp;
    }
    let slot = (*idx).offs.add((*idx).noffs as usize - 1);
    (*slot).uaddr = (*idx).ublock_addr;
    (*slot).caddr = (*fp).block_address as u64;
    0
}

pub unsafe fn bgzf_index_dump_hfile(
    fp: *mut BGZF,
    idx_file: *mut super::hts::hFILE,
    _name: *const c_char,
) -> c_int {
    if (*fp).idx.is_null() {
        *c_compat::__errno_location() = c_compat::EINVAL;
        return -1;
    }
    if bgzf_flush(fp) != 0 {
        return -1;
    }

    let idx = (*fp).idx.cast::<bgzidx_t>();
    if !(*fp).mt.is_null() && (*idx).noffs > 0 {
        (*idx).noffs -= 1;
    }
    if hwrite_uint64(
        if (*idx).noffs > 0 {
            ((*idx).noffs - 1) as u64
        } else {
            0
        },
        idx_file,
    ) < 0
    {
        return -1;
    }
    let mut i = 1;
    while i < (*idx).noffs {
        let off = (*idx).offs.add(i as usize);
        if hwrite_uint64((*off).caddr, idx_file) < 0 {
            return -1;
        }
        if hwrite_uint64((*off).uaddr, idx_file) < 0 {
            return -1;
        }
        i += 1;
    }
    0
}

pub unsafe fn bgzf_index_dump(fp: *mut BGZF, bname: *const c_char, suffix: *const c_char) -> c_int {
    if (*fp).idx.is_null() {
        *c_compat::__errno_location() = c_compat::EINVAL;
        return -1;
    }

    let mut tmp = ptr::null_mut();
    let name = if !suffix.is_null() {
        tmp = get_name_suffix(bname, suffix);
        if tmp.is_null() {
            return -1;
        }
        tmp
    } else {
        bname.cast_mut()
    };

    let fd = libc::open(name, libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC, 0o666);
    if fd < 0 {
        c_compat::free(tmp.cast());
        return -1;
    }

    let ret = bgzf_index_dump_hfile(fp, fd_to_ptr(fd), name);
    if libc::close(fd) < 0 || ret != 0 {
        c_compat::free(tmp.cast());
        return -1;
    }
    c_compat::free(tmp.cast());
    0
}

pub unsafe fn bgzf_index_load_hfile(
    fp: *mut BGZF,
    idx_file: *mut super::hts::hFILE,
    _name: *const c_char,
) -> c_int {
    (*fp).idx = c_compat::calloc(1, std::mem::size_of::<bgzidx_t>() as u64);
    if (*fp).idx.is_null() {
        return -1;
    }

    let idx = (*fp).idx.cast::<bgzidx_t>();
    let mut x = 0u64;
    if hread_uint64(&mut x, idx_file) < 0 {
        bgzf_index_destroy(fp);
        return -1;
    }
    if x >= (usize::MAX as u64 / std::mem::size_of::<bgzidx1_t>() as u64 / 2) {
        bgzf_index_destroy(fp);
        return -1;
    }

    (*idx).noffs = x.wrapping_add(1) as c_int;
    (*idx).moffs = (*idx).noffs;
    (*idx).offs =
        c_compat::malloc(((*idx).moffs as usize * std::mem::size_of::<bgzidx1_t>()) as u64)
            .cast::<bgzidx1_t>();
    if (*idx).offs.is_null() {
        bgzf_index_destroy(fp);
        return -1;
    }
    (*(*idx).offs.add(0)).caddr = 0;
    (*(*idx).offs.add(0)).uaddr = 0;

    let mut i = 1;
    while i < (*idx).noffs {
        let off = (*idx).offs.add(i as usize);
        if hread_uint64(&mut (*off).caddr, idx_file) < 0 {
            bgzf_index_destroy(fp);
            return -1;
        }
        if hread_uint64(&mut (*off).uaddr, idx_file) < 0 {
            bgzf_index_destroy(fp);
            return -1;
        }
        i += 1;
    }

    0
}

pub unsafe fn bgzf_index_load(fp: *mut BGZF, bname: *const c_char, suffix: *const c_char) -> c_int {
    let mut tmp = ptr::null_mut();
    let name = if !suffix.is_null() {
        tmp = get_name_suffix(bname, suffix);
        if tmp.is_null() {
            return -1;
        }
        tmp
    } else {
        bname.cast_mut()
    };

    let fd = libc::open(name, libc::O_RDONLY);
    if fd < 0 {
        c_compat::free(tmp.cast());
        return -1;
    }

    let ret = bgzf_index_load_hfile(fp, fd_to_ptr(fd), name);
    if libc::close(fd) != 0 || ret != 0 {
        c_compat::free(tmp.cast());
        return -1;
    }
    c_compat::free(tmp.cast());
    0
}

pub unsafe fn bgzf_peek(fp: *mut BGZF) -> c_int {
    if fp.is_null() {
        return -2;
    }
    let mut available = (*fp).block_length - (*fp).block_offset;
    if available <= 0 && bgzf_read_block(fp) < 0 {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -2;
    }
    available = (*fp).block_length - (*fp).block_offset;
    if available != 0 {
        *(*fp)
            .uncompressed_block
            .cast::<u8>()
            .add((*fp).block_offset as usize) as c_int
    } else {
        -1
    }
}

pub unsafe fn hwrite_uint64(mut x: u64, f: *mut super::hts::hFILE) -> c_int {
    if ed_is_big() != 0 {
        x = ed_swap_8(x);
    }
    if fd_write(
        ptr_to_fd(f),
        (&x as *const u64).cast(),
        std::mem::size_of::<u64>(),
    ) != std::mem::size_of::<u64>() as isize
    {
        return -1;
    }
    0
}

pub unsafe fn hread_uint64(xptr: *mut u64, f: *mut super::hts::hFILE) -> c_int {
    if fd_read(
        ptr_to_fd(f),
        xptr.cast::<c_void>(),
        std::mem::size_of::<u64>(),
    ) != std::mem::size_of::<u64>() as isize
    {
        return -1;
    }
    if ed_is_big() != 0 {
        *xptr = ed_swap_8(*xptr);
    }
    0
}

pub unsafe fn get_name_suffix(bname: *const c_char, suffix: *const c_char) -> *mut c_char {
    let len = libc::strlen(bname) + libc::strlen(suffix) + 1;
    let buff = c_compat::malloc(len as u64).cast::<c_char>();
    if buff.is_null() {
        return ptr::null_mut();
    }
    libc::snprintf(buff, len, c"%s%s".as_ptr(), bname, suffix);
    buff
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::CString,
        fs::File,
        io::{Read, Write},
    };

    #[test]
    fn translated_bgzf_reads_and_seeks_noodles_bgzf_file() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-{}-{}.gz",
            std::process::id(),
            "read_seek"
        ));
        let payload = b"abcdef0123456789".repeat(8192);
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(&payload).unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());

            let mut buf = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()),
                buf.len() as isize
            );
            assert_eq!(buf, payload);
            assert_eq!(bgzf_check_EOF(fp), 1);

            assert_eq!(bgzf_seek(fp, 0, 0), 0);
            assert_eq!(bgzf_peek(fp), b'a' as c_int);

            let mut small = [0u8; 6];
            assert_eq!(bgzf_read(fp, small.as_mut_ptr().cast(), small.len()), 6);
            assert_eq!(&small, b"abcdef");
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn translated_bgzf_writes_readable_bgzf_file() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-{}-{}.gz",
            std::process::id(),
            "write"
        ));
        let payload = b"translated-bgzf-write\n".repeat(7000);
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"w\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(
                bgzf_write(fp, payload.as_ptr().cast(), payload.len()),
                payload.len() as isize
            );
            assert_eq!(bgzf_close(fp), 0);
        }

        let file = File::open(&path).unwrap();
        let mut reader = noodles::bgzf::io::Reader::new(file);
        let mut decoded = Vec::new();
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, payload);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_dopen_and_hopen_read_fd_backed_bgzf_streams() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-open-wrappers-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let payload = b"fd-backed bgzf payload";
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(payload).unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let fd = libc::open(path_c.as_ptr(), libc::O_RDONLY);
            assert!(fd >= 0);
            let fp = bgzf_dopen(fd, c"r".as_ptr());
            assert!(!fp.is_null());
            let mut buf = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()),
                payload.len() as isize
            );
            assert_eq!(buf, payload);
            assert_eq!(bgzf_close(fp), 0);

            let fd = libc::open(path_c.as_ptr(), libc::O_RDONLY);
            assert!(fd >= 0);
            let hfp = fd_to_ptr(fd);
            let fp = bgzf_hopen(hfp, c"r".as_ptr());
            assert!(!fp.is_null());
            let mut buf = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()),
                payload.len() as isize
            );
            assert_eq!(buf, payload);
            assert_eq!(bgzf_hfile(fp), hfp);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn crc32_matches_known_vector() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    }

    #[test]
    fn hts_crc32_matches_zlib_incremental_crc_rules() {
        unsafe {
            assert_eq!(hts_crc32(0, b"123456789".as_ptr().cast(), 9), 0xcbf4_3926);
            let first = hts_crc32(0, b"1234".as_ptr().cast(), 4);
            assert_eq!(hts_crc32(first, b"56789".as_ptr().cast(), 5), 0xcbf4_3926);
        }
    }

    #[test]
    fn bgzf_compress_writes_bgzf_block_header_payload_and_footer() {
        unsafe {
            let payload = b"literal-bgzf-payload";
            let mut out = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut dlen = out.len();
            assert_eq!(
                bgzf_compress(
                    out.as_mut_ptr().cast(),
                    &mut dlen,
                    payload.as_ptr().cast(),
                    payload.len(),
                    0,
                ),
                0
            );
            assert_eq!(
                dlen,
                BLOCK_HEADER_LENGTH + 5 + payload.len() + BLOCK_FOOTER_LENGTH
            );
            assert_eq!(check_header(out.as_ptr()), 0);
            assert_eq!(unpackInt16(out.as_ptr().add(16)), dlen as c_int - 1);
            assert_eq!(out[BLOCK_HEADER_LENGTH], 1);
            assert_eq!(
                unpackInt16(out.as_ptr().add(BLOCK_HEADER_LENGTH + 1)),
                payload.len() as c_int
            );
            assert_eq!(
                unpackInt16(out.as_ptr().add(BLOCK_HEADER_LENGTH + 3)),
                (!(payload.len() as u16)) as c_int
            );
            assert_eq!(
                &out[BLOCK_HEADER_LENGTH + 5..BLOCK_HEADER_LENGTH + 5 + payload.len()],
                payload
            );
            assert_eq!(
                unpack_u32(&out[dlen - 8..dlen - 4]),
                hts_crc32(0, payload.as_ptr().cast(), payload.len())
            );
            assert_eq!(unpack_u32(&out[dlen - 4..dlen]), payload.len() as u32);

            let compressed_payload = b"compressible payload ".repeat(40);
            let mut compressed = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut compressed_len = compressed.len();
            assert_eq!(
                bgzf_compress(
                    compressed.as_mut_ptr().cast(),
                    &mut compressed_len,
                    compressed_payload.as_ptr().cast(),
                    compressed_payload.len(),
                    6,
                ),
                0
            );
            assert_eq!(check_header(compressed.as_ptr()), 0);
            let payload_end = compressed_len - BLOCK_FOOTER_LENGTH;
            let mut decoded = vec![0u8; compressed_payload.len()];
            let mut decoder = Decompress::new(false);
            assert!(matches!(
                decoder.decompress(
                    &compressed[BLOCK_HEADER_LENGTH..payload_end],
                    &mut decoded,
                    FlushDecompress::Finish,
                ),
                Ok(Status::StreamEnd)
            ));
            assert_eq!(decoded, compressed_payload);
        }
    }

    #[test]
    fn bgzf_uncompress_inflates_raw_payload_and_checks_crc() {
        unsafe {
            let payload = b"translated bgzf uncompress payload ".repeat(20);
            let mut block = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut block_len = block.len();
            assert_eq!(
                bgzf_compress(
                    block.as_mut_ptr().cast(),
                    &mut block_len,
                    payload.as_ptr().cast(),
                    payload.len(),
                    6,
                ),
                0
            );
            let expected_crc = unpack_u32(&block[block_len - 8..block_len - 4]);
            let mut decoded = vec![0u8; BGZF_MAX_BLOCK_SIZE];
            let mut decoded_len = decoded.len();
            assert_eq!(
                bgzf_uncompress(
                    decoded.as_mut_ptr(),
                    &mut decoded_len,
                    block.as_ptr().add(BLOCK_HEADER_LENGTH),
                    block_len - BLOCK_HEADER_LENGTH,
                    expected_crc,
                ),
                0
            );
            assert_eq!(decoded_len, payload.len());
            assert_eq!(&decoded[..decoded_len], payload);

            decoded_len = decoded.len();
            assert_eq!(
                bgzf_uncompress(
                    decoded.as_mut_ptr(),
                    &mut decoded_len,
                    block.as_ptr().add(BLOCK_HEADER_LENGTH),
                    block_len - BLOCK_HEADER_LENGTH,
                    expected_crc ^ 1,
                ),
                -2
            );
        }
    }

    #[test]
    fn bgzf_pack_unpack_helpers_match_little_endian_layout() {
        unsafe {
            let mut buf = [0u8; 4];
            packInt16(buf.as_mut_ptr(), 0xabcd);
            assert_eq!(buf[..2], [0xcd, 0xab]);
            assert_eq!(unpackInt16(buf.as_ptr()), 0xabcd);
            packInt32(buf.as_mut_ptr(), 0x1234_abcd);
            assert_eq!(buf, [0xcd, 0xab, 0x34, 0x12]);
        }
    }

    #[test]
    fn bgzf_leaf_helpers_match_c_rules() {
        unsafe {
            let header = [
                31u8, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, b'B', b'C', 2, 0, 27, 0,
            ];
            assert_eq!(check_header(header.as_ptr()), 0);
            let mut no_extra = header;
            no_extra[3] = 0;
            assert_eq!(check_header(no_extra.as_ptr()), -1);
            let mut not_gzip = header;
            not_gzip[0] = 0;
            assert_eq!(check_header(not_gzip.as_ptr()), -2);

            let msg = bgzf_zerr(Z_MEM_ERROR, ptr::null_mut());
            assert_eq!(CStr::from_ptr(msg).to_bytes(), b"out of memory");
            let mut zs = z_stream_s {
                next_in: ptr::null(),
                avail_in: 0,
                total_in: 0,
                next_out: ptr::null_mut(),
                avail_out: 0,
                total_out: 0,
                msg: c"z-stream-message".as_ptr().cast_mut().cast(),
                state: ptr::null_mut(),
                zalloc: ptr::null_mut(),
                zfree: ptr::null_mut(),
                opaque: ptr::null_mut(),
                data_type: 0,
                adler: 0,
                reserved: 0,
            };
            assert_eq!(bgzf_zerr(Z_DATA_ERROR, &mut zs), zs.msg.cast());

            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: 0x123usize as *mut _,
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };
            assert_eq!(bgzf_compression(&mut fp), 0);
            set_flag(&mut fp, 30, true);
            assert_eq!(bgzf_compression(&mut fp), 2);
            set_flag(&mut fp, 31, true);
            assert_eq!(bgzf_compression(&mut fp), 1);
            assert_eq!(bgzf_hfile(&mut fp), 0x123usize as *mut _);

            let mut cache_marker = 0u8;
            fp.mt = ptr::null_mut();
            fp.cache = (&mut cache_marker as *mut u8).cast();
            bgzf_set_cache_size(&mut fp, 8192);
            assert_eq!(fp.cache_size, 8192);
            fp.mt = 1usize as *mut c_void;
            bgzf_set_cache_size(&mut fp, 16384);
            assert_eq!(fp.cache_size, 8192);

            let suffix = get_name_suffix(c"base".as_ptr(), c".gzi".as_ptr());
            assert!(!suffix.is_null());
            assert_eq!(CStr::from_ptr(suffix).to_bytes(), b"base.gzi");
            c_compat::free(suffix.cast());
        }
    }

    #[test]
    fn bgzf_raw_byte_and_uint64_helpers_match_c_rules() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-raw-{}-{}",
            std::process::id(),
            "io"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"wu\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(bgzf_raw_write(fp, b"raw".as_ptr().cast(), 3), 3);
            assert_eq!(bgzf_flush_try(fp, BGZF_BLOCK_SIZE as isize), 0);
            assert_eq!(bgzf_close(fp), 0);

            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            let mut buf = [0u8; 3];
            assert_eq!(bgzf_raw_read(fp, buf.as_mut_ptr().cast(), buf.len()), 3);
            assert_eq!(&buf, b"raw");
            assert_eq!(bgzf_close(fp), 0);

            let fd = libc::open(path_c.as_ptr(), libc::O_RDWR | libc::O_TRUNC);
            assert!(fd >= 0);
            let hfile = fd_to_ptr(fd);
            assert_eq!(hwrite_uint64(0x0123_4567_89ab_cdef, hfile), 0);
            assert_eq!(libc::lseek(fd, 0, libc::SEEK_SET), 0);
            let mut x = 0;
            assert_eq!(hread_uint64(&mut x, hfile), 0);
            assert_eq!(x, 0x0123_4567_89ab_cdef);
            assert_eq!(libc::close(fd), 0);

            let fp = bgzf_open(path_c.as_ptr(), b"w\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(bgzf_index_build_init(fp), 0);
            assert_eq!(bgzf_block_write(fp, b"block".as_ptr().cast(), 5), 5);
            assert_eq!(bgzf_close(fp), 0);
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_getc_utell_and_probe_match_c_rules() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-getc-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let plain = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-getc-{}-{}.txt",
            std::process::id(),
            "plain"
        ));
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(b"abcdef").unwrap();
            writer.finish().unwrap();
        }
        std::fs::write(&plain, b"plain").unwrap();

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        let plain_c = CString::new(super::super::path_bytes(&plain).as_ref()).unwrap();
        unsafe {
            assert_eq!(bgzf_is_bgzf(path_c.as_ptr()), 1);
            assert_eq!(bgzf_is_bgzf(plain_c.as_ptr()), 0);

            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(bgzf_utell(fp), 0);
            assert_eq!(bgzf_getc(fp), b'a' as c_int);
            assert_eq!(bgzf_getc(fp), b'b' as c_int);
            assert_eq!(bgzf_utell(fp), 2);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(plain);
    }

    #[test]
    fn bgzf_getline_reads_lines_and_trims_cr_before_newline() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-getline-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(b"alpha\r\nbeta\nlast").unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: ptr::null_mut(),
            };

            assert_eq!(bgzf_getline(fp, b'\n' as c_int, &mut line), 5);
            assert_eq!(CStr::from_ptr(line.s).to_bytes(), b"alpha");
            assert_eq!(bgzf_getline(fp, b'\n' as c_int, &mut line), 4);
            assert_eq!(CStr::from_ptr(line.s).to_bytes(), b"beta");
            assert_eq!(bgzf_getline(fp, b'\n' as c_int, &mut line), 4);
            assert_eq!(CStr::from_ptr(line.s).to_bytes(), b"last");
            assert_eq!(bgzf_getline(fp, b'\n' as c_int, &mut line), -1);

            c_compat::free(line.s.cast());
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_index_build_add_and_destroy_match_c_state_transitions() {
        unsafe {
            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 123,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 99,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };

            assert_eq!(bgzf_index_build_init(&mut fp), 0);
            assert!(!fp.idx.is_null());
            assert_eq!(fp.idx_build_otf, 1);

            let idx = fp.idx.cast::<bgzidx_t>();
            (*idx).ublock_addr = 42;
            assert_eq!(bgzf_index_add_block(&mut fp), 0);
            assert_eq!((*idx).noffs, 1);
            assert!((*idx).moffs >= 1);
            assert!(!(*idx).offs.is_null());
            assert_eq!((*(*idx).offs.add(0)).uaddr, 42);
            assert_eq!((*(*idx).offs.add(0)).caddr, 123);

            fp.block_address = 456;
            (*idx).ublock_addr = 1000;
            assert_eq!(bgzf_index_add_block(&mut fp), 0);
            assert_eq!((*idx).noffs, 2);
            assert!((*idx).moffs >= 2);
            assert_eq!((*(*idx).offs.add(1)).uaddr, 1000);
            assert_eq!((*(*idx).offs.add(1)).caddr, 456);

            bgzf_index_destroy(&mut fp);
            assert!(fp.idx.is_null());
            assert_eq!(fp.idx_build_otf, 0);
        }
    }

    #[test]
    fn bgzf_index_dump_and_load_round_trip_gzi_layout() {
        let base = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-index-{}-{}",
            std::process::id(),
            "roundtrip"
        ));
        let index_path = base.with_extension("gzi");
        let base_c = CString::new(super::super::path_bytes(&base).as_ref()).unwrap();

        unsafe {
            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: ptr::null_mut(),
                idx: c_compat::calloc(1, std::mem::size_of::<bgzidx_t>() as u64),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };
            assert!(!fp.idx.is_null());
            let idx = fp.idx.cast::<bgzidx_t>();
            (*idx).noffs = 3;
            (*idx).moffs = 3;
            (*idx).offs =
                c_compat::calloc(3, std::mem::size_of::<bgzidx1_t>() as u64).cast::<bgzidx1_t>();
            assert!(!(*idx).offs.is_null());
            (*(*idx).offs.add(1)).caddr = 11;
            (*(*idx).offs.add(1)).uaddr = 101;
            (*(*idx).offs.add(2)).caddr = 22;
            (*(*idx).offs.add(2)).uaddr = 202;

            assert_eq!(
                bgzf_index_dump(&mut fp, base_c.as_ptr(), c".gzi".as_ptr()),
                0
            );

            let mut loaded = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };

            assert_eq!(
                bgzf_index_load(&mut loaded, base_c.as_ptr(), c".gzi".as_ptr()),
                0
            );
            let loaded_idx = loaded.idx.cast::<bgzidx_t>();
            assert_eq!((*loaded_idx).noffs, 3);
            assert_eq!((*loaded_idx).moffs, 3);
            assert_eq!((*(*loaded_idx).offs.add(0)).caddr, 0);
            assert_eq!((*(*loaded_idx).offs.add(0)).uaddr, 0);
            assert_eq!((*(*loaded_idx).offs.add(1)).caddr, 11);
            assert_eq!((*(*loaded_idx).offs.add(1)).uaddr, 101);
            assert_eq!((*(*loaded_idx).offs.add(2)).caddr, 22);
            assert_eq!((*(*loaded_idx).offs.add(2)).uaddr, 202);

            bgzf_index_destroy(&mut fp);
            bgzf_index_destroy(&mut loaded);
        }

        let _ = std::fs::remove_file(index_path);
    }
}
