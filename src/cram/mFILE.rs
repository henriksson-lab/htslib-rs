// Functions translated from htslib/cram/mFILE.c.
// Extracted from src/cram/mod.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int, c_void};
use std::ptr::NonNull;

use crate::htslib_rs::c_compat::{__errno_location, EINVAL};

use super::*;

pub(super) static mut M_CHANNEL: [Option<NonNull<mFILE>>; 3] = [None, None, None];
pub(super) static mut DONE_STDIN: bool = false;

pub struct OwnedFILE {
    fp: NonNull<libc::FILE>,
}

impl OwnedFILE {
    pub unsafe fn from_raw(fp: *mut libc::FILE) -> Option<Self> {
        NonNull::new(fp).map(|fp| Self { fp })
    }

    pub fn as_ptr(&self) -> *mut libc::FILE {
        self.fp.as_ptr()
    }

    pub fn into_raw(self) -> *mut libc::FILE {
        let fp = self.fp.as_ptr();
        std::mem::forget(self);
        fp
    }

    pub fn close(self) -> c_int {
        let fp = self.into_raw();
        unsafe { libc::fclose(fp) }
    }
}

impl Drop for OwnedFILE {
    fn drop(&mut self) {
        unsafe {
            libc::fclose(self.fp.as_ptr());
        }
    }
}

pub struct MmapRegion {
    ptr: NonNull<c_void>,
    len: usize,
}

