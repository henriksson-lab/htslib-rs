#[cfg(windows)]
use std::ffi::c_uint;
#[cfg(windows)]
use std::ffi::CStr;
use std::ffi::{c_char, c_int, c_void};
use std::marker::PhantomData;
use std::ptr::NonNull;

pub const EINVAL: c_int = libc::EINVAL;
pub const ENOENT: c_int = libc::ENOENT;
pub const ENOEXEC: c_int = libc::ENOEXEC;
pub const ENOMEM: c_int = libc::ENOMEM;
pub const EOVERFLOW: c_int = libc::EOVERFLOW;
pub const EPIPE: c_int = libc::EPIPE;
pub const ERANGE: c_int = libc::ERANGE;
pub const EFAULT: c_int = libc::EFAULT;
pub const EAGAIN: c_int = libc::EAGAIN;
pub const EBADF: c_int = libc::EBADF;
pub const EINTR: c_int = libc::EINTR;
pub const EWOULDBLOCK: c_int = libc::EWOULDBLOCK;
pub const ENOSYS: c_int = libc::ENOSYS;
pub const PATH_MAX: c_int = 4096;
pub const R_OK: c_int = 4;
#[cfg(not(windows))]
pub const REG_EXTENDED: c_int = libc::REG_EXTENDED;
#[cfg(windows)]
pub const REG_EXTENDED: c_int = 1;
#[cfg(not(windows))]
pub const REG_NOSUB: c_int = libc::REG_NOSUB;
#[cfg(windows)]
pub const REG_NOSUB: c_int = 2;
#[cfg(not(windows))]
pub const S_IFIFO: c_int = libc::S_IFIFO as c_int;
#[cfg(windows)]
pub const S_IFIFO: c_int = 0x1000;

fn size_to_usize(size: u64) -> Option<usize> {
    usize::try_from(size).ok()
}

unsafe fn allocation_overflow() -> *mut c_void {
    unsafe {
        *__errno_location() = ENOMEM;
    }
    std::ptr::null_mut()
}

pub unsafe fn malloc(size: u64) -> *mut c_void {
    let Some(size) = size_to_usize(size) else {
        return unsafe { allocation_overflow() };
    };
    unsafe { libc::malloc(size) }
}

pub unsafe fn calloc(nmemb: u64, size: u64) -> *mut c_void {
    let (Some(nmemb), Some(size)) = (size_to_usize(nmemb), size_to_usize(size)) else {
        return unsafe { allocation_overflow() };
    };
    unsafe { libc::calloc(nmemb, size) }
}

pub unsafe fn realloc(ptr: *mut c_void, size: u64) -> *mut c_void {
    let Some(size) = size_to_usize(size) else {
        return unsafe { allocation_overflow() };
    };
    unsafe { libc::realloc(ptr, size) }
}

pub unsafe fn free(ptr: *mut c_void) {
    unsafe { libc::free(ptr) };
}

pub struct FreeOnDrop<T = c_void> {
    ptr: *mut T,
}

impl<T> FreeOnDrop<T> {
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self { ptr }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn into_raw(self) -> *mut T {
        let ptr = self.ptr;
        std::mem::forget(self);
        ptr
    }
}

impl<T> Drop for FreeOnDrop<T> {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr.cast());
        }
    }
}

pub struct MallocBox<T> {
    ptr: NonNull<T>,
}

impl<T> MallocBox<T> {
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    pub fn calloc() -> Option<Self> {
        let ptr = unsafe { calloc(1, std::mem::size_of::<T>() as u64).cast::<T>() };
        unsafe { Self::from_raw(ptr) }
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn as_non_null(&self) -> NonNull<T> {
        self.ptr
    }

    pub fn into_raw(self) -> *mut T {
        let ptr = self.ptr.as_ptr();
        std::mem::forget(self);
        ptr
    }
}

impl<T> Drop for MallocBox<T> {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr.as_ptr().cast());
        }
    }
}

pub struct MallocSlice<T> {
    ptr: NonNull<T>,
    len: usize,
}

impl<T> MallocSlice<T> {
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> Option<Self> {
        if len == 0 {
            return None;
        }
        NonNull::new(ptr).map(|ptr| Self { ptr, len })
    }

