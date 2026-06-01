// Functions translated from htslib/cram/mFILE.c.
// Extracted from src/cram/mod.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int, c_void};

use crate::htslib_rs::c_compat::{__errno_location, free, malloc, memcpy, realloc, EINVAL};

use super::*;

pub(super) static mut M_CHANNEL: [*mut mFILE; 3] = [std::ptr::null_mut(); 3];
pub(super) static mut DONE_STDIN: c_int = 0;

pub unsafe fn cram_mFILE_c_75_mfload(
    fp: *mut libc::FILE,
    mut fn_: *const c_char,
    size: *mut usize,
    _binary: c_int,
) -> *mut c_char {
    let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
    let mut data = std::ptr::null_mut::<c_char>();
    let mut allocated = 0usize;
    let mut used = 0usize;
    let mut bufsize = 8192usize;

    if !fn_.is_null() && libc::stat(fn_, sb.as_mut_ptr()) != -1 {
        let sb = sb.assume_init();
        allocated = sb.st_size as usize;
        data = malloc(allocated as u64).cast::<c_char>();
        if data.is_null() {
            return std::ptr::null_mut();
        }
        bufsize = sb.st_size as usize;

        loop {
            if used + bufsize > allocated {
                allocated += bufsize;
                let datan = realloc(data.cast(), allocated as u64).cast::<c_char>();
                if datan.is_null() {
                    free(data.cast());
                    return std::ptr::null_mut();
                }
                data = datan;
            }
            let len = libc::fread(data.add(used).cast(), 1, allocated - used, fp);
            if len > 0 {
                used += len;
            }
            if libc::feof(fp) != 0 || used >= sb.st_size as usize {
                break;
            }
        }
    } else {
        fn_ = std::ptr::null();
        loop {
            if used + bufsize > allocated {
                allocated += bufsize;
                let datan = realloc(data.cast(), allocated as u64).cast::<c_char>();
                if datan.is_null() {
                    free(data.cast());
                    return std::ptr::null_mut();
                }
                data = datan;
            }
            let len = libc::fread(data.add(used).cast(), 1, allocated - used, fp);
            if len > 0 {
                used += len;
            }
            if libc::feof(fp) != 0 || !fn_.is_null() {
                break;
            }
        }
    }

    *size = used;
    data
}

pub unsafe fn cram_mFILE_c_127_mfmmap(
    mf: *mut mFILE,
    fp: *mut libc::FILE,
    fn_: *const c_char,
) -> c_int {
    let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
    if libc::stat(fn_, sb.as_mut_ptr()) != 0 {
        return -1;
    }
    let sb = sb.assume_init();
    (*mf).size = sb.st_size as usize;
    let data = libc::mmap(
        std::ptr::null_mut(),
        (*mf).size,
        libc::PROT_READ,
        libc::MAP_SHARED,
        libc::fileno(fp),
        0,
    );
    if data.is_null() || data == libc::MAP_FAILED {
        return -1;
    }

    (*mf).data = data.cast::<c_char>();
    (*mf).alloced = 0;
    0
}

pub unsafe fn cram_mFILE_c_151_mstdin() -> *mut mFILE {
    if !M_CHANNEL[0].is_null() {
        return M_CHANNEL[0];
    }

    M_CHANNEL[0] = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
    if M_CHANNEL[0].is_null() {
        return std::ptr::null_mut();
    }
    (*M_CHANNEL[0]).fp = HTSLIB_STDIN;
    M_CHANNEL[0]
}

pub unsafe fn cram_mFILE_c_161_init_mstdin() {
    if DONE_STDIN != 0 {
        return;
    }

    (*M_CHANNEL[0]).data =
        cram_mFILE_c_75_mfload(HTSLIB_STDIN, std::ptr::null(), &mut (*M_CHANNEL[0]).size, 1);
    (*M_CHANNEL[0]).mode = MF_READ;
    DONE_STDIN = 1;
}

pub unsafe fn cram_mFILE_c_176_mstdout() -> *mut mFILE {
    if !M_CHANNEL[1].is_null() {
        return M_CHANNEL[1];
    }

    M_CHANNEL[1] = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
    if M_CHANNEL[1].is_null() {
        return std::ptr::null_mut();
    }
    (*M_CHANNEL[1]).fp = HTSLIB_STDOUT;
    (*M_CHANNEL[1]).mode = MF_WRITE;
    M_CHANNEL[1]
}

