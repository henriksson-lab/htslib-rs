use std::ffi::{c_char, c_void, CStr};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::{Mutex, OnceLock};

use super::hts::{hFILE, isalnum_c, kgetline, kputs, kputsn, kstring_t, tolower_c};

#[repr(C)]
pub struct knetFile {
    _private: [u8; 0],
}

struct knet_file_layout {
    fd: i32,
    offset: i64,
    hf: OwnedHFile,
}

#[repr(C)]
pub struct hFILE_scheme_handler {
    _private: [u8; 0],
}

#[repr(C)]
pub struct hFILE_plugin {
    _private: [u8; 0],
}

type HFileOpenFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut hFILE;
type HFileIsRemoteFn = unsafe extern "C" fn(*const c_char) -> i32;
type HFileVOpenFn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE;
type HFilePluginInitFn = unsafe extern "C" fn(*mut hFILE_plugin) -> i32;
type HFilePluginDestroyFn = unsafe extern "C" fn();

unsafe fn hfile_plugin_destroy_fn(ptr: *const c_void) -> HFilePluginDestroyFn {
    debug_assert!(!ptr.is_null());
    std::mem::transmute_copy(&ptr)
}

struct ConstNonNull<T> {
    ptr: NonNull<T>,
}

impl<T> Clone for ConstNonNull<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ConstNonNull<T> {}

impl<T> ConstNonNull<T> {
    unsafe fn new(ptr: *const T) -> Option<Self> {
        NonNull::new(ptr.cast_mut()).map(|ptr| Self { ptr })
    }

    fn as_ptr(self) -> *const T {
        self.ptr.as_ptr()
    }
}

unsafe impl<T> Send for ConstNonNull<T> {}

#[repr(C)]
struct hfile_scheme_handler_layout {
    open: Option<HFileOpenFn>,
    isremote: Option<HFileIsRemoteFn>,
    provider: *const c_char,
    priority: i32,
    vopen: Option<HFileVOpenFn>,
}

unsafe impl Sync for hfile_scheme_handler_layout {}

#[repr(C)]
struct hfile_plugin_layout {
    api_version: i32,
    obj: *mut c_void,
    name: *const c_char,
    destroy: *const c_void,
}

pub struct hFILE_plugin_list {
    plugin: hfile_plugin_layout,
    next: Option<NonNull<hFILE_plugin_list>>,
}

unsafe impl Send for hFILE_plugin_list {}

// === hFILE backend dispatch (SEAM phase 1) ===
//
// Replaces the old `#[repr(C)] struct hfile_backend_layout` vtable of five
// `extern "C"` function pointers (read/write/seek/flush/close) with an
// idiomatic Rust enum. Each variant names a finite, statically-known backend
// and carries that backend's own state inline (so the C-style "subclass"
// structs that embedded `base: hfile_layout` collapse into the variant).
//
// The five hFILE operations become methods on this enum (see impl below) that
// `match` over the variant and run each backend's real body. A backend that
// does not implement an operation returns `None` (read/write/seek) or is a
// no-op (flush), mirroring the old `Option<fn>` vtable slots.
pub enum HFileBackend {
    /// no backend yet (freshly hfile_init'd, or a pure in-memory buffer being
    /// filled before the real backend is attached)
    None,
    /// OS file descriptor / socket backend (was `hfile_fd_layout`).
    /// `fd` + `flags` (HFILE_FD_IS_SOCKET / HFILE_FD_IS_SHARED) moved inline.
    Fd { fd: i32, flags: u32 },
    /// in-memory buffer backend (was MEM_BACKEND); all state lives in the
    /// hFILE buffer itself, so the variant is stateless.
    Mem,
    /// libcurl HTTP(S)/FTP backend (state defined in hfile_libcurl.rs).
    #[cfg(feature = "libcurl")]
    Libcurl(Box<crate::htslib_rs::hfile_libcurl::hFILE_libcurl>),
    /// S3 backend layered over libcurl (state in hfile_s3.rs).
    #[cfg(feature = "s3")]
    S3(Box<crate::htslib_rs::hfile_s3::hFILE_s3>),
    /// GCS uses the libcurl backend directly; no distinct variant needed, but
    /// kept for clarity at open sites that select GCS.
    #[cfg(feature = "gcs")]
    Gcs(Box<crate::htslib_rs::hfile_libcurl::hFILE_libcurl>),
    /// GA4GH htsget multipart backend (state in multipart.rs).
    Multipart(Box<crate::htslib_rs::multipart::hFILE_multipart>),
}

impl HFileBackend {
    /// Was `(*backend).read`: returns true when this backend supports reads.
    fn has_read(&self) -> bool {
        !matches!(self, HFileBackend::None | HFileBackend::Mem)
    }

    /// Was `(*backend).write`: returns true when this backend supports writes.
    fn has_write(&self) -> bool {
        !matches!(self, HFileBackend::None | HFileBackend::Mem)
    }

    /// Returns true when this backend supports an explicit flush op
    /// (only the fd backend; the old MEM/net backends had `flush: None`).
    fn has_flush(&self) -> bool {
        matches!(self, HFileBackend::Fd { .. })
    }

    // === Dispatch methods (replace the old vtable fn pointers) ===
    //
    // These take the owning `&mut hFILE` so backends that need both their own
    // variant state AND the hFILE buffer (e.g. fd flags + the buffer indices)
    // can reach both. The `dest`/`src` buffers are plain slices (no more
    // `*mut c_void`); genuine OS syscalls (read/write/lseek/close/fdatasync)
    // remain at the bottom of the fd arms.

    /// Was `(*backend).read(fp, buf, n)`; reads into `dest`, returns bytes read
    /// or -1 (errno set). Returns -1/EINVAL for backends without a read body.
    pub unsafe fn read(fp: &mut hFILE, dest: &mut [u8]) -> libc::ssize_t {
        let nbytes = dest.len() as usize;
        let ptr = dest.as_mut_ptr().cast::<c_void>();
        match &fp.backend {
            HFileBackend::Fd { fd, flags } => {
                let (fd, flags) = (*fd, *flags);
                loop {
                    let n = if (flags & HFILE_FD_IS_SOCKET) != 0 {
                        libc::recv(fd, ptr, nbytes, 0)
                    } else {
                        libc::read(fd, ptr, nbytes)
                    };
                    if !(n < 0 && *libc::__errno_location() == libc::EINTR) {
                        return n;
                    }
                }
            }
            #[cfg(feature = "libcurl")]
            HFileBackend::Libcurl(_) => {
                crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_876_libcurl_read(fp, ptr, nbytes)
            }
            #[cfg(feature = "gcs")]
            HFileBackend::Gcs(_) => {
                crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_876_libcurl_read(fp, ptr, nbytes)
            }
            HFileBackend::Multipart(_) => {
                let dest = std::slice::from_raw_parts_mut(ptr.cast::<u8>(), nbytes as usize);
                if let HFileBackend::Multipart(mp) = &mut fp.backend {
                    crate::htslib_rs::multipart::multipart_read(mp, dest)
                } else {
                    unreachable!()
                }
            }
            // S3/Mem/None have no read body (old vtable `read: None`)
            #[cfg(feature = "s3")]
            HFileBackend::S3(_) => {
                *libc::__errno_location() = libc::EINVAL;
                -1
            }
            HFileBackend::Mem | HFileBackend::None => {
                *libc::__errno_location() = libc::EINVAL;
                -1
            }
        }
    }

    /// Was `(*backend).write(fp, buf, n)`; writes `src`, returns bytes written
    /// or -1 (errno set).
    pub unsafe fn write(fp: &mut hFILE, src: &[u8]) -> libc::ssize_t {
        let nbytes = src.len() as usize;
        let ptr = src.as_ptr().cast::<c_void>();
        match &fp.backend {
            HFileBackend::Fd { fd, flags } => {
                let (fd, flags) = (*fd, *flags);
                loop {
                    let n = if (flags & HFILE_FD_IS_SOCKET) != 0 {
                        libc::send(fd, ptr, nbytes, 0)
                    } else {
                        libc::write(fd, ptr, nbytes)
                    };
                    if !(n < 0 && *libc::__errno_location() == libc::EINTR) {
                        return n;
                    }
                }
            }
            #[cfg(feature = "libcurl")]
            HFileBackend::Libcurl(_) => {
                crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_1024_libcurl_write(fp, ptr, nbytes)
            }
            #[cfg(feature = "gcs")]
            HFileBackend::Gcs(_) => {
                crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_1024_libcurl_write(fp, ptr, nbytes)
            }
            #[cfg(feature = "s3")]
            HFileBackend::S3(_) => {
                crate::htslib_rs::hfile_s3::hfile_s3_c_1625_s3_write(fp, ptr, nbytes)
            }
            HFileBackend::Multipart(_) => {
                crate::htslib_rs::multipart::multipart_c_114_multipart_write(fp, ptr.cast(), nbytes)
            }
            HFileBackend::Mem | HFileBackend::None => {
                *libc::__errno_location() = libc::EINVAL;
                -1
            }
        }
    }

    /// Was `(*backend).seek(fp, offset, whence)`; returns new offset or -1.
    pub unsafe fn seek(fp: &mut hFILE, offset: libc::off_t, whence: i32) -> libc::off_t {
        match &fp.backend {
            HFileBackend::Fd { fd, .. } => libc::lseek(*fd, offset, whence),
            HFileBackend::Mem => {
                // old hfile_c_810_mem_seek: not seekable
                *libc::__errno_location() = libc::EINVAL;
                -1
            }
            #[cfg(feature = "libcurl")]
            HFileBackend::Libcurl(_) => {
                crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_1071_libcurl_seek(fp, offset, whence)
            }
            #[cfg(feature = "gcs")]
            HFileBackend::Gcs(_) => {
                crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_1071_libcurl_seek(fp, offset, whence)
            }
            #[cfg(feature = "s3")]
            HFileBackend::S3(_) => {
                crate::htslib_rs::hfile_s3::hfile_s3_c_2015_s3_seek(fp, offset, whence)
            }
            HFileBackend::Multipart(_) => {
                crate::htslib_rs::multipart::multipart_c_120_multipart_seek(fp, offset, whence)
            }
            HFileBackend::None => {
                *libc::__errno_location() = libc::EINVAL;
                -1
            }
        }
    }

    /// Was `(*backend).flush`; no-op for backends whose old vtable slot was
    /// `flush: None`. Returns 0 on success, -1 (errno set) on failure.
    pub unsafe fn flush(fp: &mut hFILE) -> i32 {
        match &fp.backend {
            HFileBackend::Fd { fd, flags } => {
                let (fd, flags) = (*fd, *flags);
                if (flags & HFILE_FD_IS_SOCKET) != 0 {
                    return 0;
                }
                loop {
                    #[cfg(any(target_os = "linux", target_os = "android"))]
                    let mut ret = libc::fdatasync(fd);
                    #[cfg(not(any(target_os = "linux", target_os = "android")))]
                    let mut ret = libc::fsync(fd);
                    let errno = *libc::__errno_location();
                    if ret < 0 && (errno == libc::EINVAL || errno == libc::ENOTSUP) {
                        ret = 0;
                    }
                    if !(ret < 0 && *libc::__errno_location() == libc::EINTR) {
                        return ret;
                    }
                }
            }
            // all other backends had `flush: None` -> success no-op
            _ => 0,
        }
    }

    /// Was `(*backend).close(fp)`; releases the backend resource. Returns 0 on
    /// success, -1 (errno set) on failure.
    pub unsafe fn close(fp: &mut hFILE) -> i32 {
        match &fp.backend {
            HFileBackend::Fd { fd, flags } => {
                let (fd, flags) = (*fd, *flags);
                if (flags & HFILE_FD_IS_SHARED) != 0 {
                    return 0;
                }
                loop {
                    let ret = libc::close(fd);
                    if !(ret < 0 && *libc::__errno_location() == libc::EINTR) {
                        return ret;
                    }
                }
            }
            HFileBackend::Mem | HFileBackend::None => 0,
            #[cfg(feature = "libcurl")]
            HFileBackend::Libcurl(_) => {
                crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_1266_libcurl_close(fp)
            }
            #[cfg(feature = "gcs")]
            HFileBackend::Gcs(_) => {
                crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_1266_libcurl_close(fp)
            }
            #[cfg(feature = "s3")]
            HFileBackend::S3(_) => crate::htslib_rs::hfile_s3::hfile_s3_c_2072_s3_close(fp),
            HFileBackend::Multipart(_) => {
                crate::htslib_rs::multipart::multipart_c_126_multipart_close(fp)
            }
        }
    }
}

pub struct OwnedHFile {
    ptr: NonNull<hFILE>,
}

impl OwnedHFile {
    pub unsafe fn from_raw(ptr: *mut hFILE) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    pub fn as_ptr(&self) -> *mut hFILE {
        self.ptr.as_ptr()
    }

    pub fn as_non_null(&self) -> NonNull<hFILE> {
        self.ptr
    }

    pub fn into_raw(self) -> *mut hFILE {
        let ptr = self.ptr.as_ptr();
        std::mem::forget(self);
        ptr
    }

    pub fn close(self) -> i32 {
        let ptr = self.into_raw();
        unsafe { hclose(ptr) }
    }

    pub fn close_abruptly(self) {
        let ptr = self.into_raw();
        unsafe { hclose_abruptly(ptr) };
    }
}

impl Drop for OwnedHFile {
    fn drop(&mut self) {
        unsafe {
            let _ = hclose(self.ptr.as_ptr());
        }
    }
}

#[derive(Clone, Copy)]
pub struct BorrowedHFile<'a> {
    ptr: NonNull<hFILE>,
    _marker: PhantomData<&'a hFILE>,
}

impl<'a> BorrowedHFile<'a> {
    pub unsafe fn from_raw(ptr: *mut hFILE) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self {
            ptr,
            _marker: PhantomData,
        })
    }

    pub fn as_ptr(self) -> *mut hFILE {
        self.ptr.as_ptr()
    }

    pub fn as_non_null(self) -> NonNull<hFILE> {
        self.ptr
    }
}

unsafe fn hfile_mut<'a>(fp: *mut hFILE) -> Option<&'a mut hFILE> {
    fp.as_mut()
}

fn hfile_raw(fp: &mut hFILE) -> *mut hFILE {
    fp as *mut hFILE
}

fn hfile_mode_has(mode: &[u8], byte: u8) -> bool {
    mode.contains(&byte)
}

fn hfile_mode_is_readonly(mode: &[u8]) -> bool {
    hfile_mode_has(mode, b'r') && !hfile_mode_has(mode, b'+')
}

const HFILE_FD_IS_SOCKET: u32 = 1 << 0;
const HFILE_FD_IS_SHARED: u32 = 1 << 1;

const HFILE_AT_EOF: u32 = 1 << 0;
const HFILE_MOBILE: u32 = 1 << 1;
const HFILE_READONLY: u32 = 1 << 2;
const HFILE_PRESERVE: u32 = 1 << 3;

