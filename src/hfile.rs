use std::ffi::{c_char, c_int, c_uint, c_void, CStr};

use super::hts::{
    hFILE, hts_base64_decoded_length, hts_decode_base64, hts_decode_percent, kgetline, kputs,
    kputsn, kstring_t, size_t, tolower_c,
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

#[repr(C)]
struct hfile_scheme_handler_layout {
    open: Option<HFileOpenFn>,
    isremote: Option<HFileIsRemoteFn>,
    provider: *const c_char,
    priority: c_int,
    vopen: *const c_void,
}

unsafe impl Sync for hfile_scheme_handler_layout {}

#[repr(C)]
struct hfile_plugin_layout {
    api_version: c_int,
    obj: *mut c_void,
    name: *const c_char,
    destroy: *const c_void,
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
    *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EINVAL;
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
        let n = libc::read((*fp).fd, buffer, nbytes);
        if !(n < 0 && *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EINTR) {
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
        let n = libc::write((*fp).fd, buffer, nbytes);
        if !(n < 0 && *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EINTR) {
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

unsafe extern "C" {
    #[link_name = "hfile_init"]
    fn htslib_hfile_init(struct_size: size_t, mode: *const c_char, capacity: size_t) -> *mut hFILE;
    #[link_name = "hfile_destroy"]
    fn htslib_hfile_destroy(fp: *mut hFILE);
    #[link_name = "hfile_oflags"]
    fn htslib_hfile_oflags(mode: *const c_char) -> c_int;
    #[link_name = "hdopen"]
    fn htslib_hdopen(fd: c_int, mode: *const c_char) -> *mut hFILE;
    #[link_name = "hisremote"]
    fn htslib_hisremote(filename: *const c_char) -> c_int;
    #[link_name = "haddextension"]
    fn htslib_haddextension(
        buffer: *mut kstring_t,
        filename: *const c_char,
        replace: c_int,
        extension: *const c_char,
    ) -> *mut c_char;
    #[link_name = "hclose"]
    fn htslib_hclose(fp: *mut hFILE) -> c_int;
    #[link_name = "hclose_abruptly"]
    fn htslib_hclose_abruptly(fp: *mut hFILE);
    #[link_name = "hseek"]
    fn htslib_hseek(fp: *mut hFILE, offset: libc::off_t, whence: c_int) -> libc::off_t;
    #[link_name = "hfile_set_blksize"]
    fn htslib_hfile_set_blksize(fp: *mut hFILE, bufsiz: size_t) -> c_int;
    #[link_name = "hgetc2"]
    fn htslib_hgetc2(fp: *mut hFILE) -> c_int;
    #[link_name = "hgetdelim"]
    fn htslib_hgetdelim(
        buffer: *mut c_char,
        size: size_t,
        delim: c_int,
        fp: *mut hFILE,
    ) -> libc::ssize_t;
    #[link_name = "hgets"]
    fn htslib_hgets(buffer: *mut c_char, size: c_int, fp: *mut hFILE) -> *mut c_char;
    #[link_name = "khgetline"]
    fn htslib_khgetline(kstr: *mut kstring_t, fp: *mut hFILE) -> c_int;
    #[link_name = "hpeek"]
    fn htslib_hpeek(fp: *mut hFILE, buffer: *mut c_void, nbytes: size_t) -> libc::ssize_t;
    #[link_name = "hread2"]
    fn htslib_hread2(
        fp: *mut hFILE,
        destv: *mut c_void,
        nbytes: size_t,
        nread: size_t,
    ) -> libc::ssize_t;
    #[link_name = "hputc2"]
    fn htslib_hputc2(c: c_int, fp: *mut hFILE) -> c_int;
    #[link_name = "hwrite2"]
    fn htslib_hwrite2(
        fp: *mut hFILE,
        srcv: *const c_void,
        totalbytes: size_t,
        ncopied: size_t,
    ) -> libc::ssize_t;
    #[link_name = "hputs2"]
    fn htslib_hputs2(
        text: *const c_char,
        totalbytes: size_t,
        ncopied: size_t,
        fp: *mut hFILE,
    ) -> c_int;
    #[link_name = "hflush"]
    fn htslib_hflush(fp: *mut hFILE) -> c_int;
    #[link_name = "hfile_mem_get_buffer"]
    fn htslib_hfile_mem_get_buffer(file: *mut hFILE, length: *mut size_t) -> *mut c_char;
    #[link_name = "hfile_mem_steal_buffer"]
    fn htslib_hfile_mem_steal_buffer(file: *mut hFILE, length: *mut size_t) -> *mut c_char;
    #[link_name = "hfile_list_schemes"]
    fn htslib_hfile_list_schemes(
        plugin: *const c_char,
        sc_list: *mut *const c_char,
        nschemes: *mut c_int,
    ) -> c_int;
    #[link_name = "hfile_list_plugins"]
    fn htslib_hfile_list_plugins(plist: *mut *const c_char, nplugins: *mut c_int) -> c_int;
    #[link_name = "hfile_has_plugin"]
    fn htslib_hfile_has_plugin(name: *const c_char) -> c_int;
    #[link_name = "hfile_add_scheme_handler"]
    fn htslib_hfile_add_scheme_handler(scheme: *const c_char, handler: *const hFILE_scheme_handler);
    #[link_name = "hfile_shutdown"]
    fn htslib_hfile_shutdown(do_close_plugin: c_int);
    #[link_name = "hopen"]
    fn htslib_hopen(fname: *const c_char, mode: *const c_char, ...) -> *mut hFILE;
    #[link_name = "knet_open"]
    fn htslib_knet_open(fn_: *const c_char, mode: *const c_char) -> *mut knetFile;
    #[link_name = "knet_dopen"]
    fn htslib_knet_dopen(fd: c_int, mode: *const c_char) -> *mut knetFile;
    #[link_name = "knet_read"]
    fn htslib_knet_read(fp: *mut knetFile, buf: *mut c_void, len: size_t) -> libc::ssize_t;
    #[link_name = "knet_seek"]
    fn htslib_knet_seek(fp: *mut knetFile, off: libc::off_t, whence: c_int) -> libc::off_t;
    #[link_name = "knet_close"]
    fn htslib_knet_close(fp: *mut knetFile) -> c_int;
}

pub unsafe fn hfile_init(struct_size: size_t, mode: *const c_char, capacity: size_t) -> *mut hFILE {
    hfile_c_104_hfile_init(struct_size, mode, capacity)
}

pub unsafe fn hfile_c_104_hfile_init(
    struct_size: size_t,
    mode: *const c_char,
    mut capacity: size_t,
) -> *mut hFILE {
    let fp = crate::htslib_mini_rs::c_compat::malloc(struct_size as u64).cast::<hfile_layout>();
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
    if libc::posix_memalign(
        (&mut (*fp).buffer as *mut *mut c_char).cast(),
        256,
        capacity,
    ) < 0
    {
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
    let fp = crate::htslib_mini_rs::c_compat::malloc(struct_size as u64).cast::<hfile_layout>();
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
    let save = *crate::htslib_mini_rs::c_compat::__errno_location();
    if !fp.is_null() {
        crate::htslib_mini_rs::c_compat::free((*(fp.cast::<hfile_layout>())).buffer.cast());
    }
    crate::htslib_mini_rs::c_compat::free(fp.cast());
    *crate::htslib_mini_rs::c_compat::__errno_location() = save;
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
    crate::htslib_mini_rs::c_compat::memcpy(buffer, (*layout).begin.cast(), n as u64);
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
    crate::htslib_mini_rs::c_compat::memcpy((*layout).begin.cast(), text.cast(), n as u64);
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
    crate::htslib_mini_rs::c_compat::memcpy((*layout).begin.cast(), buffer, n as u64);
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
        crate::htslib_mini_rs::c_compat::memmove(
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
            (*fp_layout).has_errno = *crate::htslib_mini_rs::c_compat::__errno_location();
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

    let buffer =
        crate::htslib_mini_rs::c_compat::realloc((*fp_layout).buffer.cast(), bufsiz as u64)
            .cast::<c_char>();
    if buffer.is_null() {
        return -1;
    }

    (*fp_layout).begin = buffer.add((*fp_layout).begin.offset_from((*fp_layout).buffer) as usize);
    (*fp_layout).end = buffer.add((*fp_layout).end.offset_from((*fp_layout).buffer) as usize);
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
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EINVAL;
        return -1;
    }
    if (*fp_layout).begin > (*fp_layout).end {
        (*fp_layout).has_errno = libc::EBADF;
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EBADF;
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
            crate::htslib_mini_rs::c_compat::memcpy(
                buffer.add(copied).cast(),
                (*fp_layout).begin.cast(),
                n as u64,
            );
            *buffer.add(n + copied) = 0;
            (*fp_layout).begin = (*fp_layout).begin.add(n);
            return (n + copied) as libc::ssize_t;
        }

        crate::htslib_mini_rs::c_compat::memcpy(
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
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EINVAL;
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
    crate::htslib_mini_rs::c_compat::memcpy(buffer, (*fp_layout).begin.cast(), n as u64);
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
            (*fp_layout).has_errno = *crate::htslib_mini_rs::c_compat::__errno_location();
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
        crate::htslib_mini_rs::c_compat::memcpy(dest.cast(), (*fp_layout).begin.cast(), n as u64);
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
            (*fp_layout).has_errno = *crate::htslib_mini_rs::c_compat::__errno_location();
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
            (*fp_layout).has_errno = *crate::htslib_mini_rs::c_compat::__errno_location();
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
            (*fp_layout).has_errno = *crate::htslib_mini_rs::c_compat::__errno_location();
            return n;
        }
        (*fp_layout).offset += n as libc::off_t;
        src = src.add(n as usize);
        remaining -= n as size_t;
    }

    crate::htslib_mini_rs::c_compat::memcpy(
        (*fp_layout).begin.cast(),
        src.cast(),
        remaining as u64,
    );
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
                    crate::htslib_mini_rs::c_compat::EOVERFLOW
                };
                (*fp_layout).has_errno = err;
                *crate::htslib_mini_rs::c_compat::__errno_location() = err;
                return -1;
            }
        }
    } else if ((*fp_layout).flags & HFILE_MOBILE) == 0 && whence == libc::SEEK_END {
        let length = (*fp_layout).end.offset_from((*fp_layout).buffer) as libc::off_t;
        if offset > 0 || -offset > length {
            (*fp_layout).has_errno = libc::EINVAL;
            *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EINVAL;
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
        (*fp_layout).has_errno = *crate::htslib_mini_rs::c_compat::__errno_location();
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
            err = *crate::htslib_mini_rs::c_compat::__errno_location();
        }
        hfile_destroy(fp);
    }

    if err != 0 {
        *crate::htslib_mini_rs::c_compat::__errno_location() = err;
        libc::EOF
    } else {
        0
    }
}

