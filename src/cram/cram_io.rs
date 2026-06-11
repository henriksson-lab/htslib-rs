// Functions translated from htslib/cram/cram_io.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int, c_uint, c_void, CStr};

use super::*;

unsafe fn c_char_bytes<'a>(ptr: *const c_char) -> Option<&'a [u8]> {
    if ptr.is_null() {
        return None;
    }

    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    Some(std::slice::from_raw_parts(ptr.cast::<u8>(), len))
}

/// original: cram_flush (htslib/cram/cram_io.c:5446)
///
/// Flushes a CRAM file. Useful for when writing to stdout without wishing to
/// close the stream. Byte-faithful 1:1 translation.
///
/// MT-pool deviation: when fd->pool is set, the bridge's
/// `cram_cram_io_c_4275_cram_flush_container_mt` runs the flush
/// single-threaded (the C cram_flush_thread / cram_flush_result / reset_metrics
/// helpers are not yet translated). On-disk output is bit-identical to the
/// MT path; only throughput is lost for that one flush.
pub unsafe fn cram_cram_io_c_5446_cram_flush(fd: *mut cram_fd) -> c_int {
    if fd.is_null() {
        return -1;
    }
    let mut ret: c_int = 0;

    let fdl = fd.cast::<cram_fd_layout>();
    if (*fdl).mode == b'w' as c_int && !(*fdl).ctr.is_null() {
        let ctr = (*fdl).ctr;
        if !(*ctr).slice.is_null() {
            cram_update_curr_slice_native(ctr, (*fdl).version);
        }

        if -1 == cram_cram_io_c_4275_cram_flush_container_mt(fd, ctr.cast()) {
            ret = -1;
        }

        cram_cram_io_c_3705_cram_free_container(ctr.cast());
        if (*fdl).ctr_mt == (*fdl).ctr {
            (*fdl).ctr_mt = std::ptr::null_mut();
        }
        (*fdl).ctr = std::ptr::null_mut();
    }

    ret
}
// original: cram_set_voption (htslib/cram/cram_io.c:5692)
pub unsafe fn cram_cram_io_c_5692_cram_set_voption(
    fd: *mut cram_fd,
    opt: hts_fmt_option,
    args: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> c_int {
    unsafe {
        if fd.is_null() {
            *__errno_location() = libc::EBADF;
            return -1;
        }

        let fdl = fd.cast::<cram_fd_layout>();
        match opt {
            x if x == crate::htslib_rs::cram::CRAM_OPT_DECODE_MD => {
                (*fdl).decode_md = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_PREFIX => {
                let prefix = cram_voption_va_arg_ptr::<c_char>(args);
                free((*fdl).prefix.cast());
                (*fdl).prefix = if prefix.is_null() {
                    std::ptr::null_mut()
                } else {
                    strdup(prefix)
                };
                if !prefix.is_null() && (*fdl).prefix.is_null() {
                    return -1;
                }
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_VERBOSITY => {}
            x if x == crate::htslib_rs::cram::CRAM_OPT_SEQS_PER_SLICE => {
                (*fdl).seqs_per_slice = cram_voption_va_arg_int(args);
                if (*fdl).bases_per_slice == CRAM_DEFAULT_BASES_PER_SLICE {
                    (*fdl).bases_per_slice = (*fdl).seqs_per_slice * 500;
                }
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_BASES_PER_SLICE => {
                (*fdl).bases_per_slice = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_SLICES_PER_CONTAINER => {
                (*fdl).slices_per_container = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_EMBED_REF => {
                (*fdl).embed_ref = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_NO_REF => {
                (*fdl).no_ref = cram_voption_va_arg_int(args);
            }
            x if x == CRAM_OPT_POS_DELTA => {
                (*fdl).ap_delta = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_IGNORE_MD5 => {
                (*fdl).ignore_md5 = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_LOSSY_NAMES => {
                (*fdl).lossy_read_names = cram_voption_va_arg_int(args);
                (*fdl).tlen_approx = (*fdl).lossy_read_names;
                (*fdl).tlen_zero = (*fdl).lossy_read_names;
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_USE_BZIP2 => {
                (*fdl).use_bz2 = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_USE_RANS => {
                (*fdl).use_rans = cram_voption_va_arg_int(args);
            }
            x if x == CRAM_OPT_USE_TOK => {
                (*fdl).use_tok = cram_voption_va_arg_int(args);
            }
            x if x == CRAM_OPT_USE_FQZ => {
                (*fdl).use_fqz = cram_voption_va_arg_int(args);
            }
            x if x == CRAM_OPT_USE_ARITH => {
                (*fdl).use_arith = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_USE_LZMA => {
                (*fdl).use_lzma = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_SHARED_REF => {
                (*fdl).shared_ref = 1;
                let refs = cram_voption_va_arg_ptr::<refs_t_layout>(args);
                if refs != (*fdl).refs {
                    if !(*fdl).refs.is_null() {
                        cram_cram_io_c_2427_refs_free((*fdl).refs.cast());
                    }
                    (*fdl).refs = refs;
                    if !(*fdl).refs.is_null() {
                        (*(*fdl).refs.cast::<refs_t_layout>()).count += 1;
                    }
                }
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_RANGE => {
                // Byte-faithful translation of htslib/cram/cram_io.c:5792-5801:
                //
                //   int r = cram_seek_to_refpos(fd, va_arg(args, cram_range *));
                //   pthread_mutex_lock(&fd->range_lock);
                //   if (fd->range.refid != -2)
                //       fd->required_fields |= SAM_POS;
                //   pthread_mutex_unlock(&fd->range_lock);
                //   return r;
                let range_ptr = cram_voption_va_arg_ptr::<cram_range_layout>(args);
                let r = cram_seek_to_refpos(&mut *fdl, &mut *range_ptr);
                crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).range_lock);
                if (*fdl).range.refid != -2 {
                    (*fdl).required_fields |= crate::htslib_rs::cram::SAM_POS;
                }
                crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).range_lock);
                return r;
            }
            x if x == CRAM_OPT_RANGE_NOSEEK => {
                return cram_voption_set_range_noseek(
                    fd,
                    cram_voption_va_arg_ptr::<cram_range_layout>(args),
                );
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_REFERENCE => {
                return cram_cram_io_c_3597_cram_load_reference(
                    fd,
                    cram_voption_va_arg_ptr::<c_char>(args),
                );
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_VERSION => {
                return cram_voption_set_version(fd, cram_voption_va_arg_ptr::<c_char>(args));
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_MULTI_SEQ_PER_SLICE => {
                let multi_seq = cram_voption_va_arg_int(args);
                (*fdl).multi_seq = multi_seq;
                (*fdl).multi_seq_user = multi_seq;
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_NTHREADS => {
                let nthreads = cram_voption_va_arg_int(args);
                if !(*fdl).pool.is_null() {
                    return -2;
                }
                if nthreads >= 1 {
                    (*fdl).pool = hts_tpool_init(nthreads).cast();
                    if (*fdl).pool.is_null() {
                        return -1;
                    }
                    (*fdl).rqueue =
                        hts_tpool_process_init((*fdl).pool.cast(), nthreads * 2, 0).cast();
                    (*fdl).shared_ref = 1;
                    (*fdl).own_pool = 1;
                }
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_THREAD_POOL => {
                let p = cram_voption_va_arg_ptr::<crate::htslib_rs::hts::htsThreadPool>(args);
                if !(*fdl).pool.is_null() {
                    return -2;
                }
                (*fdl).pool = if p.is_null() {
                    std::ptr::null_mut()
                } else {
                    (*p).pool.cast()
                };
                if !(*fdl).pool.is_null() {
                    let qsize = if (*p).qsize != 0 {
                        (*p).qsize
                    } else {
                        hts_tpool_size((*fdl).pool.cast()) * 2
                    };
                    (*fdl).rqueue = hts_tpool_process_init((*fdl).pool.cast(), qsize, 0).cast();
                }
                (*fdl).shared_ref = 1;
                (*fdl).own_pool = 0;
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_REQUIRED_FIELDS => {
                (*fdl).required_fields = cram_voption_va_arg_int(args) as c_uint;
                if (*fdl).range.refid != -2 {
                    (*fdl).required_fields |= crate::htslib_rs::cram::SAM_POS;
                }
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_STORE_MD => {
                (*fdl).store_md = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_STORE_NM => {
                (*fdl).store_nm = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::HTS_OPT_COMPRESSION_LEVEL => {
                (*fdl).level = cram_voption_va_arg_int(args);
            }
            x if x == HTS_OPT_PROFILE => {
                match cram_voption_va_arg_int(args) {
                    HTS_PROFILE_FAST => {
                        if (*fdl).level == CRAM_DEFAULT_LEVEL {
                            (*fdl).level = 1;
                        }
                        (*fdl).use_tok = 0;
                        (*fdl).seqs_per_slice = 10000;
                    }
                    HTS_PROFILE_NORMAL => {}
                    HTS_PROFILE_SMALL => {
                        if (*fdl).level == CRAM_DEFAULT_LEVEL {
                            (*fdl).level = 6;
                        }
                        (*fdl).use_bz2 = 1;
                        (*fdl).use_fqz = 1;
                        (*fdl).seqs_per_slice = 25000;
                    }
                    HTS_PROFILE_ARCHIVE => {
                        if (*fdl).level == CRAM_DEFAULT_LEVEL {
                            (*fdl).level = 7;
                        }
                        (*fdl).use_bz2 = 1;
                        (*fdl).use_fqz = 1;
                        (*fdl).use_arith = 1;
                        if (*fdl).level > 7 {
                            (*fdl).use_lzma = 1;
                        }
                        (*fdl).seqs_per_slice = 100000;
                    }
                    _ => {}
                }
                if (*fdl).bases_per_slice == CRAM_DEFAULT_BASES_PER_SLICE {
                    (*fdl).bases_per_slice = (*fdl).seqs_per_slice * 500;
                }
            }
            _ => {
                *__errno_location() = EINVAL;
                return -1;
            }
        }

        0
    }
}
pub fn cram_eof(fd: &cram_fd_layout) -> c_int {
    fd.eof
}
/// `cram_seek` (htslib/cram/cram_io.c:5431).
///
/// Seeks within a CRAM file: clears the out-of-coord flag, drains the
/// per-fd decode queue (if any), then forwards the request to `hseek`.
/// Returns 0 on success, -1 on failure.
pub unsafe fn cram_seek(fd: &mut cram_fd_layout, offset: libc::off_t, whence: c_int) -> c_int {
    fd.ooc = 0;

    // Drain any in-flight decode jobs natively (matches htslib's
    // cram_drain_rqueue path before the hseek).
    cram_drain_rqueue_native((fd as *mut cram_fd_layout).cast());

    if crate::htslib_rs::hfile::hseek(fd.fp, offset, whence) >= 0 {
        0
    } else {
        -1
    }
}
/// `cram_check_EOF` (htslib/cram/cram_io.c:5960).
///
/// Detects the CRAM EOF block by reading and comparing a fixed template
/// from the tail of the file. Returns 1 if found, 0 if not, 2 if the
/// underlying stream is not seekable, 3 if the CRAM version doesn't
/// support an EOF marker, and -1 on I/O failure.
pub unsafe fn cram_check_eof(fd: &mut cram_fd_layout) -> c_int {
    // Byte 9 in these templates is & with 0x0f to resolve differences
    // between ITF-8 interpretations between early Java and C
    // implementations of CRAM.
    const TEMPLATE_2_1: [u8; 30] = [
        0x0b, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xe0, 0x45, 0x4f, 0x46, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x06, 0x06, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00,
    ];
    const TEMPLATE_3: [u8; 38] = [
        0x0f, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xe0, 0x45, 0x4f, 0x46, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x05, 0xbd, 0xd9, 0x4f, 0x00, 0x01, 0x00, 0x06, 0x06, 0x01, 0x00,
        0x01, 0x00, 0x01, 0x00, 0xee, 0x63, 0x01, 0x4b,
    ];

    let version = fd.version;
    let major = (version >> 8) as u8;
    let minor = (version & 0xff) as u8;

    let template: &[u8] = if major < 2 || (major == 2 && minor == 0) {
        return 3; // No EOF support in cram versions less than 2.1
    } else if major == 2 && minor == 1 {
        &TEMPLATE_2_1
    } else {
        &TEMPLATE_3
    };
    let template_len = template.len() as libc::ssize_t;

    let fp = fd.fp;
    // htell() == fp->offset + (fp->begin - fp->buffer); begin is now a byte
    // index into fp->buffer, so the pointer subtraction collapses to `begin`.
    let offset: libc::off_t = (*fp).offset + (*fp).begin as libc::off_t;

    if crate::htslib_rs::hfile::hseek(fp, -(template_len as libc::off_t), libc::SEEK_END) < 0 {
        if *__errno_location() == libc::ESPIPE {
            // hclearerr(fp): clear pending error.
            (*fp).has_errno = 0;
            return 2;
        } else {
            return -1;
        }
    }

    let mut buf = [0u8; 38];
    if crate::htslib_rs::hfile::htslib_hfile_h_247_hread(
        fp,
        buf.as_mut_ptr().cast(),
        template_len as libc::size_t,
    ) != template_len
    {
        return -1;
    }
    if crate::htslib_rs::hfile::hseek(fp, offset, libc::SEEK_SET) < 0 {
        return -1;
    }
    buf[8] &= 0x0f;
    if buf[..template_len as usize] == *template {
        1
    } else {
        0
    }
}
pub unsafe fn cram_cram_io_c_1388_cram_new_block(
    content_type: cram_content_type,
    content_id: c_int,
) -> *mut cram_block {
    Box::into_raw(Box::new(cram_new_block_owned(content_type, content_id))).cast()
}

fn cram_new_block_owned(content_type: cram_content_type, content_id: c_int) -> cram_block_layout {
    cram_block_layout {
        method: crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW,
        orig_method: crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW,
        content_type,
        content_id,
        comp_size: 0,
        uncomp_size: 0,
        data: std::ptr::null_mut(),
        alloc: 0,
        byte: 0,
        bit: 7,
        crc32: 0,
        idx: 0,
        m: std::ptr::null_mut(),
        crc32_checked: 0,
        crc_part: 0,
    }
}
pub fn itf8_put(out: &mut [u8], val: i32) -> c_int {
    let v = val as u32;
    let (bytes, len): ([u8; 5], usize) = if (v & !0x0000_007f) == 0 {
        ([v as u8, 0, 0, 0, 0], 1)
    } else if (v & !0x0000_3fff) == 0 {
        ([(v >> 8 | 0x80) as u8, (v & 0xff) as u8, 0, 0, 0], 2)
    } else if (v & !0x001f_ffff) == 0 {
        (
            [
                (v >> 16 | 0xc0) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
                0,
                0,
            ],
            3,
        )
    } else if (v & !0x0fff_ffff) == 0 {
        (
            [
                (v >> 24 | 0xe0) as u8,
                ((v >> 16) & 0xff) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
                0,
            ],
            4,
        )
    } else {
        (
            [
                (0xf0 | ((v >> 28) & 0xff)) as u8,
                ((v >> 20) & 0xff) as u8,
                ((v >> 12) & 0xff) as u8,
                ((v >> 4) & 0xff) as u8,
                (v & 0x0f) as u8,
            ],
            5,
        )
    };
    if out.len() < len {
        return -1;
    }
    out[..len].copy_from_slice(&bytes[..len]);
    len as c_int
}

pub fn ltf8_put(out: &mut [u8], val: i64) -> c_int {
    let v = val as u64;
    let (bytes, len): ([u8; 9], usize) = if (v & !((1u64 << 7) - 1)) == 0 {
        ([v as u8, 0, 0, 0, 0, 0, 0, 0, 0], 1)
    } else if (v & !((1u64 << (6 + 8)) - 1)) == 0 {
        (
            [(v >> 8 | 0x80) as u8, (v & 0xff) as u8, 0, 0, 0, 0, 0, 0, 0],
            2,
        )
    } else if (v & !((1u64 << (5 + 2 * 8)) - 1)) == 0 {
        (
            [
                (v >> 16 | 0xc0) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            3,
        )
    } else if (v & !((1u64 << (4 + 3 * 8)) - 1)) == 0 {
        (
            [
                (v >> 24 | 0xe0) as u8,
                ((v >> 16) & 0xff) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
                0,
                0,
                0,
                0,
                0,
            ],
            4,
        )
    } else if (v & !((1u64 << (3 + 4 * 8)) - 1)) == 0 {
        (
            [
                (v >> 32 | 0xf0) as u8,
                ((v >> 24) & 0xff) as u8,
                ((v >> 16) & 0xff) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
                0,
                0,
                0,
                0,
            ],
            5,
        )
    } else if (v & !((1u64 << (2 + 5 * 8)) - 1)) == 0 {
        (
            [
                (v >> 40 | 0xf8) as u8,
                ((v >> 32) & 0xff) as u8,
                ((v >> 24) & 0xff) as u8,
                ((v >> 16) & 0xff) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
                0,
                0,
                0,
            ],
            6,
        )
    } else if (v & !((1u64 << (1 + 6 * 8)) - 1)) == 0 {
        (
            [
                (v >> 48 | 0xfc) as u8,
                ((v >> 40) & 0xff) as u8,
                ((v >> 32) & 0xff) as u8,
                ((v >> 24) & 0xff) as u8,
                ((v >> 16) & 0xff) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
                0,
                0,
            ],
            7,
        )
    } else if (v & !((1u64 << (7 * 8)) - 1)) == 0 {
        (
            [
                (v >> 56 | 0xfe) as u8,
                ((v >> 48) & 0xff) as u8,
                ((v >> 40) & 0xff) as u8,
                ((v >> 32) & 0xff) as u8,
                ((v >> 24) & 0xff) as u8,
                ((v >> 16) & 0xff) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
                0,
            ],
            8,
        )
    } else {
        (
            [
                0xff,
                ((v >> 56) & 0xff) as u8,
                ((v >> 48) & 0xff) as u8,
                ((v >> 40) & 0xff) as u8,
                ((v >> 32) & 0xff) as u8,
                ((v >> 24) & 0xff) as u8,
                ((v >> 16) & 0xff) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
            ],
            9,
        )
    };
    if out.len() < len {
        return -1;
    }
    out[..len].copy_from_slice(&bytes[..len]);
    len as c_int
}
pub unsafe fn cram_cram_io_c_138_itf8_decode(fd: *mut cram_fd, val_p: *mut i32) -> c_int {
    let nbytes = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 3, 4];
    let nbits = [
        0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x3f, 0x3f, 0x3f, 0x3f, 0x1f, 0x1f, 0x0f,
        0x0f,
    ];
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let mut val = htslib_hfile_h_163_hgetc(fp);
    if val == -1 {
        return -1;
    }

    let i = nbytes[(val >> 4) as usize];
    val &= nbits[(val >> 4) as usize];

    match i {
        0 => {
            *val_p = val;
            1
        }
        1 => {
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            *val_p = val;
            2
        }
        2 => {
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            *val_p = val;
            3
        }
        3 => {
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            *val_p = val;
            4
        }
        _ => {
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 4) | ((htslib_hfile_h_163_hgetc(fp) as u8 as c_int) & 0x0f);
            *val_p = val;
            5
        }
    }
}
pub unsafe fn cram_cram_io_c_196_itf8_decode_crc(
    fd: *mut cram_fd,
    val_p: *mut i32,
    crc: *mut u32,
) -> c_int {
    let nbytes = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 3, 4];
    let nbits = [
        0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x3f, 0x3f, 0x3f, 0x3f, 0x1f, 0x1f, 0x0f,
        0x0f,
    ];
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let mut c = [0u8; 5];

    let mut val = htslib_hfile_h_163_hgetc(fp);
    if val == -1 {
        return -1;
    }
    c[0] = val as u8;

    let i = nbytes[(val >> 4) as usize];
    val &= nbits[(val >> 4) as usize];

    if i > 0 && htslib_hfile_h_247_hread(fp, c.as_mut_ptr().add(1).cast(), i as usize) < i as isize
    {
        return -1;
    }

    match i {
        0 => {
            *val_p = val;
            *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 1);
            1
        }
        1 => {
            val = (val << 8) | c[1] as c_int;
            *val_p = val;
            *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 2);
            2
        }
        2 => {
            val = (val << 8) | c[1] as c_int;
            val = (val << 8) | c[2] as c_int;
            *val_p = val;
            *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 3);
            3
        }
        3 => {
            val = (val << 8) | c[1] as c_int;
            val = (val << 8) | c[2] as c_int;
            val = (val << 8) | c[3] as c_int;
            *val_p = val;
            *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 4);
            4
        }
        _ => {
            let mut uv = val as u32;
            uv = (uv << 8) | c[1] as u32;
            uv = (uv << 8) | c[2] as u32;
            uv = (uv << 8) | c[3] as u32;
            uv = (uv << 4) | (c[4] as u32 & 0x0f);
            *val_p = uv as i32;
            *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 5);
            5
        }
    }
}
pub unsafe fn cram_cram_io_c_382_itf8_encode(fd: *mut cram_fd, val: i32) -> c_int {
    let mut buf = [0u8; 5];
    let len = itf8_put(&mut buf, val);
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    if htslib_hfile_h_292_hwrite(fp, buf.as_ptr().cast(), len as usize) == len as libc::ssize_t {
        0
    } else {
        -1
    }
}
pub unsafe fn cram_cram_io_c_420_ltf8_decode(fd: *mut cram_fd, val_p: *mut i64) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let c = htslib_hfile_h_163_hgetc(fp);
    if c == -1 {
        return -1;
    }

    let mut val = c as u8 as u64;
    if val < 0x80 {
        *val_p = val as i64;
        1
    } else if val < 0xc0 {
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        *val_p = (val & ((1u64 << (6 + 8)) - 1)) as i64;
        2
    } else if val < 0xe0 {
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        *val_p = (val & ((1u64 << (5 + 2 * 8)) - 1)) as i64;
        3
    } else if val < 0xf0 {
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        *val_p = (val & ((1u64 << (4 + 3 * 8)) - 1)) as i64;
        4
    } else if val < 0xf8 {
        for _ in 0..4 {
            val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        }
        *val_p = (val & ((1u64 << (3 + 4 * 8)) - 1)) as i64;
        5
    } else if val < 0xfc {
        for _ in 0..5 {
            val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        }
        *val_p = (val & ((1u64 << (2 + 5 * 8)) - 1)) as i64;
        6
    } else if val < 0xfe {
        for _ in 0..6 {
            val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        }
        *val_p = (val & ((1u64 << (1 + 6 * 8)) - 1)) as i64;
        7
    } else if val < 0xff {
        for _ in 0..7 {
            val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        }
        *val_p = (val & ((1u64 << (7 * 8)) - 1)) as i64;
        8
    } else {
        for _ in 0..8 {
            val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        }
        *val_p = val as i64;
        9
    }
}
pub unsafe fn cram_cram_io_c_1068_zlib_mem_inflate(
    cdata: *mut c_char,
    csize: usize,
    size: *mut usize,
) -> *mut c_char {
    cram_cram_io_c_1157_zlib_mem_inflate(cdata, csize, size)
}
pub unsafe fn cram_cram_io_c_1157_zlib_mem_inflate(
    cdata: *mut c_char,
    csize: usize,
    size: *mut usize,
) -> *mut c_char {
    let input = std::slice::from_raw_parts(cdata.cast::<u8>(), csize);
    let mut decoder = flate2::read::GzDecoder::new(input);
    let mut out = Vec::with_capacity((csize as f64 * 1.2) as usize + 100);
    if decoder.read_to_end(&mut out).is_err() {
        return std::ptr::null_mut();
    }

    let alloc_len = out.len().max(1);
    let data = malloc(alloc_len as u64).cast::<c_char>();
    if data.is_null() {
        return std::ptr::null_mut();
    }
    if !out.is_empty() {
        memcpy(data.cast(), out.as_ptr().cast(), out.len() as u64);
    }
    *size = out.len();
    data
}
pub unsafe fn cram_cram_io_c_5127_cram_init_varint(vv: *mut c_void, version: c_int) {
    let vv = vv.cast::<varint_vec_layout>();
    if version >= 4 {
        (*vv).varint_get32 = Some(cram_cram_io_c_772_uint7_get_32);
        (*vv).varint_get32s = Some(cram_cram_io_c_780_sint7_get_32);
        (*vv).varint_get64 = Some(cram_cram_io_c_788_uint7_get_64);
        (*vv).varint_get64s = Some(cram_cram_io_c_796_sint7_get_64);
        (*vv).varint_put32 = Some(cram_cram_io_c_804_uint7_put_32);
        (*vv).varint_put32s = Some(cram_cram_io_c_808_sint7_put_32);
        (*vv).varint_put64 = Some(cram_cram_io_c_812_uint7_put_64);
        (*vv).varint_put64s = Some(cram_cram_io_c_816_sint7_put_64);
        (*vv).varint_put32_blk = Some(cram_cram_io_c_821_uint7_put_blk_32);
        (*vv).varint_put32s_blk = Some(cram_cram_io_c_831_sint7_put_blk_32);
        (*vv).varint_put64_blk = Some(cram_cram_io_c_841_uint7_put_blk_64);
        (*vv).varint_put64s_blk = Some(cram_cram_io_c_851_sint7_put_blk_64);
        (*vv).varint_size = Some(cram_cram_io_c_768_uint7_size);
        (*vv).varint_decode32_crc = cram_fn_ptr(cram_cram_io_c_862_uint7_decode_crc32 as usize);
        (*vv).varint_decode32s_crc = cram_fn_ptr(cram_cram_io_c_907_sint7_decode_crc32 as usize);
        (*vv).varint_decode64_crc = cram_fn_ptr(cram_cram_io_c_953_uint7_decode_crc64 as usize);
    } else {
        (*vv).varint_get32 = Some(cram_cram_io_c_644_safe_itf8_get);
        (*vv).varint_get32s = Some(cram_cram_io_c_644_safe_itf8_get);
        (*vv).varint_get64 = Some(cram_cram_io_c_673_safe_ltf8_get);
        (*vv).varint_get64s = Some(cram_cram_io_c_673_safe_ltf8_get);
        (*vv).varint_put32 = Some(cram_cram_io_c_747_safe_itf8_put);
        (*vv).varint_put32s = Some(cram_cram_io_c_747_safe_itf8_put);
        (*vv).varint_put64 = Some(cram_cram_io_c_751_safe_ltf8_put);
        (*vv).varint_put64s = Some(cram_cram_io_c_751_safe_ltf8_put);
        (*vv).varint_put32_blk = Some(cram_cram_io_c_620_itf8_put_blk);
        (*vv).varint_put32s_blk = Some(cram_cram_io_c_620_itf8_put_blk);
        (*vv).varint_put64_blk = Some(cram_cram_io_c_632_ltf8_put_blk);
        (*vv).varint_put64s_blk = Some(cram_cram_io_c_632_ltf8_put_blk);
        (*vv).varint_size = Some(cram_cram_io_c_755_itf8_size);
        (*vv).varint_decode32_crc = cram_fn_ptr(cram_cram_io_c_196_itf8_decode_crc as usize);
        (*vv).varint_decode32s_crc = cram_fn_ptr(cram_cram_io_c_196_itf8_decode_crc as usize);
        (*vv).varint_decode64_crc = cram_fn_ptr(cram_cram_io_c_501_ltf8_decode_crc as usize);
    }
}
pub unsafe fn cram_cram_io_c_5170_cram_init_tables(fd: *mut cram_fd) {
    let fd = fd.cast::<cram_fd_layout>();

    (*fd).l1 = [4; 256];
    (*fd).l1[b'A' as usize] = 0;
    (*fd).l1[b'a' as usize] = 0;
    (*fd).l1[b'C' as usize] = 1;
    (*fd).l1[b'c' as usize] = 1;
    (*fd).l1[b'G' as usize] = 2;
    (*fd).l1[b'g' as usize] = 2;
    (*fd).l1[b'T' as usize] = 3;
    (*fd).l1[b't' as usize] = 3;

    (*fd).l2 = [5; 256];
    (*fd).l2[b'A' as usize] = 0;
    (*fd).l2[b'a' as usize] = 0;
    (*fd).l2[b'C' as usize] = 1;
    (*fd).l2[b'c' as usize] = 1;
    (*fd).l2[b'G' as usize] = 2;
    (*fd).l2[b'g' as usize] = 2;
    (*fd).l2[b'T' as usize] = 3;
    (*fd).l2[b't' as usize] = 3;
    (*fd).l2[b'N' as usize] = 4;
    (*fd).l2[b'n' as usize] = 4;

    if ((*fd).version >> 8) == 1 {
        for i in 0..0x200usize {
            let mut f = 0;
            let i_c = i as c_int;
            if (i_c & CRAM_FPAIRED) != 0 {
                f |= BAM_FPAIRED;
            }
            if (i_c & CRAM_FPROPER_PAIR) != 0 {
                f |= BAM_FPROPER_PAIR;
            }
            if (i_c & CRAM_FUNMAP) != 0 {
                f |= BAM_FUNMAP;
            }
            if (i_c & CRAM_FREVERSE) != 0 {
                f |= BAM_FREVERSE;
            }
            if (i_c & CRAM_FREAD1) != 0 {
                f |= BAM_FREAD1;
            }
            if (i_c & CRAM_FREAD2) != 0 {
                f |= BAM_FREAD2;
            }
            if (i_c & CRAM_FSECONDARY) != 0 {
                f |= BAM_FSECONDARY;
            }
            if (i_c & CRAM_FQCFAIL) != 0 {
                f |= BAM_FQCFAIL;
            }
            if (i_c & CRAM_FDUP) != 0 {
                f |= BAM_FDUP;
            }
            (*fd).bam_flag_swap[i] = f as c_uint;
        }

        for i in 0..0x1000usize {
            let mut g = 0;
            let i_c = i as c_int;
            if (i_c & BAM_FPAIRED) != 0 {
                g |= CRAM_FPAIRED;
            }
            if (i_c & BAM_FPROPER_PAIR) != 0 {
                g |= CRAM_FPROPER_PAIR;
            }
            if (i_c & BAM_FUNMAP) != 0 {
                g |= CRAM_FUNMAP;
            }
            if (i_c & BAM_FREVERSE) != 0 {
                g |= CRAM_FREVERSE;
            }
            if (i_c & BAM_FREAD1) != 0 {
                g |= CRAM_FREAD1;
            }
            if (i_c & BAM_FREAD2) != 0 {
                g |= CRAM_FREAD2;
            }
            if (i_c & BAM_FSECONDARY) != 0 {
                g |= CRAM_FSECONDARY;
            }
            if (i_c & BAM_FQCFAIL) != 0 {
                g |= CRAM_FQCFAIL;
            }
            if (i_c & BAM_FDUP) != 0 {
                g |= CRAM_FDUP;
            }
            (*fd).cram_flag_swap[i] = g as c_uint;
        }
    } else {
        for i in 0..0x1000usize {
            (*fd).bam_flag_swap[i] = i as c_uint;
        }
        for i in 0..0x1000usize {
            (*fd).cram_flag_swap[i] = i as c_uint;
        }
    }

    (*fd).cram_sub_matrix = [[4; 32]; 32];
    for i in 0..32usize {
        (*fd).cram_sub_matrix[i][(b'A' & 0x1f) as usize] = 0;
        (*fd).cram_sub_matrix[i][(b'C' & 0x1f) as usize] = 1;
        (*fd).cram_sub_matrix[i][(b'G' & 0x1f) as usize] = 2;
        (*fd).cram_sub_matrix[i][(b'T' & 0x1f) as usize] = 3;
        (*fd).cram_sub_matrix[i][(b'N' & 0x1f) as usize] = 4;
    }
    for i in (0..20usize).step_by(4) {
        for j in 0..20usize {
            (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize][j] = 3;
            (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize][j] = 3;
            (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize][j] = 3;
            (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize][j] = 3;
        }
        (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize]
            [(CRAM_SUBST_MATRIX[i] & 0x1f) as usize] = 0;
        (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize]
            [(CRAM_SUBST_MATRIX[i + 1] & 0x1f) as usize] = 1;
        (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize]
            [(CRAM_SUBST_MATRIX[i + 2] & 0x1f) as usize] = 2;
        (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize]
            [(CRAM_SUBST_MATRIX[i + 3] & 0x1f) as usize] = 3;
    }

    cram_cram_io_c_5127_cram_init_varint(
        (&mut (*fd).vv as *mut varint_vec_layout).cast(),
        (*fd).version >> 8,
    );
}
pub unsafe fn cram_cram_io_c_4236_reset_metrics(fd: *mut cram_fd) {
    let fd = fd.cast::<cram_fd_layout>();

    if !(*fd).pool.is_null() {
        for i in 0..CRAM_DS_END {
            let m = (*fd).m[i].cast::<cram_metrics_layout>();
            if m.is_null() {
                continue;
            }
            (*m).next_trial = 999;
        }

        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fd).metrics_lock);
        hts_tpool_process_flush(&mut *(*fd).rqueue.cast::<hts_tpool_process>());
        crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fd).metrics_lock);
    }

    for i in 0..CRAM_DS_END {
        let m = (*fd).m[i].cast::<cram_metrics_layout>();
        if m.is_null() {
            continue;
        }

        (*m).trial = NTRIALS;
        (*m).next_trial = TRIAL_SPAN;
        (*m).revised_method = 0;
        (*m).unpackable = 0;
        (*m).sz = [0; 32];
    }
}
pub unsafe fn cram_cram_io_c_501_ltf8_decode_crc(
    fd: *mut cram_fd,
    val_p: *mut i64,
    crc: *mut u32,
) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let mut c = [0u8; 9];

    let mut val = htslib_hfile_h_163_hgetc(fp);
    if val < 0 {
        return -1;
    }
    c[0] = val as u8;

    if val < 0x80 {
        *val_p = val as i64;
        *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 1);
        return 1;
    } else if val < 0xc0 {
        let v = htslib_hfile_h_163_hgetc(fp);
        if v < 0 {
            return -1;
        }
        c[1] = v as u8;
        val = (val << 8) | c[1] as c_int;
        *val_p = (val & ((1 << (6 + 8)) - 1)) as i64;
        *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 2);
        return 2;
    }

    let nread = if val < 0xe0 {
        2
    } else if val < 0xf0 {
        3
    } else if val < 0xf8 {
        4
    } else if val < 0xfc {
        5
    } else if val < 0xfe {
        6
    } else if val < 0xff {
        7
    } else {
        8
    };
    if htslib_hfile_h_247_hread(fp, c.as_mut_ptr().add(1).cast(), nread) < nread as isize {
        return -1;
    }

    let len = nread + 1;
    if c[0] < 0xff {
        let mut uval = c[0] as u64;
        for &byte in c.iter().take(len).skip(1) {
            uval = (uval << 8) | byte as u64;
        }
        let bits = match len {
            3 => 5 + 2 * 8,
            4 => 4 + 3 * 8,
            5 => 3 + 4 * 8,
            6 => 2 + 5 * 8,
            7 => 1 + 6 * 8,
            8 => 7 * 8,
            _ => unreachable!(),
        };
        *val_p = (uval & ((1u64 << bits) - 1)) as i64;
    } else {
        let mut uval = c[1] as u64;
        for &byte in c.iter().skip(2) {
            uval = (uval << 8) | byte as u64;
        }
        *val_p = if c[1] < 0x80 {
            uval as i64
        } else {
            -((0xffff_ffff_ffff_ffffu64 - uval) as i64) - 1
        };
    }
    *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), len);
    len as c_int
}
pub unsafe extern "C" fn cram_cram_io_c_620_itf8_put_blk(blk: *mut cram_block, val: i32) -> c_int {
    let mut buf = [0u8; 5];
    let sz = itf8_put(&mut buf, val);
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}
pub unsafe extern "C" fn cram_cram_io_c_632_ltf8_put_blk(blk: *mut cram_block, val: i64) -> c_int {
    let mut buf = [0u8; 9];
    let sz = ltf8_put(&mut buf, val);
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}
pub unsafe extern "C" fn cram_cram_io_c_644_safe_itf8_get(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let up = (*cp).cast::<u8>();
    if !endp.is_null() {
        let remaining = endp.offset_from(*cp);
        let needed = if remaining <= 0 {
            1
        } else {
            match *up >> 4 {
                0..=7 => 1,
                8..=11 => 2,
                12..=13 => 3,
                14 => 4,
                _ => 5,
            }
        };
        if remaining < 5 && (remaining <= 0 || remaining < needed) {
            if !err.is_null() {
                *err = 1;
            }
            return 0;
        }
    }

    if *up < 0x80 {
        *cp = (*cp).add(1);
        *up as i64
    } else if *up < 0xc0 {
        *cp = (*cp).add(2);
        ((((*up as u32) << 8) | *up.add(1) as u32) & 0x3fff) as i32 as i64
    } else if *up < 0xe0 {
        *cp = (*cp).add(3);
        ((((*up as u32) << 16) | ((*up.add(1) as u32) << 8) | *up.add(2) as u32) & 0x1f_ffff) as i32
            as i64
    } else if *up < 0xf0 {
        *cp = (*cp).add(4);
        ((((*up as u32) << 24)
            | ((*up.add(1) as u32) << 16)
            | ((*up.add(2) as u32) << 8)
            | *up.add(3) as u32)
            & 0x0fff_ffff) as i32 as i64
    } else {
        *cp = (*cp).add(5);
        (((((*up as u32) & 0x0f) << 28)
            | ((*up.add(1) as u32) << 20)
            | ((*up.add(2) as u32) << 12)
            | ((*up.add(3) as u32) << 4)
            | ((*up.add(4) as u32) & 0x0f)) as i32) as i64
    }
}
pub unsafe extern "C" fn cram_cram_io_c_673_safe_ltf8_get(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let up = (*cp).cast::<u8>();
    if !endp.is_null() {
        let remaining = endp.offset_from(*cp);
        let needed = if remaining <= 0 || *up < 0x80 {
            1
        } else if *up < 0xc0 {
            2
        } else if *up < 0xe0 {
            3
        } else if *up < 0xf0 {
            4
        } else if *up < 0xf8 {
            5
        } else if *up < 0xfc {
            6
        } else if *up < 0xfe {
            7
        } else if *up < 0xff {
            8
        } else {
            9
        };
        if remaining < 9 && (remaining <= 0 || remaining < needed) {
            if !err.is_null() {
                *err = 1;
            }
            return 0;
        }
    }

    if *up < 0x80 {
        *cp = (*cp).add(1);
        *up as i64
    } else if *up < 0xc0 {
        *cp = (*cp).add(2);
        ((((*up as u64) << 8) | *up.add(1) as u64) & ((1u64 << (6 + 8)) - 1)) as i64
    } else if *up < 0xe0 {
        *cp = (*cp).add(3);
        ((((*up as u64) << 16) | ((*up.add(1) as u64) << 8) | *up.add(2) as u64)
            & ((1u64 << (5 + 2 * 8)) - 1)) as i64
    } else if *up < 0xf0 {
        *cp = (*cp).add(4);
        ((((*up as u64) << 24)
            | ((*up.add(1) as u64) << 16)
            | ((*up.add(2) as u64) << 8)
            | *up.add(3) as u64)
            & ((1u64 << (4 + 3 * 8)) - 1)) as i64
    } else if *up < 0xf8 {
        *cp = (*cp).add(5);
        ((((*up as u64) << 32)
            | ((*up.add(1) as u64) << 24)
            | ((*up.add(2) as u64) << 16)
            | ((*up.add(3) as u64) << 8)
            | *up.add(4) as u64)
            & ((1u64 << (3 + 4 * 8)) - 1)) as i64
    } else if *up < 0xfc {
        *cp = (*cp).add(6);
        ((((*up as u64) << 40)
            | ((*up.add(1) as u64) << 32)
            | ((*up.add(2) as u64) << 24)
            | ((*up.add(3) as u64) << 16)
            | ((*up.add(4) as u64) << 8)
            | *up.add(5) as u64)
            & ((1u64 << (2 + 5 * 8)) - 1)) as i64
    } else if *up < 0xfe {
        *cp = (*cp).add(7);
        ((((*up as u64) << 48)
            | ((*up.add(1) as u64) << 40)
            | ((*up.add(2) as u64) << 32)
            | ((*up.add(3) as u64) << 24)
            | ((*up.add(4) as u64) << 16)
            | ((*up.add(5) as u64) << 8)
            | *up.add(6) as u64)
            & ((1u64 << (1 + 6 * 8)) - 1)) as i64
    } else if *up < 0xff {
        *cp = (*cp).add(8);
        ((((*up.add(1) as u64) << 48)
            | ((*up.add(2) as u64) << 40)
            | ((*up.add(3) as u64) << 32)
            | ((*up.add(4) as u64) << 24)
            | ((*up.add(5) as u64) << 16)
            | ((*up.add(6) as u64) << 8)
            | *up.add(7) as u64)
            & ((1u64 << (7 * 8)) - 1)) as i64
    } else {
        *cp = (*cp).add(9);
        (((*up.add(1) as u64) << 56)
            | ((*up.add(2) as u64) << 48)
            | ((*up.add(3) as u64) << 40)
            | ((*up.add(4) as u64) << 32)
            | ((*up.add(5) as u64) << 24)
            | ((*up.add(6) as u64) << 16)
            | ((*up.add(7) as u64) << 8)
            | *up.add(8) as u64) as i64
    }
}
pub unsafe extern "C" fn cram_cram_io_c_747_safe_itf8_put(
    cp: *mut c_char,
    _cp_end: *mut c_char,
    val: i32,
) -> c_int {
    let Some(out) = cp.cast::<u8>().as_mut() else {
        return -1;
    };
    let out = std::slice::from_raw_parts_mut(out, 5);
    itf8_put(out, val)
}
pub unsafe extern "C" fn cram_cram_io_c_751_safe_ltf8_put(
    cp: *mut c_char,
    _cp_end: *mut c_char,
    val: i64,
) -> c_int {
    let Some(out) = cp.cast::<u8>().as_mut() else {
        return -1;
    };
    let out = std::slice::from_raw_parts_mut(out, 9);
    ltf8_put(out, val)
}
pub extern "C" fn cram_cram_io_c_755_itf8_size(v: i64) -> c_int {
    if (v & !0x7f) == 0 {
        1
    } else if (v & !0x3fff) == 0 {
        2
    } else if (v & !0x1f_ffff) == 0 {
        3
    } else if (v & !0x0fff_ffff) == 0 {
        4
    } else {
        5
    }
}
pub extern "C" fn cram_cram_io_c_768_uint7_size(v: i64) -> c_int {
    let mut v = v as u64;
    let mut n = 1;
    while v >= 0x80 {
        n += 1;
        v >>= 7;
    }
    n
}
pub unsafe extern "C" fn cram_cram_io_c_772_uint7_get_32(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let mut p = (*cp).cast::<u8>();
    let end = endp.cast::<u8>();
    let mut val = 0u32;
    let mut nb = 0usize;
    let limit = if end.is_null() || end.offset_from(p) >= 6 {
        6
    } else if p >= end as *mut u8 {
        if !err.is_null() {
            *err = 1;
        }
        return 0;
    } else {
        end.offset_from(p) as usize
    };

    while nb < limit {
        let c = *p;
        p = p.add(1);
        nb += 1;
        val = (val << 7) | (c & 0x7f) as u32;
        if (c & 0x80) == 0 {
            break;
        }
    }

    *cp = p.cast();
    val as i64
}
pub unsafe extern "C" fn cram_cram_io_c_780_sint7_get_32(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let u = cram_cram_io_c_772_uint7_get_32(cp, endp, err) as u32;
    ((u >> 1) as i32 ^ -((u & 1) as i32)) as i64
}
pub unsafe extern "C" fn cram_cram_io_c_788_uint7_get_64(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let mut p = (*cp).cast::<u8>();
    let end = endp.cast::<u8>();
    let mut val = 0u64;
    let mut nb = 0usize;
    let limit = if end.is_null() || end.offset_from(p) >= 11 {
        11
    } else if p >= end as *mut u8 {
        if !err.is_null() {
            *err = 1;
        }
        return 0;
    } else {
        end.offset_from(p) as usize
    };

    while nb < limit {
        let c = *p;
        p = p.add(1);
        nb += 1;
        val = (val << 7) | (c & 0x7f) as u64;
        if (c & 0x80) == 0 {
            break;
        }
    }

    *cp = p.cast();
    val as i64
}
pub unsafe extern "C" fn cram_cram_io_c_796_sint7_get_64(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let u = cram_cram_io_c_788_uint7_get_64(cp, endp, err) as u64;
    ((u >> 1) as i64) ^ -((u & 1) as i64)
}
pub unsafe extern "C" fn cram_cram_io_c_804_uint7_put_32(
    cp: *mut c_char,
    endp: *mut c_char,
    val: i32,
) -> c_int {
    cram_cram_io_c_812_uint7_put_64(cp, endp, val as u32 as i64)
}
pub unsafe extern "C" fn cram_cram_io_c_808_sint7_put_32(
    cp: *mut c_char,
    endp: *mut c_char,
    val: i32,
) -> c_int {
    cram_cram_io_c_804_uint7_put_32(cp, endp, ((val as u32) << 1 ^ (val >> 31) as u32) as i32)
}
pub unsafe extern "C" fn cram_cram_io_c_812_uint7_put_64(
    cp: *mut c_char,
    endp: *mut c_char,
    val: i64,
) -> c_int {
    let mut p = cp.cast::<u8>();
    let end = endp.cast::<u8>();
    let v = val as u64;
    let n = cram_cram_io_c_768_uint7_size(val);

    if !end.is_null() && end.offset_from(p) < n as isize {
        return 0;
    }

    for i in (0..n).rev() {
        let mut c = ((v >> (i * 7)) & 0x7f) as u8;
        if i != 0 {
            c |= 0x80;
        }
        *p = c;
        p = p.add(1);
    }

    n
}
pub unsafe extern "C" fn cram_cram_io_c_816_sint7_put_64(
    cp: *mut c_char,
    endp: *mut c_char,
    val: i64,
) -> c_int {
    cram_cram_io_c_812_uint7_put_64(cp, endp, ((val as u64) << 1 ^ (val >> 63) as u64) as i64)
}
pub unsafe extern "C" fn cram_cram_io_c_821_uint7_put_blk_32(
    blk: *mut cram_block,
    v: i32,
) -> c_int {
    let mut buf = [0u8; 10];
    let sz = cram_cram_io_c_804_uint7_put_32(
        buf.as_mut_ptr().cast(),
        buf.as_mut_ptr().add(10).cast(),
        v,
    );
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}
pub unsafe extern "C" fn cram_cram_io_c_831_sint7_put_blk_32(
    blk: *mut cram_block,
    v: i32,
) -> c_int {
    let mut buf = [0u8; 10];
    let sz = cram_cram_io_c_808_sint7_put_32(
        buf.as_mut_ptr().cast(),
        buf.as_mut_ptr().add(10).cast(),
        v,
    );
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}
pub unsafe extern "C" fn cram_cram_io_c_841_uint7_put_blk_64(
    blk: *mut cram_block,
    v: i64,
) -> c_int {
    let mut buf = [0u8; 10];
    let sz = cram_cram_io_c_812_uint7_put_64(
        buf.as_mut_ptr().cast(),
        buf.as_mut_ptr().add(10).cast(),
        v,
    );
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}
pub unsafe extern "C" fn cram_cram_io_c_851_sint7_put_blk_64(
    blk: *mut cram_block,
    v: i64,
) -> c_int {
    let mut buf = [0u8; 10];
    let sz = cram_cram_io_c_816_sint7_put_64(
        buf.as_mut_ptr().cast(),
        buf.as_mut_ptr().add(10).cast(),
        v,
    );
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}
pub unsafe fn cram_cram_io_c_862_uint7_decode_crc32(
    fd: *mut crate::htslib_rs::hts::cram_fd,
    val_p: *mut i32,
    crc: *mut u32,
) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let mut b = [0u8; 5];
    let mut i = 0usize;
    let mut v = 0u32;

    loop {
        let c = if (*fp).end > (*fp).begin {
            let buf = &(*fp).buffer;
            let c = buf[(*fp).begin];
            (*fp).begin += 1;
            c as c_int
        } else {
            crate::htslib_rs::hfile::hgetc2(fp)
        };
        if c < 0 {
            return -1;
        }
        b[i] = c as u8;
        i += 1;
        v = (v << 7) | (c as u32 & 0x7f);
        if i >= 5 || (c & 0x80) == 0 {
            break;
        }
    }

    *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, b.as_ptr().cast(), i);
    *val_p = v as i32;
    i as c_int
}
pub unsafe fn cram_cram_io_c_907_sint7_decode_crc32(
    fd: *mut crate::htslib_rs::hts::cram_fd,
    val_p: *mut i32,
    crc: *mut u32,
) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let mut b = [0u8; 5];
    let mut i = 0usize;
    let mut v = 0u32;

    loop {
        let c = if (*fp).end > (*fp).begin {
            let buf = &(*fp).buffer;
            let c = buf[(*fp).begin];
            (*fp).begin += 1;
            c as c_int
        } else {
            crate::htslib_rs::hfile::hgetc2(fp)
        };
        if c < 0 {
            return -1;
        }
        b[i] = c as u8;
        i += 1;
        v = (v << 7) | (c as u32 & 0x7f);
        if i >= 5 || (c & 0x80) == 0 {
            break;
        }
    }

    *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, b.as_ptr().cast(), i);
    *val_p = ((v >> 1) as i32) ^ -((v & 1) as i32);
    i as c_int
}
pub unsafe fn cram_cram_io_c_953_uint7_decode_crc64(
    fd: *mut crate::htslib_rs::hts::cram_fd,
    val_p: *mut i64,
    crc: *mut u32,
) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let mut b = [0u8; 10];
    let mut i = 0usize;
    let mut v = 0u64;

    loop {
        let c = if (*fp).end > (*fp).begin {
            let buf = &(*fp).buffer;
            let c = buf[(*fp).begin];
            (*fp).begin += 1;
            c as c_int
        } else {
            crate::htslib_rs::hfile::hgetc2(fp)
        };
        if c < 0 {
            return -1;
        }
        b[i] = c as u8;
        i += 1;
        v = (v << 7) | (c as u64 & 0x7f);
        if i >= 5 || (c & 0x80) == 0 {
            break;
        }
    }

    *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, b.as_ptr().cast(), i);
    *val_p = v as i64;
    i as c_int
}
pub unsafe fn int32_decode(fd: &mut cram_fd_layout, val: &mut i32) -> c_int {
    let fp = fd.fp;
    let mut i = 0i32;
    let buffer = (&mut i as *mut i32).cast::<c_void>();
    let nbytes = std::mem::size_of::<i32>();
    let mut n = (*fp).end - (*fp).begin;
    if n > nbytes {
        n = nbytes;
    }
    let begin = (*fp).begin;
    std::ptr::copy_nonoverlapping((*fp).buffer.as_ptr().add(begin), buffer.cast::<u8>(), n);
    (*fp).begin += n;
    let got = if n == nbytes || ((*fp).flags & HFILE_MOBILE) == 0 {
        n as libc::ssize_t
    } else {
        crate::htslib_rs::hfile::hread2(fp, buffer, nbytes, n)
    };
    if got != nbytes as libc::ssize_t {
        return -1;
    }

    *val = i32::from_le(i);
    4
}
pub unsafe fn int32_encode(fd: &mut cram_fd_layout, val: i32) -> c_int {
    let fp = fd.fp;
    let v = val.to_le();
    let buffer = (&v as *const i32).cast::<c_void>();
    let nbytes = std::mem::size_of::<i32>();

    if ((*fp).flags & HFILE_MOBILE) == 0 {
        let n = (*fp).limit - (*fp).begin;
        if n < nbytes {
            crate::htslib_rs::hfile::hfile_set_blksize(fp, (*fp).limit + nbytes);
            (*fp).end = (*fp).limit;
        }
    }

    let mut n = (*fp).limit - (*fp).begin;
    let wrote = if nbytes >= n && (*fp).begin == 0 {
        crate::htslib_rs::hfile::hwrite2(fp, buffer, nbytes, 0)
    } else {
        if n > nbytes {
            n = nbytes;
        }
        let begin = (*fp).begin;
        std::ptr::copy_nonoverlapping(buffer.cast::<u8>(), (*fp).buffer.as_mut_ptr().add(begin), n);
        (*fp).begin += n;
        if n == nbytes {
            n as libc::ssize_t
        } else {
            crate::htslib_rs::hfile::hwrite2(fp, buffer, nbytes, n)
        }
    };

    if wrote != nbytes as libc::ssize_t {
        return -1;
    }
    4
}
pub unsafe fn int32_get_blk(block: &mut cram_block_layout, val: &mut i32) -> c_int {
    if block.uncomp_size < 0 || (block.uncomp_size as usize).saturating_sub(block.byte) < 4 {
        return -1;
    }

    let data = block.data.add(block.byte);
    let v = (*data as u32)
        | ((*data.add(1) as u32) << 8)
        | ((*data.add(2) as u32) << 16)
        | ((*data.add(3) as u32) << 24);
    *val = v as i32;
    block.byte += 4;
    4
}
pub unsafe fn int32_put_blk(block: &mut cram_block_layout, val: i32) -> c_int {
    let v = val as u32;
    let cp = [
        (v & 0xff) as u8,
        ((v >> 8) & 0xff) as u8,
        ((v >> 16) & 0xff) as u8,
        ((v >> 24) & 0xff) as u8,
    ];
    if cram_cram_io_h_248_block_append(
        (block as *mut cram_block_layout).cast(),
        cp.as_ptr().cast(),
        4,
    ) != 0
    {
        return -1;
    }
    0
}
struct CramReadBlockOwner {
    ptr: *mut cram_block_layout,
    owns_data: bool,
}

