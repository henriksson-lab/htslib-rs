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
    hts_sys::stdout.cast()
}

#[inline]
pub unsafe fn stderr() -> *mut libc::FILE {
    hts_sys::stderr.cast()
}

#[inline]
pub unsafe fn getopt_(argc: c_int, argv: *mut *mut c_char, optstring: *const c_char) -> c_int {
    getopt(argc, argv, optstring)
}

#[inline]
pub unsafe fn optarg() -> *const c_char {
    C_OPTARG
}

#[inline]
pub unsafe fn optind() -> c_int {
    C_OPTIND
}

#[inline]
pub fn s_issock(mode: libc::mode_t) -> bool {
    (mode & libc::S_IFMT) == libc::S_IFSOCK
}
