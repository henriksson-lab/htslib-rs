/*  upstream.c -- download ref-cache files from upstream host

    Copyright (C) 2025 Genome Research Ltd.

    Author: Rob Davies <rmd@sanger.ac.uk>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.  */

use super::cmsg_wrap::{
    ref_cache_cmsg_wrap_c_46_make_scm_rights_cmsg, ref_cache_cmsg_wrap_c_67_get_scm_rights_fd,
};
use super::misc::{ref_cache_misc_h_38_hexval, ref_cache_misc_h_55_do_write_all};
use super::options::Options;
use super::poll_wrap::{Pw_fd_type, PW_ERR, PW_HUP, PW_IN, PW_OUT};
use super::poll_wrap_epoll as poll_impl;
use crate::htslib_rs::c_compat::__errno_location;
use crate::htslib_rs::md5::{
    hts_md5_context, hts_md5_final, hts_md5_init, hts_md5_update,
};
use std::ffi::{c_char, c_int, c_long, c_uint, c_void};

// Concurrency note (audit 2026-05):
//
// The entire `ref_cache` subsystem (this file, `ref_files.rs`, `transaction.rs`,
// `http_parser.rs`, etc.) is the back-end of the `ref-cache` daemon binary.
// The daemon's process model is fork-based (see `main.rs` and `server.rs`):
// each worker process runs a single-threaded epoll loop and is the sole
// owner of these globals. No `pthread_create` / `std::thread::spawn` is
// reachable anywhere in `src/ref_cache/`; verify with
// `grep -rn 'pthread_create\|thread::spawn' src/ref_cache/`.
//
// `CURL_VERSION_NUM` is written exactly once at the top of
// `run_upstream_handler` and is currently unread (preserved verbatim from
// the C original so future libcurl version-gated paths land cleanly). It
// is single-threaded by design.
//
// SAFETY: single-threaded daemon worker; do not introduce additional
// threads in this module without revisiting every `static mut` here.
static mut CURL_VERSION_NUM: c_uint = 0;

const MD5_LEN: usize = 32;

const DL_OK: c_int = 1;
const DL_CLENGTH: c_int = 2;
const DL_WAITING: c_int = 4;
const DL_ABANDON: c_int = 8;

const ACTIVE_SIZE: usize = 0x4000;
const ACTIVE_MASK: c_int = 0x3fff;
const MAX_EVENTS: c_int = 128;

type CURLcode = c_int;
type CURLMcode = c_int;
type CURLoption = c_int;
type CURLMoption = c_int;
type Curlinfo = c_int;
type CURLversion = c_int;
type curl_off_t = i64;
type curl_socket_t = c_int;

#[repr(C)]
pub struct CURL {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CURLM {
    _private: [u8; 0],
}

#[repr(C)]
struct curl_version_info_data {
    age: CURLversion,
    version: *const c_char,
    version_num: c_uint,
}

#[repr(C)]
struct CURLMsg {
    msg: c_int,
    easy_handle: *mut CURL,
    data: CURLMsgData,
}

#[repr(C)]
union CURLMsgData {
    whatever: *mut c_void,
    result: CURLcode,
}

const CURLE_OK: CURLcode = 0;
const CURLM_CALL_MULTI_PERFORM: CURLMcode = -1;
const CURLM_OK: CURLMcode = 0;
const CURLMSG_DONE: c_int = 1;
const CURL_POLL_IN: c_int = 1;
const CURL_POLL_OUT: c_int = 2;
const CURL_POLL_REMOVE: c_int = 4;
const CURL_SOCKET_TIMEOUT: curl_socket_t = -1;
const CURL_CSELECT_IN: c_int = 0x01;
const CURL_CSELECT_OUT: c_int = 0x02;
const CURL_CSELECT_ERR: c_int = 0x04;
const CURL_GLOBAL_ALL: c_long = 3;
const CURLVERSION_NOW: CURLversion = 9;

const CURLOPT_WRITEDATA: CURLoption = 10001;
const CURLOPT_URL: CURLoption = 10002;
const CURLOPT_WRITEFUNCTION: CURLoption = 20011;
const CURLOPT_NOPROGRESS: CURLoption = 43;
const CURLOPT_PROGRESSFUNCTION: CURLoption = 20056;
const CURLOPT_XFERINFODATA: CURLoption = 10057;
const CURLOPT_PRIVATE: CURLoption = 10103;
const CURLOPT_XFERINFOFUNCTION: CURLoption = 20219;

const CURLMOPT_SOCKETFUNCTION: CURLMoption = 20001;
const CURLMOPT_SOCKETDATA: CURLMoption = 10002;
const CURLMOPT_TIMERFUNCTION: CURLMoption = 20004;
const CURLMOPT_TIMERDATA: CURLMoption = 10005;

const CURLINFO_RESPONSE_CODE: Curlinfo = 0x200000 + 2;
const CURLINFO_CONTENT_LENGTH_DOWNLOAD_T: Curlinfo = 0x600000 + 15;
const CURLINFO_PRIVATE: Curlinfo = 0x100000 + 21;

type WriteCallback = unsafe extern "C" fn(
    buffer: *mut c_void,
    size: usize,
    nmemb: usize,
    userp: *mut c_void,
) -> usize;
type XferInfoCallback = unsafe extern "C" fn(
    clientp: *mut c_void,
    dltotal: curl_off_t,
    dlnow: curl_off_t,
    ultotal: curl_off_t,
    ulnow: curl_off_t,
) -> c_int;
type ProgressCallback = unsafe extern "C" fn(
    clientp: *mut c_void,
    dltotal: f64,
    dlnow: f64,
    ultotal: f64,
    ulnow: f64,
) -> c_int;
type SocketCallback = unsafe extern "C" fn(
    easy: *mut CURL,
    s: curl_socket_t,
    action: c_int,
    userp: *mut c_void,
    socketp: *mut c_void,
) -> c_int;
type TimerCallback =
    unsafe extern "C" fn(multi: *mut CURLM, timeout_ms: c_long, userp: *mut c_void) -> c_int;

#[link(name = "curl")]
#[allow(clashing_extern_declarations)]
extern "C" {
    #[link_name = "curl_global_init"]
    fn curl_global_init(flags: c_long) -> CURLcode;
    #[link_name = "curl_global_cleanup"]
    fn curl_global_cleanup();
    #[link_name = "curl_version_info"]
    fn curl_version_info(type_: CURLversion) -> *mut curl_version_info_data;
    #[link_name = "curl_easy_init"]
    fn curl_easy_init() -> *mut CURL;
    #[link_name = "curl_easy_cleanup"]
    fn curl_easy_cleanup(curl: *mut CURL);
    #[link_name = "curl_easy_strerror"]
    fn curl_easy_strerror(code: CURLcode) -> *const c_char;
    #[link_name = "curl_easy_setopt"]
    fn curl_easy_setopt_ptr(
        curl: *mut CURL,
        option: CURLoption,
        parameter: *mut c_void,
    ) -> CURLcode;
    #[link_name = "curl_easy_setopt"]
    fn curl_easy_setopt_cstr(
        curl: *mut CURL,
        option: CURLoption,
        parameter: *const c_char,
    ) -> CURLcode;
    #[link_name = "curl_easy_setopt"]
    fn curl_easy_setopt_long(curl: *mut CURL, option: CURLoption, parameter: c_long) -> CURLcode;
    #[link_name = "curl_easy_setopt"]
    fn curl_easy_setopt_write_callback(
        curl: *mut CURL,
        option: CURLoption,
        parameter: Option<WriteCallback>,
    ) -> CURLcode;
    #[link_name = "curl_easy_setopt"]
    fn curl_easy_setopt_xfer_callback(
        curl: *mut CURL,
        option: CURLoption,
        parameter: Option<XferInfoCallback>,
    ) -> CURLcode;
    #[link_name = "curl_easy_setopt"]
    fn curl_easy_setopt_progress_callback(
        curl: *mut CURL,
        option: CURLoption,
        parameter: Option<ProgressCallback>,
    ) -> CURLcode;
    #[link_name = "curl_easy_getinfo"]
    fn curl_easy_getinfo_long(curl: *mut CURL, info: Curlinfo, value: *mut c_long) -> CURLcode;
    #[link_name = "curl_easy_getinfo"]
    fn curl_easy_getinfo_off_t(curl: *mut CURL, info: Curlinfo, value: *mut curl_off_t)
        -> CURLcode;
    #[link_name = "curl_easy_getinfo"]
    fn curl_easy_getinfo_ptr(curl: *mut CURL, info: Curlinfo, value: *mut *mut c_void) -> CURLcode;
    #[link_name = "curl_multi_init"]
    fn curl_multi_init() -> *mut CURLM;
    #[link_name = "curl_multi_cleanup"]
    fn curl_multi_cleanup(multi_handle: *mut CURLM) -> CURLMcode;
    #[link_name = "curl_multi_strerror"]
    fn curl_multi_strerror(code: CURLMcode) -> *const c_char;
    #[link_name = "curl_multi_add_handle"]
    fn curl_multi_add_handle(multi_handle: *mut CURLM, curl_handle: *mut CURL) -> CURLMcode;
    #[link_name = "curl_multi_remove_handle"]
    fn curl_multi_remove_handle(multi_handle: *mut CURLM, curl_handle: *mut CURL) -> CURLMcode;
    #[link_name = "curl_multi_socket_action"]
    fn curl_multi_socket_action(
        multi_handle: *mut CURLM,
        s: curl_socket_t,
        ev_bitmask: c_int,
        running_handles: *mut c_int,
    ) -> CURLMcode;
    #[link_name = "curl_multi_info_read"]
    fn curl_multi_info_read(multi_handle: *mut CURLM, msgs_in_queue: *mut c_int) -> *mut CURLMsg;
    #[link_name = "curl_multi_assign"]
    fn curl_multi_assign(
        multi_handle: *mut CURLM,
        sockfd: curl_socket_t,
        sockp: *mut c_void,
    ) -> CURLMcode;
    #[link_name = "curl_multi_setopt"]
    fn curl_multi_setopt_ptr(
        multi_handle: *mut CURLM,
        option: CURLMoption,
        parameter: *mut c_void,
    ) -> CURLMcode;
    #[link_name = "curl_multi_setopt"]
    fn curl_multi_setopt_socket_callback(
        multi_handle: *mut CURLM,
        option: CURLMoption,
        parameter: Option<SocketCallback>,
    ) -> CURLMcode;
    #[link_name = "curl_multi_setopt"]
    fn curl_multi_setopt_timer_callback(
        multi_handle: *mut CURLM,
        option: CURLMoption,
        parameter: Option<TimerCallback>,
    ) -> CURLMcode;
}

// original: Upstream_msg_code (htslib/ref_cache/upstream.h:33)
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Upstream_msg_code {
    US_START,
    US_CONTENT_LENGTH,
    US_PARTIAL_LENGTH,
    US_RESULT,
}

// original: Upstream_msg (htslib/ref_cache/upstream.h:39)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Upstream_msg {
    pub id: c_uint,
    pub code: Upstream_msg_code,
    pub val: i64,
}

