use std::ffi::{c_char, c_int, c_void};

pub const EINVAL: c_int = libc::EINVAL;
pub const ENOENT: c_int = libc::ENOENT;
pub const ENOEXEC: c_int = libc::ENOEXEC;
pub const ENOMEM: c_int = libc::ENOMEM;
pub const EOVERFLOW: c_int = libc::EOVERFLOW;
pub const EPIPE: c_int = libc::EPIPE;
pub const ERANGE: c_int = libc::ERANGE;
pub const EFAULT: c_int = libc::EFAULT;

pub unsafe fn malloc(size: u64) -> *mut c_void {
    libc::malloc(size as usize)
}

pub unsafe fn calloc(nmemb: u64, size: u64) -> *mut c_void {
    libc::calloc(nmemb as usize, size as usize)
}

pub unsafe fn realloc(ptr: *mut c_void, size: u64) -> *mut c_void {
    libc::realloc(ptr, size as usize)
}

pub unsafe fn free(ptr: *mut c_void) {
    libc::free(ptr);
}

pub unsafe fn memcpy(dst: *mut c_void, src: *const c_void, n: u64) -> *mut c_void {
    libc::memcpy(dst, src, n as usize)
}

pub unsafe fn memmove(dst: *mut c_void, src: *const c_void, n: u64) -> *mut c_void {
    libc::memmove(dst, src, n as usize)
}

pub unsafe fn strdup(s: *const c_char) -> *mut c_char {
    libc::strdup(s)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub unsafe fn __errno_location() -> *mut c_int {
    libc::__errno_location()
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
pub unsafe fn __errno_location() -> *mut c_int {
    libc::__error()
}
