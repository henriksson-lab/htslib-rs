// Functions translated from htslib/cram/mFILE.c.
// Extracted from src/cram/mod.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int, c_void};
use std::ptr::NonNull;

use crate::htslib_rs::c_compat::{__errno_location, free, malloc, EINVAL};

use super::*;

pub(super) static mut M_CHANNEL: [Option<NonNull<mFILE>>; 3] = [None, None, None];
pub(super) static mut DONE_STDIN: c_int = 0;

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
                free(self.ptr.as_ptr());
            }
        }
    }
}

struct OwnedMfileBuffer {
    data: Vec<u8>,
    used: usize,
}

impl OwnedMfileBuffer {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let capacity = bytes.len().max(1);
        let mut data = Self::with_capacity(capacity)?.data;
        data[..bytes.len()].copy_from_slice(bytes);
        Some(Self {
            data,
            used: bytes.len(),
        })
    }

    fn with_capacity(capacity: usize) -> Option<Self> {
        let capacity = capacity.max(1);
        let mut data = Vec::new();
        if data.try_reserve_exact(capacity).is_err() {
            return None;
        }
        data.resize(capacity, 0);
        Some(Self {
            data,
            used: capacity,
        })
    }

    fn as_ptr(&self) -> *mut c_char {
        self.data.as_ptr().cast::<c_char>().cast_mut()
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data[..self.used]
    }

    fn data_len(&self) -> usize {
        self.used
    }

    fn as_slice(&self) -> &[u8] {
        &self.data[..self.used]
    }

    fn into_raw_parts(mut self) -> (*mut c_char, usize) {
        let ptr = self.data.as_mut_ptr().cast::<c_char>();
        let capacity = self.data.capacity();
        std::mem::forget(self);
        (ptr, capacity)
    }
}

unsafe fn reclaim_mfile_buffer(data: *mut c_char, alloced: usize) {
    if !data.is_null() && alloced != 0 {
        drop(Vec::from_raw_parts(data.cast::<u8>(), alloced, alloced));
    }
}

unsafe fn c_buffer_bytes<'a>(data: *mut c_char, size: c_int) -> Option<&'a [u8]> {
    if data.is_null() || size <= 0 {
        return None;
    }
    Some(std::slice::from_raw_parts(data.cast::<u8>(), size as usize))
}

unsafe fn mfile_data(mf: &mFILE) -> &[u8] {
    if mf.data.is_null() || mf.size == 0 {
        return &[];
    }
    std::slice::from_raw_parts(mf.data.cast::<u8>(), mf.size)
}

unsafe fn mfile_allocated_mut(mf: &mut mFILE) -> Option<&mut [u8]> {
    if mf.alloced == 0 {
        return Some(&mut []);
    }
    NonNull::new(mf.data.cast::<u8>())
        .map(|data| std::slice::from_raw_parts_mut(data.as_ptr(), mf.alloced))
}

unsafe fn copy_to_libc_buffer(bytes: &[u8]) -> *mut c_char {
    let len = bytes.len();
    let ptr = malloc(len.max(1) as u64).cast::<c_char>();
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    if len != 0 {
        std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), ptr, len);
    }
    ptr
}