    pub fn calloc(len: usize) -> Option<Self> {
        if len == 0 {
            return None;
        }
        let ptr = unsafe { calloc(len as u64, std::mem::size_of::<T>() as u64).cast::<T>() };
        unsafe { Self::from_raw_parts(ptr, len) }
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub unsafe fn as_slice(&self) -> &[T] {
        std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
    }

    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
    }

    pub fn into_raw_parts(self) -> (*mut T, usize) {
        let parts = (self.ptr.as_ptr(), self.len);
        std::mem::forget(self);
        parts
    }
}

impl<T> Drop for MallocSlice<T> {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr.as_ptr().cast());
        }
    }
}

pub struct CArray<T> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T> CArray<T> {
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize, cap: usize) -> Option<Self> {
        if cap == 0 || len > cap {
            return None;
        }
        NonNull::new(ptr).map(|ptr| Self {
            ptr,
            len,
            cap,
            _marker: PhantomData,
        })
    }

    pub fn with_capacity(cap: usize) -> Option<Self> {
        if cap == 0 {
            return None;
        }
        let bytes = cap.checked_mul(std::mem::size_of::<T>())?;
        let ptr = unsafe { malloc(bytes as u64).cast::<T>() };
        unsafe { Self::from_raw_parts(ptr, 0, cap) }
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    pub fn set_len(&mut self, len: usize) {
        assert!(len <= self.cap);
        self.len = len;
    }

    pub fn into_raw_parts(self) -> (*mut T, usize, usize) {
        let parts = (self.ptr.as_ptr(), self.len, self.cap);
        std::mem::forget(self);
        parts
    }
}

impl<T> Drop for CArray<T> {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr.as_ptr().cast());
        }
    }
}

pub unsafe fn memcpy(dst: *mut c_void, src: *const c_void, n: u64) -> *mut c_void {
    let Some(n) = size_to_usize(n) else {
        return dst;
    };
    unsafe { libc::memcpy(dst, src, n) }
}

pub unsafe fn memmove(dst: *mut c_void, src: *const c_void, n: u64) -> *mut c_void {
    let Some(n) = size_to_usize(n) else {
        return dst;
    };
    unsafe { libc::memmove(dst, src, n) }
}