// original: Download (htslib/ref_cache/upstream.c:74)
//
// The C `Download` was an intrusive hash-table node whose raw address was also
// handed to libcurl via CURLOPT_PRIVATE / CURLOPT_WRITEDATA / CURLOPT_XFERINFODATA
// and read back out of curl callbacks. We restructure it into an OWNED ARENA
// entry: downloads live in `Multi_data::downloads_arena` (a `Vec<Option<Download>>`)
// and the former `next` / `waiting` raw links are now `Option<usize>` indices
// into that arena. The `downstream` head is an index into the downstream arena.
// What libcurl carries across the FFI boundary is the integer download index,
// not a pointer; the callbacks recover `idx = userp as usize` and look the entry
// up in the arena reached through the single-worker `MDATA` pointer.
//
// `curl: *mut CURL` stays raw: it is a genuine opaque libcurl handle with no
// Rust equivalent.
pub struct Download {
    hexmd5: [u8; MD5_LEN],
    md5_ctx: Option<Box<hts_md5_context>>,
    next: Option<usize>,
    waiting: Option<usize>,
    downstream: Option<usize>,
    cache_dir: Vec<u8>,
    file: Vec<u8>,
    url: Vec<u8>,
    curl: *mut CURL,
    cmd_fd: c_int,
    curlid: c_int,
    flags: c_int,
    cache_fd: c_int,
    file_fd: c_int,
    size: libc::off_t,
    received: libc::off_t,
}

// original: Downstream (htslib/ref_cache/upstream.c:94)
//
// Restructured from a raw doubly-linked list node into an owned arena entry; the
// `download` / `prev` / `next` raw pointers are now `Option<usize>` indices.
pub struct Downstream {
    download: Option<usize>,
    prev: Option<usize>,
    next: Option<usize>,
    cmd_fd: c_int,
    id: c_uint,
}

// original: Multi_data (htslib/ref_cache/upstream.c:106)
//
// Owns both arenas. `downloads` is the hash table: each bucket is the head
// download index of an intrusive `next` chain. `downloads_arena` /
// `downstream_arena` are the backing storage; `*_free` recycle vacated slots.
pub struct Multi_data {
    multi: *mut CURLM,
    pw: Option<Box<poll_impl::Poll_wrap>>,
    timeout: c_long,
    downloads: Vec<Option<usize>>,
    downloads_arena: Vec<Option<Download>>,
    download_free: Vec<usize>,
    downstream_arena: Vec<Option<Downstream>>,
    downstream_free: Vec<usize>,
    waiting: Option<usize>,
    last_waiting: Option<usize>,
    ncurls: c_uint,
    free_curls: c_uint,
    running: c_int,
    curls: Vec<*mut CURL>,
}

// SAFETY: single-threaded daemon worker (see the concurrency note at the top of
// this file). `MDATA` points at the one `Multi_data` owned by the current worker
// for the duration of `run_multi_upstream_handler`; libcurl callbacks, which can
// only carry an integer index across the C boundary, recover the owning arena
// through it. It is the genuine FFI/OS boundary that replaces the old
// `*mut Download` / `*mut Multi_data` userp round-trips.
static mut MDATA: *mut Multi_data = std::ptr::null_mut();

// `mdata!()` borrows the current worker's owning `Multi_data`; `dl!(i)` /
// `ds!(i)` borrow the live download / downstream in arena slot `i`. Macros (not
// helper functions) so they expand inline at the call sites.
macro_rules! mdata {
    () => {
        (*MDATA)
    };
}
macro_rules! dl {
    ($i:expr) => {
        (&mut mdata!().downloads_arena)[$i].as_mut().expect("download slot occupied")
    };
}
macro_rules! ds {
    ($i:expr) => {
        (&mut mdata!().downstream_arena)[$i].as_mut().expect("downstream slot occupied")
    };
}

// original: upstream_send_cmd (htslib/ref_cache/upstream.c:122)
pub unsafe fn ref_cache_upstream_c_122_upstream_send_cmd(
    cmd_fd: c_int,
    hexmd5: &[u8],
    mut id: c_uint,
) -> c_int {
    let mut msg: libc::msghdr = std::mem::zeroed();
    let mut iov = [
        libc::iovec {
            iov_base: hexmd5.as_ptr().cast_mut().cast(),
            iov_len: MD5_LEN,
        },
        libc::iovec {
            iov_base: (&mut id as *mut c_uint).cast(),
            iov_len: std::mem::size_of_val(&id),
        },
    ];

    msg.msg_name = std::ptr::null_mut();
    msg.msg_control = std::ptr::null_mut();
    msg.msg_iov = iov.as_mut_ptr();
    msg.msg_iovlen = 2;

    let mut res;
    loop {
        res = libc::sendmsg(cmd_fd, &msg, 0);
        if !(res == -1
            && (*__errno_location() == libc::EINTR
                || *__errno_location() == libc::EAGAIN
                || *__errno_location() == libc::EWOULDBLOCK))
        {
            break;
        }
    }
    if res == (MD5_LEN + std::mem::size_of::<c_int>()) as libc::ssize_t {
        0
    } else {
        -1
    }
}

// original: recv_cmd_data (htslib/ref_cache/upstream.c:146)
unsafe fn ref_cache_upstream_c_146_recv_cmd_data(
    cmd_fd: c_int,
    hexmd5: &mut [u8],
    id: &mut c_uint,
) -> libc::ssize_t {
    let mut msg: libc::msghdr = std::mem::zeroed();
    let mut iov = [
        libc::iovec {
            iov_base: hexmd5.as_mut_ptr().cast(),
            iov_len: MD5_LEN,
        },
        libc::iovec {
            iov_base: (id as *mut c_uint).cast(),
            iov_len: std::mem::size_of::<c_uint>(),
        },
    ];

    msg.msg_name = std::ptr::null_mut();
    msg.msg_control = std::ptr::null_mut();
    msg.msg_iov = iov.as_mut_ptr();
    msg.msg_iovlen = 2;

    let mut res;
    loop {
        res = libc::recvmsg(cmd_fd, &mut msg, 0);
        if !(res == -1
            && (*__errno_location() == libc::EINTR
                || *__errno_location() == libc::EAGAIN
                || *__errno_location() == libc::EWOULDBLOCK))
        {
            break;
        }
    }
    if res == 0 {
        return 0;
    }
    if res != (MD5_LEN + std::mem::size_of::<c_uint>()) as libc::ssize_t {
        return -1;
    }
    res
}

