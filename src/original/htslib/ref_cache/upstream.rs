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
use super::poll_wrap::{Pw_fd_type, Pw_item, PW_ERR, PW_HUP, PW_IN, PW_OUT};
use super::poll_wrap_epoll as poll_impl;
use crate::htslib_mini_rs::c_compat::__errno_location;
use crate::htslib_mini_rs::md5::{
    hts_md5_context, hts_md5_destroy, hts_md5_final, hts_md5_init, hts_md5_update,
};
use std::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_void};

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
type CURLINFO = c_int;
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

const CURLINFO_RESPONSE_CODE: CURLINFO = 0x200000 + 2;
const CURLINFO_CONTENT_LENGTH_DOWNLOAD_T: CURLINFO = 0x600000 + 15;
const CURLINFO_PRIVATE: CURLINFO = 0x100000 + 21;

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
    fn curl_easy_getinfo_long(curl: *mut CURL, info: CURLINFO, value: *mut c_long) -> CURLcode;
    #[link_name = "curl_easy_getinfo"]
    fn curl_easy_getinfo_off_t(curl: *mut CURL, info: CURLINFO, value: *mut curl_off_t)
        -> CURLcode;
    #[link_name = "curl_easy_getinfo"]
    fn curl_easy_getinfo_ptr(curl: *mut CURL, info: CURLINFO, value: *mut *mut c_void) -> CURLcode;
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
#[repr(C)]
pub struct Download {
    hexmd5: [c_char; MD5_LEN],
    md5_ctx: *mut hts_md5_context,
    mdata: *mut Multi_data,
    next: *mut Download,
    waiting: *mut Download,
    downstream: *mut Downstream,
    cache_dir: *const c_char,
    file: *mut c_char,
    url: *mut c_char,
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
#[repr(C)]
pub struct Downstream {
    download: *mut Download,
    prev: *mut Downstream,
    next: *mut Downstream,
    cmd_fd: c_int,
    id: c_uint,
}

// original: Multi_data (htslib/ref_cache/upstream.c:106)
#[repr(C)]
pub struct Multi_data {
    multi: *mut CURLM,
    pw: *mut poll_impl::Poll_wrap,
    timeout: c_long,
    downloads: *mut *mut Download,
    waiting: *mut Download,
    last_waiting: *mut Download,
    ncurls: c_uint,
    free_curls: c_uint,
    running: c_int,
    curls: *mut *mut CURL,
}

// original: upstream_send_cmd (htslib/ref_cache/upstream.c:122)
pub unsafe fn ref_cache_upstream_c_122_upstream_send_cmd(
    cmd_fd: c_int,
    hexmd5: *const c_char,
    mut id: c_uint,
) -> c_int {
    let mut msg: libc::msghdr = std::mem::zeroed();
    let mut iov = [
        libc::iovec {
            iov_base: hexmd5.cast_mut().cast(),
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
    hexmd5: *mut c_char,
    id: *mut c_uint,
) -> libc::ssize_t {
    let mut msg: libc::msghdr = std::mem::zeroed();
    let mut iov = [
        libc::iovec {
            iov_base: hexmd5.cast(),
            iov_len: MD5_LEN,
        },
        libc::iovec {
            iov_base: id.cast(),
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
    umsg: *mut Upstream_msg,
    fd: c_int,
) -> c_int {
    let mut msg: libc::msghdr = std::mem::zeroed();
    let mut iov = [libc::iovec {
        iov_base: umsg.cast(),
        iov_len: std::mem::size_of::<Upstream_msg>(),
    }];
    let mut buf = [0 as c_char; 256];

    msg.msg_name = std::ptr::null_mut();
    msg.msg_control = std::ptr::null_mut();
    msg.msg_iov = iov.as_mut_ptr();
    msg.msg_iovlen = 1;

    libc::memset(buf.as_mut_ptr().cast(), 0, buf.len());

    if (*umsg).code == Upstream_msg_code::US_START
        && ref_cache_cmsg_wrap_c_46_make_scm_rights_cmsg(&mut msg, fd, buf.as_mut_ptr(), buf.len())
            < 0
    {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"upstream_send_msg: cmsg buffer not big enough.\n".as_ptr(),
        );
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
    download: *mut Download,
    code: Upstream_msg_code,
    val: i64,
) -> c_int {
    let mut d = (*download).downstream;
    let mut res = 0;
    while !d.is_null() {
        let mut msg = Upstream_msg {
            id: (*d).id,
            code,
            val,
        };
        if ref_cache_upstream_c_172_upstream_send_msg((*d).cmd_fd, &mut msg, -1) < 0 {
            res = -1;
        }
        d = (*d).next;
    }
    res
}

// original: upstream_recv_msg (htslib/ref_cache/upstream.c:215)
pub unsafe fn ref_cache_upstream_c_215_upstream_recv_msg(
    cmd_fd: c_int,
    umsg: *mut Upstream_msg,
    fd: *mut c_int,
) -> c_int {
    let mut msg: libc::msghdr = std::mem::zeroed();
    let mut iov = [libc::iovec {
        iov_base: umsg.cast(),
        iov_len: std::mem::size_of::<Upstream_msg>(),
    }];
    let mut buf = [0 as c_char; 16384];

    msg.msg_name = std::ptr::null_mut();
    msg.msg_control = std::ptr::null_mut();
    msg.msg_iov = iov.as_mut_ptr();
    msg.msg_iovlen = 1;
    msg.msg_control = buf.as_mut_ptr().cast();
    msg.msg_controllen = buf.len();
    libc::memset(umsg.cast(), 0, std::mem::size_of::<Upstream_msg>());

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

    if (*umsg).code == Upstream_msg_code::US_START {
        *fd = ref_cache_cmsg_wrap_c_67_get_scm_rights_fd(&mut msg);
        if *fd < 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Failed to get file descriptor in upstream message\n".as_ptr(),
            );
            return -1;
        }
    }

    1
}

// original: make_subdir (htslib/ref_cache/upstream.c:252)
unsafe fn ref_cache_upstream_c_252_make_subdir(opts: *mut Options, hexmd5: *mut c_char) -> c_int {
    let mut path = [0 as c_char; 6];

    libc::memcpy(path.as_mut_ptr().cast(), hexmd5.cast(), 2);
    path[2] = 0;
    if libc::mkdirat((*opts).cache_fd, path.as_ptr(), 0o1755) != 0
        && *__errno_location() != libc::EEXIST
    {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't make directory %s/%s : %s\n".as_ptr(),
            (*opts).cache_dir,
            path.as_ptr(),
            libc::strerror(*__errno_location()),
        );
        return -1;
    }

    path[2] = b'/' as c_char;
    libc::memcpy(path.as_mut_ptr().add(3).cast(), hexmd5.add(2).cast(), 2);
    path[5] = 0;
    if libc::mkdirat((*opts).cache_fd, path.as_ptr(), 0o1755) != 0
        && *__errno_location() != libc::EEXIST
    {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't make directory %s/%s : %s\n".as_ptr(),
            (*opts).cache_dir,
            path.as_ptr(),
            libc::strerror(*__errno_location()),
        );
        return -1;
    }
    0
}

// original: get_free_curl (htslib/ref_cache/upstream.c:277)
unsafe fn ref_cache_upstream_c_277_get_free_curl(mdata: *mut Multi_data) -> c_int {
    if (*mdata).free_curls == 0 {
        return -1;
    }
    for i in 0..(*mdata).ncurls {
        if ((*mdata).free_curls & (1u32 << i)) != 0 {
            (*mdata).free_curls &= !(1u32 << i);
            return i as c_int;
        }
    }
    -1
}

// original: release_curl (htslib/ref_cache/upstream.c:290)
unsafe fn ref_cache_upstream_c_290_release_curl(download: *mut Download) {
    (*(*download).mdata).free_curls |= 1u32 << (*download).curlid;
    (*download).curlid = -1;
    (*download).curl = std::ptr::null_mut();
}

// original: new_downstream (htslib/ref_cache/upstream.c:296)
unsafe fn ref_cache_upstream_c_296_new_downstream(
    cmd_fd: c_int,
    downstream_id: c_uint,
) -> *mut Downstream {
    let downstream = libc::calloc(1, std::mem::size_of::<Downstream>()).cast::<Downstream>();
    if downstream.is_null() {
        libc::perror(c"new_downstream".as_ptr());
        return std::ptr::null_mut();
    }
    (*downstream).cmd_fd = cmd_fd;
    (*downstream).id = downstream_id;
    downstream
}

// original: new_download (htslib/ref_cache/upstream.c:306)
pub unsafe fn ref_cache_upstream_c_306_new_download(
    opts: *mut Options,
    mdata: *mut Multi_data,
    hexmd5: *mut c_char,
) -> *mut Download {
    let download = libc::calloc(1, std::mem::size_of::<Download>()).cast::<Download>();
    if download.is_null() {
        libc::perror(c"new_download".as_ptr());
        return std::ptr::null_mut();
    }
    (*download).mdata = mdata;
    (*download).cache_dir = (*opts).cache_dir;
    libc::memcpy(
        (*download).hexmd5.as_mut_ptr().cast(),
        hexmd5.cast(),
        MD5_LEN,
    );
    (*download).cache_fd = (*opts).cache_fd;
    (*download).file_fd = -1;
    (*download).curlid = -1;
    download
}

// original: remove_downstream (htslib/ref_cache/upstream.c:319)
unsafe fn ref_cache_upstream_c_319_remove_downstream(
    downstream: *mut Downstream,
    download: *mut Download,
) {
    if (*downstream).prev.is_null() {
        (*download).downstream = (*downstream).next;
    } else {
        assert!(downstream != (*download).downstream);
        (*(*downstream).prev).next = (*downstream).next;
    }
    if !(*downstream).next.is_null() {
        (*(*downstream).next).prev = (*downstream).prev;
    }
    libc::free(downstream.cast());
}

// original: free_download (htslib/ref_cache/upstream.c:331)
unsafe fn ref_cache_upstream_c_331_free_download(download: *mut Download) {
    let mdata = (*download).mdata;
    while !(*download).downstream.is_null() {
        ref_cache_upstream_c_319_remove_downstream((*download).downstream, download);
    }

    let hash = ((ref_cache_misc_h_38_hexval((*download).hexmd5[0]) << 12)
        | (ref_cache_misc_h_38_hexval((*download).hexmd5[1]) << 8)
        | (ref_cache_misc_h_38_hexval((*download).hexmd5[2]) << 4)
        | ref_cache_misc_h_38_hexval((*download).hexmd5[3]))
        & ACTIVE_MASK;
    let slot = (*mdata).downloads.add(hash as usize);
    if *slot == download {
        *slot = (*download).next;
    } else {
        let mut d = *slot;
        while !d.is_null() && (*d).next != download {
            d = (*d).next;
        }
        if !d.is_null() {
            (*d).next = (*download).next;
        }
    }

    if ((*download).flags & DL_WAITING) != 0 {
        if (*mdata).waiting == download {
            if (*mdata).last_waiting == download {
                (*mdata).waiting = std::ptr::null_mut();
                (*mdata).last_waiting = std::ptr::null_mut();
            } else {
                (*mdata).waiting = (*download).waiting;
            }
        } else {
            let mut d = (*mdata).waiting;
            while !d.is_null() && (*d).waiting != download {
                d = (*d).waiting;
            }
            if !d.is_null() {
                if (*mdata).last_waiting == download {
                    (*mdata).last_waiting = d;
                }
                (*d).waiting = (*download).waiting;
            }
        }
    }

    if (*download).file_fd != -1 {
        libc::close((*download).file_fd);
        if ((*download).flags & DL_OK) == 0 {
            libc::unlinkat((*download).cache_fd, (*download).file, 0);
        }
    }
    libc::free((*download).file.cast());
    libc::free((*download).url.cast());
    if (*download).curlid != -1 {
        ref_cache_upstream_c_290_release_curl(download);
    }
    if !(*download).md5_ctx.is_null() {
        hts_md5_destroy((*download).md5_ctx);
    }
    libc::free(download.cast());
}

// original: start_new_download (htslib/ref_cache/upstream.c:385)
unsafe fn ref_cache_upstream_c_385_start_new_download(
    multi: *mut CURLM,
    mdata: *mut Multi_data,
) -> c_int {
    let download = (*mdata).waiting;
    if download.is_null() {
        return 0;
    }
    assert!(((*download).flags & DL_WAITING) != 0);

    (*download).curlid = ref_cache_upstream_c_277_get_free_curl(mdata);
    if (*download).curlid == -1 {
        return 0;
    }

    (*download).curl = *(*mdata).curls.add((*download).curlid as usize);
    (*mdata).waiting = (*download).waiting;
    if (*mdata).last_waiting == download {
        (*mdata).last_waiting = std::ptr::null_mut();
    }
    (*download).flags &= !DL_WAITING;

    let mut cc = curl_easy_setopt_cstr((*download).curl, CURLOPT_URL, (*download).url);
    if cc != CURLE_OK {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't set URL %s : %s\n".as_ptr(),
            (*download).url,
            curl_easy_strerror(cc),
        );
        ref_cache_upstream_c_331_free_download(download);
        return -1;
    }
    cc = curl_easy_setopt_ptr((*download).curl, CURLOPT_WRITEDATA, download.cast());
    if cc != CURLE_OK {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't set user data in CURL handle : %s\n".as_ptr(),
            curl_easy_strerror(cc),
        );
        ref_cache_upstream_c_331_free_download(download);
        return -1;
    }
    cc = curl_easy_setopt_ptr((*download).curl, CURLOPT_PRIVATE, download.cast());
    if cc != CURLE_OK {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't set private data in CURL handle : %s\n".as_ptr(),
            curl_easy_strerror(cc),
        );
        ref_cache_upstream_c_331_free_download(download);
        return -1;
    }
    cc = curl_easy_setopt_ptr((*download).curl, CURLOPT_XFERINFODATA, download.cast());
    if cc != CURLE_OK {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't set progress data in CURL handle : %s\n".as_ptr(),
            curl_easy_strerror(cc),
        );
        ref_cache_upstream_c_331_free_download(download);
        return -1;
    }

    let mc = curl_multi_add_handle(multi, (*download).curl);
    if mc != CURLM_OK {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't add handle to curl_multi : %s\n".as_ptr(),
            curl_multi_strerror(mc),
        );
        ref_cache_upstream_c_331_free_download(download);
        return -1;
    }
    (*mdata).running += 1;
    1
}

