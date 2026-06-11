#[cfg(windows)]
use std::ffi::CStr;
use std::ffi::{c_char, c_int, c_void};

pub const EINVAL: c_int = libc::EINVAL;
pub const ENOENT: c_int = libc::ENOENT;
pub const ENOMEM: c_int = libc::ENOMEM;
pub const ERANGE: c_int = libc::ERANGE;
#[cfg(windows)]
const ENOSYS: c_int = libc::ENOSYS;
#[cfg(not(windows))]
pub const REG_EXTENDED: c_int = libc::REG_EXTENDED;
#[cfg(windows)]
pub const REG_EXTENDED: c_int = 1;
#[cfg(not(windows))]
pub const REG_NOSUB: c_int = libc::REG_NOSUB;
#[cfg(windows)]
pub const REG_NOSUB: c_int = 2;

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

pub unsafe fn memcpy(dst: *mut c_void, src: *const c_void, n: u64) -> *mut c_void {
    let Some(n) = size_to_usize(n) else {
        return dst;
    };
    unsafe { libc::memcpy(dst, src, n) }
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
}

#[cfg(windows)]
pub unsafe fn __errno_location() -> *mut c_int {
    windows_errno_location()
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

#[cfg(windows)]
fn regex_new_raw(regex: regex::bytes::Regex) -> *mut c_void {
    Box::into_raw(Box::new(regex)).cast()
}

#[cfg(windows)]
unsafe fn regex_ref_raw(ptr: *mut c_void) -> Option<&'static regex::bytes::Regex> {
    if ptr.is_null() {
        None
    } else {
        Some(&*ptr.cast::<regex::bytes::Regex>())
    }
}

#[cfg(windows)]
unsafe fn regex_free_raw(ptr: *mut c_void) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr.cast::<regex::bytes::Regex>()));
    }
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
                (*preg).inner = regex_new_raw(regex);
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
        let Some(regex) = regex_ref_raw((*preg).inner) else {
            return EINVAL;
        };
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
            regex_free_raw((*preg).inner);
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

#[cfg(not(windows))]
pub type pthread_t = libc::pthread_t;
#[cfg(not(windows))]
pub type pthread_mutex_t = libc::pthread_mutex_t;
#[cfg(not(windows))]
pub type pthread_attr_t = libc::pthread_attr_t;
#[cfg(not(windows))]
pub type pthread_mutexattr_t = libc::pthread_mutexattr_t;

#[cfg(windows)]
pub type pthread_t = usize;
#[cfg(windows)]
pub type pthread_mutex_t = usize;
#[cfg(windows)]
pub type pthread_attr_t = usize;
#[cfg(windows)]
pub type pthread_mutexattr_t = usize;

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
    pub static mut stdout: *mut libc::FILE;
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

}