// original: upstream_send_msg (htslib/ref_cache/upstream.c:172)
unsafe fn ref_cache_upstream_c_172_upstream_send_msg(
    cmd_fd: c_int,
    umsg: &Upstream_msg,
    fd: c_int,
) -> c_int {
    let mut msg: libc::msghdr = std::mem::zeroed();
    let mut iov = [libc::iovec {
        iov_base: (umsg as *const Upstream_msg as *mut Upstream_msg).cast(),
        iov_len: std::mem::size_of::<Upstream_msg>(),
    }];
    let mut buf = [0u8; 256];

    msg.msg_name = std::ptr::null_mut();
    msg.msg_control = std::ptr::null_mut();
    msg.msg_iov = iov.as_mut_ptr();
    msg.msg_iovlen = 1;

    if umsg.code == Upstream_msg_code::US_START
        && ref_cache_cmsg_wrap_c_46_make_scm_rights_cmsg(&mut msg, fd, &mut buf) < 0
    {
        eprintln!("upstream_send_msg: cmsg buffer not big enough.");
        return -1;
    }

    let mut res;
    loop {
        res = libc::sendmsg(cmd_fd, &msg, 0);
        if !(res == -1
            && (*__errno_location() == libc::EINTR
                || *__errno_location() == libc::EAGAIN
                || *__errno_location() == libc::EWOULDBLOCK))
        {
            break;
        }
    }

    if res == std::mem::size_of::<Upstream_msg>() as libc::ssize_t {
        0
    } else {
        -1
    }
}

// original: send_msg_all (htslib/ref_cache/upstream.c:204)
unsafe fn ref_cache_upstream_c_204_send_msg_all(
    download: usize,
    code: Upstream_msg_code,
    val: i64,
) -> c_int {
    let mut d = dl!(download).downstream;
    let mut res = 0;
    while let Some(cur) = d {
        let msg = Upstream_msg {
            id: ds!(cur).id,
            code,
            val,
        };
        if ref_cache_upstream_c_172_upstream_send_msg(ds!(cur).cmd_fd, &msg, -1) < 0 {
            res = -1;
        }
        d = ds!(cur).next;
    }
    res
}

// original: upstream_recv_msg (htslib/ref_cache/upstream.c:215)
pub unsafe fn ref_cache_upstream_c_215_upstream_recv_msg(
    cmd_fd: c_int,
    umsg: &mut Upstream_msg,
    fd: &mut c_int,
) -> c_int {
    let mut msg: libc::msghdr = std::mem::zeroed();
    let mut iov = [libc::iovec {
        iov_base: (umsg as *mut Upstream_msg).cast(),
        iov_len: std::mem::size_of::<Upstream_msg>(),
    }];
    let mut buf = [0u8; 16384];

    msg.msg_name = std::ptr::null_mut();
    msg.msg_control = std::ptr::null_mut();
    msg.msg_iov = iov.as_mut_ptr();
    msg.msg_iovlen = 1;
    msg.msg_control = buf.as_mut_ptr().cast();
    msg.msg_controllen = buf.len();
    *umsg = Upstream_msg {
        id: 0,
        code: Upstream_msg_code::US_START,
        val: 0,
    };

    let mut res;
    loop {
        res = libc::recvmsg(cmd_fd, &mut msg, 0);
        if !(res == -1
            && (*__errno_location() == libc::EINTR
                || *__errno_location() == libc::EAGAIN
                || *__errno_location() == libc::EWOULDBLOCK))
        {
            break;
        }
    }
    if res == 0 {
        return 0;
    }
    if res != std::mem::size_of::<Upstream_msg>() as libc::ssize_t {
        return -1;
    }

    if umsg.code == Upstream_msg_code::US_START {
        *fd = ref_cache_cmsg_wrap_c_67_get_scm_rights_fd(&mut msg);
        if *fd < 0 {
            eprintln!("Failed to get file descriptor in upstream message");
            return -1;
        }
    }

    1
}

// original: make_subdir (htslib/ref_cache/upstream.c:252)
unsafe fn ref_cache_upstream_c_252_make_subdir(opts: &Options, hexmd5: &[u8]) -> c_int {
    let cache_dir = opts.cache_dir.as_deref().unwrap_or(b"");

    // NUL-terminated only at the mkdirat(2) boundary; the path content itself
    // is assembled from owned byte slices.
    let mut path = [0u8; 6];

    path[..2].copy_from_slice(&hexmd5[..2]);
    path[2] = 0;
    if libc::mkdirat((*opts).cache_fd, path.as_ptr().cast(), 0o1755) != 0
        && *__errno_location() != libc::EEXIST
    {
        eprintln!(
            "Couldn't make directory {}/{} : {}",
            String::from_utf8_lossy(cache_dir),
            String::from_utf8_lossy(&path[..2]),
            std::io::Error::last_os_error()
        );
        return -1;
    }

    path[2] = b'/';
    path[3..5].copy_from_slice(&hexmd5[2..4]);
    path[5] = 0;
    if libc::mkdirat((*opts).cache_fd, path.as_ptr().cast(), 0o1755) != 0
        && *__errno_location() != libc::EEXIST
    {
        eprintln!(
            "Couldn't make directory {}/{} : {}",
            String::from_utf8_lossy(cache_dir),
            String::from_utf8_lossy(&path[..5]),
            std::io::Error::last_os_error()
        );
        return -1;
    }
    0
}

// original: get_free_curl (htslib/ref_cache/upstream.c:277)
unsafe fn ref_cache_upstream_c_277_get_free_curl() -> c_int {
    if mdata!().free_curls == 0 {
        return -1;
    }
    for i in 0..mdata!().ncurls {
        if (mdata!().free_curls & (1u32 << i)) != 0 {
            mdata!().free_curls &= !(1u32 << i);
            return i as c_int;
        }
    }
    -1
}

// original: release_curl (htslib/ref_cache/upstream.c:290)
unsafe fn ref_cache_upstream_c_290_release_curl(download: usize) {
    let curlid = dl!(download).curlid;
    mdata!().free_curls |= 1u32 << curlid;
    dl!(download).curlid = -1;
    dl!(download).curl = std::ptr::null_mut();
}

// original: new_downstream (htslib/ref_cache/upstream.c:296)
//
// Allocates an owned downstream arena slot and returns its index.
unsafe fn ref_cache_upstream_c_296_new_downstream(cmd_fd: c_int, downstream_id: c_uint) -> usize {
    let ds = Downstream {
        download: None,
        prev: None,
        next: None,
        cmd_fd,
        id: downstream_id,
    };
    if let Some(slot) = (&mut mdata!().downstream_free).pop() {
        (&mut mdata!().downstream_arena)[slot] = Some(ds);
        slot
    } else {
        (&mut mdata!().downstream_arena).push(Some(ds));
        (&mdata!().downstream_arena).len() - 1
    }
}

// original: new_download (htslib/ref_cache/upstream.c:306)
//
// Allocates an owned download arena slot and returns its index.
pub unsafe fn ref_cache_upstream_c_306_new_download(opts: &Options, hexmd5: &[u8]) -> usize {
    let mut download = Download {
        hexmd5: [0; MD5_LEN],
        md5_ctx: None,
        next: None,
        waiting: None,
        downstream: None,
        cache_dir: opts.cache_dir.clone().unwrap_or_default(),
        file: Vec::new(),
        url: Vec::new(),
        curl: std::ptr::null_mut(),
        cmd_fd: 0,
        curlid: -1,
        flags: 0,
        cache_fd: opts.cache_fd,
        file_fd: -1,
        size: 0,
        received: 0,
    };
    download.hexmd5.copy_from_slice(&hexmd5[..MD5_LEN]);
    if let Some(slot) = (&mut mdata!().download_free).pop() {
        (&mut mdata!().downloads_arena)[slot] = Some(download);
        slot
    } else {
        (&mut mdata!().downloads_arena).push(Some(download));
        (&mdata!().downloads_arena).len() - 1
    }
}

// original: remove_downstream (htslib/ref_cache/upstream.c:319)
unsafe fn ref_cache_upstream_c_319_remove_downstream(downstream: usize, download: usize) {
    let prev = ds!(downstream).prev;
    let next = ds!(downstream).next;
    match prev {
        None => dl!(download).downstream = next,
        Some(prev) => {
            assert!(Some(downstream) != dl!(download).downstream);
            ds!(prev).next = next;
        }
    }
    if let Some(next) = next {
        ds!(next).prev = prev;
    }
    // Return the slot to the downstream arena.
    (&mut mdata!().downstream_arena)[downstream] = None;
    (&mut mdata!().downstream_free).push(downstream);
}