impl CramReadBlockOwner {
    fn new(ptr: *mut cram_block_layout) -> Self {
        Self {
            ptr,
            owns_data: false,
        }
    }

    fn set_owns_data(&mut self) {
        self.owns_data = true;
    }

    fn release(mut self) -> *mut cram_block {
        let ptr = self.ptr;
        self.ptr = std::ptr::null_mut();
        self.owns_data = false;
        ptr.cast()
    }
}

impl Drop for CramReadBlockOwner {
    fn drop(&mut self) {
        unsafe {
            if self.ptr.is_null() {
                return;
            }
            if self.owns_data {
                free((*self.ptr).data.cast());
            }
            drop(Box::from_raw(self.ptr));
        }
    }
}

pub unsafe fn cram_read_block(fd_layout: &mut cram_fd_layout) -> *mut cram_block {
    let fp = fd_layout.fp;
    let b = Box::into_raw(Box::new(cram_new_block_owned(
        CRAM_CONTENT_TYPE_FILE_HEADER,
        0,
    )));
    let mut block_owner = CramReadBlockOwner::new(b);

    let c = if (*fp).end > (*fp).begin {
        let buf = &(*fp).buffer;
        let c = buf[(*fp).begin];
        (*fp).begin += 1;
        c as c_int
    } else {
        crate::htslib_rs::hfile::hgetc2(fp)
    };
    if c == -1 {
        return std::ptr::null_mut();
    }
    (*b).method = c;
    if (*b).method > 8 {
        return std::ptr::null_mut();
    }
    let c_byte = c as u8;
    let mut crc = crate::htslib_rs::bgzf::hts_crc32(0, (&c_byte as *const u8).cast(), 1);

    let c = if (*fp).end > (*fp).begin {
        let buf = &(*fp).buffer;
        let c = buf[(*fp).begin];
        (*fp).begin += 1;
        c as c_int
    } else {
        crate::htslib_rs::hfile::hgetc2(fp)
    };
    if c == -1 {
        return std::ptr::null_mut();
    }
    (*b).content_type = c;
    let c_byte = c as u8;
    crc = crate::htslib_rs::bgzf::hts_crc32(crc, (&c_byte as *const u8).cast(), 1);

    if (fd_layout.version >> 8) >= 4 {
        if cram_cram_io_c_862_uint7_decode_crc32(
            (fd_layout as *mut cram_fd_layout).cast(),
            &mut (*b).content_id,
            &mut crc,
        ) == -1
            || cram_cram_io_c_862_uint7_decode_crc32(
                (fd_layout as *mut cram_fd_layout).cast(),
                &mut (*b).comp_size,
                &mut crc,
            ) == -1
            || cram_cram_io_c_862_uint7_decode_crc32(
                (fd_layout as *mut cram_fd_layout).cast(),
                &mut (*b).uncomp_size,
                &mut crc,
            ) == -1
        {
            return std::ptr::null_mut();
        }
    } else {
        for out in [
            &mut (*b).content_id,
            &mut (*b).comp_size,
            &mut (*b).uncomp_size,
        ] {
            let mut buf = [0u8; 5];
            let mut n = 0usize;
            let mut want = 0usize;
            loop {
                let c = if (*fp).end > (*fp).begin {
                    let buf = &(*fp).buffer;
                    let c = buf[(*fp).begin];
                    (*fp).begin += 1;
                    c as c_int
                } else {
                    crate::htslib_rs::hfile::hgetc2(fp)
                };
                if c < 0 {
                    return std::ptr::null_mut();
                }
                buf[n] = c as u8;
                n += 1;
                if n == 1 {
                    want = match buf[0] {
                        0x00..=0x7f => 1,
                        0x80..=0xbf => 2,
                        0xc0..=0xdf => 3,
                        0xe0..=0xef => 4,
                        _ => 5,
                    };
                }
                if n == want {
                    break;
                }
            }
            let mut cp = buf.as_mut_ptr().cast::<c_char>();
            *out = cram_cram_io_c_644_safe_itf8_get(
                &mut cp,
                buf.as_ptr().add(n).cast::<c_char>(),
                std::ptr::null_mut(),
            ) as i32;
            crc = crate::htslib_rs::bgzf::hts_crc32(crc, buf.as_ptr().cast(), n);
        }
    }

    let data_len = if (*b).method == crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW {
        if (*b).uncomp_size < 0 || (*b).comp_size != (*b).uncomp_size {
            return std::ptr::null_mut();
        }
        (*b).uncomp_size as usize
    } else {
        if (*b).comp_size < 0 || (*b).uncomp_size < 0 {
            return std::ptr::null_mut();
        }
        (*b).comp_size as usize
    };

    (*b).alloc = data_len;
    (*b).data = if data_len == 0 {
        std::ptr::null_mut()
    } else {
        malloc(data_len as u64).cast::<u8>()
    };
    if data_len != 0 && (*b).data.is_null() {
        return std::ptr::null_mut();
    }
    block_owner.set_owns_data();
    let mut n = (*fp).end - (*fp).begin;
    if n > data_len {
        n = data_len;
    }
    if data_len != 0 {
        let begin = (*fp).begin;
        std::ptr::copy_nonoverlapping((*fp).buffer.as_ptr().add(begin), (*b).data, n);
    }
    (*fp).begin += n;
    let got = if n == data_len || ((*fp).flags & HFILE_MOBILE) == 0 {
        n as libc::ssize_t
    } else {
        crate::htslib_rs::hfile::hread2(fp, (*b).data.cast(), data_len, n)
    };
    if got != data_len as libc::ssize_t {
        return std::ptr::null_mut();
    }

    if (fd_layout.version >> 8) >= 3 {
        let mut crc32 = 0i32;
        if int32_decode(fd_layout, &mut crc32) == -1 {
            return std::ptr::null_mut();
        }
        (*b).crc32 = crc32 as u32;
        (*b).crc32_checked = fd_layout.ignore_md5;
        (*b).crc_part = crc;
    } else {
        (*b).crc32_checked = 1;
        (*b).crc_part = 0;
        (*b).crc32 = 0;
    }
    (*b).orig_method = (*b).method;
    (*b).idx = 0;
    (*b).byte = 0;
    (*b).bit = 7;
    (*b).m = std::ptr::null_mut();
    block_owner.release()
}
pub unsafe fn cram_write_block(
    fd_layout: &mut cram_fd_layout,
    b: &mut cram_block_layout,
) -> c_int {
    let fp = fd_layout.fp;

    for c in [b.method, b.content_type] {
        let r = if (*fp).begin < (*fp).limit {
            let begin = (*fp).begin;
            let buf = &mut (*fp).buffer;
            buf[begin] = c as u8;
            (*fp).begin += 1;
            c
        } else {
            crate::htslib_rs::hfile::hputc2(c, fp)
        };
        if r == libc::EOF {
            return -1;
        }
    }

    let mut vardata = [0u8; 100];
    let mut vardata_o = 0usize;
    if (fd_layout.version >> 8) >= 4 {
        for val in [b.content_id, b.comp_size, b.uncomp_size] {
            let n = cram_cram_io_c_804_uint7_put_32(
                vardata.as_mut_ptr().add(vardata_o).cast(),
                vardata.as_mut_ptr().add(vardata.len()).cast(),
                val,
            );
            if n <= 0 {
                return -1;
            }
            vardata_o += n as usize;
        }
    } else {
        for val in [b.content_id, b.comp_size, b.uncomp_size] {
            let n = itf8_put(&mut vardata[vardata_o..], val);
            vardata_o += n as usize;
        }
    }

    let mut n = (*fp).limit - (*fp).begin;
    let wrote = if vardata_o >= n && (*fp).begin == 0 {
        crate::htslib_rs::hfile::hwrite2(fp, vardata.as_ptr().cast(), vardata_o, 0)
    } else {
        if n > vardata_o {
            n = vardata_o;
        }
        let begin = (*fp).begin;
        std::ptr::copy_nonoverlapping(vardata.as_ptr(), (*fp).buffer.as_mut_ptr().add(begin), n);
        (*fp).begin += n;
        if n == vardata_o {
            n as libc::ssize_t
        } else {
            crate::htslib_rs::hfile::hwrite2(fp, vardata.as_ptr().cast(), vardata_o, n)
        }
    };
    if wrote != vardata_o as libc::ssize_t {
        return -1;
    }

    let data_len = if b.method == crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW {
        b.uncomp_size
    } else {
        b.comp_size
    };
    if !b.data.is_null() {
        let data_len = data_len as usize;
        let mut n = (*fp).limit - (*fp).begin;
        let wrote = if data_len >= n && (*fp).begin == 0 {
            crate::htslib_rs::hfile::hwrite2(fp, b.data.cast(), data_len, 0)
        } else {
            if n > data_len {
                n = data_len;
            }
            let begin = (*fp).begin;
            std::ptr::copy_nonoverlapping(b.data, (*fp).buffer.as_mut_ptr().add(begin), n);
            (*fp).begin += n;
            if n == data_len {
                n as libc::ssize_t
            } else {
                crate::htslib_rs::hfile::hwrite2(fp, b.data.cast(), data_len, n)
            }
        };
        if wrote != data_len as libc::ssize_t {
            return -1;
        }
    }

    if (fd_layout.version >> 8) >= 3 {
        let mut dat = [0u8; 100];
        let mut cp = 0usize;
        dat[cp] = b.method as u8;
        cp += 1;
        dat[cp] = b.content_type as u8;
        cp += 1;
        if (fd_layout.version >> 8) >= 4 {
            for val in [b.content_id, b.comp_size, b.uncomp_size] {
                cp += cram_cram_io_c_804_uint7_put_32(
                    dat.as_mut_ptr().add(cp).cast(),
                    dat.as_mut_ptr().add(dat.len()).cast(),
                    val,
                ) as usize;
            }
        } else {
            for val in [b.content_id, b.comp_size, b.uncomp_size] {
                cp += itf8_put(&mut dat[cp..], val) as usize;
            }
        }
        let mut crc = crate::htslib_rs::bgzf::hts_crc32(0, dat.as_ptr().cast(), cp);
        let data_len = if b.method == crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW {
            b.uncomp_size
        } else {
            b.comp_size
        } as usize;
        crc = crate::htslib_rs::bgzf::hts_crc32(
            crc,
            if b.data.is_null() {
                c"".as_ptr().cast()
            } else {
                b.data.cast()
            },
            data_len,
        );
        b.crc32 = crc;
        if int32_encode(fd_layout, crc as i32) == -1 {
            return -1;
        }
    }

    0
}
pub unsafe fn cram_uncompress_block(b: &mut cram_block_layout) -> c_int {
    if b.crc32_checked == 0 {
        let crc = crate::htslib_rs::bgzf::hts_crc32(
            b.crc_part,
            if b.data.is_null() {
                c"".as_ptr().cast()
            } else {
                b.data.cast()
            },
            b.alloc,
        );
        b.crc32_checked = 1;
        if crc != b.crc32 {
            return -1;
        }
    }

    if b.uncomp_size == 0 {
        b.method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
        return 0;
    }

    match b.method {
        crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW => 0,
        crate::htslib_rs::cram::CRAM_BLOCK_METHOD_GZIP => {
            let mut uncomp_size = b.uncomp_size as usize;
            let uncomp = cram_cram_io_c_1157_zlib_mem_inflate(
                b.data.cast::<c_char>(),
                b.comp_size as usize,
                &mut uncomp_size,
            );
            if uncomp.is_null() {
                return -1;
            }
            if uncomp_size != b.uncomp_size as usize {
                free(uncomp.cast());
                return -1;
            }
            free(b.data.cast());
            b.data = uncomp.cast();
            b.alloc = uncomp_size;
            b.method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            0
        }
        crate::htslib_rs::cram::CRAM_BLOCK_METHOD_BZIP2 => -1,
        crate::htslib_rs::cram::CRAM_BLOCK_METHOD_LZMA => -1,
        crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RANS => {
            let usize = b.uncomp_size as c_uint;
            let input = std::slice::from_raw_parts(b.data, b.comp_size as usize);
            let v = match crate::htslib_rs::htscodecs::rans_static::rans_uncompress(input) {
                Some(v) => v,
                None => return -1,
            };
            let usize2 = v.len() as c_uint;
            let uncomp = cram_dup_to_malloc(&v);
            if uncomp.is_null() {
                return -1;
            }
            if usize != usize2 {
                free(uncomp.cast());
                return -1;
            }
            free(b.data.cast());
            b.data = uncomp;
            b.alloc = usize2 as usize;
            b.method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            b.uncomp_size = usize2 as i32;
            0
        }
        7 => {
            let mut uncomp_size = b.uncomp_size as usize;
            let input = std::slice::from_raw_parts(b.data, b.comp_size as usize);
            let v = crate::htslib_rs::htscodecs::fqzcomp_qual::fqz_decompress(
                input,
                &mut uncomp_size,
                &mut [],
                0,
            );
            let uncomp = cram_dup_to_malloc(&v);
            if uncomp.is_null() {
                return -1;
            }
            free(b.data.cast());
            b.data = uncomp;
            b.alloc = uncomp_size;
            b.method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            b.uncomp_size = uncomp_size as i32;
            0
        }
        5 => {
            let usize = b.uncomp_size as c_uint;
            let mut usize2 = 0 as c_uint;
            let input = std::slice::from_raw_parts(b.data, b.comp_size as usize);
            let v = crate::htslib_rs::htscodecs::rans_static4x16pr::rans_uncompress_4x16(
                input,
                &mut usize2,
            );
            let uncomp = cram_dup_to_malloc(&v);
            if uncomp.is_null() {
                return -1;
            }
            if usize != usize2 {
                free(uncomp.cast());
                return -1;
            }
            b.orig_method = 5;
            free(b.data.cast());
            b.data = uncomp;
            b.alloc = usize2 as usize;
            b.method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            b.uncomp_size = usize2 as i32;
            0
        }
        6 => {
            let usize = b.uncomp_size as c_uint;
            let mut usize2 = 0 as c_uint;
            let input = std::slice::from_raw_parts(b.data, b.comp_size as usize);
            let v = crate::htslib_rs::htscodecs::arith_dynamic::arith_uncompress_to(
                input,
                None,
                &mut usize2,
            );
            let uncomp = cram_dup_to_malloc(&v);
            if uncomp.is_null() {
                return -1;
            }
            if usize != usize2 {
                free(uncomp.cast());
                return -1;
            }
            b.orig_method = 6;
            free(b.data.cast());
            b.data = uncomp;
            b.alloc = usize2 as usize;
            b.method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            b.uncomp_size = usize2 as i32;
            0
        }
        8 => {
            let mut out_len = 0u32;
            let input = std::slice::from_raw_parts(b.data, b.comp_size as usize);
            let cp = match crate::htslib_rs::htscodecs::tokenise_name3::tok3_decode_names(
                input,
                b.comp_size as u32,
                &mut out_len,
            ) {
                Some(v) => cram_dup_to_malloc(&v),
                None => std::ptr::null_mut(),
            };
            if cp.is_null() {
                return -1;
            }
            b.orig_method = 8;
            b.method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            free(b.data.cast());
            b.data = cp;
            b.alloc = out_len as usize;
            b.uncomp_size = out_len as i32;
            0
        }
        _ => -1,
    }
}
/// `cram_compress_block2` (htslib/cram/cram_io.c:2317): thin wrapper.
///
/// The slice pointer is opaque here (cram_slice doesn't have a pub native
/// type alias yet); pass it via `*mut c_void` and cast inside.
pub unsafe fn cram_cram_io_c_2317_cram_compress_block2(
    fd: *mut cram_fd,
    s: *mut c_void,
    b: *mut cram_block,
    metrics: *mut cram_metrics,
    method: c_int,
    level: c_int,
) -> c_int {
    cram_cram_io_c_1913_cram_compress_block3(
        fd,
        s.cast::<cram_slice_layout>(),
        b.cast::<cram_block_layout>(),
        metrics.cast::<cram_metrics_layout>(),
        method,
        level,
        0,
    )
}
/// `cram_compress_block` (htslib/cram/cram_io.c:2323): public entry, no slice.
pub unsafe fn cram_cram_io_c_2323_cram_compress_block(
    fd: *mut cram_fd,
    b: *mut cram_block,
    metrics: *mut cram_metrics,
    method: c_int,
    level: c_int,
) -> c_int {
    cram_cram_io_c_2317_cram_compress_block2(fd, std::ptr::null_mut(), b, metrics, method, level)
}
pub unsafe fn cram_cram_io_c_2327_cram_new_metrics() -> *mut cram_metrics {
    let m =
        calloc(1, std::mem::size_of::<cram_metrics_layout>() as u64).cast::<cram_metrics_layout>();
    if m.is_null() {
        return std::ptr::null_mut();
    }

    (*m).trial = 2;
    (*m).next_trial = 35;
    (*m).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
    (*m).strat = 0;
    (*m).revised_method = 0;
    (*m).unpackable = 0;

    m.cast()
}
pub unsafe fn cram_cram_io_c_2417_ref_entry_free_seq(e: *mut c_void) {
    let e = e.cast::<ref_entry_layout>();
    if !(*e).mf.is_null() {
        cram_mFILE_c_361_mfclose((*e).mf);
    }

    (*e).seq = Vec::new();
    (*e).mf = std::ptr::null_mut();
}
pub unsafe fn cram_cram_io_c_2427_refs_free(r: *mut refs_t) {
    let r = r.cast::<refs_t_layout>();
    if r.is_null() {
        return;
    }

    (*r).count -= 1;
    if (*r).count > 0 {
        return;
    }

    if let Some(p) = (*r).pool.take() {
        cram_string_alloc_c_103_string_pool_destroy(p);
    }

    if !(*r).h_meta.is_null() {
        let h = (*r).h_meta;
        for k in 0..(*h).n_buckets {
            if ((*(*h).flags.add((k >> 4) as usize) >> ((k & 0x0f) << 1)) & 3) != 0 {
                continue;
            }
            let e = *(*h).vals.add(k as usize);
            if e.is_null() {
                continue;
            }
            cram_cram_io_c_2417_ref_entry_free_seq(e.cast());
            free(e.cast());
        }
        free((*h).flags.cast());
        free((*h).keys.cast());
        free((*h).vals.cast());
        free(h.cast());
    }

    if !(*r).ref_id.is_null() {
        free((*r).ref_id.cast());
    }

    if !(*r).fp.is_null() {
        bgzf_close((*r).fp);
    }

    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut (*r).lock);

    drop(Box::from_raw(r));
}
pub unsafe fn cram_cram_io_c_2467_refs_create() -> *mut refs_t {
    let r = Box::into_raw(Box::new(refs_t_layout {
        pool: None,
        h_meta: std::ptr::null_mut(),
        ref_id: std::ptr::null_mut(),
        nref: 0,
        fn_: Vec::new(),
        fp: std::ptr::null_mut(),
        count: 1,
        lock: unsafe { std::mem::zeroed() },
        last: std::ptr::null_mut(),
        last_id: -1,
    }));

    (*r).pool = Some(cram_string_alloc_c_55_string_pool_create(8192));

    (*r).h_meta = calloc(1, std::mem::size_of::<kh_refs_layout>() as u64).cast::<kh_refs_layout>();
    if (*r).h_meta.is_null() {
        cram_cram_io_c_2427_refs_free(r.cast());
        return std::ptr::null_mut();
    }

    crate::htslib_rs::c_compat::pthread_mutex_init(&mut (*r).lock, std::ptr::null());

    r.cast()
}
pub unsafe fn cram_cram_io_c_2503_bgzf_open_ref(
    fn_in: &[u8],
    mode: *mut c_char,
    is_md5: c_int,
) -> *mut BGZF {
    let fn_bytes: &[u8] = if fn_in.starts_with(b"file://") {
        &fn_in[7..]
    } else {
        fn_in
    };
    let fn_cstring = std::ffi::CString::new(fn_bytes).unwrap();
    let fn_ = fn_cstring.as_ptr().cast_mut();

    if is_md5 == 0 && hisremote(fn_) == 0 {
        let mut fai_file = [0 as c_char; crate::htslib_rs::c_compat::PATH_MAX as usize];
        libc::snprintf(
            fai_file.as_mut_ptr(),
            crate::htslib_rs::c_compat::PATH_MAX as usize,
            c"%s.fai".as_ptr(),
            fn_,
        );
        if crate::htslib_rs::c_compat::access(fai_file.as_ptr(), crate::htslib_rs::c_compat::R_OK)
            != 0
            && fai_build(fn_) != 0
        {
            return std::ptr::null_mut();
        }
    }

    let fp = bgzf_open(fn_, mode);
    if fp.is_null() {
        libc::perror(fn_);
        return std::ptr::null_mut();
    }

    if ((*fp).bitfields & (1 << 30)) != 0 && bgzf_index_load(fp, fn_, c".gzi".as_ptr()) < 0 {
        let msg = std::ffi::CString::new(format!(
            "Unable to load .gzi index '{}.gzi'",
            CStr::from_ptr(fn_).to_string_lossy()
        ))
        .unwrap();
        hts_log_cstr(HTS_LOG_ERROR, c"bgzf_open_ref".as_ptr(), msg.as_ptr());
        bgzf_close(fp);
        return std::ptr::null_mut();
    }

    fp
}
pub unsafe fn cram_cram_io_c_2541_refs_load_fai(
    r_orig: *mut refs_t,
    fn_: *const c_char,
    is_err: c_int,
) -> *mut refs_t {
    let mut fai_fn = [0 as c_char; crate::htslib_rs::c_compat::PATH_MAX as usize];
    let mut line = [0 as c_char; 8192];
    let mut r = r_orig.cast::<refs_t_layout>();
    let fn_bytes = c_char_bytes(fn_).unwrap_or(&[]);
    let fn_l = fn_bytes.len();
    let mut id = 0i32;
    let mut id_alloc = 0i32;

    if r.is_null() {
        r = cram_cram_io_c_2467_refs_create().cast::<refs_t_layout>();
        if r.is_null() {
            return std::ptr::null_mut();
        }
    }

    if !(*r).fp.is_null() && bgzf_close((*r).fp) != 0 {
        if r_orig.is_null() {
            cram_cram_io_c_2427_refs_free(r.cast());
        }
        return std::ptr::null_mut();
    }
    (*r).fp = std::ptr::null_mut();

    if let Some(fn_delim_offset) = fn_bytes.windows(7).position(|window| window == b"##idx##") {
        (*r).fn_ = fn_bytes[..fn_delim_offset].to_vec();
        let idx = fn_.add(fn_delim_offset + 7);
        libc::snprintf(
            fai_fn.as_mut_ptr(),
            crate::htslib_rs::c_compat::PATH_MAX as usize,
            c"%s".as_ptr(),
            idx,
        );
    } else if fn_bytes.ends_with(b".fai") {
        if (*r).fn_.is_empty() {
            (*r).fn_ = fn_bytes[..fn_l - 4].to_vec();
        }
        libc::snprintf(
            fai_fn.as_mut_ptr(),
            crate::htslib_rs::c_compat::PATH_MAX as usize,
            c"%s".as_ptr(),
            fn_,
        );
    } else {
        (*r).fn_ = fn_bytes.to_vec();
        libc::snprintf(
            fai_fn.as_mut_ptr(),
            crate::htslib_rs::c_compat::PATH_MAX as usize,
            c"%.*s.fai".as_ptr(),
            crate::htslib_rs::c_compat::PATH_MAX - 5,
            fn_,
        );
    }

    (*r).fp = cram_cram_io_c_2503_bgzf_open_ref(&(*r).fn_, c"r".as_ptr().cast_mut(), 0);
    if (*r).fp.is_null() {
        let msg = std::ffi::CString::new(format!(
            "Failed to open reference file '{}'",
            String::from_utf8_lossy(&(*r).fn_)
        ))
        .unwrap();
        hts_log_cstr(HTS_LOG_ERROR, c"refs_load_fai".as_ptr(), msg.as_ptr());
        if r_orig.is_null() {
            cram_cram_io_c_2427_refs_free(r.cast());
        }
        return std::ptr::null_mut();
    }

    let fp = hopen(fai_fn.as_ptr(), c"r".as_ptr());
    if fp.is_null() {
        let msg = std::ffi::CString::new(format!(
            "Failed to open index file '{}'",
            CStr::from_ptr(fai_fn.as_ptr()).to_string_lossy()
        ))
        .unwrap();
        hts_log_cstr(HTS_LOG_ERROR, c"refs_load_fai".as_ptr(), msg.as_ptr());
        if is_err != 0 {
            libc::perror(fai_fn.as_ptr());
        }
        if r_orig.is_null() {
            cram_cram_io_c_2427_refs_free(r.cast());
        }
        return std::ptr::null_mut();
    }

    while !hgets(line.as_mut_ptr(), 8192, fp).is_null() {
        let e = Box::into_raw(Box::new(ref_entry_layout {
            name: Vec::new(),
            fn_: Vec::new(),
            length: 0,
            ln_length: 0,
            offset: 0,
            bases_per_line: 0,
            line_length: 0,
            count: 0,
            seq: Vec::new(),
            mf: std::ptr::null_mut(),
            is_md5: 0,
            validated_md5: 0,
        }));

        let mut cp = line.as_mut_ptr();
        while *cp != 0 && isspace_c(*cp) == 0 {
            cp = cp.add(1);
        }
        *cp = 0;
        cp = cp.add(1);
        (*e).name = c_char_bytes(line.as_ptr()).unwrap_or(&[]).to_vec();

        while *cp != 0 && isspace_c(*cp) != 0 {
            cp = cp.add(1);
        }
        (*e).length = libc::strtoll(cp, &mut cp, 10);

        while *cp != 0 && isspace_c(*cp) != 0 {
            cp = cp.add(1);
        }
        (*e).offset = libc::strtoll(cp, &mut cp, 10);

        while *cp != 0 && isspace_c(*cp) != 0 {
            cp = cp.add(1);
        }
        (*e).bases_per_line = libc::strtol(cp, &mut cp, 10) as c_int;

        while *cp != 0 && isspace_c(*cp) != 0 {
            cp = cp.add(1);
        }
        (*e).line_length = libc::strtol(cp, &mut cp, 10) as c_int;
        (*e).fn_ = (*r).fn_.clone();
        (*e).count = 0;
        (*e).is_md5 = 0;
        (*e).validated_md5 = 0;

        if (*e).name.is_empty() {
            drop(Box::from_raw(e));
            hclose_abruptly(fp);
            if r_orig.is_null() {
                cram_cram_io_c_2427_refs_free(r.cast());
            }
            return std::ptr::null_mut();
        }

        let h = (*r).h_meta;
        if (*h).n_occupied >= (*h).upper_bound {
            let mut new_n_buckets = (*h).n_buckets + 1;
            new_n_buckets = new_n_buckets.wrapping_sub(1);
            new_n_buckets |= new_n_buckets >> 1;
            new_n_buckets |= new_n_buckets >> 2;
            new_n_buckets |= new_n_buckets >> 4;
            new_n_buckets |= new_n_buckets >> 8;
            new_n_buckets |= new_n_buckets >> 16;
            new_n_buckets = new_n_buckets.wrapping_add(1);
            if new_n_buckets < 4 {
                new_n_buckets = 4;
            }

            let flags_words = if new_n_buckets < 16 {
                1
            } else {
                new_n_buckets >> 4
            };
            let new_flags =
                malloc((flags_words as usize * std::mem::size_of::<u32>()) as u64).cast::<u32>();
            let new_keys = calloc(
                new_n_buckets as u64,
                std::mem::size_of::<*const c_char>() as u64,
            )
            .cast::<*const c_char>();
            let new_vals = calloc(
                new_n_buckets as u64,
                std::mem::size_of::<*mut ref_entry_layout>() as u64,
            )
            .cast::<*mut ref_entry_layout>();
            if new_flags.is_null() || new_keys.is_null() || new_vals.is_null() {
                free(new_flags.cast());
                free(new_keys.cast());
                free(new_vals.cast());
                free(e.cast());
                hclose_abruptly(fp);
                if r_orig.is_null() {
                    cram_cram_io_c_2427_refs_free(r.cast());
                }
                return std::ptr::null_mut();
            }
            for x in 0..flags_words {
                *new_flags.add(x as usize) = 0xaaaa_aaaa;
            }

            for old in 0..(*h).n_buckets {
                if ((*(*h).flags.add((old >> 4) as usize) >> ((old & 0x0f) << 1)) & 3) != 0 {
                    continue;
                }
                let key = *(*h).keys.add(old as usize);
                let val = *(*h).vals.add(old as usize);
                let mut hash = 2166136261u32;
                for &b in (*val).name.iter() {
                    hash = (hash ^ (b as u32)).wrapping_mul(16777619);
                }
                let mut i = hash & (new_n_buckets - 1);
                let mut step = 0u32;
                while ((*new_flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) == 0 {
                    step += 1;
                    i = (i + step) & (new_n_buckets - 1);
                }
                *new_keys.add(i as usize) = key;
                *new_vals.add(i as usize) = val;
                *new_flags.add((i >> 4) as usize) &= !(3u32 << ((i & 0x0f) << 1));
            }

            free((*h).flags.cast());
            free((*h).keys.cast());
            free((*h).vals.cast());
            (*h).flags = new_flags;
            (*h).keys = new_keys;
            (*h).vals = new_vals;
            (*h).n_buckets = new_n_buckets;
            (*h).n_occupied = (*h).size;
            (*h).upper_bound = ((*h).n_buckets as f64 * 0.77 + 0.5) as u32;
        }

        let mut ret = 0;
        let mut hash = 2166136261u32;
        for &b in (*e).name.iter() {
            hash = (hash ^ (b as u32)).wrapping_mul(16777619);
        }
        let mask = (*h).n_buckets - 1;
        let mut x = (*h).n_buckets;
        let mut site = (*h).n_buckets;
        let mut i = hash & mask;
        if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) != 0 {
            x = i;
        } else {
            let last = i;
            let mut step = 0u32;
            while ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) == 0
                && (((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 1) != 0
                    || (*(*(*h).vals.add(i as usize))).name.as_slice() != (*e).name.as_slice())
            {
                if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 1) != 0 {
                    site = i;
                }
                step += 1;
                i = (i + step) & mask;
                if i == last {
                    x = site;
                    break;
                }
            }
            if x == (*h).n_buckets {
                if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) != 0
                    && site != (*h).n_buckets
                {
                    x = site;
                } else {
                    x = i;
                }
            }
        }

        if ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) != 0 {
            *(*h).keys.add(x as usize) = (*e).name.as_ptr().cast();
            *(*h).flags.add((x >> 4) as usize) &= !(3u32 << ((x & 0x0f) << 1));
            (*h).size += 1;
            (*h).n_occupied += 1;
            ret = 1;
        } else if ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 1) != 0 {
            *(*h).keys.add(x as usize) = (*e).name.as_ptr().cast();
            *(*h).flags.add((x >> 4) as usize) &= !(3u32 << ((x & 0x0f) << 1));
            (*h).size += 1;
            ret = 2;
        }

        if ret != 0 {
            *(*h).vals.add(x as usize) = e;
        } else {
            let re = *(*h).vals.add(x as usize);
            if !re.is_null() && ((*re).count != 0 || (*re).length != 0) {
                free(e.cast());
            } else {
                if !re.is_null() {
                    free(re.cast());
                }
                *(*h).vals.add(x as usize) = e;
            }
        }

        if id >= id_alloc {
            id_alloc = if id_alloc != 0 { id_alloc * 2 } else { 16 };
            let new_refs = realloc(
                (*r).ref_id.cast(),
                (id_alloc as usize * std::mem::size_of::<*mut ref_entry_layout>()) as u64,
            )
            .cast::<*mut ref_entry_layout>();
            if new_refs.is_null() {
                hclose_abruptly(fp);
                if r_orig.is_null() {
                    cram_cram_io_c_2427_refs_free(r.cast());
                }
                return std::ptr::null_mut();
            }
            (*r).ref_id = new_refs;
            for x in id..id_alloc {
                *(*r).ref_id.add(x as usize) = std::ptr::null_mut();
            }
        }
        *(*r).ref_id.add(id as usize) = e;
        id += 1;
        (*r).nref = id;
    }

    if hclose(fp) < 0 {
        if r_orig.is_null() {
            cram_cram_io_c_2427_refs_free(r.cast());
        }
        return std::ptr::null_mut();
    }

    r.cast()
}
pub unsafe fn cram_cram_io_c_2693_sanitise_SQ_lines(fd: *mut cram_fd) {
    let fd = fd.cast::<cram_fd_layout>();
    if (*fd).header.is_null() || (*(*fd).header.cast::<sam_hdr_t>()).hrecs.is_null() {
        return;
    }

    if (*fd).refs.is_null() || (*(*fd).refs.cast::<refs_t_layout>()).h_meta.is_null() {
        return;
    }

    let hdr = (*fd).header.cast::<sam_hdr_t>();
    let hrecs = (*hdr).hrecs;
    let refs = (*fd).refs.cast::<refs_t_layout>();
    let h = (*refs).h_meta;
    for iref in 0..(*hrecs).nref {
        let name = (*(*hrecs).ref_.add(iref as usize)).name;
        let mut k = (*h).n_buckets;
        if (*h).n_buckets != 0 {
            let mut hash = 2166136261u32;
            let mut p = name;
            while *p != 0 {
                hash = (hash ^ (*p as u8 as u32)).wrapping_mul(16777619);
                p = p.add(1);
            }
            let mask = (*h).n_buckets - 1;
            let mut x = hash & mask;
            let last = x;
            let mut step = 0u32;
            while ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) == 0
                && (((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 1) != 0
                    || (*(*(*h).vals.add(x as usize))).name.as_slice()
                        != c_char_bytes(name).unwrap_or(&[]))
            {
                step += 1;
                x = (x + step) & mask;
                if x == last {
                    break;
                }
            }
            if ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 3) == 0 {
                k = x;
            }
        }
        if k == (*h).n_buckets {
            continue;
        }

        let r = *(*h).vals.add(k as usize);
        if r.is_null() {
            continue;
        }

        if (*r).length != 0 && (*r).length != (*(*hrecs).ref_.add(iref as usize)).len {
            assert_eq!(
                c_char_bytes((*(*hrecs).ref_.add(iref as usize)).name).unwrap_or(&[]),
                (*r).name.as_slice(),
            );
            let msg = std::ffi::CString::new(format!(
                "Header @SQ length mismatch for ref {}, {} vs {}",
                String::from_utf8_lossy(&(*r).name),
                (*(*hrecs).ref_.add(iref as usize)).len,
                (*r).length as c_int,
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_WARNING, c"sanitise_SQ_lines".as_ptr(), msg.as_ptr());
            (*(*hrecs).ref_.add(iref as usize)).len = (*r).length;
        }
    }
}
pub unsafe fn cram_cram_io_c_2737_refs2id(r: *mut refs_t, hdr: *mut sam_hdr_t) -> c_int {
    let r = r.cast::<refs_t_layout>();
    let hrec = (*hdr).hrecs;

    if !(*r).ref_id.is_null() {
        free((*r).ref_id.cast());
    }
    if !(*r).last.is_null() {
        (*r).last = std::ptr::null_mut();
    }

    (*r).ref_id = calloc(
        (*hrec).nref as u64,
        std::mem::size_of::<*mut ref_entry_layout>() as u64,
    )
    .cast::<*mut ref_entry_layout>();
    if (*r).ref_id.is_null() {
        return -1;
    }

    (*r).nref = (*hrec).nref;
    let h = (*r).h_meta;
    for iref in 0..(*hrec).nref {
        let name = (*(*hrec).ref_.add(iref as usize)).name;
        let mut k = (*h).n_buckets;
        if (*h).n_buckets != 0 {
            let mut hash = 2166136261u32;
            let mut p = name;
            while *p != 0 {
                hash = (hash ^ (*p as u8 as u32)).wrapping_mul(16777619);
                p = p.add(1);
            }
            let mask = (*h).n_buckets - 1;
            let mut x = hash & mask;
            let last = x;
            let mut step = 0u32;
            while ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) == 0
                && (((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 1) != 0
                    || (*(*(*h).vals.add(x as usize))).name.as_slice()
                        != c_char_bytes(name).unwrap_or(&[]))
            {
                step += 1;
                x = (x + step) & mask;
                if x == last {
                    break;
                }
            }
            if ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 3) == 0 {
                k = x;
            }
        }

        if k != (*h).n_buckets {
            *(*r).ref_id.add(iref as usize) = *(*h).vals.add(k as usize);
        } else {
            let msg = std::ffi::CString::new(format!(
                "Unable to find ref name '{}'",
                CStr::from_ptr(name).to_string_lossy()
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_WARNING, c"refs2id".as_ptr(), msg.as_ptr());
        }
    }

    0
}
pub unsafe fn cram_cram_io_c_2852_cram_set_header2(
    fd: *mut cram_fd,
    hdr: *const sam_hdr_t,
) -> c_int {
    if fd.is_null() || hdr.is_null() {
        return -1;
    }

    let fd = fd.cast::<cram_fd_layout>();
    if (*fd).header != hdr.cast_mut().cast() {
        if !(*fd).header.is_null() {
            sam_hdr_destroy((*fd).header.cast());
        }
        (*fd).header = sam_hdr_dup(hdr).cast();
        if (*fd).header.is_null() {
            return -1;
        }
    }

    cram_cram_io_c_2768_refs_from_header(fd.cast())
}
pub unsafe fn cram_cram_io_c_2866_cram_set_header(fd: *mut cram_fd, hdr: *mut sam_hdr_t) -> c_int {
    cram_cram_io_c_2852_cram_set_header2(fd, hdr)
}
pub unsafe fn cram_cram_io_c_2768_refs_from_header(fd: *mut cram_fd) -> c_int {
    if fd.is_null() {
        return -1;
    }

    let fd = fd.cast::<cram_fd_layout>();
    let r = (*fd).refs.cast::<refs_t_layout>();
    if r.is_null() {
        return -1;
    }

    let h = (*fd).header.cast::<sam_hdr_t>();
    if h.is_null() {
        return 0;
    }

    if (*h).hrecs.is_null() && crate::htslib_rs::sam::sam_hdr_fill_hrecs(&mut *h) == -1 {
        return -1;
    }

    let hrecs = (*h).hrecs;
    if (*hrecs).nref == 0 {
        return 0;
    }

    let new_ref_id = realloc(
        (*r).ref_id.cast(),
        (((*r).nref + (*hrecs).nref) as usize * std::mem::size_of::<*mut ref_entry_layout>())
            as u64,
    )
    .cast::<*mut ref_entry_layout>();
    if new_ref_id.is_null() {
        return -1;
    }
    (*r).ref_id = new_ref_id;

    let mut j = (*r).nref;
    for i in 0..(*hrecs).nref {
        let h_ref = (*hrecs).ref_.add(i as usize);
        let name = (*h_ref).name;

        let kh = (*r).h_meta;
        let mut k = (*kh).n_buckets;
        if (*kh).n_buckets != 0 {
            let mut hash = 2166136261u32;
            let mut p = name;
            while *p != 0 {
                hash = (hash ^ (*p as u8 as u32)).wrapping_mul(16777619);
                p = p.add(1);
            }
            let mask = (*kh).n_buckets - 1;
            let mut x = hash & mask;
            let last = x;
            let mut step = 0u32;
            while ((*(*kh).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) == 0
                && (((*(*kh).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 1) != 0
                    || (*(*(*kh).vals.add(x as usize))).name.as_slice()
                        != c_char_bytes(name).unwrap_or(&[]))
            {
                step += 1;
                x = (x + step) & mask;
                if x == last {
                    break;
                }
            }
            if ((*(*kh).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 3) == 0 {
                k = x;
            }
        }
        if k != (*kh).n_buckets {
            continue;
        }

        let e =
            calloc(1, std::mem::size_of::<ref_entry_layout>() as u64).cast::<ref_entry_layout>();
        if e.is_null() {
            return -1;
        }
        *(*r).ref_id.add(j as usize) = e;

        if name.is_null() {
            return -1;
        }

        (*e).name = cram_string_alloc_c_149_string_dup(
            (*r).pool.as_deref_mut().unwrap(),
            c_char_bytes(name).unwrap_or(&[]),
        )
        .map(|s| s[..s.len().saturating_sub(1)].to_vec())
        .unwrap_or_default();
        if (*e).name.is_empty() {
            return -1;
        }
        (*e).length = 0;

        if !(*h_ref).ty.is_null() {
            let tag = crate::htslib_rs::sam::sam_hrecs_find_key(
                &mut *(*h_ref).ty.cast::<crate::htslib_rs::sam::sam_hrec_type_t>(),
                c"M5",
            )
            .0
            .map(|p| p.as_ptr().cast::<sam_hrec_tag_layout>())
            .unwrap_or(std::ptr::null_mut());
            if !tag.is_null() {
                (*e).fn_ = cram_string_alloc_c_149_string_dup(
                    (*r).pool.as_deref_mut().unwrap(),
                    c_char_bytes((*tag).str_.add(3)).unwrap_or(&[]),
                )
                .map(|s| s[..s.len().saturating_sub(1)].to_vec())
                .unwrap_or_default();
            }

            let tag = crate::htslib_rs::sam::sam_hrecs_find_key(
                &mut *(*h_ref).ty.cast::<crate::htslib_rs::sam::sam_hrec_type_t>(),
                c"LN",
            )
            .0
            .map(|p| p.as_ptr().cast::<sam_hrec_tag_layout>())
            .unwrap_or(std::ptr::null_mut());
            if !tag.is_null() {
                (*e).ln_length = libc::strtoll((*tag).str_.add(3), std::ptr::null_mut(), 0);
                if (*e).ln_length < 0 {
                    (*e).ln_length = 0;
                }
            }
        }

        if (*kh).n_occupied >= (*kh).upper_bound {
            let mut new_n_buckets = (*kh).n_buckets + 1;
            new_n_buckets = new_n_buckets.wrapping_sub(1);
            new_n_buckets |= new_n_buckets >> 1;
            new_n_buckets |= new_n_buckets >> 2;
            new_n_buckets |= new_n_buckets >> 4;
            new_n_buckets |= new_n_buckets >> 8;
            new_n_buckets |= new_n_buckets >> 16;
            new_n_buckets = new_n_buckets.wrapping_add(1);
            if new_n_buckets < 4 {
                new_n_buckets = 4;
            }

            let flags_words = if new_n_buckets < 16 {
                1
            } else {
                new_n_buckets >> 4
            };
            let new_flags =
                malloc((flags_words as usize * std::mem::size_of::<u32>()) as u64).cast::<u32>();
            let new_keys = calloc(
                new_n_buckets as u64,
                std::mem::size_of::<*const c_char>() as u64,
            )
            .cast::<*const c_char>();
            let new_vals = calloc(
                new_n_buckets as u64,
                std::mem::size_of::<*mut ref_entry_layout>() as u64,
            )
            .cast::<*mut ref_entry_layout>();
            if new_flags.is_null() || new_keys.is_null() || new_vals.is_null() {
                free(new_flags.cast());
                free(new_keys.cast());
                free(new_vals.cast());
                return -1;
            }
            for x in 0..flags_words {
                *new_flags.add(x as usize) = 0xaaaa_aaaa;
            }

            for old in 0..(*kh).n_buckets {
                if ((*(*kh).flags.add((old >> 4) as usize) >> ((old & 0x0f) << 1)) & 3) != 0 {
                    continue;
                }
                let key = *(*kh).keys.add(old as usize);
                let val = *(*kh).vals.add(old as usize);
                let mut hash = 2166136261u32;
                for &b in (*val).name.iter() {
                    hash = (hash ^ (b as u32)).wrapping_mul(16777619);
                }
                let mut x = hash & (new_n_buckets - 1);
                let mut step = 0u32;
                while ((*new_flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) == 0 {
                    step += 1;
                    x = (x + step) & (new_n_buckets - 1);
                }
                *new_keys.add(x as usize) = key;
                *new_vals.add(x as usize) = val;
                *new_flags.add((x >> 4) as usize) &= !(3u32 << ((x & 0x0f) << 1));
            }

            free((*kh).flags.cast());
            free((*kh).keys.cast());
            free((*kh).vals.cast());
            (*kh).flags = new_flags;
            (*kh).keys = new_keys;
            (*kh).vals = new_vals;
            (*kh).n_buckets = new_n_buckets;
            (*kh).n_occupied = (*kh).size;
            (*kh).upper_bound = ((*kh).n_buckets as f64 * 0.77 + 0.5) as u32;
        }

        let mut ret = 0;
        let mut hash = 2166136261u32;
        for &b in (*e).name.iter() {
            hash = (hash ^ (b as u32)).wrapping_mul(16777619);
        }
        let mask = (*kh).n_buckets - 1;
        let mut x = (*kh).n_buckets;
        let mut site = (*kh).n_buckets;
        let mut pos = hash & mask;
        if ((*(*kh).flags.add((pos >> 4) as usize) >> ((pos & 0x0f) << 1)) & 2) != 0 {
            x = pos;
        } else {
            let last = pos;
            let mut step = 0u32;
            while ((*(*kh).flags.add((pos >> 4) as usize) >> ((pos & 0x0f) << 1)) & 2) == 0
                && (((*(*kh).flags.add((pos >> 4) as usize) >> ((pos & 0x0f) << 1)) & 1) != 0
                    || (*(*(*kh).vals.add(pos as usize))).name.as_slice() != (*e).name.as_slice())
            {
                if ((*(*kh).flags.add((pos >> 4) as usize) >> ((pos & 0x0f) << 1)) & 1) != 0 {
                    site = pos;
                }
                step += 1;
                pos = (pos + step) & mask;
                if pos == last {
                    x = site;
                    break;
                }
            }
            if x == (*kh).n_buckets {
                if ((*(*kh).flags.add((pos >> 4) as usize) >> ((pos & 0x0f) << 1)) & 2) != 0
                    && site != (*kh).n_buckets
                {
                    x = site;
                } else {
                    x = pos;
                }
            }
        }

        if ((*(*kh).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) != 0 {
            *(*kh).keys.add(x as usize) = (*e).name.as_ptr().cast();
            *(*kh).flags.add((x >> 4) as usize) &= !(3u32 << ((x & 0x0f) << 1));
            (*kh).size += 1;
            (*kh).n_occupied += 1;
            ret = 1;
        } else if ((*(*kh).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 1) != 0 {
            *(*kh).keys.add(x as usize) = (*e).name.as_ptr().cast();
            *(*kh).flags.add((x >> 4) as usize) &= !(3u32 << ((x & 0x0f) << 1));
            (*kh).size += 1;
            ret = 2;
        }
        if ret <= 0 {
            return -1;
        }
        *(*kh).vals.add(x as usize) = e;
        j += 1;
    }
    (*r).nref = j;

    0
}
pub unsafe fn cram_cram_io_c_3169_cram_ref_incr_locked(r: *mut refs_t, id: c_int) {
    let r = r.cast::<refs_t_layout>();
    if id < 0
        || (*(*r).ref_id.add(id as usize)).is_null()
        || (*(*(*r).ref_id.add(id as usize))).seq.is_empty()
    {
        return;
    }

    if (*r).last_id == id {
        (*r).last_id = -1;
    }

    (*(*(*r).ref_id.add(id as usize))).count += 1;
}
pub unsafe fn cram_cram_io_c_3183_cram_ref_incr(r: *mut refs_t, id: c_int) {
    let rl = r.cast::<refs_t_layout>();
    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*rl).lock);
    cram_cram_io_c_3169_cram_ref_incr_locked(r, id);
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*rl).lock);
}
pub unsafe fn cram_cram_io_c_3189_cram_ref_decr_locked(r: *mut refs_t, id: c_int) {
    let r = r.cast::<refs_t_layout>();
    if id < 0
        || (*(*r).ref_id.add(id as usize)).is_null()
        || (*(*(*r).ref_id.add(id as usize))).seq.is_empty()
    {
        return;
    }

    let e = *(*r).ref_id.add(id as usize);
    (*e).count -= 1;
    if (*e).count <= 0 {
        assert_eq!((*e).count, 0);
        if (*r).last_id >= 0 {
            let last = *(*r).ref_id.add((*r).last_id as usize);
            if (*last).count <= 0 && !(*last).seq.is_empty() {
                cram_cram_io_c_2417_ref_entry_free_seq(last.cast());
                if (*last).is_md5 != 0 {
                    (*last).length = 0;
                }
            }
        }
        (*r).last_id = id;
    }
}
pub unsafe fn cram_cram_io_c_3213_cram_ref_decr(r: *mut refs_t, id: c_int) {
    let rl = r.cast::<refs_t_layout>();
    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*rl).lock);
    cram_cram_io_c_3189_cram_ref_decr_locked(r, id);
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*rl).lock);
}
pub unsafe fn cram_cram_io_c_3228_load_ref_portion(
    fp: *mut BGZF,
    e: *mut c_void,
    start: i64,
    mut end: i64,
) -> *mut c_char {
    let e = e.cast::<ref_entry_layout>();

    if end < start {
        end = start;
    }

    let offset = if (*e).line_length != 0 {
        (*e).offset
            + (start - 1) / (*e).bases_per_line as i64 * (*e).line_length as i64
            + (start - 1) % (*e).bases_per_line as i64
    } else {
        start - 1
    };

    let len = (if (*e).line_length != 0 {
        (*e).offset
            + (end - 1) / (*e).bases_per_line as i64 * (*e).line_length as i64
            + (end - 1) % (*e).bases_per_line as i64
    } else {
        end - 1
    }) - offset
        + 1;

    if bgzf_useek(fp, offset, libc::SEEK_SET) < 0 {
        libc::perror(c"bgzf_useek() on reference file".as_ptr());
        return std::ptr::null_mut();
    }

    if len == 0 {
        return std::ptr::null_mut();
    }
    let seq = malloc(len as u64).cast::<c_char>();
    if seq.is_null() {
        return std::ptr::null_mut();
    }

    if bgzf_read(fp, seq.cast(), len as usize) != len as isize {
        libc::perror(c"bgzf_read() on reference file".as_ptr());
        free(seq.cast());
        return std::ptr::null_mut();
    }

    if len != end - start + 1 {
        let mut i = 0i64;
        let mut j = 0i64;
        while i < len {
            let ch = *seq.add(i as usize);
            if isspace_c(ch) == 0 {
                *seq.add(j as usize) = ((ch as u8) & !0x20) as c_char;
                j += 1;
            } else {
                break;
            }
            i += 1;
        }
        while i < len && isspace_c(*seq.add(i as usize)) != 0 {
            i += 1;
        }
        while i < len - (*e).line_length as i64 {
            let j_end = j + (*e).bases_per_line as i64;
            while j < j_end {
                *seq.add(j as usize) = ((*seq.add(i as usize) as u8) & !0x20) as c_char;
                j += 1;
                i += 1;
            }
            i += ((*e).line_length - (*e).bases_per_line) as i64;
        }
        while i < len {
            let ch = *seq.add(i as usize);
            if isspace_c(ch) == 0 {
                *seq.add(j as usize) = ((ch as u8) & !0x20) as c_char;
                j += 1;
            }
            i += 1;
        }

        if j != end - start + 1 {
            hts_log_cstr(
                HTS_LOG_ERROR,
                c"load_ref_portion".as_ptr(),
                c"Malformed reference file".as_ptr(),
            );
            free(seq.cast());
            return std::ptr::null_mut();
        }
    } else {
        for i in 0..len {
            *seq.add(i as usize) = toupper_c(*seq.add(i as usize));
        }
    }

    seq
}
pub unsafe fn cram_cram_io_c_3323_cram_ref_load(
    r: *mut refs_t,
    id: c_int,
    is_md5: c_int,
) -> *mut c_void {
    let r = r.cast::<refs_t_layout>();
    let e = *(*r).ref_id.add(id as usize);
    let start = 1i64;
    let end = (*e).length;

    if !(*e).seq.is_empty() {
        return e.cast();
    }

    assert_eq!((*e).count, 0);

    if !(*r).last.is_null() {
        assert!((*(*r).last).count > 0);
        (*(*r).last).count -= 1;
        if (*(*r).last).count <= 0 && !(*(*r).last).seq.is_empty() {
            cram_cram_io_c_2417_ref_entry_free_seq((*r).last.cast());
        }
    }

    if (*r).fn_.is_empty() {
        return std::ptr::null_mut();
    }

    if (*r).fn_ != (*e).fn_ || (*r).fp.is_null() {
        if !(*r).fp.is_null() && bgzf_close((*r).fp) != 0 {
            return std::ptr::null_mut();
        }
        (*r).fn_ = (*e).fn_.clone();
        (*r).fp = cram_cram_io_c_2503_bgzf_open_ref(&(*r).fn_, c"r".as_ptr().cast_mut(), is_md5);
        if (*r).fp.is_null() {
            return std::ptr::null_mut();
        }
    }

    let seq = cram_cram_io_c_3228_load_ref_portion((*r).fp, e.cast(), start, end);
    if seq.is_null() {
        return std::ptr::null_mut();
    }

    // load_ref_portion still returns a malloc'd C buffer of exactly end-start+1
    // bytes (no NUL); copy it into the owned Vec<u8> seq and release the raw buffer.
    let seq_len = (end - start + 1) as usize;
    (*e).seq = std::slice::from_raw_parts(seq.cast::<u8>(), seq_len).to_vec();
    crate::htslib_rs::c_compat::free(seq.cast());
    (*e).mf = std::ptr::null_mut();
    (*e).count += 1;
    (*r).last = e;
    (*e).count += 1;

    e.cast()
}
// original: cram_populate_ref (htslib/cram/cram_io.c:2979)
//
// Locates the on-disk reference for ref id `id` and, where possible, fills in
// the ref_entry so that cram_get_ref can read the bases via load_ref_portion.
//
// Mirrors the HAVE_MMAP-defined build of htslib (the default): the
// `#ifndef HAVE_MMAP` REF_PATH `find_path` shortcut is omitted, and we rely on
// the native `open_path_mfile` to load full sequences from REF_PATH/REF_CACHE.
struct RefCacheTmpPath {
    inner: kstring_t,
}