// original: get_download (htslib/ref_cache/upstream.c:445)
unsafe fn ref_cache_upstream_c_445_get_download(
    opts: *mut Options,
    multi: *mut CURLM,
    mdata: *mut Multi_data,
    hexmd5: *mut c_char,
) -> *mut Download {
    let hash = ((ref_cache_misc_h_38_hexval(*hexmd5.add(0)) << 12)
        | (ref_cache_misc_h_38_hexval(*hexmd5.add(1)) << 8)
        | (ref_cache_misc_h_38_hexval(*hexmd5.add(2)) << 4)
        | ref_cache_misc_h_38_hexval(*hexmd5.add(3)))
        & ACTIVE_MASK;

    let mut download = *(*mdata).downloads.add(hash as usize);
    while !download.is_null() {
        if libc::memcmp(hexmd5.cast(), (*download).hexmd5.as_ptr().cast(), MD5_LEN) == 0 {
            return download;
        }
        download = (*download).next;
    }

    download = ref_cache_upstream_c_306_new_download(opts, mdata, hexmd5);
    if download.is_null() {
        return std::ptr::null_mut();
    }

    (*download).file = libc::malloc(MD5_LEN + 16).cast();
    if (*download).file.is_null() {
        libc::perror(c"Allocating download->file".as_ptr());
        libc::free(download.cast());
        return std::ptr::null_mut();
    }

    libc::snprintf(
        (*download).file,
        MD5_LEN + 16,
        c"%.2s/%.2s/%.28s".as_ptr(),
        (*download).hexmd5.as_ptr(),
        (*download).hexmd5.as_ptr().add(2),
        (*download).hexmd5.as_ptr().add(4),
    );

    (*download).file_fd = libc::openat((*opts).cache_fd, (*download).file, libc::O_RDONLY);
    if (*download).file_fd >= 0 {
        let mut st: libc::stat = std::mem::zeroed();
        if libc::fstat((*download).file_fd, &mut st) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't stat %s/%s : %s\n".as_ptr(),
                (*opts).cache_dir,
                (*download).file,
                libc::strerror(*__errno_location()),
            );
            ref_cache_upstream_c_331_free_download(download);
            return std::ptr::null_mut();
        }
        (*download).size = st.st_size;
        (*download).received = st.st_size;
        (*download).flags = DL_OK | DL_CLENGTH;
        (*download).next = *(*mdata).downloads.add(hash as usize);
        *(*mdata).downloads.add(hash as usize) = download;
        return download;
    }

    if *__errno_location() != libc::ENOENT {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't open %s/%s : %s\n".as_ptr(),
            (*opts).cache_dir,
            (*download).file,
            libc::strerror(*__errno_location()),
        );
        ref_cache_upstream_c_331_free_download(download);
        return std::ptr::null_mut();
    }

    let url_len = (*opts).upstream_url_len + MD5_LEN + 2;
    let need_sep = ((*opts).upstream_url_len == 0
        || *(*opts).upstream_url.add((*opts).upstream_url_len - 1) != b'/' as c_char)
        as c_int;
    (*download).url = libc::malloc(url_len).cast();
    if (*download).url.is_null() {
        libc::perror(c"Allocating download->url".as_ptr());
        ref_cache_upstream_c_331_free_download(download);
        return std::ptr::null_mut();
    }
    libc::snprintf(
        (*download).url,
        url_len,
        c"%s%s%.32s".as_ptr(),
        (*opts).upstream_url,
        if need_sep != 0 {
            c"/".as_ptr()
        } else {
            c"".as_ptr()
        },
        hexmd5,
    );

    if ref_cache_upstream_c_252_make_subdir(opts, hexmd5) != 0 {
        ref_cache_upstream_c_331_free_download(download);
        return std::ptr::null_mut();
    }

    for count in 0..1000 {
        libc::snprintf(
            (*download).file,
            MD5_LEN + 16,
            c"%.2s/%.2s/%.28s.%03d".as_ptr(),
            hexmd5,
            hexmd5.add(2),
            hexmd5.add(4),
            count,
        );
        loop {
            (*download).file_fd = libc::openat(
                (*opts).cache_fd,
                (*download).file,
                libc::O_RDWR | libc::O_CREAT | libc::O_EXCL,
                0o644,
            );
            if !((*download).file_fd == -1 && *__errno_location() == libc::EINTR) {
                break;
            }
        }
        if (*download).file_fd >= 0 {
            break;
        }
        if *__errno_location() != libc::EEXIST {
            break;
        }
    }
    if (*download).file_fd == -1 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't open %s/%s for writing: %s\n".as_ptr(),
            (*opts).cache_dir,
            (*download).file,
            libc::strerror(*__errno_location()),
        );
        ref_cache_upstream_c_331_free_download(download);
        return std::ptr::null_mut();
    }

    (*download).md5_ctx = hts_md5_init();
    if (*download).md5_ctx.is_null() {
        ref_cache_upstream_c_331_free_download(download);
        return std::ptr::null_mut();
    }

    (*download).waiting = std::ptr::null_mut();
    if !(*mdata).last_waiting.is_null() {
        (*(*mdata).last_waiting).waiting = download;
    }
    if (*mdata).waiting.is_null() {
        (*mdata).waiting = download;
    }
    (*mdata).last_waiting = download;
    (*download).flags |= DL_WAITING;

    if ref_cache_upstream_c_385_start_new_download(multi, mdata) < 0 {
        return std::ptr::null_mut();
    }

    (*download).next = *(*mdata).downloads.add(hash as usize);
    *(*mdata).downloads.add(hash as usize) = download;
    download
}