// original: free_download (htslib/ref_cache/upstream.c:331)
unsafe fn ref_cache_upstream_c_331_free_download(download: usize) {
    while let Some(ds) = dl!(download).downstream {
        ref_cache_upstream_c_319_remove_downstream(ds, download);
    }

    let hash = ((ref_cache_misc_h_38_hexval(dl!(download).hexmd5[0]) << 12)
        | (ref_cache_misc_h_38_hexval(dl!(download).hexmd5[1]) << 8)
        | (ref_cache_misc_h_38_hexval(dl!(download).hexmd5[2]) << 4)
        | ref_cache_misc_h_38_hexval(dl!(download).hexmd5[3]))
        & ACTIVE_MASK;
    let next = dl!(download).next;
    if (&mdata!().downloads)[hash as usize] == Some(download) {
        (&mut mdata!().downloads)[hash as usize] = next;
    } else {
        let mut d = (&mdata!().downloads)[hash as usize];
        while let Some(cur) = d {
            if dl!(cur).next == Some(download) {
                break;
            }
            d = dl!(cur).next;
        }
        if let Some(cur) = d {
            dl!(cur).next = next;
        }
    }

    if (dl!(download).flags & DL_WAITING) != 0 {
        let waiting = dl!(download).waiting;
        if (&mdata!().waiting) == &Some(download) {
            if (&mdata!().last_waiting) == &Some(download) {
                mdata!().waiting = None;
                mdata!().last_waiting = None;
            } else {
                mdata!().waiting = waiting;
            }
        } else {
            let mut d = mdata!().waiting;
            while let Some(cur) = d {
                if dl!(cur).waiting == Some(download) {
                    break;
                }
                d = dl!(cur).waiting;
            }
            if let Some(cur) = d {
                if (&mdata!().last_waiting) == &Some(download) {
                    mdata!().last_waiting = Some(cur);
                }
                dl!(cur).waiting = waiting;
            }
        }
    }

    if dl!(download).file_fd != -1 {
        libc::close(dl!(download).file_fd);
        if (dl!(download).flags & DL_OK) == 0 {
            // NUL-terminate only at the unlinkat(2) boundary.
            let mut path = dl!(download).file.clone();
            path.push(0);
            let cache_fd = dl!(download).cache_fd;
            libc::unlinkat(cache_fd, path.as_ptr().cast(), 0);
        }
    }
    if dl!(download).curlid != -1 {
        ref_cache_upstream_c_290_release_curl(download);
    }
    // The boxed md5 context (if any) drops with the arena slot below.
    (&mut mdata!().downloads_arena)[download] = None;
    (&mut mdata!().download_free).push(download);
}

// original: start_new_download (htslib/ref_cache/upstream.c:385)
unsafe fn ref_cache_upstream_c_385_start_new_download(multi: *mut CURLM) -> c_int {
    let Some(download) = mdata!().waiting else {
        return 0;
    };
    assert!((dl!(download).flags & DL_WAITING) != 0);

    let curlid = ref_cache_upstream_c_277_get_free_curl();
    dl!(download).curlid = curlid;
    if curlid == -1 {
        return 0;
    }

    let curl = (&mdata!().curls)[curlid as usize];
    dl!(download).curl = curl;
    let waiting = dl!(download).waiting;
    mdata!().waiting = waiting;
    if (&mdata!().last_waiting) == &Some(download) {
        mdata!().last_waiting = None;
    }
    dl!(download).flags &= !DL_WAITING;

    // libcurl requires a NUL-terminated C string for the URL; terminate only
    // at this boundary, the stored `url` carries no trailing NUL.
    let mut url_c = dl!(download).url.clone();
    url_c.push(0);
    let mut cc = curl_easy_setopt_cstr(curl, CURLOPT_URL, url_c.as_ptr().cast());
    if cc != CURLE_OK {
        eprintln!(
            "Couldn't set URL {} : {}",
            String::from_utf8_lossy(&dl!(download).url),
            std::ffi::CStr::from_ptr(curl_easy_strerror(cc)).to_string_lossy()
        );
        ref_cache_upstream_c_331_free_download(download);
        return -1;
    }
    // CURLOPT_WRITEDATA / CURLOPT_PRIVATE / CURLOPT_XFERINFODATA all carry the
    // integer download arena index across the FFI boundary (cast to a void*),
    // NOT a Rust borrow; the callbacks recover it with `userp as usize`.
    cc = curl_easy_setopt_ptr(curl, CURLOPT_WRITEDATA, download as *mut c_void);
    if cc != CURLE_OK {
        eprintln!(
            "Couldn't set user data in CURL handle : {}",
            std::ffi::CStr::from_ptr(curl_easy_strerror(cc)).to_string_lossy()
        );
        ref_cache_upstream_c_331_free_download(download);
        return -1;
    }
    cc = curl_easy_setopt_ptr(curl, CURLOPT_PRIVATE, download as *mut c_void);
    if cc != CURLE_OK {
        eprintln!(
            "Couldn't set private data in CURL handle : {}",
            std::ffi::CStr::from_ptr(curl_easy_strerror(cc)).to_string_lossy()
        );
        ref_cache_upstream_c_331_free_download(download);
        return -1;
    }
    cc = curl_easy_setopt_ptr(curl, CURLOPT_XFERINFODATA, download as *mut c_void);
    if cc != CURLE_OK {
        eprintln!(
            "Couldn't set progress data in CURL handle : {}",
            std::ffi::CStr::from_ptr(curl_easy_strerror(cc)).to_string_lossy()
        );
        ref_cache_upstream_c_331_free_download(download);
        return -1;
    }

    let mc = curl_multi_add_handle(multi, curl);
    if mc != CURLM_OK {
        eprintln!(
            "Couldn't add handle to curl_multi : {}",
            std::ffi::CStr::from_ptr(curl_multi_strerror(mc)).to_string_lossy()
        );
        ref_cache_upstream_c_331_free_download(download);
        return -1;
    }
    mdata!().running += 1;
    1
}

// original: get_download (htslib/ref_cache/upstream.c:445)
unsafe fn ref_cache_upstream_c_445_get_download(
    opts: &Options,
    multi: *mut CURLM,
    hexmd5: &[u8],
) -> Option<usize> {
    let cache_dir = opts.cache_dir.as_deref().unwrap_or(b"");
    let hash = ((ref_cache_misc_h_38_hexval(hexmd5[0]) << 12)
        | (ref_cache_misc_h_38_hexval(hexmd5[1]) << 8)
        | (ref_cache_misc_h_38_hexval(hexmd5[2]) << 4)
        | ref_cache_misc_h_38_hexval(hexmd5[3]))
        & ACTIVE_MASK;

    let mut cur = (&mdata!().downloads)[hash as usize];
    while let Some(download) = cur {
        if dl!(download).hexmd5[..] == hexmd5[..MD5_LEN] {
            return Some(download);
        }
        cur = dl!(download).next;
    }

    let download = ref_cache_upstream_c_306_new_download(opts, hexmd5);

    // "aa/bb/cccc..." (md5[0..2] / md5[2..4] / md5[4..32]); no trailing NUL.
    let mut file = Vec::with_capacity(MD5_LEN + 16);
    file.extend_from_slice(&dl!(download).hexmd5[0..2]);
    file.push(b'/');
    file.extend_from_slice(&dl!(download).hexmd5[2..4]);
    file.push(b'/');
    file.extend_from_slice(&dl!(download).hexmd5[4..MD5_LEN]);
    dl!(download).file = file;

    // NUL-terminate only at the openat(2) boundary.
    let mut file_c = dl!(download).file.clone();
    file_c.push(0);
    let file_fd = libc::openat(opts.cache_fd, file_c.as_ptr().cast(), libc::O_RDONLY);
    dl!(download).file_fd = file_fd;
    if file_fd >= 0 {
        let mut st: libc::stat = std::mem::zeroed();
        if libc::fstat(file_fd, &mut st) != 0 {
            eprintln!(
                "Couldn't stat {}/{} : {}",
                String::from_utf8_lossy(cache_dir),
                String::from_utf8_lossy(&dl!(download).file),
                std::io::Error::last_os_error()
            );
            ref_cache_upstream_c_331_free_download(download);
            return None;
        }
        dl!(download).size = st.st_size;
        dl!(download).received = st.st_size;
        dl!(download).flags = DL_OK | DL_CLENGTH;
        dl!(download).next = (&mdata!().downloads)[hash as usize];
        (&mut mdata!().downloads)[hash as usize] = Some(download);
        return Some(download);
    }

    if *__errno_location() != libc::ENOENT {
        eprintln!(
            "Couldn't open {}/{} : {}",
            String::from_utf8_lossy(cache_dir),
            String::from_utf8_lossy(&dl!(download).file),
            std::io::Error::last_os_error()
        );
        ref_cache_upstream_c_331_free_download(download);
        return None;
    }

    let upstream_url = opts.upstream_url.as_deref().unwrap_or(b"");
    // Join upstream URL and the 32-char md5, inserting a '/' unless the URL
    // already ends with one; no trailing NUL.
    let mut url = Vec::with_capacity(upstream_url.len() + MD5_LEN + 2);
    url.extend_from_slice(upstream_url);
    if upstream_url.last() != Some(&b'/') {
        url.push(b'/');
    }
    url.extend_from_slice(&hexmd5[..MD5_LEN]);
    dl!(download).url = url;

    if ref_cache_upstream_c_252_make_subdir(opts, hexmd5) != 0 {
        ref_cache_upstream_c_331_free_download(download);
        return None;
    }

    for count in 0..1000 {
        // "aa/bb/cccc....NNN" with a zero-padded 3-digit suffix; no trailing NUL.
        let mut file = Vec::with_capacity(MD5_LEN + 16);
        file.extend_from_slice(&hexmd5[0..2]);
        file.push(b'/');
        file.extend_from_slice(&hexmd5[2..4]);
        file.push(b'/');
        file.extend_from_slice(&hexmd5[4..MD5_LEN]);
        file.extend_from_slice(format!(".{:03}", count).as_bytes());
        dl!(download).file = file;
        // NUL-terminate only at the openat(2) boundary.
        let mut file_c = dl!(download).file.clone();
        file_c.push(0);
        loop {
            let fd = libc::openat(
                opts.cache_fd,
                file_c.as_ptr().cast(),
                libc::O_RDWR | libc::O_CREAT | libc::O_EXCL,
                0o644,
            );
            dl!(download).file_fd = fd;
            if !(fd == -1 && *__errno_location() == libc::EINTR) {
                break;
            }
        }
        if dl!(download).file_fd >= 0 {
            break;
        }
        if *__errno_location() != libc::EEXIST {
            break;
        }
    }
    if dl!(download).file_fd == -1 {
        eprintln!(
            "Couldn't open {}/{} for writing: {}",
            String::from_utf8_lossy(cache_dir),
            String::from_utf8_lossy(&dl!(download).file),
            std::io::Error::last_os_error()
        );
        ref_cache_upstream_c_331_free_download(download);
        return None;
    }

    dl!(download).md5_ctx = Some(hts_md5_init());

    dl!(download).waiting = None;
    if let Some(last) = mdata!().last_waiting {
        dl!(last).waiting = Some(download);
    }
    if (&mdata!().waiting).is_none() {
        mdata!().waiting = Some(download);
    }
    mdata!().last_waiting = Some(download);
    dl!(download).flags |= DL_WAITING;

    if ref_cache_upstream_c_385_start_new_download(multi) < 0 {
        return None;
    }

    dl!(download).next = (&mdata!().downloads)[hash as usize];
    (&mut mdata!().downloads)[hash as usize] = Some(download);
    Some(download)
}

