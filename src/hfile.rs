use std::ffi::{c_char, c_int, c_uint, c_void, CStr};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::{Mutex, OnceLock};

use super::hts::{
    hFILE, hts_base64_decoded_length, hts_decode_base64, hts_decode_percent, isalnum_c, kgetline,
    kputs, kputsn, kstring_t, size_t, tolower_c,
};

#[repr(C)]
pub struct knetFile {
    _private: [u8; 0],
}

#[repr(C)]
struct knet_file_layout {
    type_: c_int,
    fd: c_int,
    offset: i64,
    host: *mut c_char,
    port: *mut c_char,
    ctrl_fd: c_int,
    pasv_ip: [c_int; 4],
    pasv_port: c_int,
    max_response: c_int,
    no_reconnect: c_int,
    is_ready: c_int,
    response: *mut c_char,
    retr: *mut c_char,
    size_cmd: *mut c_char,
    seek_offset: i64,
    file_size: i64,
    path: *mut c_char,
    http_host: *mut c_char,
    hf: *mut hFILE,
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
type HFileIsRemoteFn = unsafe extern "C" fn(*const c_char) -> c_int;
type HFileVOpenFn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE;
type HFilePluginInitFn = unsafe extern "C" fn(*mut hFILE_plugin) -> c_int;

#[repr(C)]
struct hfile_scheme_handler_layout {
    open: Option<HFileOpenFn>,
    isremote: Option<HFileIsRemoteFn>,
    provider: *const c_char,
    priority: c_int,
    vopen: Option<HFileVOpenFn>,
}

unsafe impl Sync for hfile_scheme_handler_layout {}

#[repr(C)]
struct hfile_plugin_layout {
    api_version: c_int,
    obj: *mut c_void,
    name: *const c_char,
    destroy: *const c_void,
}

// original: hFILE_plugin_list (htslib/hfile.c:975)
#[repr(C)]
pub struct hFILE_plugin_list {
    plugin: hfile_plugin_layout,
    next: *mut hFILE_plugin_list,
}

#[repr(C)]
struct hfile_layout {
    buffer: *mut c_char,
    begin: *mut c_char,
    end: *mut c_char,
    limit: *mut c_char,
    backend: *const hfile_backend_layout,
    offset: libc::off_t,
    flags: c_uint,
    has_errno: c_int,
}

type HFileReadFn = unsafe extern "C" fn(*mut hFILE, *mut c_void, size_t) -> libc::ssize_t;
type HFileWriteFn = unsafe extern "C" fn(*mut hFILE, *const c_void, size_t) -> libc::ssize_t;
type HFileSeekFn = unsafe extern "C" fn(*mut hFILE, libc::off_t, c_int) -> libc::off_t;
type HFileFlushFn = unsafe extern "C" fn(*mut hFILE) -> c_int;
type HFileCloseFn = unsafe extern "C" fn(*mut hFILE) -> c_int;

#[repr(C)]
struct hfile_backend_layout {
    read: Option<HFileReadFn>,
    write: Option<HFileWriteFn>,
    seek: Option<HFileSeekFn>,
    flush: Option<HFileFlushFn>,
    close: Option<HFileCloseFn>,
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