unsafe fn null_terminated_bytes<'a>(ptr: *const c_char) -> Option<&'a [u8]> {
    if ptr.is_null() {
        return None;
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    Some(std::slice::from_raw_parts(ptr.cast::<u8>(), len))
}

pub unsafe fn cram_mFILE_c_75_mfload(
    fp: *mut libc::FILE,
    fn_: *const c_char,
    size: *mut usize,
    _binary: c_int,
) -> *mut c_char {
    let Some(buffer) = mfload_buffer(fp, (!fn_.is_null()).then_some(fn_)) else {
        return std::ptr::null_mut();
    };
    let data = copy_to_libc_buffer(buffer.as_slice());
    if data.is_null() {
        return std::ptr::null_mut();
    }
    if !size.is_null() {
        *size = buffer.data_len();
    }
    data
}

unsafe fn mfload_buffer(
    fp: *mut libc::FILE,
    path: Option<*const c_char>,
) -> Option<OwnedMfileBuffer> {
    let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
    let mut data = Vec::<u8>::new();
    let bufsize = 8192usize;
    let mut target_size = None;

    if let Some(path) = path {
        if libc::stat(path, sb.as_mut_ptr()) != -1 {
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

    OwnedMfileBuffer::from_bytes(&data)
}

fn install_mfile_buffer(mf: &mut mFILE, buffer: OwnedMfileBuffer, size: usize) {
    let (data, alloced) = buffer.into_raw_parts();
    mf.data = data;
    mf.size = size;
    mf.alloced = alloced;
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
        let Some(buffer) = mfload_buffer(fp, (!fn_.is_null()).then_some(fn_)) else {
            return -1;
        };
        let size = buffer.data_len();
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
        let (data, _) = mapping.into_raw();

        mf.data = data.cast::<c_char>();
        mf.alloced = 0;
        0
    }
}

pub unsafe fn cram_mFILE_c_151_mstdin() -> *mut mFILE {
    if let Some(mf) = M_CHANNEL[0] {
        return mf.as_ptr();
    }

    let mf = Box::new(new_empty_mfile());
    let Some(mut mf) = NonNull::new(Box::into_raw(mf)) else {
        return std::ptr::null_mut();
    };
    mf.as_mut().fp = HTSLIB_STDIN;
    M_CHANNEL[0] = Some(mf);
    mf.as_ptr()
}

pub unsafe fn cram_mFILE_c_161_init_mstdin() {
    if DONE_STDIN != 0 {
        return;
    }

    let Some(mut mf) = M_CHANNEL[0] else {
        return;
    };
    let mf = mf.as_mut();
    if let Some(buffer) = mfload_buffer(HTSLIB_STDIN, None) {
        let size = buffer.data_len();
        install_mfile_buffer(mf, buffer, size);
    }
    mf.mode = MF_READ;
    DONE_STDIN = 1;
}

pub unsafe fn cram_mFILE_c_176_mstdout() -> *mut mFILE {
    if let Some(mf) = M_CHANNEL[1] {
        return mf.as_ptr();
    }

    let mf = Box::new(new_empty_mfile());
    let Some(mut mf) = NonNull::new(Box::into_raw(mf)) else {
        return std::ptr::null_mut();
    };
    let mf_ref = mf.as_mut();
    mf_ref.fp = HTSLIB_STDOUT;
    mf_ref.mode = MF_WRITE;
    M_CHANNEL[1] = Some(mf);
    mf.as_ptr()
}

pub unsafe fn cram_mFILE_c_192_mstderr() -> *mut mFILE {
    if let Some(mf) = M_CHANNEL[2] {
        return mf.as_ptr();
    }

    let mf = Box::new(new_empty_mfile());
    let Some(mut mf) = NonNull::new(Box::into_raw(mf)) else {
        return std::ptr::null_mut();
    };
    let mf_ref = mf.as_mut();
    mf_ref.fp = HTSLIB_STDERR;
    mf_ref.mode = MF_WRITE;
    M_CHANNEL[2] = Some(mf);
    mf.as_ptr()
}

pub unsafe fn cram_mFILE_c_207_mfcreate(data: *mut c_char, size: c_int) -> *mut mFILE {
    let mut mf = new_empty_mfile();
    adopt_buffer_bytes(&mut mf, c_buffer_bytes(data, size));
    if !data.is_null() {
        free(data.cast());
    }
    Box::into_raw(Box::new(mf))
}

fn new_empty_mfile() -> mFILE {
    mFILE {
        fp: std::ptr::null_mut(),
        data: std::ptr::null_mut(),
        alloced: 0,
        eof: 0,
        mode: MF_READ | MF_WRITE,
        size: 0,
        offset: 0,
        flush_pos: 0,
    }
}

pub unsafe fn cram_mFILE_c_225_mfrecreate(mf: *mut mFILE, data: *mut c_char, size: c_int) {
    if let Some(mf) = mf.as_mut() {
        mfrecreate_borrowed(mf, c_buffer_bytes(data, size));
    }
    if !data.is_null() {
        free(data.cast());
    }
}

unsafe fn mfrecreate_borrowed(mf: &mut mFILE, data: Option<&[u8]>) {
    reclaim_mfile_buffer(mf.data, mf.alloced);
    adopt_buffer_bytes(mf, data);
    mf.eof = 0;
    mf.offset = 0;
    mf.flush_pos = 0;
}

fn adopt_buffer_bytes(mf: &mut mFILE, data: Option<&[u8]>) {
    let Some(data) = data else {
        mf.data = std::ptr::null_mut();
        mf.size = 0;
        mf.alloced = 0;
        return;
    };

    let size = data.len();
    if let Some(buffer) = OwnedMfileBuffer::from_bytes(data) {
        install_mfile_buffer(mf, buffer, size);
    } else {
        mf.data = std::ptr::null_mut();
        mf.size = 0;
        mf.alloced = 0;
    }
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
    if let Some(mf) = mf.as_mut() {
        mf.fp = std::ptr::null_mut();
    }
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
    let mut x = 0;
    let mut mode = 0;

    if mode_str.is_null() {
        return std::ptr::null_mut();
    }

    let Some(mode_bytes) = null_terminated_bytes(mode_str) else {
        return std::ptr::null_mut();
    };
    if mode_bytes.contains(&b'r') {
        r = 1;
        mode |= MF_READ;
    }
    if mode_bytes.contains(&b'w') {
        w = 1;
        mode |= MF_WRITE | MF_TRUNC;
    }
    if mode_bytes.contains(&b'a') {
        w = 1;
        a = 1;
        mode |= MF_WRITE | MF_APPEND;
    }
    if mode_bytes.contains(&b'b') {
        mode |= MF_BINARY;
    }
    if mode_bytes.contains(&b'x') {
        x = 1;
    }
    if mode_bytes.contains(&b'+') {
        w = 1;
        mode |= MF_READ | MF_WRITE;
        if a != 0 {
            r = 1;
        }
    }
    if mode_bytes.contains(&b'm') && w == 0 {
        mode |= MF_MMAP;
    }

    let mf;
    if r != 0 {
        mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
        if mf.is_null() {
            return std::ptr::null_mut();
        }
        let mf_ref = &mut *mf;
        if (mode & MF_TRUNC) == 0 {
            if (mode & MF_MMAP) != 0 && mfmmap_borrowed(mf_ref, fp, path) == -1 {
                mf_ref.data = std::ptr::null_mut();
                mode &= !MF_MMAP;
            }
            if mf_ref.data.is_null() {
                let Some(buffer) = mfload_buffer(fp, (!path.is_null()).then_some(path)) else {
                    drop(Box::from_raw(mf));
                    return std::ptr::null_mut();
                };
                let size = buffer.data_len();
                install_mfile_buffer(mf_ref, buffer, size);
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

    let mf_ref = &mut *mf;
    mf_ref.fp = fp;
    mf_ref.mode = mode;
    if x != 0 {
        mf_ref.mode |= MF_MODEX;
    }
    if a != 0 {
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
    if (mf_ref.mode & MF_MMAP) != 0 && !mf_ref.data.is_null() {
        drop(MmapRegion::from_raw(mf_ref.data.cast(), mf_ref.size));
        mf_ref.data = std::ptr::null_mut();
    }
    if !mf_ref.fp.is_null() {
        libc::fclose(mf_ref.fp);
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
    if !mf.fp.is_null() {
        libc::fclose(mf.fp);
        mf.fp = std::ptr::null_mut();
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
    let mf = Box::from_raw(mf.as_ptr());
    reclaim_mfile_buffer(mf.data, mf.alloced);
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
    let data = if mf_ref.data.is_null() {
        std::ptr::null_mut()
    } else {
        let bytes = std::slice::from_raw_parts(mf_ref.data.cast::<u8>(), mf_ref.size);
        copy_to_libc_buffer(bytes).cast::<c_void>()
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

    mf.eof = 0;
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
    mf.eof = 0;
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
    mf.as_ref().map(|mf| mf.eof).unwrap_or(1)
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

unsafe fn mfread_borrowed(out: &mut [u8], size: usize, nmemb: usize, mf: &mut mFILE) -> usize {
    let data = mfile_data(mf);
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
        mf.eof = 1;
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

unsafe fn mfwrite_borrowed(input: &[u8], size: usize, nmemb: usize, mf: &mut mFILE) -> usize {
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
    if required > mf.alloced && !grow_mfwrite_buffer(mf, required) {
        return 0;
    }

    if mf.offset < mf.flush_pos {
        mf.flush_pos = mf.offset;
    }

    let offset = mf.offset;
    let Some(data) = mfile_allocated_mut(mf) else {
        return 0;
    };
    data[offset..required].copy_from_slice(input);
    mf.offset += wanted;
    if mf.size < mf.offset {
        mf.size = mf.offset;
    }

    nmemb
}

unsafe fn grow_mfwrite_buffer(mf: &mut mFILE, required: usize) -> bool {
    let mut new_alloced = mf.alloced.max(1024);
    while required > new_alloced {
        let Some(doubled) = new_alloced.checked_mul(2) else {
            new_alloced = required;
            break;
        };
        new_alloced = doubled;
    }

    let copy_len = mf.size.min(mf.alloced);
    let mut preserved = Vec::new();
    if copy_len != 0 {
        let data = mfile_data(mf);
        if data.len() < copy_len || preserved.try_reserve_exact(copy_len).is_err() {
            return false;
        }
        preserved.extend_from_slice(&data[..copy_len]);
    }

    let Some(mut new_data) = OwnedMfileBuffer::with_capacity(new_alloced) else {
        return false;
    };
    if !preserved.is_empty() {
        new_data.as_mut_slice()[..preserved.len()].copy_from_slice(&preserved);
    }
    reclaim_mfile_buffer(mf.data, mf.alloced);
    let (data, alloced) = new_data.into_raw_parts();
    mf.data = data;
    mf.alloced = alloced;
    true
}

pub unsafe fn cram_mFILE_c_557_mfgetc(mf: *mut mFILE) -> c_int {
    let Some(mf) = mf.as_mut() else {
        return -1;
    };
    mfgetc_borrowed(mf)
}

unsafe fn mfgetc_borrowed(mf: &mut mFILE) -> c_int {
    let data = mfile_data(mf);
    if mf.offset < data.len() {
        let c = data[mf.offset];
        mf.offset += 1;
        return c as c_int;
    }

    mf.eof = 1;
    -1
}

pub unsafe fn cram_mFILE_c_567_mungetc(c: c_int, mf: *mut mFILE) -> c_int {
    let Some(mf) = mf.as_mut() else {
        return -1;
    };
    mungetc_borrowed(c, mf)
}

unsafe fn mungetc_borrowed(c: c_int, mf: &mut mFILE) -> c_int {
    if mf.offset > 0 {
        let new_offset = mf.offset - 1;
        let Some(data) = mfile_allocated_mut(mf) else {
            return -1;
        };
        if new_offset >= data.len() {
            return -1;
        }
        data[new_offset] = c as u8;
        mf.offset = new_offset;
        return c;
    }

    mf.eof = 1;
    -1
}

pub unsafe fn cram_mFILE_c_577_mfgets(s: *mut c_char, size: c_int, mf: *mut mFILE) -> *mut c_char {
    let Some(mf) = mf.as_mut() else {
        return std::ptr::null_mut();
    };
    if s.is_null() || size <= 0 {
        return std::ptr::null_mut();
    }
    let out = std::slice::from_raw_parts_mut(s, size as usize);
    mfgets_borrowed(out, mf).map_or(std::ptr::null_mut(), |line| line.as_mut_ptr())
}

unsafe fn mfgets_borrowed<'a>(s: &'a mut [c_char], mf: &mut mFILE) -> Option<&'a mut [c_char]> {
    let mut i = 0usize;
    let mut offset = mf.offset;
    let data = mfile_data(mf);

    s[0] = 0;
    while i + 1 < s.len() {
        if offset < data.len() {
            s[i] = data[offset] as c_char;
            offset += 1;
            i += 1;
            if s[i - 1] == b'\n' as c_char {
                break;
            }
        } else {
            mf.eof = 1;
            break;
        }
    }

    mf.offset = offset;
    s[i] = 0;
    if i != 0 {
        Some(s)
    } else {
        None
    }
}

pub unsafe fn cram_mFILE_c_607_mfflush(mf: *mut mFILE) -> c_int {
    let Some(mf) = mf.as_mut() else {
        return -1;
    };
    mfflush_borrowed(mf)
}

unsafe fn mfflush_borrowed(mf: &mut mFILE) -> c_int {
    if mf.fp.is_null() {
        return 0;
    }

    let mf_ptr = NonNull::from(&mut *mf);
    if M_CHANNEL[1]
        .map(|channel| channel == mf_ptr)
        .unwrap_or(false)
        || M_CHANNEL[2]
            .map(|channel| channel == mf_ptr)
            .unwrap_or(false)
    {
        if mf.flush_pos < mf.size {
            let data = mfile_data(mf);
            let bytes = &data[mf.flush_pos..mf.size];
            if libc::fwrite(bytes.as_ptr().cast(), 1, bytes.len(), mf.fp) < bytes.len() {
                return -1;
            }
            if libc::fflush(mf.fp) != 0 {
                return -1;
            }
        }
        mf.offset = 0;
        mf.size = 0;
        mf.flush_pos = 0;
    }

    if (mf.mode & MF_WRITE) != 0 {
        if mf.flush_pos < mf.size {
            let data = mfile_data(mf);
            let bytes = &data[mf.flush_pos..mf.size];
            if (mf.mode & MF_MODEX) == 0 {
                libc::fseek(mf.fp, mf.flush_pos as libc::c_long, libc::SEEK_SET);
            }
            if libc::fwrite(bytes.as_ptr().cast(), 1, bytes.len(), mf.fp) < bytes.len() {
                return -1;
            }
            if libc::fflush(mf.fp) != 0 {
                return -1;
            }
        }
        let pos = libc::ftell(mf.fp);
        if pos != -1 && crate::htslib_rs::c_compat::ftruncate(libc::fileno(mf.fp), pos) == -1 {
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

unsafe fn mfascii_borrowed(mf: &mut mFILE) {
    let mut p1 = 1usize;
    let mut p2 = 1usize;
    let size = mf.size;
    let Some(data) = mfile_allocated_mut(mf) else {
        return;
    };

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