pub unsafe fn cram_mFILE_c_192_mstderr() -> *mut mFILE {
    if !M_CHANNEL[2].is_null() {
        return M_CHANNEL[2];
    }

    M_CHANNEL[2] = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
    if M_CHANNEL[2].is_null() {
        return std::ptr::null_mut();
    }
    (*M_CHANNEL[2]).fp = HTSLIB_STDERR;
    (*M_CHANNEL[2]).mode = MF_WRITE;
    M_CHANNEL[2]
}

pub unsafe fn cram_mFILE_c_207_mfcreate(data: *mut c_char, size: c_int) -> *mut mFILE {
    let mf = malloc(std::mem::size_of::<mFILE>() as u64).cast::<mFILE>();
    if mf.is_null() {
        return std::ptr::null_mut();
    }
    (*mf).fp = std::ptr::null_mut();
    (*mf).data = data;
    (*mf).alloced = size as usize;
    (*mf).size = size as usize;
    (*mf).eof = 0;
    (*mf).offset = 0;
    (*mf).flush_pos = 0;
    (*mf).mode = MF_READ | MF_WRITE;
    mf
}

pub unsafe fn cram_mFILE_c_225_mfrecreate(mf: *mut mFILE, data: *mut c_char, size: c_int) {
    if !(*mf).data.is_null() {
        free((*mf).data.cast());
    }
    (*mf).data = data;
    (*mf).size = size as usize;
    (*mf).alloced = size as usize;
    (*mf).eof = 0;
    (*mf).offset = 0;
    (*mf).flush_pos = 0;
}

pub unsafe fn cram_mFILE_c_246_mfcreate_from(
    path: *const c_char,
    mode_str: *const c_char,
    fp: *mut libc::FILE,
) -> *mut mFILE {
    let mf = cram_mFILE_c_264_mfreopen(path, mode_str, fp);
    if mf.is_null() {
        return std::ptr::null_mut();
    }
    (*mf).fp = std::ptr::null_mut();
    mf
}

pub unsafe fn cram_mFILE_c_264_mfreopen(
    path: *const c_char,
    mode_str: *const c_char,
    fp: *mut libc::FILE,
) -> *mut mFILE {
    let mut r = 0;
    let mut w = 0;
    let mut a = 0;
    let mut b = 0;
    let mut x = 0;
    let mut mode = 0;

    if !libc::strchr(mode_str, b'r' as c_int).is_null() {
        r = 1;
        mode |= MF_READ;
    }
    if !libc::strchr(mode_str, b'w' as c_int).is_null() {
        w = 1;
        mode |= MF_WRITE | MF_TRUNC;
    }
    if !libc::strchr(mode_str, b'a' as c_int).is_null() {
        w = 1;
        a = 1;
        mode |= MF_WRITE | MF_APPEND;
    }
    if !libc::strchr(mode_str, b'b' as c_int).is_null() {
        b = 1;
        mode |= MF_BINARY;
    }
    if !libc::strchr(mode_str, b'x' as c_int).is_null() {
        x = 1;
    }
    if !libc::strchr(mode_str, b'+' as c_int).is_null() {
        w = 1;
        mode |= MF_READ | MF_WRITE;
        if a != 0 {
            r = 1;
        }
    }
    if !libc::strchr(mode_str, b'm' as c_int).is_null() && w == 0 {
        mode |= MF_MMAP;
    }

    let mf;
    if r != 0 {
        mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
        if mf.is_null() {
            return std::ptr::null_mut();
        }
        if (mode & MF_TRUNC) == 0 {
            if (mode & MF_MMAP) != 0 && cram_mFILE_c_127_mfmmap(mf, fp, path) == -1 {
                (*mf).data = std::ptr::null_mut();
                mode &= !MF_MMAP;
            }
            if (*mf).data.is_null() {
                (*mf).data = cram_mFILE_c_75_mfload(fp, path, &mut (*mf).size, b);
                if (*mf).data.is_null() {
                    free(mf.cast());
                    return std::ptr::null_mut();
                }
                (*mf).alloced = (*mf).size;
                if a == 0 {
                    libc::fseek(fp, 0, libc::SEEK_SET);
                }
            }
        }
    } else if w != 0 {
        mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
        if mf.is_null() {
            return std::ptr::null_mut();
        }
    } else {
        return std::ptr::null_mut();
    }

    (*mf).fp = fp;
    (*mf).mode = mode;
    if x != 0 {
        (*mf).mode |= MF_MODEX;
    }
    if a != 0 {
        (*mf).flush_pos = (*mf).size;
        libc::fseek(fp, 0, libc::SEEK_END);
    }

    mf
}

