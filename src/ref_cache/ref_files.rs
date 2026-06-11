use super::misc::ref_cache_misc_h_38_hexval;
use super::options::Options;
use super::upstream::ref_cache_upstream_c_122_upstream_send_cmd;
use std::ffi::{c_int, c_uint};

pub const MD5_LEN: usize = 32;
pub const REF_WAITING_UPSTREAM: c_int = 0;
pub const REF_NOT_FOUND: c_int = 1;
pub const REF_DOWNLOAD_STARTED: c_int = 2;
pub const REF_IS_COMPLETE: c_int = 3;

const HASH_SZ: usize = 0x10000;
const HASH_MASK: c_int = (HASH_SZ as c_int) - 1;

pub type RefFileStatus = c_int;

// original: RefFile (htslib/ref_cache/ref_files.c:43)
//
// The C version chained entries through an intrusive doubly-linked list using
// raw `prev_md5`/`next_md5` pointers. Here the entries live in an arena
// (`RefFiles::slots`) and the links are arena indices (`Option<usize>`), so
// ownership is fully expressed in owned Rust without aliasing raw pointers.
pub struct RefFile {
    hexmd5: [u8; MD5_LEN],
    prev_md5: Option<usize>,
    next_md5: Option<usize>,
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
            prev_md5: None,
            next_md5: None,
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
//
// `slots` is the arena owning every `RefFile`; a `RefFile` handle returned to
// callers is just an index into this vector. `free` tracks reclaimed slots for
// reuse. `by_md5` holds the head index of each hash chain.
pub struct RefFiles {
    slots: Vec<Option<RefFile>>,
    free: Vec<usize>,
    by_md5: Vec<Option<usize>>,
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
static mut REFS: Option<RefFiles> = None;

fn refs() -> &'static mut RefFiles {
    // SAFETY: see concurrency note above — single-threaded daemon worker.
    unsafe {
        let r = &mut *std::ptr::addr_of_mut!(REFS);
        r.get_or_insert_with(|| RefFiles {
            slots: Vec::new(),
            free: Vec::new(),
            by_md5: vec![None; HASH_SZ],
            id: 0,
        })
    }
}

fn ref_hash(md5: &[u8; MD5_LEN]) -> usize {
    (((ref_cache_misc_h_38_hexval(md5[0]) << 12)
        | (ref_cache_misc_h_38_hexval(md5[1]) << 8)
        | (ref_cache_misc_h_38_hexval(md5[2]) << 4)
        | ref_cache_misc_h_38_hexval(md5[3]))
        & HASH_MASK) as usize
}

fn cache_path_from_md5(md5: &[u8; MD5_LEN]) -> Vec<u8> {
    let mut fname = Vec::with_capacity(MD5_LEN + 3);
    fname.extend_from_slice(&md5[0..2]);
    fname.push(b'/');
    fname.extend_from_slice(&md5[2..4]);
    fname.push(b'/');
    fname.extend_from_slice(&md5[4..]);
    // NUL terminator retained only for the openat() syscall boundary below.
    fname.push(0);
    fname
}

// original: get_ref_placeholder (htslib/ref_cache/ref_files.c:62)
//
// Returns the arena index of the (existing or freshly inserted) entry.
fn ref_cache_ref_files_c_62_get_ref_placeholder(md5: &[u8; MD5_LEN]) -> usize {
    let m5hash = ref_hash(md5);
    let refs = refs();

    let mut cur = refs.by_md5[m5hash];
    while let Some(idx) = cur {
        let entry = refs.slots[idx].as_mut().unwrap();
        if entry.hexmd5 == *md5 {
            entry.ref_count += 1;
            return idx;
        }
        cur = entry.next_md5;
    }

    refs.id += 1;
    let mut new_entry = RefFile::new(*md5, refs.id);
    let head = refs.by_md5[m5hash];
    new_entry.next_md5 = head;

    let new_idx = match refs.free.pop() {
        Some(idx) => {
            refs.slots[idx] = Some(new_entry);
            idx
        }
        None => {
            refs.slots.push(Some(new_entry));
            refs.slots.len() - 1
        }
    };

    if let Some(next) = head {
        refs.slots[next].as_mut().unwrap().prev_md5 = Some(new_idx);
    }
    refs.by_md5[m5hash] = Some(new_idx);

    new_idx
}

// original: get_ref_file (htslib/ref_cache/ref_files.c:94)
//
// Returns the arena index of the entry on success, or `None` on failure.
pub fn ref_cache_ref_files_c_94_get_ref_file(
    opts: &Options,
    md5: &[u8; MD5_LEN],
    upstream_fd: c_int,
) -> Option<usize> {
    let idx = ref_cache_ref_files_c_62_get_ref_placeholder(md5);

    if refs().slots[idx].as_ref().unwrap().ref_count > 1 {
        return Some(idx);
    }

    let fname = cache_path_from_md5(md5);

    // SAFETY: openat is an OS syscall with no portable std equivalent for the
    // dir-fd-relative form; buffers around it are owned Rust.
    let fd = unsafe { libc::openat(opts.cache_fd, fname.as_ptr().cast(), libc::O_RDONLY) };
    refs().slots[idx].as_mut().unwrap().fd = fd;
    if fd < 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ENOENT) {
            if upstream_fd >= 0 {
                let id = refs().slots[idx].as_ref().unwrap().id;
                // SAFETY: upstream_send_cmd wraps a sendmsg() syscall over the
                // upstream command socket; the md5 slice and id passed in are
                // owned Rust and only read for the duration of the call.
                let sent =
                    unsafe { ref_cache_upstream_c_122_upstream_send_cmd(upstream_fd, md5, id) };
                if sent != 0 {
                    ref_cache_ref_files_c_193_release_ref_file(idx);
                    return None;
                }
                refs().slots[idx].as_mut().unwrap().status = REF_WAITING_UPSTREAM;
            } else {
                refs().slots[idx].as_mut().unwrap().status = REF_NOT_FOUND;
            }
            return Some(idx);
        } else {
            ref_cache_ref_files_c_193_release_ref_file(idx);
            return None;
        }
    }

    // SAFETY: fstat is an OS syscall; the stat buffer is owned here.
    let mut stat_buf: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(fd, &mut stat_buf) } != 0 {
        eprintln!(
            "Couldn't get length of {}/{} : {}",
            String::from_utf8_lossy(opts.cache_dir.as_deref().unwrap_or(&[])),
            String::from_utf8_lossy(&fname[..fname.len() - 1]),
            std::io::Error::last_os_error(),
        );
        ref_cache_ref_files_c_193_release_ref_file(idx);
        return None;
    }

    let entry = refs().slots[idx].as_mut().unwrap();
    entry.size = stat_buf.st_size;
    entry.available = stat_buf.st_size;
    entry.status = REF_IS_COMPLETE;

    Some(idx)
}