impl RefCacheTmpPath {
    fn new() -> Self {
        Self {
            inner: kstring_t { data: Vec::new() },
        }
    }

    fn as_mut_ptr(&mut self) -> *mut kstring_t {
        &mut self.inner
    }

    // FFI boundary: build a NUL-terminated C string from the owned bytes.
    fn c_path(&self) -> std::ffi::CString {
        std::ffi::CString::new(self.inner.data.clone()).unwrap_or_default()
    }

    unsafe fn unlink(&self) {
        libc::unlink(self.c_path().as_ptr());
    }
}

impl Drop for RefCacheTmpPath {
    fn drop(&mut self) {
        crate::htslib_rs::hts::ks_free(&mut self.inner);
    }
}

pub unsafe fn cram_cram_io_c_2977_cram_populate_ref(
    fd: *mut cram_fd,
    id: c_int,
    r: *mut c_void,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let r = r.cast::<ref_entry_layout>();
    let ref_path = libc::getenv(c"REF_PATH".as_ptr());
    let local_cache = libc::getenv(c"REF_CACHE".as_ptr());
    let mut path = [0i8; crate::htslib_rs::c_compat::PATH_MAX as usize];
    let mut path_tmp = RefCacheTmpPath::new();
    let mut local_path = 0i32;

    {
        let msg = std::ffi::CString::new(format!(
            "Running cram_populate_ref on fd {:p}, id {}",
            fd, id
        ))
        .unwrap();
        hts_log_cstr(HTS_LOG_INFO, c"cram_populate_ref".as_ptr(), msg.as_ptr());
    }

    if (*r).name.is_empty() {
        return -1;
    }

    let r_name_c = std::ffi::CString::new((*r).name.clone()).unwrap();
    let hrecs = (*(*fdl).header.cast::<sam_hdr_t>()).hrecs;
    let ty = match crate::htslib_rs::sam::sam_hrecs_find_type_id(
        &mut *hrecs,
        c"SQ",
        Some((c"SN", r_name_c.as_c_str())),
    ) {
        Some(ty) => ty.as_ptr(),
        None => return -1,
    };

    let m5tag =
        crate::htslib_rs::sam::sam_hrecs_find_key(&mut *ty, c"M5")
            .0
            .map_or(std::ptr::null_mut(), |t| t.as_ptr().cast::<sam_hrec_tag_layout>());

    // `'no_M5` block models C's `goto no_M5;` target.
    let from_m5: bool = !m5tag.is_null();
    if from_m5 {
        let m5 = (*m5tag).str_.add(3);
        {
            let msg = std::ffi::CString::new(format!(
                "Querying ref {}",
                CStr::from_ptr(m5).to_string_lossy()
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_INFO, c"cram_populate_ref".as_ptr(), msg.as_ptr());
        }

        // Use cache if available.
        if !local_cache.is_null() && *local_cache != 0 {
            let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
            if cram_cram_io_c_2884_expand_cache_path(path.as_mut_ptr(), local_cache, m5) == 0
                && libc::stat(path.as_ptr(), sb.as_mut_ptr()) == 0
            {
                local_path = 1;
            }
        }

        // Found via REF_CACHE: open it and fall back to cram_get_ref().
        if local_path != 0 {
            let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
            if libc::stat(path.as_ptr(), sb.as_mut_ptr()) == 0 {
                let sb = sb.assume_init();
                if crate::htslib_rs::c_compat::stat_mode_matches(
                    sb.st_mode,
                    libc::S_IFMT,
                    libc::S_IFREG,
                ) {
                    let fp = bgzf_open(path.as_ptr(), c"r".as_ptr());
                    if !fp.is_null() {
                        (*r).length = sb.st_size;
                        (*r).offset = 0;
                        (*r).line_length = 0;
                        (*r).bases_per_line = 0;
                        (*r).fn_ = c_char_bytes(path.as_ptr()).unwrap_or(&[]).to_vec();
                        let refs = (*fdl).refs.cast::<refs_t_layout>();
                        if !(*refs).fp.is_null() && bgzf_close((*refs).fp) != 0 {
                            return -1;
                        }
                        (*refs).fp = fp;
                        (*refs).fn_ = (*r).fn_.clone();
                        (*r).is_md5 = 1;
                        (*r).validated_md5 = 1;
                        return 0;
                    }
                }
            }
        }

        // Otherwise search full REF_PATH; slower as it loads the entire file.
        let ref_path_bytes = if ref_path.is_null() {
            None
        } else {
            Some(CStr::from_ptr(ref_path).to_bytes())
        };
        let opened = cram_open_trace_file_c_352_open_path_mfile(
            CStr::from_ptr(m5).to_bytes(),
            ref_path_bytes,
            None,
        );
        if let Some(opened) = opened {
            let is_local = if opened.local { 1i32 } else { 0i32 };
            let mf = opened.mf.as_ptr();
            let mut sz: usize = 0;
            let stolen = cram_mFILE_c_428_mfsteal(mf, &mut sz).cast::<u8>();
            if !stolen.is_null() {
                (*r).seq = Vec::from_raw_parts(stolen, sz, sz);
                (*r).mf = std::ptr::null_mut();
            } else {
                // Couldn't detach; keep mf around.
                (*r).seq = (*mf).data.clone();
                (*r).mf = mf;
            }
            (*r).length = sz as i64;
            (*r).is_md5 = 1;
            (*r).validated_md5 = 1;

            // Populate the local disk cache if required.
            if is_local == 0 && !local_cache.is_null() && *local_cache != 0 {
                if cram_cram_io_c_2884_expand_cache_path(path.as_mut_ptr(), local_cache, m5) < 0 {
                    return 0; // Not fatal - we have the data already.
                }
                {
                    let msg = std::ffi::CString::new(format!(
                        "Writing cache file '{}'",
                        CStr::from_ptr(path.as_ptr()).to_string_lossy()
                    ))
                    .unwrap();
                    hts_log_cstr(HTS_LOG_INFO, c"cram_populate_ref".as_ptr(), msg.as_ptr());
                }
                cram_cram_io_c_2947_mkdir_prefix(path.as_mut_ptr(), 0o1777);

                let fp = crate::htslib_rs::hts::hts_open_tmpfile(
                    path.as_ptr(),
                    c"wx".as_ptr(),
                    path_tmp.as_mut_ptr(),
                );
                if fp.is_null() {
                    libc::perror(path_tmp.c_path().as_ptr());
                    return 0; // Not fatal.
                }

                // Verify md5sum (native MD5).
                let mut md5 = crate::htslib_rs::md5::hts_md5_init();
                let mut md5_buf1 = [0u8; 16];
                let mut md5_buf2 = [0u8; 33];
                crate::htslib_rs::md5::hts_md5_update(
                    &mut md5,
                    std::slice::from_raw_parts((*r).seq.as_ptr(), (*r).length as usize),
                    (*r).length as usize,
                );
                crate::htslib_rs::md5::hts_md5_final(&mut md5_buf1, &mut md5);
                crate::htslib_rs::md5::hts_md5_destroy(Some(md5));
                crate::htslib_rs::md5::hts_md5_hex(&mut md5_buf2, &md5_buf1);

                let m5_bytes = std::slice::from_raw_parts(m5.cast::<u8>(), 32);
                let md5_hex = &md5_buf2[..32];
                if m5_bytes != md5_hex {
                    hts_log_cstr(
                        HTS_LOG_ERROR,
                        c"cram_populate_ref".as_ptr(),
                        c"Mismatching md5sum for downloaded reference".as_ptr(),
                    );
                    hclose_abruptly(fp);
                    path_tmp.unlink();
                    return -1;
                }

                let length_written =
                    htslib_hfile_h_292_hwrite(fp, (*r).seq.as_ptr().cast(), (*r).length as usize);
                let path_tmp_c = path_tmp.c_path();
                if hclose(fp) < 0
                    || length_written != (*r).length as isize
                    || crate::htslib_rs::c_compat::chmod(path_tmp_c.as_ptr(), 0o444) < 0
                    || libc::rename(path_tmp_c.as_ptr(), path.as_ptr()) < 0
                {
                    let msg = std::ffi::CString::new(format!(
                        "Creating reference at {} failed: {}",
                        CStr::from_ptr(path.as_ptr()).to_string_lossy(),
                        CStr::from_ptr(libc::strerror(*__errno_location())).to_string_lossy()
                    ))
                    .unwrap();
                    hts_log_cstr(HTS_LOG_ERROR, c"cram_populate_ref".as_ptr(), msg.as_ptr());
                    path_tmp.unlink();
                }
            }

            return 0;
        }
    }

    // no_M5: failed to find in search path or M5 cache; try @SQ UR: tag.
    let ur_tag =
        crate::htslib_rs::sam::sam_hrecs_find_key(&mut *ty, c"UR")
            .0
            .map_or(std::ptr::null_mut(), |t| t.as_ptr().cast::<sam_hrec_tag_layout>());
    if ur_tag.is_null() {
        return -1;
    }

    let ur = (*ur_tag).str_.add(3);
    let ur_bytes = c_char_bytes(ur).unwrap_or(&[]);
    if ur_bytes.windows(3).any(|window| window == b"://") && !ur_bytes.starts_with(b"file:") {
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"cram_populate_ref".as_ptr(),
            c"UR tags pointing to remote files are not supported".as_ptr(),
        );
        return -1;
    }

    let fn_ = if ur_bytes.starts_with(b"file:") {
        ur.add(5)
    } else {
        ur
    };

    let refs0 = (*fdl).refs.cast::<refs_t_layout>();
    if !(*refs0).fp.is_null() {
        if bgzf_close((*refs0).fp) != 0 {
            return -1;
        }
        (*refs0).fp = std::ptr::null_mut();
    }

    let refs = cram_cram_io_c_2541_refs_load_fai((*fdl).refs.cast(), fn_, 0);
    if refs.is_null() {
        return -1;
    }
    cram_cram_io_c_2693_sanitise_SQ_lines(fd);

    (*fdl).refs = refs.cast();
    let refsl = (*fdl).refs.cast::<refs_t_layout>();
    if !(*refsl).fp.is_null() {
        if bgzf_close((*refsl).fp) != 0 {
            return -1;
        }
        (*refsl).fp = std::ptr::null_mut();
    }

    if (*refsl).fn_.is_empty() {
        return -1;
    }

    if cram_cram_io_c_2737_refs2id((*fdl).refs.cast(), (*fdl).header.cast()) == -1 {
        return -1;
    }
    if (*refsl).ref_id.is_null() || (*(*refsl).ref_id.add(id as usize)).is_null() {
        return -1;
    }

    // Local copy already, so fall back to cram_get_ref().
    0
}
// original: cram_get_ref (htslib/cram/cram_io.c:3411)
pub unsafe fn cram_cram_io_c_3409_cram_get_ref(
    fd: *mut cram_fd,
    id: c_int,
    mut start: i64,
    mut end: i64,
) -> *mut c_char {
    let fdl = fd.cast::<cram_fd_layout>();
    let ostart = start;

    if id == -1 || start < 1 {
        return std::ptr::null_mut();
    }

    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).ref_lock);

    // Unsorted data implies we want to fetch an entire reference at a time.
    if (*fdl).unsorted != 0 {
        (*fdl).shared_ref = 1;
    }

    // Sanity checking: does this ID exist?
    let refs = (*fdl).refs.cast::<refs_t_layout>();
    if (*fdl).refs.is_null()
        || id < 0
        || id >= (*refs).nref
        || (*(*refs).ref_id.add(id as usize)).is_null()
    {
        let msg = std::ffi::CString::new(format!("No reference found for id {}", id)).unwrap();
        hts_log_cstr(HTS_LOG_ERROR, c"cram_get_ref".as_ptr(), msg.as_ptr());
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
        return std::ptr::null_mut();
    }

    let mut r = *(*refs).ref_id.add(id as usize);

    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*refs).lock);
    if (*r).length == 0 {
        if !(*fdl).ref_fn.is_null() {
            let msg = std::ffi::CString::new(format!(
                "Reference file given, but ref '{}' not present",
                String::from_utf8_lossy(&(*r).name)
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_WARNING, c"cram_get_ref".as_ptr(), msg.as_ptr());
        }
        if cram_cram_io_c_2977_cram_populate_ref(fd, id, r.cast()) == -1 {
            let msg = std::ffi::CString::new(format!(
                "Failed to populate reference \"{}\"",
                String::from_utf8_lossy(&(*r).name)
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_WARNING, c"cram_get_ref".as_ptr(), msg.as_ptr());
            hts_log_cstr(
                HTS_LOG_WARNING,
                c"cram_get_ref".as_ptr(),
                c"See https://www.htslib.org/doc/reference_seqs.html for further suggestions"
                    .as_ptr(),
            );
            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*refs).lock);
            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
            return std::ptr::null_mut();
        }
        // cram_populate_ref may have replaced fd->refs.
        let refs = (*fdl).refs.cast::<refs_t_layout>();
        r = *(*refs).ref_id.add(id as usize);
        if (*fdl).unsorted != 0 {
            cram_cram_io_c_3169_cram_ref_incr_locked((*fdl).refs.cast(), id);
        }
    }

    // Re-read refs in case cram_populate_ref reassigned fd->refs.
    let refs = (*fdl).refs.cast::<refs_t_layout>();

    if end < 1 {
        end = (*r).length;
    }
    if end >= (*r).length {
        end = (*r).length;
    }

    if (end - start) as f64 >= 0.5 * (*r).length as f64 || (*fdl).shared_ref != 0 {
        start = 1;
        end = (*r).length;
    }

    if (*fdl).shared_ref != 0 || !(*r).seq.is_empty() || (start == 1 && end == (*r).length) {
        let cp: *mut c_char;
        if id >= 0 {
            if !(*r).seq.is_empty() {
                cram_cram_io_c_3169_cram_ref_incr_locked((*fdl).refs.cast(), id);
            } else {
                let e = cram_cram_io_c_3323_cram_ref_load((*fdl).refs.cast(), id, (*r).is_md5);
                if e.is_null() {
                    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*refs).lock);
                    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
                    return std::ptr::null_mut();
                }
                if (*fdl).unsorted != 0 {
                    cram_cram_io_c_3169_cram_ref_incr_locked((*fdl).refs.cast(), id);
                }
            }

            (*fdl).ref_ = std::ptr::null_mut();
            (*fdl).ref_start = 1;
            (*fdl).ref_end = (*r).length;
            (*fdl).ref_id = id;

            cp = (*(*(*refs).ref_id.add(id as usize)))
                .seq
                .as_ptr()
                .add((ostart - 1) as usize)
                .cast::<c_char>()
                .cast_mut();
        } else {
            (*fdl).ref_ = std::ptr::null_mut();
            cp = std::ptr::null_mut();
        }

        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*refs).lock);
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
        return cp;
    }

    // Not sharing, no cached copy, only a small portion requested.

    // Unmapped ref ID.
    if id < 0 || (*refs).fn_.is_empty() {
        if !(*fdl).ref_free.is_null() {
            free((*fdl).ref_free.cast());
            (*fdl).ref_free = std::ptr::null_mut();
        }
        (*fdl).ref_ = std::ptr::null_mut();
        (*fdl).ref_id = id;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*refs).lock);
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
        return std::ptr::null_mut();
    }

    // Open file if it's not already the current open reference.
    if (*refs).fn_ != (*r).fn_ || (*refs).fp.is_null() {
        if !(*refs).fp.is_null() && bgzf_close((*refs).fp) != 0 {
            return std::ptr::null_mut();
        }
        (*refs).fn_ = (*r).fn_.clone();
        (*refs).fp =
            cram_cram_io_c_2503_bgzf_open_ref(&(*refs).fn_, c"r".as_ptr().cast_mut(), (*r).is_md5);
        if (*refs).fp.is_null() {
            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*refs).lock);
            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
            return std::ptr::null_mut();
        }
    }

    let loaded = cram_cram_io_c_3228_load_ref_portion((*refs).fp, r.cast(), start, end);
    if loaded.is_null() {
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*refs).lock);
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
        return std::ptr::null_mut();
    }
    (*fdl).ref_ = loaded;

    if !(*fdl).ref_free.is_null() {
        free((*fdl).ref_free.cast());
    }

    (*fdl).ref_id = id;
    (*fdl).ref_start = start;
    (*fdl).ref_end = end;
    (*fdl).ref_free = (*fdl).ref_;
    let seq = (*fdl).ref_;

    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*refs).lock);
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);

    if seq.is_null() {
        std::ptr::null_mut()
    } else {
        seq.add((ostart - start) as usize)
    }
}
pub unsafe fn cram_cram_io_c_3597_cram_load_reference(
    fd: *mut cram_fd,
    mut fn_: *mut c_char,
) -> c_int {
    let fd = fd.cast::<cram_fd_layout>();
    let mut ret = 0;

    if !fn_.is_null() {
        (*fd).refs = cram_cram_io_c_2541_refs_load_fai(
            (*fd).refs.cast(),
            fn_,
            !((*fd).embed_ref > 0 && (*fd).mode == b'r' as c_int) as c_int,
        )
        .cast();
        fn_ = if !(*fd).refs.is_null()
            && !(*(*fd).refs.cast::<refs_t_layout>()).fn_.is_empty()
        {
            (*(*fd).refs.cast::<refs_t_layout>())
                .fn_
                .as_ptr()
                .cast_mut()
                .cast()
        } else {
            std::ptr::null_mut()
        };
        if fn_.is_null() {
            ret = -1;
        }
        cram_cram_io_c_2693_sanitise_SQ_lines(fd.cast());
    }
    // C aliases fd->ref_fn = fd->refs->fn (a NUL-terminated char*). Here
    // refs->fn_ is a Vec<u8> with no NUL terminator, so aliasing its data
    // pointer would let downstream C-string consumers (full_path -> UR tag)
    // read past the Vec into garbage. Own a NUL-terminated copy instead,
    // freeing any previous one so repeated cram_load_reference calls don't leak.
    crate::htslib_rs::c_compat::free((*fd).ref_fn.cast());
    (*fd).ref_fn = if fn_.is_null() {
        std::ptr::null_mut()
    } else {
        let bytes = (*(*fd).refs.cast::<refs_t_layout>()).fn_.as_slice();
        let dst = crate::htslib_rs::c_compat::malloc(bytes.len() as u64 + 1).cast::<c_char>();
        if !dst.is_null() {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst.cast::<u8>(), bytes.len());
            *dst.add(bytes.len()) = 0;
        }
        dst
    };

    if ((*fd).refs.is_null() || ((*(*fd).refs.cast::<refs_t_layout>()).nref == 0 && fn_.is_null()))
        && !(*fd).header.is_null()
    {
        if !(*fd).refs.is_null() {
            cram_cram_io_c_2427_refs_free((*fd).refs.cast());
        }
        (*fd).refs = cram_cram_io_c_2467_refs_create().cast();
        if (*fd).refs.is_null() {
            return -1;
        }
        if cram_cram_io_c_2768_refs_from_header(fd.cast()) == -1 {
            return -1;
        }
    }

    if !(*fd).header.is_null()
        && cram_cram_io_c_2737_refs2id((*fd).refs.cast(), (*fd).header.cast()) == -1
    {
        return -1;
    }

    ret
}
pub unsafe fn cram_cram_io_c_1490_cram_block_size(b: *mut cram_block) -> u32 {
    let b = b.cast::<cram_block_layout>();
    let itf8_len = |v: i64| -> u32 {
        if (v & !0x7f) == 0 {
            1
        } else if (v & !0x3fff) == 0 {
            2
        } else if (v & !0x1f_ffff) == 0 {
            3
        } else if (v & !0xfff_ffff) == 0 {
            4
        } else {
            5
        }
    };

    let header = 2
        + itf8_len((*b).content_id as i64)
        + itf8_len((*b).comp_size as i64)
        + itf8_len((*b).uncomp_size as i64)
        + 4;
    let payload = if (*b).method == crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW {
        (*b).uncomp_size
    } else {
        (*b).comp_size
    };
    header + payload as u32
}
pub unsafe fn cram_cram_io_c_4330_cram_new_compression_header() -> *mut cram_block_compression_hdr {
    let hdr = Box::into_raw(Box::new(cram_block_compression_hdr_layout {
        ref_seq_id: 0,
        ref_seq_start: 0,
        ref_seq_span: 0,
        num_records: 0,
        num_landmarks: 0,
        landmark: std::ptr::null_mut(),
        read_names_included: 0,
        ap_delta: 0,
        substitution_matrix: [[0; 4]; 5],
        no_ref: 0,
        qs_seq_orient: 0,
        td_blk: std::ptr::null_mut(),
        ntl: 0,
        tl: std::ptr::null_mut(),
        td_hash: std::ptr::null_mut(),
        td_keys: None,
        preservation_map: std::ptr::null_mut(),
        rec_encoding_map: [std::ptr::null_mut(); 32],
        tag_encoding_map: [std::ptr::null_mut(); 32],
        codecs: [std::ptr::null_mut(); 47],
        uncomp: std::ptr::null_mut(),
        uncomp_size: 0,
        uncomp_alloc: 0,
        ncodecs: 0,
    }));

    (*hdr).td_blk =
        cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE, 0)
            .cast();
    if (*hdr).td_blk.is_null() {
        cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
        return std::ptr::null_mut();
    }

    (*hdr).td_hash = calloc(1, std::mem::size_of::<kh_generic_layout>() as u64).cast::<c_void>();
    if (*hdr).td_hash.is_null() {
        cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
        return std::ptr::null_mut();
    }

    (*hdr).td_keys = Some(cram_string_alloc_c_55_string_pool_create(8192));

    hdr.cast()
}
pub unsafe fn cram_cram_io_c_4356_cram_free_compression_header(
    hdr: *mut cram_block_compression_hdr,
) {
    let hdr = hdr.cast::<cram_block_compression_hdr_layout>();
    if hdr.is_null() {
        return;
    }

    if !(*hdr).landmark.is_null() {
        free((*hdr).landmark.cast());
    }

    if !(*hdr).preservation_map.is_null() {
        let h = (*hdr).preservation_map.cast::<kh_generic_layout>();
        free((*h).flags.cast());
        free((*h).keys.cast());
        free((*h).vals.cast());
        free(h.cast());
    }

    for i in 0..32usize {
        let mut m = (*hdr).rec_encoding_map[i].cast::<cram_map_layout>();
        while !m.is_null() {
            let m2 = (*m).next;
            if !(*m).codec.is_null() {
                let c = (*m).codec.cast::<cram_codec_base_layout>();
                if let Some(free_fn) = (*c).free {
                    free_fn(c);
                }
            }
            free(m.cast());
            m = m2;
        }
    }

    for i in 0..32usize {
        let mut m = (*hdr).tag_encoding_map[i].cast::<cram_map_layout>();
        while !m.is_null() {
            let m2 = (*m).next;
            if !(*m).codec.is_null() {
                let c = (*m).codec.cast::<cram_codec_base_layout>();
                if let Some(free_fn) = (*c).free {
                    free_fn(c);
                }
            }
            free(m.cast());
            m = m2;
        }
    }

    for i in 0..CRAM_DS_END {
        let c = (*hdr).codecs[i].cast::<cram_codec_base_layout>();
        if !c.is_null() {
            if let Some(free_fn) = (*c).free {
                free_fn(c);
            }
        }
    }

    if !(*hdr).tl.is_null() {
        free((*hdr).tl.cast());
    }
    if !(*hdr).td_blk.is_null() {
        cram_free_block((*hdr).td_blk.cast());
    }
    if !(*hdr).td_hash.is_null() {
        let h = (*hdr).td_hash.cast::<kh_generic_layout>();
        free((*h).flags.cast());
        free((*h).keys.cast());
        free((*h).vals.cast());
        free(h.cast());
    }
    if let Some(p) = (*hdr).td_keys.take() {
        cram_string_alloc_c_103_string_pool_destroy(p);
    }

    drop(Box::from_raw(hdr));
}
pub unsafe fn cram_cram_io_c_4660_cram_read_file_def(
    fd: *mut cram_fd,
) -> *mut cram_file_def_layout {
    let mut def = Box::new(cram_file_def_layout {
        magic: [0; 4],
        major_version: 0,
        minor_version: 0,
        file_id: [0; 20],
    });

    let fd_layout = fd.cast::<cram_fd_layout>();
    if htslib_hfile_h_247_hread((*fd_layout).fp, def.magic.as_mut_ptr().cast(), 26) != 26 {
        return std::ptr::null_mut();
    }

    if def.magic
        != [
            b'C' as c_char,
            b'R' as c_char,
            b'A' as c_char,
            b'M' as c_char,
        ]
    {
        return std::ptr::null_mut();
    }

    if def.major_version > 4 {
        return std::ptr::null_mut();
    }

    (*fd_layout).first_container += 26;
    (*fd_layout).curr_position = (*fd_layout).first_container;
    (*fd_layout).last_slice = 0;

    Box::into_raw(def)
}
pub unsafe fn cram_cram_io_c_4694_cram_write_file_def(
    fd: *mut cram_fd,
    def: *mut cram_file_def_layout,
) -> c_int {
    let fd_layout = fd.cast::<cram_fd_layout>();
    if htslib_hfile_h_292_hwrite(
        (*fd_layout).fp,
        &(*def).magic[0] as *const c_char as *const c_void,
        26,
    ) == 26
    {
        0
    } else {
        -1
    }
}
pub unsafe fn cram_cram_io_c_4698_cram_free_file_def(def: *mut cram_file_def_layout) {
    if !def.is_null() {
        drop(Box::from_raw(def));
    }
}
/// original: cram_write_SAM_hdr (htslib/cram/cram_io.c:4891)
///
/// Writes a CRAM SAM header out. For CRAM 2.x/3.x/4.x this is a header
/// container holding a FILE_HEADER block with the int32-length-prefixed SAM
/// header text (compressed at v3+). For CRAM 1.0 the header is written
/// inline (int32 length followed by raw header text) and an UNKNOWN @RG is
/// added if missing.
///
/// Updates @SQ M5 tags from references on disk when M5 is absent and an
/// external reference is available; mirrors the libhts behaviour exactly,
/// including the auto-bump to embed_ref=2 when no reference can be found.
///
/// Returns 0 on success, -1 on failure.
pub unsafe fn cram_cram_io_c_4889_cram_write_SAM_hdr(
    fd: *mut cram_fd,
    hdr: *mut sam_hdr_t,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let major = (*fdl).version >> 8;
    let minor = (*fdl).version & 0xff;
    let blank_block = major >= 3;
    let is_cram_3 = major >= 3;

    // Write CRAM MAGIC if not yet written.
    let file_def = (*fdl).file_def.cast::<cram_file_def_layout>();
    if (*file_def).major_version == 0 {
        (*file_def).major_version = major as u8;
        (*file_def).minor_version = minor as u8;
        if cram_cram_io_c_4694_cram_write_file_def(fd, file_def) != 0 {
            return -1;
        }
    }

    // CRAM 1.0 requires an UNKNOWN read-group.
    if major == 1 {
        let hrecs = (*hdr).hrecs;
        if !hrecs.is_null()
            && crate::htslib_rs::sam::sam_hrecs_find_type_id(
                &mut *hrecs,
                c"RG",
                Some((c"ID", c"UNKNOWN")),
            )
            .is_none()
            && crate::htslib_rs::sam::sam_hdr_add_line(
                &mut *hdr,
                c"RG",
                &[
                    (c"ID".as_ptr(), c"UNKNOWN".as_ptr()),
                    (c"SM".as_ptr(), c"UNKNOWN".as_ptr()),
                ],
            ) != 0
        {
            return -1;
        }
    }

    if cram_cram_io_c_2768_refs_from_header(fd) == -1 {
        return -1;
    }
    if cram_cram_io_c_2737_refs2id((*fdl).refs.cast(), (*fdl).header.cast()) == -1 {
        return -1;
    }

    // Fix M5 strings — only when an external reference is in play.
    if !(*fdl).refs.is_null() && (*fdl).no_ref == 0 && (*fdl).embed_ref <= 1 {
        let hrecs = (*hdr).hrecs;
        if hrecs.is_null() {
            return -1;
        }
        let nref = (*hrecs).nref;
        let mut i = 0;
        while i < nref {
            let ref_name = (*(*hrecs).ref_.add(i as usize)).name;
            let ty = crate::htslib_rs::sam::sam_hrecs_find_type_id(
                &mut *hrecs,
                c"SQ",
                Some((c"SN", CStr::from_ptr(ref_name))),
            );
            let ty = match ty {
                Some(ty) => ty.as_ptr(),
                None => return -1,
            };

            if crate::htslib_rs::sam::sam_hrecs_find_key(&mut *ty, c"M5")
                .0
                .is_none()
            {
                let refs = (*fdl).refs.cast::<refs_t_layout>();
                if (*fdl).refs.is_null()
                    || (*refs).ref_id.is_null()
                    || (*(*refs).ref_id.add(i as usize)).is_null()
                {
                    return -1;
                }
                let mut rlen = (*(*(*refs).ref_id.add(i as usize))).length;
                let ref_seq = cram_cram_io_c_3409_cram_get_ref(fd, i, 1, rlen);
                if ref_seq.is_null() {
                    if (*fdl).embed_ref == -1 {
                        // auto embed-ref
                        hts_log_cstr(
                            HTS_LOG_WARNING,
                            c"cram_write_SAM_hdr".as_ptr(),
                            c"No M5 tags present and could not find reference".as_ptr(),
                        );
                        hts_log_cstr(
                            HTS_LOG_WARNING,
                            c"cram_write_SAM_hdr".as_ptr(),
                            c"Enabling embed_ref=2 option".as_ptr(),
                        );
                        hts_log_cstr(
                            HTS_LOG_WARNING,
                            c"cram_write_SAM_hdr".as_ptr(),
                            c"NOTE: the CRAM file will be bigger than using an external reference"
                                .as_ptr(),
                        );
                        crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).ref_lock);
                        // Best guess. It may be unmapped data with broken
                        // headers, in which case this will get ignored.
                        (*fdl).embed_ref = 2;
                        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
                        break;
                    }
                    return -1;
                }
                // In case it just loaded.
                rlen = (*(*(*refs).ref_id.add(i as usize))).length;

                let mut md5 = crate::htslib_rs::md5::hts_md5_init();
                // hts_pos_t is i64; on all our targets unsigned long is 64-bit,
                // so the single-shot update applies (matches the
                // HTS_POS_MAX <= ULONG_MAX branch).
                crate::htslib_rs::md5::hts_md5_update(
                    &mut md5,
                    std::slice::from_raw_parts(ref_seq.cast::<u8>(), rlen as usize),
                    rlen as usize,
                );
                let mut buf = [0u8; 16];
                crate::htslib_rs::md5::hts_md5_final(&mut buf, &mut md5);
                crate::htslib_rs::md5::hts_md5_destroy(Some(md5));
                cram_cram_io_c_3213_cram_ref_decr((*fdl).refs.cast(), i);

                let mut buf2 = [0u8; 33];
                crate::htslib_rs::md5::hts_md5_hex(&mut buf2, &buf);
                (*(*(*refs).ref_id.add(i as usize))).validated_md5 = 1;
                if crate::htslib_rs::sam::sam_hdr_update_line(
                    &mut *hdr,
                    c"SQ",
                    Some((c"SN", CStr::from_ptr(ref_name))),
                    &[(c"M5".as_ptr(), buf2.as_ptr().cast())],
                ) != 0
                {
                    return -1;
                }
            }

            if !(*fdl).ref_fn.is_null() {
                let mut ref_fn_buf = [0 as c_char; crate::htslib_rs::c_compat::PATH_MAX as usize];
                cram_cram_io_c_4850_full_path(ref_fn_buf.as_mut_ptr(), (*fdl).ref_fn);
                if crate::htslib_rs::sam::sam_hdr_update_line(
                    &mut *hdr,
                    c"SQ",
                    Some((c"SN", CStr::from_ptr(ref_name))),
                    &[(c"UR".as_ptr(), ref_fn_buf.as_ptr())],
                ) != 0
                {
                    return -1;
                }
            }
            i += 1;
        }
    }

    // Length
    let header_len = crate::htslib_rs::sam::sam_hdr_length(&mut *hdr);
    if header_len > i32::MAX as usize {
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"cram_write_SAM_hdr".as_ptr(),
            c"Header is too long for CRAM format".as_ptr(),
        );
        return -1;
    }
    if major == 1 {
        if int32_encode(&mut *fdl, header_len as i32) == -1 {
            return -1;
        }
        // Text data
        let text = crate::htslib_rs::sam::sam_hdr_str(&mut *hdr);
        if htslib_hfile_h_292_hwrite((*fdl).fp, text.cast(), header_len)
            != header_len as libc::ssize_t
        {
            return -1;
        }
    } else {
        // Create block(s) inside a container.
        let b = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_FILE_HEADER, 0);
        let c = cram_cram_io_c_3639_cram_new_container(0, 0);
        if b.is_null() || c.is_null() {
            if !b.is_null() {
                cram_free_block(b);
            }
            if !c.is_null() {
                cram_cram_io_c_3705_cram_free_container(c);
            }
            return -1;
        }

        if int32_put_blk(&mut *b.cast::<cram_block_layout>(), header_len as i32) < 0 {
            return -1;
        }
        if header_len != 0 {
            let text = crate::htslib_rs::sam::sam_hdr_str(&mut *hdr);
            if cram_cram_io_h_248_block_append(b, text.cast(), header_len) != 0 {
                return -1;
            }
        }
        // BLOCK_UPLEN(b): macro sets both comp_size and uncomp_size to byte.
        let bl = b.cast::<cram_block_layout>();
        (*bl).uncomp_size = (*bl).byte as i32;
        (*bl).comp_size = (*bl).uncomp_size;

        // Compress header block if V3.0 and above.
        if major >= 3
            && cram_cram_io_c_2323_cram_compress_block(fd, b, std::ptr::null_mut(), -1, -1) < 0
        {
            return -1;
        }

        let cl = c.cast::<cram_container_layout>();
        let varint_size = (*fdl).vv.varint_size.expect("set by cram_init_tables");
        let crc_extra: i32 = if is_cram_3 { 4 } else { 0 };

        let padded_length: i32;
        if blank_block {
            (*cl).length = (*bl).comp_size
                + 2
                + crc_extra
                + varint_size((*bl).content_id as i64) as i32
                + varint_size((*bl).uncomp_size as i64) as i32
                + varint_size((*bl).comp_size as i64) as i32;

            (*cl).num_blocks = 2;
            (*cl).num_landmarks = 2;
            (*cl).landmark = malloc((2 * std::mem::size_of::<i32>()) as u64).cast::<i32>();
            if (*cl).landmark.is_null() {
                cram_free_block(b);
                cram_cram_io_c_3705_cram_free_container(c);
                return -1;
            }
            *(*cl).landmark.add(0) = 0;
            *(*cl).landmark.add(1) = (*cl).length;

            // Plus extra storage for uncompressed secondary blank block.
            // MIN(c->length*.5, 10000)
            let half = ((*cl).length as f64) * 0.5;
            padded_length = if half < 10000.0 { half as i32 } else { 10000 };
            (*cl).length += padded_length
                + 2
                + crc_extra
                + varint_size((*bl).content_id as i64) as i32
                + varint_size(padded_length as i64) * 2;
        } else {
            // Pad the block instead.
            (*cl).num_blocks = 1;
            (*cl).num_landmarks = 1;
            (*cl).landmark = malloc(std::mem::size_of::<i32>() as u64).cast::<i32>();
            if (*cl).landmark.is_null() {
                return -1;
            }
            *(*cl).landmark.add(0) = 0;

            // MAX(c->length*1.5, 10000) - c->length
            let one_and_half = ((*cl).length as f64) * 1.5;
            let max_val = if one_and_half > 10000.0 {
                one_and_half as i32
            } else {
                10000
            };
            padded_length = max_val - (*cl).length;

            (*cl).length = (*bl).comp_size
                + padded_length
                + 2
                + crc_extra
                + varint_size((*bl).content_id as i64) as i32
                + varint_size((*bl).uncomp_size as i64) as i32
                + varint_size((*bl).comp_size as i64) as i32;

            let pad_len = padded_length as usize;
            let mut pads = Vec::new();
            if pads.try_reserve_exact(pad_len).is_err() {
                cram_free_block(b);
                cram_cram_io_c_3705_cram_free_container(c);
                return -1;
            }
            pads.resize(pad_len, 0);
            if cram_cram_io_h_248_block_append(b, pads.as_ptr().cast(), pads.len()) != 0 {
                return -1;
            }
            // BLOCK_UPLEN(b): macro sets both comp_size and uncomp_size to byte.
            (*bl).uncomp_size = (*bl).byte as i32;
            (*bl).comp_size = (*bl).uncomp_size;
        }

        if cram_cram_io_c_4023_cram_write_container(fd.cast(), c.cast()) == -1 {
            cram_free_block(b);
            cram_cram_io_c_3705_cram_free_container(c);
            return -1;
        }

        if cram_write_block(&mut *fdl, &mut *bl) == -1 {
            cram_free_block(b);
            cram_cram_io_c_3705_cram_free_container(c);
            return -1;
        }

        if blank_block {
            // BLOCK_RESIZE(b, padded_length)
            if cram_cram_io_h_226_block_resize(b, padded_length as usize) != 0 {
                cram_free_block(b);
                cram_cram_io_c_3705_cram_free_container(c);
                return -1;
            }
            // memset(BLOCK_DATA(b), 0, padded_length)
            libc::memset((*bl).data.cast(), 0, padded_length as usize);
            // BLOCK_SIZE(b) = padded_length
            (*bl).byte = padded_length as usize;
            // BLOCK_UPLEN(b): macro sets both comp_size and uncomp_size to byte.
            (*bl).uncomp_size = (*bl).byte as i32;
            (*bl).comp_size = (*bl).uncomp_size;
            (*bl).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            if cram_write_block(&mut *fdl, &mut *bl) == -1 {
                cram_free_block(b);
                cram_cram_io_c_3705_cram_free_container(c);
                return -1;
            }
        }

        cram_free_block(b);
        cram_cram_io_c_3705_cram_free_container(c);
    }

    if crate::htslib_rs::hfile::hflush((*fdl).fp) != 0 {
        return -1;
    }

    0
}
/// original: cram_read_SAM_hdr (htslib/cram/cram_io.c:4717)
///
/// Reads the SAM header from the first CRAM data block of a CRAM file. Reads
/// the SAM-header container (content_type=FILE_HEADER, ref_seq_id=0),
/// decompresses the first block (which holds an int32 length-prefix followed
/// by the SAM header text), parses the text into a native `sam_hdr_t`, and
/// returns it.
///
/// Returns the freshly-built header on success, NULL on failure.
pub unsafe fn cram_cram_io_c_4717_cram_read_SAM_hdr(fd: *mut cram_fd) -> *mut sam_hdr_t {
    let fdl = fd.cast::<cram_fd_layout>();
    let major = (*fdl).version >> 8;

    // v1.x stores the header inline (no container). Our consumers only see
    // 2.x/3.x/4.x CRAMs, so port the 2+ path and leave v1 unsupported; mirrors
    // what the live decode pipeline already accepts.
    if major == 1 {
        return std::ptr::null_mut();
    }

    let c = cram_cram_io_c_3788_cram_read_container(fd.cast());
    if c.is_null() {
        return std::ptr::null_mut();
    }
    let cl = c.cast::<cram_container_layout>();
    (*fdl).first_container += (*cl).length as libc::off_t + (*cl).offset as libc::off_t;
    (*fdl).curr_position = (*fdl).first_container;

    if (*cl).num_blocks < 1 {
        cram_cram_io_c_3705_cram_free_container(c);
        return std::ptr::null_mut();
    }

    let b = cram_read_block(&mut *fdl);
    if b.is_null() {
        cram_cram_io_c_3705_cram_free_container(c);
        return std::ptr::null_mut();
    }
    if cram_uncompress_block(&mut *b.cast::<cram_block_layout>()) != 0 {
        cram_cram_io_c_3705_cram_free_container(c);
        cram_free_block(b);
        return std::ptr::null_mut();
    }

    let vv = &(*fdl).vv;
    let varint_size = vv.varint_size.expect("varint_size set by cram_init_tables");
    let crc_extra = if major >= 3 { 4 } else { 0 };

    let bl = b.cast::<cram_block_layout>();
    let mut len: i64 = (*bl).comp_size as i64
        + 2
        + crc_extra
        + varint_size((*bl).content_id as i64) as i64
        + varint_size((*bl).uncomp_size as i64) as i64
        + varint_size((*bl).comp_size as i64) as i64;

    /* Extract header from 1st block */
    let mut header_len: i32 = 0;
    if int32_get_blk(&mut *bl, &mut header_len) == -1
        || header_len < 0
        || ((*bl).uncomp_size as i64) - 4 < header_len as i64
    {
        cram_cram_io_c_3705_cram_free_container(c);
        cram_free_block(b);
        return std::ptr::null_mut();
    }
    let header_len = header_len as usize;
    let mut header = Vec::new();
    if header.try_reserve_exact(header_len + 1).is_err() {
        cram_cram_io_c_3705_cram_free_container(c);
        cram_free_block(b);
        return std::ptr::null_mut();
    }
    header.resize(header_len + 1, 0);
    // memcpy(header, BLOCK_END(b), header_len) where BLOCK_END = &b->data[b->byte]
    memcpy(
        header.as_mut_ptr().cast(),
        (*bl).data.add((*bl).byte).cast(),
        header_len as u64,
    );
    cram_free_block(b);

    /* Consume any remaining blocks */
    for _ in 1..(*cl).num_blocks {
        let b2 = cram_read_block(&mut *fdl);
        if b2.is_null() {
            cram_cram_io_c_3705_cram_free_container(c);
            return std::ptr::null_mut();
        }
        let bl2 = b2.cast::<cram_block_layout>();
        len += (*bl2).comp_size as i64
            + 2
            + crc_extra
            + varint_size((*bl2).content_id as i64) as i64
            + varint_size((*bl2).uncomp_size as i64) as i64
            + varint_size((*bl2).comp_size as i64) as i64;
        cram_free_block(b2);
    }

    // Consume padding
    if (*cl).length > 0 && len > 0 && (*cl).length as i64 > len {
        let pad_len = (*cl).length as i64 - len;
        let Ok(pad_len) = usize::try_from(pad_len) else {
            cram_cram_io_c_3705_cram_free_container(c);
            return std::ptr::null_mut();
        };
        let mut pads = Vec::new();
        if pads.try_reserve_exact(pad_len).is_err() {
            cram_cram_io_c_3705_cram_free_container(c);
            return std::ptr::null_mut();
        }
        pads.resize(pad_len, 0);
        if pad_len as i64
            != htslib_hfile_h_247_hread((*fdl).fp, pads.as_mut_ptr().cast(), pad_len) as i64
        {
            cram_cram_io_c_3705_cram_free_container(c);
            return std::ptr::null_mut();
        }
    }

    cram_cram_io_c_3705_cram_free_container(c);

    /* Parse */
    let hdr = crate::htslib_rs::sam::sam_hdr_init();
    if hdr.is_null() {
        return std::ptr::null_mut();
    }

    if crate::htslib_rs::sam::sam_hdr_add_lines(&mut *hdr, &header[..header_len]) == -1 {
        crate::htslib_rs::sam::sam_hdr_destroy(hdr);
        return std::ptr::null_mut();
    }

    let header_text = malloc(header.len() as u64).cast::<c_char>();
    if header_text.is_null() {
        crate::htslib_rs::sam::sam_hdr_destroy(hdr);
        return std::ptr::null_mut();
    }
    memcpy(
        header_text.cast(),
        header.as_ptr().cast(),
        header.len() as u64,
    );
    (*hdr).l_text = header_len;
    (*hdr).text = header_text;

    hdr
}
/// original: cram_dopen (htslib/cram/cram_io.c:5289)
///
/// Wraps an existing hFILE as a CRAM file descriptor. For read mode reads the
/// file-def + SAM header; for write mode initialises a blank file-def and
/// leaves the SAM header to be written later. The fd is allocated via
/// `calloc` so byte-identical to C `calloc(1, sizeof(cram_fd))`.
pub unsafe fn cram_cram_io_c_5289_cram_dopen(
    fp: *mut hFILE,
    filename: *const c_char,
    mode: *const c_char,
) -> *mut cram_fd {
    let fd = calloc(1, std::mem::size_of::<cram_fd_layout>() as u64).cast::<cram_fd_layout>();
    if fd.is_null() {
        return std::ptr::null_mut();
    }

    (*fd).level = CRAM_DEFAULT_LEVEL;
    let mut i: usize = 0;
    loop {
        let c = *mode.add(i);
        if c == 0 {
            break;
        }
        if c >= b'0' as c_char && c <= b'9' as c_char {
            (*fd).level = (c - b'0' as c_char) as c_int;
            break;
        }
        i += 1;
    }

    (*fd).fp = fp;
    (*fd).mode = *mode as c_int;
    (*fd).first_container = 0;
    (*fd).curr_position = 0;

    if (*fd).mode == b'r' as c_int {
        /* Reader */
        let def = cram_cram_io_c_4660_cram_read_file_def(fd.cast());
        if def.is_null() {
            free(fd.cast());
            return std::ptr::null_mut();
        }
        (*fd).file_def = def.cast();

        (*fd).version = (*def).major_version as c_int * 256 + (*def).minor_version as c_int;

        cram_cram_io_c_5170_cram_init_tables(fd.cast());

        let hdr = cram_cram_io_c_4717_cram_read_SAM_hdr(fd.cast());
        if hdr.is_null() {
            cram_cram_io_c_4698_cram_free_file_def(def);
            free(fd.cast());
            return std::ptr::null_mut();
        }
        (*fd).header = hdr.cast();
    } else {
        /* Writer */
        let mut def = Box::new(cram_file_def_layout {
            magic: [
                b'C' as c_char,
                b'R' as c_char,
                b'A' as c_char,
                b'M' as c_char,
            ],
            major_version: 0, // Indicator to write file def later.
            minor_version: 0,
            file_id: [0; 20],
        });
        libc::strncpy(def.file_id.as_mut_ptr(), filename, 20);
        let def = Box::into_raw(def);

        (*fd).file_def = def.cast();

        (*fd).version = CRAM_OPEN_DEFAULT_MAJOR * 256 + CRAM_OPEN_DEFAULT_MINOR;
        cram_cram_io_c_5170_cram_init_tables(fd.cast());

        /* SAM header written later along with this file_def */
    }

    // prefix = strdup(basename(filename))
    let bn = libc::strrchr(filename, b'/' as c_int);
    let bn = if bn.is_null() { filename } else { bn.add(1) };
    (*fd).prefix = strdup(bn);
    if (*fd).prefix.is_null() {
        if !(*fd).file_def.is_null() {
            cram_cram_io_c_4698_cram_free_file_def((*fd).file_def.cast());
        }
        if !(*fd).header.is_null() {
            crate::htslib_rs::sam::sam_hdr_destroy((*fd).header.cast());
        }
        free(fd.cast());
        return std::ptr::null_mut();
    }
    (*fd).first_base = -1;
    (*fd).last_base = -1;
    (*fd).record_counter = 0;

    (*fd).ctr = std::ptr::null_mut();
    (*fd).ctr_mt = std::ptr::null_mut();
    (*fd).refs = cram_cram_io_c_2467_refs_create().cast();
    if (*fd).refs.is_null() {
        cram_cram_io_c_4698_cram_free_file_def((*fd).file_def.cast());
        if !(*fd).header.is_null() {
            crate::htslib_rs::sam::sam_hdr_destroy((*fd).header.cast());
        }
        free((*fd).prefix.cast());
        free(fd.cast());
        return std::ptr::null_mut();
    }
    (*fd).ref_id = -2;
    (*fd).ref_ = std::ptr::null_mut();

    (*fd).decode_md = 0;
    (*fd).seqs_per_slice = CRAM_DEFAULT_SEQS_PER_SLICE;
    (*fd).bases_per_slice = CRAM_DEFAULT_BASES_PER_SLICE;
    (*fd).slices_per_container = 1; // SLICE_PER_CNT
    (*fd).embed_ref = -1;
    (*fd).no_ref = 0;
    (*fd).no_ref_counter = 0;
    (*fd).ap_delta = 0;
    (*fd).ignore_md5 = 0;
    (*fd).lossy_read_names = 0;
    (*fd).use_bz2 = 0;
    let major = (*fd).version >> 8;
    let minor = (*fd).version & 0xff;
    (*fd).use_rans = if major >= 3 { 1 } else { 0 };
    (*fd).use_tok = if major >= 3 && minor >= 1 { 1 } else { 0 };
    (*fd).use_lzma = 0;
    (*fd).multi_seq = -1;
    (*fd).multi_seq_user = -1;
    (*fd).unsorted = 0;
    (*fd).shared_ref = 0;
    (*fd).store_md = 0;
    (*fd).store_nm = 0;
    (*fd).last_ri_count = 0;

    (*fd).index = std::ptr::null_mut();
    (*fd).own_pool = 0;
    (*fd).pool = std::ptr::null_mut();
    (*fd).rqueue = std::ptr::null_mut();
    (*fd).job_pending = std::ptr::null_mut();
    (*fd).ooc = 0;
    (*fd).required_fields = c_int::MAX as c_uint;

    crate::htslib_rs::c_compat::pthread_mutex_init(&mut (*fd).metrics_lock, std::ptr::null());
    crate::htslib_rs::c_compat::pthread_mutex_init(&mut (*fd).ref_lock, std::ptr::null());
    crate::htslib_rs::c_compat::pthread_mutex_init(&mut (*fd).range_lock, std::ptr::null());
    crate::htslib_rs::c_compat::pthread_mutex_init(&mut (*fd).bam_list_lock, std::ptr::null());

    for k in 0..CRAM_DS_END {
        (*fd).m[k] = cram_cram_io_c_2327_cram_new_metrics().cast();
        if (*fd).m[k].is_null() {
            cram_cram_io_c_5560_cram_dopen_cleanup_err(fd);
            return std::ptr::null_mut();
        }
    }

    // fd->tags_used = kh_init(m_metrics)  ==  calloc(1, sizeof(kh_m_metrics_t))
    (*fd).tags_used = calloc(1, std::mem::size_of::<kh_generic_layout>() as u64).cast::<c_void>();
    if (*fd).tags_used.is_null() {
        cram_cram_io_c_5560_cram_dopen_cleanup_err(fd);
        return std::ptr::null_mut();
    }

    (*fd).range.refid = -2; // no ref.
    (*fd).eof = 1;
    (*fd).ref_fn = std::ptr::null_mut();

    (*fd).bl = std::ptr::null_mut();

    /* Initialise dummy refs from the @SQ headers (only meaningful if header set) */
    if cram_cram_io_c_2768_refs_from_header(fd.cast()) == -1 {
        cram_cram_io_c_5560_cram_dopen_cleanup_err(fd);
        return std::ptr::null_mut();
    }

    fd.cast()
}
/// original: cram_open (htslib/cram/cram_io.c:5264)
///
/// Open a CRAM file by name for read ("r") or write ("w"). Returns the fd on
/// success or NULL on failure.
pub unsafe fn cram_cram_io_c_5264_cram_open(
    filename: *const c_char,
    mode: *const c_char,
) -> *mut cram_fd {
    let mut fmode: [c_char; 3] = [*mode, 0, 0];
    let mode_bytes = c_char_bytes(mode).unwrap_or(&[]);
    if matches!(mode_bytes.get(1), Some(b'b' | b'c')) {
        fmode[1] = b'b' as c_char;
    }

    let fp = hopen(filename, fmode.as_ptr());
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    let fd = cram_cram_io_c_5289_cram_dopen(fp, filename, mode);
    if fd.is_null() {
        hclose_abruptly(fp);
    }
    fd
}
/// original: cram_close (htslib/cram/cram_io.c:5558)
///
/// Closes a CRAM file. For READ mode this drains any decode-queue work (a
/// no-op for single-threaded fds, which the live read tests always use),
/// then releases every per-fd allocation. For WRITE mode it now drives the
/// native `cram_flush_container_mt` + `cram_write_eof_block` pair from the
/// `cram_flush_bridge` translation before tearing down. The pool/rqueue,
/// spare_bams (`bl`), and loaded-CRAI (`index`) paths still fall back to
/// the C library because their dependent helpers (cram_flush_thread /
/// cram_flush_result / reset_metrics / free_bam_list / cram_index_free)
/// have not been translated yet. Returns 0 on success, -1 on failure.
pub unsafe fn cram_cram_io_c_5558_cram_close(fd: *mut cram_fd) -> c_int {
    if fd.is_null() {
        return -1;
    }
    let fdl = fd.cast::<cram_fd_layout>();

    // Decide upfront whether the native teardown can handle this fd. The
    // following C-only paths are not yet ported and require the libhts
    // close to be safe:
    //   * pool/rqueue set → multi-threaded drain + hts_tpool_process_destroy.
    //     (We have native `hts_tpool_process_*` but the cram-side helpers
    //     `cram_flush_thread` / `cram_flush_result` / `reset_metrics` are
    //     not yet translated; the MT path needs all of those.)
    //
    // The `bl` (spare_bams chain) and `index` (loaded CRAI tree) paths are
    // now handled natively below via the cram_flush_bridge translations of
    // `free_bam_list` (htslib/cram/cram_io.c:3697) and `cram_index_free`
    // (htslib/cram/cram_index.c:374).
    let mut ret: c_int = 0;

    // C: if (fd->mode == 'w' && fd->ctr) { ... flush ... }
    if (*fdl).mode == b'w' as c_int && !(*fdl).ctr.is_null() {
        let ctr_w = (*fdl).ctr;
        if !(*ctr_w).slice.is_null() {
            cram_update_curr_slice_native(ctr_w, (*fdl).version);
        }
        if -1 == cram_cram_io_c_4275_cram_flush_container_mt(fd, ctr_w.cast()) {
            ret = -1;
        }
    }

    // C (cram_io.c:5572-5589): MT-pool cleanup. Drain in-flight decode jobs
    // (READ mode), flush the encoder result queue (WRITE mode), then destroy
    // the pool's result queue. Single-threaded (pool/rqueue null) is a no-op.
    if (*fdl).mode != b'w' as c_int {
        cram_drain_rqueue_native(fdl);
    }
    if !(*fdl).pool.is_null() && (*fdl).eof >= 0 && !(*fdl).rqueue.is_null() {
        let rq = (*fdl)
            .rqueue
            .cast::<crate::htslib_rs::thread_pool::hts_tpool_process>();
        crate::htslib_rs::thread_pool::hts_tpool_process_flush(&mut *rq);
        if cram_flush_result_native(fd) != 0 {
            ret = -1;
        }
        if (*fdl).mode == b'w' as c_int {
            // prevent double-freeing: cram_flush_result freed lc, which was the ctr
            (*fdl).ctr = std::ptr::null_mut();
        }
        crate::htslib_rs::thread_pool::hts_tpool_process_destroy(rq);
        (*fdl).rqueue = std::ptr::null_mut();
    }

    // C: if (ret == 0 && fd->mode == 'w') cram_write_eof_block(fd)
    if ret == 0 && (*fdl).mode == b'w' as c_int && 0 != cram_cram_io_c_5474_cram_write_eof_block(fd)
    {
        ret = -1;
    }

    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut (*fdl).metrics_lock);
    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut (*fdl).ref_lock);
    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut (*fdl).range_lock);
    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut (*fdl).bam_list_lock);

    // C (cram_io.c:5601-5607): walk the spare_bams chain freeing each per-slot
    // bam1_t array and the chain node itself.
    //
    //     for (bl = fd->bl; bl; bl = next) {
    //         int max_rec = fd->seqs_per_slice * fd->slices_per_container;
    //         next = bl->next;
    //         free_bam_list(bl->bams, max_rec);
    //         free(bl);
    //     }
    //
    // The struct layout is htslib/cram/cram_structs.h:747-750: { bams; next; }.
    #[repr(C)]
    struct SpareBamsLayout {
        bams: *mut *mut bam1_t,
        next: *mut SpareBamsLayout,
    }
    let mut bl = (*fdl).bl.cast::<SpareBamsLayout>();
    while !bl.is_null() {
        let max_rec = (*fdl).seqs_per_slice * (*fdl).slices_per_container;
        let next = (*bl).next;
        cram_cram_io_c_3697_free_bam_list((*bl).bams, max_rec);
        free(bl.cast());
        bl = next;
    }

    if hclose((*fdl).fp) != 0 {
        ret = -1;
    }

    if !(*fdl).file_def.is_null() {
        cram_cram_io_c_4698_cram_free_file_def((*fdl).file_def.cast());
    }

    if !(*fdl).header.is_null() {
        crate::htslib_rs::sam::sam_hdr_destroy((*fdl).header.cast());
    }

    free((*fdl).prefix.cast());

    if !(*fdl).ctr.is_null() {
        cram_cram_io_c_3705_cram_free_container((*fdl).ctr.cast());
    }

    if !(*fdl).ctr_mt.is_null() && (*fdl).ctr_mt != (*fdl).ctr {
        cram_cram_io_c_3705_cram_free_container((*fdl).ctr_mt.cast());
    }

    if !(*fdl).refs.is_null() {
        cram_cram_io_c_2427_refs_free((*fdl).refs.cast());
    }
    // C aliases fd->ref_fn to fd->refs->fn (freed by refs_free above), so it
    // frees nothing here. Our cram_load_reference owns a separate NUL-terminated
    // copy in ref_fn, so free it to match the net allocation balance.
    if !(*fdl).ref_fn.is_null() {
        free((*fdl).ref_fn.cast());
        (*fdl).ref_fn = std::ptr::null_mut();
    }
    if !(*fdl).ref_free.is_null() {
        free((*fdl).ref_free.cast());
    }

    for k in 0..CRAM_DS_END {
        if !(*fdl).m[k].is_null() {
            free((*fdl).m[k].cast());
        }
    }

    if !(*fdl).tags_used.is_null() {
        // For read-mode CRAM fds, tags_used is only populated by the encode
        // side; here it's the empty kh_init(m_metrics) table allocated at
        // open. khash buckets are NULL ⇒ free the (NULL) arrays and the
        // table.
        let h = (*fdl).tags_used.cast::<kh_generic_layout>();
        if !(*h).flags.is_null() {
            // Walk vals freeing kh_val entries (cram_metrics*) if any exist.
            for k in 0..(*h).n_buckets {
                if ((*(*h).flags.add((k >> 4) as usize) >> ((k & 0x0f) << 1)) & 3) != 0 {
                    continue;
                }
                // m_metrics values are cram_metrics*. cram_metrics is just a
                // freeable malloc'd struct in C (cram_new_metrics).
                let val_ptr = (*h).vals.cast::<*mut c_void>().add(k as usize);
                let v = *val_ptr;
                if !v.is_null() {
                    free(v);
                }
            }
            free((*h).flags.cast());
        }
        if !(*h).keys.is_null() {
            free((*h).keys.cast());
        }
        if !(*h).vals.is_null() {
            free((*h).vals.cast());
        }
        free(h.cast());
    }

    // C (cram_io.c:5646-5647): if (fd->index) cram_index_free(fd);
    if !(*fdl).index.is_null() {
        cram_index_free(&mut *fdl);
    }

    // idxfp: only set when a CRAI index was loaded; we don't load one in the
    // native open path, so this is NULL.
    if !(*fdl).idxfp.is_null() && crate::htslib_rs::bgzf::bgzf_close((*fdl).idxfp) < 0 {
        ret = -1;
    }

    free(fdl.cast());

    ret
}
pub unsafe fn cram_free_block(b: *mut cram_block) {
    let Some(block) = b.cast::<cram_block_layout>().as_mut() else {
        return;
    };
    if !block.data.is_null() {
        free(block.data.cast());
        block.data = std::ptr::null_mut();
    }
    drop(Box::from_raw(block));
}
/// original: cram_container_size (htslib/cram/cram_io.c:3947)
/// MAXIMUM storage size needed for the container.
pub unsafe fn cram_cram_io_c_3947_cram_container_size(c: *mut cram_container) -> c_int {
    let c = c.cast::<cram_container_layout>();
    55 + 5 * (*c).num_landmarks
}
/// original: cram_new_container (htslib/cram/cram_io.c:3639)
/// Creates a new empty container for subsequent writing (write path).
pub unsafe fn cram_cram_io_c_3639_cram_new_container(
    nrec: c_int,
    nslice: c_int,
) -> *mut cram_container {
    let c = Box::into_raw(Box::new(cram_container_layout {
        length: 0,
        ref_seq_id: 0,
        ref_seq_start: 0,
        ref_seq_span: 0,
        record_counter: 0,
        num_bases: 0,
        num_records: 0,
        num_blocks: 0,
        num_landmarks: 0,
        landmark: std::ptr::null_mut(),
        offset: 0,
        comp_hdr: std::ptr::null_mut(),
        comp_hdr_block: std::ptr::null_mut(),
        max_slice: nslice,
        curr_slice: 0,
        curr_slice_mt: 0,
        max_rec: nrec,
        curr_rec: 0,
        max_c_rec: nrec * nslice,
        curr_c_rec: 0,
        slice_rec: 0,
        curr_ref: -2,
        last_pos: 0,
        slices: std::ptr::null_mut(),
        slice: std::ptr::null_mut(),
        pos_sorted: 1,
        max_apos: 0,
        last_slice: 0,
        multi_seq: 0,
        unsorted: 0,
        qs_seq_orient: 1,
        ref_id: 0,
        ref_start: 0,
        first_base: 0,
        last_base: 0,
        ref_end: 0,
        ref_: std::ptr::null_mut(),
        embed_ref: -1, // automatic selection
        no_ref: 0,
        bams: std::ptr::null_mut(),
        stats: std::array::from_fn(|_| None),
        tags_used: std::ptr::null_mut(),
        refs_used: std::ptr::null_mut(),
        crc32: 0,
        s_num_bases: 0,
        s_aux_bytes: 0,
        n_mapped: 0,
        ref_free: 0,
    }));

    (*c).slices = calloc(
        if nslice != 0 { nslice as u64 } else { 1 },
        std::mem::size_of::<*mut cram_slice_layout>() as u64,
    )
    .cast::<*mut cram_slice_layout>();
    if (*c).slices.is_null() {
        cram_cram_io_c_3705_cram_free_container(c.cast());
        return std::ptr::null_mut();
    }
    (*c).slice = std::ptr::null_mut();

    (*c).comp_hdr = cram_cram_io_c_4330_cram_new_compression_header().cast();
    if (*c).comp_hdr.is_null() {
        cram_cram_io_c_3705_cram_free_container(c.cast());
        return std::ptr::null_mut();
    }
    (*c).comp_hdr_block = std::ptr::null_mut();

    // for (id = DS_RN; id < DS_TN; id++) c->stats[id] = cram_stats_create();
    let mut id = DS_RN;
    while id < DS_TN {
        (*c).stats[id as usize] = Some(cram_cram_stats_c_48_cram_stats_create());
        id += 1;
    }

    // c->tags_used = kh_init(m_tagmap)  ==  kcalloc(1, sizeof(kh_m_tagmap_t))
    (*c).tags_used =
        calloc(1, std::mem::size_of::<kh_generic_layout>() as u64).cast::<kh_generic_layout>();
    if (*c).tags_used.is_null() {
        cram_cram_io_c_3705_cram_free_container(c.cast());
        return std::ptr::null_mut();
    }
    (*c).refs_used = std::ptr::null_mut();
    (*c).ref_free = 0;

    c.cast()
}
// original: cram_flush_container (htslib/cram/cram_io.c:4143)
//
// Flushes a container: encode the body via the native cram_encode_container,
// then write it out via flush_container2.
pub unsafe fn cram_cram_io_c_4143_cram_flush_container(
    fd: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    if 0 != cram_cram_encode_c_1850_cram_encode_container(fd, c) {
        return -1;
    }
    cram_cram_io_c_4089_cram_flush_container2(fd, c)
}
// original: cram_write_eof_block (htslib/cram/cram_io.c:5474)
//
// Writes the empty-container EOF marker for CRAM v2+. v1 lacks the concept
// and returns 0 (no-op). Byte-faithful translation.
pub unsafe fn cram_cram_io_c_5474_cram_write_eof_block(fd: *mut cram_fd) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let major = (*fdl).version >> 8;
    if major >= 2 {
        let mut c: cram_container_layout = std::mem::zeroed();
        c.ref_seq_id = -1;
        c.ref_seq_start = 0x454f46; // "EOF"
        c.ref_seq_span = 0;
        c.record_counter = 0;
        c.num_bases = 0;
        c.num_blocks = 1;
        let mut land: [i32; 1] = [0];
        c.landmark = land.as_mut_ptr();
        // C leaves num_landmarks at the memset(0) value (cram_io.c:5486-5496);
        // the container encoder writes varint(num_landmarks)=0 and an empty
        // landmark loop, matching cram_io.c:5532-5535. Don't set num_landmarks=1
        // here, even though the array has one cell.

        let mut ch: cram_block_compression_hdr_layout = std::mem::zeroed();

        c.comp_hdr_block = cram_cram_encode_c_2810_cram_encode_compression_header(
            fd,
            (&mut c as *mut cram_container_layout).cast(),
            (&mut ch as *mut cram_block_compression_hdr_layout).cast(),
            0,
        )
        .cast();

        let comp_hdr_byte = (*c.comp_hdr_block).byte as i32;
        c.length = comp_hdr_byte + 5 + 4 * (if major >= 3 { 1 } else { 0 });

        let comp_hdr_blk: *mut cram_block = c.comp_hdr_block.cast();
        if cram_write_container(
            fd,
            (&mut c as *mut cram_container_layout).cast::<cram_container>(),
        ) < 0
            || cram_write_block(&mut *fdl, &mut *comp_hdr_blk.cast::<cram_block_layout>()) < 0
        {
            // cram_close + cram_free_block on failure. The C source
            // (cram_io.c:5516) calls cram_close from inside cram_write_eof_block
            // even when that produces a double-close (i.e., when reached via
            // cram_close); we mirror that byte-faithfully.
            cram_close(fd);
            cram_free_block(comp_hdr_blk);
            return -1;
        }

        // if (ch.preservation_map) kh_destroy(map, ch.preservation_map)
        if !ch.preservation_map.is_null() {
            let h = ch.preservation_map.cast::<kh_generic_layout>();
            if !(*h).flags.is_null() {
                free((*h).flags.cast());
            }
            if !(*h).keys.is_null() {
                free((*h).keys.cast());
            }
            if !(*h).vals.is_null() {
                free((*h).vals.cast());
            }
            free(h.cast());
        }
        cram_free_block(comp_hdr_blk);
    }

    0
}
// original: cram_flush_container2 (htslib/cram/cram_io.c:4089)
//
// Writes the container header, the compression-header block, and every slice
// block to disk. If the fd has an attached CRAI index file, drives
// `cram_index_slice` for each slice. Byte-faithful 1:1 translation.
pub unsafe fn cram_cram_io_c_4089_cram_flush_container2(
    fd: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let cl = c.cast::<cram_container_layout>();
    let fp = (*fdl).fp;

    if (*cl).curr_slice > 0 && (*cl).slices.is_null() {
        return -1;
    }

    let c_offset = crate::htslib_rs::hfile::htslib_hfile_h_155_htell(fp);

    if 0 != cram_write_container(fd, c) {
        return -1;
    }

    let hdr_size = crate::htslib_rs::hfile::htslib_hfile_h_155_htell(fp) - c_offset;

    if 0 != cram_write_block(&mut *fdl, &mut *(*cl).comp_hdr_block.cast::<cram_block_layout>()) {
        return -1;
    }

    let mut file_offset = crate::htslib_rs::hfile::htslib_hfile_h_155_htell(fp);

    let mut i: c_int = 0;
    while i < (*cl).curr_slice {
        let s = *(*cl).slices.offset(i as isize);
        let spos = file_offset - c_offset - hdr_size;

        if 0 != cram_write_block(&mut *fdl, &mut *(*s).hdr_block.cast::<cram_block_layout>()) {
            return -1;
        }

        let num_blocks = (*(*s).hdr).num_blocks;
        let mut j: c_int = 0;
        while j < num_blocks {
            let blk = *(*s).block.offset(j as isize);
            if 0 != cram_write_block(&mut *fdl, &mut *blk.cast::<cram_block_layout>()) {
                return -1;
            }
            j += 1;
        }

        file_offset = crate::htslib_rs::hfile::htslib_hfile_h_155_htell(fp);
        let sz = file_offset - c_offset - hdr_size - spos;

        let idxfp = cram_fd_idxfp_get(fd);
        if !idxfp.is_null() {
            let rc = cram_index_slice(
                fd,
                c,
                &mut *s.cast::<cram_slice_layout>(),
                &mut *idxfp,
                c_offset,
                spos,
                sz,
            );
            if rc < 0 {
                return -1;
            }
        }

        i += 1;
    }

    0
}
// original: cram_flush_container_mt (htslib/cram/cram_io.c:4275)
//
// Single-threaded path only — the MT pool path is forfeited (see
// cram_flush_bridge for the rationale; semantically equivalent on-disk).
pub unsafe fn cram_cram_io_c_4275_cram_flush_container_mt(
    fd: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    // The metrics_lock / reset_metrics block (cram_io.c:4288-4294) only
    // matters across MT job boundaries; we skip it.
    cram_cram_io_c_4143_cram_flush_container(fd, c)
}
// original: free_bam_list (htslib/cram/cram_io.c:3697)
//
// Frees each `bam1_t *` slot in `bams[0..max_rec]`, then frees the array
// itself. Byte-for-byte equivalent to the C original.
pub unsafe fn cram_cram_io_c_3697_free_bam_list(
    bams: *mut *mut crate::htslib_rs::sam::bam1_t,
    max_rec: c_int,
) {
    let mut i: c_int = 0;
    while i < max_rec {
        crate::htslib_rs::sam::bam_destroy1(*bams.offset(i as isize));
        i += 1;
    }
    free(bams.cast());
}
/// original: cram_free_container (htslib/cram/cram_io.c:3705)
pub unsafe fn cram_cram_io_c_3705_cram_free_container(c: *mut cram_container) {
    if c.is_null() {
        return;
    }
    let c = c.cast::<cram_container_layout>();

    if !(*c).refs_used.is_null() {
        free((*c).refs_used.cast());
    }
    if !(*c).landmark.is_null() {
        free((*c).landmark.cast());
    }
    if !(*c).comp_hdr.is_null() {
        cram_cram_io_c_4356_cram_free_compression_header((*c).comp_hdr.cast());
    }
    if !(*c).comp_hdr_block.is_null() {
        cram_free_block((*c).comp_hdr_block.cast());
    }

    // Free the slices; filled out by encoder only.
    if !(*c).slices.is_null() {
        for i in 0..(*c).max_slice {
            let sp = *(*c).slices.add(i as usize);
            if !sp.is_null() {
                cram_cram_io_c_4421_cram_free_slice(sp.cast());
            }
            if sp == (*c).slice {
                (*c).slice = std::ptr::null_mut();
            }
        }
        free((*c).slices.cast());
    }

    // Free the current slice; set by both encoder & decoder.
    if !(*c).slice.is_null() {
        cram_cram_io_c_4421_cram_free_slice((*c).slice.cast());
        (*c).slice = std::ptr::null_mut();
    }

    let mut id = DS_RN;
    while id < DS_TN {
        if let Some(b) = (*c).stats[id as usize].take() {
            cram_cram_stats_c_223_cram_stats_free(b);
        }
        id += 1;
    }

    // tags_used: a kh_init'd (empty) m_tagmap.  For containers produced by
    // cram_new_container / cram_read_container it carries no entries (the codec-
    // freeing loop in the C original only triggers after cram_encode_container
    // populates it), so kh_destroy reduces to freeing the (NULL) bucket arrays
    // and the table itself.  See report note re: populated write-path tables.
    if !(*c).tags_used.is_null() {
        let h = (*c).tags_used;
        if !(*h).flags.is_null() {
            free((*h).flags.cast());
        }
        if !(*h).keys.is_null() {
            free((*h).keys.cast());
        }
        if !(*h).vals.is_null() {
            free((*h).vals.cast());
        }
        free(h.cast());
    }

    if (*c).ref_free != 0 && !(*c).ref_.is_null() {
        free((*c).ref_.cast());
    }

    if !(*c).bams.is_null() {
        cram_cram_io_c_3695_free_bam_list((*c).bams, (*c).max_c_rec);
    }

    drop(Box::from_raw(c));
}
/// original: cram_read_container (htslib/cram/cram_io.c:3788)
/// Reads a container header. Returns the container on success, NULL on failure
/// or when no container is left (fd->err == 0).
pub unsafe fn cram_cram_io_c_3788_cram_read_container(
    fd: *mut crate::htslib_rs::hts::cram_fd,
) -> *mut cram_container {
    let fdl = fd.cast::<cram_fd_layout>();
    let major = (*fdl).version >> 8;
    let minor = (*fdl).version & 0xff;

    (*fdl).err = 0;
    (*fdl).eof = 0;

    // Stack-local scratch container header (`cram_container c2` in C).
    let mut c2: cram_container_layout = std::mem::zeroed();
    let vv_ptr = &(*fdl).vv as *const varint_vec_layout;
    let vv = &*vv_ptr;
    let mut crc: u32 = 0;
    let mut rd: usize = 0;
    let mut s: c_int;

    if major == 1 {
        s = vv_decode32(vv, fd, &mut c2.length, &mut crc);
        if s == -1 {
            (*fdl).eof = if (*fdl).empty_container != 0 { 1 } else { 2 };
            return std::ptr::null_mut();
        }
        rd += s as usize;
    } else if major < 4 {
        s = int32_decode(&mut *fdl, &mut c2.length);
        if s == -1 {
            if major == 2 && minor == 0 {
                (*fdl).eof = 1; // EOF blocks arrived in v2.1
            } else {
                (*fdl).eof = if (*fdl).empty_container != 0 { 1 } else { 2 };
            }
            return std::ptr::null_mut();
        }
        rd += s as usize;
        let len = c2.length; // little-endian on disk; le_int4 is identity on LE
        crc = crate::htslib_rs::bgzf::hts_crc32(0, (&len as *const i32).cast(), 4);
    } else {
        s = vv_decode32(vv, fd, &mut c2.length, &mut crc);
        if s == -1 {
            (*fdl).eof = if (*fdl).empty_container != 0 { 1 } else { 2 };
            return std::ptr::null_mut();
        }
        rd += s as usize;
    }

    s = vv_decode32s(vv, fd, &mut c2.ref_seq_id, &mut crc);
    if s == -1 {
        return std::ptr::null_mut();
    }
    rd += s as usize;

    if major >= 4 {
        let mut i64v: i64 = 0;
        s = vv_decode64(vv, fd, &mut i64v, &mut crc);
        if s == -1 {
            return std::ptr::null_mut();
        }
        rd += s as usize;
        c2.ref_seq_start = i64v;
        s = vv_decode64(vv, fd, &mut i64v, &mut crc);
        if s == -1 {
            return std::ptr::null_mut();
        }
        rd += s as usize;
        c2.ref_seq_span = i64v;
    } else {
        let mut i32v: i32 = 0;
        s = vv_decode32(vv, fd, &mut i32v, &mut crc);
        if s == -1 {
            return std::ptr::null_mut();
        }
        rd += s as usize;
        c2.ref_seq_start = i32v as i64;
        s = vv_decode32(vv, fd, &mut i32v, &mut crc);
        if s == -1 {
            return std::ptr::null_mut();
        }
        rd += s as usize;
        c2.ref_seq_span = i32v as i64;
    }

    s = vv_decode32(vv, fd, &mut c2.num_records, &mut crc);
    if s == -1 {
        return std::ptr::null_mut();
    }
    rd += s as usize;

    if major == 1 {
        c2.record_counter = 0;
        c2.num_bases = 0;
    } else {
        if major >= 3 {
            s = vv_decode64(vv, fd, &mut c2.record_counter, &mut crc);
            if s == -1 {
                return std::ptr::null_mut();
            }
            rd += s as usize;
        } else {
            let mut i32v: i32 = 0;
            s = vv_decode32(vv, fd, &mut i32v, &mut crc);
            if s == -1 {
                return std::ptr::null_mut();
            }
            rd += s as usize;
            c2.record_counter = i32v as i64;
        }
        s = vv_decode64(vv, fd, &mut c2.num_bases, &mut crc);
        if s == -1 {
            return std::ptr::null_mut();
        }
        rd += s as usize;
    }

    s = vv_decode32(vv, fd, &mut c2.num_blocks, &mut crc);
    if s == -1 {
        return std::ptr::null_mut();
    }
    rd += s as usize;
    s = vv_decode32(vv, fd, &mut c2.num_landmarks, &mut crc);
    if s == -1 {
        return std::ptr::null_mut();
    }
    rd += s as usize;

    // c2.num_landmarks < 0 || c2.num_landmarks >= SIZE_MAX / sizeof(int32_t)
    if c2.num_landmarks < 0
        || (c2.num_landmarks as u64) >= (usize::MAX as u64) / (std::mem::size_of::<i32>() as u64)
    {
        return std::ptr::null_mut();
    }

    let c = calloc(1, std::mem::size_of::<cram_container_layout>() as u64)
        .cast::<cram_container_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    // *c = c2;
    std::ptr::write(c, c2);

    if (*c).num_landmarks != 0 {
        (*c).landmark =
            calloc((*c).num_landmarks as u64, std::mem::size_of::<i32>() as u64).cast::<i32>();
        if (*c).landmark.is_null() {
            (*fdl).err = *__errno_location();
            cram_cram_io_c_3705_cram_free_container(c.cast());
            return std::ptr::null_mut();
        }
    }
    for i in 0..(*c).num_landmarks {
        s = vv_decode32(vv, fd, (*c).landmark.add(i as usize), &mut crc);
        if s == -1 {
            cram_cram_io_c_3705_cram_free_container(c.cast());
            return std::ptr::null_mut();
        }
        rd += s as usize;
    }

    if major >= 3 {
        if int32_decode(&mut *fdl, &mut *(&mut (*c).crc32 as *mut u32).cast::<i32>()) == -1 {
            cram_cram_io_c_3705_cram_free_container(c.cast());
            return std::ptr::null_mut();
        }
        rd += 4;
        if crc != (*c).crc32 {
            hts_log_cstr(
                HTS_LOG_ERROR,
                c"cram_read_container".as_ptr(),
                c"Container header CRC32 failure".as_ptr(),
            );
            cram_cram_io_c_3705_cram_free_container(c.cast());
            return std::ptr::null_mut();
        }
    }

    (*c).offset = rd;
    (*c).slices = std::ptr::null_mut();
    (*c).slice = std::ptr::null_mut();
    (*c).curr_slice = 0;
    (*c).max_slice = (*c).num_landmarks;
    (*c).slice_rec = 0;
    (*c).curr_rec = 0;
    (*c).max_rec = 0;

    if (*c).ref_seq_id == -2 {
        (*c).multi_seq = 1;
        (*fdl).multi_seq = 1;
    }

    (*fdl).empty_container =
        if (*c).num_records == 0 && (*c).ref_seq_id == -1 && (*c).ref_seq_start == 0x454f46 {
            1
        } else {
            0
        };

    c.cast()
}
/// original: cram_free_slice (htslib/cram/cram_io.c:4421)
pub unsafe fn cram_cram_io_c_4421_cram_free_slice(s: *mut cram_slice) {
    if s.is_null() {
        return;
    }
    let s = s.cast::<cram_slice_layout>();

    if !(*s).hdr_block.is_null() {
        cram_free_block((*s).hdr_block.cast());
    }

    if !(*s).block.is_null() {
        if !(*s).hdr.is_null() {
            let n = (*(*s).hdr).num_blocks;
            for i in 0..n {
                let bi = *(*s).block.add(i as usize);
                if i > 0 && bi == *(*s).block.add(0) {
                    continue;
                }
                cram_free_block(bi.cast());
            }
        }
        free((*s).block.cast());
    }

    // Normally already copied into s->block[], but possibly still here on a
    // partial cram_encode_slice failure.
    for i in 0..(*s).naux_block {
        cram_free_block((*(*s).aux_block.add(i as usize)).cast());
    }

    if !(*s).block_by_id.is_null() {
        free((*s).block_by_id.cast());
    }
    if !(*s).hdr.is_null() {
        cram_cram_io_c_4409_cram_free_slice_header((*s).hdr.cast());
    }
    if !(*s).seqs_blk.is_null() {
        cram_free_block((*s).seqs_blk.cast());
    }
    if !(*s).qual_blk.is_null() {
        cram_free_block((*s).qual_blk.cast());
    }
    if !(*s).name_blk.is_null() {
        cram_free_block((*s).name_blk.cast());
    }
    if !(*s).aux_blk.is_null() {
        cram_free_block((*s).aux_blk.cast());
    }
    if !(*s).base_blk.is_null() {
        cram_free_block((*s).base_blk.cast());
    }
    if !(*s).soft_blk.is_null() {
        cram_free_block((*s).soft_blk.cast());
    }
    if !(*s).cigar.is_null() {
        free((*s).cigar.cast());
    }
    if !(*s).crecs.is_null() {
        free((*s).crecs.cast());
    }
    if !(*s).features.is_null() {
        free((*s).features.cast());
    }
    if !(*s).tn.is_null() {
        free((*s).tn.cast());
    }
    if let Some(p) = (*s).pair_keys.take() {
        cram_string_alloc_c_103_string_pool_destroy(p);
    }
    // pair[0]/pair[1] are kh_init(m_s2i) tables.  Slices produced by
    // cram_read_slice never allocate them (they stay NULL from calloc); those
    // produced by cram_new_slice carry empty tables, so kh_destroy reduces to
    // freeing the (NULL) bucket arrays and the table.  See report note.
    for k in 0..2usize {
        let h = (*s).pair[k];
        if !h.is_null() {
            if !(*h).flags.is_null() {
                free((*h).flags.cast());
            }
            if !(*h).keys.is_null() {
                free((*h).keys.cast());
            }
            if !(*h).vals.is_null() {
                free((*h).vals.cast());
            }
            free(h.cast());
        }
    }
    if !(*s).aux_block.is_null() {
        free((*s).aux_block.cast());
    }

    drop(Box::from_raw(s));
}
/// original: cram_new_slice (htslib/cram/cram_io.c:4506)
/// Creates a new empty in-memory slice (write path).
pub unsafe fn cram_cram_io_c_4506_cram_new_slice(
    type_: cram_content_type,
    nrecs: c_int,
) -> *mut cram_slice {
    let s = Box::into_raw(Box::new(cram_slice_layout {
        hdr: std::ptr::null_mut(),
        hdr_block: std::ptr::null_mut(),
        block: std::ptr::null_mut(),
        block_by_id: std::ptr::null_mut(),
        last_apos: 0,
        max_apos: 0,
        crecs: std::ptr::null_mut(),
        cigar: std::ptr::null_mut(),
        cigar_alloc: 0,
        ncigar: 0,
        features: std::ptr::null_mut(),
        nfeatures: 0,
        afeatures: 0,
        tn: std::ptr::null_mut(),
        n_tn: 0,
        a_tn: 0,
        name_blk: std::ptr::null_mut(),
        seqs_blk: std::ptr::null_mut(),
        qual_blk: std::ptr::null_mut(),
        base_blk: std::ptr::null_mut(),
        soft_blk: std::ptr::null_mut(),
        aux_blk: std::ptr::null_mut(),
        pair_keys: None,
        pair: [std::ptr::null_mut(); 2],
        ref_: std::ptr::null_mut(),
        ref_start: 0,
        ref_end: 0,
        ref_id: 0,
        naux_block: 0,
        aux_block: std::ptr::null_mut(),
        data_series: 0,
        decode_md: 0,
        max_rec: 0,
        curr_rec: 0,
        slice_num: 0,
    }));

    (*s).hdr = calloc(1, std::mem::size_of::<cram_block_slice_hdr_layout>() as u64)
        .cast::<cram_block_slice_hdr_layout>();
    if (*s).hdr.is_null() {
        cram_cram_io_c_4421_cram_free_slice(s.cast());
        return std::ptr::null_mut();
    }
    (*(*s).hdr).content_type = type_;

    (*s).hdr_block = std::ptr::null_mut();
    (*s).block = std::ptr::null_mut();
    (*s).block_by_id = std::ptr::null_mut();
    (*s).last_apos = 0;

    (*s).crecs = calloc(
        nrecs as u64,
        std::mem::size_of::<cram_record_layout>() as u64,
    )
    .cast::<cram_record_layout>();
    if (*s).crecs.is_null() {
        cram_cram_io_c_4421_cram_free_slice(s.cast());
        return std::ptr::null_mut();
    }
    (*s).cigar_alloc = 1024;
    (*s).cigar = calloc((*s).cigar_alloc as u64, std::mem::size_of::<u32>() as u64).cast::<u32>();
    if (*s).cigar.is_null() {
        cram_cram_io_c_4421_cram_free_slice(s.cast());
        return std::ptr::null_mut();
    }
    (*s).ncigar = 0;

    (*s).seqs_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, 0).cast();
    (*s).qual_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_QS).cast();
    (*s).name_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_RN).cast();
    (*s).aux_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_AUX).cast();
    (*s).base_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_IN).cast();
    (*s).soft_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_SC).cast();
    if (*s).seqs_blk.is_null()
        || (*s).qual_blk.is_null()
        || (*s).name_blk.is_null()
        || (*s).aux_blk.is_null()
        || (*s).base_blk.is_null()
        || (*s).soft_blk.is_null()
    {
        cram_cram_io_c_4421_cram_free_slice(s.cast());
        return std::ptr::null_mut();
    }

    (*s).features = std::ptr::null_mut();
    (*s).nfeatures = 0;
    (*s).afeatures = 0;

    (*s).tn = std::ptr::null_mut();
    (*s).n_tn = 0;
    (*s).a_tn = 0;

    (*s).pair_keys = Some(cram_string_alloc_c_55_string_pool_create(8192));
    // s->pair[0] = kh_init(m_s2i); s->pair[1] = kh_init(m_s2i);
    (*s).pair[0] =
        calloc(1, std::mem::size_of::<kh_generic_layout>() as u64).cast::<kh_generic_layout>();
    (*s).pair[1] =
        calloc(1, std::mem::size_of::<kh_generic_layout>() as u64).cast::<kh_generic_layout>();
    if (*s).pair[0].is_null() || (*s).pair[1].is_null() {
        cram_cram_io_c_4421_cram_free_slice(s.cast());
        return std::ptr::null_mut();
    }

    s.cast()
}
/// original: cram_read_slice (htslib/cram/cram_io.c:4568)
/// Loads an entire slice (read path).
struct CramReadSliceOwner {
    ptr: *mut cram_slice_layout,
    header_block: *mut cram_block,
}