impl MmapRegion {
    pub unsafe fn from_raw(ptr: *mut c_void, len: usize) -> Option<Self> {
        if len == 0 || ptr.is_null() || {
            #[cfg(not(windows))]
            {
                ptr == libc::MAP_FAILED
            }
            #[cfg(windows)]
            {
                false
            }
        } {
            return None;
        }
        NonNull::new(ptr).map(|ptr| Self { ptr, len })
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn into_raw(self) -> (*mut c_void, usize) {
        let parts = (self.ptr.as_ptr(), self.len);
        std::mem::forget(self);
        parts
    }
}

impl Drop for MmapRegion {
    fn drop(&mut self) {
        unsafe {
            #[cfg(not(windows))]
            {
                libc::munmap(self.ptr.as_ptr(), self.len);
            }
            #[cfg(windows)]
            {
                // Anonymous fallback path keeps an owned Vec; nothing to unmap.
            }
        }
    }
}

/// Read the entire contents of a stdio stream into an owned buffer. When `path`
/// is provided its `stat` size is used as a sizing hint. Returns `None` on
/// allocation failure.
unsafe fn mfload_buffer(fp: *mut libc::FILE, path: Option<&[u8]>) -> Option<Vec<u8>> {
    let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
    let mut data = Vec::<u8>::new();
    let bufsize = 8192usize;
    let mut target_size = None;

    if let Some(path) = path {
        // path is a NUL-terminated byte slice owned by the caller.
        if libc::stat(path.as_ptr().cast::<c_char>(), sb.as_mut_ptr()) != -1 {
            let sb = sb.assume_init();
            target_size = Some(sb.st_size as usize);
            if data.try_reserve_exact(sb.st_size as usize).is_err() {
                return None;
            }
        }
    }

    loop {
        let read_size = match target_size {
            Some(target_size) => {
                if data.len() >= target_size {
                    break;
                }
                (target_size - data.len()).min(bufsize)
            }
            None => bufsize,
        };
        let offset = data.len();
        if data.try_reserve(read_size).is_err() {
            return None;
        }
        let spare = data.spare_capacity_mut();
        let len = libc::fread(spare.as_mut_ptr().cast(), 1, read_size, fp);
        if len > 0 {
            data.set_len(offset + len);
        }
        if libc::feof(fp) != 0 {
            break;
        }
    }

    Some(data)
}

pub unsafe fn cram_mFILE_c_75_mfload(
    fp: *mut libc::FILE,
    fn_: *const c_char,
    size: *mut usize,
    _binary: c_int,
) -> *mut c_char {
    let path = if fn_.is_null() {
        None
    } else {
        let mut len = 0usize;
        while *fn_.add(len) != 0 {
            len += 1;
        }
        Some(std::slice::from_raw_parts(fn_.cast::<u8>(), len + 1))
    };
    let Some(buffer) = mfload_buffer(fp, path) else {
        return std::ptr::null_mut();
    };
    let content_len = buffer.len();
    // Hand ownership of the buffer to the (C-style) caller. Allocate at least
    // one byte so the returned pointer is never null for an empty file.
    let mut out = buffer;
    if out.capacity() == 0 {
        out.reserve_exact(1);
    }
    let ptr = out.as_mut_ptr().cast::<c_char>();
    std::mem::forget(out);
    if !size.is_null() {
        *size = content_len;
    }
    ptr
}

/// Replace `mf`'s owned buffer with `buffer`, setting the logical content size.
fn install_mfile_buffer(mf: &mut mFILE, buffer: Vec<u8>, size: usize) {
    mf.data = buffer;
    mf.size = size;
}

pub unsafe fn cram_mFILE_c_127_mfmmap(
    mf: *mut mFILE,
    fp: *mut libc::FILE,
    fn_: *const c_char,
) -> c_int {
    let Some(mf) = mf.as_mut() else {
        return -1;
    };
    mfmmap_borrowed(mf, fp, fn_)
}

unsafe fn mfmmap_borrowed(mf: &mut mFILE, fp: *mut libc::FILE, fn_: *const c_char) -> c_int {
    #[cfg(windows)]
    {
        let path = if fn_.is_null() {
            None
        } else {
            let mut len = 0usize;
            while *fn_.add(len) != 0 {
                len += 1;
            }
            Some(std::slice::from_raw_parts(fn_.cast::<u8>(), len + 1))
        };
        let Some(buffer) = mfload_buffer(fp, path) else {
            return -1;
        };
        let size = buffer.len();
        install_mfile_buffer(mf, buffer, size);
        mf.mode &= !MF_MMAP;
        return 0;
    }

    #[cfg(not(windows))]
    {
        let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
        if libc::stat(fn_, sb.as_mut_ptr()) != 0 {
            return -1;
        }
        let sb = sb.assume_init();
        mf.size = sb.st_size as usize;
        let data = libc::mmap(
            std::ptr::null_mut(),
            mf.size,
            libc::PROT_READ,
            libc::MAP_SHARED,
            libc::fileno(fp),
            0,
        );
        let Some(mapping) = MmapRegion::from_raw(data, mf.size) else {
            return -1;
        };
        let (data, len) = mapping.into_raw();

        // Adopt the mapped region as the owned buffer for the lifetime of mf.
        // It is reclaimed via munmap in cram_mFILE_c_361_mfclose.
        mf.data = Vec::from_raw_parts(data.cast::<u8>(), len, len);
        0
    }
}

pub unsafe fn cram_mFILE_c_151_mstdin() -> *mut mFILE {
    if let Some(mf) = M_CHANNEL[0] {
        return mf.as_ptr();
    }

    let mut mf = Box::new(new_empty_mfile());
    mf.fp = Some(HTSLIB_STDIN);
    let mf = NonNull::from(Box::leak(mf));
    M_CHANNEL[0] = Some(mf);
    mf.as_ptr()
}

pub unsafe fn cram_mFILE_c_161_init_mstdin() {
    if DONE_STDIN {
        return;
    }

    let Some(mut mf) = M_CHANNEL[0] else {
        return;
    };
    let mf = mf.as_mut();
    if let Some(buffer) = mfload_buffer(HTSLIB_STDIN, None) {
        let size = buffer.len();
        install_mfile_buffer(mf, buffer, size);
    }
    mf.mode = MF_READ;
    DONE_STDIN = true;
}

pub unsafe fn cram_mFILE_c_176_mstdout() -> *mut mFILE {
    if let Some(mf) = M_CHANNEL[1] {
        return mf.as_ptr();
    }

    let mut mf = Box::new(new_empty_mfile());
    mf.fp = Some(HTSLIB_STDOUT);
    mf.mode = MF_WRITE;
    let mf = NonNull::from(Box::leak(mf));
    M_CHANNEL[1] = Some(mf);
    mf.as_ptr()
}

pub unsafe fn cram_mFILE_c_192_mstderr() -> *mut mFILE {
    if let Some(mf) = M_CHANNEL[2] {
        return mf.as_ptr();
    }

    let mut mf = Box::new(new_empty_mfile());
    mf.fp = Some(HTSLIB_STDERR);
    mf.mode = MF_WRITE;
    let mf = NonNull::from(Box::leak(mf));
    M_CHANNEL[2] = Some(mf);
    mf.as_ptr()
}

pub unsafe fn cram_mFILE_c_207_mfcreate(data: *mut c_char, size: c_int) -> *mut mFILE {
    let mut mf = new_empty_mfile();
    if !data.is_null() && size > 0 {
        let bytes = std::slice::from_raw_parts(data.cast::<u8>(), size as usize);
        adopt_buffer_bytes(&mut mf, Some(bytes));
    } else {
        adopt_buffer_bytes(&mut mf, None);
    }
    if !data.is_null() {
        // The caller transferred ownership of `data` (originally malloc'd by C).
        // Reclaim it as a Vec so it is dropped here.
        drop(Vec::from_raw_parts(
            data.cast::<u8>(),
            size.max(0) as usize,
            size.max(0) as usize,
        ));
    }
    Box::into_raw(Box::new(mf))
}

fn new_empty_mfile() -> mFILE {
    mFILE {
        fp: None,
        data: Vec::new(),
        eof: false,
        mode: MF_READ | MF_WRITE,
        size: 0,
        offset: 0,
        flush_pos: 0,
    }
}

pub unsafe fn cram_mFILE_c_225_mfrecreate(mf: *mut mFILE, data: *mut c_char, size: c_int) {
    let bytes = if !data.is_null() && size > 0 {
        Some(std::slice::from_raw_parts(data.cast::<u8>(), size as usize))
    } else {
        None
    };
    if let Some(mf) = mf.as_mut() {
        adopt_buffer_bytes(mf, bytes);
        mf.eof = false;
        mf.offset = 0;
        mf.flush_pos = 0;
    }
    if !data.is_null() {
        drop(Vec::from_raw_parts(
            data.cast::<u8>(),
            size.max(0) as usize,
            size.max(0) as usize,
        ));
    }
}

/// Replace `mf`'s buffer with a copy of `data` (or empty it when `None`).
fn adopt_buffer_bytes(mf: &mut mFILE, data: Option<&[u8]>) {
    match data {
        Some(data) => {
            mf.data = data.to_vec();
            mf.size = data.len();
        }
        None => {
            mf.data = Vec::new();
            mf.size = 0;
        }
    }
}

pub unsafe fn cram_mFILE_c_246_mfcreate_from(
    path: *const c_char,
    mode_str: *const c_char,
    fp: *mut libc::FILE,
) -> *mut mFILE {
    let mf = cram_mFILE_c_264_mfreopen(path, mode_str, fp);
    if let Some(mf) = mf.as_mut() {
        mf.fp = None;
    }
    mf
}

pub unsafe fn cram_mFILE_c_264_mfreopen(
    path: *const c_char,
    mode_str: *const c_char,
    fp: *mut libc::FILE,
) -> *mut mFILE {
    let mut r = false;
    let mut w = false;
    let mut a = false;
    let mut x = false;
    let mut mode = 0;

    if mode_str.is_null() {
        return std::ptr::null_mut();
    }

    let mut len = 0usize;
    while *mode_str.add(len) != 0 {
        len += 1;
    }
    let mode_bytes = std::slice::from_raw_parts(mode_str.cast::<u8>(), len);
    if mode_bytes.contains(&b'r') {
        r = true;
        mode |= MF_READ;
    }
    if mode_bytes.contains(&b'w') {
        w = true;
        mode |= MF_WRITE | MF_TRUNC;
    }
    if mode_bytes.contains(&b'a') {
        w = true;
        a = true;
        mode |= MF_WRITE | MF_APPEND;
    }
    if mode_bytes.contains(&b'b') {
        mode |= MF_BINARY;
    }
    if mode_bytes.contains(&b'x') {
        x = true;
    }
    if mode_bytes.contains(&b'+') {
        w = true;
        mode |= MF_READ | MF_WRITE;
        if a {
            r = true;
        }
    }
    if mode_bytes.contains(&b'm') && !w {
        mode |= MF_MMAP;
    }

    let mf;
    if r {
        mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
        if mf.is_null() {
            return std::ptr::null_mut();
        }
        let mf_ref = &mut *mf;
        if (mode & MF_TRUNC) == 0 {
            if (mode & MF_MMAP) != 0 && mfmmap_borrowed(mf_ref, fp, path) == -1 {
                mf_ref.data = Vec::new();
                mode &= !MF_MMAP;
            }
            if mf_ref.data.is_empty() {
                let path = if path.is_null() {
                    None
                } else {
                    let mut plen = 0usize;
                    while *path.add(plen) != 0 {
                        plen += 1;
                    }
                    Some(std::slice::from_raw_parts(path.cast::<u8>(), plen + 1))
                };
                let Some(buffer) = mfload_buffer(fp, path) else {
                    drop(Box::from_raw(mf));
                    return std::ptr::null_mut();
                };
                let size = buffer.len();
                install_mfile_buffer(mf_ref, buffer, size);
                if !a {
                    libc::fseek(fp, 0, libc::SEEK_SET);
                }
            }
        }
    } else if w {
        mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
        if mf.is_null() {
            return std::ptr::null_mut();
        }
    } else {
        return std::ptr::null_mut();
    }

    let mf_ref = &mut *mf;
    mf_ref.fp = Some(fp);
    mf_ref.mode = mode;
    if x {
        mf_ref.mode |= MF_MODEX;
    }
    if a {
        mf_ref.flush_pos = mf_ref.size;
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
    let Some(mut mf) = NonNull::new(mf) else {
        return -1;
    };
    let mf_ref = mf.as_mut();
    mfflush_borrowed(mf_ref);
    if (mf_ref.mode & MF_MMAP) != 0 && !mf_ref.data.is_empty() {
        // The buffer is a mapped region adopted in mfmmap; release it via munmap
        // and avoid dropping it as an ordinary Vec.
        let buf = std::mem::take(&mut mf_ref.data);
        let len = buf.len();
        let ptr = buf.as_ptr() as *mut c_void;
        std::mem::forget(buf);
        drop(MmapRegion::from_raw(ptr, len));
    }
    if let Some(fp) = mf_ref.fp.take() {
        libc::fclose(fp);
    }
    mfdestroy_owned(mf);
    0
}

pub unsafe fn cram_mFILE_c_389_mfdetach(mf: *mut mFILE) -> c_int {
    let Some(mf) = mf.as_mut() else {
        return -1;
    };
    mfdetach_borrowed(mf)
}

unsafe fn mfdetach_borrowed(mf: &mut mFILE) -> c_int {
    if mfflush_borrowed(mf) != 0 {
        return -1;
    }
    if (mf.mode & MF_MMAP) != 0 {
        return -1;
    }
    if let Some(fp) = mf.fp.take() {
        libc::fclose(fp);
    }
    0
}

pub unsafe fn cram_mFILE_c_408_mfdestroy(mf: *mut mFILE) -> c_int {
    let Some(mf) = NonNull::new(mf) else {
        return -1;
    };
    mfdestroy_owned(mf);
    0
}

unsafe fn mfdestroy_owned(mf: NonNull<mFILE>) {
    for channel in 0..3 {
        if M_CHANNEL[channel]
            .map(|channel| channel == mf)
            .unwrap_or(false)
        {
            M_CHANNEL[channel] = None;
        }
    }
    // Dropping the Box frees the owned mFILE and its Vec buffer.
    drop(Box::from_raw(mf.as_ptr()));
}

pub unsafe fn cram_mFILE_c_428_mfsteal(mf: *mut mFILE, size_out: *mut usize) -> *mut c_void {
    let Some(mut mf) = NonNull::new(mf) else {
        return std::ptr::null_mut();
    };
    let mf_ref = mf.as_mut();
    if !size_out.is_null() {
        *size_out = mf_ref.size;
    }
    if mfdetach_borrowed(mf_ref) != 0 {
        return std::ptr::null_mut();
    }
    let data = if mf_ref.data.is_empty() {
        std::ptr::null_mut()
    } else {
        // Hand the first `size` bytes to the C-style caller as an owned buffer.
        let mut out: Vec<u8> = mf_ref.data[..mf_ref.size].to_vec();
        if out.capacity() == 0 {
            out.reserve_exact(1);
        }
        let ptr = out.as_mut_ptr().cast::<c_void>();
        std::mem::forget(out);
        ptr
    };
    if data.is_null() && mf_ref.size != 0 {
        return std::ptr::null_mut();
    }
    mfdestroy_owned(mf);
    data
}

pub unsafe fn cram_mFILE_c_451_mfseek(
    mf: *mut mFILE,
    offset: libc::c_long,
    whence: c_int,
) -> c_int {
    let Some(mf) = mf.as_mut() else {
        return -1;
    };
    mfseek_borrowed(mf, offset, whence)
}

unsafe fn mfseek_borrowed(mf: &mut mFILE, offset: libc::c_long, whence: c_int) -> c_int {
    match whence {
        libc::SEEK_SET => {
            mf.offset = offset as usize;
        }
        libc::SEEK_CUR => {
            mf.offset = mf.offset.wrapping_add(offset as usize);
        }
        libc::SEEK_END => {
            mf.offset = mf.size.wrapping_add(offset as usize);
        }
        _ => {
            *__errno_location() = EINVAL;
            return -1;
        }
    }

    mf.eof = false;
    0
}

pub unsafe fn cram_mFILE_c_471_mftell(mf: *mut mFILE) -> libc::c_long {
    mf.as_ref()
        .map(|mf| mf.offset as libc::c_long)
        .unwrap_or(-1)
}

pub unsafe fn cram_mFILE_c_475_mrewind(mf: *mut mFILE) {
    if let Some(mf) = mf.as_mut() {
        mrewind_borrowed(mf);
    }
}

fn mrewind_borrowed(mf: &mut mFILE) {
    mf.offset = 0;
    mf.eof = false;
}

pub unsafe fn cram_mFILE_c_488_mftruncate(mf: *mut mFILE, offset: libc::c_long) {
    if let Some(mf) = mf.as_mut() {
        mftruncate_borrowed(mf, offset);
    }
}

fn mftruncate_borrowed(mf: &mut mFILE, offset: libc::c_long) {
    mf.size = if offset != -1 {
        offset as usize
    } else {
        mf.offset
    };
    if mf.offset > mf.size {
        mf.offset = mf.size;
    }
}

pub unsafe fn cram_mFILE_c_494_mfeof(mf: *mut mFILE) -> c_int {
    mf.as_ref().map(|mf| mf.eof as c_int).unwrap_or(1)
}

pub unsafe fn cram_mFILE_c_502_mfread(
    ptr: *mut c_void,
    size: usize,
    nmemb: usize,
    mf: *mut mFILE,
) -> usize {
    let Some(mf) = mf.as_mut() else {
        return 0;
    };
    if ptr.is_null() || size == 0 {
        return 0;
    }
    let Some(wanted) = size.checked_mul(nmemb) else {
        return 0;
    };
    let out = std::slice::from_raw_parts_mut(ptr.cast::<u8>(), wanted);
    mfread_borrowed(out, size, nmemb, mf)
}

fn mfread_borrowed(out: &mut [u8], size: usize, nmemb: usize, mf: &mut mFILE) -> usize {
    let data = &mf.data[..mf.size];
    if data.len() <= mf.offset {
        return 0;
    }

    let wanted = size.wrapping_mul(nmemb);
    let available = data.len() - mf.offset;
    let len = if wanted <= available {
        wanted
    } else {
        available
    };
    if size == 0 || out.len() < len {
        return 0;
    }

    out[..len].copy_from_slice(&data[mf.offset..mf.offset + len]);
    mf.offset += len;

    if len != wanted {
        mf.eof = true;
    }

    len / size
}

pub unsafe fn cram_mFILE_c_527_mfwrite(
    ptr: *mut c_void,
    size: usize,
    nmemb: usize,
    mf: *mut mFILE,
) -> usize {
    let Some(mf) = mf.as_mut() else {
        return 0;
    };
    let Some(wanted) = size.checked_mul(nmemb) else {
        return 0;
    };
    if wanted == 0 {
        return nmemb;
    }
    if ptr.is_null() {
        return 0;
    }
    let input = std::slice::from_raw_parts(ptr.cast::<u8>(), wanted);
    mfwrite_borrowed(input, size, nmemb, mf)
}

fn mfwrite_borrowed(input: &[u8], size: usize, nmemb: usize, mf: &mut mFILE) -> usize {
    let _ = size;
    if (mf.mode & MF_WRITE) == 0 {
        return 0;
    }

    if (mf.mode & MF_APPEND) != 0 {
        mf.offset = mf.size;
    }

    let wanted = input.len();
    if wanted == 0 {
        return nmemb;
    }
    let Some(required) = mf.offset.checked_add(wanted) else {
        return 0;
    };
    // Grow the owned buffer so it can hold `required` bytes. The old C realloc
    // doubling is replaced by Vec growth that preserves existing content.
    if required > mf.data.len() {
        let mut new_alloced = mf.data.len().max(1024);
        while required > new_alloced {
            let Some(doubled) = new_alloced.checked_mul(2) else {
                new_alloced = required;
                break;
            };
            new_alloced = doubled;
        }
        let copy_len = mf.size.min(mf.data.len());
        let mut grown = Vec::new();
        if grown.try_reserve_exact(new_alloced).is_err() {
            return 0;
        }
        grown.resize(new_alloced, 0);
        grown[..copy_len].copy_from_slice(&mf.data[..copy_len]);
        mf.data = grown;
    }

    if mf.offset < mf.flush_pos {
        mf.flush_pos = mf.offset;
    }

    let offset = mf.offset;
    mf.data[offset..required].copy_from_slice(input);
    mf.offset += wanted;
    if mf.size < mf.offset {
        mf.size = mf.offset;
    }

    nmemb
}

pub unsafe fn cram_mFILE_c_557_mfgetc(mf: *mut mFILE) -> c_int {
    let Some(mf) = mf.as_mut() else {
        return -1;
    };
    mfgetc_borrowed(mf)
}

fn mfgetc_borrowed(mf: &mut mFILE) -> c_int {
    let data = &mf.data[..mf.size];
    if mf.offset < data.len() {
        let c = data[mf.offset];
        mf.offset += 1;
        return c as c_int;
    }

    mf.eof = true;
    -1
}

pub unsafe fn cram_mFILE_c_567_mungetc(c: c_int, mf: *mut mFILE) -> c_int {
    let Some(mf) = mf.as_mut() else {
        return -1;
    };
    mungetc_borrowed(c, mf)
}

fn mungetc_borrowed(c: c_int, mf: &mut mFILE) -> c_int {
    if mf.offset > 0 {
        let new_offset = mf.offset - 1;
        if new_offset >= mf.data.len() {
            return -1;
        }
        mf.data[new_offset] = c as u8;
        mf.offset = new_offset;
        return c;
    }

    mf.eof = true;
    -1
}

pub unsafe fn cram_mFILE_c_577_mfgets(s: *mut c_char, size: c_int, mf: *mut mFILE) -> *mut c_char {
    let Some(mf) = mf.as_mut() else {
        return std::ptr::null_mut();
    };
    if s.is_null() || size <= 0 {
        return std::ptr::null_mut();
    }
    let out = std::slice::from_raw_parts_mut(s.cast::<u8>(), size as usize);
    if mfgets_borrowed(out, mf) {
        s
    } else {
        std::ptr::null_mut()
    }
}

/// Read a line (up to and including a trailing `\n`) into `s`, NUL-terminating
/// it. Returns `true` if any bytes were read.
fn mfgets_borrowed(s: &mut [u8], mf: &mut mFILE) -> bool {
    let mut i = 0usize;
    let mut offset = mf.offset;
    let data = &mf.data[..mf.size];

    s[0] = 0;
    while i + 1 < s.len() {
        if offset < data.len() {
            s[i] = data[offset];
            offset += 1;
            i += 1;
            if s[i - 1] == b'\n' {
                break;
            }
        } else {
            mf.eof = true;
            break;
        }
    }

    mf.offset = offset;
    s[i] = 0;
    i != 0
}

pub unsafe fn cram_mFILE_c_607_mfflush(mf: *mut mFILE) -> c_int {
    let Some(mf) = mf.as_mut() else {
        return -1;
    };
    mfflush_borrowed(mf)
}

unsafe fn mfflush_borrowed(mf: &mut mFILE) -> c_int {
    let Some(fp) = mf.fp else {
        return 0;
    };

    let mf_ptr = NonNull::from(&mut *mf);
    if M_CHANNEL[1]
        .map(|channel| channel == mf_ptr)
        .unwrap_or(false)
        || M_CHANNEL[2]
            .map(|channel| channel == mf_ptr)
            .unwrap_or(false)
    {
        if mf.flush_pos < mf.size {
            let bytes = &mf.data[mf.flush_pos..mf.size];
            if libc::fwrite(bytes.as_ptr().cast(), 1, bytes.len(), fp) < bytes.len() {
                return -1;
            }
            if libc::fflush(fp) != 0 {
                return -1;
            }
        }
        mf.offset = 0;
        mf.size = 0;
        mf.flush_pos = 0;
    }

    if (mf.mode & MF_WRITE) != 0 {
        if mf.flush_pos < mf.size {
            let bytes = &mf.data[mf.flush_pos..mf.size];
            if (mf.mode & MF_MODEX) == 0 {
                libc::fseek(fp, mf.flush_pos as libc::c_long, libc::SEEK_SET);
            }
            if libc::fwrite(bytes.as_ptr().cast(), 1, bytes.len(), fp) < bytes.len() {
                return -1;
            }
            if libc::fflush(fp) != 0 {
                return -1;
            }
        }
        let pos = libc::ftell(fp);
        if pos != -1 && crate::htslib_rs::c_compat::ftruncate(libc::fileno(fp), pos) == -1 {
            return -1;
        }
        mf.flush_pos = mf.size;
    }

    0
}

pub unsafe fn cram_mFILE_c_656_mfascii(mf: *mut mFILE) {
    if let Some(mf) = mf.as_mut() {
        mfascii_borrowed(mf);
    }
}

fn mfascii_borrowed(mf: &mut mFILE) {
    let mut p1 = 1usize;
    let mut p2 = 1usize;
    let size = mf.size;
    let data = &mut mf.data;
    if data.is_empty() {
        return;
    }

    while p1 < size {
        if data[p1] == b'\n' && data[p1 - 1] == b'\r' {
            p2 -= 1;
        }
        data[p2] = data[p1];
        p1 += 1;
        p2 += 1;
    }
    mf.size = p2;

    mf.offset = 0;
    mf.flush_pos = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn owned_file_closes_or_releases_raw_file_pointer() {
        unsafe {
            assert!(OwnedFILE::from_raw(std::ptr::null_mut()).is_none());

            let path = std::env::temp_dir()
                .join(format!("htslib-rs-owned-file-{}-a.tmp", std::process::id()));
            let c_path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let fp = libc::fopen(c_path.as_ptr(), c"w+b".as_ptr());
            assert!(!fp.is_null());
            let owned = OwnedFILE::from_raw(fp).expect("non-null FILE");
            assert_eq!(owned.as_ptr(), fp);
            assert_eq!(
                libc::fwrite(c"abc".as_ptr().cast(), 1, 3, owned.as_ptr()),
                3
            );
            assert_eq!(owned.close(), 0);
            assert_eq!(std::fs::read(&path).unwrap(), b"abc");
            std::fs::remove_file(&path).unwrap();

            let path = std::env::temp_dir()
                .join(format!("htslib-rs-owned-file-{}-b.tmp", std::process::id()));
            let c_path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let fp = libc::fopen(c_path.as_ptr(), c"w+b".as_ptr());
            assert!(!fp.is_null());
            let owned = OwnedFILE::from_raw(fp).expect("non-null FILE");
            let raw = owned.into_raw();
            assert_eq!(raw, fp);
            assert_eq!(libc::fclose(raw), 0);
            std::fs::remove_file(&path).unwrap();
        }
    }

    #[test]
    #[cfg(not(windows))]
    fn mmap_region_releases_or_returns_raw_mapping() {
        unsafe {
            assert!(MmapRegion::from_raw(std::ptr::null_mut(), 4).is_none());
            assert!(MmapRegion::from_raw(libc::MAP_FAILED, 4).is_none());
            let mut zero_len_marker = 0u8;
            assert!(MmapRegion::from_raw((&mut zero_len_marker as *mut u8).cast(), 0).is_none());

            let ptr = libc::mmap(
                std::ptr::null_mut(),
                4096,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            );
            assert_ne!(ptr, libc::MAP_FAILED);
            let region = MmapRegion::from_raw(ptr, 4096).expect("anonymous mmap");
            assert_eq!(region.as_ptr(), ptr);
            assert_eq!(region.len(), 4096);
            assert!(!region.is_empty());
            let (raw, len) = region.into_raw();
            assert_eq!((raw, len), (ptr, 4096));
            assert_eq!(libc::munmap(raw, len), 0);

            let ptr = libc::mmap(
                std::ptr::null_mut(),
                4096,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            );
            assert_ne!(ptr, libc::MAP_FAILED);
            let _region = MmapRegion::from_raw(ptr, 4096).expect("anonymous mmap");
        }
    }
}