// original: get_cmd_multi (htslib/ref_cache/upstream.c:563)
unsafe fn ref_cache_upstream_c_563_get_cmd_multi(
    cmd_fd: c_int,
    opts: &Options,
    multi: *mut CURLM,
    running: c_int,
) -> c_int {
    let mut hexmd5 = [0u8; MD5_LEN];
    let mut downstream: Option<usize> = None;
    let mut msg = Upstream_msg {
        id: 0,
        code: Upstream_msg_code::US_RESULT,
        val: 0,
    };
    let res = -1;
    let mut downstream_id: c_uint = 0;

    let clen = ref_cache_upstream_c_146_recv_cmd_data(cmd_fd, &mut hexmd5, &mut downstream_id);
    if clen < 0 {
        return -1;
    }
    if clen == 0 {
        return 0;
    }

    if running == 0 {
        msg.id = downstream_id;
        msg.code = Upstream_msg_code::US_RESULT;
        msg.val = 503;
        ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &msg, -1);
    }

    let Some(download) = ref_cache_upstream_c_445_get_download(opts, multi, &hexmd5) else {
        if res != 1 {
            msg.id = downstream_id;
            msg.code = Upstream_msg_code::US_RESULT;
            msg.val = 500;
            ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &msg, -1);
            if let Some(ds) = downstream {
                (&mut mdata!().downstream_arena)[ds] = None;
                (&mut mdata!().downstream_free).push(ds);
            }
        }
        return res;
    };

    downstream = dl!(download).downstream;
    while let Some(cur) = downstream {
        if ds!(cur).cmd_fd == cmd_fd {
            break;
        }
        downstream = ds!(cur).next;
    }
    if downstream.is_some() {
        return 1;
    }

    let ds_idx = ref_cache_upstream_c_296_new_downstream(cmd_fd, downstream_id);
    downstream = Some(ds_idx);

    let downstream_fd = libc::dup(dl!(download).file_fd);
    if downstream_fd < 0 {
        if res != 1 {
            msg.id = downstream_id;
            msg.code = Upstream_msg_code::US_RESULT;
            msg.val = 500;
            ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &msg, -1);
            if let Some(ds) = downstream {
                (&mut mdata!().downstream_arena)[ds] = None;
                (&mut mdata!().downstream_free).push(ds);
            }
        }
        return res;
    }

    msg.id = downstream_id;
    msg.code = Upstream_msg_code::US_START;
    msg.val = if (dl!(download).flags & DL_CLENGTH) != 0 {
        dl!(download).size
    } else {
        -1
    };
    if ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &msg, downstream_fd) < 0 {
        libc::close(downstream_fd);
        if res != 1 {
            msg.id = downstream_id;
            msg.code = Upstream_msg_code::US_RESULT;
            msg.val = 500;
            ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &msg, -1);
            if let Some(ds) = downstream {
                (&mut mdata!().downstream_arena)[ds] = None;
                (&mut mdata!().downstream_free).push(ds);
            }
        }
        return res;
    }
    libc::close(downstream_fd);

    let head = dl!(download).downstream;
    ds!(ds_idx).download = Some(download);
    ds!(ds_idx).prev = None;
    ds!(ds_idx).next = head;
    if let Some(h) = head {
        ds!(h).prev = Some(ds_idx);
    }
    dl!(download).downstream = Some(ds_idx);
    1
}

// original: send_result_code (htslib/ref_cache/upstream.c:635)
unsafe fn ref_cache_upstream_c_635_send_result_code(downstream: usize, code: c_int) -> c_int {
    let msg = Upstream_msg {
        id: ds!(downstream).id,
        code: Upstream_msg_code::US_RESULT,
        val: code as i64,
    };
    if ref_cache_upstream_c_172_upstream_send_msg(ds!(downstream).cmd_fd, &msg, -1) != 0 {
        eprintln!(
            "Error sending result code to downstream #{} : {}",
            ds!(downstream).id,
            std::io::Error::last_os_error()
        );
        return -1;
    }
    0
}

// original: send_result_code_all (htslib/ref_cache/upstream.c:646)
unsafe fn ref_cache_upstream_c_646_send_result_code_all(download: usize, code: c_int) -> c_int {
    let mut res = 0;
    let mut d = dl!(download).downstream;
    while let Some(cur) = d {
        res |= ref_cache_upstream_c_635_send_result_code(cur, code);
        d = ds!(cur).next;
    }
    res
}

// original: progress_callback (htslib/ref_cache/upstream.c:656)
unsafe extern "C" fn ref_cache_upstream_c_656_progress_callback(
    clientp: *mut c_void,
    _dltotal: curl_off_t,
    _dlnow: curl_off_t,
    _ultotal: curl_off_t,
    _ulnow: curl_off_t,
) -> c_int {
    // `clientp` carries the download arena index, not a pointer.
    let download = clientp as usize;
    if (dl!(download).flags & DL_ABANDON) == 0 {
        0
    } else {
        1
    }
}

// original: progress_callback (htslib/ref_cache/upstream.c:666)
unsafe extern "C" fn ref_cache_upstream_c_666_progress_callback(
    clientp: *mut c_void,
    _dltotal: f64,
    _dlnow: f64,
    _ultotal: f64,
    _ulnow: f64,
) -> c_int {
    let download = clientp as usize;
    if (dl!(download).flags & DL_ABANDON) == 0 {
        0
    } else {
        1
    }
}