pub unsafe fn cram_mFILE_c_347_mfopen(path: *const c_char, mode: *const c_char) -> *mut mFILE {
    let fp = libc::fopen(path, mode);
    if fp.is_null() {
        return std::ptr::null_mut();
    }
    cram_mFILE_c_264_mfreopen(path, mode, fp)
}

pub unsafe fn cram_mFILE_c_361_mfclose(mf: *mut mFILE) -> c_int {
    if mf.is_null() {
        return -1;
    }
    cram_mFILE_c_607_mfflush(mf);
    if ((*mf).mode & MF_MMAP) != 0 && !(*mf).data.is_null() {
        libc::munmap((*mf).data.cast(), (*mf).size);
        (*mf).data = std::ptr::null_mut();
    }
    if !(*mf).fp.is_null() {
        libc::fclose((*mf).fp);
    }
    cram_mFILE_c_408_mfdestroy(mf);
    0
}

pub unsafe fn cram_mFILE_c_389_mfdetach(mf: *mut mFILE) -> c_int {
    if mf.is_null() {
        return -1;
    }
    if cram_mFILE_c_607_mfflush(mf) != 0 {
        return -1;
    }
    if ((*mf).mode & MF_MMAP) != 0 {
        return -1;
    }
    if !(*mf).fp.is_null() {
        libc::fclose((*mf).fp);
        (*mf).fp = std::ptr::null_mut();
    }
    0
}

pub unsafe fn cram_mFILE_c_408_mfdestroy(mf: *mut mFILE) -> c_int {
    if mf.is_null() {
        return -1;
    }
    if !(*mf).data.is_null() {
        free((*mf).data.cast());
    }
    free(mf.cast());
    0
}

pub unsafe fn cram_mFILE_c_428_mfsteal(mf: *mut mFILE, size_out: *mut usize) -> *mut c_void {
    if mf.is_null() {
        return std::ptr::null_mut();
    }
    let data = (*mf).data;
    if !size_out.is_null() {
        *size_out = (*mf).size;
    }
    if cram_mFILE_c_389_mfdetach(mf) != 0 {
        return std::ptr::null_mut();
    }
    (*mf).data = std::ptr::null_mut();
    cram_mFILE_c_408_mfdestroy(mf);
    data.cast()
}

pub unsafe fn cram_mFILE_c_451_mfseek(
    mf: *mut mFILE,
    offset: libc::c_long,
    whence: c_int,
) -> c_int {
    match whence {
        libc::SEEK_SET => {
            (*mf).offset = offset as usize;
        }
        libc::SEEK_CUR => {
            (*mf).offset = (*mf).offset.wrapping_add(offset as usize);
        }
        libc::SEEK_END => {
            (*mf).offset = (*mf).size.wrapping_add(offset as usize);
        }
        _ => {
            *__errno_location() = EINVAL;
            return -1;
        }
    }

    (*mf).eof = 0;
    0
}

pub unsafe fn cram_mFILE_c_471_mftell(mf: *mut mFILE) -> libc::c_long {
    (*mf).offset as libc::c_long
}

pub unsafe fn cram_mFILE_c_475_mrewind(mf: *mut mFILE) {
    (*mf).offset = 0;
    (*mf).eof = 0;
}

pub unsafe fn cram_mFILE_c_488_mftruncate(mf: *mut mFILE, offset: libc::c_long) {
    (*mf).size = if offset != -1 {
        offset as usize
    } else {
        (*mf).offset
    };
    if (*mf).offset > (*mf).size {
        (*mf).offset = (*mf).size;
    }
}

pub unsafe fn cram_mFILE_c_494_mfeof(mf: *mut mFILE) -> c_int {
    (*mf).eof
}

pub unsafe fn cram_mFILE_c_502_mfread(
    ptr: *mut c_void,
    size: usize,
    nmemb: usize,
    mf: *mut mFILE,
) -> usize {
    if (*mf).size <= (*mf).offset {
        return 0;
    }

    let wanted = size.wrapping_mul(nmemb);
    let available = (*mf).size - (*mf).offset;
    let len = if wanted <= available {
        wanted
    } else {
        available
    };
    if size == 0 {
        return 0;
    }

    memcpy(
        ptr,
        (*mf).data.add((*mf).offset).cast::<c_void>(),
        len as u64,
    );
    (*mf).offset += len;

    if len != wanted {
        (*mf).eof = 1;
    }

    len / size
}