impl CramReadSliceOwner {
    fn new(ptr: *mut cram_slice_layout, header_block: *mut cram_block) -> Self {
        Self { ptr, header_block }
    }

    fn release(mut self) -> *mut cram_slice {
        let ptr = self.ptr;
        self.ptr = std::ptr::null_mut();
        self.header_block = std::ptr::null_mut();
        ptr.cast()
    }
}

impl Drop for CramReadSliceOwner {
    fn drop(&mut self) {
        unsafe {
            if !self.header_block.is_null() {
                cram_free_block(self.header_block);
            }
            if !self.ptr.is_null() {
                (*self.ptr).hdr_block = std::ptr::null_mut();
                cram_cram_io_c_4421_cram_free_slice(self.ptr.cast());
            }
        }
    }
}

pub unsafe fn cram_cram_io_c_4568_cram_read_slice(
    fd: *mut crate::htslib_rs::hts::cram_fd,
) -> *mut cram_slice {
    let Some(fdl) = fd.cast::<cram_fd_layout>().as_mut() else {
        return std::ptr::null_mut();
    };
    let b = cram_read_block(fdl);
    let s = calloc(1, std::mem::size_of::<cram_slice_layout>() as u64).cast::<cram_slice_layout>();
    let slice_owner = CramReadSliceOwner::new(s, b);

    if b.is_null() || s.is_null() {
        return std::ptr::null_mut();
    }

    (*s).hdr_block = b.cast();
    let bl = b.cast::<cram_block_layout>();
    match (*bl).content_type {
        x if x == CRAM_CONTENT_TYPE_MAPPED_SLICE || x == CRAM_CONTENT_TYPE_UNMAPPED_SLICE => {
            (*s).hdr = cram_cram_decode_c_955_cram_decode_slice_header(fd.cast(), b.cast()).cast();
            if (*s).hdr.is_null() {
                return std::ptr::null_mut();
            }
        }
        _ => {
            hts_log_cstr(
                HTS_LOG_ERROR,
                c"cram_read_slice".as_ptr(),
                c"Unexpected block type in slice".as_ptr(),
            );
            return std::ptr::null_mut();
        }
    }

    if (*(*s).hdr).num_blocks < 1 {
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"cram_read_slice".as_ptr(),
            c"Slice does not include any data blocks".as_ptr(),
        );
        return std::ptr::null_mut();
    }

    let n = (*(*s).hdr).num_blocks;
    (*s).block = calloc(
        n as u64,
        std::mem::size_of::<*mut cram_block_layout>() as u64,
    )
    .cast::<*mut cram_block_layout>();
    if (*s).block.is_null() {
        return std::ptr::null_mut();
    }

    let mut max_id: i32 = 0;
    let mut min_id: i32 = i32::MAX;
    for i in 0..n {
        let blk = cram_read_block(fdl);
        *(*s).block.add(i as usize) = blk.cast();
        if blk.is_null() {
            return std::ptr::null_mut();
        }
        let blkl = blk.cast::<cram_block_layout>();
        if (*blkl).content_type == CRAM_CONTENT_TYPE_EXTERNAL {
            if max_id < (*blkl).content_id {
                max_id = (*blkl).content_id;
            }
            if min_id > (*blkl).content_id {
                min_id = (*blkl).content_id;
            }
        }
    }

    (*s).block_by_id = calloc(512, std::mem::size_of::<*mut cram_block_layout>() as u64)
        .cast::<*mut cram_block_layout>();
    if (*s).block_by_id.is_null() {
        return std::ptr::null_mut();
    }

    for i in 0..n {
        let blk = *(*s).block.add(i as usize);
        let blkl = blk.cast::<cram_block_layout>();
        if (*blkl).content_type != CRAM_CONTENT_TYPE_EXTERNAL {
            continue;
        }
        let mut v = (*blkl).content_id as u32;
        if v >= 256 {
            v = 256 + v % 251;
        }
        *(*s).block_by_id.add(v as usize) = blk;
    }

    // Initialise encoding/decoding tables.
    (*s).cigar_alloc = 1024;
    (*s).cigar = calloc((*s).cigar_alloc as u64, std::mem::size_of::<u32>() as u64).cast::<u32>();
    if (*s).cigar.is_null() {
        return std::ptr::null_mut();
    }
    (*s).ncigar = 0;

    (*s).seqs_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, 0).cast();
    (*s).qual_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_QS).cast();
    (*s).name_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_RN).cast();
    (*s).aux_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_AUX).cast();
    (*s).base_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_IN).cast();
    (*s).soft_blk = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_EXTERNAL, DS_SC).cast();
    if (*s).seqs_blk.is_null()
        || (*s).qual_blk.is_null()
        || (*s).name_blk.is_null()
        || (*s).aux_blk.is_null()
        || (*s).base_blk.is_null()
        || (*s).soft_blk.is_null()
    {
        return std::ptr::null_mut();
    }

    (*s).crecs = std::ptr::null_mut();
    (*s).last_apos = (*(*s).hdr).ref_seq_start;
    (*s).decode_md = fdl.decode_md;

    slice_owner.release()
}
/// original: cram_free_slice_header (htslib/cram/cram_io.c:4409)
pub unsafe fn cram_cram_io_c_4409_cram_free_slice_header(hdr: *mut cram_block_slice_hdr) {
    if hdr.is_null() {
        return;
    }
    let hdr = hdr.cast::<cram_block_slice_hdr_layout>();
    if !(*hdr).block_content_ids.is_null() {
        free((*hdr).block_content_ids.cast());
    }
    free(hdr.cast());
}
pub fn cram_cram_io_c_2341_cram_block_method2str(m: c_int) -> *mut c_char {
    let s: &'static [u8] = match m {
        -1 => b"?\0",
        0 => b"RAW\0",
        1 => b"GZIP\0",
        2 => b"BZIP2\0",
        3 => b"LZMA\0",
        4 => b"RANS0\0",
        5 => b"RANS_PR0\0",
        6 => b"ARITH_PR0\0",
        7 => b"FQZ\0",
        8 => b"TOK3_R\0",
        11 => b"GZIP_RLE\0",
        12 => b"GZIP_1\0",
        13 => b"FQZ_b\0",
        14 => b"FQZ_c\0",
        15 => b"FQZ_d\0",
        16 => b"RANS1\0",
        17 => b"RANS_PR1\0",
        18 => b"RANS_PR64\0",
        19 => b"RANS_PR9\0",
        20 => b"RANS_PR128\0",
        21 => b"RANS_PR129\0",
        22 => b"RANS_PR192\0",
        23 => b"RANS_PR193\0",
        24 => b"TOK3_A\0",
        25 => b"ARITH_PR1\0",
        26 => b"ARITH_PR64\0",
        27 => b"ARITH_PR9\0",
        28 => b"ARITH_PR128\0",
        29 => b"ARITH_PR129\0",
        30 => b"ARITH_PR192\0",
        31 => b"ARITH_PR193\0",
        _ => b"?\0",
    };
    s.as_ptr().cast::<c_char>().cast_mut()
}
pub fn cram_cram_io_c_2378_cram_content_type2str(t: cram_content_type) -> *mut c_char {
    let s: &'static [u8] = match t {
        0 => b"FILE_HEADER\0",
        1 => b"COMPRESSION_HEADER\0",
        2 => b"MAPPED_SLICE\0",
        3 => b"UNMAPPED_SLICE\0",
        4 => b"EXTERNAL\0",
        5 => b"CORE\0",
        _ => b"?\0",
    };
    s.as_ptr().cast::<c_char>().cast_mut()
}
pub unsafe fn cram_cram_io_c_2873_is_directory(fn_: *mut c_char) -> c_int {
    let mut buf = std::mem::MaybeUninit::<libc::stat>::uninit();
    if libc::stat(fn_, buf.as_mut_ptr()) != 0 {
        return 0;
    }
    let buf = buf.assume_init();
    crate::htslib_rs::c_compat::stat_mode_matches(buf.st_mode, libc::S_IFMT, libc::S_IFDIR) as c_int
}
pub unsafe fn cram_cram_io_c_2884_expand_cache_path(
    path: *mut c_char,
    dir: *mut c_char,
    fn_: *const c_char,
) -> c_int {
    if path.is_null() || dir.is_null() || fn_.is_null() {
        return -1;
    }

    let mut dir_len = 0usize;
    while *dir.add(dir_len) != 0 {
        dir_len += 1;
    }
    let mut fn_len = 0usize;
    while *fn_.add(fn_len) != 0 {
        fn_len += 1;
    }

    let dir_bytes = std::slice::from_raw_parts(dir.cast::<u8>(), dir_len);
    let fn_bytes = std::slice::from_raw_parts(fn_.cast::<u8>(), fn_len);
    let Some(bytes) = expand_cache_path_bytes(
        dir_bytes,
        fn_bytes,
        crate::htslib_rs::c_compat::PATH_MAX as usize,
    ) else {
        return -1;
    };

    std::ptr::copy_nonoverlapping(bytes.as_ptr(), path.cast::<u8>(), bytes.len());
    *path.add(bytes.len()) = 0;
    0
}