pub unsafe fn strdup(s: *const c_char) -> *mut c_char {
    unsafe { libc::strdup(s) }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub unsafe fn __errno_location() -> *mut c_int {
    libc::__errno_location()
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
pub unsafe fn __errno_location() -> *mut c_int {
    libc::__error()
}

#[cfg(windows)]
extern "C" {
    #[link_name = "_errno"]
    fn windows_errno_location() -> *mut c_int;
    #[link_name = "_commit"]
    fn windows_commit(fd: c_int) -> c_int;
}

#[cfg(windows)]
pub unsafe fn __errno_location() -> *mut c_int {
    windows_errno_location()
}

pub unsafe fn get_errno() -> c_int {
    *__errno_location()
}

pub unsafe fn set_errno(errno: c_int) {
    *__errno_location() = errno;
}

pub unsafe fn with_saved_errno<T>(f: impl FnOnce() -> T) -> T {
    let saved = get_errno();
    let result = f();
    set_errno(saved);
    result
}

pub unsafe fn is_retry_errno(errno: c_int) -> bool {
    errno == EINTR || errno == EAGAIN || errno == EWOULDBLOCK
}

fn time_t_to_i64<T>(time: T) -> i64
where
    i64: From<T>,
{
    i64::from(time)
}

pub fn unix_time_utc_parts(now: libc::time_t) -> (i32, u32, u32, u32, u32, u32, usize) {
    let secs = time_t_to_i64(now);
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let hour = (sod / 3_600) as u32;
    let minute = ((sod % 3_600) / 60) as u32;
    let second = (sod % 60) as u32;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = (mp + if mp < 10 { 3 } else { -9 }) as u32;
    if month <= 2 {
        year += 1;
    }
    let weekday = (days + 4).rem_euclid(7) as usize;
    (year, month, day, hour, minute, second, weekday)
}

pub fn write_c_str(buf: &mut [c_char], text: &str) -> usize {
    if buf.is_empty() {
        return 0;
    }
    let bytes = text.as_bytes();
    let n = bytes.len().min(buf.len() - 1);
    for (dst, src) in buf.iter_mut().take(n).zip(bytes.iter().copied()) {
        *dst = src as c_char;
    }
    buf[n] = 0;
    n
}

pub unsafe fn retry_eintr<T>(mut f: impl FnMut() -> T) -> T
where
    T: PartialOrd + From<i8> + Copy,
{
    loop {
        let result = f();
        if !(result < T::from(0) && get_errno() == EINTR) {
            return result;
        }
    }
}

pub unsafe fn fd_read(fd: c_int, buffer: *mut c_void, nbytes: usize) -> libc::ssize_t {
    #[cfg(windows)]
    {
        libc::read(fd, buffer, nbytes.min(c_uint::MAX as usize) as c_uint) as libc::ssize_t
    }
    #[cfg(not(windows))]
    {
        libc::read(fd, buffer, nbytes)
    }
}

pub unsafe fn fd_write(fd: c_int, buffer: *const c_void, nbytes: usize) -> libc::ssize_t {
    #[cfg(windows)]
    {
        libc::write(fd, buffer, nbytes.min(c_uint::MAX as usize) as c_uint) as libc::ssize_t
    }
    #[cfg(not(windows))]
    {
        libc::write(fd, buffer, nbytes)
    }
}

pub unsafe fn socket_recv(
    fd: c_int,
    buffer: *mut c_void,
    nbytes: usize,
    flags: c_int,
) -> libc::ssize_t {
    #[cfg(windows)]
    {
        let _ = flags;
        fd_read(fd, buffer, nbytes)
    }
    #[cfg(not(windows))]
    {
        if flags == 0 {
            fd_read(fd, buffer, nbytes)
        } else {
            libc::recv(fd, buffer, nbytes, flags)
        }
    }
}

pub unsafe fn socket_send(
    fd: c_int,
    buffer: *const c_void,
    nbytes: usize,
    flags: c_int,
) -> libc::ssize_t {
    #[cfg(windows)]
    {
        let _ = flags;
        fd_write(fd, buffer, nbytes)
    }
    #[cfg(not(windows))]
    {
        if flags == 0 {
            fd_write(fd, buffer, nbytes)
        } else {
            libc::send(fd, buffer, nbytes, flags)
        }
    }
}

pub unsafe fn posix_memalign(out: *mut *mut c_void, align: usize, size: usize) -> c_int {
    #[cfg(not(windows))]
    {
        libc::posix_memalign(out, align, size)
    }
    #[cfg(windows)]
    {
        let _ = align;
        let ptr = libc::malloc(size);
        if ptr.is_null() {
            ENOMEM
        } else {
            *out = ptr;
            0
        }
    }
}

pub unsafe fn fsync(fd: c_int) -> c_int {
    #[cfg(not(windows))]
    {
        libc::fsync(fd)
    }
    #[cfg(windows)]
    {
        windows_commit(fd)
    }
}

pub unsafe fn fdatasync(fd: c_int) -> c_int {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        libc::fdatasync(fd)
    }
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    {
        fsync(fd)
    }
}

pub unsafe fn access(path: *const c_char, mode: c_int) -> c_int {
    libc::access(path, mode)
}

pub unsafe fn chmod(path: *const c_char, mode: c_int) -> c_int {
    #[cfg(windows)]
    {
        libc::chmod(path, mode)
    }
    #[cfg(not(windows))]
    {
        libc::chmod(path, mode as libc::mode_t)
    }
}

pub unsafe fn ftruncate(fd: c_int, len: libc::c_long) -> c_int {
    #[cfg(windows)]
    {
        let _ = (fd, len);
        set_errno(ENOSYS);
        -1
    }
    #[cfg(not(windows))]
    {
        libc::ftruncate(fd, len as libc::off_t)
    }
}

pub unsafe fn getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
    #[cfg(windows)]
    {
        libc::getcwd(buf, size.min(c_int::MAX as usize) as c_int)
    }
    #[cfg(not(windows))]
    {
        libc::getcwd(buf, size)
    }
}

#[cfg(not(windows))]
pub type regex_t = libc::regex_t;
#[cfg(not(windows))]
pub type regmatch_t = libc::regmatch_t;

#[cfg(windows)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct regmatch_t {
    pub rm_so: libc::c_long,
    pub rm_eo: libc::c_long,
}

#[cfg(windows)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct regex_t {
    inner: *mut c_void,
    nosub: c_int,
}