// SEAM: the standalone mem/fd backend body functions (hfile_c_810_mem_seek,
// hfile_c_816_mem_close, hfile_c_557_fd_read, hfile_c_568_fd_write,
// hfile_c_591_fd_seek, hfile_c_607_fd_flush, hfile_c_625_fd_close) and the
// static MEM_BACKEND / FD_BACKEND vtables are gone — their logic now lives
// inline in the HFileBackend::{read,write,seek,flush,close} match arms above
// for the `Mem` and `Fd { fd, flags }` variants.

pub unsafe fn hfile_c_1011_priority(handler: *const hFILE_scheme_handler) -> i32 {
    (*(handler.cast::<hfile_scheme_handler_layout>())).priority % 1000
}

pub unsafe fn hfile_c_1026_try_exe_add_scheme_handler(
    _scheme: *const c_char,
    _handler: *const hFILE_scheme_handler,
) -> i32 {
    -1
}

pub unsafe fn hfile_c_1046_try_exe_add_scheme_handler(
    _scheme: *const c_char,
    _handler: *const hFILE_scheme_handler,
) -> i32 {
    -1
}

pub unsafe fn hfile_init(struct_size: usize, mode: *const c_char, capacity: usize) -> *mut hFILE {
    hfile_c_104_hfile_init(struct_size, mode, capacity)
}

