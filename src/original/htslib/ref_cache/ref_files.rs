use crate::original_stubs::{functions, structs};
use std::ffi::{c_char, c_int, c_uint};

pub const MD5_LEN: usize = 32;
pub const REF_WAITING_UPSTREAM: c_int = 0;
pub const REF_NOT_FOUND: c_int = 1;
pub const REF_DOWNLOAD_STARTED: c_int = 2;
pub const REF_IS_COMPLETE: c_int = 3;

const HASH_SZ: usize = 0x10000;
const HASH_MASK: c_int = (HASH_SZ as c_int) - 1;

pub type RefFileStatus = c_int;

// original: RefFile (htslib/ref_cache/ref_files.c:43)
#[repr(C)]
pub struct RefFile {
    hexmd5: [c_char; MD5_LEN],
    prev_md5: *mut RefFile,
    next_md5: *mut RefFile,
    size: libc::off_t,
    available: libc::off_t,
    ref_count: c_uint,
    id: c_uint,
    status: RefFileStatus,
    fd: c_int,
}

// original: RefFiles (htslib/ref_cache/ref_files.c:55)
#[repr(C)]
pub struct RefFiles {
    by_md5: [*mut RefFile; HASH_SZ],
    id: c_uint,
}

static mut REFS: RefFiles = RefFiles {
    by_md5: [std::ptr::null_mut(); HASH_SZ],
    id: 0,
};

// original: get_ref_placeholder (htslib/ref_cache/ref_files.c:62)
unsafe fn ref_cache_ref_files_c_62_get_ref_placeholder(md5: *const c_char) -> *mut RefFile {
    let m5hash = ((functions::ref_cache_misc_h_38_hexval(*md5.add(0)) << 12)
        | (functions::ref_cache_misc_h_38_hexval(*md5.add(1)) << 8)
        | (functions::ref_cache_misc_h_38_hexval(*md5.add(2)) << 4)
        | (functions::ref_cache_misc_h_38_hexval(*md5.add(3)) << 0))
        & HASH_MASK;

    let mut r = REFS.by_md5[m5hash as usize];
    while !r.is_null() {
        if libc::strncmp(md5, (*r).hexmd5.as_ptr(), MD5_LEN) == 0 {
            (*r).ref_count += 1;
            return r;
        }
        r = (*r).next_md5;
    }

    r = libc::calloc(1, std::mem::size_of::<RefFile>()).cast();
    if r.is_null() {
        return std::ptr::null_mut();
    }

    (*r).status = REF_WAITING_UPSTREAM;
    (*r).fd = -1;
    (*r).ref_count = 1;
    REFS.id += 1;
    (*r).id = REFS.id;
    libc::memcpy((*r).hexmd5.as_mut_ptr().cast(), md5.cast(), MD5_LEN);
    (*r).prev_md5 = std::ptr::null_mut();
    (*r).next_md5 = REFS.by_md5[m5hash as usize];
    if !(*r).next_md5.is_null() {
        (*(*r).next_md5).prev_md5 = r;
    }
    REFS.by_md5[m5hash as usize] = r;

    r
}

// original: get_ref_file (htslib/ref_cache/ref_files.c:94)
pub unsafe fn ref_cache_ref_files_c_94_get_ref_file(
    opts: *const structs::Options,
    md5: *const c_char,
    upstream_fd: c_int,
) -> *mut RefFile {
    let r = ref_cache_ref_files_c_62_get_ref_placeholder(md5);
    let mut fname = [0 as c_char; MD5_LEN + 3];
    let mut stat_buf: libc::stat = std::mem::zeroed();

    if (*r).ref_count > 1 {
        return r;
    }

    fname[0] = *md5.add(0);
    fname[1] = *md5.add(1);
    fname[2] = b'/' as c_char;
    fname[3] = *md5.add(2);
    fname[4] = *md5.add(3);
    fname[5] = b'/' as c_char;
    libc::memcpy(
        fname.as_mut_ptr().add(6).cast(),
        md5.add(4).cast(),
        MD5_LEN - 4,
    );
    fname[MD5_LEN + 2] = 0;

    (*r).fd = libc::openat((*opts).cache_fd, fname.as_ptr(), libc::O_RDONLY);
    if (*r).fd < 0 {
        if *crate::htslib_mini_rs::c_compat::__errno_location() == libc::ENOENT {
            if upstream_fd >= 0 {
                if functions::ref_cache_upstream_c_122_upstream_send_cmd(upstream_fd, md5, (*r).id)
                    != 0
                {
                    ref_cache_ref_files_c_193_release_ref_file(r);
                    return std::ptr::null_mut();
                }
                (*r).status = REF_WAITING_UPSTREAM;
            } else {
                (*r).status = REF_NOT_FOUND;
            }
            return r;
        } else {
            ref_cache_ref_files_c_193_release_ref_file(r);
            return std::ptr::null_mut();
        }
    }

    if libc::fstat((*r).fd, &mut stat_buf) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't get length of %s/%s : %s\n".as_ptr(),
            (*opts).cache_dir,
            fname.as_ptr(),
            libc::strerror(*crate::htslib_mini_rs::c_compat::__errno_location()),
        );
        ref_cache_ref_files_c_193_release_ref_file(r);
        return std::ptr::null_mut();
    }

    (*r).size = stat_buf.st_size;
    (*r).available = stat_buf.st_size;
    (*r).status = REF_IS_COMPLETE;

    r
}