// original: get_cmd_multi (htslib/ref_cache/upstream.c:563)
unsafe fn ref_cache_upstream_c_563_get_cmd_multi(
    cmd_fd: c_int,
    opts: *mut Options,
    multi: *mut CURLM,
    mdata: *mut Multi_data,
    running: c_int,
) -> c_int {
    let mut hexmd5 = [0 as c_char; MD5_LEN];
    let mut downstream: *mut Downstream = std::ptr::null_mut();
    let mut msg = Upstream_msg {
        id: 0,
        code: Upstream_msg_code::US_RESULT,
        val: 0,
    };
    let mut res = -1;
    let mut downstream_id: c_uint = 0;

    let clen =
        ref_cache_upstream_c_146_recv_cmd_data(cmd_fd, hexmd5.as_mut_ptr(), &mut downstream_id);
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
        ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &mut msg, -1);
    }

    let download = ref_cache_upstream_c_445_get_download(opts, multi, mdata, hexmd5.as_mut_ptr());
    if download.is_null() {
        {
            if res != 1 {
                msg.id = downstream_id;
                msg.code = Upstream_msg_code::US_RESULT;
                msg.val = 500;
                ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &mut msg, -1);
                libc::free(downstream.cast());
            }
        }
        return res;
    }

    downstream = (*download).downstream;
    while !downstream.is_null() && (*downstream).cmd_fd != cmd_fd {
        downstream = (*downstream).next;
    }
    if !downstream.is_null() {
        return 1;
    }

    downstream = ref_cache_upstream_c_296_new_downstream(cmd_fd, downstream_id);
    if downstream.is_null() {
        {
            if res != 1 {
                msg.id = downstream_id;
                msg.code = Upstream_msg_code::US_RESULT;
                msg.val = 500;
                ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &mut msg, -1);
                libc::free(downstream.cast());
            }
        }
        return res;
    }

    let downstream_fd = libc::dup((*download).file_fd);
    if downstream_fd < 0 {
        {
            if res != 1 {
                msg.id = downstream_id;
                msg.code = Upstream_msg_code::US_RESULT;
                msg.val = 500;
                ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &mut msg, -1);
                libc::free(downstream.cast());
            }
        }
        return res;
    }

    msg.id = downstream_id;
    msg.code = Upstream_msg_code::US_START;
    msg.val = if ((*download).flags & DL_CLENGTH) != 0 {
        (*download).size
    } else {
        -1
    };
    if ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &mut msg, downstream_fd) < 0 {
        libc::close(downstream_fd);
        {
            if res != 1 {
                msg.id = downstream_id;
                msg.code = Upstream_msg_code::US_RESULT;
                msg.val = 500;
                ref_cache_upstream_c_172_upstream_send_msg(cmd_fd, &mut msg, -1);
                libc::free(downstream.cast());
            }
        }
        return res;
    }
    libc::close(downstream_fd);

    (*downstream).download = download;
    (*downstream).prev = std::ptr::null_mut();
    (*downstream).next = (*download).downstream;
    if !(*download).downstream.is_null() {
        (*(*download).downstream).prev = downstream;
    }
    (*download).downstream = downstream;
    1
}