// original: multi_receive_data (htslib/ref_cache/upstream.c:675)
unsafe extern "C" fn ref_cache_upstream_c_675_multi_receive_data(
    buffer: *mut c_void,
    size: usize,
    nmemb: usize,
    userp: *mut c_void,
) -> usize {
    // `userp` carries the download arena index, not a pointer.
    let download = userp as usize;
    let bytes = size * nmemb;
    // curl hands us a writable buffer of `bytes` octets; view it as a slice.
    let data = std::slice::from_raw_parts_mut(buffer.cast::<u8>(), bytes);

    if (dl!(download).flags & DL_ABANDON) != 0 {
        return bytes;
    }

    if (dl!(download).flags & DL_CLENGTH) == 0 {
        let mut clen: curl_off_t = 0;
        let mut rcode: c_long = 500;
        let curl = dl!(download).curl;
        if curl_easy_getinfo_long(curl, CURLINFO_RESPONSE_CODE, &mut rcode) != CURLE_OK {
            return 0;
        }
        if rcode != 200 {
            if ref_cache_upstream_c_646_send_result_code_all(download, rcode as c_int) != 0 {
                return 0;
            }
            dl!(download).flags |= DL_ABANDON;
            return bytes;
        }
        if curl_easy_getinfo_off_t(curl, CURLINFO_CONTENT_LENGTH_DOWNLOAD_T, &mut clen) != CURLE_OK {
            return 0;
        }
        dl!(download).flags |= DL_CLENGTH;
        dl!(download).size = clen as libc::off_t;

        let size_val = dl!(download).size;
        if ref_cache_upstream_c_204_send_msg_all(
            download,
            Upstream_msg_code::US_CONTENT_LENGTH,
            size_val,
        ) != 0
        {
            return 0;
        }
    }

    let file_fd = dl!(download).file_fd;
    if ref_cache_misc_h_55_do_write_all(file_fd, data) != 0 {
        eprintln!(
            "Error writing to {}/{} : {}",
            String::from_utf8_lossy(&dl!(download).cache_dir),
            String::from_utf8_lossy(&dl!(download).file),
            std::io::Error::last_os_error()
        );
        return 0;
    }

    // Strip ASCII whitespace and upper-case the remaining bytes in place, then
    // feed the compacted prefix to the running MD5 via the owned context API.
    let mut out = 0usize;
    for in_ in 0..bytes {
        let b = data[in_];
        if !b.is_ascii_whitespace() {
            data[out] = b.to_ascii_uppercase();
            out += 1;
        }
    }
    if out > 0 {
        if let Some(ctx) = dl!(download).md5_ctx.as_mut() {
            hts_md5_update(ctx, &data[..out], out);
        }
    }

    dl!(download).received += bytes as libc::off_t;
    let received = dl!(download).received;
    if ref_cache_upstream_c_204_send_msg_all(
        download,
        Upstream_msg_code::US_PARTIAL_LENGTH,
        received,
    ) != 0
    {
        return 0;
    }

    bytes
}

// original: sock_func (htslib/ref_cache/upstream.c:753)
unsafe extern "C" fn ref_cache_upstream_c_753_sock_func(
    _easy: *mut CURL,
    s: curl_socket_t,
    action: c_int,
    userp: *mut c_void,
    socketp: *mut c_void,
) -> c_int {
    // CURLMOPT_SOCKETDATA still hands back the owning `Multi_data` (the single
    // per-worker structure); curl_multi_assign stashes a poller arena index + 1
    // per-socket, recovered here (0 meaning "not yet registered").
    let mdata = userp.cast::<Multi_data>();
    let pw = (*mdata).pw.as_deref_mut().expect("poller initialised");
    let polled = if socketp.is_null() {
        None
    } else {
        Some(socketp as usize - 1)
    };

    if action == CURL_POLL_REMOVE {
        let polled = polled.expect("curl removes a registered socket");
        let res = poll_impl::ref_cache_poll_wrap_epoll_c_126_pw_remove(pw, polled, false);
        if res != 0 {
            if *__errno_location() == libc::EBADF {
                return 0;
            }
            eprintln!(
                "Removing file descriptor from poller: {}",
                std::io::Error::last_os_error()
            );
            return res;
        }
        let mc = curl_multi_assign((*mdata).multi, s, std::ptr::null_mut());
        if mc != CURLM_OK {
            eprintln!(
                "curl_multi_assign failed : {}",
                std::ffi::CStr::from_ptr(curl_multi_strerror(mc)).to_string_lossy()
            );
            return -1;
        }
        return 0;
    }

    let events = (if (action & CURL_POLL_IN) != 0 {
        PW_IN
    } else {
        0
    }) | (if (action & CURL_POLL_OUT) != 0 {
        PW_OUT
    } else {
        0
    });

    match polled {
        None => {
            let Some(polled) = poll_impl::ref_cache_poll_wrap_epoll_c_78_pw_register(
                pw,
                s,
                Pw_fd_type::US_CURL,
                events as u32,
                0,
            ) else {
                return -1;
            };
            // Store index+1 so that a null socketp (0) still means "unregistered".
            let mc = curl_multi_assign((*mdata).multi, s, (polled + 1) as *mut c_void);
            if mc != CURLM_OK {
                eprintln!(
                    "curl_multi_assign failed : {}",
                    std::ffi::CStr::from_ptr(curl_multi_strerror(mc)).to_string_lossy()
                );
                return -1;
            }
            0
        }
        Some(polled) => {
            poll_impl::ref_cache_poll_wrap_epoll_c_106_pw_mod(pw, polled, events as u32)
        }
    }
}

// original: timer_func (htslib/ref_cache/upstream.c:801)
unsafe extern "C" fn ref_cache_upstream_c_801_timer_func(
    _multi: *mut CURLM,
    timeout: c_long,
    userp: *mut c_void,
) -> c_int {
    let mdata = userp.cast::<Multi_data>();
    (*mdata).timeout = timeout;
    0
}

// original: get_multi (htslib/ref_cache/upstream.c:808)
unsafe fn ref_cache_upstream_c_808_get_multi(mdata: *mut Multi_data) -> *mut CURLM {
    let multi = curl_multi_init();
    if multi.is_null() {
        eprint!("curl_multi_init() failed");
        return std::ptr::null_mut();
    }

    let mut mc = curl_multi_setopt_socket_callback(
        multi,
        CURLMOPT_SOCKETFUNCTION,
        Some(ref_cache_upstream_c_753_sock_func),
    );
    if mc == CURLM_OK {
        mc = curl_multi_setopt_ptr(multi, CURLMOPT_SOCKETDATA, mdata.cast());
    }
    if mc == CURLM_OK {
        mc = curl_multi_setopt_timer_callback(
            multi,
            CURLMOPT_TIMERFUNCTION,
            Some(ref_cache_upstream_c_801_timer_func),
        );
    }
    if mc == CURLM_OK {
        mc = curl_multi_setopt_ptr(multi, CURLMOPT_TIMERDATA, mdata.cast());
    }
    if mc != CURLM_OK {
        eprintln!(
            "Failed to set options for curl multi handle : {}",
            std::ffi::CStr::from_ptr(curl_multi_strerror(mc)).to_string_lossy()
        );
        curl_multi_cleanup(multi);
        return std::ptr::null_mut();
    }
    multi
}

// original: init_multi_data (htslib/ref_cache/upstream.c:838)
unsafe fn ref_cache_upstream_c_838_init_multi_data(
    multi: *mut CURLM,
    mdata: *mut Multi_data,
) -> c_int {
    (*mdata).multi = multi;
    (*mdata).pw = poll_impl::ref_cache_poll_wrap_epoll_c_49_pw_init(false);
    if (&(*mdata).pw).is_none() {
        eprintln!("Initalizing poller: {}", std::io::Error::last_os_error());
        return -1;
    }
    (*mdata).downloads.clear();
    (*mdata).waiting = None;
    (*mdata).last_waiting = None;
    (*mdata).timeout = -1;
    if (&mut (*mdata).downloads)
        .try_reserve_exact(ACTIVE_SIZE)
        .is_err()
    {
        eprintln!("{}", std::io::Error::last_os_error());
        return -1;
    }
    (&mut (*mdata).downloads).resize(ACTIVE_SIZE, None);
    (*mdata).ncurls = 4;
    (*mdata).free_curls = 0;
    (*mdata).running = 0;
    let ncurls = (*mdata).ncurls as usize;
    if (&mut (*mdata).curls)
        .try_reserve_exact(ncurls)
        .is_err()
    {
        eprintln!("{}", std::io::Error::last_os_error());
        return -1;
    }
    (&mut (*mdata).curls).resize(ncurls, std::ptr::null_mut());

    for i in 0..(*mdata).ncurls {
        (&mut (*mdata).curls)[i as usize] = curl_easy_init();
        if (&(*mdata).curls)[i as usize].is_null() {
            for j in 0..(*mdata).ncurls {
                if !(&(*mdata).curls)[j as usize].is_null() {
                    curl_easy_cleanup((&(*mdata).curls)[j as usize]);
                }
            }
            return -1;
        }
        let mut cc = curl_easy_setopt_write_callback(
            (&(*mdata).curls)[i as usize],
            CURLOPT_WRITEFUNCTION,
            Some(ref_cache_upstream_c_675_multi_receive_data),
        );
        if cc != CURLE_OK {
            eprintln!(
                "Couldn't set WRITEFUNCTION on curl handle : {}",
                std::ffi::CStr::from_ptr(curl_easy_strerror(cc)).to_string_lossy()
            );
            for j in 0..(*mdata).ncurls {
                if !(&(*mdata).curls)[j as usize].is_null() {
                    curl_easy_cleanup((&(*mdata).curls)[j as usize]);
                }
            }
            return -1;
        }
        cc = curl_easy_setopt_long((&(*mdata).curls)[i as usize], CURLOPT_NOPROGRESS, 0);
        if cc == CURLE_OK {
            cc = curl_easy_setopt_xfer_callback(
                (&(*mdata).curls)[i as usize],
                CURLOPT_XFERINFOFUNCTION,
                Some(ref_cache_upstream_c_656_progress_callback),
            );
        }
        if cc != CURLE_OK {
            eprintln!(
                "Couldn't set progress callback in CURL handle : {}",
                std::ffi::CStr::from_ptr(curl_easy_strerror(cc)).to_string_lossy()
            );
            for j in 0..(*mdata).ncurls {
                if !(&(*mdata).curls)[j as usize].is_null() {
                    curl_easy_cleanup((&(*mdata).curls)[j as usize]);
                }
            }
            return -1;
        }
        (*mdata).free_curls |= 1u32 << i;
    }
    0
}