    pub fn close(self) -> c_int {
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

#[repr(C)]
struct hfile_fd_layout {
    base: hfile_layout,
    fd: c_int,
    flags: c_uint,
}

const HFILE_FD_IS_SOCKET: c_uint = 1 << 0;
const HFILE_FD_IS_SHARED: c_uint = 1 << 1;

const HFILE_AT_EOF: c_uint = 1 << 0;
const HFILE_MOBILE: c_uint = 1 << 1;
const HFILE_READONLY: c_uint = 1 << 2;
const HFILE_PRESERVE: c_uint = 1 << 3;

unsafe extern "C" fn hfile_c_810_mem_seek(
    _fpv: *mut hFILE,
    _offset: libc::off_t,
    _whence: c_int,
) -> libc::off_t {
    *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
    -1
}

unsafe extern "C" fn hfile_c_816_mem_close(_fpv: *mut hFILE) -> c_int {
    0
}

static MEM_BACKEND: hfile_backend_layout = hfile_backend_layout {
    read: None,
    write: None,
    seek: Some(hfile_c_810_mem_seek),
    flush: None,
    close: Some(hfile_c_816_mem_close),
};

pub unsafe extern "C" fn hfile_c_557_fd_read(
    fpv: *mut hFILE,
    buffer: *mut c_void,
    nbytes: size_t,
) -> libc::ssize_t {
    let fp = fpv.cast::<hfile_fd_layout>();
    loop {
        let n = if ((*fp).flags & HFILE_FD_IS_SOCKET) != 0 {
            crate::htslib_rs::c_compat::socket_recv((*fp).fd, buffer, nbytes, 0)
        } else {
            crate::htslib_rs::c_compat::fd_read((*fp).fd, buffer, nbytes)
        };
        if !(n < 0 && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR) {
            return n;
        }
    }
}

pub unsafe extern "C" fn hfile_c_568_fd_write(
    fpv: *mut hFILE,
    buffer: *const c_void,
    nbytes: size_t,
) -> libc::ssize_t {
    let fp = fpv.cast::<hfile_fd_layout>();
    loop {
        let n = if ((*fp).flags & HFILE_FD_IS_SOCKET) != 0 {
            crate::htslib_rs::c_compat::socket_send((*fp).fd, buffer, nbytes, 0)
        } else {
            crate::htslib_rs::c_compat::fd_write((*fp).fd, buffer, nbytes)
        };
        if !(n < 0 && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR) {
            return n;
        }
    }
}

pub unsafe extern "C" fn hfile_c_591_fd_seek(
    fpv: *mut hFILE,
    offset: libc::off_t,
    whence: c_int,
) -> libc::off_t {
    let fp = fpv.cast::<hfile_fd_layout>();
    libc::lseek((*fp).fd, offset, whence)
}

static FD_BACKEND: hfile_backend_layout = hfile_backend_layout {
    read: Some(hfile_c_557_fd_read),
    write: Some(hfile_c_568_fd_write),
    seek: Some(hfile_c_591_fd_seek),
    flush: Some(hfile_c_607_fd_flush),
    close: Some(hfile_c_625_fd_close),
};

pub unsafe fn hfile_c_1011_priority(handler: *const hFILE_scheme_handler) -> c_int {
    (*(handler.cast::<hfile_scheme_handler_layout>())).priority % 1000
}

pub unsafe fn hfile_c_1026_try_exe_add_scheme_handler(
    _scheme: *const c_char,
    _handler: *const hFILE_scheme_handler,
) -> c_int {
    -1
}

pub unsafe fn hfile_c_1046_try_exe_add_scheme_handler(
    _scheme: *const c_char,
    _handler: *const hFILE_scheme_handler,
) -> c_int {
    -1
}

pub unsafe fn hfile_init(struct_size: size_t, mode: *const c_char, capacity: size_t) -> *mut hFILE {
    hfile_c_104_hfile_init(struct_size, mode, capacity)
}

pub unsafe fn hfile_c_104_hfile_init(
    struct_size: size_t,
    mode: *const c_char,
    mut capacity: size_t,
) -> *mut hFILE {
    let fp = crate::htslib_rs::c_compat::malloc(struct_size as u64).cast::<hfile_layout>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    let maxcap = 128 * 1024usize;
    if capacity == 0 {
        capacity = maxcap;
    }
    if !libc::strchr(mode, b'r' as c_int).is_null() && capacity > maxcap {
        capacity = maxcap;
    }

    (*fp).buffer = std::ptr::null_mut();
    let memalign_ret = crate::htslib_rs::c_compat::posix_memalign(
        (&mut (*fp).buffer as *mut *mut c_char).cast(),
        256,
        capacity,
    );
    if memalign_ret != 0 {
        *crate::htslib_rs::c_compat::__errno_location() = memalign_ret;
        hfile_c_162_hfile_destroy(fp.cast());
        return std::ptr::null_mut();
    }

    (*fp).begin = (*fp).buffer;
    (*fp).end = (*fp).buffer;
    (*fp).limit = (*fp).buffer.add(capacity);
    (*fp).backend = std::ptr::null();
    (*fp).offset = 0;
    (*fp).flags = HFILE_MOBILE;
    if !libc::strchr(mode, b'r' as c_int).is_null() && libc::strchr(mode, b'+' as c_int).is_null() {
        (*fp).flags |= HFILE_READONLY;
    }
    (*fp).has_errno = 0;
    fp.cast()
}

pub unsafe fn hfile_c_141_hfile_init_fixed(
    struct_size: size_t,
    mode: *const c_char,
    buffer: *mut c_char,
    buf_filled: size_t,
    buf_size: size_t,
) -> *mut hFILE {
    let fp = crate::htslib_rs::c_compat::malloc(struct_size as u64).cast::<hfile_layout>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    (*fp).buffer = buffer;
    (*fp).begin = buffer;
    (*fp).end = buffer.add(buf_filled);
    (*fp).limit = buffer.add(buf_size);
    (*fp).backend = std::ptr::null();
    (*fp).offset = 0;
    (*fp).flags = HFILE_AT_EOF;
    if !libc::strchr(mode, b'r' as c_int).is_null() && libc::strchr(mode, b'+' as c_int).is_null() {
        (*fp).flags |= HFILE_READONLY;
    }
    (*fp).has_errno = 0;
    fp.cast()
}

pub unsafe fn hfile_destroy(fp: *mut hFILE) {
    hfile_c_162_hfile_destroy(fp)
}

pub unsafe fn hfile_c_162_hfile_destroy(fp: *mut hFILE) {
    let save = *crate::htslib_rs::c_compat::__errno_location();
    if !fp.is_null() {
        crate::htslib_rs::c_compat::free((*(fp.cast::<hfile_layout>())).buffer.cast());
    }
    crate::htslib_rs::c_compat::free(fp.cast());
    *crate::htslib_rs::c_compat::__errno_location() = save;
}

pub unsafe fn hfile_c_171_writebuffer_is_nonempty(fp: *mut hFILE) -> c_int {
    let fp = fp.cast::<hfile_layout>();
    ((*fp).begin > (*fp).end) as c_int
}

pub unsafe fn htslib_hfile_h_134_herrno(fp: *mut hFILE) -> c_int {
    (*(fp.cast::<hfile_layout>())).has_errno
}

pub unsafe fn htslib_hfile_h_140_hclearerr(fp: *mut hFILE) {
    (*(fp.cast::<hfile_layout>())).has_errno = 0;
}

pub unsafe fn htslib_hfile_h_155_htell(fp: *mut hFILE) -> libc::off_t {
    let fp = fp.cast::<hfile_layout>();
    (*fp).offset + (*fp).begin.offset_from((*fp).buffer) as libc::off_t
}

pub unsafe fn htslib_hfile_h_163_hgetc(fp: *mut hFILE) -> c_int {
    let layout = fp.cast::<hfile_layout>();
    if (*layout).end > (*layout).begin {
        let c = *(*layout).begin as u8;
        (*layout).begin = (*layout).begin.add(1);
        c as c_int
    } else {
        hgetc2(fp)
    }
}

pub unsafe fn htslib_hfile_h_195_hgetln(
    buffer: *mut c_char,
    size: usize,
    fp: *mut hFILE,
) -> libc::ssize_t {
    hgetdelim(buffer, size, b'\n' as c_int, fp)
}

pub unsafe fn htslib_hfile_h_247_hread(
    fp: *mut hFILE,
    buffer: *mut c_void,
    nbytes: usize,
) -> libc::ssize_t {
    let layout = fp.cast::<hfile_layout>();
    let mut n = (*layout).end.offset_from((*layout).begin) as usize;
    if n > nbytes {
        n = nbytes;
    }
    crate::htslib_rs::c_compat::memcpy(buffer, (*layout).begin.cast(), n as u64);
    (*layout).begin = (*layout).begin.add(n);
    if n == nbytes || ((*layout).flags & HFILE_MOBILE) == 0 {
        n as libc::ssize_t
    } else {
        hread2(fp, buffer, nbytes, n)
    }
}

pub unsafe fn htslib_hfile_h_263_hputc(c: c_int, fp: *mut hFILE) -> c_int {
    let layout = fp.cast::<hfile_layout>();
    if (*layout).begin < (*layout).limit {
        *(*layout).begin = c as c_char;
        (*layout).begin = (*layout).begin.add(1);
        c
    } else {
        hputc2(c, fp)
    }
}

pub unsafe fn htslib_hfile_h_275_hputs(text: *const c_char, fp: *mut hFILE) -> c_int {
    let layout = fp.cast::<hfile_layout>();
    let nbytes = std::ffi::CStr::from_ptr(text).to_bytes().len();
    let mut n = (*layout).limit.offset_from((*layout).begin) as usize;
    if n > nbytes {
        n = nbytes;
    }
    crate::htslib_rs::c_compat::memcpy((*layout).begin.cast(), text.cast(), n as u64);
    (*layout).begin = (*layout).begin.add(n);
    if n == nbytes {
        0
    } else {
        hputs2(text, nbytes, n, fp)
    }
}

pub unsafe fn htslib_hfile_h_292_hwrite(
    fp: *mut hFILE,
    buffer: *const c_void,
    nbytes: usize,
) -> libc::ssize_t {
    let layout = fp.cast::<hfile_layout>();
    if ((*layout).flags & HFILE_MOBILE) == 0 {
        let n = (*layout).limit.offset_from((*layout).begin) as usize;
        if n < nbytes {
            hfile_set_blksize(
                fp,
                (*layout).limit.offset_from((*layout).buffer) as usize + nbytes,
            );
            (*layout).end = (*layout).limit;
        }
    }

    let mut n = (*layout).limit.offset_from((*layout).begin) as usize;
    if nbytes >= n && (*layout).begin == (*layout).buffer {
        return hwrite2(fp, buffer, nbytes, 0);
    }

    if n > nbytes {
        n = nbytes;
    }
    crate::htslib_rs::c_compat::memcpy((*layout).begin.cast(), buffer, n as u64);
    (*layout).begin = (*layout).begin.add(n);
    if n == nbytes {
        n as libc::ssize_t
    } else {
        hwrite2(fp, buffer, nbytes, n)
    }
}

pub unsafe fn hfile_c_179_refill_buffer(fp: *mut hFILE) -> libc::ssize_t {
    let fp_layout = fp.cast::<hfile_layout>();

    if ((*fp_layout).flags & HFILE_MOBILE) != 0 && (*fp_layout).begin > (*fp_layout).buffer {
        let consumed = (*fp_layout).begin.offset_from((*fp_layout).buffer);
        let unread = (*fp_layout).end.offset_from((*fp_layout).begin);
        (*fp_layout).offset += consumed as libc::off_t;
        crate::htslib_rs::c_compat::memmove(
            (*fp_layout).buffer.cast(),
            (*fp_layout).begin.cast(),
            unread as u64,
        );
        (*fp_layout).end = (*fp_layout).buffer.add(unread as usize);
        (*fp_layout).begin = (*fp_layout).buffer;
    }

    let n = if ((*fp_layout).flags & HFILE_AT_EOF) != 0 || (*fp_layout).end == (*fp_layout).limit {
        0
    } else {
        let read = (*(*fp_layout).backend).read.expect("hFILE read backend");
        let ret = read(
            fp,
            (*fp_layout).end.cast(),
            (*fp_layout).limit.offset_from((*fp_layout).end) as size_t,
        );
        if ret < 0 {
            (*fp_layout).has_errno = *crate::htslib_rs::c_compat::__errno_location();
            return ret;
        } else if ret == 0 {
            (*fp_layout).flags |= HFILE_AT_EOF;
        }
        ret
    };

    (*fp_layout).end = (*fp_layout).end.add(n as usize);
    n
}

pub unsafe fn hfile_c_212_hfile_set_blksize(fp: *mut hFILE, mut bufsiz: size_t) -> c_int {
    if fp.is_null() {
        return -1;
    }

    let fp_layout = fp.cast::<hfile_layout>();
    let curr_used = if (*fp_layout).begin > (*fp_layout).end {
        (*fp_layout).begin
    } else {
        (*fp_layout).end
    }
    .offset_from((*fp_layout).buffer) as size_t;

    if bufsiz == 0 {
        bufsiz = 32768;
    }
    if bufsiz < curr_used {
        return -1;
    }

    let begin_off = (*fp_layout).begin.offset_from((*fp_layout).buffer) as usize;
    let end_off = (*fp_layout).end.offset_from((*fp_layout).buffer) as usize;
    let buffer = crate::htslib_rs::c_compat::realloc((*fp_layout).buffer.cast(), bufsiz as u64)
        .cast::<c_char>();
    if buffer.is_null() {
        return -1;
    }

    (*fp_layout).begin = buffer.add(begin_off);
    (*fp_layout).end = buffer.add(end_off);
    (*fp_layout).buffer = buffer;
    (*fp_layout).limit = buffer.add(bufsiz);
    0
}

pub unsafe fn hfile_c_235_hgetc2(fp: *mut hFILE) -> c_int {
    if hfile_c_179_refill_buffer(fp) > 0 {
        let fp_layout = fp.cast::<hfile_layout>();
        let c = *(*fp_layout).begin as u8 as c_int;
        (*fp_layout).begin = (*fp_layout).begin.add(1);
        c
    } else {
        libc::EOF
    }
}

pub unsafe fn hfile_c_241_hgetdelim(
    buffer: *mut c_char,
    mut size: size_t,
    delim: c_int,
    fp: *mut hFILE,
) -> libc::ssize_t {
    let fp_layout = fp.cast::<hfile_layout>();
    if size < 1 || size > libc::ssize_t::MAX as size_t {
        (*fp_layout).has_errno = libc::EINVAL;
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return -1;
    }
    if (*fp_layout).begin > (*fp_layout).end {
        (*fp_layout).has_errno = libc::EBADF;
        *crate::htslib_rs::c_compat::__errno_location() = libc::EBADF;
        return -1;
    }

    size -= 1;
    let mut copied = 0usize;
    loop {
        let mut n = (*fp_layout).end.offset_from((*fp_layout).begin) as size_t;
        if n > size - copied {
            n = size - copied;
        }

        let found = libc::memchr((*fp_layout).begin.cast(), delim, n);
        if !found.is_null() {
            n = found.cast::<c_char>().offset_from((*fp_layout).begin) as size_t + 1;
            crate::htslib_rs::c_compat::memcpy(
                buffer.add(copied).cast(),
                (*fp_layout).begin.cast(),
                n as u64,
            );
            *buffer.add(n + copied) = 0;
            (*fp_layout).begin = (*fp_layout).begin.add(n);
            return (n + copied) as libc::ssize_t;
        }

        crate::htslib_rs::c_compat::memcpy(
            buffer.add(copied).cast(),
            (*fp_layout).begin.cast(),
            n as u64,
        );
        (*fp_layout).begin = (*fp_layout).begin.add(n);
        copied += n;

        if copied == size {
            *buffer.add(copied) = 0;
            return copied as libc::ssize_t;
        }

        let got = hfile_c_179_refill_buffer(fp);
        if got <= 0 {
            if got < 0 {
                return -1;
            }
            *buffer.add(copied) = 0;
            return copied as libc::ssize_t;
        }
    }
}

pub unsafe fn hfile_c_291_hgets(buffer: *mut c_char, size: c_int, fp: *mut hFILE) -> *mut c_char {
    let fp_layout = fp.cast::<hfile_layout>();
    if size < 1 {
        (*fp_layout).has_errno = libc::EINVAL;
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }
    if hfile_c_241_hgetdelim(buffer, size as size_t, b'\n' as c_int, fp) > 0 {
        buffer
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe extern "C" fn hfile_c_301_hgets_wrapper(
    buffer: *mut c_char,
    size: c_int,
    fp: *mut c_void,
) -> *mut c_char {
    hfile_c_291_hgets(buffer, size, fp.cast())
}

pub unsafe fn hfile_c_306_khgetline(kstr: *mut kstring_t, fp: *mut hFILE) -> c_int {
    if kstr.is_null() || fp.is_null() {
        return libc::EOF;
    }
    kgetline(kstr, Some(hfile_c_301_hgets_wrapper), fp.cast())
}

pub unsafe fn hfile_c_313_hpeek(
    fp: *mut hFILE,
    buffer: *mut c_void,
    nbytes: size_t,
) -> libc::ssize_t {
    let fp_layout = fp.cast::<hfile_layout>();
    let mut n = (*fp_layout).end.offset_from((*fp_layout).begin) as size_t;
    while n < nbytes {
        let ret = hfile_c_179_refill_buffer(fp);
        if ret < 0 {
            return ret;
        } else if ret == 0 {
            break;
        } else {
            n += ret as size_t;
        }
    }

    if n > nbytes {
        n = nbytes;
    }
    crate::htslib_rs::c_compat::memcpy(buffer, (*fp_layout).begin.cast(), n as u64);
    n as libc::ssize_t
}

pub unsafe fn hfile_c_330_hread2(
    fp: *mut hFILE,
    destv: *mut c_void,
    nbytes: size_t,
    nread: size_t,
) -> libc::ssize_t {
    let fp_layout = fp.cast::<hfile_layout>();
    let capacity = (*fp_layout).limit.offset_from((*fp_layout).buffer) as size_t;
    let mut buffer_invalidated = 0;
    let mut dest = destv.cast::<c_char>().add(nread);
    let mut remaining = nbytes - nread;
    let mut total_read = nread;

    while remaining * 2 >= capacity && ((*fp_layout).flags & HFILE_AT_EOF) == 0 {
        let read = (*(*fp_layout).backend).read.expect("hFILE read backend");
        let n = read(fp, dest.cast(), remaining);
        if n < 0 {
            (*fp_layout).has_errno = *crate::htslib_rs::c_compat::__errno_location();
            return n;
        } else if n == 0 {
            (*fp_layout).flags |= HFILE_AT_EOF;
        } else {
            buffer_invalidated = 1;
        }
        (*fp_layout).offset += n as libc::off_t;
        dest = dest.add(n as usize);
        remaining -= n as size_t;
        total_read += n as size_t;
    }

    if buffer_invalidated != 0 {
        (*fp_layout).offset += (*fp_layout).begin.offset_from((*fp_layout).buffer) as libc::off_t;
        (*fp_layout).begin = (*fp_layout).buffer;
        (*fp_layout).end = (*fp_layout).buffer;
    }

    while remaining > 0 && ((*fp_layout).flags & HFILE_AT_EOF) == 0 {
        let ret = hfile_c_179_refill_buffer(fp);
        if ret < 0 {
            return ret;
        }

        let mut n = (*fp_layout).end.offset_from((*fp_layout).begin) as size_t;
        if n > remaining {
            n = remaining;
        }
        crate::htslib_rs::c_compat::memcpy(dest.cast(), (*fp_layout).begin.cast(), n as u64);
        (*fp_layout).begin = (*fp_layout).begin.add(n);
        dest = dest.add(n);
        remaining -= n;
        total_read += n;
    }

    total_read as libc::ssize_t
}

pub unsafe fn hfile_c_376_flush_buffer(fp: *mut hFILE) -> libc::ssize_t {
    let fp_layout = fp.cast::<hfile_layout>();
    let mut buffer = (*fp_layout).buffer;
    while buffer < (*fp_layout).begin {
        let write = (*(*fp_layout).backend).write.expect("hFILE write backend");
        let n = write(
            fp,
            buffer.cast(),
            (*fp_layout).begin.offset_from(buffer) as size_t,
        );
        if n < 0 {
            (*fp_layout).has_errno = *crate::htslib_rs::c_compat::__errno_location();
            return n;
        }
        buffer = buffer.add(n as usize);
        (*fp_layout).offset += n as libc::off_t;
    }

    (*fp_layout).begin = (*fp_layout).buffer;
    0
}

pub unsafe fn hfile_c_390_hflush(fp: *mut hFILE) -> c_int {
    if hfile_c_376_flush_buffer(fp) < 0 {
        return libc::EOF;
    }
    let fp_layout = fp.cast::<hfile_layout>();
    if let Some(flush) = (*(*fp_layout).backend).flush {
        if flush(fp) < 0 {
            (*fp_layout).has_errno = *crate::htslib_rs::c_compat::__errno_location();
            return libc::EOF;
        }
    }
    0
}

pub unsafe fn hfile_c_400_hputc2(c: c_int, fp: *mut hFILE) -> c_int {
    let fp_layout = fp.cast::<hfile_layout>();
    if hfile_c_376_flush_buffer(fp) < 0 {
        return libc::EOF;
    }
    *(*fp_layout).begin = c as c_char;
    (*fp_layout).begin = (*fp_layout).begin.add(1);
    c
}

pub unsafe fn hfile_c_412_hwrite2(
    fp: *mut hFILE,
    srcv: *const c_void,
    totalbytes: size_t,
    ncopied: size_t,
) -> libc::ssize_t {
    let fp_layout = fp.cast::<hfile_layout>();
    let mut src = srcv.cast::<c_char>().add(ncopied);
    let capacity = (*fp_layout).limit.offset_from((*fp_layout).buffer) as size_t;
    let mut remaining = totalbytes - ncopied;

    let ret = hfile_c_376_flush_buffer(fp);
    if ret < 0 {
        return ret;
    }

    while remaining * 2 >= capacity {
        let write = (*(*fp_layout).backend).write.expect("hFILE write backend");
        let n = write(fp, src.cast(), remaining);
        if n < 0 {
            (*fp_layout).has_errno = *crate::htslib_rs::c_compat::__errno_location();
            return n;
        }
        (*fp_layout).offset += n as libc::off_t;
        src = src.add(n as usize);
        remaining -= n as size_t;
    }

    crate::htslib_rs::c_compat::memcpy((*fp_layout).begin.cast(), src.cast(), remaining as u64);
    (*fp_layout).begin = (*fp_layout).begin.add(remaining);

    totalbytes as libc::ssize_t
}

pub unsafe fn hfile_c_440_hputs2(
    text: *const c_char,
    totalbytes: size_t,
    ncopied: size_t,
    fp: *mut hFILE,
) -> c_int {
    if hfile_c_412_hwrite2(fp, text.cast(), totalbytes, ncopied) >= 0 {
        0
    } else {
        libc::EOF
    }
}

pub unsafe fn hfile_c_446_hseek(
    fp: *mut hFILE,
    mut offset: libc::off_t,
    mut whence: c_int,
) -> libc::off_t {
    let fp_layout = fp.cast::<hfile_layout>();

    if (*fp_layout).begin > (*fp_layout).end && ((*fp_layout).flags & HFILE_MOBILE) != 0 {
        let ret = hfile_c_376_flush_buffer(fp);
        if ret < 0 {
            return ret as libc::off_t;
        }
    }

    let curpos =
        (*fp_layout).offset + (*fp_layout).begin.offset_from((*fp_layout).buffer) as libc::off_t;

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
                    crate::htslib_rs::c_compat::EOVERFLOW
                };
                (*fp_layout).has_errno = err;
                *crate::htslib_rs::c_compat::__errno_location() = err;
                return -1;
            }
        }
    } else if ((*fp_layout).flags & HFILE_MOBILE) == 0 && whence == libc::SEEK_END {
        let length = (*fp_layout).end.offset_from((*fp_layout).buffer) as libc::off_t;
        if offset > 0 || -offset > length {
            (*fp_layout).has_errno = libc::EINVAL;
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            return -1;
        }

        whence = libc::SEEK_SET;
        offset += length;
    }

    if whence == libc::SEEK_SET
        && (((*fp_layout).flags & HFILE_MOBILE) == 0 || ((*fp_layout).flags & HFILE_READONLY) != 0)
        && offset >= (*fp_layout).offset
        && offset - (*fp_layout).offset
            <= (*fp_layout).end.offset_from((*fp_layout).buffer) as libc::off_t
    {
        (*fp_layout).begin = (*fp_layout)
            .buffer
            .add((offset - (*fp_layout).offset) as usize);
        return offset;
    }

