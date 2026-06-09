use super::misc::ref_cache_misc_h_38_hexval;
use super::options::Options;
use super::upstream::ref_cache_upstream_c_122_upstream_send_cmd;
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
    hexmd5: [u8; MD5_LEN],
    prev_md5: *mut RefFile,
    next_md5: *mut RefFile,
    size: libc::off_t,
    available: libc::off_t,
    ref_count: c_uint,
    id: c_uint,
    status: RefFileStatus,
    fd: c_int,
}

impl RefFile {
    fn new(hexmd5: [u8; MD5_LEN], id: c_uint) -> Self {
        Self {
            hexmd5,
            prev_md5: std::ptr::null_mut(),
            next_md5: std::ptr::null_mut(),
            size: 0,
            available: 0,
            ref_count: 1,
            id,
            status: REF_WAITING_UPSTREAM,
            fd: -1,
        }
    }
}

// original: RefFiles (htslib/ref_cache/ref_files.c:55)
#[repr(C)]
pub struct RefFiles {
    by_md5: [*mut RefFile; HASH_SZ],
    id: c_uint,
}

// Concurrency note (audit 2026-05):
//
// `REFS` is the in-memory hash table of MD5 -> RefFile entries owned by the
// `ref-cache` daemon server worker. The daemon is single-threaded (fork-based
// process model, epoll event loop — see `src/ref_cache/main.rs` and
// `src/ref_cache/server.rs`). No threading primitives are used in the
// `ref_cache` subsystem; confirm via
// `grep -rn 'pthread_create\|thread::spawn' src/ref_cache/`.
//
// SAFETY: single-threaded daemon worker; all mutation here happens on the
// epoll loop's owning thread, so no synchronization is required.
static mut REFS: RefFiles = RefFiles {
    by_md5: [std::ptr::null_mut(); HASH_SZ],
    id: 0,
};

fn ref_hash(md5: &[u8; MD5_LEN]) -> usize {
    (((ref_cache_misc_h_38_hexval(md5[0] as c_char) << 12)
        | (ref_cache_misc_h_38_hexval(md5[1] as c_char) << 8)
        | (ref_cache_misc_h_38_hexval(md5[2] as c_char) << 4)
        | ref_cache_misc_h_38_hexval(md5[3] as c_char))
        & HASH_MASK) as usize
}

unsafe fn md5_from_raw(md5: *const c_char) -> [u8; MD5_LEN] {
    let mut buf = [0; MD5_LEN];
    std::ptr::copy_nonoverlapping(md5.cast::<u8>(), buf.as_mut_ptr(), MD5_LEN);
    buf
}

fn cache_path_from_md5(md5: &[u8; MD5_LEN]) -> Vec<u8> {
    let mut fname = Vec::with_capacity(MD5_LEN + 3);
    fname.extend_from_slice(&md5[0..2]);
    fname.push(b'/');
    fname.extend_from_slice(&md5[2..4]);
    fname.push(b'/');
    fname.extend_from_slice(&md5[4..]);
    fname.push(0);
    fname
}

// original: get_ref_placeholder (htslib/ref_cache/ref_files.c:62)
unsafe fn ref_cache_ref_files_c_62_get_ref_placeholder(md5: &[u8; MD5_LEN]) -> *mut RefFile {
    let m5hash = ref_hash(md5);

    let mut r = REFS.by_md5[m5hash];
    while !r.is_null() {
        if (*r).hexmd5 == *md5 {
            (*r).ref_count += 1;
            return r;
        }
        r = (*r).next_md5;
    }

    REFS.id += 1;
    r = Box::into_raw(Box::new(RefFile::new(*md5, REFS.id)));
    (*r).next_md5 = REFS.by_md5[m5hash];
    if !(*r).next_md5.is_null() {
        (*(*r).next_md5).prev_md5 = r;
    }
    REFS.by_md5[m5hash] = r;

    r
}