// original: send_result_code (htslib/ref_cache/upstream.c:635)
unsafe fn ref_cache_upstream_c_635_send_result_code(
    downstream: *mut Downstream,
    code: c_int,
) -> c_int {
    let mut msg = Upstream_msg {
        id: (*downstream).id,
        code: Upstream_msg_code::US_RESULT,
        val: code as i64,
    };
    if ref_cache_upstream_c_172_upstream_send_msg((*downstream).cmd_fd, &mut msg, -1) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Error sending result code to downstream #%u : %s\n".as_ptr(),
            (*downstream).id,
            libc::strerror(*__errno_location()),
        );
        return -1;
    }
    0
}

// original: send_result_code_all (htslib/ref_cache/upstream.c:646)
unsafe fn ref_cache_upstream_c_646_send_result_code_all(
    download: *mut Download,
    code: c_int,
) -> c_int {
    let mut res = 0;
    let mut d = (*download).downstream;
    while !d.is_null() {
        res |= ref_cache_upstream_c_635_send_result_code(d, code);
        d = (*d).next;
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
    let download = clientp.cast::<Download>();
    if ((*download).flags & DL_ABANDON) == 0 {
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
    let download = clientp.cast::<Download>();
    if ((*download).flags & DL_ABANDON) == 0 {
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
    let download = userp.cast::<Download>();
    let bytes = size * nmemb;
    let ucb = buffer.cast::<c_uchar>();

    if ((*download).flags & DL_ABANDON) != 0 {
        return bytes;
    }

    if ((*download).flags & DL_CLENGTH) == 0 {
        let mut clen: curl_off_t = 0;
        let mut rcode: c_long = 500;
        if curl_easy_getinfo_long((*download).curl, CURLINFO_RESPONSE_CODE, &mut rcode) != CURLE_OK
        {
            return 0;
        }
        if rcode != 200 {
            if ref_cache_upstream_c_646_send_result_code_all(download, rcode as c_int) != 0 {
                return 0;
            }
            (*download).flags |= DL_ABANDON;
            return bytes;
        }
        if curl_easy_getinfo_off_t(
            (*download).curl,
            CURLINFO_CONTENT_LENGTH_DOWNLOAD_T,
            &mut clen,
        ) != CURLE_OK
        {
            return 0;
        }
        (*download).flags |= DL_CLENGTH;
        (*download).size = clen as libc::off_t;

        if ref_cache_upstream_c_204_send_msg_all(
            download,
            Upstream_msg_code::US_CONTENT_LENGTH,
            (*download).size,
        ) != 0
        {
            return 0;
        }
    }

    if ref_cache_misc_h_55_do_write_all((*download).file_fd, buffer, bytes) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Error writing to %s/%s : %s\n".as_ptr(),
            (*download).cache_dir,
            (*download).file,
            libc::strerror(*__errno_location()),
        );
        return 0;
    }

    let mut in_ = 0usize;
    let mut out = 0usize;
    while in_ < bytes {
        if libc::isspace(*ucb.add(in_) as c_int) == 0 {
            *ucb.add(out) = libc::toupper(*ucb.add(in_) as c_int) as c_uchar;
            out += 1;
        }
        in_ += 1;
    }
    if out > 0 {
        hts_md5_update((*download).md5_ctx, ucb.cast(), out as c_ulong);
    }

    (*download).received += bytes as libc::off_t;
    if ref_cache_upstream_c_204_send_msg_all(
        download,
        Upstream_msg_code::US_PARTIAL_LENGTH,
        (*download).received,
    ) != 0
    {
        return 0;
    }

    bytes
}

use std::ffi::c_ulong;

// original: sock_func (htslib/ref_cache/upstream.c:753)
unsafe extern "C" fn ref_cache_upstream_c_753_sock_func(
    _easy: *mut CURL,
    s: curl_socket_t,
    action: c_int,
    userp: *mut c_void,
    socketp: *mut c_void,
) -> c_int {
    let mdata = userp.cast::<Multi_data>();
    let polled = socketp.cast::<Pw_item>();

    if action == CURL_POLL_REMOVE {
        assert!(!polled.is_null());
        let res = poll_impl::ref_cache_poll_wrap_epoll_c_126_pw_remove((*mdata).pw, polled, 0);
        if res != 0 {
            if *__errno_location() == libc::EBADF {
                return 0;
            }
            libc::perror(c"Removing file descriptor from poller".as_ptr());
            return res;
        }
        let mc = curl_multi_assign((*mdata).multi, s, std::ptr::null_mut());
        if mc != CURLM_OK {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"curl_multi_assign failed : %s\n".as_ptr(),
                curl_multi_strerror(mc),
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

    if polled.is_null() {
        let polled = poll_impl::ref_cache_poll_wrap_epoll_c_78_pw_register(
            (*mdata).pw,
            s,
            Pw_fd_type::US_CURL,
            events as u32,
            std::ptr::null_mut(),
        );
        if polled.is_null() {
            return -1;
        }
        let mc = curl_multi_assign((*mdata).multi, s, polled.cast());
        if mc != CURLM_OK {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"curl_multi_assign failed : %s\n".as_ptr(),
                curl_multi_strerror(mc),
            );
            return -1;
        }
        0
    } else {
        poll_impl::ref_cache_poll_wrap_epoll_c_106_pw_mod((*mdata).pw, polled, events as u32)
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
        libc::fprintf(hts_sys::stderr.cast(), c"curl_multi_init() failed".as_ptr());
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
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Failed to set options for curl multi handle : %s\n".as_ptr(),
            curl_multi_strerror(mc),
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
    (*mdata).pw = poll_impl::ref_cache_poll_wrap_epoll_c_49_pw_init(0);
    if (*mdata).pw.is_null() {
        libc::perror(c"Initalizing poller".as_ptr());
        return -1;
    }
    (*mdata).downloads = std::ptr::null_mut();
    (*mdata).waiting = std::ptr::null_mut();
    (*mdata).last_waiting = std::ptr::null_mut();
    (*mdata).timeout = -1;
    (*mdata).downloads =
        libc::calloc(ACTIVE_SIZE, std::mem::size_of::<*mut Download>()).cast::<*mut Download>();
    if (*mdata).downloads.is_null() {
        libc::perror(c"".as_ptr());
        return -1;
    }
    (*mdata).ncurls = 4;
    (*mdata).free_curls = 0;
    (*mdata).running = 0;
    (*mdata).curls =
        libc::calloc((*mdata).ncurls as usize, std::mem::size_of::<*mut CURL>()).cast();
    if (*mdata).curls.is_null() {
        libc::perror(c"".as_ptr());
        libc::free((*mdata).downloads.cast());
        return -1;
    }

    for i in 0..(*mdata).ncurls {
        *(*mdata).curls.add(i as usize) = curl_easy_init();
        if (*(*mdata).curls.add(i as usize)).is_null() {
            for j in 0..(*mdata).ncurls {
                if !(*(*mdata).curls.add(j as usize)).is_null() {
                    curl_easy_cleanup(*(*mdata).curls.add(j as usize));
                }
            }
            libc::free((*mdata).curls.cast());
            libc::free((*mdata).downloads.cast());
            return -1;
        }
        let mut cc = curl_easy_setopt_write_callback(
            *(*mdata).curls.add(i as usize),
            CURLOPT_WRITEFUNCTION,
            Some(ref_cache_upstream_c_675_multi_receive_data),
        );
        if cc != CURLE_OK {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't set WRITEFUNCTION on curl handle : %s\n".as_ptr(),
                curl_easy_strerror(cc),
            );
            for j in 0..(*mdata).ncurls {
                if !(*(*mdata).curls.add(j as usize)).is_null() {
                    curl_easy_cleanup(*(*mdata).curls.add(j as usize));
                }
            }
            libc::free((*mdata).curls.cast());
            libc::free((*mdata).downloads.cast());
            return -1;
        }
        cc = curl_easy_setopt_long(*(*mdata).curls.add(i as usize), CURLOPT_NOPROGRESS, 0);
        if cc == CURLE_OK {
            cc = curl_easy_setopt_xfer_callback(
                *(*mdata).curls.add(i as usize),
                CURLOPT_XFERINFOFUNCTION,
                Some(ref_cache_upstream_c_656_progress_callback),
            );
        }
        if cc != CURLE_OK {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't set progress callback in CURL handle : %s\n".as_ptr(),
                curl_easy_strerror(cc),
            );
            for j in 0..(*mdata).ncurls {
                if !(*(*mdata).curls.add(j as usize)).is_null() {
                    curl_easy_cleanup(*(*mdata).curls.add(j as usize));
                }
            }
            libc::free((*mdata).curls.cast());
            libc::free((*mdata).downloads.cast());
            return -1;
        }
        (*mdata).free_curls |= 1u32 << i;
    }
    0
}

// original: rename_download_file (htslib/ref_cache/upstream.c:897)
unsafe fn ref_cache_upstream_c_897_rename_download_file(download: *mut Download) -> c_int {
    if ((*download).flags & DL_OK) != 0 {
        let mut dest = [0 as c_char; MD5_LEN + 3];
        libc::snprintf(
            dest.as_mut_ptr(),
            dest.len(),
            c"%.2s/%.2s/%.28s".as_ptr(),
            (*download).hexmd5.as_ptr(),
            (*download).hexmd5.as_ptr().add(2),
            (*download).hexmd5.as_ptr().add(4),
        );
        if libc::renameat(
            (*download).cache_fd,
            (*download).file,
            (*download).cache_fd,
            dest.as_ptr(),
        ) != 0
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't rename %s/%s to %s/%s: %s\n".as_ptr(),
                (*download).cache_dir,
                (*download).file,
                (*download).cache_dir,
                dest.as_ptr(),
                libc::strerror(*__errno_location()),
            );
            (*download).flags &= !DL_OK;
            return -1;
        }
        libc::memcpy((*download).file.cast(), dest.as_ptr().cast(), MD5_LEN + 3);
    }
    0
}