pub unsafe fn regcomp(preg: *mut regex_t, pattern: *const c_char, flags: c_int) -> c_int {
    #[cfg(not(windows))]
    {
        libc::regcomp(preg, pattern, flags)
    }
    #[cfg(windows)]
    {
        if preg.is_null() || pattern.is_null() {
            return EINVAL;
        }
        let pattern = CStr::from_ptr(pattern).to_string_lossy();
        match regex::bytes::Regex::new(&pattern) {
            Ok(regex) => {
                (*preg).inner = Box::into_raw(Box::new(regex)).cast();
                (*preg).nosub = ((flags & REG_NOSUB) != 0) as c_int;
                0
            }
            Err(_) => EINVAL,
        }
    }
}

pub unsafe fn regexec(
    preg: *const regex_t,
    string: *const c_char,
    nmatch: usize,
    pmatch: *mut regmatch_t,
    flags: c_int,
) -> c_int {
    #[cfg(not(windows))]
    {
        libc::regexec(preg, string, nmatch, pmatch, flags)
    }
    #[cfg(windows)]
    {
        let _ = flags;
        if preg.is_null() || string.is_null() || (*preg).inner.is_null() {
            return EINVAL;
        }
        let regex = &*((*preg).inner.cast::<regex::bytes::Regex>());
        let haystack = CStr::from_ptr(string).to_bytes();
        let Some(captures) = regex.captures(haystack) else {
            return 1;
        };
        if nmatch > 0 && !pmatch.is_null() && (*preg).nosub == 0 {
            for i in 0..nmatch {
                let out = pmatch.add(i);
                if let Some(matched) = captures.get(i) {
                    (*out).rm_so = matched.start() as _;
                    (*out).rm_eo = matched.end() as _;
                } else {
                    (*out).rm_so = -1;
                    (*out).rm_eo = -1;
                }
            }
        }
        0
    }
}

pub unsafe fn regfree(preg: *mut regex_t) {
    #[cfg(not(windows))]
    {
        libc::regfree(preg);
    }
    #[cfg(windows)]
    {
        if !preg.is_null() && !(*preg).inner.is_null() {
            drop(Box::from_raw((*preg).inner.cast::<regex::bytes::Regex>()));
            (*preg).inner = std::ptr::null_mut();
        }
    }
}

pub unsafe fn regerror(
    errcode: c_int,
    preg: *const regex_t,
    errbuf: *mut c_char,
    errbuf_size: usize,
) -> usize {
    #[cfg(not(windows))]
    {
        libc::regerror(errcode, preg, errbuf, errbuf_size)
    }
    #[cfg(windows)]
    {
        let _ = preg;
        let msg = if errcode == 0 {
            b"success\0".as_slice()
        } else {
            b"regex error\0".as_slice()
        };
        if !errbuf.is_null() && errbuf_size > 0 {
            let n = msg.len().min(errbuf_size);
            std::ptr::copy_nonoverlapping(msg.as_ptr().cast::<c_char>(), errbuf, n);
            *errbuf.add(n - 1) = 0;
        }
        msg.len()
    }
}

pub fn stat_mode_matches<M, S, V>(st_mode: M, mask: S, value: V) -> bool
where
    M: TryInto<u64>,
    S: TryInto<u64>,
    V: TryInto<u64>,
{
    let Ok(st_mode) = st_mode.try_into() else {
        return false;
    };
    let Ok(mask) = mask.try_into() else {
        return false;
    };
    let Ok(value) = value.try_into() else {
        return false;
    };
    (st_mode & mask) == value
}

#[cfg(not(windows))]
pub type pthread_t = libc::pthread_t;
#[cfg(not(windows))]
pub type pthread_mutex_t = libc::pthread_mutex_t;
#[cfg(not(windows))]
pub type pthread_cond_t = libc::pthread_cond_t;
#[cfg(not(windows))]
pub type pthread_attr_t = libc::pthread_attr_t;
#[cfg(not(windows))]
pub type pthread_mutexattr_t = libc::pthread_mutexattr_t;

#[cfg(windows)]
pub type pthread_t = usize;
#[cfg(windows)]
pub type pthread_mutex_t = usize;
#[cfg(windows)]
pub type pthread_cond_t = usize;
#[cfg(windows)]
pub type pthread_attr_t = usize;
#[cfg(windows)]
pub type pthread_mutexattr_t = usize;