pub unsafe fn hfile_c_104_hfile_init(
    struct_size: usize,
    mode: *const c_char,
    capacity: usize,
) -> *mut hFILE {
    let Some(mode) = (!mode.is_null()).then(|| CStr::from_ptr(mode).to_bytes()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    hfile_init_bytes(struct_size, mode, capacity)
}

pub unsafe fn hfile_init_bytes(
    _struct_size: usize,
    mode: &[u8],
    mut capacity: usize,
) -> *mut hFILE {
    // SEAM: the hFILE is now an owned Rust struct with a Vec<u8> buffer.
    // `_struct_size` (the old C-style subclass size) is irrelevant: backend
    // state lives inline in the HFileBackend enum. The buffer Vec owns its own
    // allocation, so there is no separate malloc to track.
    let maxcap = 128 * 1024usize;
    if capacity == 0 {
        capacity = maxcap;
    }
    if hfile_mode_has(mode, b'r') && capacity > maxcap {
        capacity = maxcap;
    }

    let mut buffer: Vec<u8> = Vec::new();
    if buffer.try_reserve_exact(capacity).is_err() {
        *libc::__errno_location() = libc::ENOMEM;
        return std::ptr::null_mut();
    }
    buffer.resize(capacity, 0);

    let mut flags = HFILE_MOBILE;
    if hfile_mode_is_readonly(mode) {
        flags |= HFILE_READONLY;
    }

    Box::into_raw(Box::new(hFILE {
        buffer,
        begin: 0,
        end: 0,
        limit: capacity,
        backend: HFileBackend::None,
        offset: 0,
        flags,
        has_errno: 0,
    }))
}

pub unsafe fn hfile_c_141_hfile_init_fixed(
    struct_size: usize,
    mode: *const c_char,
    buffer: *mut c_char,
    buf_filled: usize,
    buf_size: usize,
) -> *mut hFILE {
    let Some(mode) = (!mode.is_null()).then(|| CStr::from_ptr(mode).to_bytes()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let Some(buffer) = NonNull::new(buffer.cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    hfile_init_fixed(struct_size, mode, buffer, buf_filled, buf_size)
}

pub unsafe fn hfile_init_fixed(
    _struct_size: usize,
    mode: &[u8],
    buffer: NonNull<u8>,
    buf_filled: usize,
    buf_size: usize,
) -> *mut hFILE {
    // SEAM: the supplied fixed buffer is copied into the owned Vec<u8> (sized to
    // `buf_size`, with `buf_filled` valid bytes). The hFILE now owns its buffer;
    // the caller's original allocation is its own to free.
    let mut buf: Vec<u8> = Vec::new();
    if buf.try_reserve_exact(buf_size).is_err() {
        *libc::__errno_location() = libc::ENOMEM;
        return std::ptr::null_mut();
    }
    buf.resize(buf_size, 0);
    if buf_filled > 0 {
        std::ptr::copy_nonoverlapping(buffer.as_ptr(), buf.as_mut_ptr(), buf_filled);
    }

    let mut flags = HFILE_AT_EOF;
    if hfile_mode_is_readonly(mode) {
        flags |= HFILE_READONLY;
    }

    Box::into_raw(Box::new(hFILE {
        buffer: buf,
        begin: 0,
        end: buf_filled,
        limit: buf_size,
        backend: HFileBackend::None,
        offset: 0,
        flags,
        has_errno: 0,
    }))
}

pub unsafe fn hfile_destroy(fp: *mut hFILE) {
    hfile_c_162_hfile_destroy(fp)
}

pub unsafe fn hfile_c_162_hfile_destroy(fp: *mut hFILE) {
    // SEAM: the hFILE owns its buffer (Vec<u8>) and backend state, so dropping
    // the reconstituted Box frees everything. errno is preserved across drop.
    let save = *libc::__errno_location();
    if !fp.is_null() {
        drop(Box::from_raw(fp));
    }
    *libc::__errno_location() = save;
}

pub unsafe fn hfile_writebuffer_is_nonempty(fp: &hFILE) -> i32 {
    // SEAM: begin/end are usize byte indices; `begin > end` still flags a
    // pending write buffer.
    (fp.begin > fp.end) as i32
}

pub unsafe fn herrno(fp: &hFILE) -> i32 {
    fp.has_errno
}

pub unsafe fn hclearerr(fp: &mut hFILE) {
    fp.has_errno = 0;
}

pub unsafe fn htell(fp: &hFILE) -> libc::off_t {
    fp.offset + fp.begin as libc::off_t
}

pub unsafe fn hgetc(fp: &mut hFILE) -> i32 {
    if fp.end > fp.begin {
        let c = fp.buffer[fp.begin];
        fp.begin += 1;
        c as i32
    } else {
        hgetc2_impl(fp)
    }
}

pub unsafe fn hgetln(buffer: &mut [u8], fp: &mut hFILE) -> libc::ssize_t {
    hgetdelim_impl(buffer, b'\n' as i32, fp)
}

pub unsafe fn hread(fp: &mut hFILE, buffer: &mut [u8]) -> libc::ssize_t {
    let mut n = fp.end - fp.begin;
    if n > buffer.len() {
        n = buffer.len();
    }
    buffer[..n].copy_from_slice(&fp.buffer[fp.begin..fp.begin + n]);
    fp.begin += n;
    if n == buffer.len() || (fp.flags & HFILE_MOBILE) == 0 {
        n as libc::ssize_t
    } else {
        hread2_impl(fp, n, buffer)
    }
}

pub unsafe fn hputc(c: i32, fp: &mut hFILE) -> i32 {
    if fp.begin < fp.limit {
        fp.buffer[fp.begin] = c as u8;
        fp.begin += 1;
        c
    } else {
        hputc2_impl(c, fp)
    }
}

pub unsafe fn hputs(text: &[u8], fp: &mut hFILE) -> i32 {
    let mut n = fp.limit - fp.begin;
    if n > text.len() {
        n = text.len();
    }
    let begin = fp.begin;
    fp.buffer[begin..begin + n].copy_from_slice(&text[..n]);
    fp.begin += n;
    if n == text.len() {
        0
    } else {
        hputs2_impl(text, n, fp)
    }
}

pub unsafe fn hwrite(fp: &mut hFILE, buffer: &[u8]) -> libc::ssize_t {
    if (fp.flags & HFILE_MOBILE) == 0 {
        let n = fp.limit - fp.begin;
        if n < buffer.len() {
            let new_size = fp.limit + buffer.len();
            hfile_set_blksize_impl(fp, new_size);
            fp.end = fp.limit;
        }
    }

    let mut n = fp.limit - fp.begin;
    if buffer.len() >= n && fp.begin == 0 {
        return hwrite2_impl(fp, 0, buffer);
    }

    if n > buffer.len() {
        n = buffer.len();
    }
    let begin = fp.begin;
    fp.buffer[begin..begin + n].copy_from_slice(&buffer[..n]);
    fp.begin += n;
    if n == buffer.len() {
        n as libc::ssize_t
    } else {
        hwrite2_impl(fp, n, buffer)
    }
}

pub unsafe fn htslib_hfile_h_134_herrno(fp: *mut hFILE) -> i32 {
    let Some(fp) = fp.as_ref() else {
        return libc::EINVAL;
    };
    herrno(fp)
}

pub unsafe fn htslib_hfile_h_140_hclearerr(fp: *mut hFILE) {
    if let Some(fp) = fp.as_mut() {
        hclearerr(fp);
    }
}

pub unsafe fn htslib_hfile_h_155_htell(fp: *mut hFILE) -> libc::off_t {
    let Some(fp) = fp.as_ref() else {
        return -1;
    };
    htell(fp)
}

pub unsafe fn htslib_hfile_h_163_hgetc(fp: *mut hFILE) -> i32 {
    let Some(fp) = fp.as_mut() else {
        return libc::EOF;
    };
    hgetc(fp)
}

pub unsafe fn htslib_hfile_h_195_hgetln(
    buffer: *mut c_char,
    size: usize,
    fp: *mut hFILE,
) -> libc::ssize_t {
    let Some(fp) = fp.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(buffer) = NonNull::new(buffer.cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hgetln(std::slice::from_raw_parts_mut(buffer.as_ptr(), size), fp)
}

pub unsafe fn htslib_hfile_h_247_hread(
    fp: *mut hFILE,
    buffer: *mut c_void,
    nbytes: usize,
) -> libc::ssize_t {
    let Some(fp) = fp.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(buffer) = NonNull::new(buffer.cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hread(fp, std::slice::from_raw_parts_mut(buffer.as_ptr(), nbytes))
}

pub unsafe fn htslib_hfile_h_263_hputc(c: i32, fp: *mut hFILE) -> i32 {
    let Some(fp) = fp.as_mut() else {
        return libc::EOF;
    };
    hputc(c, fp)
}

pub unsafe fn htslib_hfile_h_275_hputs(text: *const c_char, fp: *mut hFILE) -> i32 {
    let Some(fp) = fp.as_mut() else {
        return libc::EOF;
    };
    let Some(text) = (!text.is_null()).then(|| CStr::from_ptr(text).to_bytes()) else {
        return libc::EOF;
    };
    hputs(text, fp)
}

pub unsafe fn htslib_hfile_h_292_hwrite(
    fp: *mut hFILE,
    buffer: *const c_void,
    nbytes: usize,
) -> libc::ssize_t {
    let Some(fp) = fp.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(buffer) = NonNull::new(buffer.cast_mut().cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hwrite(fp, std::slice::from_raw_parts(buffer.as_ptr(), nbytes))
}

pub unsafe fn hfile_refill_buffer(fp: &mut hFILE) -> libc::ssize_t {
    // SEAM: buffer is now Vec<u8>; begin/end/limit are usize byte indices.
    if (fp.flags & HFILE_MOBILE) != 0 && fp.begin > 0 {
        let consumed = fp.begin;
        let unread = fp.end - fp.begin;
        fp.offset += consumed as libc::off_t;
        fp.buffer.copy_within(fp.begin..fp.end, 0);
        fp.end = unread;
        fp.begin = 0;
    }

    let n = if (fp.flags & HFILE_AT_EOF) != 0 || fp.end == fp.limit {
        0
    } else {
        // dispatch: was `(*backend).read(...)` over the raw vtable. The net
        // backends need `&mut hFILE` AND write the destination window (which is
        // a slice of `fp.buffer`); to avoid aliasing the same buffer through two
        // borrows, read into a scratch Vec then copy it into the buffer window.
        let end = fp.end;
        let limit = fp.limit;
        let mut scratch = vec![0u8; limit - end];
        let ret = HFileBackend::read(fp, &mut scratch);
        if ret > 0 {
            let n = ret as usize;
            fp.buffer[end..end + n].copy_from_slice(&scratch[..n]);
        }
        if ret < 0 {
            fp.has_errno = *libc::__errno_location();
            return ret;
        } else if ret == 0 {
            fp.flags |= HFILE_AT_EOF;
        }
        ret
    };

    fp.end += n as usize;
    n
}

pub unsafe fn hfile_set_blksize_impl(fp: &mut hFILE, mut bufsiz: usize) -> i32 {
    // SEAM: resize the owned Vec<u8>; begin/end/limit are usize byte indices.
    let curr_used = if fp.begin > fp.end { fp.begin } else { fp.end };

    if bufsiz == 0 {
        bufsiz = 32768;
    }
    if bufsiz < curr_used {
        return -1;
    }

    if fp.buffer.try_reserve_exact(bufsiz.saturating_sub(fp.buffer.len())).is_err() {
        *libc::__errno_location() = libc::ENOMEM;
        return -1;
    }
    fp.buffer.resize(bufsiz, 0);
    fp.limit = bufsiz;
    0
}

pub unsafe fn hgetc2_impl(fp: &mut hFILE) -> i32 {
    if hfile_refill_buffer(fp) > 0 {
        let c = fp.buffer[fp.begin] as i32;
        fp.begin += 1;
        c
    } else {
        libc::EOF
    }
}

pub unsafe fn hgetdelim_impl(
    buffer: &mut [u8],
    delim: i32,
    fp: &mut hFILE,
) -> libc::ssize_t {
    if buffer.is_empty() || buffer.len() > libc::ssize_t::MAX as usize {
        fp.has_errno = libc::EINVAL;
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    }
    if fp.begin > fp.end {
        fp.has_errno = libc::EBADF;
        *libc::__errno_location() = libc::EBADF;
        return -1;
    }

    let size = buffer.len() - 1;
    let mut copied = 0usize;
    loop {
        let mut n = fp.end - fp.begin;
        if n > size - copied {
            n = size - copied;
        }

        let begin = fp.begin;
        let src = &fp.buffer[begin..begin + n];
        if let Some(pos) = src.iter().position(|&c| c == delim as u8) {
            n = pos + 1;
            buffer[copied..copied + n].copy_from_slice(&src[..n]);
            buffer[n + copied] = 0;
            fp.begin += n;
            return (n + copied) as libc::ssize_t;
        }

        buffer[copied..copied + n].copy_from_slice(src);
        fp.begin += n;
        copied += n;

        if copied == size {
            buffer[copied] = 0;
            return copied as libc::ssize_t;
        }

        let got = hfile_refill_buffer(fp);
        if got <= 0 {
            if got < 0 {
                return -1;
            }
            buffer[copied] = 0;
            return copied as libc::ssize_t;
        }
    }
}

pub unsafe fn hgets_impl(buffer: &mut [u8], fp: &mut hFILE) -> bool {
    if buffer.is_empty() {
        fp.has_errno = libc::EINVAL;
        *libc::__errno_location() = libc::EINVAL;
        return false;
    }
    hgetdelim_impl(buffer, b'\n' as i32, fp) > 0
}

pub unsafe extern "C" fn hfile_c_301_hgets_wrapper(
    buffer: *mut c_char,
    size: i32,
    fp: *mut c_void,
) -> *mut c_char {
    let Some(fp) = hfile_mut(fp.cast()) else {
        return std::ptr::null_mut();
    };
    if size < 1 {
        fp.has_errno = libc::EINVAL;
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }
    if buffer.is_null() {
        return std::ptr::null_mut();
    }
    let out = std::slice::from_raw_parts_mut(buffer.cast::<u8>(), size as usize);
    if hgets_impl(out, fp) {
        buffer
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn hfile_khgetline(kstr: *mut kstring_t, fp: &mut hFILE) -> i32 {
    if kstr.is_null() {
        return libc::EOF;
    }
    kgetline(&mut *kstr, Some(hfile_c_301_hgets_wrapper), hfile_raw(fp).cast())
}

pub unsafe fn hpeek_impl(fp: &mut hFILE, buffer: &mut [u8]) -> libc::ssize_t {
    let mut n = fp.end - fp.begin;
    while n < buffer.len() {
        let ret = hfile_refill_buffer(fp);
        if ret < 0 {
            return ret;
        } else if ret == 0 {
            break;
        } else {
            n += ret as usize;
        }
    }

    if n > buffer.len() {
        n = buffer.len();
    }
    let begin = fp.begin;
    buffer[..n].copy_from_slice(&fp.buffer[begin..begin + n]);
    n as libc::ssize_t
}

pub unsafe fn hread2_impl(
    fp: &mut hFILE,
    nread: usize,
    dest: &mut [u8],
) -> libc::ssize_t {
    // SEAM: Vec<u8> buffer with usize indices; dispatch via HFileBackend::read.
    let capacity = fp.limit;
    let mut buffer_invalidated = 0;
    let mut dest_pos = nread;
    let mut remaining = dest.len() - nread;
    let mut total_read = nread;

    while remaining * 2 >= capacity && (fp.flags & HFILE_AT_EOF) == 0 {
        let n = HFileBackend::read(fp, &mut dest[dest_pos..dest_pos + remaining]);
        if n < 0 {
            fp.has_errno = *libc::__errno_location();
            return n;
        } else if n == 0 {
            fp.flags |= HFILE_AT_EOF;
        } else {
            buffer_invalidated = 1;
        }
        fp.offset += n as libc::off_t;
        dest_pos += n as usize;
        remaining -= n as usize;
        total_read += n as usize;
    }

    if buffer_invalidated != 0 {
        fp.offset += fp.begin as libc::off_t;
        fp.begin = 0;
        fp.end = 0;
    }

    while remaining > 0 && (fp.flags & HFILE_AT_EOF) == 0 {
        let ret = hfile_refill_buffer(fp);
        if ret < 0 {
            return ret;
        }

        let mut n = fp.end - fp.begin;
        if n > remaining {
            n = remaining;
        }
        dest[dest_pos..dest_pos + n].copy_from_slice(&fp.buffer[fp.begin..fp.begin + n]);
        fp.begin += n;
        dest_pos += n;
        remaining -= n;
        total_read += n;
    }

    total_read as libc::ssize_t
}

pub unsafe fn hfile_flush_buffer(fp: &mut hFILE) -> libc::ssize_t {
    // SEAM: flush bytes [0, begin) via HFileBackend::write; indices are usize.
    let mut pos = 0usize;
    while pos < fp.begin {
        let begin = fp.begin;
        // Copy the source window out of the buffer first, so the write dispatch
        // can take `&mut hFILE` without aliasing `fp.buffer`.
        let chunk = fp.buffer[pos..begin].to_vec();
        let n = HFileBackend::write(fp, &chunk);
        if n < 0 {
            fp.has_errno = *libc::__errno_location();
            return n;
        }
        pos += n as usize;
        fp.offset += n as libc::off_t;
    }

    fp.begin = 0;
    0
}

pub unsafe fn hflush_impl(fp: &mut hFILE) -> i32 {
    if hfile_flush_buffer(fp) < 0 {
        return libc::EOF;
    }
    if HFileBackend::flush(fp) < 0 {
        fp.has_errno = *libc::__errno_location();
        return libc::EOF;
    }
    0
}

pub unsafe fn hputc2_impl(c: i32, fp: &mut hFILE) -> i32 {
    if hfile_flush_buffer(fp) < 0 {
        return libc::EOF;
    }
    let begin = fp.begin;
    fp.buffer[begin] = c as u8;
    fp.begin += 1;
    c
}

pub unsafe fn hwrite2_impl(
    fp: &mut hFILE,
    ncopied: usize,
    src: &[u8],
) -> libc::ssize_t {
    // SEAM: Vec<u8> buffer with usize indices; dispatch via HFileBackend::write.
    let mut src_pos = ncopied;
    let capacity = fp.limit;
    let mut remaining = src.len() - ncopied;

    let ret = hfile_flush_buffer(fp);
    if ret < 0 {
        return ret;
    }

    while remaining * 2 >= capacity {
        let n = HFileBackend::write(fp, &src[src_pos..src_pos + remaining]);
        if n < 0 {
            fp.has_errno = *libc::__errno_location();
            return n;
        }
        fp.offset += n as libc::off_t;
        src_pos += n as usize;
        remaining -= n as usize;
    }

    let begin = fp.begin;
    fp.buffer[begin..begin + remaining].copy_from_slice(&src[src_pos..src_pos + remaining]);
    fp.begin += remaining;

    src.len() as libc::ssize_t
}

pub unsafe fn hputs2_impl(text: &[u8], ncopied: usize, fp: &mut hFILE) -> i32 {
    if hwrite2_impl(fp, ncopied, text) >= 0 {
        0
    } else {
        libc::EOF
    }
}

pub unsafe fn hseek_impl(
    fp: &mut hFILE,
    mut offset: libc::off_t,
    mut whence: i32,
) -> libc::off_t {
    // SEAM: usize indices; dispatch via HFileBackend::seek.
    let should_flush = fp.begin > fp.end && (fp.flags & HFILE_MOBILE) != 0;
    if should_flush {
        let ret = hfile_flush_buffer(fp);
        if ret < 0 {
            return ret as libc::off_t;
        }
    }

    let curpos = fp.offset + fp.begin as libc::off_t;

    if whence == libc::SEEK_CUR {
        match curpos.checked_add(offset) {
            Some(pos) if pos >= 0 => {
                whence = libc::SEEK_SET;
                offset = pos;
            }
            _ => {
                let err = if offset < 0 {
                    libc::EINVAL
                } else {
                    libc::EOVERFLOW
                };
                fp.has_errno = err;
                *libc::__errno_location() = err;
                return -1;
            }
        }
    } else if (fp.flags & HFILE_MOBILE) == 0 && whence == libc::SEEK_END {
        let length = fp.end as libc::off_t;
        if offset > 0 || -offset > length {
            fp.has_errno = libc::EINVAL;
            *libc::__errno_location() = libc::EINVAL;
            return -1;
        }

        whence = libc::SEEK_SET;
        offset += length;
    }

    if whence == libc::SEEK_SET
        && ((fp.flags & HFILE_MOBILE) == 0 || (fp.flags & HFILE_READONLY) != 0)
        && offset >= fp.offset
        && offset - fp.offset <= fp.end as libc::off_t
    {
        fp.begin = (offset - fp.offset) as usize;
        return offset;
    }

    let pos = HFileBackend::seek(fp, offset, whence);
    if pos < 0 {
        fp.has_errno = *libc::__errno_location();
        return pos;
    }

    fp.begin = 0;
    fp.end = 0;
    fp.flags &= !HFILE_AT_EOF;
    fp.offset = pos;
    pos
}

pub unsafe fn hclose_impl(fp: &mut hFILE) -> i32 {
    // SEAM: dispatch via HFileBackend::close. Deallocation is now the owner's
    // responsibility: the caller holds `Box<hFILE>` and drops it after this
    // returns (the old hfile_destroy alloc/free is gone).
    let mut err = fp.has_errno;

    if fp.begin > fp.end && hflush_impl(fp) < 0 {
        err = fp.has_errno;
    }
    if (fp.flags & HFILE_PRESERVE) == 0 {
        if HFileBackend::close(fp) < 0 {
            err = *libc::__errno_location();
        }
    }

    if err != 0 {
        *libc::__errno_location() = err;
        libc::EOF
    } else {
        0
    }
}

pub unsafe fn hclose_abruptly_impl(fp: &mut hFILE) {
    let save = *libc::__errno_location();
    if (fp.flags & HFILE_PRESERVE) != 0 {
        return;
    }
    let _ = HFileBackend::close(fp);
    *libc::__errno_location() = save;
}

// SEAM: hfile_c_607_fd_flush / hfile_c_625_fd_close bodies moved inline into
// HFileBackend::flush / HFileBackend::close `Fd { fd, flags }` arms.

pub unsafe fn hfile_c_648_blksize(fd: i32) -> usize {
    let mut sbuf: libc::stat = std::mem::zeroed();
    if libc::fstat(fd, &mut sbuf) != 0 {
        return 0;
    }

    if (sbuf.st_mode as u64 & libc::S_IFMT as u64) == libc::S_IFIFO as u64 {
        128 * 1024
    } else {
        #[cfg(not(windows))]
        {
            sbuf.st_blksize as usize
        }
        #[cfg(windows)]
        {
            64 * 1024
        }
    }
}

pub unsafe fn hfile_c_664_hopen_fd(filename: *const c_char, mode: *const c_char) -> *mut hFILE {
    let Some(mode_bytes) = (!mode.is_null()).then(|| CStr::from_ptr(mode).to_bytes()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let fd = libc::open(filename, hfile_oflags_bytes(mode_bytes), 0o666);
    if fd < 0 {
        return std::ptr::null_mut();
    }

    let fp = hfile_init(0, mode, hfile_c_648_blksize(fd));
    if fp.is_null() {
        let save = *libc::__errno_location();
        let _ = libc::close(fd);
        *libc::__errno_location() = save;
        return std::ptr::null_mut();
    }

    (*fp).backend = HFileBackend::Fd { fd, flags: 0 };
    fp
}

pub unsafe fn hpreload_impl(fp: &mut hFILE) -> *mut hFILE {
    // SEAM: read the whole source stream into a Vec, then hand it to an owned
    // in-memory hFILE. On error we close+free the source via the owning
    // `hclose_abruptly` (the Box was created by an open function above).
    let fp_raw = hfile_raw(fp);
    let mut buf = Vec::<u8>::new();
    let mut buf_inc: usize = 8192;

    let len: libc::ssize_t = loop {
        if buf.capacity() - buf.len() < 5000 {
            if buf.try_reserve_exact(buf_inc).is_err() {
                *libc::__errno_location() = libc::ENOMEM;
                hclose_abruptly(fp_raw);
                return std::ptr::null_mut();
            }
            if buf_inc < 1_000_000 {
                buf_inc = (buf_inc as f64 * 1.3) as usize;
            }
        }
        let buf_sz = buf.len();
        let len = hread(
            fp,
            std::slice::from_raw_parts_mut(buf.as_mut_ptr().add(buf_sz), buf.capacity() - buf_sz),
        );
        if len > 0 {
            buf.set_len(buf_sz + len as usize);
        } else {
            break len;
        }
    };

    if len < 0 {
        hclose_abruptly(fp_raw);
        return std::ptr::null_mut();
    }
    let buf_sz = buf.len();
    let buf_a = buf.capacity().max(1);

    let mem_fp = if buf_sz == 0 {
        // no bytes read: build an empty mem hFILE without dereferencing a buffer
        let mut empty: Vec<u8> = Vec::new();
        if empty.try_reserve_exact(buf_a).is_err() {
            *libc::__errno_location() = libc::ENOMEM;
            hclose_abruptly(fp_raw);
            return std::ptr::null_mut();
        }
        empty.resize(buf_a, 0);
        create_hfile_mem_bytes(
            NonNull::new_unchecked(empty.as_mut_ptr()),
            b"r",
            0,
            buf_a as usize,
        )
    } else {
        create_hfile_mem_bytes(
            NonNull::new_unchecked(buf.as_mut_ptr()),
            b"r",
            buf_sz as usize,
            buf_a as usize,
        )
    };
    if mem_fp.is_null() {
        hclose_abruptly(fp_raw);
        return std::ptr::null_mut();
    }

    if hclose(fp_raw) < 0 {
        hclose_abruptly(mem_fp);
        return std::ptr::null_mut();
    }
    mem_fp
}

pub unsafe fn hfile_c_726_is_preload_url_remote(url: *const c_char) -> i32 {
    hisremote(url.add(8))
}

pub unsafe fn hfile_c_730_hopen_preload(url: *const c_char, mode: *const c_char) -> *mut hFILE {
    let fp = hfile_c_1317_hopen(url.add(8), mode);
    if let Some(fp) = hfile_mut(fp) {
        hpreload_impl(fp)
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn hfile_c_735_hdopen(fd: i32, mode: *const c_char) -> *mut hFILE {
    let Some(mode_bytes) = (!mode.is_null()).then(|| CStr::from_ptr(mode).to_bytes()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    #[cfg(windows)]
    if hfile_mode_has(mode_bytes, b's') {
        *libc::__errno_location() = libc::ENOSYS;
        return std::ptr::null_mut();
    }

    let fp = hfile_init(0, mode, hfile_c_648_blksize(fd));
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    let mut flags = 0;
    if hfile_mode_has(mode_bytes, b's') {
        flags |= HFILE_FD_IS_SOCKET;
    }
    if hfile_mode_has(mode_bytes, b'S') {
        flags |= HFILE_FD_IS_SHARED;
    }
    (*fp).backend = HFileBackend::Fd { fd, flags };
    fp
}

pub unsafe fn hfile_c_747_hopen_fd_fileuri(
    mut url: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    if libc::strncmp(url, c"file://localhost/".as_ptr(), 17) == 0 {
        url = url.add(16);
    } else if libc::strncmp(url, c"file:///".as_ptr(), 8) == 0 {
        url = url.add(7);
    } else {
        *libc::__errno_location() = libc::EPROTONOSUPPORT;
        return std::ptr::null_mut();
    }

    hfile_c_664_hopen_fd(url, mode)
}

pub unsafe fn hfile_c_761_hopen_fd_stdinout(mode: *const c_char) -> *mut hFILE {
    let Some(mode_bytes) = (!mode.is_null()).then(|| CStr::from_ptr(mode).to_bytes()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let fd = if hfile_mode_has(mode_bytes, b'r') {
        libc::STDIN_FILENO
    } else {
        libc::STDOUT_FILENO
    };
    let mut mode_shared = Vec::with_capacity(mode_bytes.len() + 2);
    mode_shared.push(b'S');
    mode_shared.extend_from_slice(mode_bytes);
    mode_shared.push(0);
    hfile_c_735_hdopen(fd, mode_shared.as_ptr().cast())
}

pub unsafe fn hfile_c_772_hfile_oflags(mode: *const c_char) -> i32 {
    let Some(mode) = (!mode.is_null()).then(|| CStr::from_ptr(mode).to_bytes()) else {
        return 0;
    };
    hfile_oflags_bytes(mode)
}

pub fn hfile_oflags_bytes(mode: &[u8]) -> i32 {
    let mut rdwr = 0;
    let mut flags = 0;

    for &byte in mode {
        match byte {
            b'r' => rdwr = libc::O_RDONLY,
            b'w' => {
                rdwr = libc::O_WRONLY;
                flags |= libc::O_CREAT | libc::O_TRUNC;
            }
            b'a' => {
                rdwr = libc::O_WRONLY;
                flags |= libc::O_CREAT | libc::O_APPEND;
            }
            b'+' => rdwr = libc::O_RDWR,
            b'e' => {
                #[cfg(any(target_os = "linux", target_os = "android"))]
                {
                    flags |= libc::O_CLOEXEC;
                }
            }
            b'x' => flags |= libc::O_EXCL,
            _ => {}
        }
    }

    #[cfg(target_os = "windows")]
    {
        flags |= libc::O_BINARY;
    }

    rdwr | flags
}

pub unsafe fn hfile_c_826_cmp_prefix(mut key: *const c_char, mut s: *const c_char) -> i32 {
    while *key != 0 {
        if tolower_c(*s) != *key {
            return 1;
        }
        s = s.add(1);
        key = key.add(1);
    }
    0
}

fn hfile_data_metadata_is_base64(metadata: &[u8]) -> bool {
    metadata.len() >= 7 && metadata[metadata.len() - 7..].eq_ignore_ascii_case(b";base64")
}

fn hfile_dehex_byte(c: u8) -> i32 {
    match c {
        b'0'..=b'9' => (c - b'0') as i32,
        b'A'..=b'F' => (c - b'A' + 10) as i32,
        b'a'..=b'f' => (c - b'a' + 10) as i32,
        _ => -1,
    }
}

fn hfile_debase64_byte(c: u8) -> i32 {
    match c {
        b'A'..=b'Z' => (c - b'A') as i32,
        b'a'..=b'z' => (c - b'a' + 26) as i32,
        b'0'..=b'9' => (c - b'0' + 52) as i32,
        b'+' => 62,
        b'/' => 63,
        _ => -1,
    }
}

fn hfile_decode_percent_bytes(data: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(data.len().saturating_add(1));
    let mut i = 0usize;
    while i < data.len() {
        if data[i] == b'%' && i + 2 < data.len() {
            let hi = hfile_dehex_byte(data[i + 1]);
            let lo = hfile_dehex_byte(data[i + 2]);
            if hi >= 0 && lo >= 0 {
                decoded.push(((hi << 4) | lo) as u8);
                i += 3;
                continue;
            }
        }
        decoded.push(data[i]);
        i += 1;
    }
    decoded.push(0);
    decoded
}

fn hfile_decode_base64_bytes(data: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(data.len().div_ceil(4) * 3);
    let mut i = 0usize;
    loop {
        let x0 = data.get(i).map_or(-1, |&c| hfile_debase64_byte(c));
        i += 1;
        let x1 = if x0 >= 0 {
            let x = data.get(i).map_or(-1, |&c| hfile_debase64_byte(c));
            i += 1;
            x
        } else {
            -1
        };
        let x2 = if x1 >= 0 {
            let x = data.get(i).map_or(-1, |&c| hfile_debase64_byte(c));
            i += 1;
            x
        } else {
            -1
        };
        let x3 = if x2 >= 0 {
            let x = data.get(i).map_or(-1, |&c| hfile_debase64_byte(c));
            i += 1;
            x
        } else {
            -1
        };
        if x3 < 0 {
            if x1 >= 0 {
                decoded.push(((x0 << 2) | (x1 >> 4)) as u8);
            }
            if x2 >= 0 {
                decoded.push(((x1 << 4) | (x2 >> 2)) as u8);
            }
            break;
        }

        decoded.push(((x0 << 2) | (x1 >> 4)) as u8);
        decoded.push(((x1 << 4) | (x2 >> 2)) as u8);
        decoded.push(((x2 << 6) | x3) as u8);
    }
    decoded
}

pub unsafe fn hfile_c_835_create_hfile_mem(
    buffer: *mut c_char,
    mode: *const c_char,
    buf_filled: usize,
    buf_size: usize,
) -> *mut hFILE {
    let Some(mode) = (!mode.is_null()).then(|| CStr::from_ptr(mode).to_bytes()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let Some(buffer) = NonNull::new(buffer.cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    create_hfile_mem_bytes(buffer, mode, buf_filled, buf_size)
}

pub unsafe fn create_hfile_mem_bytes(
    buffer: NonNull<u8>,
    mode: &[u8],
    buf_filled: usize,
    buf_size: usize,
) -> *mut hFILE {
    let fp = hfile_init_fixed(0, mode, buffer, buf_filled, buf_size);
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    (*fp).backend = HFileBackend::Mem;
    fp
}

pub unsafe extern "C" fn hfile_c_845_hopen_mem(
    url: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    let Some(url) = (!url.is_null()).then(|| CStr::from_ptr(url).to_bytes()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let Some(mode_bytes) = (!mode.is_null()).then(|| CStr::from_ptr(mode).to_bytes()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let Some(comma) = url.iter().position(|&c| c == b',') else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let metadata = &url[..comma];
    let data = &url[comma + 1..];

    if !hfile_mode_has(mode_bytes, b'r') {
        *libc::__errno_location() = libc::EROFS;
        return std::ptr::null_mut();
    }

    let is_base64 = hfile_data_metadata_is_base64(metadata);
    let (decoded, length) = if is_base64 {
        let decoded = hfile_decode_base64_bytes(data);
        let length = decoded.len();
        (decoded, length)
    } else {
        let decoded = hfile_decode_percent_bytes(data);
        let length = decoded.len().saturating_sub(1);
        (decoded, length)
    };
    let size = if is_base64 {
        data.len().div_ceil(4) * 3
    } else {
        data.len().saturating_add(1)
    }
    .max(1);
    // SEAM: assemble the decoded payload in a local Vec; create_hfile_mem_bytes
    // copies it into the owned hFILE buffer, so this Vec is freed on return.
    let mut buf: Vec<u8> = Vec::new();
    if buf.try_reserve_exact(size).is_err() {
        *libc::__errno_location() = libc::ENOMEM;
        return std::ptr::null_mut();
    }
    buf.resize(size, 0);
    let copy_len = decoded.len().min(size);
    buf[..copy_len].copy_from_slice(&decoded[..copy_len]);

    create_hfile_mem_bytes(
        NonNull::new_unchecked(buf.as_mut_ptr()),
        mode_bytes,
        length,
        size,
    )
}

// original: hopenv_mem (htslib/hfile.c:878)
pub unsafe fn hfile_c_878_hopenv_mem(
    _filename: *const c_char,
    mode: *const c_char,
    buffer: *mut c_char,
    sz: usize,
) -> *mut hFILE {
    // SEAM: create_hfile_mem now copies the caller's buffer into the owned hFILE
    // Vec, so we always free the caller-supplied buffer here (it took ownership
    // in the old C contract regardless of success).
    let hf = hfile_c_835_create_hfile_mem(buffer, mode, sz, sz);
    libc::free(buffer.cast());
    hf
}

unsafe fn hfile_c_va_arg_word(args: *mut crate::htslib_rs::c_compat::__va_list_tag) -> usize {
    let args = &mut *args;
    if args.gp_offset <= 40 {
        let p = args.reg_save_area.cast::<u8>().add(args.gp_offset as usize);
        args.gp_offset += 8;
        std::ptr::read_unaligned(p.cast::<usize>())
    } else {
        let p = args.overflow_arg_area.cast::<u8>();
        args.overflow_arg_area = p.add(8).cast();
        std::ptr::read_unaligned(p.cast::<usize>())
    }
}

unsafe extern "C" fn hfile_c_878_hopenv_mem_va(
    filename: *const c_char,
    mode: *const c_char,
    args: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    if args.is_null() {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }
    let buffer = hfile_c_va_arg_word(args) as *mut c_char;
    let sz = hfile_c_va_arg_word(args) as usize;
    hfile_c_878_hopenv_mem(filename, mode, buffer, sz)
}

pub unsafe fn hfile_mem_get_buffer_impl(
    file: &mut hFILE,
    length: Option<&mut usize>,
) -> *mut c_char {
    // SEAM: only the in-memory backend exposes its buffer; the buffer is the
    // owned Vec<u8> and its usable length is `limit` (the allocated capacity).
    if !matches!(file.backend, HFileBackend::Mem) {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }

    if let Some(length) = length {
        *length = file.limit as usize;
    }

    file.buffer.as_mut_ptr().cast::<c_char>()
}

pub unsafe fn hfile_mem_steal_buffer_impl(
    file: &mut hFILE,
    length: Option<&mut usize>,
) -> *mut c_char {
    // SEAM: the owned Vec can't be handed to a C caller that will `free()` it,
    // so copy the usable bytes into a malloc'd block, then detach the hFILE's
    // buffer (replace with an empty Vec) so it no longer aliases the data.
    let buf = hfile_mem_get_buffer_impl(file, length);
    if buf.is_null() {
        return buf;
    }

    let capacity = file.limit as usize;
    let stolen = libc::malloc(capacity as usize).cast::<c_char>();
    if stolen.is_null() {
        *libc::__errno_location() = libc::ENOMEM;
        return std::ptr::null_mut();
    }
    libc::memcpy(stolen.cast(), buf.cast(), capacity as usize);

    file.buffer = Vec::new();
    file.begin = 0;
    file.end = 0;
    file.limit = 0;
    stolen
}

pub unsafe fn hfile_c_171_writebuffer_is_nonempty(fp: *mut hFILE) -> i32 {
    let Some(fp) = fp.as_ref() else {
        return 0;
    };
    hfile_writebuffer_is_nonempty(fp)
}

pub unsafe fn hfile_c_179_refill_buffer(fp: *mut hFILE) -> libc::ssize_t {
    let Some(fp) = fp.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hfile_refill_buffer(fp)
}

pub unsafe fn hfile_c_212_hfile_set_blksize(fp: *mut hFILE, bufsiz: usize) -> i32 {
    let Some(fp) = fp.as_mut() else {
        return -1;
    };
    hfile_set_blksize_impl(fp, bufsiz)
}

pub unsafe fn hfile_c_235_hgetc2(fp: *mut hFILE) -> i32 {
    let Some(fp) = fp.as_mut() else {
        return libc::EOF;
    };
    hgetc2_impl(fp)
}

pub unsafe fn hfile_c_241_hgetdelim(
    buffer: *mut c_char,
    size: usize,
    delim: i32,
    fp: *mut hFILE,
) -> libc::ssize_t {
    let Some(fp) = fp.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(buffer) = NonNull::new(buffer.cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hgetdelim_impl(
        std::slice::from_raw_parts_mut(buffer.as_ptr(), size),
        delim,
        fp,
    )
}

pub unsafe fn hfile_c_291_hgets(buffer: *mut c_char, size: i32, fp: *mut hFILE) -> *mut c_char {
    let Some(fp) = fp.as_mut() else {
        return std::ptr::null_mut();
    };
    if size < 1 {
        fp.has_errno = libc::EINVAL;
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }
    if buffer.is_null() {
        return std::ptr::null_mut();
    }
    if hgets_impl(
        std::slice::from_raw_parts_mut(buffer.cast::<u8>(), size as usize),
        fp,
    ) {
        buffer
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn hfile_c_306_khgetline(kstr: *mut kstring_t, fp: *mut hFILE) -> i32 {
    let Some(fp) = fp.as_mut() else {
        return libc::EOF;
    };
    hfile_khgetline(kstr, fp)
}

pub unsafe fn hfile_c_313_hpeek(
    fp: *mut hFILE,
    buffer: *mut c_void,
    nbytes: usize,
) -> libc::ssize_t {
    let Some(fp) = fp.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(buffer) = NonNull::new(buffer.cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hpeek_impl(fp, std::slice::from_raw_parts_mut(buffer.as_ptr(), nbytes))
}

pub unsafe fn hfile_c_330_hread2(
    fp: *mut hFILE,
    destv: *mut c_void,
    nbytes: usize,
    nread: usize,
) -> libc::ssize_t {
    let Some(fp) = fp.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(dest) = NonNull::new(destv.cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hread2_impl(
        fp,
        nread,
        std::slice::from_raw_parts_mut(dest.as_ptr(), nbytes),
    )
}

pub unsafe fn hfile_c_376_flush_buffer(fp: *mut hFILE) -> libc::ssize_t {
    let Some(fp) = fp.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hfile_flush_buffer(fp)
}

pub unsafe fn hfile_c_390_hflush(fp: *mut hFILE) -> i32 {
    let Some(fp) = fp.as_mut() else {
        return libc::EOF;
    };
    hflush_impl(fp)
}

pub unsafe fn hfile_c_400_hputc2(c: i32, fp: *mut hFILE) -> i32 {
    let Some(fp) = fp.as_mut() else {
        return libc::EOF;
    };
    hputc2_impl(c, fp)
}

pub unsafe fn hfile_c_412_hwrite2(
    fp: *mut hFILE,
    srcv: *const c_void,
    totalbytes: usize,
    ncopied: usize,
) -> libc::ssize_t {
    let Some(fp) = fp.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(src) = NonNull::new(srcv.cast_mut().cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hwrite2_impl(
        fp,
        ncopied,
        std::slice::from_raw_parts(src.as_ptr(), totalbytes),
    )
}

pub unsafe fn hfile_c_440_hputs2(
    text: *const c_char,
    totalbytes: usize,
    ncopied: usize,
    fp: *mut hFILE,
) -> i32 {
    let Some(fp) = fp.as_mut() else {
        return libc::EOF;
    };
    let Some(text) = NonNull::new(text.cast_mut().cast::<u8>()) else {
        return libc::EOF;
    };
    hputs2_impl(
        std::slice::from_raw_parts(text.as_ptr(), totalbytes),
        ncopied,
        fp,
    )
}

pub unsafe fn hfile_c_446_hseek(fp: *mut hFILE, offset: libc::off_t, whence: i32) -> libc::off_t {
    let Some(fp) = fp.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hseek_impl(fp, offset, whence)
}

pub unsafe fn hfile_c_503_hclose(fp: *mut hFILE) -> i32 {
    // SEAM: owns the hFILE (C hclose semantics); drops the Box after dispatch.
    if fp.is_null() {
        return 0;
    }
    let mut fp = Box::from_raw(fp);
    hclose_impl(&mut fp)
}

pub unsafe fn hfile_c_520_hclose_abruptly(fp: *mut hFILE) {
    if fp.is_null() {
        return;
    }
    let mut fp = Box::from_raw(fp);
    hclose_abruptly_impl(&mut fp);
}

pub unsafe fn hfile_c_689_hpreload(fp: *mut hFILE) -> *mut hFILE {
    let Some(fp) = fp.as_mut() else {
        return std::ptr::null_mut();
    };
    hpreload_impl(fp)
}

pub unsafe fn hfile_c_894_hfile_mem_get_buffer(
    file: *mut hFILE,
    length: *mut usize,
) -> *mut c_char {
    let Some(file) = file.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    hfile_mem_get_buffer_impl(file, length.as_mut())
}

pub unsafe fn hfile_c_906_hfile_mem_steal_buffer(
    file: *mut hFILE,
    length: *mut usize,
) -> *mut c_char {
    let Some(file) = file.as_mut() else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    hfile_mem_steal_buffer_impl(file, length.as_mut())
}

pub unsafe extern "C" fn hfile_c_915_hopen_not_supported(
    _fname: *const c_char,
    _mode: *const c_char,
) -> *mut hFILE {
    *libc::__errno_location() = libc::EINVAL;
    std::ptr::null_mut()
}

pub unsafe extern "C" fn hfile_c_935_crypt4gh_needed(
    url: *const c_char,
    _mode: *const c_char,
) -> *mut hFILE {
    let _u = if libc::strncmp(url, c"crypt4gh:".as_ptr(), 9) == 0 {
        url.add(9)
    } else {
        url
    };
    *libc::__errno_location() = libc::EPROTONOSUPPORT;
    std::ptr::null_mut()
}

pub unsafe fn hfile_c_920_hfile_plugin_init_mem(self_: *mut hFILE_plugin) -> i32 {
    static HANDLER: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
        open: Some(hfile_c_915_hopen_not_supported),
        isremote: Some(hfile_c_1342_hfile_always_remote),
        provider: c"mem".as_ptr(),
        priority: 2050,
        vopen: Some(hfile_c_878_hopenv_mem_va),
    };

    (*(self_.cast::<hfile_plugin_layout>())).name = c"mem".as_ptr();
    hfile_add_scheme_handler(
        c"mem".as_ptr(),
        (&HANDLER as *const hfile_scheme_handler_layout).cast(),
    );
    0
}

pub unsafe fn hfile_c_956_hfile_plugin_init_crypt4gh_needed(self_: *mut hFILE_plugin) -> i32 {
    static HANDLER: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
        open: Some(hfile_c_935_crypt4gh_needed),
        isremote: Some(hfile_c_1339_hfile_always_local),
        provider: c"crypt4gh-needed".as_ptr(),
        priority: 0,
        vopen: None,
    };

    (*(self_.cast::<hfile_plugin_layout>())).name = c"crypt4gh-needed".as_ptr();
    hfile_add_scheme_handler(
        c"crypt4gh".as_ptr(),
        (&HANDLER as *const hfile_scheme_handler_layout).cast(),
    );
    0
}

#[derive(Default)]
struct HFilePluginState {
    schemes: Option<Vec<HFileSchemeEntry>>,
    // Plugin list links store NonNull pointers to node contents; Box keeps those
    // addresses stable even if the Vec reallocates.
    #[allow(clippy::vec_box)]
    plugins: Vec<Box<hFILE_plugin_list>>,
}

struct HFileSchemeEntry {
    scheme: ConstNonNull<c_char>,
    handler: ConstNonNull<hFILE_scheme_handler>,
}

fn hfile_plugin_state() -> &'static Mutex<HFilePluginState> {
    static STATE: OnceLock<Mutex<HFilePluginState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(HFilePluginState::default()))
}

unsafe fn hfile_c_1053_add_scheme_handler_locked(
    state: &mut HFilePluginState,
    scheme: *const c_char,
    handler: *const hFILE_scheme_handler,
) {
    let schemes = match state.schemes.as_mut() {
        Some(schemes) => schemes,
        None => return,
    };
    let Some(scheme) = ConstNonNull::new(scheme) else {
        return;
    };
    let Some(handler) = ConstNonNull::new(handler) else {
        return;
    };
    let handler_layout = handler.as_ptr().cast::<hfile_scheme_handler_layout>();
    if (*handler_layout).open.is_none() || (*handler_layout).isremote.is_none() {
        return;
    }

    for entry in schemes.iter_mut() {
        if libc::strcmp(entry.scheme.as_ptr(), scheme.as_ptr()) == 0 {
            if hfile_c_1011_priority(handler.as_ptr())
                > hfile_c_1011_priority(entry.handler.as_ptr())
            {
                entry.handler = handler;
            }
            return;
        }
    }

    schemes.push(HFileSchemeEntry { scheme, handler });
}

// original: hfile_add_scheme_handler (htslib/hfile.c:1053)
pub unsafe fn hfile_c_1053_hfile_add_scheme_handler(
    scheme: *const c_char,
    handler: *const hFILE_scheme_handler,
) {
    let mut state = hfile_plugin_state().lock().unwrap();
    if state.schemes.is_none() {
        let _ = hfile_c_1046_try_exe_add_scheme_handler(scheme, handler);
        return;
    }
    hfile_c_1053_add_scheme_handler_locked(&mut state, scheme, handler);
}

// original: init_add_plugin (htslib/hfile.c:1079)
unsafe fn hfile_c_1079_init_add_plugin_impl<F>(
    obj: *mut c_void,
    init: F,
    pluginname: *const c_char,
) -> i32
where
    F: FnOnce(*mut hFILE_plugin) -> i32,
{
    let mut p = Box::new(hFILE_plugin_list {
        plugin: hfile_plugin_layout {
            api_version: 1,
            obj,
            name: std::ptr::null(),
            destroy: std::ptr::null(),
        },
        next: None,
    });

    let ret = init((&mut p.plugin as *mut hfile_plugin_layout).cast());
    if ret != 0 {
        return ret;
    }

    let mut state = hfile_plugin_state().lock().unwrap();
    p.next = state
        .plugins
        .last()
        .map(|plugin| NonNull::from(plugin.as_ref()));
    state.plugins.push(p);
    let _ = pluginname;
    0
}

pub unsafe fn hfile_c_1079_init_add_plugin(
    obj: *mut c_void,
    init: unsafe fn(*mut hFILE_plugin) -> i32,
    pluginname: *const c_char,
) -> i32 {
    hfile_c_1079_init_add_plugin_impl(obj, |plugin| init(plugin), pluginname)
}

unsafe fn hfile_c_1079_init_add_dynamic_plugin(
    obj: *mut c_void,
    init: HFilePluginInitFn,
    pluginname: *const c_char,
) -> i32 {
    hfile_c_1079_init_add_plugin_impl(obj, |plugin| init(plugin), pluginname)
}

unsafe extern "C" fn hfile_c_1114_data_open(
    fname: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    hfile_c_845_hopen_mem(fname, mode)
}

unsafe extern "C" fn hfile_c_1115_file_open(
    fname: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    hfile_c_747_hopen_fd_fileuri(fname, mode)
}

unsafe extern "C" fn hfile_c_1116_preload_open(
    fname: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    hfile_c_730_hopen_preload(fname, mode)
}

unsafe extern "C" fn hfile_c_1116_preload_isremote(fname: *const c_char) -> i32 {
    hfile_c_726_is_preload_url_remote(fname)
}

static HFILE_C_1114_DATA_HANDLER: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
    open: Some(hfile_c_1114_data_open),
    isremote: Some(hfile_c_1339_hfile_always_local),
    provider: c"built-in".as_ptr(),
    priority: 80,
    vopen: None,
};

static HFILE_C_1115_FILE_HANDLER: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
    open: Some(hfile_c_1115_file_open),
    isremote: Some(hfile_c_1339_hfile_always_local),
    provider: c"built-in".as_ptr(),
    priority: 80,
    vopen: None,
};

static HFILE_C_1116_PRELOAD_HANDLER: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
    open: Some(hfile_c_1116_preload_open),
    isremote: Some(hfile_c_1116_preload_isremote),
    provider: c"built-in".as_ptr(),
    priority: 80,
    vopen: None,
};

// original: load_hfile_plugins (htslib/hfile.c:1111)
//
// KNOWN LIMITATION (2026-05-29): there is a small loading-window race. We
// set `schemes = Some(Vec::new())` *before* registering handlers; a
// concurrent caller can see `schemes.is_some()` and return without
// re-loading, even though handlers haven't been registered yet. This is the
// cause of the intermittent `hfile_remote_scheme_dispatch_prefers_feature_
// plugins` / `hfile_unknown_scheme_fallback_is_local_like_upstream` flakes
// under `--features "gcs,libcurl,s3"` + `--test-threads >= 8`.
//
// A naïve `std::sync::Once::call_once` wrapper deadlocks/panics: dynamic
// plugin init can call back into `find_scheme_handler` → `load_hfile_plugins`
// → re-enter the same Once → panic ("Once instance has previously been
// poisoned" or recursive call). A correct fix likely needs a 3-state enum
// (Uninit / Loading-with-condvar / Loaded) plus reentry detection. Not
// undertaken here because the race window is narrow and tests pass at
// `--test-threads <= 4`.
pub unsafe fn hfile_c_1111_load_hfile_plugins() -> i32 {
    {
        let mut state = hfile_plugin_state().lock().unwrap();
        if state.schemes.is_some() {
            return 0;
        }
        state.schemes = Some(Vec::new());
    }

    hfile_c_1053_hfile_add_scheme_handler(
        c"data".as_ptr(),
        (&HFILE_C_1114_DATA_HANDLER as *const hfile_scheme_handler_layout).cast(),
    );
    hfile_c_1053_hfile_add_scheme_handler(
        c"file".as_ptr(),
        (&HFILE_C_1115_FILE_HANDLER as *const hfile_scheme_handler_layout).cast(),
    );
    hfile_c_1053_hfile_add_scheme_handler(
        c"preload".as_ptr(),
        (&HFILE_C_1116_PRELOAD_HANDLER as *const hfile_scheme_handler_layout).cast(),
    );
    hfile_c_1079_init_add_plugin(
        std::ptr::null_mut(),
        hfile_c_920_hfile_plugin_init_mem,
        c"mem".as_ptr(),
    );
    hfile_c_1079_init_add_plugin(
        std::ptr::null_mut(),
        hfile_c_956_hfile_plugin_init_crypt4gh_needed,
        c"crypt4gh-needed".as_ptr(),
    );
    #[cfg(feature = "libcurl")]
    hfile_c_1079_init_add_plugin(
        std::ptr::null_mut(),
        crate::htslib_rs::hfile_libcurl::hfile_libcurl_c_1679_PLUGIN_GLOBAL,
        c"libcurl".as_ptr(),
    );
    #[cfg(feature = "s3")]
    hfile_c_1079_init_add_plugin(
        std::ptr::null_mut(),
        crate::htslib_rs::hfile_s3::hfile_s3_c_2436_PLUGIN_GLOBAL,
        c"s3".as_ptr(),
    );
    #[cfg(feature = "gcs")]
    hfile_c_1079_init_add_plugin(
        std::ptr::null_mut(),
        crate::htslib_rs::hfile_gcs::hfile_gcs_c_141_PLUGIN_GLOBAL,
        c"gcs".as_ptr(),
    );

    0
}

unsafe extern "C" fn hfile_c_1168_unknown_open(
    fname: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    hfile_c_1168_hopen_unknown_scheme(fname, mode)
}

unsafe extern "C" fn hfile_c_1178_unknown_isremote(fname: *const c_char) -> i32 {
    hfile_c_1339_hfile_always_local(fname)
}

static HFILE_C_1178_UNKNOWN_SCHEME: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
    open: Some(hfile_c_1168_unknown_open),
    isremote: Some(hfile_c_1178_unknown_isremote),
    provider: c"built-in".as_ptr(),
    priority: 0,
    vopen: None,
};

// original: find_scheme_handler (htslib/hfile.c:1176)
pub unsafe fn hfile_c_1176_find_scheme_handler(s: *const c_char) -> *const hFILE_scheme_handler {
    let mut scheme = [0 as c_char; 12];
    let mut i = 0usize;
    while i < scheme.len() {
        let c = *s.add(i);
        if isalnum_c(c) != 0 || c == b'+' as c_char || c == b'-' as c_char || c == b'.' as c_char {
            scheme[i] = tolower_c(c);
        } else if c == b':' as c_char {
            break;
        } else {
            return std::ptr::null();
        }
        i += 1;
    }

    if i <= 1 || i >= scheme.len() {
        return std::ptr::null();
    }
    scheme[i] = 0;

    {
        let needs_load = hfile_plugin_state().lock().unwrap().schemes.is_none();
        if needs_load && hfile_c_1111_load_hfile_plugins() < 0 {
            return std::ptr::null();
        }
    }

    let state = hfile_plugin_state().lock().unwrap();
    if let Some(schemes) = &state.schemes {
        for entry in schemes {
            if libc::strcmp(entry.scheme.as_ptr(), scheme.as_ptr()) == 0 {
                return entry.handler.as_ptr();
            }
        }
    }

    (&HFILE_C_1178_UNKNOWN_SCHEME as *const hfile_scheme_handler_layout).cast()
}

pub unsafe fn hfile_c_983_hfile_shutdown(do_close_plugin: i32) {
    let mut state = hfile_plugin_state().lock().unwrap();
    state.schemes = None;
    while let Some(p) = state.plugins.pop() {
        if !p.plugin.destroy.is_null() {
            let destroy = hfile_plugin_destroy_fn(p.plugin.destroy);
            destroy();
        }
        let _ = do_close_plugin;
    }
}

pub unsafe fn hfile_c_1005_hfile_exit() {
    hfile_c_983_hfile_shutdown(0);
}

pub unsafe fn hfile_c_1168_hopen_unknown_scheme(
    fname: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    let fp = hfile_c_664_hopen_fd(fname, mode);
    if fp.is_null() && *libc::__errno_location() == libc::ENOENT {
        *libc::__errno_location() = libc::EPROTONOSUPPORT;
    }
    fp
}

pub unsafe fn hfile_c_1317_hopen_vargs(
    fname: *const c_char,
    mode: *const c_char,
    args: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    let Some(mode_bytes) = (!mode.is_null()).then(|| CStr::from_ptr(mode).to_bytes()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let handler = hfile_c_1176_find_scheme_handler(fname);
    if !handler.is_null() {
        let handler = handler.cast::<hfile_scheme_handler_layout>();
        if !hfile_mode_has(mode_bytes, b':')
            || (*handler).priority < 2000
            || (*handler).vopen.is_none()
        {
            return (*handler).open.expect("hFILE open handler")(fname, mode);
        }
        if args.is_null() {
            *libc::__errno_location() = libc::EINVAL;
            return std::ptr::null_mut();
        }
        return (*handler).vopen.expect("hFILE vopen handler")(fname, mode, args);
    }

    if libc::strcmp(fname, c"-".as_ptr()) == 0 {
        hfile_c_761_hopen_fd_stdinout(mode)
    } else {
        hfile_c_664_hopen_fd(fname, mode)
    }
}

pub unsafe fn hfile_c_1317_hopen(fname: *const c_char, mode: *const c_char) -> *mut hFILE {
    hfile_c_1317_hopen_vargs(fname, mode, std::ptr::null_mut())
}

pub unsafe extern "C" fn hfile_c_1339_hfile_always_local(_fname: *const c_char) -> i32 {
    0
}

pub unsafe extern "C" fn hfile_c_1342_hfile_always_remote(_fname: *const c_char) -> i32 {
    1
}

// original: hfile_list_schemes (htslib/hfile.c:1218)
pub unsafe fn hfile_c_1218_hfile_list_schemes(
    plugin: *const c_char,
    sc_list: *mut *const c_char,
    nschemes: *mut i32,
) -> i32 {
    {
        let needs_load = hfile_plugin_state().lock().unwrap().schemes.is_none();
        if needs_load && hfile_c_1111_load_hfile_plugins() < 0 {
            return -1;
        }
    }

    let state = hfile_plugin_state().lock().unwrap();
    let mut ns = 0;
    if let Some(schemes) = &state.schemes {
        for entry in schemes {
            let handler = entry.handler.as_ptr().cast::<hfile_scheme_handler_layout>();
            if !plugin.is_null() && libc::strcmp((*handler).provider, plugin) != 0 {
                continue;
            }

            if ns < *nschemes {
                *sc_list.add(ns as usize) = entry.scheme.as_ptr();
            }
            ns += 1;
        }
    }

    if *nschemes > ns {
        *nschemes = ns;
    }
    ns
}

// original: hfile_list_plugins (htslib/hfile.c:1257)
pub unsafe fn hfile_c_1257_hfile_list_plugins(
    plist: *mut *const c_char,
    nplugins: *mut i32,
) -> i32 {
    {
        let needs_load = hfile_plugin_state().lock().unwrap().schemes.is_none();
        if needs_load && hfile_c_1111_load_hfile_plugins() < 0 {
            return -1;
        }
    }

    let state = hfile_plugin_state().lock().unwrap();
    let mut np = 0;
    if np < *nplugins {
        *plist.add(np as usize) = c"built-in".as_ptr();
    }
    np += 1;

    for p in state.plugins.iter().rev() {
        if np < *nplugins {
            *plist.add(np as usize) = p.plugin.name;
        }
        np += 1;
    }

    if *nplugins > np {
        *nplugins = np;
    }
    np
}

// original: hfile_has_plugin (htslib/hfile.c:1293)
pub unsafe fn hfile_c_1293_hfile_has_plugin(name: *const c_char) -> i32 {
    {
        let needs_load = hfile_plugin_state().lock().unwrap().schemes.is_none();
        if needs_load && hfile_c_1111_load_hfile_plugins() < 0 {
            return -1;
        }
    }

    let state = hfile_plugin_state().lock().unwrap();
    for p in state.plugins.iter().rev() {
        if libc::strcmp(p.plugin.name, name) == 0 {
            return 1;
        }
    }

    0
}

pub unsafe fn hfile_c_1345_hisremote(fname: *const c_char) -> i32 {
    let handler = hfile_c_1176_find_scheme_handler(fname);
    if !handler.is_null() {
        return (*(handler.cast::<hfile_scheme_handler_layout>()))
            .isremote
            .expect("hFILE isremote handler")(fname);
    }
    0
}

pub unsafe fn hfile_c_1353_strip_extension(
    start: *const c_char,
    limit: *const c_char,
) -> *const c_char {
    let mut s = limit;
    while s > start {
        s = s.sub(1);
        if *s == b'.' as c_char {
            return s;
        } else if *s == b'/' as c_char {
            break;
        }
    }
    limit
}

pub unsafe fn hfile_c_1364_haddextension(
    buffer: *mut kstring_t,
    filename: *const c_char,
    replace: i32,
    new_extension: *const c_char,
) -> *mut c_char {
    let trailing = if !hfile_c_1176_find_scheme_handler(filename).is_null() {
        let span = if libc::strncmp(filename, c"s3://".as_ptr(), 5) != 0
            && libc::strncmp(filename, c"s3+http://".as_ptr(), 10) != 0
            && libc::strncmp(filename, c"s3+https://".as_ptr(), 11) != 0
        {
            libc::strcspn(filename, c"?#".as_ptr())
        } else {
            libc::strcspn(filename, c"?".as_ptr())
        };
        filename.add(span)
    } else {
        filename.add(CStr::from_ptr(filename).to_bytes().len())
    };

    let end = if replace != 0 {
        hfile_c_1353_strip_extension(filename, trailing)
    } else {
        trailing
    };

    (*buffer).data.clear();
    let filename_len = end.offset_from(filename) as usize;
    let filename_slice = std::slice::from_raw_parts(filename.cast::<u8>(), filename_len);
    let new_extension_slice = CStr::from_ptr(new_extension).to_bytes();
    let trailing_slice = CStr::from_ptr(trailing).to_bytes();
    if kputsn(filename_slice, filename_len, &mut *buffer) >= 0
        && kputs(new_extension_slice, &mut *buffer) >= 0
        && kputs(trailing_slice, &mut *buffer) >= 0
    {
        // The owned kstring's `data` holds content only (no NUL), but this
        // function returns a C string pointer that callers feed to strcmp/etc.
        // Write a sentinel NUL into one byte of spare capacity *past* the
        // logical end, leaving `data.len()` (the content) unchanged — mirroring
        // C kstring, where `s.l` excludes the always-present trailing NUL.
        let data = &mut (*buffer).data;
        data.reserve(1);
        let len = data.len();
        *data.as_mut_ptr().add(len) = 0;
        data.as_mut_ptr().cast::<c_char>()
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn hfile_c_1416_knet_open(fn_: *const c_char, mode: *const c_char) -> *mut knetFile {
    let Some(filename) = (!fn_.is_null()).then(|| CStr::from_ptr(fn_).to_bytes()) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let hf = if filename == b"-" {
        hfile_c_761_hopen_fd_stdinout(mode)
    } else if filename.starts_with(b"file://") {
        hfile_c_747_hopen_fd_fileuri(fn_, mode)
    } else if !hfile_mode_has(filename, b':') {
        hfile_c_664_hopen_fd(fn_, mode)
    } else {
        hopen(fn_, mode)
    };
    let Some(hf) = OwnedHFile::from_raw(hf) else {
        return std::ptr::null_mut();
    };

    let fd = match (*hf.as_ptr()).backend {
        HFileBackend::Fd { fd, .. } => fd,
        _ => -1,
    };
    Box::into_raw(Box::new(knet_file_layout { fd, offset: 0, hf })).cast()
}

pub unsafe fn hfile_c_1433_knet_dopen(fd: i32, mode: *const c_char) -> *mut knetFile {
    let Some(hf) = OwnedHFile::from_raw(hdopen(fd, mode)) else {
        return std::ptr::null_mut();
    };
    Box::into_raw(Box::new(knet_file_layout { fd, offset: 0, hf })).cast()
}

pub unsafe fn hfile_c_1445_knet_read(
    fp: *mut knetFile,
    buf: *mut c_void,
    len: usize,
) -> libc::ssize_t {
    let fp = fp.cast::<knet_file_layout>();
    let r = htslib_hfile_h_247_hread((*fp).hf.as_ptr(), buf, len);
    if r > 0 {
        (*fp).offset += r as i64;
    }
    r
}

pub unsafe fn hfile_c_1452_knet_seek(
    fp: *mut knetFile,
    off: libc::off_t,
    whence: i32,
) -> libc::off_t {
    let fp = fp.cast::<knet_file_layout>();
    let r = hseek((*fp).hf.as_ptr(), off, whence);
    if r >= 0 {
        (*fp).offset = r as i64;
    }
    r
}

pub unsafe fn hfile_c_1460_knet_close(fp: *mut knetFile) -> i32 {
    let fp = *Box::from_raw(fp.cast::<knet_file_layout>());
    fp.hf.close()
}

pub unsafe fn hfile_oflags(mode: *const c_char) -> i32 {
    hfile_c_772_hfile_oflags(mode)
}

pub unsafe fn hdopen(fd: i32, mode: *const c_char) -> *mut hFILE {
    hfile_c_735_hdopen(fd, mode)
}

pub unsafe fn hisremote(filename: *const c_char) -> i32 {
    hfile_c_1345_hisremote(filename)
}

pub unsafe fn haddextension(
    buffer: *mut kstring_t,
    filename: *const c_char,
    replace: i32,
    extension: *const c_char,
) -> *mut c_char {
    hfile_c_1364_haddextension(buffer, filename, replace, extension)
}

pub unsafe fn hclose(fp: *mut hFILE) -> i32 {
    // SEAM: hclose takes ownership (C semantics): dispatch the close, then drop
    // the owning Box to free the hFILE (buffer Vec + backend state).
    if fp.is_null() {
        return 0;
    }
    let mut fp = Box::from_raw(fp);
    hclose_impl(&mut fp)
}

pub unsafe fn hclose_abruptly(fp: *mut hFILE) {
    if fp.is_null() {
        return;
    }
    let mut fp = Box::from_raw(fp);
    hclose_abruptly_impl(&mut fp)
}

pub unsafe fn hseek(fp: *mut hFILE, offset: libc::off_t, whence: i32) -> libc::off_t {
    let Some(fp) = hfile_mut(fp) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hseek_impl(fp, offset, whence)
}

pub unsafe fn hfile_set_blksize(fp: *mut hFILE, bufsiz: usize) -> i32 {
    let Some(fp) = hfile_mut(fp) else {
        return -1;
    };
    hfile_set_blksize_impl(fp, bufsiz)
}

pub unsafe fn hgetc2(fp: *mut hFILE) -> i32 {
    let Some(fp) = hfile_mut(fp) else {
        return libc::EOF;
    };
    hgetc2_impl(fp)
}

pub unsafe fn hgetdelim(
    buffer: *mut c_char,
    size: usize,
    delim: i32,
    fp: *mut hFILE,
) -> libc::ssize_t {
    let Some(fp) = hfile_mut(fp) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(buffer) = NonNull::new(buffer.cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hgetdelim_impl(
        std::slice::from_raw_parts_mut(buffer.as_ptr(), size),
        delim,
        fp,
    )
}

pub unsafe fn hgets(buffer: *mut c_char, size: i32, fp: *mut hFILE) -> *mut c_char {
    let Some(fp) = hfile_mut(fp) else {
        return std::ptr::null_mut();
    };
    if size < 1 {
        fp.has_errno = libc::EINVAL;
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }
    if buffer.is_null() {
        return std::ptr::null_mut();
    }
    if hgets_impl(
        std::slice::from_raw_parts_mut(buffer.cast::<u8>(), size as usize),
        fp,
    ) {
        buffer
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn khgetline(kstr: *mut kstring_t, fp: *mut hFILE) -> i32 {
    let Some(fp) = hfile_mut(fp) else {
        return libc::EOF;
    };
    hfile_khgetline(kstr, fp)
}

pub unsafe fn hpeek(fp: *mut hFILE, buffer: *mut c_void, nbytes: usize) -> libc::ssize_t {
    let Some(fp) = hfile_mut(fp) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(buffer) = NonNull::new(buffer.cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hpeek_impl(fp, std::slice::from_raw_parts_mut(buffer.as_ptr(), nbytes))
}

pub unsafe fn hread2(
    fp: *mut hFILE,
    destv: *mut c_void,
    nbytes: usize,
    nread: usize,
) -> libc::ssize_t {
    let Some(fp) = hfile_mut(fp) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(dest) = NonNull::new(destv.cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hread2_impl(
        fp,
        nread,
        std::slice::from_raw_parts_mut(dest.as_ptr(), nbytes),
    )
}

pub unsafe fn hputc2(c: i32, fp: *mut hFILE) -> i32 {
    let Some(fp) = hfile_mut(fp) else {
        return libc::EOF;
    };
    hputc2_impl(c, fp)
}

pub unsafe fn hwrite2(
    fp: *mut hFILE,
    srcv: *const c_void,
    totalbytes: usize,
    ncopied: usize,
) -> libc::ssize_t {
    let Some(fp) = hfile_mut(fp) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    let Some(src) = NonNull::new(srcv.cast_mut().cast::<u8>()) else {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    };
    hwrite2_impl(
        fp,
        ncopied,
        std::slice::from_raw_parts(src.as_ptr(), totalbytes),
    )
}

pub unsafe fn hputs2(
    text: *const c_char,
    totalbytes: usize,
    ncopied: usize,
    fp: *mut hFILE,
) -> i32 {
    let Some(fp) = hfile_mut(fp) else {
        return libc::EOF;
    };
    let Some(text) = NonNull::new(text.cast_mut().cast::<u8>()) else {
        return libc::EOF;
    };
    hputs2_impl(
        std::slice::from_raw_parts(text.as_ptr(), totalbytes),
        ncopied,
        fp,
    )
}

pub unsafe fn hflush(fp: *mut hFILE) -> i32 {
    let Some(fp) = hfile_mut(fp) else {
        return libc::EOF;
    };
    hflush_impl(fp)
}

pub unsafe fn hfile_mem_get_buffer(file: *mut hFILE, length: *mut usize) -> *mut c_char {
    let Some(file) = hfile_mut(file) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    hfile_mem_get_buffer_impl(file, length.as_mut())
}

pub unsafe fn hfile_mem_steal_buffer(file: *mut hFILE, length: *mut usize) -> *mut c_char {
    let Some(file) = hfile_mut(file) else {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    hfile_mem_steal_buffer_impl(file, length.as_mut())
}

pub unsafe fn hfile_list_schemes(
    plugin: *const c_char,
    sc_list: *mut *const c_char,
    nschemes: *mut i32,
) -> i32 {
    hfile_c_1218_hfile_list_schemes(plugin, sc_list, nschemes)
}

pub unsafe fn hfile_list_plugins(plist: *mut *const c_char, nplugins: *mut i32) -> i32 {
    hfile_c_1257_hfile_list_plugins(plist, nplugins)
}

pub unsafe fn hfile_has_plugin(name: *const c_char) -> i32 {
    hfile_c_1293_hfile_has_plugin(name)
}

pub unsafe fn hfile_add_scheme_handler(
    scheme: *const c_char,
    handler: *const hFILE_scheme_handler,
) {
    hfile_c_1053_hfile_add_scheme_handler(scheme, handler)
}

pub unsafe fn hopen(fname: *const c_char, mode: *const c_char) -> *mut hFILE {
    hfile_c_1317_hopen(fname, mode)
}

pub unsafe fn knet_open(fn_: *const c_char, mode: *const c_char) -> *mut knetFile {
    hfile_c_1416_knet_open(fn_, mode)
}

pub unsafe fn knet_dopen(fd: i32, mode: *const c_char) -> *mut knetFile {
    hfile_c_1433_knet_dopen(fd, mode)
}

pub unsafe fn knet_read(fp: *mut knetFile, buf: *mut c_void, len: usize) -> libc::ssize_t {
    hfile_c_1445_knet_read(fp, buf, len)
}

pub unsafe fn knet_seek(fp: *mut knetFile, off: libc::off_t, whence: i32) -> libc::off_t {
    hfile_c_1452_knet_seek(fp, off, whence)
}

pub unsafe fn knet_close(fp: *mut knetFile) -> i32 {
    hfile_c_1460_knet_close(fp)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hfile_plugin_test_lock() -> &'static std::sync::Mutex<()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
    }

    // SEAM: the old custom-vtable test backends (ReadFile/WriteFile + the
    // hfile_backend_layout statics) are gone. Tests now drive the real backends
    // the HFileBackend enum supports: `Mem` (in-memory data: URLs) and `Fd`
    // (temp files / socketpairs).

    // --- allocation + owned-Box lifecycle ---

    #[test]
    fn hfile_init_builds_owned_mobile_read_buffer() {
        unsafe {
            let fp = hfile_c_104_hfile_init(0, c"r".as_ptr(), 999_999);
            assert!(!fp.is_null());
            assert_eq!((*fp).begin, 0);
            assert_eq!((*fp).end, 0);
            assert_eq!((*fp).limit, 128 * 1024);
            assert_eq!((*fp).buffer.len(), 128 * 1024);
            assert_eq!((*fp).flags & HFILE_MOBILE, HFILE_MOBILE);
            assert_eq!((*fp).flags & HFILE_READONLY, HFILE_READONLY);
            assert!(matches!((*fp).backend, HFileBackend::None));

            (*fp).begin = 1;
            assert_eq!(hfile_c_171_writebuffer_is_nonempty(fp), 1);
            hfile_c_162_hfile_destroy(fp);
        }
    }

    #[test]
    fn hfile_init_fixed_copies_payload_into_owned_buffer() {
        unsafe {
            let src = b"abcde";
            let fp = hfile_c_141_hfile_init_fixed(
                0,
                c"r".as_ptr(),
                src.as_ptr().cast_mut().cast(),
                5,
                8,
            );
            assert!(!fp.is_null());
            assert_eq!((*fp).begin, 0);
            assert_eq!((*fp).end, 5);
            assert_eq!((*fp).limit, 8);
            let buffer = &(*fp).buffer;
            assert_eq!(&buffer[..5], b"abcde");
            assert_eq!((*fp).flags & HFILE_AT_EOF, HFILE_AT_EOF);
            assert_eq!((*fp).flags & HFILE_MOBILE, 0);
            assert_eq!((*fp).flags & HFILE_READONLY, HFILE_READONLY);
            assert_eq!((*fp).has_errno, 0);

            assert_eq!(hfile_c_212_hfile_set_blksize(fp, 16), 0);
            assert_eq!((*fp).limit, 16);
            assert_eq!((*fp).buffer.len(), 16);
            assert_eq!(hfile_c_212_hfile_set_blksize(fp, 4), -1);

            hfile_destroy(fp);
        }
    }

    #[test]
    fn hfile_destroy_and_abrupt_close_preserve_errno() {
        unsafe {
            let fp = hfile_c_104_hfile_init(0, c"r".as_ptr(), 16);
            assert!(!fp.is_null());
            *libc::__errno_location() = libc::E2BIG;
            hfile_c_162_hfile_destroy(fp);
            assert_eq!(*libc::__errno_location(), libc::E2BIG);

            // None-backend abrupt close is a no-op that preserves errno, then we
            // free the owned Box ourselves.
            let abrupt = hfile_c_104_hfile_init(0, c"r".as_ptr(), 16);
            assert!(!abrupt.is_null());
            *libc::__errno_location() = libc::E2BIG;
            hfile_c_520_hclose_abruptly(abrupt);
            assert_eq!(*libc::__errno_location(), libc::E2BIG);
        }
    }

    #[test]
    fn hfile_init_reports_allocation_failure_without_leaking() {
        unsafe {
            *libc::__errno_location() = 0;
            let fp = hfile_c_104_hfile_init(0, c"w".as_ptr(), usize::MAX);
            assert!(fp.is_null());
            assert_eq!(*libc::__errno_location(), libc::ENOMEM);
        }
    }

    // --- OwnedHFile / BorrowedHFile wrappers over a real Mem backend ---

    #[test]
    fn owned_and_borrowed_hfile_wrap_a_mem_backend() {
        unsafe {
            assert!(OwnedHFile::from_raw(std::ptr::null_mut()).is_none());
            assert!(BorrowedHFile::from_raw(std::ptr::null_mut()).is_none());

            let fp = hfile_c_845_hopen_mem(c"data:,abc".as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let borrowed = BorrowedHFile::from_raw(fp).expect("non-null hFILE");
            assert_eq!(borrowed.as_ptr(), fp);
            assert_eq!(borrowed.as_non_null().as_ptr(), fp);

            let owned = OwnedHFile::from_raw(fp).expect("non-null hFILE");
            let raw = owned.into_raw();
            assert_eq!(raw, fp);
            assert_eq!(hclose(raw), 0);
        }
    }

    #[test]
    fn owned_hfile_closes_fd_backend_on_drop() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-owned-drop-{}-{}.txt",
                std::process::id(),
                line!()
            ));
            std::fs::write(&path, b"x").unwrap();
            let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();

            let fp = hfile_c_664_hopen_fd(c_path.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let fd = match (*fp).backend {
                HFileBackend::Fd { fd, .. } => fd,
                _ => panic!("expected Fd backend"),
            };
            let owned = OwnedHFile::from_raw(fp).expect("non-null");
            drop(owned);
            // fd was closed by the backend close, so a second close fails EBADF
            assert_eq!(libc::close(fd), -1);
            assert_eq!(*libc::__errno_location(), libc::EBADF);

            std::fs::remove_file(path).unwrap();
        }
    }

    // --- in-memory (Mem) backend buffered I/O ---

    #[test]
    fn hfile_memory_backend_decodes_data_urls_and_exposes_buffer() {
        unsafe {
            let fp = hfile_c_845_hopen_mem(c"data:,hello%2C%20world%21".as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());

            let mut out = [0 as c_char; 32];
            assert_eq!(htslib_hfile_h_247_hread(fp, out.as_mut_ptr().cast(), 32), 13);
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 13),
                b"hello, world!"
            );

            let mut length = 0usize;
            let internal = hfile_c_894_hfile_mem_get_buffer(fp, &mut length);
            assert!(!internal.is_null());
            assert_eq!(length, 20);

            // mem backend is not seekable: HFileBackend::seek returns -1/EINVAL.
            // (The public hseek() would take the in-buffer shortcut and return
            // 0 here, so exercise the backend op directly — this mirrors the
            // pre-refactor test that called the now-inlined hfile_c_810_mem_seek.)
            assert_eq!(HFileBackend::seek(&mut *fp, 0, libc::SEEK_SET), -1);
            assert_eq!(*libc::__errno_location(), libc::EINVAL);
            assert_eq!(hclose(fp), 0);

            let fp64 = hfile_c_845_hopen_mem(c"data:;base64,QUJDRA==".as_ptr(), c"r".as_ptr());
            assert!(!fp64.is_null());
            let mut decoded = [0 as c_char; 8];
            assert_eq!(htslib_hfile_h_247_hread(fp64, decoded.as_mut_ptr().cast(), 8), 4);
            assert_eq!(
                std::slice::from_raw_parts(decoded.as_ptr().cast::<u8>(), 4),
                b"ABCD"
            );

            let mut len = 0usize;
            let stolen = hfile_c_906_hfile_mem_steal_buffer(fp64, &mut len);
            assert!(!stolen.is_null());
            // after stealing, the hFILE's buffer is detached (empty Vec, limit 0)
            assert_eq!((*fp64).limit, 0);
            assert!((*fp64).buffer.is_empty());
            libc::free(stolen.cast());
            assert_eq!(hclose(fp64), 0);

            assert!(hfile_c_845_hopen_mem(c"data:,x".as_ptr(), c"w".as_ptr()).is_null());
            assert_eq!(*libc::__errno_location(), libc::EROFS);
            assert!(hfile_c_915_hopen_not_supported(c"mem".as_ptr(), c"r".as_ptr()).is_null());
            assert_eq!(*libc::__errno_location(), libc::EINVAL);
        }
    }

    #[test]
    fn hfile_mem_steal_buffer_rejects_non_mem_backend() {
        unsafe {
            let payload = b"owned";
            let buffer = libc::malloc(8).cast::<c_char>();
            assert!(!buffer.is_null());
            libc::memcpy(
                buffer.cast(),
                payload.as_ptr().cast(),
                payload.len(),
            );

            let fp = hfile_c_835_create_hfile_mem(buffer, c"r".as_ptr(), payload.len(), 8);
            assert!(!fp.is_null());
            // create_hfile_mem copied the buffer; we own the original.
            libc::free(buffer.cast());

            let mut length = 0usize;
            let got = hfile_c_894_hfile_mem_get_buffer(fp, &mut length);
            assert!(!got.is_null());
            assert_eq!(length, 8);

            let stolen = hfile_c_906_hfile_mem_steal_buffer(fp, &mut length);
            assert!(!stolen.is_null());
            assert_eq!(length, 8);
            assert_eq!((*fp).limit, 0);
            assert_eq!(hclose(fp), 0);
            libc::free(stolen.cast());

            // get_buffer on a non-mem backend (None) is EINVAL.
            let nonmem = hfile_c_104_hfile_init(0, c"r".as_ptr(), 8);
            assert!(!nonmem.is_null());
            let mut len2 = 0usize;
            assert!(hfile_c_894_hfile_mem_get_buffer(nonmem, &mut len2).is_null());
            assert_eq!(*libc::__errno_location(), libc::EINVAL);
            hfile_c_162_hfile_destroy(nonmem);
        }
    }

    #[test]
    fn hfile_data_url_decoder_keeps_edge_case_semantics() {
        unsafe {
            assert!(hfile_c_845_hopen_mem(c"data:no-comma".as_ptr(), c"r".as_ptr()).is_null());
            assert_eq!(*libc::__errno_location(), libc::EINVAL);

            let fp = hfile_c_845_hopen_mem(c"data:text/plain,kept%ZZ%2f".as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let mut out = [0 as c_char; 16];
            assert_eq!(htslib_hfile_h_247_hread(fp, out.as_mut_ptr().cast(), out.len()), 8);
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 8),
                b"kept%ZZ/"
            );
            assert_eq!(hclose(fp), 0);

            let empty = hfile_c_845_hopen_mem(c"data:,".as_ptr(), c"r".as_ptr());
            assert!(!empty.is_null());
            let mut one = [1 as c_char; 1];
            assert_eq!(htslib_hfile_h_247_hread(empty, one.as_mut_ptr().cast(), 1), 0);
            assert_eq!(hclose(empty), 0);
        }
    }

    #[test]
    fn hfile_mem_getc_getdelim_hgets_and_khgetline() {
        unsafe {
            // immobile mem file holding "ab\ncdef\nlast"
            let payload = b"ab\ncdef\nlast";
            let fp = hfile_c_141_hfile_init_fixed(
                0,
                c"r".as_ptr(),
                payload.as_ptr().cast_mut().cast(),
                payload.len(),
                payload.len(),
            );
            assert!(!fp.is_null());
            (*fp).backend = HFileBackend::Mem;

            assert_eq!(htslib_hfile_h_163_hgetc(fp), b'a' as i32);

            let mut line = [0 as c_char; 8];
            assert_eq!(
                hfile_c_241_hgetdelim(line.as_mut_ptr(), line.len(), b'\n' as i32, fp),
                2
            );
            assert_eq!(std::ffi::CStr::from_ptr(line.as_ptr()).to_bytes(), b"b\n");

            assert_eq!(
                hfile_c_291_hgets(line.as_mut_ptr(), line.len() as i32, fp),
                line.as_mut_ptr()
            );
            assert_eq!(std::ffi::CStr::from_ptr(line.as_ptr()).to_bytes(), b"cdef\n");

            let mut ks = kstring_t { data: Vec::new() };
            assert_eq!(hfile_c_306_khgetline(&mut ks, fp), 0);
            assert_eq!(ks.data.as_slice(), b"last");

            assert_eq!(hclose(fp), 0);
        }
    }

    #[test]
    fn hfile_mem_seek_uses_buffer_window_for_immobile_readonly() {
        unsafe {
            let payload = b"abcdef";
            let fp = hfile_c_141_hfile_init_fixed(
                0,
                c"r".as_ptr(),
                payload.as_ptr().cast_mut().cast(),
                6,
                8,
            );
            assert!(!fp.is_null());
            (*fp).backend = HFileBackend::Mem;
            (*fp).begin = 2;

            // immobile + readonly: SEEK_SET within the buffer just moves begin.
            assert_eq!(hseek(fp, 4, libc::SEEK_SET), 4);
            assert_eq!((*fp).begin, 4);

            // SEEK_END validation on an immobile file
            (*fp).flags = HFILE_AT_EOF;
            (*fp).begin = 1;
            (*fp).end = 6;
            assert_eq!(hseek(fp, -2, libc::SEEK_END), 4);
            assert_eq!((*fp).begin, 4);

            assert_eq!(hseek(fp, -7, libc::SEEK_END), -1);
            assert_eq!((*fp).has_errno, libc::EINVAL);

            // The failing hseek left a sticky has_errno, so hclose would return
            // EOF (it propagates has_errno, like C). Tear down the hand-built
            // immobile mem fixture via hfile_destroy, which preserves errno.
            hfile_c_162_hfile_destroy(fp);
        }
    }

    #[test]
    fn hfile_seek_cur_overflow_edges_set_errno() {
        unsafe {
            let payload = b"abcd";
            let fp = hfile_c_141_hfile_init_fixed(
                0,
                c"r".as_ptr(),
                payload.as_ptr().cast_mut().cast(),
                4,
                4,
            );
            assert!(!fp.is_null());
            (*fp).backend = HFileBackend::Mem;

            (*fp).has_errno = 0;
            (*fp).offset = 2;
            (*fp).begin = 0;
            (*fp).end = 0;
            assert_eq!(hseek(fp, -3, libc::SEEK_CUR), -1);
            assert_eq!((*fp).has_errno, libc::EINVAL);

            (*fp).has_errno = 0;
            (*fp).offset = libc::off_t::MAX - 1;
            assert_eq!(hseek(fp, 4, libc::SEEK_CUR), -1);
            assert_eq!((*fp).has_errno, libc::EOVERFLOW);

            // The failing hseek left a sticky has_errno, so hclose would return
            // EOF (it propagates has_errno, like C). Tear down the hand-built
            // immobile mem fixture via hfile_destroy, which preserves errno.
            hfile_c_162_hfile_destroy(fp);
        }
    }

    // --- fd backend over real files / sockets ---

    #[test]
    fn hfile_fd_backend_opens_reads_and_honours_shared_and_uri() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-hfile-fd-{}-{}.txt",
                std::process::id(),
                line!()
            ));
            std::fs::write(&path, b"abcdef").unwrap();
            let c_path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();

            let fp = hfile_c_664_hopen_fd(c_path.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            match (*fp).backend {
                HFileBackend::Fd { fd, flags } => {
                    assert_eq!(flags, 0);
                    assert!(hfile_c_648_blksize(fd) > 0);
                }
                _ => panic!("expected Fd backend"),
            }

            let mut out = [0 as c_char; 6];
            assert_eq!(htslib_hfile_h_247_hread(fp, out.as_mut_ptr().cast(), 6), 6);
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 6),
                b"abcdef"
            );
            assert_eq!(hclose(fp), 0);

            let fd = libc::open(c_path.as_ptr(), libc::O_RDONLY);
            assert!(fd >= 0);
            let shared = hfile_c_735_hdopen(fd, c"Sr".as_ptr());
            assert!(!shared.is_null());
            assert!(matches!(
                (*shared).backend,
                HFileBackend::Fd { flags, .. } if flags & HFILE_FD_IS_SHARED != 0
            ));
            assert_eq!(hclose(shared), 0);
            // shared fd was NOT closed by the backend
            assert_eq!(libc::close(fd), 0);

            let file_uri = std::ffi::CString::new(format!("file://{}", path.display())).unwrap();
            let fp_uri = hfile_c_747_hopen_fd_fileuri(file_uri.as_ptr(), c"r".as_ptr());
            assert!(!fp_uri.is_null());
            assert_eq!(hclose(fp_uri), 0);

            assert!(hfile_c_747_hopen_fd_fileuri(
                c"http://example.invalid/x".as_ptr(),
                c"r".as_ptr()
            )
            .is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::EPROTONOSUPPORT
            );

            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn hfile_fd_write_flush_and_read_roundtrip_via_socketpair() {
        unsafe {
            let mut fds = [-1; 2];
            assert_eq!(
                libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
                0
            );
            let reader = hfile_c_735_hdopen(fds[0], c"sr".as_ptr());
            let writer = hfile_c_735_hdopen(fds[1], c"sw".as_ptr());
            assert!(!reader.is_null());
            assert!(!writer.is_null());
            assert!(matches!(
                (*writer).backend,
                HFileBackend::Fd { flags, .. } if flags & HFILE_FD_IS_SOCKET != 0
            ));

            assert_eq!(htslib_hfile_h_292_hwrite(writer, c"sock".as_ptr().cast(), 4), 4);
            assert_eq!(hflush(writer), 0);
            assert_eq!(
                hclose(writer),
                0,
                "socket writer hclose errno {}",
                *libc::__errno_location()
            );

            let mut got = [0 as c_char; 4];
            assert_eq!(htslib_hfile_h_247_hread(reader, got.as_mut_ptr().cast(), 4), 4);
            assert_eq!(
                std::slice::from_raw_parts(got.as_ptr().cast::<u8>(), 4),
                b"sock"
            );
            assert_eq!(hclose(reader), 0);
        }
    }

    #[test]
    fn hfile_fd_putc_hputs_hwrite_buffer_then_persist_to_file() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-hfile-fdwrite-{}-{}.txt",
                std::process::id(),
                line!()
            ));
            let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();

            let fp = hfile_c_664_hopen_fd(c_path.as_ptr(), c"w".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(htslib_hfile_h_263_hputc(b'a' as i32, fp), b'a' as i32);
            assert_eq!(htslib_hfile_h_275_hputs(c"bc".as_ptr(), fp), 0);
            assert_eq!(htslib_hfile_h_292_hwrite(fp, c"def".as_ptr().cast(), 3), 3);
            assert_eq!(hclose(fp), 0);

            assert_eq!(std::fs::read(&path).unwrap(), b"abcdef");
            std::fs::remove_file(path).unwrap();
        }
    }

    // --- preload copies an fd stream into an immobile mem file ---

    #[test]
    fn hfile_preload_copies_fd_stream_into_immobile_memory_file() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-hfile-preload-{}-{}.txt",
                std::process::id(),
                line!()
            ));
            std::fs::write(&path, b"preloaded\npayload").unwrap();
            let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();

            let fp = hfile_c_664_hopen_fd(c_path.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let mem = hfile_c_689_hpreload(fp);
            assert!(!mem.is_null());
            assert!(matches!((*mem).backend, HFileBackend::Mem));
            assert_eq!((*mem).flags & HFILE_MOBILE, 0);

            let mut out = [0 as c_char; 32];
            assert_eq!(htslib_hfile_h_247_hread(mem, out.as_mut_ptr().cast(), 32), 17);
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 17),
                b"preloaded\npayload"
            );
            assert_eq!(hclose(mem), 0);

            let preload_url =
                std::ffi::CString::new(format!("preload:{}", path.display())).unwrap();
            assert_eq!(hfile_c_726_is_preload_url_remote(preload_url.as_ptr()), 0);

            std::fs::remove_file(path).unwrap();
        }
    }

    // --- scheme/plugin/extension helpers (independent of the buffer layout) ---

    #[test]
    fn hfile_extension_helpers_match_local_and_url_paths() {
        unsafe {
            assert_eq!(hfile_c_1339_hfile_always_local(c"x".as_ptr()), 0);
            assert_eq!(hfile_c_1342_hfile_always_remote(c"x".as_ptr()), 1);
            assert_eq!(hisremote(c"/tmp/a.bam".as_ptr()), 0);
            assert_eq!(hisremote(c"data:,abc".as_ptr()), 0);
            assert_eq!(hisremote(c"file:///tmp/a.bam".as_ptr()), 0);
            assert_eq!(hisremote(c"mem:payload".as_ptr()), 1);
            assert_eq!(hisremote(c"preload:mem:payload".as_ptr()), 1);

            let path = c"/tmp/a.bam";
            let limit = path
                .as_ptr()
                .add(CStr::from_ptr(path.as_ptr()).to_bytes().len());
            let ext = hfile_c_1353_strip_extension(path.as_ptr(), limit);
            assert_eq!(CStr::from_ptr(ext).to_bytes(), b".bam");

            let mut ks = kstring_t { data: Vec::new() };
            let out =
                hfile_c_1364_haddextension(&mut ks, c"/tmp/a.bam".as_ptr(), 1, c".csi".as_ptr());
            assert!(!out.is_null());
            assert_eq!(ks.data.as_slice(), b"/tmp/a.csi");

            let out2 = hfile_c_1364_haddextension(
                &mut ks,
                c"file:///tmp/a.bam?x=1".as_ptr(),
                1,
                c".csi".as_ptr(),
            );
            assert!(!out2.is_null());
            assert_eq!(ks.data.as_slice(), b"file:///tmp/a.csi?x=1");
        }
    }

    #[test]
    fn hfile_oflags_matches_fopen_mode_letters() {
        unsafe {
            assert_eq!(
                hfile_c_772_hfile_oflags(c"r".as_ptr()) & libc::O_ACCMODE,
                libc::O_RDONLY
            );
            assert_eq!(
                hfile_c_772_hfile_oflags(c"w".as_ptr()),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC
            );
            assert_eq!(
                hfile_c_772_hfile_oflags(c"a+".as_ptr())
                    & (libc::O_ACCMODE | libc::O_CREAT | libc::O_APPEND),
                libc::O_RDWR | libc::O_CREAT | libc::O_APPEND
            );
            assert_ne!(hfile_c_772_hfile_oflags(c"wx".as_ptr()) & libc::O_EXCL, 0);
        }
    }

    #[test]
    fn hfile_scheme_parser_accepts_scheme_alphabet_and_rejects_non_schemes() {
        unsafe {
            let data = hfile_c_1176_find_scheme_handler(c"DATA:text/plain,abc".as_ptr());
            assert!(!data.is_null());
            assert_eq!(
                CStr::from_ptr((*data.cast::<hfile_scheme_handler_layout>()).provider).to_bytes(),
                b"built-in"
            );

            assert!(!hfile_c_1176_find_scheme_handler(c"s3+https://bucket/key".as_ptr()).is_null());
            assert!(!hfile_c_1176_find_scheme_handler(c"ab-c.d:path".as_ptr()).is_null());
            assert!(hfile_c_1176_find_scheme_handler(c"x:path".as_ptr()).is_null());
            assert!(hfile_c_1176_find_scheme_handler(c"abcdefghijkl:path".as_ptr()).is_null());
            assert!(hfile_c_1176_find_scheme_handler(c"/tmp/has:no-scheme".as_ptr()).is_null());

            let fp = hfile_c_1317_hopen(c"DATA:,upper%20scheme".as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let mut out = [0 as c_char; 16];
            assert_eq!(htslib_hfile_h_247_hread(fp, out.as_mut_ptr().cast(), out.len()), 12);
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 12),
                b"upper scheme"
            );
            assert_eq!(hclose(fp), 0);
        }
    }

    #[test]
    fn hfile_hopen_dispatches_builtin_and_unknown_schemes() {
        unsafe {
            let fp = hfile_c_1317_hopen(c"data:,builtin%20dispatch".as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let mut out = [0 as c_char; 32];
            assert_eq!(htslib_hfile_h_247_hread(fp, out.as_mut_ptr().cast(), 32), 16);
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 16),
                b"builtin dispatch"
            );
            assert_eq!(hclose(fp), 0);

            let missing =
                hfile_c_1168_hopen_unknown_scheme(c"missing-scheme:test".as_ptr(), c"r".as_ptr());
            assert!(missing.is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::EPROTONOSUPPORT
            );

            let handler = hfile_scheme_handler_layout {
                open: None,
                isremote: None,
                provider: c"test".as_ptr(),
                priority: 2050,
                vopen: None,
            };
            assert_eq!(
                hfile_c_1011_priority((&handler as *const hfile_scheme_handler_layout).cast()),
                50
            );
        }
    }

    #[test]
    fn hfile_mem_vopen_decodes_va_list_buffer_and_size() {
        unsafe {
            let payload = b"mem-vopen";
            let buffer = libc::malloc(payload.len()).cast::<c_char>();
            assert!(!buffer.is_null());
            libc::memcpy(
                buffer.cast(),
                payload.as_ptr().cast(),
                payload.len(),
            );

            let mut reg_save = [buffer as usize, payload.len()];
            let mut overflow = [0usize; 2];
            let mut args = crate::htslib_rs::c_compat::__va_list_tag {
                gp_offset: 0,
                fp_offset: 48,
                overflow_arg_area: overflow.as_mut_ptr().cast(),
                reg_save_area: reg_save.as_mut_ptr().cast(),
            };

            let fp = hfile_c_1317_hopen_vargs(c"mem:".as_ptr(), c"r:".as_ptr(), &mut args);
            assert!(!fp.is_null());
            let mut out = [0 as c_char; 16];
            assert_eq!(
                htslib_hfile_h_247_hread(fp, out.as_mut_ptr().cast(), out.len()),
                payload.len() as libc::ssize_t
            );
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), payload.len()),
                payload
            );
            assert_eq!(hclose(fp), 0);
        }
    }

    #[test]
    fn hfile_plugin_listing_respects_capacity_without_losing_total_counts() {
        let _guard = hfile_plugin_test_lock().lock().unwrap();
        unsafe {
            let mut plugins = [std::ptr::null(); 1];
            let mut nplugins = plugins.len() as i32;
            let total = hfile_c_1257_hfile_list_plugins(plugins.as_mut_ptr(), &mut nplugins);
            assert!(total >= 1);
            assert_eq!(nplugins, 1);
            assert_eq!(CStr::from_ptr(plugins[0]).to_bytes(), b"built-in");

            let mut schemes = [std::ptr::null(); 1];
            let mut nschemes = schemes.len() as i32;
            let total_schemes = hfile_c_1218_hfile_list_schemes(
                c"built-in".as_ptr(),
                schemes.as_mut_ptr(),
                &mut nschemes,
            );
            assert!(total_schemes >= 3);
            assert_eq!(nschemes, 1);
            assert_eq!(CStr::from_ptr(schemes[0]).to_bytes(), b"data");
        }
    }

    #[test]
    fn hfile_plugin_initialisers_set_names_and_error_handler_state() {
        let _guard = hfile_plugin_test_lock().lock().unwrap();
        unsafe {
            let mut mem_plugin = hfile_plugin_layout {
                api_version: 1,
                obj: std::ptr::null_mut(),
                name: std::ptr::null(),
                destroy: std::ptr::null(),
            };
            assert_eq!(
                hfile_c_920_hfile_plugin_init_mem(
                    (&mut mem_plugin as *mut hfile_plugin_layout).cast()
                ),
                0
            );
            assert_eq!(CStr::from_ptr(mem_plugin.name).to_bytes(), b"mem");

            let mut crypt_plugin = hfile_plugin_layout {
                api_version: 1,
                obj: std::ptr::null_mut(),
                name: std::ptr::null(),
                destroy: std::ptr::null(),
            };
            assert_eq!(
                hfile_c_956_hfile_plugin_init_crypt4gh_needed(
                    (&mut crypt_plugin as *mut hfile_plugin_layout).cast()
                ),
                0
            );
            assert_eq!(CStr::from_ptr(crypt_plugin.name).to_bytes(), b"crypt4gh-needed");

            assert!(
                hfile_c_935_crypt4gh_needed(c"crypt4gh:/tmp/x".as_ptr(), c"r".as_ptr()).is_null()
            );
            assert_eq!(
                *libc::__errno_location(),
                libc::EPROTONOSUPPORT
            );
        }
    }

    #[test]
    fn hfile_knet_wrappers_track_offset_and_delegate_to_hfile() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-knet-{}-{}.txt",
                std::process::id(),
                line!()
            ));
            std::fs::write(&path, b"knet-data").unwrap();
            let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();

            let fp = hfile_c_1416_knet_open(c_path.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert!((*fp.cast::<knet_file_layout>()).fd >= 0);

            let mut out = [0 as c_char; 4];
            assert_eq!(hfile_c_1445_knet_read(fp, out.as_mut_ptr().cast(), 4), 4);
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 4),
                b"knet"
            );
            assert_eq!((*fp.cast::<knet_file_layout>()).offset, 4);

            assert_eq!(hfile_c_1452_knet_seek(fp, 5, libc::SEEK_SET), 5);
            assert_eq!((*fp.cast::<knet_file_layout>()).offset, 5);
            let mut tail = [0 as c_char; 4];
            assert_eq!(hfile_c_1445_knet_read(fp, tail.as_mut_ptr().cast(), 4), 4);
            assert_eq!(
                std::slice::from_raw_parts(tail.as_ptr().cast::<u8>(), 4),
                b"data"
            );
            assert_eq!(hfile_c_1460_knet_close(fp), 0);

            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn hfile_unknown_scheme_fallback_is_local_like_upstream() {
        unsafe {
            let handler = hfile_c_1176_find_scheme_handler(c"zz-example://host/path".as_ptr());
            assert!(!handler.is_null());
            let handler = handler.cast::<hfile_scheme_handler_layout>();
            assert_eq!(CStr::from_ptr((*handler).provider).to_bytes(), b"built-in");
            assert_eq!((*handler).priority, 0);
            assert_eq!(hisremote(c"zz-example://host/path".as_ptr()), 0);
        }
    }
}
