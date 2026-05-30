use std::ffi::{c_char, c_int, c_void};

pub const EINVAL: c_int = libc::EINVAL;
pub const ENOENT: c_int = libc::ENOENT;
pub const ENOEXEC: c_int = libc::ENOEXEC;
pub const ENOMEM: c_int = libc::ENOMEM;
pub const EOVERFLOW: c_int = libc::EOVERFLOW;
pub const EPIPE: c_int = libc::EPIPE;
pub const ERANGE: c_int = libc::ERANGE;
pub const EFAULT: c_int = libc::EFAULT;

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

// Native x86-64 System V AMD64 ABI `va_list` element, layout-identical to the C
// `__va_list_tag` that bindgen exposes. Used by the variadic FFI shims (hopen /
// vopen plugin handlers, ksprintf, vsnprintf). Defining it here removes the
// dependency on hts_sys for variadic argument handling.
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

// libc `vsnprintf` declared directly so we don't reach into hts_sys for a
// symbol that lives in the system C runtime anyway. The Rust `libc` crate
// doesn't re-export this one; bindgen otherwise synthesises a binding inside
// hts-sys.
extern "C" {
    pub fn vsnprintf(
        s: *mut c_char,
        maxlen: u64,
        format: *const c_char,
        arg: *mut __va_list_tag,
    ) -> c_int;
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
}