pub const PTHREAD_MUTEX_RECURSIVE: c_int = {
    #[cfg(not(windows))]
    {
        libc::PTHREAD_MUTEX_RECURSIVE
    }
    #[cfg(windows)]
    {
        1
    }
};
#[cfg(not(windows))]
pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
#[cfg(windows)]
pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t = 0;

pub unsafe fn pthread_self() -> pthread_t {
    #[cfg(not(windows))]
    {
        libc::pthread_self()
    }
    #[cfg(windows)]
    {
        0
    }
}

pub unsafe fn pthread_equal(a: pthread_t, b: pthread_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_equal(a, b)
    }
    #[cfg(windows)]
    {
        (a == b) as c_int
    }
}

pub unsafe fn pthread_mutex_init(
    mutex: *mut pthread_mutex_t,
    attr: *const pthread_mutexattr_t,
) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_mutex_init(mutex, attr)
    }
    #[cfg(windows)]
    {
        let _ = attr;
        if !mutex.is_null() {
            *mutex = 0;
        }
        0
    }
}

pub unsafe fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_mutex_destroy(mutex)
    }
    #[cfg(windows)]
    {
        let _ = mutex;
        0
    }
}

pub unsafe fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_mutex_lock(mutex)
    }
    #[cfg(windows)]
    {
        let _ = mutex;
        0
    }
}

pub unsafe fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_mutex_unlock(mutex)
    }
    #[cfg(windows)]
    {
        let _ = mutex;
        0
    }
}

pub unsafe fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_mutexattr_init(attr)
    }
    #[cfg(windows)]
    {
        if !attr.is_null() {
            *attr = 0;
        }
        0
    }
}

pub unsafe fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, kind: c_int) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_mutexattr_settype(attr, kind)
    }
    #[cfg(windows)]
    {
        let _ = (attr, kind);
        0
    }
}

pub unsafe fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_mutexattr_destroy(attr)
    }
    #[cfg(windows)]
    {
        let _ = attr;
        0
    }
}

pub unsafe fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *const c_void) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_cond_init(cond, attr.cast())
    }
    #[cfg(windows)]
    {
        let _ = attr;
        if !cond.is_null() {
            *cond = 0;
        }
        0
    }
}

pub unsafe fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_cond_destroy(cond)
    }
    #[cfg(windows)]
    {
        let _ = cond;
        0
    }
}

pub unsafe fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_cond_signal(cond)
    }
    #[cfg(windows)]
    {
        let _ = cond;
        0
    }
}

pub unsafe fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_cond_broadcast(cond)
    }
    #[cfg(windows)]
    {
        let _ = cond;
        0
    }
}

pub unsafe fn pthread_cond_wait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_cond_wait(cond, mutex)
    }
    #[cfg(windows)]
    {
        let _ = (cond, mutex);
        0
    }
}

pub unsafe fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    timeout: *const libc::timespec,
) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_cond_timedwait(cond, mutex, timeout)
    }
    #[cfg(windows)]
    {
        let _ = (cond, mutex, timeout);
        0
    }
}

pub unsafe fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_attr_init(attr)
    }
    #[cfg(windows)]
    {
        if !attr.is_null() {
            *attr = 0;
        }
        0
    }
}

pub unsafe fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_attr_destroy(attr)
    }
    #[cfg(windows)]
    {
        let _ = attr;
        0
    }
}

pub unsafe fn pthread_attr_getstacksize(attr: *const pthread_attr_t, stack: *mut usize) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_attr_getstacksize(attr, stack)
    }
    #[cfg(windows)]
    {
        let _ = attr;
        if !stack.is_null() {
            *stack = 0;
        }
        0
    }
}

pub unsafe fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stack: usize) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_attr_setstacksize(attr, stack)
    }
    #[cfg(windows)]
    {
        let _ = (attr, stack);
        0
    }
}

pub unsafe fn pthread_create(
    thread: *mut pthread_t,
    attr: *const pthread_attr_t,
    start: extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_create(thread, attr, start, arg)
    }
    #[cfg(windows)]
    {
        let _ = (attr, start, arg);
        if !thread.is_null() {
            *thread = 0;
        }
        ENOSYS
    }
}

pub unsafe fn pthread_join(thread: pthread_t, retval: *mut *mut c_void) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_join(thread, retval)
    }
    #[cfg(windows)]
    {
        let _ = (thread, retval);
        0
    }
}