// original: get_ref_file (htslib/ref_cache/ref_files.c:94)
pub unsafe fn ref_cache_ref_files_c_94_get_ref_file(
    opts: *const Options,
    md5: *const c_char,
    upstream_fd: c_int,
) -> *mut RefFile {
    let md5_buf = md5_from_raw(md5);
    let r = ref_cache_ref_files_c_62_get_ref_placeholder(&md5_buf);
    let mut stat_buf: libc::stat = std::mem::zeroed();

    if r.is_null() {
        return std::ptr::null_mut();
    }

    if (*r).ref_count > 1 {
        return r;
    }

    let fname = cache_path_from_md5(&md5_buf);

    (*r).fd = libc::openat((*opts).cache_fd, fname.as_ptr().cast(), libc::O_RDONLY);
    if (*r).fd < 0 {
        if *crate::htslib_rs::c_compat::__errno_location() == libc::ENOENT {
            if upstream_fd >= 0 {
                if ref_cache_upstream_c_122_upstream_send_cmd(upstream_fd, md5, (*r).id) != 0 {
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
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"Couldn't get length of %s/%s : %s\n".as_ptr(),
            (*opts).cache_dir,
            fname.as_ptr().cast::<c_char>(),
            libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
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
    (&*ref_).status
}

// original: get_ref_size (htslib/ref_cache/ref_files.c:145)
pub unsafe fn ref_cache_ref_files_c_145_get_ref_size(ref_: *const RefFile) -> libc::off_t {
    (&*ref_).size
}

// original: get_ref_available (htslib/ref_cache/ref_files.c:149)
pub unsafe fn ref_cache_ref_files_c_149_get_ref_available(ref_: *const RefFile) -> libc::off_t {
    (&*ref_).available
}

// original: get_ref_id (htslib/ref_cache/ref_files.c:153)
pub unsafe fn ref_cache_ref_files_c_153_get_ref_id(ref_: *const RefFile) -> c_uint {
    (&*ref_).id
}

// original: get_ref_complete (htslib/ref_cache/ref_files.c:157)
pub unsafe fn ref_cache_ref_files_c_157_get_ref_complete(ref_: *const RefFile) -> c_int {
    ((&*ref_).status == REF_IS_COMPLETE) as c_int
}

// original: get_ref_fd (htslib/ref_cache/ref_files.c:161)
pub unsafe fn ref_cache_ref_files_c_161_get_ref_fd(ref_: *const RefFile) -> c_int {
    (&*ref_).fd
}

// original: update_ref_download_started (htslib/ref_cache/ref_files.c:165)
pub unsafe fn ref_cache_ref_files_c_165_update_ref_download_started(
    ref_: *mut RefFile,
    fd: c_int,
    size_if_complete: i64,
) {
    let ref_file = &mut *ref_;
    ref_file.fd = fd;
    if size_if_complete >= 0 {
        ref_file.status = REF_IS_COMPLETE;
        ref_file.size = size_if_complete as libc::off_t;
        ref_file.available = size_if_complete as libc::off_t;
    }
}

// original: update_ref_available (htslib/ref_cache/ref_files.c:174)
pub unsafe fn ref_cache_ref_files_c_174_update_ref_available(ref_: *mut RefFile, available: i64) {
    let ref_file = &mut *ref_;
    assert!(ref_file.available <= available as libc::off_t);
    ref_file.available = available as libc::off_t;
}

// original: update_ref_with_content_len (htslib/ref_cache/ref_files.c:179)
pub unsafe fn ref_cache_ref_files_c_179_update_ref_with_content_len(ref_: *mut RefFile, size: i64) {
    let ref_file = &mut *ref_;
    ref_file.size = size as libc::off_t;
    if ref_file.status < REF_DOWNLOAD_STARTED {
        ref_file.status = REF_DOWNLOAD_STARTED;
    }
}

// original: set_ref_complete (htslib/ref_cache/ref_files.c:185)
pub unsafe fn ref_cache_ref_files_c_185_set_ref_complete(ref_: *mut RefFile) -> c_int {
    assert!(!ref_.is_null());
    let ref_file = &mut *ref_;
    let no_content_length = (ref_file.size == 0) as c_int;
    ref_file.status = REF_IS_COMPLETE;
    ref_file.size = ref_file.available;
    no_content_length
}

// original: release_ref_file (htslib/ref_cache/ref_files.c:193)
pub unsafe fn ref_cache_ref_files_c_193_release_ref_file(ref_: *mut RefFile) -> c_int {
    let ref_file = &mut *ref_;

    ref_file.ref_count -= 1;
    if ref_file.ref_count > 0 {
        return 0;
    }

    if ref_file.prev_md5.is_null() {
        let m5hash = ref_hash(&ref_file.hexmd5);
        REFS.by_md5[m5hash] = ref_file.next_md5;
    } else {
        (*ref_file.prev_md5).next_md5 = ref_file.next_md5;
    }
    if !ref_file.next_md5.is_null() {
        (*ref_file.next_md5).prev_md5 = ref_file.prev_md5;
    }

    let res = if ref_file.fd >= 0 {
        libc::close(ref_file.fd)
    } else {
        0
    };

    drop(Box::from_raw(ref_));

    res
}
