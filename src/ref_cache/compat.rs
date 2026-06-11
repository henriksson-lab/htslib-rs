pub type sig_atomic_t = i32;

unsafe extern "C" {
    #[link_name = "optarg"]
    static mut C_OPTARG: *mut u8;
    #[link_name = "optind"]
    static mut C_OPTIND: i32;

    fn getopt(argc: i32, argv: *mut *mut u8, optstring: *const u8) -> i32;
}

#[inline]
pub unsafe fn stdout() -> *mut libc::FILE {
    libc::fdopen(1, b"w\0".as_ptr().cast())
}

#[inline]
pub unsafe fn stderr() -> *mut libc::FILE {
    libc::fdopen(2, b"w\0".as_ptr().cast())
}

/// Wrapper over POSIX `getopt`. `optstring` is taken as a byte slice (no trailing
/// NUL required); the single C boundary call still wants a NUL-terminated string,
/// so build one locally just for that call.
#[inline]
pub unsafe fn getopt_(argc: i32, argv: *mut *mut u8, optstring: &[u8]) -> i32 {
    let mut optstring_c = optstring.to_vec();
    optstring_c.push(0);
    getopt(argc, argv, optstring_c.as_ptr())
}

/// Returns the current `optarg` value as a byte slice (without the trailing NUL),
/// or `None` when `optarg` is null.
#[inline]
pub unsafe fn optarg() -> Option<&'static [u8]> {
    if C_OPTARG.is_null() {
        None
    } else {
        let len = libc::strlen(C_OPTARG.cast());
        Some(std::slice::from_raw_parts(C_OPTARG, len))
    }
}

#[inline]
pub unsafe fn optind() -> i32 {
    C_OPTIND
}

#[inline]
pub fn s_issock(mode: libc::mode_t) -> bool {
    (mode & libc::S_IFMT) == libc::S_IFSOCK
}