pub unsafe fn pthread_kill(thread: pthread_t, signal: c_int) -> c_int {
    #[cfg(not(windows))]
    {
        libc::pthread_kill(thread, signal)
    }
    #[cfg(windows)]
    {
        let _ = (thread, signal);
        ENOSYS
    }
}

pub unsafe fn gettimeofday(tv: *mut libc::timeval, tz: *mut c_void) -> c_int {
    #[cfg(not(windows))]
    {
        libc::gettimeofday(tv, tz.cast())
    }
    #[cfg(windows)]
    {
        let _ = tz;
        if !tv.is_null() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            (*tv).tv_sec = now.as_secs() as _;
            (*tv).tv_usec = now.subsec_micros() as _;
        }
        0
    }
}

pub unsafe fn usleep(usec: u32) -> c_int {
    #[cfg(not(windows))]
    {
        libc::usleep(usec)
    }
    #[cfg(windows)]
    {
        std::thread::sleep(std::time::Duration::from_micros(usec as u64));
        0
    }
}

pub unsafe fn nanosleep(req: *const libc::timespec, rem: *mut libc::timespec) -> c_int {
    #[cfg(not(windows))]
    {
        libc::nanosleep(req, rem)
    }
    #[cfg(windows)]
    {
        let _ = rem;
        if !req.is_null() {
            let secs = (*req).tv_sec.max(0) as u64;
            let nanos = (*req).tv_nsec.max(0) as u32;
            std::thread::sleep(std::time::Duration::new(secs, nanos));
        }
        0
    }
}

pub unsafe fn monotonic_timespec(ts: *mut libc::timespec) -> c_int {
    #[cfg(not(windows))]
    {
        libc::clock_gettime(libc::CLOCK_MONOTONIC, ts)
    }
    #[cfg(windows)]
    {
        if !ts.is_null() {
            static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
            let elapsed = START.get_or_init(std::time::Instant::now).elapsed();
            (*ts).tv_sec = elapsed.as_secs() as _;
            (*ts).tv_nsec = elapsed.subsec_nanos() as _;
        }
        0
    }
}

pub unsafe fn utimes(path: *const c_char, times: *const libc::timeval) -> c_int {
    #[cfg(not(windows))]
    {
        libc::utimes(path, times)
    }
    #[cfg(windows)]
    {
        let _ = (path, times);
        set_errno(ENOSYS);
        -1
    }
}

// Native x86-64 System V AMD64 ABI `va_list` element, layout-identical to the C
// `__va_list_tag` that bindgen exposes. Used by the variadic FFI shims (hopen /
// vopen plugin handlers, ksprintf). This is now an internal synthetic argument
// cursor only; translated code must not pass it to C variadic functions.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __va_list_tag {
    pub gp_offset: u32,
    pub fp_offset: u32,
    pub overflow_arg_area: *mut c_void,
    pub reg_save_area: *mut c_void,
}