// original: get_ref_status (htslib/ref_cache/ref_files.c:141)
pub unsafe fn ref_cache_ref_files_c_141_get_ref_status(ref_: *const RefFile) -> RefFileStatus {
    (*ref_).status
}

// original: get_ref_size (htslib/ref_cache/ref_files.c:145)
pub unsafe fn ref_cache_ref_files_c_145_get_ref_size(ref_: *const RefFile) -> libc::off_t {
    (*ref_).size
}

// original: get_ref_available (htslib/ref_cache/ref_files.c:149)
pub unsafe fn ref_cache_ref_files_c_149_get_ref_available(ref_: *const RefFile) -> libc::off_t {
    (*ref_).available
}

// original: get_ref_id (htslib/ref_cache/ref_files.c:153)
pub unsafe fn ref_cache_ref_files_c_153_get_ref_id(ref_: *const RefFile) -> c_uint {
    (*ref_).id
}

// original: get_ref_complete (htslib/ref_cache/ref_files.c:157)
pub unsafe fn ref_cache_ref_files_c_157_get_ref_complete(ref_: *const RefFile) -> c_int {
    ((*ref_).status == REF_IS_COMPLETE) as c_int
}

// original: get_ref_fd (htslib/ref_cache/ref_files.c:161)
pub unsafe fn ref_cache_ref_files_c_161_get_ref_fd(ref_: *const RefFile) -> c_int {
    (*ref_).fd
}

// original: update_ref_download_started (htslib/ref_cache/ref_files.c:165)
pub unsafe fn ref_cache_ref_files_c_165_update_ref_download_started(
    ref_: *mut RefFile,
    fd: c_int,
    size_if_complete: i64,
) {
    (*ref_).fd = fd;
    if size_if_complete >= 0 {
        (*ref_).status = REF_IS_COMPLETE;
        (*ref_).size = size_if_complete as libc::off_t;
        (*ref_).available = size_if_complete as libc::off_t;
    }
}

// original: update_ref_available (htslib/ref_cache/ref_files.c:174)
pub unsafe fn ref_cache_ref_files_c_174_update_ref_available(ref_: *mut RefFile, available: i64) {
    assert!((*ref_).available <= available as libc::off_t);
    (*ref_).available = available as libc::off_t;
}

// original: update_ref_with_content_len (htslib/ref_cache/ref_files.c:179)
pub unsafe fn ref_cache_ref_files_c_179_update_ref_with_content_len(ref_: *mut RefFile, size: i64) {
    (*ref_).size = size as libc::off_t;
    if (*ref_).status < REF_DOWNLOAD_STARTED {
        (*ref_).status = REF_DOWNLOAD_STARTED;
    }
}

// original: set_ref_complete (htslib/ref_cache/ref_files.c:185)
pub unsafe fn ref_cache_ref_files_c_185_set_ref_complete(ref_: *mut RefFile) -> c_int {
    assert!(!ref_.is_null());
    let no_content_length = ((*ref_).size == 0) as c_int;
    (*ref_).status = REF_IS_COMPLETE;
    (*ref_).size = (*ref_).available;
    no_content_length
}

// original: release_ref_file (htslib/ref_cache/ref_files.c:193)
pub unsafe fn ref_cache_ref_files_c_193_release_ref_file(ref_: *mut RefFile) -> c_int {
    (*ref_).ref_count -= 1;
    if (*ref_).ref_count > 0 {
        return 0;
    }

    if (*ref_).prev_md5.is_null() {
        let m5hash = ((functions::ref_cache_misc_h_38_hexval((*ref_).hexmd5[0]) << 12)
            | (functions::ref_cache_misc_h_38_hexval((*ref_).hexmd5[1]) << 8)
            | (functions::ref_cache_misc_h_38_hexval((*ref_).hexmd5[2]) << 4)
            | (functions::ref_cache_misc_h_38_hexval((*ref_).hexmd5[3]) << 0))
            & HASH_MASK;
        REFS.by_md5[m5hash as usize] = (*ref_).next_md5;
    } else {
        (*(*ref_).prev_md5).next_md5 = (*ref_).next_md5;
    }
    if !(*ref_).next_md5.is_null() {
        (*(*ref_).next_md5).prev_md5 = (*ref_).prev_md5;
    }

    let res = if (*ref_).fd >= 0 {
        libc::close((*ref_).fd)
    } else {
        0
    };

    libc::free(ref_.cast());

    res
}