// original: rename_download_file (htslib/ref_cache/upstream.c:897)
unsafe fn ref_cache_upstream_c_897_rename_download_file(download: usize) -> c_int {
    if (dl!(download).flags & DL_OK) != 0 {
        // Final "aa/bb/cccc..." path (md5 split 0..2 / 2..4 / 4..32); no NUL.
        let mut dest = Vec::with_capacity(MD5_LEN + 2);
        dest.extend_from_slice(&dl!(download).hexmd5[0..2]);
        dest.push(b'/');
        dest.extend_from_slice(&dl!(download).hexmd5[2..4]);
        dest.push(b'/');
        dest.extend_from_slice(&dl!(download).hexmd5[4..MD5_LEN]);

        // NUL-terminate the two paths only at the renameat(2) boundary.
        let mut src_c = dl!(download).file.clone();
        src_c.push(0);
        let mut dest_c = dest.clone();
        dest_c.push(0);
        let cache_fd = dl!(download).cache_fd;
        if libc::renameat(
            cache_fd,
            src_c.as_ptr().cast(),
            cache_fd,
            dest_c.as_ptr().cast(),
        ) != 0
        {
            eprintln!(
                "Couldn't rename {}/{} to {}/{}: {}",
                String::from_utf8_lossy(&dl!(download).cache_dir),
                String::from_utf8_lossy(&dl!(download).file),
                String::from_utf8_lossy(&dl!(download).cache_dir),
                String::from_utf8_lossy(&dest),
                std::io::Error::last_os_error()
            );
            dl!(download).flags &= !DL_OK;
            return -1;
        }
        dl!(download).file = dest;
    }
    0
}

// original: finish_download (htslib/ref_cache/upstream.c:916)
unsafe fn ref_cache_upstream_c_916_finish_download(
    download: usize,
    multi: *mut CURLM,
    msg: *mut CURLMsg,
) -> c_int {
    let mut rcode: c_long = 500;
    let cc = (*msg).data.result;
    let mut md5 = [0u8; 16];
    let curl = dl!(download).curl;

    if cc != CURLE_OK {
        eprintln!(
            "Download of {} failed: {}",
            String::from_utf8_lossy(&dl!(download).hexmd5),
            std::ffi::CStr::from_ptr(curl_easy_strerror(cc)).to_string_lossy()
        );
        if ref_cache_upstream_c_646_send_result_code_all(download, rcode as c_int) != 0 {
            return -1;
        }
        let mc = curl_multi_remove_handle(multi, curl);
        if mc != CURLM_OK {
            eprintln!(
                "Couldn't remove easy handle from curl_multi : {}",
                std::ffi::CStr::from_ptr(curl_multi_strerror(mc)).to_string_lossy()
            );
            return -1;
        }
        ref_cache_upstream_c_331_free_download(download);
        return 0;
    }

    if curl_easy_getinfo_long((*msg).easy_handle, CURLINFO_RESPONSE_CODE, &mut rcode) != CURLE_OK {
        eprintln!(
            "Couldn't get response code : {}",
            std::ffi::CStr::from_ptr(curl_easy_strerror(cc)).to_string_lossy()
        );
        return -1;
    }
    if rcode != 200 {
        if ref_cache_upstream_c_646_send_result_code_all(download, rcode as c_int) != 0 {
            return -1;
        }
        let mc = curl_multi_remove_handle(multi, curl);
        if mc != CURLM_OK {
            eprintln!(
                "Couldn't remove easy handle from curl_multi : {}",
                std::ffi::CStr::from_ptr(curl_multi_strerror(mc)).to_string_lossy()
            );
            return -1;
        }
        ref_cache_upstream_c_331_free_download(download);
        return 0;
    }

    if dl!(download).size == 0 {
        dl!(download).size = dl!(download).received;
    }
    if dl!(download).received != dl!(download).size {
        eprintln!(
            "Downloading {} : Content-Length was a lie. Expected {}, got {}",
            String::from_utf8_lossy(&dl!(download).hexmd5),
            dl!(download).size as c_long,
            dl!(download).received as c_long
        );
        if ref_cache_upstream_c_646_send_result_code_all(download, 502) != 0 {
            return -1;
        }
        let mc = curl_multi_remove_handle(multi, curl);
        if mc != CURLM_OK {
            eprintln!(
                "Couldn't remove easy handle from curl_multi : {}",
                std::ffi::CStr::from_ptr(curl_multi_strerror(mc)).to_string_lossy()
            );
            return -1;
        }
        ref_cache_upstream_c_331_free_download(download);
        return 0;
    }

    // Finalise the running MD5 through the owned context API.
    if let Some(ctx) = dl!(download).md5_ctx.as_mut() {
        hts_md5_final(&mut md5, ctx);
    }
    for (i, md5_byte) in md5.iter().enumerate() {
        let byte = (ref_cache_misc_h_38_hexval(dl!(download).hexmd5[i * 2]) << 4)
            | ref_cache_misc_h_38_hexval(dl!(download).hexmd5[i * 2 + 1]);
        if byte != *md5_byte as c_int {
            eprintln!(
                "Downloading {} : MD5 checksum didn't match.",
                String::from_utf8_lossy(&dl!(download).hexmd5)
            );
            if ref_cache_upstream_c_646_send_result_code_all(download, 502) != 0 {
                return -1;
            }
            let mc = curl_multi_remove_handle(multi, curl);
            if mc != CURLM_OK {
                eprintln!(
                    "Couldn't remove easy handle from curl_multi : {}",
                    std::ffi::CStr::from_ptr(curl_multi_strerror(mc)).to_string_lossy()
                );
                return -1;
            }
            ref_cache_upstream_c_331_free_download(download);
            return 0;
        }
    }

    dl!(download).flags |= DL_OK;
    if ref_cache_upstream_c_897_rename_download_file(download) != 0 {
        return -1;
    }

    let mc = curl_multi_remove_handle(multi, curl);
    if mc != CURLM_OK {
        eprintln!(
            "Couldn't remove easy handle from curl_multi : {}",
            std::ffi::CStr::from_ptr(curl_multi_strerror(mc)).to_string_lossy()
        );
        return -1;
    }

    if ref_cache_upstream_c_646_send_result_code_all(download, 200) != 0 {
        return -1;
    }
    ref_cache_upstream_c_331_free_download(download);
    0
}

// original: handle_curl_socket (htslib/ref_cache/upstream.c:996)
unsafe fn ref_cache_upstream_c_996_handle_curl_socket(
    multi: *mut CURLM,
    events: c_int,
    fd: c_int,
) -> c_int {
    let mut running = 0;
    let mut mc;
    loop {
        mc = curl_multi_socket_action(multi, fd, events, &mut running);
        if mc != CURLM_CALL_MULTI_PERFORM {
            break;
        }
    }

    if mc != CURLM_OK {
        eprintln!(
            "Error from curl socket #{}: {}",
            fd,
            std::ffi::CStr::from_ptr(curl_multi_strerror(mc)).to_string_lossy()
        );
        return -1;
    }
    if running < mdata!().running {
        let mut msgs_in_queue = 0;
        let mut nfinished = 0;
        mdata!().running = running;
        loop {
            let msg = curl_multi_info_read(multi, &mut msgs_in_queue);
            if msg.is_null() {
                break;
            }
            if (*msg).msg == CURLMSG_DONE {
                // CURLINFO_PRIVATE returns the download arena index we stored as
                // a void* in start_new_download; recover it as `usize`.
                let mut download: *mut c_void = std::ptr::null_mut();
                let c = curl_easy_getinfo_ptr((*msg).easy_handle, CURLINFO_PRIVATE, &mut download);
                if c != CURLE_OK {
                    eprintln!(
                        "curl_easy_getinfo failed: {}",
                        std::ffi::CStr::from_ptr(curl_easy_strerror(c)).to_string_lossy()
                    );
                    return -1;
                }
                if ref_cache_upstream_c_916_finish_download(download as usize, multi, msg) != 0 {
                    return -1;
                }
                nfinished += 1;
            }
        }
        if nfinished != 0 {
            loop {
                let started = ref_cache_upstream_c_385_start_new_download(multi);
                if started <= 0 {
                    if started < 0 {
                        return -1;
                    }
                    break;
                }
            }
        }
    } else {
        mdata!().running = running;
        if mdata!().running == 0 {
            mdata!().timeout = -1;
        }
    }
    0
}