// The C standard streams, linked directly from the C runtime rather than via
// hts_sys. On glibc/musl/BSD/macOS these are real `extern FILE *` symbols.
extern "C" {
    pub static mut stderr: *mut libc::FILE;
    pub static mut stdout: *mut libc::FILE;
    pub static mut stdin: *mut libc::FILE;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn calloc_zeroes_and_realloc_preserves_prefix() {
        unsafe {
            let ptr = calloc(4, std::mem::size_of::<u32>() as u64).cast::<u32>();
            assert!(!ptr.is_null());
            assert_eq!(std::slice::from_raw_parts(ptr, 4), &[0, 0, 0, 0]);

            *ptr.add(0) = 0x1122_3344;
            *ptr.add(1) = 0x5566_7788;

            let grown = realloc(ptr.cast(), (8 * std::mem::size_of::<u32>()) as u64).cast::<u32>();
            assert!(!grown.is_null());
            assert_eq!(*grown.add(0), 0x1122_3344);
            assert_eq!(*grown.add(1), 0x5566_7788);
            free(grown.cast());
        }
    }

    #[test]
    fn malloc_realloc_null_and_free_null_follow_c_allocation_semantics() {
        unsafe {
            free(std::ptr::null_mut());

            let allocated = malloc(3).cast::<u8>();
            assert!(!allocated.is_null());
            *allocated.add(0) = b'h';
            *allocated.add(1) = b't';
            *allocated.add(2) = b's';
            assert_eq!(std::slice::from_raw_parts(allocated, 3), b"hts");
            free(allocated.cast());

            let reallocated = realloc(std::ptr::null_mut(), 3).cast::<u8>();
            assert!(!reallocated.is_null());
            *reallocated.add(0) = b'b';
            *reallocated.add(1) = b'a';
            *reallocated.add(2) = b'm';
            assert_eq!(std::slice::from_raw_parts(reallocated, 3), b"bam");
            free(reallocated.cast());
        }
    }

    #[test]
    fn malloc_box_owns_and_releases_single_allocation() {
        unsafe {
            assert!(MallocBox::<u32>::from_raw(std::ptr::null_mut()).is_none());

            let mut boxed = MallocBox::<u32>::calloc().expect("calloc MallocBox");
            assert_eq!(*boxed.as_ptr(), 0);
            *boxed.as_mut_ptr() = 0xfeed_beef;
            assert_eq!(*boxed.as_ptr(), 0xfeed_beef);

            let raw = boxed.into_raw();
            assert_eq!(*raw, 0xfeed_beef);
            free(raw.cast());
        }
    }

    #[test]
    fn malloc_slice_tracks_length_and_releases_raw_parts() {
        unsafe {
            assert!(MallocSlice::<u8>::calloc(0).is_none());
            assert!(MallocSlice::<u8>::from_raw_parts(std::ptr::null_mut(), 4).is_none());

            let mut slice = MallocSlice::<u16>::calloc(3).expect("calloc MallocSlice");
            assert_eq!(slice.len(), 3);
            assert!(!slice.is_empty());
            assert_eq!(slice.as_slice(), &[0, 0, 0]);
            slice.as_mut_slice().copy_from_slice(&[1, 2, 3]);
            assert_eq!(slice.as_slice(), &[1, 2, 3]);

            let (raw, len) = slice.into_raw_parts();
            assert_eq!(len, 3);
            assert_eq!(std::slice::from_raw_parts(raw, len), &[1, 2, 3]);
            free(raw.cast());
        }
    }

    #[test]
    fn carray_tracks_len_capacity_and_raw_release() {
        unsafe {
            assert!(CArray::<u8>::with_capacity(0).is_none());
            assert!(CArray::<u8>::from_raw_parts(std::ptr::null_mut(), 0, 4).is_none());

            let mut array = CArray::<u32>::with_capacity(4).expect("malloc CArray");
            assert_eq!(array.len(), 0);
            assert!(array.is_empty());
            assert_eq!(array.cap(), 4);
            *array.as_mut_ptr().add(0) = 10;
            *array.as_mut_ptr().add(1) = 20;
            array.set_len(2);
            assert!(!array.is_empty());
            assert_eq!(
                std::slice::from_raw_parts(array.as_ptr(), array.len()),
                &[10, 20]
            );

            let (raw, len, cap) = array.into_raw_parts();
            assert_eq!((len, cap), (2, 4));
            assert_eq!(std::slice::from_raw_parts(raw, len), &[10, 20]);
            free(raw.cast());
        }
    }

    #[test]
    fn free_on_drop_wraps_nullable_pointer_and_can_disarm() {
        unsafe {
            let null_owner = FreeOnDrop::<u8>::from_raw(std::ptr::null_mut());
            assert!(null_owner.is_null());
            drop(null_owner);

            let ptr = malloc(2).cast::<u8>();
            assert!(!ptr.is_null());
            *ptr.add(0) = b'o';
            *ptr.add(1) = b'k';
            let mut owner = FreeOnDrop::from_raw(ptr);
            assert_eq!(std::slice::from_raw_parts(owner.as_ptr(), 2), b"ok");
            *owner.as_mut_ptr().add(1) = b'!';
            let raw = owner.into_raw();
            assert_eq!(std::slice::from_raw_parts(raw, 2), b"o!");
            free(raw.cast());
        }
    }

    #[test]
    fn allocation_size_overflow_fails_without_freeing_existing_pointer() {
        if usize::MAX as u64 == u64::MAX {
            return;
        }

        unsafe {
            let too_large = usize::MAX as u64 + 1;

            *__errno_location() = 0;
            assert!(malloc(too_large).is_null());
            assert_eq!(*__errno_location(), ENOMEM);

            *__errno_location() = 0;
            assert!(calloc(too_large, 1).is_null());
            assert_eq!(*__errno_location(), ENOMEM);

            let ptr = malloc(1);
            assert!(!ptr.is_null());
            *(ptr.cast::<u8>()) = 0x5a;

            *__errno_location() = 0;
            assert!(realloc(ptr, too_large).is_null());
            assert_eq!(*__errno_location(), ENOMEM);
            assert_eq!(*(ptr.cast::<u8>()), 0x5a);
            free(ptr);
        }
    }

    #[test]
    fn memcpy_and_memmove_return_destination_and_copy_expected_bytes() {
        unsafe {
            let src = *b"abcdef";
            let mut dst = [0u8; 6];
            let copied = memcpy(
                dst.as_mut_ptr().cast(),
                src.as_ptr().cast(),
                src.len() as u64,
            );
            assert_eq!(copied, dst.as_mut_ptr().cast());
            assert_eq!(&dst, b"abcdef");

            let moved = memmove(dst.as_mut_ptr().add(1).cast(), dst.as_ptr().cast(), 5);
            assert_eq!(moved, dst.as_mut_ptr().add(1).cast());
            assert_eq!(&dst, b"aabcde");
        }
    }

    #[test]
    fn zero_length_memory_operations_return_destination_without_modifying_bytes() {
        unsafe {
            let src = *b"xyz";
            let mut dst = *b"abc";

            let copied = memcpy(dst.as_mut_ptr().cast(), src.as_ptr().cast(), 0);
            assert_eq!(copied, dst.as_mut_ptr().cast());
            assert_eq!(&dst, b"abc");

            let moved = memmove(dst.as_mut_ptr().add(1).cast(), dst.as_ptr().cast(), 0);
            assert_eq!(moved, dst.as_mut_ptr().add(1).cast());
            assert_eq!(&dst, b"abc");
        }
    }

    #[test]
    fn strdup_allocates_independent_nul_terminated_copy() {
        unsafe {
            let original = CString::new("htslib-rs").unwrap();
            let duplicate = strdup(original.as_ptr());
            assert!(!duplicate.is_null());
            assert_eq!(CStr::from_ptr(duplicate).to_bytes(), b"htslib-rs");

            free(duplicate.cast());
        }
    }

    #[test]
    fn strdup_preserves_empty_string_terminator() {
        unsafe {
            let original = CString::new("").unwrap();
            let duplicate = strdup(original.as_ptr());
            assert!(!duplicate.is_null());
            assert_eq!(CStr::from_ptr(duplicate).to_bytes_with_nul(), b"\0");

            free(duplicate.cast());
        }
    }

    #[test]
    fn errno_location_exposes_mutable_thread_errno_slot() {
        unsafe {
            let errno = __errno_location();
            assert!(!errno.is_null());

            let saved = *errno;
            *errno = ERANGE;
            assert_eq!(*__errno_location(), ERANGE);
            *errno = saved;
        }
    }

    #[test]
    fn unix_time_utc_parts_matches_known_dates() {
        assert_eq!(unix_time_utc_parts(0), (1970, 1, 1, 0, 0, 0, 4));
        assert_eq!(
            unix_time_utc_parts(1_748_868_896),
            (2025, 6, 2, 12, 54, 56, 1)
        );
    }

    #[test]
    fn write_c_str_truncates_and_terminates() {
        let mut buf = [0 as c_char; 4];
        assert_eq!(write_c_str(&mut buf, "abcd"), 3);
        assert_eq!(buf.iter().map(|&c| c as u8).collect::<Vec<_>>(), b"abc\0");
    }

    #[test]
    fn monotonic_timespec_is_non_decreasing() {
        unsafe {
            let mut first: libc::timespec = std::mem::zeroed();
            let mut second: libc::timespec = std::mem::zeroed();
            assert_eq!(monotonic_timespec(&mut first), 0);
            std::thread::sleep(std::time::Duration::from_millis(1));
            assert_eq!(monotonic_timespec(&mut second), 0);
            assert!(
                (second.tv_sec, second.tv_nsec) >= (first.tv_sec, first.tv_nsec),
                "second={:?} first={:?}",
                (second.tv_sec, second.tv_nsec),
                (first.tv_sec, first.tv_nsec)
            );
        }
    }
}