// original: finish_download (htslib/ref_cache/upstream.c:916)
unsafe fn ref_cache_upstream_c_916_finish_download(
    download: *mut Download,
    multi: *mut CURLM,
    msg: *mut CURLMsg,
) -> c_int {
    let mut rcode: c_long = 500;
    let cc = (*msg).data.result;
    let mut md5 = [0 as c_uchar; 16];

    if cc != CURLE_OK {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Download of %.32s failed: %s\n".as_ptr(),
            (*download).hexmd5.as_ptr(),
            curl_easy_strerror(cc),
        );
        if ref_cache_upstream_c_646_send_result_code_all(download, rcode as c_int) != 0 {
            return -1;
        }
        let mc = curl_multi_remove_handle(multi, (*download).curl);
        if mc != CURLM_OK {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't remove easy handle from curl_multi : %s\n".as_ptr(),
                curl_multi_strerror(mc),
            );
            return -1;
        }
        ref_cache_upstream_c_331_free_download(download);
        return 0;
    }

    if curl_easy_getinfo_long((*msg).easy_handle, CURLINFO_RESPONSE_CODE, &mut rcode) != CURLE_OK {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't get response code : %s\n".as_ptr(),
            curl_easy_strerror(cc),
        );
        return -1;
    }
    if rcode != 200 {
        if ref_cache_upstream_c_646_send_result_code_all(download, rcode as c_int) != 0 {
            return -1;
        }
        let mc = curl_multi_remove_handle(multi, (*download).curl);
        if mc != CURLM_OK {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't remove easy handle from curl_multi : %s\n".as_ptr(),
                curl_multi_strerror(mc),
            );
            return -1;
        }
        ref_cache_upstream_c_331_free_download(download);
        return 0;
    }

    if (*download).size == 0 {
        (*download).size = (*download).received;
    }
    if (*download).received != (*download).size {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Downloading %.32s : Content-Length was a lie. Expected %ld, got %ld\n".as_ptr(),
            (*download).hexmd5.as_ptr(),
            (*download).size as c_long,
            (*download).received as c_long,
        );
        if ref_cache_upstream_c_646_send_result_code_all(download, 502) != 0 {
            return -1;
        }
        let mc = curl_multi_remove_handle(multi, (*download).curl);
        if mc != CURLM_OK {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't remove easy handle from curl_multi : %s\n".as_ptr(),
                curl_multi_strerror(mc),
            );
            return -1;
        }
        ref_cache_upstream_c_331_free_download(download);
        return 0;
    }

    hts_md5_final(md5.as_mut_ptr(), (*download).md5_ctx);
    for i in 0..16 {
        let byte = (ref_cache_misc_h_38_hexval((*download).hexmd5[i * 2]) << 4)
            | ref_cache_misc_h_38_hexval((*download).hexmd5[i * 2 + 1]);
        if byte != md5[i] as c_int {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Downloading %.32s : MD5 checksum didn't match.\n".as_ptr(),
                (*download).hexmd5.as_ptr(),
            );
            if ref_cache_upstream_c_646_send_result_code_all(download, 502) != 0 {
                return -1;
            }
            let mc = curl_multi_remove_handle(multi, (*download).curl);
            if mc != CURLM_OK {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Couldn't remove easy handle from curl_multi : %s\n".as_ptr(),
                    curl_multi_strerror(mc),
                );
                return -1;
            }
            ref_cache_upstream_c_331_free_download(download);
            return 0;
        }
    }

    (*download).flags |= DL_OK;
    if ref_cache_upstream_c_897_rename_download_file(download) != 0 {
        return -1;
    }

    let mc = curl_multi_remove_handle(multi, (*download).curl);
    if mc != CURLM_OK {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't remove easy handle from curl_multi : %s\n".as_ptr(),
            curl_multi_strerror(mc),
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
    mdata: *mut Multi_data,
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
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Error from curl socket #%d: %s\n".as_ptr(),
            fd,
            curl_multi_strerror(mc),
        );
        return -1;
    }
    if running < (*mdata).running {
        let mut msgs_in_queue = 0;
        let mut nfinished = 0;
        (*mdata).running = running;
        loop {
            let msg = curl_multi_info_read(multi, &mut msgs_in_queue);
            if msg.is_null() {
                break;
            }
            if (*msg).msg == CURLMSG_DONE {
                let mut download: *mut c_void = std::ptr::null_mut();
                let c = curl_easy_getinfo_ptr((*msg).easy_handle, CURLINFO_PRIVATE, &mut download);
                if c != CURLE_OK {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"curl_easy_getinfo failed: %s\n".as_ptr(),
                        curl_easy_strerror(c),
                    );
                    return -1;
                }
                if ref_cache_upstream_c_916_finish_download(download.cast(), multi, msg) != 0 {
                    return -1;
                }
                nfinished += 1;
            }
        }
        if nfinished != 0 {
            loop {
                let started = ref_cache_upstream_c_385_start_new_download(multi, mdata);
                if started <= 0 {
                    if started < 0 {
                        return -1;
                    }
                    break;
                }
            }
        }
    } else {
        (*mdata).running = running;
        if (*mdata).running == 0 {
            (*mdata).timeout = -1;
        }
    }
    0
}