fn expand_cache_path_bytes(dir: &[u8], mut file: &[u8], cap_with_nul: usize) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(dir.len().saturating_add(file.len()).saturating_add(1));
    let mut dir_i = 0usize;

    while let Some(rel_percent) = dir[dir_i..].iter().position(|&b| b == b'%') {
        let percent = dir_i + rel_percent;
        out.extend_from_slice(&dir[dir_i..percent]);
        let mut cp = percent + 1;

        match dir.get(cp).copied() {
            Some(b's') => {
                out.extend_from_slice(file);
                file = &[];
                cp += 1;
            }
            Some(b'0'..=b'9') => {
                let digits_start = cp;
                let mut n = 0usize;
                while let Some(digit @ b'0'..=b'9') = dir.get(cp).copied() {
                    n = n.saturating_mul(10).saturating_add((digit - b'0') as usize);
                    cp += 1;
                }
                if dir.get(cp).copied() == Some(b's') {
                    let take = n.min(file.len());
                    out.extend_from_slice(&file[..take]);
                    file = &file[take..];
                    cp += 1;
                } else {
                    out.push(b'%');
                    out.push(dir[digits_start]);
                    cp = digits_start + 1;
                }
            }
            Some(byte) => {
                out.push(b'%');
                out.push(byte);
                cp += 1;
            }
            None => {
                out.push(b'%');
            }
        }

        if out.len() >= cap_with_nul {
            return None;
        }
        dir_i = cp;
    }

    out.extend_from_slice(&dir[dir_i..]);
    if !file.is_empty() && !out.is_empty() && *out.last().unwrap() != b'/' {
        out.push(b'/');
    }
    out.extend_from_slice(file);

    if out.len() >= cap_with_nul {
        None
    } else {
        Some(out)
    }
}
pub unsafe fn cram_cram_io_c_2947_mkdir_prefix(path: *mut c_char, mode: c_int) {
    let cp = libc::strrchr(path, b'/' as c_int);
    if cp.is_null() {
        return;
    }

    *cp = 0;
    if cram_cram_io_c_2873_is_directory(path) != 0 {
        *cp = b'/' as c_char;
        return;
    }

    #[cfg(windows)]
    let mkdir_ret = libc::mkdir(path);
    #[cfg(not(windows))]
    let mkdir_ret = libc::mkdir(path, mode as libc::mode_t);

    if mkdir_ret == 0 {
        crate::htslib_rs::c_compat::chmod(path, mode);
        *cp = b'/' as c_char;
        return;
    }

    cram_cram_io_c_2947_mkdir_prefix(path, mode);
    #[cfg(windows)]
    libc::mkdir(path);
    #[cfg(not(windows))]
    libc::mkdir(path, mode as libc::mode_t);
    crate::htslib_rs::c_compat::chmod(path, mode);
    *cp = b'/' as c_char;
}
pub unsafe fn cram_cram_io_c_3695_free_bam_list(bams: *mut *mut bam1_t, max_rec: c_int) {
    for i in 0..max_rec {
        bam_destroy1(*bams.add(i as usize));
    }
    free(bams.cast());
}
pub unsafe fn cram_cram_io_c_4850_full_path(out: *mut c_char, in_: *mut c_char) {
    if out.is_null() || in_.is_null() {
        return;
    }
    let mut in_l = 0usize;
    while *in_.add(in_l) != 0 {
        in_l += 1;
    }
    let input = std::slice::from_raw_parts(in_.cast::<u8>(), in_l);
    let path_max = crate::htslib_rs::c_compat::PATH_MAX as usize;

    let write_output = |bytes: &[u8]| {
        let n = bytes.len().min(path_max.saturating_sub(1));
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.cast::<u8>(), n);
        *out.add(n) = 0;
    };

    if hisremote(in_) != 0 {
        if in_l > path_max {
            let msg = std::ffi::CString::new(format!(
                "Reference path is longer than {}",
                crate::htslib_rs::c_compat::PATH_MAX
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_ERROR, c"full_path".as_ptr(), msg.as_ptr());
            return;
        }
        write_output(input);
        return;
    }

    let is_windows_abs = input.len() > 3
        && input[0].to_ascii_uppercase().is_ascii_uppercase()
        && input[1] == b':'
        && (input[2] == b'/' || input[2] == b'\\');

    if input.first().copied() == Some(b'/') || is_windows_abs {
        write_output(input);
    } else {
        if crate::htslib_rs::c_compat::getcwd(out, crate::htslib_rs::c_compat::PATH_MAX as usize)
            .is_null()
        {
            write_output(input);
            return;
        }

        let mut cwd_len = 0usize;
        while *out.add(cwd_len) != 0 {
            cwd_len += 1;
        }
        let cwd = std::slice::from_raw_parts(out.cast::<u8>(), cwd_len);
        if cwd.len() + 1 + input.len() >= path_max {
            write_output(input);
            return;
        }

        let mut full = Vec::with_capacity(cwd.len() + 1 + input.len());
        full.extend_from_slice(cwd);
        full.push(b'/');
        full.extend_from_slice(input);
        write_output(&full);
    }
}