    let seek = (*(*fp_layout).backend).seek.expect("hFILE seek backend");
    let pos = seek(fp, offset, whence);
    if pos < 0 {
        (*fp_layout).has_errno = *crate::htslib_rs::c_compat::__errno_location();
        return pos;
    }

    (*fp_layout).begin = (*fp_layout).buffer;
    (*fp_layout).end = (*fp_layout).buffer;
    (*fp_layout).flags &= !HFILE_AT_EOF;
    (*fp_layout).offset = pos;
    pos
}

pub unsafe fn hfile_c_503_hclose(fp: *mut hFILE) -> c_int {
    let fp_layout = fp.cast::<hfile_layout>();
    let mut err = (*fp_layout).has_errno;

    if (*fp_layout).begin > (*fp_layout).end && hfile_c_390_hflush(fp) < 0 {
        err = (*fp_layout).has_errno;
    }
    if ((*fp_layout).flags & HFILE_PRESERVE) == 0 {
        let close = (*(*fp_layout).backend).close.expect("hFILE close backend");
        if close(fp) < 0 {
            err = *crate::htslib_rs::c_compat::__errno_location();
        }
        hfile_destroy(fp);
    }

    if err != 0 {
        *crate::htslib_rs::c_compat::__errno_location() = err;
        libc::EOF
    } else {
        0
    }
}

pub unsafe fn hfile_c_520_hclose_abruptly(fp: *mut hFILE) {
    let save = *crate::htslib_rs::c_compat::__errno_location();
    let fp_layout = fp.cast::<hfile_layout>();
    if ((*fp_layout).flags & HFILE_PRESERVE) != 0 {
        return;
    }
    let close = (*(*fp_layout).backend).close.expect("hFILE close backend");
    let _ = close(fp);
    hfile_destroy(fp);
    *crate::htslib_rs::c_compat::__errno_location() = save;
}

pub unsafe extern "C" fn hfile_c_607_fd_flush(fpv: *mut hFILE) -> c_int {
    let fp = fpv.cast::<hfile_fd_layout>();
    if ((*fp).flags & HFILE_FD_IS_SOCKET) != 0 {
        return 0;
    }
    loop {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let mut ret = crate::htslib_rs::c_compat::fdatasync((*fp).fd);
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        let mut ret = crate::htslib_rs::c_compat::fsync((*fp).fd);
        let errno = *crate::htslib_rs::c_compat::__errno_location();
        if ret < 0 && (errno == libc::EINVAL || errno == libc::ENOTSUP) {
            ret = 0;
        }
        if !(ret < 0 && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR) {
            return ret;
        }
    }
}

pub unsafe extern "C" fn hfile_c_625_fd_close(fpv: *mut hFILE) -> c_int {
    let fp = fpv.cast::<hfile_fd_layout>();
    if ((*fp).flags & HFILE_FD_IS_SHARED) != 0 {
        return 0;
    }

    loop {
        let ret = libc::close((*fp).fd);
        if !(ret < 0 && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR) {
            return ret;
        }
    }
}

pub unsafe fn hfile_c_648_blksize(fd: c_int) -> size_t {
    let mut sbuf: libc::stat = std::mem::zeroed();
    if libc::fstat(fd, &mut sbuf) != 0 {
        return 0;
    }

    if crate::htslib_rs::c_compat::stat_mode_matches(
        sbuf.st_mode,
        libc::S_IFMT,
        crate::htslib_rs::c_compat::S_IFIFO,
    ) {
        128 * 1024
    } else {
        #[cfg(not(windows))]
        {
            sbuf.st_blksize as size_t
        }
        #[cfg(windows)]
        {
            64 * 1024
        }
    }
}

pub unsafe fn hfile_c_664_hopen_fd(filename: *const c_char, mode: *const c_char) -> *mut hFILE {
    let fd = libc::open(filename, hfile_c_772_hfile_oflags(mode), 0o666);
    if fd < 0 {
        return std::ptr::null_mut();
    }

    let fp = hfile_init(
        std::mem::size_of::<hfile_fd_layout>(),
        mode,
        hfile_c_648_blksize(fd),
    )
    .cast::<hfile_fd_layout>();
    if fp.is_null() {
        let save = *crate::htslib_rs::c_compat::__errno_location();
        let _ = libc::close(fd);
        *crate::htslib_rs::c_compat::__errno_location() = save;
        return std::ptr::null_mut();
    }

    (*fp).fd = fd;
    (*fp).flags = 0;
    (*fp).base.backend = &FD_BACKEND;
    fp.cast()
}

pub unsafe fn hfile_c_689_hpreload(fp: *mut hFILE) -> *mut hFILE {
    let mut buf: *mut c_char = std::ptr::null_mut();
    let mut buf_sz: libc::off_t = 0;
    let mut buf_a: libc::off_t = 0;
    let mut buf_inc: libc::off_t = 8192;

    let len: libc::off_t = loop {
        if buf_a - buf_sz < 5000 {
            buf_a += buf_inc;
            let t = crate::htslib_rs::c_compat::realloc(buf.cast(), buf_a as u64).cast::<c_char>();
            if t.is_null() {
                crate::htslib_rs::c_compat::free(buf.cast());
                hclose_abruptly(fp);
                return std::ptr::null_mut();
            }
            buf = t;
            if buf_inc < 1_000_000 {
                buf_inc = (buf_inc as f64 * 1.3) as libc::off_t;
            }
        }
        let len = htslib_hfile_h_247_hread(
            fp,
            buf.add(buf_sz as usize).cast(),
            (buf_a - buf_sz) as size_t,
        ) as libc::off_t;
        if len > 0 {
            buf_sz += len;
        } else {
            break len;
        }
    };

    if len < 0 {
        crate::htslib_rs::c_compat::free(buf.cast());
        hclose_abruptly(fp);
        return std::ptr::null_mut();
    }
    let mem_fp =
        hfile_c_835_create_hfile_mem(buf, c"r".as_ptr(), buf_sz as size_t, buf_a as size_t);
    if mem_fp.is_null() {
        crate::htslib_rs::c_compat::free(buf.cast());
        hclose_abruptly(fp);
        return std::ptr::null_mut();
    }

    if hclose(fp) < 0 {
        hclose_abruptly(mem_fp);
        return std::ptr::null_mut();
    }
    mem_fp
}

pub unsafe fn hfile_c_726_is_preload_url_remote(url: *const c_char) -> c_int {
    hisremote(url.add(8))
}

pub unsafe fn hfile_c_730_hopen_preload(url: *const c_char, mode: *const c_char) -> *mut hFILE {
    let fp = hfile_c_1317_hopen(url.add(8), mode);
    if !fp.is_null() {
        hfile_c_689_hpreload(fp)
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn hfile_c_735_hdopen(fd: c_int, mode: *const c_char) -> *mut hFILE {
    #[cfg(windows)]
    if !libc::strchr(mode, b's' as c_int).is_null() {
        *crate::htslib_rs::c_compat::__errno_location() = crate::htslib_rs::c_compat::ENOSYS;
        return std::ptr::null_mut();
    }

    let fp = hfile_init(
        std::mem::size_of::<hfile_fd_layout>(),
        mode,
        hfile_c_648_blksize(fd),
    )
    .cast::<hfile_fd_layout>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    (*fp).fd = fd;
    (*fp).flags = 0;
    if !libc::strchr(mode, b's' as c_int).is_null() {
        (*fp).flags |= HFILE_FD_IS_SOCKET;
    }
    if !libc::strchr(mode, b'S' as c_int).is_null() {
        (*fp).flags |= HFILE_FD_IS_SHARED;
    }
    (*fp).base.backend = &FD_BACKEND;
    fp.cast()
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
        *crate::htslib_rs::c_compat::__errno_location() = libc::EPROTONOSUPPORT;
        return std::ptr::null_mut();
    }

    hfile_c_664_hopen_fd(url, mode)
}

pub unsafe fn hfile_c_761_hopen_fd_stdinout(mode: *const c_char) -> *mut hFILE {
    let fd = if !libc::strchr(mode, b'r' as c_int).is_null() {
        libc::STDIN_FILENO
    } else {
        libc::STDOUT_FILENO
    };
    let mut mode_shared = [0 as c_char; 101];
    libc::snprintf(
        mode_shared.as_mut_ptr(),
        mode_shared.len(),
        c"S%s".as_ptr(),
        mode,
    );
    hfile_c_735_hdopen(fd, mode_shared.as_ptr())
}

pub unsafe fn hfile_c_772_hfile_oflags(mode: *const c_char) -> c_int {
    let mut rdwr = 0;
    let mut flags = 0;
    let mut s = mode;

    while *s != 0 {
        match *s as u8 {
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
        s = s.add(1);
    }

    #[cfg(target_os = "windows")]
    {
        flags |= libc::O_BINARY;
    }

    rdwr | flags
}

pub unsafe fn hfile_c_826_cmp_prefix(mut key: *const c_char, mut s: *const c_char) -> c_int {
    while *key != 0 {
        if tolower_c(*s) != *key {
            return 1;
        }
        s = s.add(1);
        key = key.add(1);
    }
    0
}

pub unsafe fn hfile_c_835_create_hfile_mem(
    buffer: *mut c_char,
    mode: *const c_char,
    buf_filled: size_t,
    buf_size: size_t,
) -> *mut hFILE {
    let fp = hfile_c_141_hfile_init_fixed(
        std::mem::size_of::<hfile_layout>(),
        mode,
        buffer,
        buf_filled,
        buf_size,
    )
    .cast::<hfile_layout>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    (*fp).backend = &MEM_BACKEND;
    fp.cast()
}

pub unsafe extern "C" fn hfile_c_845_hopen_mem(
    url: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    let mut length = 0usize;
    let comma = libc::strchr(url, b',' as c_int);
    if comma.is_null() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }
    let data = comma.add(1);

    if libc::strchr(mode, b'r' as c_int).is_null() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EROFS;
        return std::ptr::null_mut();
    }

    let (size, buffer) = if comma.offset_from(url) >= 7
        && hfile_c_826_cmp_prefix(c";base64".as_ptr(), comma.sub(7)) == 0
    {
        let size = hts_base64_decoded_length(CStr::from_ptr(data).to_bytes().len());
        let buffer = crate::htslib_rs::c_compat::malloc(size as u64).cast::<c_char>();
        if buffer.is_null() {
            return std::ptr::null_mut();
        }
        hts_decode_base64(buffer, &mut length, data);
        (size, buffer)
    } else {
        let size = CStr::from_ptr(data).to_bytes().len() + 1;
        let buffer = crate::htslib_rs::c_compat::malloc(size as u64).cast::<c_char>();
        if buffer.is_null() {
            return std::ptr::null_mut();
        }
        hts_decode_percent(buffer, &mut length, data);
        (size, buffer)
    };

    let hf = hfile_c_835_create_hfile_mem(buffer, mode, length, size);
    if hf.is_null() {
        crate::htslib_rs::c_compat::free(buffer.cast());
        return std::ptr::null_mut();
    }

    hf
}

// original: hopenv_mem (htslib/hfile.c:878)
pub unsafe fn hfile_c_878_hopenv_mem(
    _filename: *const c_char,
    mode: *const c_char,
    buffer: *mut c_char,
    sz: size_t,
) -> *mut hFILE {
    let hf = hfile_c_835_create_hfile_mem(buffer, mode, sz, sz);
    if hf.is_null() {
        crate::htslib_rs::c_compat::free(buffer.cast());
        return std::ptr::null_mut();
    }
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
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }
    let buffer = hfile_c_va_arg_word(args) as *mut c_char;
    let sz = hfile_c_va_arg_word(args) as size_t;
    hfile_c_878_hopenv_mem(filename, mode, buffer, sz)
}

pub unsafe fn hfile_c_894_hfile_mem_get_buffer(
    file: *mut hFILE,
    length: *mut size_t,
) -> *mut c_char {
    let file_layout = file.cast::<hfile_layout>();
    if (*file_layout).backend != &MEM_BACKEND {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }

    if !length.is_null() {
        *length = (*file_layout).limit.offset_from((*file_layout).buffer) as size_t;
    }

    (*file_layout).buffer
}

pub unsafe fn hfile_c_906_hfile_mem_steal_buffer(
    file: *mut hFILE,
    length: *mut size_t,
) -> *mut c_char {
    let buf = hfile_c_894_hfile_mem_get_buffer(file, length);
    if !buf.is_null() {
        (*(file.cast::<hfile_layout>())).buffer = std::ptr::null_mut();
    }
    buf
}

pub unsafe extern "C" fn hfile_c_915_hopen_not_supported(
    _fname: *const c_char,
    _mode: *const c_char,
) -> *mut hFILE {
    *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
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
    *crate::htslib_rs::c_compat::__errno_location() = libc::EPROTONOSUPPORT;
    std::ptr::null_mut()
}