pub unsafe fn cram_mFILE_c_527_mfwrite(
    ptr: *mut c_void,
    size: usize,
    nmemb: usize,
    mf: *mut mFILE,
) -> usize {
    if ((*mf).mode & MF_WRITE) == 0 {
        return 0;
    }

    if ((*mf).mode & MF_APPEND) != 0 {
        (*mf).offset = (*mf).size;
    }

    let wanted = size.wrapping_mul(nmemb);
    while wanted + (*mf).offset > (*mf).alloced {
        let new_alloced = if (*mf).alloced != 0 {
            (*mf).alloced * 2
        } else {
            1024
        };
        let new_data = realloc((*mf).data.cast(), new_alloced as u64).cast::<c_char>();
        if new_data.is_null() {
            return 0;
        }
        (*mf).alloced = new_alloced;
        (*mf).data = new_data;
    }

    if (*mf).offset < (*mf).flush_pos {
        (*mf).flush_pos = (*mf).offset;
    }

    memcpy(
        (*mf).data.add((*mf).offset).cast::<c_void>(),
        ptr,
        wanted as u64,
    );
    (*mf).offset += wanted;
    if (*mf).size < (*mf).offset {
        (*mf).size = (*mf).offset;
    }

    nmemb
}

pub unsafe fn cram_mFILE_c_557_mfgetc(mf: *mut mFILE) -> c_int {
    if (*mf).offset < (*mf).size {
        let c = *(*mf).data.add((*mf).offset) as u8;
        (*mf).offset += 1;
        return c as c_int;
    }

    (*mf).eof = 1;
    -1
}

pub unsafe fn cram_mFILE_c_567_mungetc(c: c_int, mf: *mut mFILE) -> c_int {
    if (*mf).offset > 0 {
        (*mf).offset -= 1;
        *(*mf).data.add((*mf).offset) = c as c_char;
        return c;
    }

    (*mf).eof = 1;
    -1
}

pub unsafe fn cram_mFILE_c_577_mfgets(s: *mut c_char, size: c_int, mf: *mut mFILE) -> *mut c_char {
    let mut i = 0;

    *s = 0;
    while i < size - 1 {
        if (*mf).offset < (*mf).size {
            *s.add(i as usize) = *(*mf).data.add((*mf).offset);
            (*mf).offset += 1;
            i += 1;
            if *s.add((i - 1) as usize) == b'\n' as c_char {
                break;
            }
        } else {
            (*mf).eof = 1;
            break;
        }
    }

    *s.add(i as usize) = 0;
    if i != 0 {
        s
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn cram_mFILE_c_607_mfflush(mf: *mut mFILE) -> c_int {
    if (*mf).fp.is_null() {
        return 0;
    }

    if mf == M_CHANNEL[1] || mf == M_CHANNEL[2] {
        if (*mf).flush_pos < (*mf).size {
            let bytes = (*mf).size - (*mf).flush_pos;
            if libc::fwrite((*mf).data.add((*mf).flush_pos).cast(), 1, bytes, (*mf).fp) < bytes {
                return -1;
            }
            if libc::fflush((*mf).fp) != 0 {
                return -1;
            }
        }
        (*mf).offset = 0;
        (*mf).size = 0;
        (*mf).flush_pos = 0;
    }

    if ((*mf).mode & MF_WRITE) != 0 {
        if (*mf).flush_pos < (*mf).size {
            let bytes = (*mf).size - (*mf).flush_pos;
            if ((*mf).mode & MF_MODEX) == 0 {
                libc::fseek((*mf).fp, (*mf).flush_pos as libc::c_long, libc::SEEK_SET);
            }
            if libc::fwrite((*mf).data.add((*mf).flush_pos).cast(), 1, bytes, (*mf).fp) < bytes {
                return -1;
            }
            if libc::fflush((*mf).fp) != 0 {
                return -1;
            }
        }
        let pos = libc::ftell((*mf).fp);
        if pos != -1 && libc::ftruncate(libc::fileno((*mf).fp), pos) == -1 {
            return -1;
        }
        (*mf).flush_pos = (*mf).size;
    }

    0
}

pub unsafe fn cram_mFILE_c_656_mfascii(mf: *mut mFILE) {
    let mut p1 = 1usize;
    let mut p2 = 1usize;

    while p1 < (*mf).size {
        if *(*mf).data.add(p1) == b'\n' as c_char && *(*mf).data.add(p1 - 1) == b'\r' as c_char {
            p2 -= 1;
        }
        *(*mf).data.add(p2) = *(*mf).data.add(p1);
        p1 += 1;
        p2 += 1;
    }
    (*mf).size = p2;

    (*mf).offset = 0;
    (*mf).flush_pos = 0;
}
