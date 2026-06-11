use std::ffi::{c_char, c_int};

pub type sig_atomic_t = c_int;

unsafe extern "C" {
    #[link_name = "optarg"]
    static mut C_OPTARG: *mut c_char;
    #[link_name = "optind"]
    static mut C_OPTIND: c_int;

    fn getopt(argc: c_int, argv: *mut *mut c_char, optstring: *const c_char) -> c_int;
}

#[inline]
pub unsafe fn stdout() -> *mut libc::FILE {
    crate::htslib_rs::c_compat::stdout.cast()
}

#[inline]
pub unsafe fn stderr() -> *mut libc::FILE {
    crate::htslib_rs::c_compat::stderr.cast()
}

/// Wrapper over POSIX `getopt`. `optstring` is taken as a byte slice (no trailing
/// NUL required); the single C boundary call still wants a NUL-terminated string,
/// so build one locally just for that call.
#[inline]
pub unsafe fn getopt_(argc: c_int, argv: *mut *mut c_char, optstring: &[u8]) -> c_int {
    let mut optstring_c = optstring.to_vec();
    optstring_c.push(0);
    getopt(argc, argv, optstring_c.as_ptr().cast())
}

/// Returns the current `optarg` value as a byte slice (without the trailing NUL),
/// or `None` when `optarg` is null.
#[inline]
pub unsafe fn optarg() -> Option<&'static [u8]> {
    if C_OPTARG.is_null() {
        None
    } else {
        let len = libc::strlen(C_OPTARG);
        Some(std::slice::from_raw_parts(C_OPTARG.cast::<u8>(), len))
    }
}

#[inline]
pub unsafe fn optind() -> c_int {
    C_OPTIND
}

#[inline]
pub fn s_issock(mode: libc::mode_t) -> bool {
    (mode & libc::S_IFMT) == libc::S_IFSOCK
}