// original: run_epoll_loop (htslib/ref_cache/upstream.c:1047)
unsafe fn ref_cache_upstream_c_1047_run_epoll_loop(
    opts: &Options,
    nfds: c_uint,
    cmd_fds: &[c_int],
    liveness_fd: c_int,
    multi: *mut CURLM,
) {
    let mut events: [libc::epoll_event; MAX_EVENTS as usize] = std::mem::zeroed();
    // Poller arena indices of the command / liveness registrations (was a
    // Vec<*mut Pw_item>).
    let mut cmd_pollers = Vec::<Option<usize>>::new();
    if cmd_pollers.try_reserve_exact(nfds as usize + 1).is_err() {
        return;
    }
    cmd_pollers.resize(nfds as usize + 1, None);
    let mut npolled = 0u32;
    let mut running = 1;

    while npolled < nfds {
        let pw = mdata!().pw.as_deref_mut().expect("poller initialised");
        cmd_pollers[npolled as usize] = poll_impl::ref_cache_poll_wrap_epoll_c_78_pw_register(
            pw,
            cmd_fds[npolled as usize],
            Pw_fd_type::US_COMMAND,
            PW_IN as u32,
            0,
        );
        if cmd_pollers[npolled as usize].is_none() {
            break;
        }
        npolled += 1;
    }
    if npolled == nfds {
        let pw = mdata!().pw.as_deref_mut().expect("poller initialised");
        cmd_pollers[nfds as usize] = poll_impl::ref_cache_poll_wrap_epoll_c_78_pw_register(
            pw,
            liveness_fd,
            Pw_fd_type::US_LIVE,
            (PW_HUP | PW_ERR) as u32,
            0,
        );
        if cmd_pollers[nfds as usize].is_none() {
            npolled = nfds;
        } else {
            npolled = nfds + 1;
        }
    }

    if npolled == nfds + 1 {
        while running != 0 || mdata!().running > 0 {
            let timeout = if mdata!().timeout < c_int::MAX as c_long {
                mdata!().timeout as c_int
            } else {
                c_int::MAX
            };
            let pw = mdata!().pw.as_deref_mut().expect("poller initialised");
            let nevents =
                poll_impl::ref_cache_poll_wrap_epoll_c_120_pw_wait(pw, &mut events, timeout);
            if nevents == -1 {
                if *__errno_location() == libc::EINTR {
                    continue;
                }
                eprintln!("poll_wait: {}", std::io::Error::last_os_error());
                break;
            }
            if nevents == 0
                && ref_cache_upstream_c_996_handle_curl_socket(multi, 0, CURL_SOCKET_TIMEOUT) != 0
            {
                break;
            }

            for n in 0..nevents {
                let evts = events[n as usize].events;
                // epoll's `u64` carries the poller arena index.
                let item_idx = events[n as usize].u64 as usize;
                let pw = mdata!().pw.as_deref().expect("poller initialised");
                let Some(item) = pw.item_at(item_idx) else {
                    continue;
                };
                let fd_type = item.fd_type;
                let item_fd = item.fd;
                match fd_type {
                    Pw_fd_type::US_COMMAND => {
                        if opts.verbosity > 2 {
                            eprintln!("Upstream received command");
                        }
                        if ref_cache_upstream_c_563_get_cmd_multi(item_fd, opts, multi, running) <= 0
                        {
                            running = 0;
                            break;
                        }
                    }
                    Pw_fd_type::US_CURL => {
                        let e = (if (evts & PW_IN as u32) != 0 {
                            CURL_CSELECT_IN
                        } else {
                            0
                        }) | (if (evts & PW_OUT as u32) != 0 {
                            CURL_CSELECT_OUT
                        } else {
                            0
                        }) | (if (evts & PW_ERR as u32) != 0 {
                            CURL_CSELECT_ERR
                        } else {
                            0
                        });
                        if opts.verbosity > 2 {
                            eprintln!("Upstream received curl event {} on fd #{}", e, item_fd);
                        }
                        if ref_cache_upstream_c_996_handle_curl_socket(multi, e, item_fd) != 0 {
                            running = 0;
                            break;
                        }
                    }
                    Pw_fd_type::US_LIVE => {
                        if (evts & (PW_HUP | PW_ERR) as u32) != 0 {
                            running = 0;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    for i in 0..npolled {
        if let Some(idx) = cmd_pollers[i as usize] {
            let pw = mdata!().pw.as_deref_mut().expect("poller initialised");
            poll_impl::ref_cache_poll_wrap_epoll_c_126_pw_remove(pw, idx, false);
        }
    }
}

// original: run_multi_upstream_handler (htslib/ref_cache/upstream.c:1131)
unsafe fn ref_cache_upstream_c_1131_run_multi_upstream_handler(
    opts: &Options,
    cmd_fds: &[c_int],
    liveness_fd: c_int,
) -> c_int {
    let res = -1;
    let mut mdata = Box::new(Multi_data {
        multi: std::ptr::null_mut(),
        pw: None,
        timeout: -1,
        downloads: Vec::new(),
        downloads_arena: Vec::new(),
        download_free: Vec::new(),
        downstream_arena: Vec::new(),
        downstream_free: Vec::new(),
        waiting: None,
        last_waiting: None,
        ncurls: 0,
        free_curls: 0,
        running: 0,
        curls: Vec::new(),
    });
    let multi = ref_cache_upstream_c_808_get_multi(&mut *mdata);
    if multi.is_null() {
        return -1;
    }

    if ref_cache_upstream_c_838_init_multi_data(multi, &mut *mdata) != 0 {
        curl_multi_cleanup(multi);
        return -1;
    }

    // Publish the owning Multi_data for the libcurl callbacks, which can only
    // carry the integer download index across the C boundary and recover the
    // arena through `MDATA`. Cleared again before `mdata` drops.
    MDATA = &mut *mdata;

    ref_cache_upstream_c_1047_run_epoll_loop(
        opts,
        opts.max_kids as c_uint,
        cmd_fds,
        liveness_fd,
        multi,
    );

    for i in 0..mdata.ncurls {
        curl_multi_remove_handle(multi, mdata.curls[i as usize]);
        curl_easy_cleanup(mdata.curls[i as usize]);
    }
    curl_multi_cleanup(multi);
    MDATA = std::ptr::null_mut();
    res
}

// original: run_upstream_handler (htslib/ref_cache/upstream.c:1157)
pub unsafe fn ref_cache_upstream_c_1157_run_upstream_handler(
    opts: &Options,
    sockets: &[c_int],
    liveness_fd: c_int,
) -> c_int {
    let mut sa: libc::sigaction = std::mem::zeroed();
    let mut old_sa: libc::sigaction = std::mem::zeroed();

    sa.sa_sigaction = libc::SIG_IGN;
    libc::sigemptyset(&mut sa.sa_mask);
    sa.sa_flags = 0;

    if libc::sigaction(libc::SIGPIPE, &sa, &mut old_sa) != 0 {
        eprintln!("sigaction(SIGPIPE): {}", std::io::Error::last_os_error());
        return -1;
    }

    if curl_global_init(CURL_GLOBAL_ALL) != 0 {
        eprintln!("Couldn't initialize libcurl");
        return -1;
    }

    let cvi = curl_version_info(CURLVERSION_NOW);
    if cvi.is_null() {
        eprintln!("Couldn't get curl version information");
        return -1;
    }
    CURL_VERSION_NUM = (*cvi).version_num;

    let res = ref_cache_upstream_c_1131_run_multi_upstream_handler(opts, sockets, liveness_fd);

    curl_global_cleanup();

    if libc::sigaction(libc::SIGPIPE, &old_sa, std::ptr::null_mut()) != 0 {
        eprintln!("sigaction(SIGPIPE): {}", std::io::Error::last_os_error());
    }
    res
}