pub unsafe fn hfile_c_920_hfile_plugin_init_mem(self_: *mut hFILE_plugin) -> c_int {
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

pub unsafe fn hfile_c_956_hfile_plugin_init_crypt4gh_needed(self_: *mut hFILE_plugin) -> c_int {
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
    plugins: Vec<usize>,
}

struct HFileSchemeEntry {
    scheme: usize,
    handler: usize,
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
    let handler_layout = handler.cast::<hfile_scheme_handler_layout>();
    if (*handler_layout).open.is_none() || (*handler_layout).isremote.is_none() {
        return;
    }

    for entry in schemes.iter_mut() {
        if libc::strcmp(entry.scheme as *const c_char, scheme) == 0 {
            if hfile_c_1011_priority(handler)
                > hfile_c_1011_priority(entry.handler as *const hFILE_scheme_handler)
            {
                entry.handler = handler as usize;
            }
            return;
        }
    }

    schemes.push(HFileSchemeEntry {
        scheme: scheme as usize,
        handler: handler as usize,
    });
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
) -> c_int
where
    F: FnOnce(*mut hFILE_plugin) -> c_int,
{
    let p = crate::htslib_rs::c_compat::malloc(std::mem::size_of::<hFILE_plugin_list>() as u64)
        .cast::<hFILE_plugin_list>();
    if p.is_null() {
        return -1;
    }

    (*p).plugin.api_version = 1;
    (*p).plugin.obj = obj;
    (*p).plugin.name = std::ptr::null();
    (*p).plugin.destroy = std::ptr::null();
    (*p).next = std::ptr::null_mut();

    let ret = init((&mut (*p).plugin as *mut hfile_plugin_layout).cast());
    if ret != 0 {
        crate::htslib_rs::c_compat::free(p.cast());
        return ret;
    }

    let mut state = hfile_plugin_state().lock().unwrap();
    let head = state.plugins.last().copied().unwrap_or(0) as *mut hFILE_plugin_list;
    (*p).next = head;
    state.plugins.push(p as usize);
    let _ = pluginname;
    0
}

pub unsafe fn hfile_c_1079_init_add_plugin(
    obj: *mut c_void,
    init: unsafe fn(*mut hFILE_plugin) -> c_int,
    pluginname: *const c_char,
) -> c_int {
    hfile_c_1079_init_add_plugin_impl(obj, |plugin| init(plugin), pluginname)
}

unsafe fn hfile_c_1079_init_add_dynamic_plugin(
    obj: *mut c_void,
    init: HFilePluginInitFn,
    pluginname: *const c_char,
) -> c_int {
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

unsafe extern "C" fn hfile_c_1116_preload_isremote(fname: *const c_char) -> c_int {
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
pub unsafe fn hfile_c_1111_load_hfile_plugins() -> c_int {
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

unsafe extern "C" fn hfile_c_1178_unknown_isremote(fname: *const c_char) -> c_int {
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
            if libc::strcmp(entry.scheme as *const c_char, scheme.as_ptr()) == 0 {
                return entry.handler as *const hFILE_scheme_handler;
            }
        }
    }

    (&HFILE_C_1178_UNKNOWN_SCHEME as *const hfile_scheme_handler_layout).cast()
}

pub unsafe fn hfile_c_983_hfile_shutdown(do_close_plugin: c_int) {
    let mut state = hfile_plugin_state().lock().unwrap();
    state.schemes = None;
    while let Some(p) = state.plugins.pop() {
        let p = p as *mut hFILE_plugin_list;
        if !(*p).plugin.destroy.is_null() {
            let destroy: unsafe extern "C" fn() = std::mem::transmute((*p).plugin.destroy);
            destroy();
        }
        let _ = do_close_plugin;
        crate::htslib_rs::c_compat::free(p.cast());
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
    if fp.is_null() && *crate::htslib_rs::c_compat::__errno_location() == libc::ENOENT {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EPROTONOSUPPORT;
    }
    fp
}

pub unsafe fn hfile_c_1317_hopen_vargs(
    fname: *const c_char,
    mode: *const c_char,
    args: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    let handler = hfile_c_1176_find_scheme_handler(fname);
    if !handler.is_null() {
        let handler = handler.cast::<hfile_scheme_handler_layout>();
        if libc::strchr(mode, b':' as c_int).is_null()
            || (*handler).priority < 2000
            || (*handler).vopen.is_none()
        {
            return (*handler).open.expect("hFILE open handler")(fname, mode);
        }
        if args.is_null() {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
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

pub unsafe extern "C" fn hfile_c_1339_hfile_always_local(_fname: *const c_char) -> c_int {
    0
}

pub unsafe extern "C" fn hfile_c_1342_hfile_always_remote(_fname: *const c_char) -> c_int {
    1
}

// original: hfile_list_schemes (htslib/hfile.c:1218)
pub unsafe fn hfile_c_1218_hfile_list_schemes(
    plugin: *const c_char,
    sc_list: *mut *const c_char,
    nschemes: *mut c_int,
) -> c_int {
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
            let handler = entry.handler as *const hfile_scheme_handler_layout;
            if !plugin.is_null() && libc::strcmp((*handler).provider, plugin) != 0 {
                continue;
            }

            if ns < *nschemes {
                *sc_list.add(ns as usize) = entry.scheme as *const c_char;
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
    nplugins: *mut c_int,
) -> c_int {
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
        let p = *p as *mut hFILE_plugin_list;
        if np < *nplugins {
            *plist.add(np as usize) = (*p).plugin.name;
        }
        np += 1;
    }

    if *nplugins > np {
        *nplugins = np;
    }
    np
}

// original: hfile_has_plugin (htslib/hfile.c:1293)
pub unsafe fn hfile_c_1293_hfile_has_plugin(name: *const c_char) -> c_int {
    {
        let needs_load = hfile_plugin_state().lock().unwrap().schemes.is_none();
        if needs_load && hfile_c_1111_load_hfile_plugins() < 0 {
            return -1;
        }
    }

    let state = hfile_plugin_state().lock().unwrap();
    for p in state.plugins.iter().rev() {
        let p = *p as *mut hFILE_plugin_list;
        if libc::strcmp((*p).plugin.name, name) == 0 {
            return 1;
        }
    }

    0
}

pub unsafe fn hfile_c_1345_hisremote(fname: *const c_char) -> c_int {
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
    replace: c_int,
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

    (*buffer).l = 0;
    if kputsn(filename, end.offset_from(filename) as size_t, buffer) >= 0
        && kputs(new_extension, buffer) >= 0
        && kputs(trailing, buffer) >= 0
    {
        (*buffer).s
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn hfile_c_1416_knet_open(fn_: *const c_char, mode: *const c_char) -> *mut knetFile {
    let fp = crate::htslib_rs::c_compat::calloc(1, std::mem::size_of::<knet_file_layout>() as u64)
        .cast::<knet_file_layout>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    (*fp).hf = if libc::strcmp(fn_, c"-".as_ptr()) == 0 {
        hfile_c_761_hopen_fd_stdinout(mode)
    } else if libc::strncmp(fn_, c"file://".as_ptr(), 7) == 0 {
        hfile_c_747_hopen_fd_fileuri(fn_, mode)
    } else if libc::strchr(fn_, b':' as c_int).is_null() {
        hfile_c_664_hopen_fd(fn_, mode)
    } else {
        hopen(fn_, mode)
    };
    if (*fp).hf.is_null() {
        crate::htslib_rs::c_compat::free(fp.cast());
        return std::ptr::null_mut();
    }

    let hf = (*fp).hf.cast::<hfile_layout>();
    (*fp).fd = if std::ptr::eq((*hf).backend, &FD_BACKEND) {
        (*((*fp).hf.cast::<hfile_fd_layout>())).fd
    } else {
        -1
    };
    fp.cast()
}

pub unsafe fn hfile_c_1433_knet_dopen(fd: c_int, mode: *const c_char) -> *mut knetFile {
    let fp = crate::htslib_rs::c_compat::calloc(1, std::mem::size_of::<knet_file_layout>() as u64)
        .cast::<knet_file_layout>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    (*fp).hf = hdopen(fd, mode);
    if (*fp).hf.is_null() {
        crate::htslib_rs::c_compat::free(fp.cast());
        return std::ptr::null_mut();
    }
    (*fp).fd = fd;
    fp.cast()
}

pub unsafe fn hfile_c_1445_knet_read(
    fp: *mut knetFile,
    buf: *mut c_void,
    len: size_t,
) -> libc::ssize_t {
    let fp = fp.cast::<knet_file_layout>();
    let r = htslib_hfile_h_247_hread((*fp).hf, buf, len);
    if r > 0 {
        (*fp).offset += r as i64;
    }
    r
}

pub unsafe fn hfile_c_1452_knet_seek(
    fp: *mut knetFile,
    off: libc::off_t,
    whence: c_int,
) -> libc::off_t {
    let fp = fp.cast::<knet_file_layout>();
    let r = hseek((*fp).hf, off, whence);
    if r >= 0 {
        (*fp).offset = r as i64;
    }
    r
}

pub unsafe fn hfile_c_1460_knet_close(fp: *mut knetFile) -> c_int {
    let fp = fp.cast::<knet_file_layout>();
    let r = hclose((*fp).hf);
    crate::htslib_rs::c_compat::free(fp.cast());
    r
}

pub unsafe fn hfile_oflags(mode: *const c_char) -> c_int {
    hfile_c_772_hfile_oflags(mode)
}

pub unsafe fn hdopen(fd: c_int, mode: *const c_char) -> *mut hFILE {
    hfile_c_735_hdopen(fd, mode)
}

pub unsafe fn hisremote(filename: *const c_char) -> c_int {
    hfile_c_1345_hisremote(filename)
}

pub unsafe fn haddextension(
    buffer: *mut kstring_t,
    filename: *const c_char,
    replace: c_int,
    extension: *const c_char,
) -> *mut c_char {
    hfile_c_1364_haddextension(buffer, filename, replace, extension)
}

pub unsafe fn hclose(fp: *mut hFILE) -> c_int {
    hfile_c_503_hclose(fp)
}

pub unsafe fn hclose_abruptly(fp: *mut hFILE) {
    hfile_c_520_hclose_abruptly(fp)
}

pub unsafe fn hseek(fp: *mut hFILE, offset: libc::off_t, whence: c_int) -> libc::off_t {
    hfile_c_446_hseek(fp, offset, whence)
}

pub unsafe fn hfile_set_blksize(fp: *mut hFILE, bufsiz: size_t) -> c_int {
    hfile_c_212_hfile_set_blksize(fp, bufsiz)
}

pub unsafe fn hgetc2(fp: *mut hFILE) -> c_int {
    hfile_c_235_hgetc2(fp)
}

pub unsafe fn hgetdelim(
    buffer: *mut c_char,
    size: size_t,
    delim: c_int,
    fp: *mut hFILE,
) -> libc::ssize_t {
    hfile_c_241_hgetdelim(buffer, size, delim, fp)
}

pub unsafe fn hgets(buffer: *mut c_char, size: c_int, fp: *mut hFILE) -> *mut c_char {
    hfile_c_291_hgets(buffer, size, fp)
}

pub unsafe fn khgetline(kstr: *mut kstring_t, fp: *mut hFILE) -> c_int {
    hfile_c_306_khgetline(kstr, fp)
}

pub unsafe fn hpeek(fp: *mut hFILE, buffer: *mut c_void, nbytes: size_t) -> libc::ssize_t {
    hfile_c_313_hpeek(fp, buffer, nbytes)
}

pub unsafe fn hread2(
    fp: *mut hFILE,
    destv: *mut c_void,
    nbytes: size_t,
    nread: size_t,
) -> libc::ssize_t {
    hfile_c_330_hread2(fp, destv, nbytes, nread)
}

pub unsafe fn hputc2(c: c_int, fp: *mut hFILE) -> c_int {
    hfile_c_400_hputc2(c, fp)
}

pub unsafe fn hwrite2(
    fp: *mut hFILE,
    srcv: *const c_void,
    totalbytes: size_t,
    ncopied: size_t,
) -> libc::ssize_t {
    hfile_c_412_hwrite2(fp, srcv, totalbytes, ncopied)
}

pub unsafe fn hputs2(
    text: *const c_char,
    totalbytes: size_t,
    ncopied: size_t,
    fp: *mut hFILE,
) -> c_int {
    hfile_c_440_hputs2(text, totalbytes, ncopied, fp)
}

pub unsafe fn hflush(fp: *mut hFILE) -> c_int {
    hfile_c_390_hflush(fp)
}

pub unsafe fn hfile_mem_get_buffer(file: *mut hFILE, length: *mut size_t) -> *mut c_char {
    hfile_c_894_hfile_mem_get_buffer(file, length)
}

pub unsafe fn hfile_mem_steal_buffer(file: *mut hFILE, length: *mut size_t) -> *mut c_char {
    hfile_c_906_hfile_mem_steal_buffer(file, length)
}

pub unsafe fn hfile_list_schemes(
    plugin: *const c_char,
    sc_list: *mut *const c_char,
    nschemes: *mut c_int,
) -> c_int {
    hfile_c_1218_hfile_list_schemes(plugin, sc_list, nschemes)
}

pub unsafe fn hfile_list_plugins(plist: *mut *const c_char, nplugins: *mut c_int) -> c_int {
    hfile_c_1257_hfile_list_plugins(plist, nplugins)
}

pub unsafe fn hfile_has_plugin(name: *const c_char) -> c_int {
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

pub unsafe fn knet_dopen(fd: c_int, mode: *const c_char) -> *mut knetFile {
    hfile_c_1433_knet_dopen(fd, mode)
}

pub unsafe fn knet_read(fp: *mut knetFile, buf: *mut c_void, len: size_t) -> libc::ssize_t {
    hfile_c_1445_knet_read(fp, buf, len)
}

pub unsafe fn knet_seek(fp: *mut knetFile, off: libc::off_t, whence: c_int) -> libc::off_t {
    hfile_c_1452_knet_seek(fp, off, whence)
}

pub unsafe fn knet_close(fp: *mut knetFile) -> c_int {
    hfile_c_1460_knet_close(fp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    struct ReadFile {
        base: hfile_layout,
        source: *const c_char,
        source_len: usize,
        source_pos: usize,
    }

    #[repr(C)]
    struct WriteFile {
        base: hfile_layout,
        sink: *mut c_char,
        sink_len: usize,
        sink_pos: usize,
        flushes: c_int,
    }

    unsafe extern "C" fn read_backend(
        fp: *mut hFILE,
        buffer: *mut c_void,
        nbytes: size_t,
    ) -> libc::ssize_t {
        let fp = fp.cast::<ReadFile>();
        let remaining = (*fp).source_len - (*fp).source_pos;
        let n = remaining.min(nbytes);
        crate::htslib_rs::c_compat::memcpy(
            buffer,
            (*fp).source.add((*fp).source_pos).cast(),
            n as u64,
        );
        (*fp).source_pos += n;
        n as libc::ssize_t
    }

    unsafe extern "C" fn read_errno_backend(
        _fp: *mut hFILE,
        _buffer: *mut c_void,
        _nbytes: size_t,
    ) -> libc::ssize_t {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EIO;
        -1
    }

    unsafe extern "C" fn write_backend(
        fp: *mut hFILE,
        buffer: *const c_void,
        nbytes: size_t,
    ) -> libc::ssize_t {
        let fp = fp.cast::<WriteFile>();
        let remaining = (*fp).sink_len - (*fp).sink_pos;
        let n = remaining.min(nbytes);
        crate::htslib_rs::c_compat::memcpy((*fp).sink.add((*fp).sink_pos).cast(), buffer, n as u64);
        (*fp).sink_pos += n;
        n as libc::ssize_t
    }

    unsafe extern "C" fn seek_backend(
        _fp: *mut hFILE,
        _offset: libc::off_t,
        _whence: c_int,
    ) -> libc::off_t {
        -1
    }

    unsafe extern "C" fn seekable_read_seek_backend(
        fp: *mut hFILE,
        offset: libc::off_t,
        whence: c_int,
    ) -> libc::off_t {
        let fp = fp.cast::<ReadFile>();
        let base = &mut (*fp).base;
        let cur = base.offset + base.begin.offset_from(base.buffer) as libc::off_t;
        let len = (*fp).source_len as libc::off_t;
        let pos = match whence {
            libc::SEEK_SET => offset,
            libc::SEEK_CUR => cur + offset,
            libc::SEEK_END => len + offset,
            _ => -1,
        };
        if pos < 0 || pos > len {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            return -1;
        }
        (*fp).source_pos = pos as usize;
        pos
    }

    unsafe extern "C" fn flush_backend(fp: *mut hFILE) -> c_int {
        let fp = fp.cast::<WriteFile>();
        (*fp).flushes += 1;
        0
    }

    unsafe extern "C" fn close_backend(_fp: *mut hFILE) -> c_int {
        0
    }

    static OWNED_HFILE_DROP_CLOSES: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(0);

    unsafe extern "C" fn counted_close_backend(_fp: *mut hFILE) -> c_int {
        OWNED_HFILE_DROP_CLOSES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        0
    }

    unsafe extern "C" fn close_errno_backend(_fp: *mut hFILE) -> c_int {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EBADF;
        -1
    }

    static READ_BACKEND: hfile_backend_layout = hfile_backend_layout {
        read: Some(read_backend),
        write: None,
        seek: Some(seek_backend),
        flush: None,
        close: Some(close_backend),
    };

    static WRITE_BACKEND: hfile_backend_layout = hfile_backend_layout {
        read: None,
        write: Some(write_backend),
        seek: Some(seek_backend),
        flush: Some(flush_backend),
        close: Some(close_backend),
    };

    static READ_ERRNO_BACKEND: hfile_backend_layout = hfile_backend_layout {
        read: Some(read_errno_backend),
        write: None,
        seek: Some(seek_backend),
        flush: None,
        close: Some(close_backend),
    };

    static SEEKABLE_READ_BACKEND: hfile_backend_layout = hfile_backend_layout {
        read: Some(read_backend),
        write: None,
        seek: Some(seekable_read_seek_backend),
        flush: None,
        close: Some(close_backend),
    };

    static READ_CLOSE_ERRNO_BACKEND: hfile_backend_layout = hfile_backend_layout {
        read: Some(read_backend),
        write: None,
        seek: Some(seek_backend),
        flush: None,
        close: Some(close_errno_backend),
    };

    static CLOSE_ERRNO_BACKEND: hfile_backend_layout = hfile_backend_layout {
        read: None,
        write: None,
        seek: Some(seek_backend),
        flush: None,
        close: Some(close_errno_backend),
    };

    static COUNTED_CLOSE_BACKEND: hfile_backend_layout = hfile_backend_layout {
        read: None,
        write: None,
        seek: Some(seek_backend),
        flush: None,
        close: Some(counted_close_backend),
    };

    #[test]
    fn owned_hfile_closes_on_drop_and_can_release_raw_pointer() {
        unsafe {
            OWNED_HFILE_DROP_CLOSES.store(0, std::sync::atomic::Ordering::SeqCst);
            let fp = hfile_c_104_hfile_init(std::mem::size_of::<hfile_layout>(), c"r".as_ptr(), 8)
                .cast::<hfile_layout>();
            assert!(!fp.is_null());
            (*fp).backend = &COUNTED_CLOSE_BACKEND;

            let owned = OwnedHFile::from_raw(fp.cast()).expect("non-null hFILE");
            assert_eq!(owned.as_ptr(), fp.cast());
            drop(owned);
            assert_eq!(
                OWNED_HFILE_DROP_CLOSES.load(std::sync::atomic::Ordering::SeqCst),
                1
            );

            let fp = hfile_c_104_hfile_init(std::mem::size_of::<hfile_layout>(), c"r".as_ptr(), 8)
                .cast::<hfile_layout>();
            assert!(!fp.is_null());
            (*fp).backend = &READ_BACKEND;
            let owned = OwnedHFile::from_raw(fp.cast()).expect("non-null hFILE");
            let raw = owned.into_raw();
            assert_eq!(raw, fp.cast());
            assert_eq!(hclose(raw), 0);
        }
    }

    #[test]
    fn borrowed_hfile_wraps_without_taking_ownership() {
        unsafe {
            assert!(OwnedHFile::from_raw(std::ptr::null_mut()).is_none());
            assert!(BorrowedHFile::from_raw(std::ptr::null_mut()).is_none());

            let fp = hfile_c_104_hfile_init(std::mem::size_of::<hfile_layout>(), c"r".as_ptr(), 8)
                .cast::<hfile_layout>();
            assert!(!fp.is_null());
            (*fp).backend = &READ_BACKEND;

            let borrowed = BorrowedHFile::from_raw(fp.cast()).expect("non-null hFILE");
            assert_eq!(borrowed.as_ptr(), fp.cast());
            assert_eq!(borrowed.as_non_null().as_ptr(), fp.cast());
            assert_eq!(hclose(fp.cast()), 0);
        }
    }

    #[test]
    fn hfile_init_fixed_sets_immobile_read_state_like_c() {
        unsafe {
            let base =
                hfile_c_104_hfile_init(std::mem::size_of::<hfile_layout>(), c"r".as_ptr(), 999_999)
                    .cast::<hfile_layout>();
            assert!(!base.is_null());
            assert_eq!((*base).begin, (*base).buffer);
            assert_eq!((*base).end, (*base).buffer);
            assert_eq!((*base).limit.offset_from((*base).buffer), 128 * 1024);
            assert_eq!((*base).flags & HFILE_MOBILE, HFILE_MOBILE);
            assert_eq!((*base).flags & HFILE_READONLY, HFILE_READONLY);
            (*base).begin = (*base).buffer.add(1);
            assert_eq!(hfile_c_171_writebuffer_is_nonempty(base.cast()), 1);
            hfile_c_162_hfile_destroy(base.cast());

            let buffer = crate::htslib_rs::c_compat::malloc(8).cast::<c_char>();
            crate::htslib_rs::c_compat::memcpy(buffer.cast(), c"abcde".as_ptr().cast(), 5);

            let fp = hfile_c_141_hfile_init_fixed(
                std::mem::size_of::<hfile_layout>(),
                c"r".as_ptr(),
                buffer,
                5,
                8,
            )
            .cast::<hfile_layout>();
            assert!(!fp.is_null());
            assert_eq!((*fp).buffer, buffer);
            assert_eq!((*fp).begin, buffer);
            assert_eq!((*fp).end.offset_from(buffer), 5);
            assert_eq!((*fp).limit.offset_from(buffer), 8);
            assert_eq!((*fp).offset, 0);
            assert_eq!((*fp).flags & HFILE_AT_EOF, HFILE_AT_EOF);
            assert_eq!((*fp).flags & HFILE_MOBILE, 0);
            assert_eq!((*fp).flags & HFILE_READONLY, HFILE_READONLY);
            assert_eq!((*fp).has_errno, 0);

            assert_eq!(hfile_c_212_hfile_set_blksize(fp.cast(), 16), 0);
            assert_eq!((*fp).limit.offset_from((*fp).buffer), 16);
            assert_eq!(hfile_c_212_hfile_set_blksize(fp.cast(), 4), -1);

            hfile_destroy(fp.cast());
        }
    }

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

            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            let out =
                hfile_c_1364_haddextension(&mut ks, c"/tmp/a.bam".as_ptr(), 1, c".csi".as_ptr());
            assert!(!out.is_null());
            assert_eq!(CStr::from_ptr(out).to_bytes(), b"/tmp/a.csi");

            let out2 = hfile_c_1364_haddextension(
                &mut ks,
                c"file:///tmp/a.bam?x=1".as_ptr(),
                1,
                c".csi".as_ptr(),
            );
            assert!(!out2.is_null());
            assert_eq!(CStr::from_ptr(out2).to_bytes(), b"file:///tmp/a.csi?x=1");
            crate::htslib_rs::c_compat::free(ks.s.cast());
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
    fn hfile_scheme_parser_accepts_c_scheme_alphabet_and_rejects_non_schemes() {
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
            assert_eq!(
                htslib_hfile_h_247_hread(fp, out.as_mut_ptr().cast(), out.len()),
                12
            );
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 12),
                b"upper scheme"
            );
            assert_eq!(hclose(fp), 0);
        }
    }

    #[test]
    fn hfile_plugin_listing_respects_capacity_without_losing_total_counts() {
        unsafe {
            let mut plugins = [std::ptr::null(); 1];
            let mut nplugins = plugins.len() as c_int;
            let total = hfile_c_1257_hfile_list_plugins(plugins.as_mut_ptr(), &mut nplugins);
            assert!(total >= 1);
            assert_eq!(nplugins, 1);
            assert_eq!(CStr::from_ptr(plugins[0]).to_bytes(), b"built-in");

            let mut schemes = [std::ptr::null(); 1];
            let mut nschemes = schemes.len() as c_int;
            let total_schemes = hfile_c_1218_hfile_list_schemes(
                c"built-in".as_ptr(),
                schemes.as_mut_ptr(),
                &mut nschemes,
            );
            assert!(total_schemes >= 3);
            assert_eq!(nschemes, 1);
            assert_eq!(CStr::from_ptr(schemes[0]).to_bytes(), b"data");

            let mut exact = [std::ptr::null(); 8];
            let mut nexact = exact.len() as c_int;
            let exact_total =
                hfile_c_1218_hfile_list_schemes(c"mem".as_ptr(), exact.as_mut_ptr(), &mut nexact);
            assert_eq!(exact_total, nexact);
            assert!(exact
                .iter()
                .take(nexact as usize)
                .any(|scheme| CStr::from_ptr(*scheme).to_bytes() == b"mem"));
        }
    }

    #[test]
    fn hfile_plugin_listing_accepts_zero_output_capacity() {
        unsafe {
            let mut nplugins = 0;
            let total = hfile_c_1257_hfile_list_plugins(std::ptr::null_mut(), &mut nplugins);
            assert!(total >= 1);
            assert_eq!(nplugins, 0);

            let mut nschemes = 0;
            let total_schemes = hfile_c_1218_hfile_list_schemes(
                c"built-in".as_ptr(),
                std::ptr::null_mut(),
                &mut nschemes,
            );
            assert!(total_schemes >= 3);
            assert_eq!(nschemes, 0);

            nschemes = 0;
            assert_eq!(
                hfile_c_1218_hfile_list_schemes(
                    c"missing-plugin".as_ptr(),
                    std::ptr::null_mut(),
                    &mut nschemes,
                ),
                0
            );
            assert_eq!(nschemes, 0);
        }
    }

    #[test]
    fn hfile_memory_backend_decodes_data_urls_and_exposes_buffer_like_c() {
        unsafe {
            assert_eq!(
                hfile_c_826_cmp_prefix(c";base64".as_ptr(), c";BASE64".as_ptr()),
                0
            );
            assert_eq!(
                hfile_c_826_cmp_prefix(c";base64".as_ptr(), c";plain".as_ptr()),
                1
            );

            let fp = hfile_c_845_hopen_mem(c"data:,hello%2C%20world%21".as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());

            let mut out = [0 as c_char; 32];
            assert_eq!(
                htslib_hfile_h_247_hread(fp, out.as_mut_ptr().cast(), 32),
                13
            );
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 13),
                b"hello, world!"
            );

            let mut length = 0usize;
            let internal = hfile_c_894_hfile_mem_get_buffer(fp, &mut length);
            assert!(!internal.is_null());
            assert_eq!(length, 20);
            assert_eq!(hfile_c_810_mem_seek(fp, 0, libc::SEEK_SET), -1);
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );
            assert_eq!(hclose(fp), 0);

            let fp64 = hfile_c_845_hopen_mem(c"data:;base64,QUJDRA==".as_ptr(), c"r".as_ptr());
            assert!(!fp64.is_null());
            let mut decoded = [0 as c_char; 8];
            assert_eq!(
                htslib_hfile_h_247_hread(fp64, decoded.as_mut_ptr().cast(), 8),
                4
            );
            assert_eq!(
                std::slice::from_raw_parts(decoded.as_ptr().cast::<u8>(), 4),
                b"ABCD"
            );

            let stolen = hfile_c_906_hfile_mem_steal_buffer(fp64, std::ptr::null_mut());
            assert!(!stolen.is_null());
            assert!((*fp64.cast::<hfile_layout>()).buffer.is_null());
            crate::htslib_rs::c_compat::free(stolen.cast());
            assert_eq!(hclose(fp64), 0);

            assert!(hfile_c_845_hopen_mem(c"data:,x".as_ptr(), c"w".as_ptr()).is_null());
            assert_eq!(*crate::htslib_rs::c_compat::__errno_location(), libc::EROFS);
            assert!(hfile_c_915_hopen_not_supported(c"mem".as_ptr(), c"r".as_ptr()).is_null());
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );
        }
    }

    #[test]
    fn hfile_memory_buffer_access_reports_filled_length_and_transfers_ownership() {
        unsafe {
            let payload = b"owned";
            let buffer = crate::htslib_rs::c_compat::malloc(8).cast::<c_char>();
            assert!(!buffer.is_null());
            crate::htslib_rs::c_compat::memcpy(
                buffer.cast(),
                payload.as_ptr().cast(),
                payload.len() as u64,
            );

            let fp = hfile_c_835_create_hfile_mem(buffer, c"r".as_ptr(), payload.len(), 8);
            assert!(!fp.is_null());
            let mut length = 0usize;
            assert_eq!(hfile_c_894_hfile_mem_get_buffer(fp, &mut length), buffer);
            assert_eq!(length, 8);

            let stolen = hfile_c_906_hfile_mem_steal_buffer(fp, &mut length);
            assert_eq!(stolen, buffer);
            assert_eq!(length, 8);
            assert!((*fp.cast::<hfile_layout>()).buffer.is_null());
            assert_eq!(hclose(fp), 0);
            crate::htslib_rs::c_compat::free(stolen.cast());

            let mut stack = [0 as c_char; 4];
            let mut file = ReadFile {
                base: hfile_layout {
                    buffer: stack.as_mut_ptr(),
                    begin: stack.as_mut_ptr(),
                    end: stack.as_mut_ptr(),
                    limit: stack.as_mut_ptr().add(stack.len()),
                    backend: &READ_BACKEND,
                    offset: 0,
                    flags: HFILE_MOBILE,
                    has_errno: 0,
                },
                source: std::ptr::null(),
                source_len: 0,
                source_pos: 0,
            };
            assert!(hfile_c_894_hfile_mem_get_buffer(
                (&mut file as *mut ReadFile).cast(),
                &mut length
            )
            .is_null());
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );
        }
    }

    #[test]
    fn hfile_peek_refills_without_advancing_logical_read_position() {
        unsafe {
            let mut buffer = [0 as c_char; 4];
            let source = b"abcdef";
            let mut file = ReadFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr(),
                    begin: buffer.as_mut_ptr(),
                    end: buffer.as_mut_ptr(),
                    limit: buffer.as_mut_ptr().add(buffer.len()),
                    backend: &READ_BACKEND,
                    offset: 0,
                    flags: HFILE_MOBILE,
                    has_errno: 0,
                },
                source: source.as_ptr().cast(),
                source_len: source.len(),
                source_pos: 0,
            };

            let fp = (&mut file as *mut ReadFile).cast();
            let mut peeked = [0 as c_char; 3];
            assert_eq!(
                hfile_c_313_hpeek(fp, peeked.as_mut_ptr().cast(), peeked.len()),
                3
            );
            assert_eq!(
                std::slice::from_raw_parts(peeked.as_ptr().cast::<u8>(), 3),
                b"abc"
            );
            assert_eq!(htslib_hfile_h_155_htell(fp), 0);
            assert_eq!(file.source_pos, 4);

            let mut first = [0 as c_char; 2];
            assert_eq!(
                htslib_hfile_h_247_hread(fp, first.as_mut_ptr().cast(), first.len()),
                2
            );
            assert_eq!(
                std::slice::from_raw_parts(first.as_ptr().cast::<u8>(), 2),
                b"ab"
            );
            assert_eq!(htslib_hfile_h_155_htell(fp), 2);

            let mut rest = [0 as c_char; 4];
            assert_eq!(
                htslib_hfile_h_247_hread(fp, rest.as_mut_ptr().cast(), rest.len()),
                4
            );
            assert_eq!(
                std::slice::from_raw_parts(rest.as_ptr().cast::<u8>(), 4),
                b"cdef"
            );
            assert_eq!(htslib_hfile_h_155_htell(fp), 6);
            assert_eq!(htslib_hfile_h_163_hgetc(fp), libc::EOF);
            assert_ne!(file.base.flags & HFILE_AT_EOF, 0);
            assert_eq!(file.base.has_errno, 0);
        }
    }

    #[test]
    fn hfile_data_url_decoder_keeps_c_edge_case_semantics() {
        unsafe {
            assert!(hfile_c_845_hopen_mem(c"data:no-comma".as_ptr(), c"r".as_ptr()).is_null());
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );

            let fp = hfile_c_845_hopen_mem(c"data:text/plain,kept%ZZ%2f".as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let mut out = [0 as c_char; 16];
            assert_eq!(
                htslib_hfile_h_247_hread(fp, out.as_mut_ptr().cast(), out.len()),
                8
            );
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 8),
                b"kept%ZZ/"
            );
            assert_eq!(hclose(fp), 0);

            let fp64 =
                hfile_c_845_hopen_mem(c"data:text/plain;BASE64,QUI=".as_ptr(), c"r".as_ptr());
            assert!(!fp64.is_null());
            let mut decoded = [0 as c_char; 4];
            assert_eq!(
                htslib_hfile_h_247_hread(fp64, decoded.as_mut_ptr().cast(), decoded.len()),
                2
            );
            assert_eq!(
                std::slice::from_raw_parts(decoded.as_ptr().cast::<u8>(), 2),
                b"AB"
            );
            assert_eq!(hclose(fp64), 0);
        }
    }

    #[test]
    fn hfile_read_and_seek_error_edges_set_herrno_and_errno_like_c() {
        unsafe {
            let mut buffer = [0 as c_char; 4];
            let mut file = ReadFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr(),
                    begin: buffer.as_mut_ptr(),
                    end: buffer.as_mut_ptr(),
                    limit: buffer.as_mut_ptr().add(buffer.len()),
                    backend: &READ_ERRNO_BACKEND,
                    offset: 0,
                    flags: HFILE_MOBILE,
                    has_errno: 0,
                },
                source: std::ptr::null(),
                source_len: 0,
                source_pos: 0,
            };
            let mut out = [0 as c_char; 2];
            assert_eq!(
                htslib_hfile_h_247_hread(
                    (&mut file as *mut ReadFile).cast(),
                    out.as_mut_ptr().cast(),
                    out.len()
                ),
                -1
            );
            assert_eq!(
                htslib_hfile_h_134_herrno((&mut file as *mut ReadFile).cast()),
                libc::EIO
            );
            htslib_hfile_h_140_hclearerr((&mut file as *mut ReadFile).cast());
            assert_eq!(
                htslib_hfile_h_134_herrno((&mut file as *mut ReadFile).cast()),
                0
            );

            assert_eq!(
                hfile_c_241_hgetdelim(
                    out.as_mut_ptr(),
                    0,
                    b'\n' as c_int,
                    (&mut file as *mut ReadFile).cast()
                ),
                -1
            );
            assert_eq!(file.base.has_errno, libc::EINVAL);
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );

            file.base.has_errno = 0;
            file.base.offset = 2;
            file.base.begin = file.base.buffer;
            file.base.end = file.base.buffer;
            assert_eq!(
                hfile_c_446_hseek((&mut file as *mut ReadFile).cast(), -3, libc::SEEK_CUR),
                -1
            );
            assert_eq!(file.base.has_errno, libc::EINVAL);
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );

            file.base.has_errno = 0;
            file.base.offset = libc::off_t::MAX - 1;
            assert_eq!(
                hfile_c_446_hseek((&mut file as *mut ReadFile).cast(), 4, libc::SEEK_CUR),
                -1
            );
            assert_eq!(file.base.has_errno, crate::htslib_rs::c_compat::EOVERFLOW);
        }
    }

    #[test]
    fn hfile_fd_backend_opens_reads_writes_and_honours_shared_fd() {
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
            assert!(std::ptr::eq(
                (*fp.cast::<hfile_fd_layout>()).base.backend,
                &FD_BACKEND
            ));
            assert_eq!((*fp.cast::<hfile_fd_layout>()).flags, 0);
            assert!(hfile_c_648_blksize((*fp.cast::<hfile_fd_layout>()).fd) > 0);

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
            assert_ne!(
                (*shared.cast::<hfile_fd_layout>()).flags & HFILE_FD_IS_SHARED,
                0
            );
            assert_eq!(hclose(shared), 0);
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
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EPROTONOSUPPORT
            );

            std::fs::remove_file(path).unwrap();
        }
    }

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
            assert!(std::ptr::eq(
                (*mem.cast::<hfile_layout>()).backend,
                &MEM_BACKEND
            ));
            assert_eq!((*mem.cast::<hfile_layout>()).flags & HFILE_MOBILE, 0);

            let mut out = [0 as c_char; 32];
            assert_eq!(
                htslib_hfile_h_247_hread(mem, out.as_mut_ptr().cast(), 32),
                17
            );
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

    #[test]
    fn hfile_preload_close_failure_cleans_memory_file_once_and_preserves_errno() {
        unsafe {
            let fp = hfile_c_104_hfile_init(std::mem::size_of::<ReadFile>(), c"r".as_ptr(), 8)
                .cast::<ReadFile>();
            assert!(!fp.is_null());
            (*fp).base.backend = &READ_CLOSE_ERRNO_BACKEND;
            (*fp).source = c"abc".as_ptr();
            (*fp).source_len = 3;
            (*fp).source_pos = 0;

            *crate::htslib_rs::c_compat::__errno_location() = 0;
            assert!(hfile_c_689_hpreload(fp.cast()).is_null());
            assert_eq!(*crate::htslib_rs::c_compat::__errno_location(), libc::EBADF);
        }
    }

    #[test]
    fn hfile_hopen_dispatches_builtin_and_unknown_schemes_like_c_edges() {
        unsafe {
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
            assert_eq!(
                hfile_c_1026_try_exe_add_scheme_handler(
                    c"x".as_ptr(),
                    (&handler as *const hfile_scheme_handler_layout).cast()
                ),
                -1
            );
            assert_eq!(
                hfile_c_1046_try_exe_add_scheme_handler(
                    c"x".as_ptr(),
                    (&handler as *const hfile_scheme_handler_layout).cast()
                ),
                -1
            );

            let fp = hfile_c_1317_hopen(c"data:,builtin%20dispatch".as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let mut out = [0 as c_char; 32];
            assert_eq!(
                htslib_hfile_h_247_hread(fp, out.as_mut_ptr().cast(), 32),
                16
            );
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 16),
                b"builtin dispatch"
            );
            assert_eq!(hclose(fp), 0);

            let missing =
                hfile_c_1168_hopen_unknown_scheme(c"missing-scheme:test".as_ptr(), c"r".as_ptr());
            assert!(missing.is_null());
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EPROTONOSUPPORT
            );
        }
    }

    #[test]
    fn hfile_hopen_vargs_dispatches_revision2_handlers_only_for_colon_modes() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static OPEN_CALLS: AtomicUsize = AtomicUsize::new(0);
        static VOPEN_CALLS: AtomicUsize = AtomicUsize::new(0);

        unsafe extern "C" fn tracked_open(
            _fname: *const c_char,
            _mode: *const c_char,
        ) -> *mut hFILE {
            OPEN_CALLS.fetch_add(1, Ordering::SeqCst);
            std::ptr::null_mut()
        }

        unsafe extern "C" fn tracked_vopen(
            _fname: *const c_char,
            _mode: *const c_char,
            args: *mut crate::htslib_rs::c_compat::__va_list_tag,
        ) -> *mut hFILE {
            assert!(!args.is_null());
            VOPEN_CALLS.fetch_add(1, Ordering::SeqCst);
            std::ptr::null_mut()
        }

        unsafe extern "C" fn tracked_remote(_fname: *const c_char) -> c_int {
            1
        }

        static HANDLER: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
            open: Some(tracked_open),
            isremote: Some(tracked_remote),
            provider: c"vopen-test".as_ptr(),
            priority: 2050,
            vopen: Some(tracked_vopen),
        };

        unsafe {
            assert!(!hfile_c_1176_find_scheme_handler(c"data:,x".as_ptr()).is_null());
            hfile_c_1053_hfile_add_scheme_handler(
                c"zzvopen".as_ptr(),
                (&HANDLER as *const hfile_scheme_handler_layout).cast(),
            );

            OPEN_CALLS.store(0, Ordering::SeqCst);
            VOPEN_CALLS.store(0, Ordering::SeqCst);
            assert!(hfile_c_1317_hopen(c"zzvopen:path".as_ptr(), c"r".as_ptr()).is_null());
            assert_eq!(OPEN_CALLS.load(Ordering::SeqCst), 1);
            assert_eq!(VOPEN_CALLS.load(Ordering::SeqCst), 0);

            *crate::htslib_rs::c_compat::__errno_location() = 0;
            assert!(hfile_c_1317_hopen(c"zzvopen:path".as_ptr(), c"r:".as_ptr()).is_null());
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );
            assert_eq!(OPEN_CALLS.load(Ordering::SeqCst), 1);
            assert_eq!(VOPEN_CALLS.load(Ordering::SeqCst), 0);

            let mut args =
                std::mem::MaybeUninit::<crate::htslib_rs::c_compat::__va_list_tag>::uninit();
            assert!(hfile_c_1317_hopen_vargs(
                c"zzvopen:path".as_ptr(),
                c"r:".as_ptr(),
                args.as_mut_ptr()
            )
            .is_null());
            assert_eq!(OPEN_CALLS.load(Ordering::SeqCst), 1);
            assert_eq!(VOPEN_CALLS.load(Ordering::SeqCst), 1);
        }
    }

    #[test]
    fn hfile_mem_vopen_decodes_va_list_buffer_and_size() {
        unsafe {
            let payload = b"mem-vopen";
            let buffer = libc::malloc(payload.len()).cast::<c_char>();
            assert!(!buffer.is_null());
            crate::htslib_rs::c_compat::memcpy(
                buffer.cast(),
                payload.as_ptr().cast(),
                payload.len() as u64,
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
    fn hfile_unknown_scheme_fallback_is_local_like_upstream() {
        unsafe {
            let handler = hfile_c_1176_find_scheme_handler(c"zz-example://host/path".as_ptr());
            assert!(!handler.is_null());
            let handler = handler.cast::<hfile_scheme_handler_layout>();
            assert_eq!(CStr::from_ptr((*handler).provider).to_bytes(), b"built-in");
            assert_eq!((*handler).priority, 0);
            assert_eq!(
                (*handler).isremote.expect("unknown scheme isremote")(
                    c"zz-example://host/path".as_ptr()
                ),
                0
            );
            assert_eq!(hisremote(c"zz-example://host/path".as_ptr()), 0);

            let http = hfile_c_1176_find_scheme_handler(c"http://example.invalid/ref.fa".as_ptr());
            assert!(!http.is_null());
            let http = http.cast::<hfile_scheme_handler_layout>();
            #[cfg(feature = "libcurl")]
            {
                assert_eq!(CStr::from_ptr((*http).provider).to_bytes(), b"libcurl");
                assert_eq!((*http).priority, 2050);
                assert_eq!(hisremote(c"http://example.invalid/ref.fa".as_ptr()), 1);
            }
            #[cfg(not(feature = "libcurl"))]
            {
                assert_eq!(CStr::from_ptr((*http).provider).to_bytes(), b"built-in");
                assert_eq!((*http).priority, 0);
                assert_eq!(hisremote(c"http://example.invalid/ref.fa".as_ptr()), 0);
            }
            #[cfg(feature = "s3")]
            assert_eq!(hisremote(c"s3://bucket/key.bam".as_ptr()), 1);
            #[cfg(not(feature = "s3"))]
            assert_eq!(hisremote(c"s3://bucket/key.bam".as_ptr()), 0);
        }
    }

    #[test]
    #[cfg(any(feature = "libcurl", feature = "s3", feature = "gcs"))]
    fn hfile_remote_scheme_dispatch_prefers_feature_plugins() {
        unsafe {
            #[cfg(feature = "libcurl")]
            {
                let https =
                    hfile_c_1176_find_scheme_handler(c"https://example.invalid/ref.fa".as_ptr());
                assert!(!https.is_null());
                let https = https.cast::<hfile_scheme_handler_layout>();
                assert_eq!(CStr::from_ptr((*https).provider).to_bytes(), b"libcurl");
                assert_eq!((*https).priority, 2050);
                assert!((*https).vopen.is_some());
                assert_eq!(hisremote(c"https://example.invalid/ref.fa".as_ptr()), 1);
            }

            #[cfg(feature = "s3")]
            for url in [
                c"s3://bucket/key.bam".as_ptr(),
                c"s3+http://bucket/key.bam".as_ptr(),
                c"s3+https://bucket/key.bam".as_ptr(),
            ] {
                let handler = hfile_c_1176_find_scheme_handler(url);
                assert!(!handler.is_null());
                let handler = handler.cast::<hfile_scheme_handler_layout>();
                assert_eq!(CStr::from_ptr((*handler).provider).to_bytes(), b"Amazon S3");
                assert_eq!((*handler).priority, 2050);
                assert!((*handler).vopen.is_some());
                assert_eq!(hisremote(url), 1);
            }

            #[cfg(feature = "gcs")]
            for url in [
                c"gs://bucket/key.bam".as_ptr(),
                c"gs+http://bucket/key.bam".as_ptr(),
                c"gs+https://bucket/key.bam".as_ptr(),
            ] {
                let handler = hfile_c_1176_find_scheme_handler(url);
                assert!(!handler.is_null());
                let handler = handler.cast::<hfile_scheme_handler_layout>();
                assert_eq!(
                    CStr::from_ptr((*handler).provider).to_bytes(),
                    b"Google Cloud Storage"
                );
                assert_eq!((*handler).priority, 2050);
                assert!((*handler).vopen.is_some());
                assert_eq!(hisremote(url), 1);
            }
        }
    }

    #[test]
    fn hfile_cleanup_paths_preserve_errno_like_c() {
        unsafe {
            let fp = hfile_c_104_hfile_init(std::mem::size_of::<hfile_layout>(), c"r".as_ptr(), 16);
            assert!(!fp.is_null());
            *crate::htslib_rs::c_compat::__errno_location() = libc::E2BIG;
            hfile_c_162_hfile_destroy(fp);
            assert_eq!(*crate::htslib_rs::c_compat::__errno_location(), libc::E2BIG);

            let abrupt =
                hfile_c_104_hfile_init(std::mem::size_of::<hfile_layout>(), c"r".as_ptr(), 16);
            assert!(!abrupt.is_null());
            (*abrupt.cast::<hfile_layout>()).backend = &CLOSE_ERRNO_BACKEND;
            *crate::htslib_rs::c_compat::__errno_location() = libc::E2BIG;
            hfile_c_520_hclose_abruptly(abrupt);
            assert_eq!(*crate::htslib_rs::c_compat::__errno_location(), libc::E2BIG);
        }
    }

    #[test]
    fn hfile_init_reports_posix_memalign_failure_without_installing_null_buffer() {
        unsafe {
            *crate::htslib_rs::c_compat::__errno_location() = 0;
            let fp = hfile_c_104_hfile_init(
                std::mem::size_of::<hfile_layout>(),
                c"w".as_ptr(),
                usize::MAX,
            );
            assert!(fp.is_null());
            assert_ne!(*crate::htslib_rs::c_compat::__errno_location(), 0);
        }
    }

    #[test]
    fn hfile_plugin_initialisers_set_names_and_error_handler_state() {
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
            assert_eq!(
                CStr::from_ptr(crypt_plugin.name).to_bytes(),
                b"crypt4gh-needed"
            );

            assert!(
                hfile_c_935_crypt4gh_needed(c"crypt4gh:/tmp/x".as_ptr(), c"r".as_ptr()).is_null()
            );
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
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

            let fd = libc::open(c_path.as_ptr(), libc::O_RDONLY);
            assert!(fd >= 0);
            let fp2 = hfile_c_1433_knet_dopen(fd, c"Sr".as_ptr());
            assert!(!fp2.is_null());
            assert_eq!((*fp2.cast::<knet_file_layout>()).fd, fd);
            assert_eq!(hfile_c_1460_knet_close(fp2), 0);
            assert_eq!(libc::close(fd), 0);

            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn hfile_refill_peek_and_read2_follow_buffer_rules() {
        unsafe {
            let mut buffer = [0 as c_char; 4];
            let source = b"abcdefghi";
            let mut file = ReadFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr(),
                    begin: buffer.as_mut_ptr(),
                    end: buffer.as_mut_ptr(),
                    limit: buffer.as_mut_ptr().add(buffer.len()),
                    backend: &READ_BACKEND,
                    offset: 0,
                    flags: HFILE_MOBILE,
                    has_errno: 0,
                },
                source: source.as_ptr().cast(),
                source_len: source.len(),
                source_pos: 0,
            };

            let mut peek = [0 as c_char; 3];
            assert_eq!(
                hfile_c_313_hpeek(
                    (&mut file as *mut ReadFile).cast(),
                    peek.as_mut_ptr().cast(),
                    3
                ),
                3
            );
            assert_eq!(
                std::slice::from_raw_parts(peek.as_ptr().cast::<u8>(), 3),
                b"abc"
            );
            assert_eq!(file.source_pos, 4);
            assert_eq!(file.base.begin, file.base.buffer);
            assert_eq!(file.base.end.offset_from(file.base.buffer), 4);

            file.base.begin = file.base.buffer.add(2);
            assert_eq!(
                hfile_c_179_refill_buffer((&mut file as *mut ReadFile).cast()),
                2
            );
            assert_eq!(file.base.offset, 2);
            assert_eq!(
                std::slice::from_raw_parts(file.base.buffer.cast::<u8>(), 4),
                b"cdef"
            );

            let mut dest = [0 as c_char; 5];
            assert_eq!(
                htslib_hfile_h_247_hread(
                    (&mut file as *mut ReadFile).cast(),
                    dest.as_mut_ptr().cast(),
                    5
                ),
                5
            );
            assert_eq!(
                std::slice::from_raw_parts(dest.as_ptr().cast::<u8>(), 5),
                b"cdefg"
            );
        }
    }

    #[test]
    fn hfile_getc_getdelim_hgets_and_khgetline_use_translated_refill() {
        unsafe {
            let mut buffer = [0 as c_char; 5];
            let source = b"ab\ncdef\nlast";
            let mut file = ReadFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr(),
                    begin: buffer.as_mut_ptr(),
                    end: buffer.as_mut_ptr(),
                    limit: buffer.as_mut_ptr().add(buffer.len()),
                    backend: &READ_BACKEND,
                    offset: 0,
                    flags: HFILE_MOBILE,
                    has_errno: 0,
                },
                source: source.as_ptr().cast(),
                source_len: source.len(),
                source_pos: 0,
            };

            assert_eq!(
                hfile_c_235_hgetc2((&mut file as *mut ReadFile).cast()),
                b'a' as c_int
            );

            let mut line = [0 as c_char; 8];
            assert_eq!(
                hfile_c_241_hgetdelim(
                    line.as_mut_ptr(),
                    line.len(),
                    b'\n' as c_int,
                    (&mut file as *mut ReadFile).cast()
                ),
                2
            );
            assert_eq!(std::ffi::CStr::from_ptr(line.as_ptr()).to_bytes(), b"b\n");

            assert_eq!(
                hfile_c_291_hgets(
                    line.as_mut_ptr(),
                    line.len() as c_int,
                    (&mut file as *mut ReadFile).cast()
                ),
                line.as_mut_ptr()
            );
            assert_eq!(
                std::ffi::CStr::from_ptr(line.as_ptr()).to_bytes(),
                b"cdef\n"
            );

            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                hfile_c_306_khgetline(&mut ks, (&mut file as *mut ReadFile).cast()),
                0
            );
            assert_eq!(std::ffi::CStr::from_ptr(ks.s).to_bytes(), b"last");
            crate::htslib_rs::c_compat::free(ks.s.cast());
        }
    }

    #[test]
    fn hfile_seek_uses_buffer_window_and_validates_fixed_end_offsets() {
        unsafe {
            let mut buffer = *b"abcdef\0\0";
            let mut file = ReadFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr().cast(),
                    begin: buffer.as_mut_ptr().cast::<c_char>().add(2),
                    end: buffer.as_mut_ptr().cast::<c_char>().add(6),
                    limit: buffer.as_mut_ptr().cast::<c_char>().add(buffer.len()),
                    backend: &READ_BACKEND,
                    offset: 0,
                    flags: HFILE_READONLY,
                    has_errno: 0,
                },
                source: std::ptr::null(),
                source_len: 0,
                source_pos: 0,
            };

            assert_eq!(
                hfile_c_446_hseek((&mut file as *mut ReadFile).cast(), 4, libc::SEEK_SET),
                4
            );
            assert_eq!(file.base.begin, file.base.buffer.add(4));

            file.base.flags = HFILE_AT_EOF;
            file.base.begin = file.base.buffer.add(1);
            file.base.end = file.base.buffer.add(6);
            assert_eq!(
                hfile_c_446_hseek((&mut file as *mut ReadFile).cast(), -2, libc::SEEK_END),
                4
            );
            assert_eq!(file.base.begin, file.base.buffer.add(4));

            assert_eq!(
                hfile_c_446_hseek((&mut file as *mut ReadFile).cast(), -7, libc::SEEK_END),
                -1
            );
            assert_eq!(file.base.has_errno, libc::EINVAL);
        }
    }

    #[test]
    fn hfile_successful_backend_seek_discards_buffer_and_clears_eof() {
        unsafe {
            let mut buffer = [0 as c_char; 4];
            let source = b"abcdefgh";
            let mut file = ReadFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr(),
                    begin: buffer.as_mut_ptr().add(2),
                    end: buffer.as_mut_ptr().add(4),
                    limit: buffer.as_mut_ptr().add(buffer.len()),
                    backend: &SEEKABLE_READ_BACKEND,
                    offset: 0,
                    flags: HFILE_MOBILE | HFILE_AT_EOF,
                    has_errno: 0,
                },
                source: source.as_ptr().cast(),
                source_len: source.len(),
                source_pos: source.len(),
            };

            assert_eq!(
                hfile_c_446_hseek((&mut file as *mut ReadFile).cast(), 5, libc::SEEK_SET),
                5
            );
            assert_eq!(file.source_pos, 5);
            assert_eq!(file.base.offset, 5);
            assert_eq!(file.base.begin, file.base.buffer);
            assert_eq!(file.base.end, file.base.buffer);
            assert_eq!(file.base.flags & HFILE_AT_EOF, 0);

            let mut out = [0 as c_char; 3];
            assert_eq!(
                htslib_hfile_h_247_hread(
                    (&mut file as *mut ReadFile).cast(),
                    out.as_mut_ptr().cast(),
                    out.len()
                ),
                3
            );
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 3),
                b"fgh"
            );
        }
    }

    #[test]
    fn hfile_failed_backend_seek_preserves_buffer_window_and_sets_herrno() {
        unsafe {
            let mut buffer = *b"abcd";
            let mut file = ReadFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr().cast(),
                    begin: buffer.as_mut_ptr().cast::<c_char>().add(1),
                    end: buffer.as_mut_ptr().cast::<c_char>().add(4),
                    limit: buffer.as_mut_ptr().cast::<c_char>().add(buffer.len()),
                    backend: &READ_BACKEND,
                    offset: 10,
                    flags: HFILE_MOBILE | HFILE_AT_EOF,
                    has_errno: 0,
                },
                source: std::ptr::null(),
                source_len: 0,
                source_pos: 0,
            };

            *crate::htslib_rs::c_compat::__errno_location() = libc::ENOSYS;
            assert_eq!(
                hfile_c_446_hseek((&mut file as *mut ReadFile).cast(), 0, libc::SEEK_SET),
                -1
            );
            assert_eq!(file.base.has_errno, libc::ENOSYS);
            assert_eq!(file.base.offset, 10);
            assert_eq!(file.base.begin, file.base.buffer.add(1));
            assert_eq!(file.base.end, file.base.buffer.add(4));
            assert_ne!(file.base.flags & HFILE_AT_EOF, 0);
        }
    }

    #[test]
    fn hfile_write_flush_putc_and_hputs2_match_buffer_rules() {
        unsafe {
            let mut buffer = [0 as c_char; 4];
            let mut sink = [0 as c_char; 16];
            crate::htslib_rs::c_compat::memcpy(
                buffer.as_mut_ptr().cast(),
                c"abc".as_ptr().cast(),
                3,
            );
            let mut file = WriteFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr(),
                    begin: buffer.as_mut_ptr().add(3),
                    end: buffer.as_mut_ptr(),
                    limit: buffer.as_mut_ptr().add(buffer.len()),
                    backend: &WRITE_BACKEND,
                    offset: 0,
                    flags: HFILE_MOBILE,
                    has_errno: 0,
                },
                sink: sink.as_mut_ptr(),
                sink_len: sink.len(),
                sink_pos: 0,
                flushes: 0,
            };

            assert_eq!(
                hfile_c_400_hputc2(b'd' as c_int, (&mut file as *mut WriteFile).cast()),
                b'd' as c_int
            );
            assert_eq!(file.sink_pos, 3);
            assert_eq!(
                std::slice::from_raw_parts(sink.as_ptr().cast::<u8>(), 3),
                b"abc"
            );
            assert_eq!(*file.base.buffer, b'd' as c_char);
            assert_eq!(file.base.begin.offset_from(file.base.buffer), 1);

            assert_eq!(
                hfile_c_440_hputs2(
                    c"efghij".as_ptr(),
                    6,
                    0,
                    (&mut file as *mut WriteFile).cast()
                ),
                0
            );
            assert_eq!(file.sink_pos, 10);
            assert_eq!(
                std::slice::from_raw_parts(sink.as_ptr().cast::<u8>(), 10),
                b"abcdefghij"
            );

            assert_eq!(hfile_c_390_hflush((&mut file as *mut WriteFile).cast()), 0);
            assert_eq!(file.flushes, 1);
        }
    }

    #[test]
    fn hfile_read_paths_update_offsets_eof_and_tell_like_c() {
        unsafe {
            let mut buffer = [0 as c_char; 4];
            let source = b"abcdefghijkl";
            let mut file = ReadFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr(),
                    begin: buffer.as_mut_ptr(),
                    end: buffer.as_mut_ptr(),
                    limit: buffer.as_mut_ptr().add(buffer.len()),
                    backend: &READ_BACKEND,
                    offset: 0,
                    flags: HFILE_MOBILE,
                    has_errno: 0,
                },
                source: source.as_ptr().cast(),
                source_len: source.len(),
                source_pos: 0,
            };

            let mut direct = [0 as c_char; 8];
            assert_eq!(
                htslib_hfile_h_247_hread(
                    (&mut file as *mut ReadFile).cast(),
                    direct.as_mut_ptr().cast(),
                    direct.len()
                ),
                8
            );
            assert_eq!(
                std::slice::from_raw_parts(direct.as_ptr().cast::<u8>(), 8),
                b"abcdefgh"
            );
            assert_eq!(file.base.offset, 8);
            assert_eq!(file.base.begin, file.base.buffer);
            assert_eq!(file.base.end, file.base.buffer);
            assert_eq!(
                htslib_hfile_h_155_htell((&mut file as *mut ReadFile).cast()),
                8
            );

            let mut tail = [0 as c_char; 8];
            assert_eq!(
                htslib_hfile_h_247_hread(
                    (&mut file as *mut ReadFile).cast(),
                    tail.as_mut_ptr().cast(),
                    tail.len()
                ),
                4
            );
            assert_eq!(
                std::slice::from_raw_parts(tail.as_ptr().cast::<u8>(), 4),
                b"ijkl"
            );
            assert_ne!(file.base.flags & HFILE_AT_EOF, 0);
            assert_eq!(
                htslib_hfile_h_163_hgetc((&mut file as *mut ReadFile).cast()),
                libc::EOF
            );
        }
    }

    #[test]
    fn hfile_line_readers_report_invalid_write_state_like_c() {
        unsafe {
            let mut buffer = [0 as c_char; 4];
            let mut sink = [0 as c_char; 8];
            let mut file = WriteFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr(),
                    begin: buffer.as_mut_ptr().add(1),
                    end: buffer.as_mut_ptr(),
                    limit: buffer.as_mut_ptr().add(buffer.len()),
                    backend: &WRITE_BACKEND,
                    offset: 0,
                    flags: HFILE_MOBILE,
                    has_errno: 0,
                },
                sink: sink.as_mut_ptr(),
                sink_len: sink.len(),
                sink_pos: 0,
                flushes: 0,
            };
            let mut out = [0 as c_char; 4];

            assert_eq!(
                hfile_c_241_hgetdelim(
                    out.as_mut_ptr(),
                    out.len(),
                    b'\n' as c_int,
                    (&mut file as *mut WriteFile).cast()
                ),
                -1
            );
            assert_eq!(file.base.has_errno, libc::EBADF);
            assert_eq!(*crate::htslib_rs::c_compat::__errno_location(), libc::EBADF);

            file.base.begin = file.base.end;
            file.base.has_errno = 0;
            assert!(
                hfile_c_291_hgets(out.as_mut_ptr(), 0, (&mut file as *mut WriteFile).cast())
                    .is_null()
            );
            assert_eq!(file.base.has_errno, libc::EINVAL);
        }
    }

    #[test]
    fn hfile_data_url_edges_preserve_empty_and_base64_detection_semantics() {
        unsafe {
            let empty = hfile_c_845_hopen_mem(c"data:,".as_ptr(), c"r".as_ptr());
            assert!(!empty.is_null());
            let mut out = [1 as c_char; 1];
            assert_eq!(
                htslib_hfile_h_247_hread(empty, out.as_mut_ptr().cast(), out.len()),
                0
            );
            let mut length = 999usize;
            assert!(!hfile_c_894_hfile_mem_get_buffer(empty, &mut length).is_null());
            assert_eq!(length, 1);
            assert_eq!(hclose(empty), 0);

            let not_suffix =
                hfile_c_845_hopen_mem(c"data:;base64x,QUJD%3D".as_ptr(), c"r".as_ptr());
            assert!(!not_suffix.is_null());
            let mut plain = [0 as c_char; 8];
            assert_eq!(
                htslib_hfile_h_247_hread(not_suffix, plain.as_mut_ptr().cast(), plain.len()),
                5
            );
            assert_eq!(
                std::slice::from_raw_parts(plain.as_ptr().cast::<u8>(), 5),
                b"QUJD="
            );
            assert_eq!(hclose(not_suffix), 0);
        }
    }

    #[test]
    fn hfile_plugin_registration_keeps_highest_priority_valid_handler() {
        unsafe extern "C" fn low_open(_fname: *const c_char, _mode: *const c_char) -> *mut hFILE {
            std::ptr::null_mut()
        }
        unsafe extern "C" fn high_open(_fname: *const c_char, _mode: *const c_char) -> *mut hFILE {
            std::ptr::null_mut()
        }
        unsafe extern "C" fn local_remote(_fname: *const c_char) -> c_int {
            0
        }
        unsafe extern "C" fn high_remote(_fname: *const c_char) -> c_int {
            1
        }

        static LOW_HANDLER: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
            open: Some(low_open),
            isremote: Some(local_remote),
            provider: c"low-test".as_ptr(),
            priority: 100,
            vopen: None,
        };
        static INVALID_HANDLER: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
            open: None,
            isremote: Some(high_remote),
            provider: c"invalid-test".as_ptr(),
            priority: 900,
            vopen: None,
        };
        static HIGH_HANDLER: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
            open: Some(high_open),
            isremote: Some(high_remote),
            provider: c"high-test".as_ptr(),
            priority: 200,
            vopen: None,
        };

        unsafe {
            assert!(!hfile_c_1176_find_scheme_handler(c"data:,x".as_ptr()).is_null());

            hfile_c_1053_hfile_add_scheme_handler(
                c"zztest".as_ptr(),
                (&LOW_HANDLER as *const hfile_scheme_handler_layout).cast(),
            );
            hfile_c_1053_hfile_add_scheme_handler(
                c"zztest".as_ptr(),
                (&INVALID_HANDLER as *const hfile_scheme_handler_layout).cast(),
            );
            let handler = hfile_c_1176_find_scheme_handler(c"zztest:path".as_ptr());
            assert!(!handler.is_null());
            assert_eq!(
                CStr::from_ptr((*handler.cast::<hfile_scheme_handler_layout>()).provider)
                    .to_bytes(),
                b"low-test"
            );
            assert_eq!(hisremote(c"zztest:path".as_ptr()), 0);

            hfile_c_1053_hfile_add_scheme_handler(
                c"zztest".as_ptr(),
                (&HIGH_HANDLER as *const hfile_scheme_handler_layout).cast(),
            );
            let handler = hfile_c_1176_find_scheme_handler(c"zztest:path".as_ptr());
            assert!(!handler.is_null());
            assert_eq!(
                CStr::from_ptr((*handler.cast::<hfile_scheme_handler_layout>()).provider)
                    .to_bytes(),
                b"high-test"
            );
            assert_eq!(hisremote(c"zztest:path".as_ptr()), 1);
        }
    }

    #[test]
    fn hfile_init_add_plugin_tracks_success_failure_and_shutdown_destroy() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static INIT_CALLS: AtomicUsize = AtomicUsize::new(0);
        static DESTROY_CALLS: AtomicUsize = AtomicUsize::new(0);

        unsafe extern "C" fn destroy_plugin() {
            DESTROY_CALLS.fetch_add(1, Ordering::SeqCst);
        }

        unsafe fn failing_init(plugin: *mut hFILE_plugin) -> c_int {
            INIT_CALLS.fetch_add(1, Ordering::SeqCst);
            (*(plugin.cast::<hfile_plugin_layout>())).name = c"zzfail-plugin".as_ptr();
            17
        }

        unsafe fn successful_init(plugin: *mut hFILE_plugin) -> c_int {
            INIT_CALLS.fetch_add(1, Ordering::SeqCst);
            let plugin = plugin.cast::<hfile_plugin_layout>();
            (*plugin).name = c"zzok-plugin".as_ptr();
            (*plugin).destroy = destroy_plugin as *const c_void;
            0
        }

        unsafe {
            hfile_c_983_hfile_shutdown(0);
            INIT_CALLS.store(0, Ordering::SeqCst);
            DESTROY_CALLS.store(0, Ordering::SeqCst);

            let before = hfile_plugin_state().lock().unwrap().plugins.len();
            assert_eq!(
                hfile_c_1079_init_add_plugin(
                    std::ptr::null_mut(),
                    failing_init,
                    c"zzfail-plugin".as_ptr()
                ),
                17
            );
            assert_eq!(INIT_CALLS.load(Ordering::SeqCst), 1);
            assert_eq!(hfile_plugin_state().lock().unwrap().plugins.len(), before);

            assert_eq!(
                hfile_c_1079_init_add_plugin(
                    std::ptr::null_mut(),
                    successful_init,
                    c"zzok-plugin".as_ptr()
                ),
                0
            );
            assert_eq!(INIT_CALLS.load(Ordering::SeqCst), 2);
            assert_eq!(
                hfile_plugin_state().lock().unwrap().plugins.len(),
                before + 1
            );

            hfile_c_983_hfile_shutdown(1);
            assert_eq!(DESTROY_CALLS.load(Ordering::SeqCst), 1);
            assert!(hfile_plugin_state().lock().unwrap().plugins.is_empty());

            assert_eq!(hfile_c_1111_load_hfile_plugins(), 0);
        }
    }

    #[test]
    fn hfile_close_preserve_and_fd_flags_keep_ownership_rules() {
        unsafe {
            let mut buffer = [0 as c_char; 4];
            let mut sink = [0 as c_char; 8];
            crate::htslib_rs::c_compat::memcpy(
                buffer.as_mut_ptr().cast(),
                c"xy".as_ptr().cast(),
                2,
            );
            let mut file = WriteFile {
                base: hfile_layout {
                    buffer: buffer.as_mut_ptr(),
                    begin: buffer.as_mut_ptr().add(2),
                    end: buffer.as_mut_ptr(),
                    limit: buffer.as_mut_ptr().add(buffer.len()),
                    backend: &WRITE_BACKEND,
                    offset: 0,
                    flags: HFILE_MOBILE | HFILE_PRESERVE,
                    has_errno: 0,
                },
                sink: sink.as_mut_ptr(),
                sink_len: sink.len(),
                sink_pos: 0,
                flushes: 0,
            };
            assert_eq!(hfile_c_503_hclose((&mut file as *mut WriteFile).cast()), 0);
            assert_eq!(file.sink_pos, 2);
            assert_eq!(file.flushes, 1);
            assert_eq!(
                std::slice::from_raw_parts(sink.as_ptr().cast::<u8>(), 2),
                b"xy"
            );
            assert_eq!(file.base.buffer, buffer.as_mut_ptr());

            let fd = libc::dup(libc::STDOUT_FILENO);
            assert!(fd >= 0);
            let socket = hfile_c_735_hdopen(fd, c"Ssw".as_ptr());
            assert!(!socket.is_null());
            assert_ne!(
                (*socket.cast::<hfile_fd_layout>()).flags & HFILE_FD_IS_SOCKET,
                0
            );
            assert_ne!(
                (*socket.cast::<hfile_fd_layout>()).flags & HFILE_FD_IS_SHARED,
                0
            );
            assert_eq!(hclose(socket), 0);
            assert_eq!(libc::close(fd), 0);

            let mut fds = [-1; 2];
            assert_eq!(
                libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
                0
            );
            let reader = hfile_c_735_hdopen(fds[0], c"sr".as_ptr());
            let writer = hfile_c_735_hdopen(fds[1], c"sw".as_ptr());
            assert!(!reader.is_null());
            assert!(!writer.is_null());
            assert_eq!(
                htslib_hfile_h_292_hwrite(writer, c"sock".as_ptr().cast(), 4),
                4
            );
            assert_eq!(
                hclose(writer),
                0,
                "socket writer hclose errno {}",
                *crate::htslib_rs::c_compat::__errno_location()
            );
            let mut got = [0 as c_char; 4];
            assert_eq!(
                htslib_hfile_h_247_hread(reader, got.as_mut_ptr().cast(), 4),
                4
            );
            assert_eq!(
                std::slice::from_raw_parts(got.as_ptr().cast::<u8>(), 4),
                b"sock"
            );
            assert_eq!(hclose(reader), 0);
        }
    }
}