// original: run_epoll_loop (htslib/ref_cache/upstream.c:1047)
unsafe fn ref_cache_upstream_c_1047_run_epoll_loop(
    opts: *mut Options,
    nfds: c_uint,
    cmd_fds: *mut c_int,
    liveness_fd: c_int,
    multi: *mut CURLM,
    mdata: *mut Multi_data,
) {
    let mut events: [libc::epoll_event; MAX_EVENTS as usize] = std::mem::zeroed();
    let cmd_pollers = libc::malloc((nfds as usize + 1) * std::mem::size_of::<*mut Pw_item>())
        .cast::<*mut Pw_item>();
    if cmd_pollers.is_null() {
        return;
    }
    let mut npolled = 0u32;
    let mut running = 1;

    while npolled < nfds {
        *cmd_pollers.add(npolled as usize) = poll_impl::ref_cache_poll_wrap_epoll_c_78_pw_register(
            (*mdata).pw,
            *cmd_fds.add(npolled as usize),
            Pw_fd_type::US_COMMAND,
            PW_IN as u32,
            std::ptr::null_mut(),
        );
        if (*cmd_pollers.add(npolled as usize)).is_null() {
            break;
        }
        npolled += 1;
    }
    if npolled == nfds {
        *cmd_pollers.add(nfds as usize) = poll_impl::ref_cache_poll_wrap_epoll_c_78_pw_register(
            (*mdata).pw,
            liveness_fd,
            Pw_fd_type::US_LIVE,
            (PW_HUP | PW_ERR) as u32,
            std::ptr::null_mut(),
        );
        if (*cmd_pollers.add(nfds as usize)).is_null() {
            npolled = nfds;
        } else {
            npolled = nfds + 1;
        }
    }

    if npolled == nfds + 1 {
        while running != 0 || (*mdata).running > 0 {
            let nevents = poll_impl::ref_cache_poll_wrap_epoll_c_120_pw_wait(
                (*mdata).pw,
                events.as_mut_ptr(),
                MAX_EVENTS,
                if (*mdata).timeout < c_int::MAX as c_long {
                    (*mdata).timeout as c_int
                } else {
                    c_int::MAX
                },
            );
            if nevents == -1 {
                if *__errno_location() == libc::EINTR {
                    continue;
                }
                libc::perror(c"poll_wait".as_ptr());
                break;
            }
            if nevents == 0
                && ref_cache_upstream_c_996_handle_curl_socket(multi, mdata, 0, CURL_SOCKET_TIMEOUT)
                    != 0
            {
                break;
            }

            for n in 0..nevents {
                let evts = events[n as usize].events;
                let polled = events[n as usize].u64 as *mut Pw_item;
                match (*polled).fd_type {
                    Pw_fd_type::US_COMMAND => {
                        if (*opts).verbosity > 2 {
                            libc::fprintf(
                                hts_sys::stderr.cast(),
                                c"Upstream received command\n".as_ptr(),
                            );
                        }
                        if ref_cache_upstream_c_563_get_cmd_multi(
                            (*polled).fd,
                            opts,
                            multi,
                            mdata,
                            running,
                        ) <= 0
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
                        if (*opts).verbosity > 2 {
                            libc::fprintf(
                                hts_sys::stderr.cast(),
                                c"Upstream received curl event %d on fd #%d\n".as_ptr(),
                                e,
                                (*polled).fd,
                            );
                        }
                        if ref_cache_upstream_c_996_handle_curl_socket(
                            multi,
                            mdata,
                            e,
                            (*polled).fd,
                        ) != 0
                        {
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
        if !(*cmd_pollers.add(i as usize)).is_null() {
            poll_impl::ref_cache_poll_wrap_epoll_c_126_pw_remove(
                (*mdata).pw,
                *cmd_pollers.add(i as usize),
                0,
            );
        }
    }
    libc::free(cmd_pollers.cast());
}

// original: run_multi_upstream_handler (htslib/ref_cache/upstream.c:1131)
unsafe fn ref_cache_upstream_c_1131_run_multi_upstream_handler(
    opts: *mut Options,
    cmd_fds: *mut c_int,
    liveness_fd: c_int,
) -> c_int {
    let res = -1;
    let mut mdata: Multi_data = std::mem::zeroed();
    let multi = ref_cache_upstream_c_808_get_multi(&mut mdata);
    if multi.is_null() {
        return -1;
    }

    if ref_cache_upstream_c_838_init_multi_data(multi, &mut mdata) != 0 {
        curl_multi_cleanup(multi);
        return -1;
    }

    ref_cache_upstream_c_1047_run_epoll_loop(
        opts,
        (*opts).max_kids as c_uint,
        cmd_fds,
        liveness_fd,
        multi,
        &mut mdata,
    );

    for i in 0..mdata.ncurls {
        curl_multi_remove_handle(multi, *mdata.curls.add(i as usize));
        curl_easy_cleanup(*mdata.curls.add(i as usize));
    }
    libc::free(mdata.downloads.cast());
    curl_multi_cleanup(multi);
    res
}

// original: run_upstream_handler (htslib/ref_cache/upstream.c:1157)
pub unsafe fn ref_cache_upstream_c_1157_run_upstream_handler(
    opts: *mut Options,
    sockets: *mut c_int,
    liveness_fd: c_int,
) -> c_int {
    let mut sa: libc::sigaction = std::mem::zeroed();
    let mut old_sa: libc::sigaction = std::mem::zeroed();

    sa.sa_sigaction = libc::SIG_IGN;
    libc::sigemptyset(&mut sa.sa_mask);
    sa.sa_flags = 0;

    if libc::sigaction(libc::SIGPIPE, &sa, &mut old_sa) != 0 {
        libc::perror(c"sigaction(SIGPIPE)".as_ptr());
        return -1;
    }

    if curl_global_init(CURL_GLOBAL_ALL) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't initialize libcurl\n".as_ptr(),
        );
        return -1;
    }

    let cvi = curl_version_info(CURLVERSION_NOW);
    if cvi.is_null() {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't get curl version information\n".as_ptr(),
        );
        return -1;
    }
    CURL_VERSION_NUM = (*cvi).version_num;

    let res = ref_cache_upstream_c_1131_run_multi_upstream_handler(opts, sockets, liveness_fd);

    curl_global_cleanup();

    if libc::sigaction(libc::SIGPIPE, &old_sa, std::ptr::null_mut()) != 0 {
        libc::perror(c"sigaction(SIGPIPE)".as_ptr());
    }
    res
}