// original: get_ref_status (htslib/ref_cache/ref_files.c:141)
pub fn ref_cache_ref_files_c_141_get_ref_status(ref_: usize) -> RefFileStatus {
    refs().slots[ref_].as_ref().unwrap().status
}

// original: get_ref_size (htslib/ref_cache/ref_files.c:145)
pub fn ref_cache_ref_files_c_145_get_ref_size(ref_: usize) -> libc::off_t {
    refs().slots[ref_].as_ref().unwrap().size
}

// original: get_ref_available (htslib/ref_cache/ref_files.c:149)
pub fn ref_cache_ref_files_c_149_get_ref_available(ref_: usize) -> libc::off_t {
    refs().slots[ref_].as_ref().unwrap().available
}

// original: get_ref_id (htslib/ref_cache/ref_files.c:153)
pub fn ref_cache_ref_files_c_153_get_ref_id(ref_: usize) -> c_uint {
    refs().slots[ref_].as_ref().unwrap().id
}

// original: get_ref_complete (htslib/ref_cache/ref_files.c:157)
pub fn ref_cache_ref_files_c_157_get_ref_complete(ref_: usize) -> c_int {
    (refs().slots[ref_].as_ref().unwrap().status == REF_IS_COMPLETE) as c_int
}

// original: get_ref_fd (htslib/ref_cache/ref_files.c:161)
pub fn ref_cache_ref_files_c_161_get_ref_fd(ref_: usize) -> c_int {
    refs().slots[ref_].as_ref().unwrap().fd
}

// original: update_ref_download_started (htslib/ref_cache/ref_files.c:165)
pub fn ref_cache_ref_files_c_165_update_ref_download_started(
    ref_: usize,
    fd: c_int,
    size_if_complete: i64,
) {
    let ref_file = refs().slots[ref_].as_mut().unwrap();
    ref_file.fd = fd;
    if size_if_complete >= 0 {
        ref_file.status = REF_IS_COMPLETE;
        ref_file.size = size_if_complete as libc::off_t;
        ref_file.available = size_if_complete as libc::off_t;
    }
}

// original: update_ref_available (htslib/ref_cache/ref_files.c:174)
pub fn ref_cache_ref_files_c_174_update_ref_available(ref_: usize, available: i64) {
    let ref_file = refs().slots[ref_].as_mut().unwrap();
    assert!(ref_file.available <= available as libc::off_t);
    ref_file.available = available as libc::off_t;
}

// original: update_ref_with_content_len (htslib/ref_cache/ref_files.c:179)
pub fn ref_cache_ref_files_c_179_update_ref_with_content_len(ref_: usize, size: i64) {
    let ref_file = refs().slots[ref_].as_mut().unwrap();
    ref_file.size = size as libc::off_t;
    if ref_file.status < REF_DOWNLOAD_STARTED {
        ref_file.status = REF_DOWNLOAD_STARTED;
    }
}

// original: set_ref_complete (htslib/ref_cache/ref_files.c:185)
pub fn ref_cache_ref_files_c_185_set_ref_complete(ref_: usize) -> c_int {
    let ref_file = refs().slots[ref_].as_mut().unwrap();
    let no_content_length = (ref_file.size == 0) as c_int;
    ref_file.status = REF_IS_COMPLETE;
    ref_file.size = ref_file.available;
    no_content_length
}

// original: release_ref_file (htslib/ref_cache/ref_files.c:193)
pub fn ref_cache_ref_files_c_193_release_ref_file(ref_: usize) -> c_int {
    let refs = refs();

    {
        let ref_file = refs.slots[ref_].as_mut().unwrap();
        ref_file.ref_count -= 1;
        if ref_file.ref_count > 0 {
            return 0;
        }
    }

    // Unlink from the hash chain.
    let (prev, next, hexmd5, fd) = {
        let ref_file = refs.slots[ref_].as_ref().unwrap();
        (ref_file.prev_md5, ref_file.next_md5, ref_file.hexmd5, ref_file.fd)
    };

    match prev {
        None => {
            let m5hash = ref_hash(&hexmd5);
            refs.by_md5[m5hash] = next;
        }
        Some(p) => {
            refs.slots[p].as_mut().unwrap().next_md5 = next;
        }
    }
    if let Some(n) = next {
        refs.slots[n].as_mut().unwrap().prev_md5 = prev;
    }

    let res = if fd >= 0 {
        // SAFETY: close is a raw fd syscall with no owned-Rust equivalent here.
        unsafe { libc::close(fd) }
    } else {
        0
    };

    // Drop the owned entry and reclaim its arena slot.
    refs.slots[ref_] = None;
    refs.free.push(ref_);

    res
}