pub unsafe fn hfile_c_520_hclose_abruptly(fp: *mut hFILE) {
    let save = *crate::htslib_mini_rs::c_compat::__errno_location();
    let fp_layout = fp.cast::<hfile_layout>();
    if ((*fp_layout).flags & HFILE_PRESERVE) != 0 {
        return;
    }
    let close = (*(*fp_layout).backend).close.expect("hFILE close backend");
    let _ = close(fp);
    hfile_destroy(fp);
    *crate::htslib_mini_rs::c_compat::__errno_location() = save;
}

pub unsafe extern "C" fn hfile_c_607_fd_flush(fpv: *mut hFILE) -> c_int {
    let fp = fpv.cast::<hfile_fd_layout>();
    loop {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let mut ret = libc::fdatasync((*fp).fd);
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        let mut ret = libc::fsync((*fp).fd);
        let errno = *crate::htslib_mini_rs::c_compat::__errno_location();
        if ret < 0 && (errno == libc::EINVAL || errno == libc::ENOTSUP) {
            ret = 0;
        }
        if !(ret < 0 && *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EINTR) {
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
        if !(ret < 0 && *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EINTR) {
            return ret;
        }
    }
}

pub unsafe fn hfile_c_648_blksize(fd: c_int) -> size_t {
    let mut sbuf: libc::stat = std::mem::zeroed();
    if libc::fstat(fd, &mut sbuf) != 0 {
        return 0;
    }

    if (sbuf.st_mode & libc::S_IFMT) == libc::S_IFIFO {
        128 * 1024
    } else {
        sbuf.st_blksize as size_t
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
        let save = *crate::htslib_mini_rs::c_compat::__errno_location();
        let _ = libc::close(fd);
        *crate::htslib_mini_rs::c_compat::__errno_location() = save;
        return std::ptr::null_mut();
    }

    (*fp).fd = fd;
    (*fp).flags = 0;
    (*fp).base.backend = &FD_BACKEND;
    fp.cast()
}

pub unsafe fn hfile_c_689_hpreload(fp: *mut hFILE) -> *mut hFILE {
    let mem_fp: *mut hFILE;
    let mut buf: *mut c_char = std::ptr::null_mut();
    let mut buf_sz: libc::off_t = 0;
    let mut buf_a: libc::off_t = 0;
    let mut buf_inc: libc::off_t = 8192;
    let mut len: libc::off_t;

    loop {
        if buf_a - buf_sz < 5000 {
            buf_a += buf_inc;
            let t =
                crate::htslib_mini_rs::c_compat::realloc(buf.cast(), buf_a as u64).cast::<c_char>();
            if t.is_null() {
                crate::htslib_mini_rs::c_compat::free(buf.cast());
                hclose_abruptly(fp);
                return std::ptr::null_mut();
            }
            buf = t;
            if buf_inc < 1_000_000 {
                buf_inc = (buf_inc as f64 * 1.3) as libc::off_t;
            }
        }
        len = htslib_hfile_h_247_hread(
            fp,
            buf.add(buf_sz as usize).cast(),
            (buf_a - buf_sz) as size_t,
        ) as libc::off_t;
        if len > 0 {
            buf_sz += len;
        } else {
            break;
        }
    }

    if len < 0 {
        crate::htslib_mini_rs::c_compat::free(buf.cast());
        hclose_abruptly(fp);
        return std::ptr::null_mut();
    }
    mem_fp = hfile_c_835_create_hfile_mem(buf, c"r".as_ptr(), buf_sz as size_t, buf_a as size_t);
    if mem_fp.is_null() {
        crate::htslib_mini_rs::c_compat::free(buf.cast());
        hclose_abruptly(fp);
        return std::ptr::null_mut();
    }

    if hclose(fp) < 0 {
        hclose_abruptly(mem_fp);
        crate::htslib_mini_rs::c_compat::free(buf.cast());
        hclose_abruptly(fp);
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
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EPROTONOSUPPORT;
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

    #[cfg(any(target_os = "windows"))]
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
    let size;
    let buffer;
    let comma = libc::strchr(url, b',' as c_int);
    if comma.is_null() {
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }
    let data = comma.add(1);

    if libc::strchr(mode, b'r' as c_int).is_null() {
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EROFS;
        return std::ptr::null_mut();
    }

    if comma.offset_from(url) >= 7 && hfile_c_826_cmp_prefix(c";base64".as_ptr(), comma.sub(7)) == 0
    {
        size = hts_base64_decoded_length(CStr::from_ptr(data).to_bytes().len());
        buffer = crate::htslib_mini_rs::c_compat::malloc(size as u64).cast::<c_char>();
        if buffer.is_null() {
            return std::ptr::null_mut();
        }
        hts_decode_base64(buffer, &mut length, data);
    } else {
        size = CStr::from_ptr(data).to_bytes().len() + 1;
        buffer = crate::htslib_mini_rs::c_compat::malloc(size as u64).cast::<c_char>();
        if buffer.is_null() {
            return std::ptr::null_mut();
        }
        hts_decode_percent(buffer, &mut length, data);
    }

    let hf = hfile_c_835_create_hfile_mem(buffer, mode, length, size);
    if hf.is_null() {
        crate::htslib_mini_rs::c_compat::free(buffer.cast());
        return std::ptr::null_mut();
    }

    hf
}

pub unsafe fn hfile_c_894_hfile_mem_get_buffer(
    file: *mut hFILE,
    length: *mut size_t,
) -> *mut c_char {
    let file_layout = file.cast::<hfile_layout>();
    if (*file_layout).backend != &MEM_BACKEND {
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }

    if !length.is_null() {
        *length = (*file_layout).buffer.offset_from((*file_layout).limit) as size_t;
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
    *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EINVAL;
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
    *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EPROTONOSUPPORT;
    std::ptr::null_mut()
}

pub unsafe fn hfile_c_920_hfile_plugin_init_mem(self_: *mut hFILE_plugin) -> c_int {
    static HANDLER: hfile_scheme_handler_layout = hfile_scheme_handler_layout {
        open: Some(hfile_c_915_hopen_not_supported),
        isremote: Some(hfile_c_1342_hfile_always_remote),
        provider: c"mem".as_ptr(),
        priority: 2050,
        vopen: std::ptr::null(),
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
        vopen: std::ptr::null(),
    };

    (*(self_.cast::<hfile_plugin_layout>())).name = c"crypt4gh-needed".as_ptr();
    hfile_add_scheme_handler(
        c"crypt4gh".as_ptr(),
        (&HANDLER as *const hfile_scheme_handler_layout).cast(),
    );
    0
}

pub unsafe fn hfile_c_983_hfile_shutdown(do_close_plugin: c_int) {
    htslib_hfile_shutdown(do_close_plugin);
}

pub unsafe fn hfile_c_1005_hfile_exit() {
    hfile_c_983_hfile_shutdown(0);
}

pub unsafe fn hfile_c_1168_hopen_unknown_scheme(
    fname: *const c_char,
    mode: *const c_char,
) -> *mut hFILE {
    let fp = hfile_c_664_hopen_fd(fname, mode);
    if fp.is_null() && *crate::htslib_mini_rs::c_compat::__errno_location() == libc::ENOENT {
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EPROTONOSUPPORT;
    }
    fp
}

pub unsafe fn hfile_c_1317_hopen(fname: *const c_char, mode: *const c_char) -> *mut hFILE {
    if libc::strncmp(fname, c"data:".as_ptr(), 5) == 0 {
        hfile_c_845_hopen_mem(fname, mode)
    } else if libc::strncmp(fname, c"file://".as_ptr(), 7) == 0 {
        hfile_c_747_hopen_fd_fileuri(fname, mode)
    } else if libc::strncmp(fname, c"preload:".as_ptr(), 8) == 0 {
        hfile_c_730_hopen_preload(fname, mode)
    } else if libc::strcmp(fname, c"-".as_ptr()) == 0 {
        hfile_c_761_hopen_fd_stdinout(mode)
    } else if !libc::strchr(fname, b':' as c_int).is_null() {
        hfile_c_1168_hopen_unknown_scheme(fname, mode)
    } else {
        hfile_c_664_hopen_fd(fname, mode)
    }
}

pub unsafe extern "C" fn hfile_c_1339_hfile_always_local(_fname: *const c_char) -> c_int {
    0
}

pub unsafe extern "C" fn hfile_c_1342_hfile_always_remote(_fname: *const c_char) -> c_int {
    1
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
    let trailing;
    if hisremote(filename) != 0
        || !libc::strchr(filename, b':' as c_int).is_null()
            && !libc::strchr(filename, b'/' as c_int).is_null()
    {
        let span = if libc::strncmp(filename, c"s3://".as_ptr(), 5) != 0
            && libc::strncmp(filename, c"s3+http://".as_ptr(), 10) != 0
            && libc::strncmp(filename, c"s3+https://".as_ptr(), 11) != 0
        {
            libc::strcspn(filename, c"?#".as_ptr())
        } else {
            libc::strcspn(filename, c"?".as_ptr())
        };
        trailing = filename.add(span);
    } else {
        trailing = filename.add(CStr::from_ptr(filename).to_bytes().len());
    }

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
    let fp =
        crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<knet_file_layout>() as u64)
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
        crate::htslib_mini_rs::c_compat::free(fp.cast());
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
    let fp =
        crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<knet_file_layout>() as u64)
            .cast::<knet_file_layout>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    (*fp).hf = hdopen(fd, mode);
    if (*fp).hf.is_null() {
        crate::htslib_mini_rs::c_compat::free(fp.cast());
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
    crate::htslib_mini_rs::c_compat::free(fp.cast());
    r
}

pub unsafe fn hfile_oflags(mode: *const c_char) -> c_int {
    hfile_c_772_hfile_oflags(mode)
}

pub unsafe fn hdopen(fd: c_int, mode: *const c_char) -> *mut hFILE {
    hfile_c_735_hdopen(fd, mode)
}

pub unsafe fn hisremote(filename: *const c_char) -> c_int {
    unsafe { htslib_hisremote(filename) }
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
    unsafe { htslib_hfile_list_schemes(plugin, sc_list, nschemes) }
}

pub unsafe fn hfile_list_plugins(plist: *mut *const c_char, nplugins: *mut c_int) -> c_int {
    unsafe { htslib_hfile_list_plugins(plist, nplugins) }
}

pub unsafe fn hfile_has_plugin(name: *const c_char) -> c_int {
    unsafe { htslib_hfile_has_plugin(name) }
}

pub unsafe fn hfile_add_scheme_handler(
    scheme: *const c_char,
    handler: *const hFILE_scheme_handler,
) {
    unsafe { htslib_hfile_add_scheme_handler(scheme, handler) }
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
        crate::htslib_mini_rs::c_compat::memcpy(
            buffer,
            (*fp).source.add((*fp).source_pos).cast(),
            n as u64,
        );
        (*fp).source_pos += n;
        n as libc::ssize_t
    }

    unsafe extern "C" fn write_backend(
        fp: *mut hFILE,
        buffer: *const c_void,
        nbytes: size_t,
    ) -> libc::ssize_t {
        let fp = fp.cast::<WriteFile>();
        let remaining = (*fp).sink_len - (*fp).sink_pos;
        let n = remaining.min(nbytes);
        crate::htslib_mini_rs::c_compat::memcpy(
            (*fp).sink.add((*fp).sink_pos).cast(),
            buffer,
            n as u64,
        );
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

    unsafe extern "C" fn flush_backend(fp: *mut hFILE) -> c_int {
        let fp = fp.cast::<WriteFile>();
        (*fp).flushes += 1;
        0
    }

    unsafe extern "C" fn close_backend(_fp: *mut hFILE) -> c_int {
        0
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

            let buffer = crate::htslib_mini_rs::c_compat::malloc(8).cast::<c_char>();
            crate::htslib_mini_rs::c_compat::memcpy(buffer.cast(), c"abcde".as_ptr().cast(), 5);

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
            crate::htslib_mini_rs::c_compat::free(ks.s.cast());
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
            assert_eq!(
                length,
                (*fp.cast::<hfile_layout>())
                    .buffer
                    .offset_from((*fp.cast::<hfile_layout>()).limit) as usize
            );
            assert_eq!(hfile_c_810_mem_seek(fp, 0, libc::SEEK_SET), -1);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
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
            crate::htslib_mini_rs::c_compat::free(stolen.cast());
            assert_eq!(hclose(fp64), 0);

            assert!(hfile_c_845_hopen_mem(c"data:,x".as_ptr(), c"w".as_ptr()).is_null());
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                libc::EROFS
            );
            assert!(hfile_c_915_hopen_not_supported(c"mem".as_ptr(), c"r".as_ptr()).is_null());
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                libc::EINVAL
            );
        }
    }

    #[test]
    fn hfile_fd_backend_opens_reads_writes_and_honours_shared_fd() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib-mini-rs-hfile-fd-{}-{}.txt",
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
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                libc::EPROTONOSUPPORT
            );

            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn hfile_preload_copies_fd_stream_into_immobile_memory_file() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib-mini-rs-hfile-preload-{}-{}.txt",
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
    fn hfile_hopen_dispatches_builtin_and_unknown_schemes_like_c_edges() {
        unsafe {
            let handler = hfile_scheme_handler_layout {
                open: None,
                isremote: None,
                provider: c"test".as_ptr(),
                priority: 2050,
                vopen: std::ptr::null(),
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
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                libc::EPROTONOSUPPORT
            );
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
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                libc::EPROTONOSUPPORT
            );
        }
    }

    #[test]
    fn hfile_knet_wrappers_track_offset_and_delegate_to_hfile() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib-mini-rs-knet-{}-{}.txt",
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
            crate::htslib_mini_rs::c_compat::free(ks.s.cast());
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
    fn hfile_write_flush_putc_and_hputs2_match_buffer_rules() {
        unsafe {
            let mut buffer = [0 as c_char; 4];
            let mut sink = [0 as c_char; 16];
            crate::htslib_mini_rs::c_compat::memcpy(
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
}
