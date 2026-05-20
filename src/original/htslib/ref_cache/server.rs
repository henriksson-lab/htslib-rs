use crate::htslib_mini_rs::cram;
use crate::original_stubs::{functions::*, structs};
use std::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

const REF_CACHE_ON_READ_LIST: c_uint = 0x01;
const REF_CACHE_ON_WRITE_LIST: c_uint = 0x02;
const REF_CACHE_READ_CLOSED: c_uint = 0x04;
const REF_CACHE_READ_DRAIN: c_uint = 0x08;
const REF_CACHE_CAN_WRITE: c_uint = 0x10;

const REF_CACHE_SV_LISTENER: c_int = 1;
const REF_CACHE_SV_UPSTREAM: c_int = 2;
const REF_CACHE_SV_LOG: c_int = 3;
const REF_CACHE_SV_CLIENT: c_int = 4;

const REF_CACHE_NI_MAXHOST: usize = 1025;
const REF_CACHE_MAX_EVENTS: c_int = 128;
const REF_CACHE_MAX_UA_LEN: usize = 128;
const REF_CACHE_MAX_REFERRER_LEN: usize = 128;
const REF_CACHE_MAX_REQUEST_LEN: usize = 64;

const REF_CACHE_US_START: c_int = 0;
const REF_CACHE_US_CONTENT_LENGTH: c_int = 1;
const REF_CACHE_US_PARTIAL_LENGTH: c_int = 2;
const REF_CACHE_US_RESULT: c_int = 3;

const REF_CACHE_READ_BLOCKED: c_int = 0;
const REF_CACHE_READ_MORE: c_int = 1;
const REF_CACHE_READ_EOF: c_int = 2;
const REF_CACHE_READ_ERROR: c_int = 3;

const REF_CACHE_WRITE_BLOCKED: c_int = 0;
const REF_CACHE_WRITE_BLOCKED_UPSTREAM: c_int = 1;
const REF_CACHE_WRITE_MORE: c_int = 2;
const REF_CACHE_WRITE_COMPLETE: c_int = 3;
const REF_CACHE_WRITE_EOF: c_int = 4;
const REF_CACHE_WRITE_ERROR: c_int = 5;

const REF_CACHE_LOG_BUF_SZ: usize = 0x10000;
const REF_CACHE_LOG_BUF_MASK: usize = REF_CACHE_LOG_BUF_SZ - 1;

#[repr(C)]
struct HttpParserLayout {
    state: c_int,
    req_type: c_int,
    http_vers: c_int,
    trans_enc: c_int,
    content_length: libc::c_ulong,
    bytes: libc::c_ulong,
    uri: *mut c_char,
    key: *mut c_char,
    val: *mut c_char,
    buffer: *mut c_char,
    user_agent: *mut c_char,
    referrer: *mut c_char,
    range_from: libc::off_t,
    range_to: libc::off_t,
    key_sz: usize,
    key_used: usize,
    val_sz: usize,
    val_used: usize,
    upstream: c_int,
    flags: c_uint,
    in_: c_uint,
    out: c_uint,
    pos: c_uint,
    used: c_uint,
}

#[repr(C)]
struct ClientLayout {
    polled: *mut structs::Pw_item,
    prev: *mut structs::Client,
    next: *mut structs::Client,
    prev_write: *mut structs::Client,
    next_write: *mut structs::Client,
    host: *mut c_char,
    transact: *mut structs::Transaction,
    last_transact: *mut structs::Transaction,
    parser: HttpParserLayout,
    flags: c_uint,
}

#[repr(C)]
pub struct RefCacheClientsLayout {
    client_pool: *mut cram::pool_alloc_t,
    can_read: *mut structs::Client,
    can_write: *mut structs::Client,
    count: usize,
}

#[repr(C)]
struct RefCacheLogBufferLayout {
    buffer: [c_char; REF_CACHE_LOG_BUF_SZ],
    in_: usize,
    out: usize,
}

#[repr(C)]
struct RefCacheMatchAddrLayout {
    family: libc::sa_family_t,
    mask_bytes: u8,
    mask: u8,
    addr: [c_uchar; 16],
}

#[repr(C)]
struct RefCacheOptionsLayout {
    cache_dir: *const c_char,
    log_dir: *const c_char,
    error_log_file: *const c_char,
    log: *mut libc::FILE,
    upstream_url: *const c_char,
    upstream_url_len: usize,
    match_addrs: *mut RefCacheMatchAddrLayout,
    num_match_addrs: usize,
    match_addrs_size: usize,
    first_ip6: usize,
    max_log_sz: libc::off_t,
    cache_fd: c_int,
    listen_fds: c_int,
    daemon: c_int,
    port: u16,
    nlogs: u16,
    max_kids: u16,
    verbosity: u8,
    no_log: u8,
}

#[repr(C)]
struct RefCacheListenersLayout {
    nsocks: usize,
    sockets: *mut c_int,
}

#[repr(C)]
struct RefCacheUpstreamMsgLayout {
    id: c_uint,
    code: c_int,
    val: i64,
}

#[repr(C)]
struct RefCachePwItemLayout {
    fd: c_int,
    fd_type: c_int,
    userp: *mut c_void,
}

// original: Client (htslib/ref_cache/server.c:71)
#[repr(C)]
pub struct Client {
    _private: [u8; 0],
}

// original: Log_buffer (htslib/ref_cache/server.c:99)
#[repr(C)]
pub struct Log_buffer {
    _private: [u8; 0],
}

// original: SocketAddress (htslib/ref_cache/server.c:100)
#[repr(C)]
pub struct SocketAddress {
    _private: [u8; 0],
}

// original: init_clients (htslib/ref_cache/server.c:107)
pub unsafe fn ref_cache_server_c_107_init_clients(clients: *mut RefCacheClientsLayout) -> c_int {
    (*clients).client_pool =
        cram::cram_pooled_alloc_c_64_pool_create(std::mem::size_of::<ClientLayout>());
    if (*clients).client_pool.is_null() {
        return -1;
    }
    (*clients).can_read = std::ptr::null_mut();
    (*clients).can_write = std::ptr::null_mut();
    (*clients).count = 0;
    0
}

// original: read_stack_push (htslib/ref_cache/server.c:115)
pub unsafe fn ref_cache_server_c_115_read_stack_push(
    client: *mut structs::Client,
    stack: *mut *mut structs::Client,
) {
    let client_l = client.cast::<ClientLayout>();
    if ((*client_l).flags & REF_CACHE_ON_READ_LIST) != 0 {
        return;
    }
    (*client_l).flags |= REF_CACHE_ON_READ_LIST;
    if !(*stack).is_null() {
        (*(*stack).cast::<ClientLayout>()).prev = client;
    }
    (*client_l).next = *stack;
    (*client_l).prev = std::ptr::null_mut();
    *stack = client;
}

// original: read_stack_pop (htslib/ref_cache/server.c:125)
pub unsafe fn ref_cache_server_c_125_read_stack_pop(
    clients: *mut RefCacheClientsLayout,
) -> *mut structs::Client {
    let can_read = (*clients).can_read;
    if can_read.is_null() {
        return std::ptr::null_mut();
    }
    let can_read_l = can_read.cast::<ClientLayout>();
    (*clients).can_read = (*can_read_l).next;
    if !(*clients).can_read.is_null() {
        (*(*clients).can_read.cast::<ClientLayout>()).prev = std::ptr::null_mut();
    }
    (*can_read_l).prev = std::ptr::null_mut();
    (*can_read_l).next = std::ptr::null_mut();
    assert!(((*can_read_l).flags & REF_CACHE_ON_READ_LIST) != 0);
    (*can_read_l).flags &= !REF_CACHE_ON_READ_LIST;
    can_read
}

// original: read_stack_remove (htslib/ref_cache/server.c:137)
pub unsafe fn ref_cache_server_c_137_read_stack_remove(
    clients: *mut RefCacheClientsLayout,
    client: *mut structs::Client,
) {
    let client_l = client.cast::<ClientLayout>();
    (*client_l).flags &= !REF_CACHE_ON_READ_LIST;
    if (*clients).can_read == client {
        (*clients).can_read = (*client_l).next;
        if !(*clients).can_read.is_null() {
            (*(*clients).can_read.cast::<ClientLayout>()).prev = std::ptr::null_mut();
        }
    } else {
        let prev = (*client_l).prev;
        let next = (*client_l).next;
        (*prev.cast::<ClientLayout>()).next = next;
        if !next.is_null() {
            (*next.cast::<ClientLayout>()).prev = prev;
        }
    }
    (*client_l).prev = std::ptr::null_mut();
    (*client_l).next = std::ptr::null_mut();
}

// original: write_stack_push (htslib/ref_cache/server.c:151)
pub unsafe fn ref_cache_server_c_151_write_stack_push(
    client: *mut structs::Client,
    stack: *mut *mut structs::Client,
) {
    let client_l = client.cast::<ClientLayout>();
    if ((*client_l).flags & REF_CACHE_ON_WRITE_LIST) != 0 {
        return;
    }
    (*client_l).flags |= REF_CACHE_ON_WRITE_LIST;
    if !(*stack).is_null() {
        (*(*stack).cast::<ClientLayout>()).prev_write = client;
    }
    (*client_l).next_write = *stack;
    (*client_l).prev_write = std::ptr::null_mut();
    *stack = client;
}

// original: write_stack_pop (htslib/ref_cache/server.c:161)
pub unsafe fn ref_cache_server_c_161_write_stack_pop(
    clients: *mut RefCacheClientsLayout,
) -> *mut structs::Client {
    let can_write = (*clients).can_write;
    if can_write.is_null() {
        return std::ptr::null_mut();
    }
    let can_write_l = can_write.cast::<ClientLayout>();
    (*clients).can_write = (*can_write_l).next_write;
    if !(*clients).can_write.is_null() {
        (*(*clients).can_write.cast::<ClientLayout>()).prev_write = std::ptr::null_mut();
    }
    (*can_write_l).prev_write = std::ptr::null_mut();
    (*can_write_l).next_write = std::ptr::null_mut();
    assert!(((*can_write_l).flags & REF_CACHE_ON_WRITE_LIST) != 0);
    (*can_write_l).flags &= !REF_CACHE_ON_WRITE_LIST;
    can_write
}

// original: write_stack_remove (htslib/ref_cache/server.c:173)
pub unsafe fn ref_cache_server_c_173_write_stack_remove(
    clients: *mut RefCacheClientsLayout,
    client: *mut structs::Client,
) {
    let client_l = client.cast::<ClientLayout>();
    (*client_l).flags &= !REF_CACHE_ON_WRITE_LIST;
    if (*clients).can_write == client {
        (*clients).can_write = (*client_l).next_write;
        if !(*clients).can_write.is_null() {
            (*(*clients).can_write.cast::<ClientLayout>()).prev_write = std::ptr::null_mut();
        }
    } else {
        let prev = (*client_l).prev_write;
        let next = (*client_l).next_write;
        (*prev.cast::<ClientLayout>()).next_write = next;
        if !next.is_null() {
            (*next.cast::<ClientLayout>()).prev_write = prev;
        }
    }
    (*client_l).prev_write = std::ptr::null_mut();
    (*client_l).next_write = std::ptr::null_mut();
}

// original: get_free_client (htslib/ref_cache/server.c:187)
pub unsafe fn ref_cache_server_c_187_get_free_client(
    clients: *mut RefCacheClientsLayout,
) -> *mut structs::Client {
    let client =
        cram::cram_pooled_alloc_c_115_pool_alloc((*clients).client_pool).cast::<structs::Client>();
    libc::memset(client.cast(), 0, std::mem::size_of::<ClientLayout>());
    (*clients).count += 1;
    client
}

// original: close_client (htslib/ref_cache/server.c:194)
pub unsafe fn ref_cache_server_c_194_close_client(
    clients: *mut RefCacheClientsLayout,
    client: *mut structs::Client,
    pw: *mut structs::Poll_wrap,
) {
    let client_l = client.cast::<ClientLayout>();
    assert!((*clients).count > 0);
    if !(*client_l).polled.is_null() {
        ref_cache_poll_wrap_epoll_c_126_pw_remove(pw, (*client_l).polled, 1);
        (*client_l).polled = std::ptr::null_mut();
    }

    if ((*client_l).flags & REF_CACHE_ON_READ_LIST) != 0 {
        ref_cache_server_c_137_read_stack_remove(clients, client);
    }

    if ((*client_l).flags & REF_CACHE_ON_WRITE_LIST) != 0 {
        ref_cache_server_c_173_write_stack_remove(clients, client);
    }

    ref_cache_http_parser_c_131_cleanup_http_parser(
        (&mut (*client_l).parser as *mut HttpParserLayout).cast(),
    );

    ref_cache_transaction_c_196_free_transaction_list((*client_l).transact);
    (*client_l).transact = std::ptr::null_mut();
    (*client_l).last_transact = std::ptr::null_mut();
    if !(*client_l).host.is_null() {
        libc::free((*client_l).host.cast());
        (*client_l).host = std::ptr::null_mut();
    }

    cram::cram_pooled_alloc_c_144_pool_free((*clients).client_pool, client.cast());
    (*clients).count -= 1;
}

// original: check_addr_allowed (htslib/ref_cache/server.c:222)
pub unsafe fn ref_cache_server_c_222_check_addr_allowed(
    opts: *const structs::Options,
    mut family: libc::sa_family_t,
    addr: *const structs::SocketAddress,
    addrlen: libc::socklen_t,
) -> c_int {
    let opts = opts.cast::<RefCacheOptionsLayout>();
    let start;
    let end;
    let mut addrp: *const c_uchar;

    if (*opts).match_addrs.is_null() {
        return 0;
    }

    if family == libc::AF_INET as libc::sa_family_t
        && addrlen as usize == std::mem::size_of::<libc::sockaddr_in>()
    {
        let in4 = addr.cast::<libc::sockaddr_in>();
        addrp = (&(*in4).sin_addr as *const libc::in_addr).cast();
        start = 0usize;
        end = (*opts).first_ip6;
    } else if family == libc::AF_INET6 as libc::sa_family_t
        && addrlen as usize == std::mem::size_of::<libc::sockaddr_in6>()
    {
        let v4mapped_prefix: [c_uchar; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff];
        let in6 = addr.cast::<libc::sockaddr_in6>();
        addrp = (&(*in6).sin6_addr as *const libc::in6_addr).cast();
        if libc::memcmp(
            addrp.cast(),
            v4mapped_prefix.as_ptr().cast(),
            v4mapped_prefix.len(),
        ) == 0
        {
            family = libc::AF_INET as libc::sa_family_t;
            addrp = addrp.add(12);
            start = 0;
            end = (*opts).first_ip6;
        } else {
            start = (*opts).first_ip6;
            end = (*opts).num_match_addrs;
        }
    } else {
        return -1;
    }

    for i in start..end {
        let ma = (*opts).match_addrs.add(i);
        if family != (*ma).family {
            continue;
        }
        if libc::memcmp(
            addrp.cast(),
            (*ma).addr.as_ptr().cast(),
            (*ma).mask_bytes as usize,
        ) == 0
            && (*addrp.add((*ma).mask_bytes as usize) & (*ma).mask)
                == ((*ma).addr[(*ma).mask_bytes as usize] & (*ma).mask)
        {
            return 0;
        }
    }
    -1
}

// original: handle_incoming (htslib/ref_cache/server.c:268)
pub unsafe fn ref_cache_server_c_268_handle_incoming(
    opts: *const structs::Options,
    listener: c_int,
    upstream: c_int,
    pw: *mut structs::Poll_wrap,
    clients: *mut RefCacheClientsLayout,
) {
    let opts_l = opts.cast::<RefCacheOptionsLayout>();
    let mut addr: libc::sockaddr_storage = std::mem::zeroed();
    let mut addrlen = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    let mut host = [0 as c_char; REF_CACHE_NI_MAXHOST];

    loop {
        let incoming = libc::accept(
            listener,
            (&mut addr as *mut libc::sockaddr_storage).cast(),
            &mut addrlen,
        );
        if incoming < 0 {
            break;
        }

        let res = libc::getnameinfo(
            (&addr as *const libc::sockaddr_storage).cast(),
            addrlen,
            host.as_mut_ptr(),
            host.len() as libc::socklen_t,
            std::ptr::null_mut(),
            0,
            libc::NI_NUMERICHOST,
        );
        if res != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't resolve host for incoming client: %s\n".as_ptr(),
                libc::gai_strerror(res),
            );
            libc::strcpy(host.as_mut_ptr(), c"<unknown>".as_ptr());
        }

        if ref_cache_server_c_222_check_addr_allowed(
            opts,
            addr.ss_family,
            (&addr as *const libc::sockaddr_storage).cast(),
            addrlen,
        ) != 0
        {
            if (*opts_l).verbosity > 1 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Rejected incoming client from %s\n".as_ptr(),
                    host.as_ptr(),
                );
            }
            libc::close(incoming);
            continue;
        }

        if (*opts_l).verbosity > 1 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Accepted incoming client from %s on fd #%d\n".as_ptr(),
                host.as_ptr(),
                incoming,
            );
        }

        if ref_cache_misc_h_40_setnonblock(incoming) != 0 {
            libc::close(incoming);
            return;
        }

        let client = ref_cache_server_c_187_get_free_client(clients);
        if client.is_null() {
            libc::perror(c"Getting free client struct".as_ptr());
            libc::close(incoming);
            return;
        }

        let events =
            (libc::EPOLLIN | libc::EPOLLOUT | libc::EPOLLERR | libc::EPOLLHUP | libc::EPOLLET)
                as u32;
        let client_l = client.cast::<ClientLayout>();
        (*client_l).polled = ref_cache_poll_wrap_epoll_c_78_pw_register(
            pw,
            incoming,
            REF_CACHE_SV_CLIENT,
            events,
            client.cast(),
        );
        if (*client_l).polled.is_null() {
            libc::perror(c"Registering client with poll".as_ptr());
            ref_cache_server_c_194_close_client(clients, client, pw);
            return;
        }
        (*client_l).flags = 0;
        (*client_l).host = libc::strdup(host.as_ptr());
        (*client_l).transact = std::ptr::null_mut();
        (*client_l).last_transact = std::ptr::null_mut();
        if ref_cache_http_parser_c_111_init_http_parser(
            (&mut (*client_l).parser as *mut HttpParserLayout).cast(),
            upstream,
        ) < 0
        {
            libc::perror(c"Setting up Http_Parser struct".as_ptr());
            ref_cache_server_c_194_close_client(clients, client, pw);
            return;
        }
    }

    match *crate::htslib_mini_rs::c_compat::__errno_location() {
        libc::EAGAIN
        | libc::EINTR
        | libc::ENETDOWN
        | libc::EPROTO
        | libc::ENOPROTOOPT
        | libc::EHOSTUNREACH
        | libc::EOPNOTSUPP
        | libc::ENETUNREACH => {}
        _ => libc::perror(c"Error while accepting incoming client".as_ptr()),
    }
}

// original: client_add_transaction (htslib/ref_cache/server.c:352)
pub unsafe fn ref_cache_server_c_352_client_add_transaction(
    client: *mut structs::Client,
    transact: *mut structs::Transaction,
) {
    let client_l = client.cast::<ClientLayout>();
    if (*client_l).transact.is_null() {
        assert!((*client_l).last_transact.is_null());
        (*client_l).transact = transact;
        (*client_l).last_transact = transact;
    } else {
        ref_cache_transaction_c_623_transaction_set_next((*client_l).last_transact, transact);
        (*client_l).last_transact = transact;
    }
}

// original: handle_upstream (htslib/ref_cache/server.c:362)
pub unsafe fn ref_cache_server_c_362_handle_upstream(
    upstream: c_int,
    clients: *mut RefCacheClientsLayout,
    verbosity: c_int,
) {
    let mut msg = RefCacheUpstreamMsgLayout {
        id: 0,
        code: 0,
        val: 0,
    };
    let mut fd = -1;

    let res = ref_cache_upstream_c_215_upstream_recv_msg(
        upstream,
        (&mut msg as *mut RefCacheUpstreamMsgLayout).cast(),
        &mut fd,
    );
    if res <= 0 {
        return;
    }

    if verbosity > 2 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Message from upstream: %u, %d, %lld\n".as_ptr(),
            msg.id,
            msg.code,
            msg.val as libc::c_longlong,
        );
    }

    match msg.code {
        REF_CACHE_US_START => {
            ref_cache_transaction_c_680_got_download_started(
                msg.id,
                msg.val,
                fd,
                &mut (*clients).can_write,
            );
        }
        REF_CACHE_US_PARTIAL_LENGTH => {
            ref_cache_transaction_c_698_got_download_part(
                msg.id,
                msg.val,
                &mut (*clients).can_write,
            );
        }
        REF_CACHE_US_CONTENT_LENGTH => {
            ref_cache_transaction_c_715_got_download_clen(
                msg.id,
                msg.val,
                &mut (*clients).can_write,
            );
        }
        REF_CACHE_US_RESULT => {
            ref_cache_transaction_c_730_got_download_result(
                msg.id,
                msg.val,
                &mut (*clients).can_write,
            );
        }
        _ => libc::abort(),
    }
}

// original: queue_transaction_write (htslib/ref_cache/server.c:395)
pub unsafe fn ref_cache_server_c_395_queue_transaction_write(
    transact: *mut structs::Transaction,
    write_stack: *mut *mut structs::Client,
) {
    let client = ref_cache_transaction_c_210_transaction_get_client(transact);
    let client_l = client.cast::<ClientLayout>();

    if ((*client_l).flags & REF_CACHE_ON_WRITE_LIST) != 0 {
        return;
    }
    if transact != (*client_l).transact {
        return;
    }
    if ref_cache_transaction_c_628_transaction_has_data_to_send(transact) == 0 {
        return;
    }
    ref_cache_server_c_151_write_stack_push(client, write_stack);
}

// original: eat_input (htslib/ref_cache/server.c:422)
pub unsafe fn ref_cache_server_c_422_eat_input(fd: c_int) -> c_int {
    let mut buffer = [0 as c_char; 65536];
    let mut bytes;

    loop {
        bytes = libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len());
        if !(bytes < 0 && *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EINTR) {
            break;
        }
    }
    if bytes < 0 {
        let errno = *crate::htslib_mini_rs::c_compat::__errno_location();
        if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
            return REF_CACHE_READ_BLOCKED;
        }
        return REF_CACHE_READ_ERROR;
    }
    if bytes == 0 {
        return REF_CACHE_READ_EOF;
    }
    if (bytes as usize) < buffer.len() {
        REF_CACHE_READ_BLOCKED
    } else {
        REF_CACHE_READ_MORE
    }
}

// original: queue_log_msg (htslib/ref_cache/server.c:436)
pub unsafe fn ref_cache_server_c_436_queue_log_msg(
    log_buf: *mut structs::Log_buffer,
    transact: *mut structs::Transaction,
) {
    let log_buf = log_buf.cast::<RefCacheLogBufferLayout>();
    let mut buf = [0 as c_char;
        128 + REF_CACHE_MAX_UA_LEN + REF_CACHE_MAX_REFERRER_LEN + REF_CACHE_MAX_REQUEST_LEN + 1025];
    let mut b = buf.as_mut_ptr();
    let mut bytes =
        ref_cache_transaction_c_769_make_log_message(transact, buf.as_mut_ptr(), buf.len());
    debug_assert!(bytes < buf.len());
    if bytes >= buf.len() {
        bytes = buf.len() - 1;
    }

    let space = if (*log_buf).in_ >= (*log_buf).out {
        REF_CACHE_LOG_BUF_SZ - (*log_buf).in_ + (*log_buf).out - 1
    } else {
        (*log_buf).out - (*log_buf).in_ - 1
    };
    if bytes > space {
        return;
    }

    if (*log_buf).in_ >= (*log_buf).out {
        let mut l = REF_CACHE_LOG_BUF_SZ - (*log_buf).in_ - if (*log_buf).out == 0 { 1 } else { 0 };
        if l > bytes {
            l = bytes;
        }
        libc::memcpy(
            (*log_buf).buffer.as_mut_ptr().add((*log_buf).in_).cast(),
            b.cast(),
            l,
        );
        b = b.add(l);
        bytes -= l;
        (*log_buf).in_ = ((*log_buf).in_ + l) & REF_CACHE_LOG_BUF_MASK;
    }
    if bytes > 0 && (*log_buf).in_ < (*log_buf).out {
        assert!(bytes < (*log_buf).out - (*log_buf).in_ - 1);
        libc::memcpy(
            (*log_buf).buffer.as_mut_ptr().add((*log_buf).in_).cast(),
            b.cast(),
            bytes,
        );
        (*log_buf).in_ += bytes;
    }
    assert!((*log_buf).in_ < REF_CACHE_LOG_BUF_SZ);
    assert!((*log_buf).in_ != (*log_buf).out);
}

// original: send_to_log (htslib/ref_cache/server.c:470)
pub unsafe fn ref_cache_server_c_470_send_to_log(
    log_buf: *mut structs::Log_buffer,
    log_fd: c_int,
) -> c_int {
    let log_buf = log_buf.cast::<RefCacheLogBufferLayout>();
    let mut bytes;

    if (*log_buf).out == (*log_buf).in_ {
        return REF_CACHE_WRITE_MORE;
    }
    if (*log_buf).out > (*log_buf).in_ {
        let len = REF_CACHE_LOG_BUF_SZ - (*log_buf).out;
        loop {
            bytes = libc::write(
                log_fd,
                (*log_buf).buffer.as_ptr().add((*log_buf).out).cast(),
                len,
            );
            if !(bytes < 0 && *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EINTR) {
                break;
            }
        }

        if bytes < 0 {
            let errno = *crate::htslib_mini_rs::c_compat::__errno_location();
            if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                return REF_CACHE_WRITE_BLOCKED;
            }
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Couldn't write to log: %s\n".as_ptr(),
                libc::strerror(errno),
            );
            return REF_CACHE_WRITE_ERROR;
        }

        (*log_buf).out = ((*log_buf).out + bytes as usize) & REF_CACHE_LOG_BUF_MASK;
        if (bytes as usize) < len {
            return REF_CACHE_WRITE_BLOCKED;
        }
    }

    if (*log_buf).out == (*log_buf).in_ {
        return REF_CACHE_WRITE_MORE;
    }
    assert!((*log_buf).out < (*log_buf).in_);

    let len = (*log_buf).in_ - (*log_buf).out;
    loop {
        bytes = libc::write(
            log_fd,
            (*log_buf).buffer.as_ptr().add((*log_buf).out).cast(),
            len,
        );
        if !(bytes < 0 && *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EINTR) {
            break;
        }
    }

    if bytes < 0 {
        let errno = *crate::htslib_mini_rs::c_compat::__errno_location();
        if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
            return REF_CACHE_WRITE_BLOCKED;
        }
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't write to log: %s\n".as_ptr(),
            libc::strerror(errno),
        );
        return REF_CACHE_WRITE_ERROR;
    }

    (*log_buf).out += bytes as usize;
    assert!((*log_buf).out < REF_CACHE_LOG_BUF_SZ);
    if (bytes as usize) < len {
        REF_CACHE_WRITE_BLOCKED
    } else {
        REF_CACHE_WRITE_MORE
    }
}

// original: handle_client_events (htslib/ref_cache/server.c:509)
pub unsafe fn ref_cache_server_c_509_handle_client_events(
    clients: *mut RefCacheClientsLayout,
    opts: *const structs::Options,
    pw: *mut structs::Poll_wrap,
    item: *mut structs::Pw_item,
    evts: c_uint,
) {
    let opts_l = opts.cast::<RefCacheOptionsLayout>();
    let item_l = item.cast::<RefCachePwItemLayout>();
    let client = (*item_l).userp.cast::<structs::Client>();
    assert!(!client.is_null());
    let client_l = client.cast::<ClientLayout>();
    let polled_l = (*client_l).polled.cast::<RefCachePwItemLayout>();

    if (*opts_l).verbosity > 2 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Events %04x for fd #%d\n".as_ptr(),
            evts,
            (*polled_l).fd,
        );
    }
    if (evts & libc::EPOLLHUP as c_uint) != 0 {
        if (*opts_l).verbosity > 1 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Got hangup on fd#%d\n".as_ptr(),
                (*polled_l).fd,
            );
        }
        ref_cache_server_c_194_close_client(clients, client, pw);
        return;
    } else if (evts & (libc::EPOLLIN | libc::EPOLLERR) as c_uint) != 0 {
        if ((*client_l).flags & REF_CACHE_READ_CLOSED) == 0 {
            ref_cache_server_c_115_read_stack_push(client, &mut (*clients).can_read);
        } else {
            assert!(((*client_l).flags & REF_CACHE_ON_READ_LIST) == 0);
        }
    }

    if (evts & libc::EPOLLOUT as c_uint) != 0 {
        (*client_l).flags |= REF_CACHE_CAN_WRITE;
        if ((*client_l).flags & REF_CACHE_ON_WRITE_LIST) == 0
            && !(*client_l).transact.is_null()
            && ref_cache_transaction_c_619_transaction_have_content((*client_l).transact) != 0
        {
            ref_cache_server_c_151_write_stack_push(client, &mut (*clients).can_write);
        }
    }
}

// original: do_reads (htslib/ref_cache/server.c:565)
pub unsafe fn ref_cache_server_c_565_do_reads(
    clients: *mut RefCacheClientsLayout,
    opts: *const structs::Options,
    pw: *mut structs::Poll_wrap,
) {
    let opts_l = opts.cast::<RefCacheOptionsLayout>();
    let mut can_read: *mut structs::Client = std::ptr::null_mut();

    loop {
        let client = ref_cache_server_c_125_read_stack_pop(clients);
        if client.is_null() {
            break;
        }
        let client_l = client.cast::<ClientLayout>();
        let fd = (*((*client_l).polled.cast::<RefCachePwItemLayout>())).fd;

        if (*opts_l).verbosity > 2 {
            libc::fprintf(hts_sys::stderr.cast(), c"Reading fd #%d\n".as_ptr(), fd);
        }

        let res = if ((*client_l).flags & REF_CACHE_READ_DRAIN) == 0 {
            let res = ref_cache_http_parser_c_601_parser_read_data(
                opts,
                client,
                (&mut (*client_l).parser as *mut HttpParserLayout).cast(),
                fd,
            );
            if (*opts_l).verbosity > 2 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"parser_read_data returned %d\n".as_ptr(),
                    res,
                );
            }
            if ((*client_l).flags & (REF_CACHE_CAN_WRITE | REF_CACHE_ON_WRITE_LIST))
                == REF_CACHE_CAN_WRITE
                && !(*client_l).transact.is_null()
                && ref_cache_transaction_c_619_transaction_have_content((*client_l).transact) != 0
            {
                ref_cache_server_c_151_write_stack_push(client, &mut (*clients).can_write);
            }
            res
        } else {
            let res = ref_cache_server_c_422_eat_input(fd);
            if (*opts_l).verbosity > 2 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"eat_input returned %d\n".as_ptr(),
                    res,
                );
            }
            res
        };

        match res {
            REF_CACHE_READ_MORE => {
                if (*opts_l).verbosity > 2 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"fd #%d has more data...\n".as_ptr(),
                        fd,
                    );
                }
                ref_cache_server_c_115_read_stack_push(client, &mut can_read);
            }
            REF_CACHE_READ_EOF => {
                (*client_l).flags |= REF_CACHE_READ_CLOSED;
                if (((*client_l).flags & (REF_CACHE_READ_CLOSED | REF_CACHE_READ_DRAIN)) != 0)
                    && ((*client_l).transact.is_null()
                        || ref_cache_transaction_c_619_transaction_have_content(
                            (*client_l).transact,
                        ) == 0)
                {
                    if (*opts_l).verbosity > 1 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Closing fd #%d (EOF)\n".as_ptr(),
                            fd,
                        );
                    }
                    assert!(((*client_l).flags & REF_CACHE_ON_READ_LIST) == 0);
                    ref_cache_server_c_194_close_client(clients, client, pw);
                }
            }
            REF_CACHE_READ_BLOCKED => {
                if (((*client_l).flags & (REF_CACHE_READ_CLOSED | REF_CACHE_READ_DRAIN)) != 0)
                    && ((*client_l).transact.is_null()
                        || ref_cache_transaction_c_619_transaction_have_content(
                            (*client_l).transact,
                        ) == 0)
                {
                    if (*opts_l).verbosity > 1 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Closing fd #%d (EOF)\n".as_ptr(),
                            fd,
                        );
                    }
                    assert!(((*client_l).flags & REF_CACHE_ON_READ_LIST) == 0);
                    ref_cache_server_c_194_close_client(clients, client, pw);
                }
            }
            REF_CACHE_READ_ERROR => {
                if (*opts_l).verbosity > 1 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Closing fd #%d (read error)\n".as_ptr(),
                        fd,
                    );
                }
                assert!(((*client_l).flags & REF_CACHE_ON_READ_LIST) == 0);
                ref_cache_server_c_194_close_client(clients, client, pw);
            }
            _ => {}
        }
    }
    (*clients).can_read = can_read;
}

// original: do_writes (htslib/ref_cache/server.c:642)
pub unsafe fn ref_cache_server_c_642_do_writes(
    clients: *mut RefCacheClientsLayout,
    opts: *const structs::Options,
    pw: *mut structs::Poll_wrap,
    log_buf: *mut structs::Log_buffer,
) {
    let opts_l = opts.cast::<RefCacheOptionsLayout>();
    let mut can_write: *mut structs::Client = std::ptr::null_mut();

    loop {
        let client = ref_cache_server_c_161_write_stack_pop(clients);
        if client.is_null() {
            break;
        }
        let client_l = client.cast::<ClientLayout>();
        let fd = (*((*client_l).polled.cast::<RefCachePwItemLayout>())).fd;

        if (*opts_l).verbosity > 1 {
            libc::fprintf(hts_sys::stderr.cast(), c"Writing to fd #%d\n".as_ptr(), fd);
        }
        let res = ref_cache_transaction_c_556_transaction_send_data((*client_l).transact, fd);
        if (*opts_l).verbosity > 2 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"transact_send_data returned %d\n".as_ptr(),
                res,
            );
        }

        match res {
            REF_CACHE_WRITE_BLOCKED => {
                (*client_l).flags &= !REF_CACHE_CAN_WRITE;
            }
            REF_CACHE_WRITE_BLOCKED_UPSTREAM => {}
            REF_CACHE_WRITE_COMPLETE => {
                ref_cache_server_c_436_queue_log_msg(log_buf, (*client_l).transact);
                if ref_cache_transaction_c_214_transaction_get_keep_alive((*client_l).transact) != 0
                {
                    (*client_l).transact = ref_cache_transaction_c_204_switch_to_next_transaction(
                        (*client_l).transact,
                    );
                } else {
                    ref_cache_transaction_c_196_free_transaction_list((*client_l).transact);
                    (*client_l).transact = std::ptr::null_mut();
                    (*client_l).flags |= REF_CACHE_READ_DRAIN;
                }

                if (*client_l).transact.is_null() {
                    (*client_l).last_transact = std::ptr::null_mut();
                    if ((*client_l).flags & (REF_CACHE_READ_CLOSED | REF_CACHE_READ_DRAIN)) != 0 {
                        if (*opts_l).verbosity > 2 {
                            libc::fprintf(
                                hts_sys::stderr.cast(),
                                c"Shutting down fd #%d\n".as_ptr(),
                                fd,
                            );
                        }
                        if libc::shutdown(fd, libc::SHUT_WR) == -1 {
                            libc::fprintf(
                                hts_sys::stderr.cast(),
                                c"Error from shutdown(%d, SHUT_WR) : %s\n".as_ptr(),
                                fd,
                                libc::strerror(*crate::htslib_mini_rs::c_compat::__errno_location()),
                            );
                            ref_cache_server_c_194_close_client(clients, client, pw);
                        }
                    }
                } else if ref_cache_transaction_c_619_transaction_have_content((*client_l).transact)
                    != 0
                {
                    if (*opts_l).verbosity > 2 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"fd #%d can send more data...\n".as_ptr(),
                            fd,
                        );
                    }
                    ref_cache_server_c_151_write_stack_push(client, &mut can_write);
                }
            }
            REF_CACHE_WRITE_MORE => {
                if (*opts_l).verbosity > 2 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"fd #%d can send more data...\n".as_ptr(),
                        fd,
                    );
                }
                ref_cache_server_c_151_write_stack_push(client, &mut can_write);
            }
            REF_CACHE_WRITE_EOF | REF_CACHE_WRITE_ERROR => {
                if (*opts_l).verbosity > 1 {
                    libc::fprintf(hts_sys::stderr.cast(), c"Closing fd #%d\n".as_ptr(), fd);
                }
                ref_cache_server_c_194_close_client(clients, client, pw);
            }
            _ => {}
        }
    }
    (*clients).can_write = can_write;
}

// original: run_poll_loop (htslib/ref_cache/server.c:721)
pub unsafe fn ref_cache_server_c_721_run_poll_loop(
    opts: *const structs::Options,
    lsocks: *mut structs::Listeners,
    upstream: c_int,
    log_fd: c_int,
) -> c_int {
    let opts_l = opts.cast::<RefCacheOptionsLayout>();
    let mut clients: RefCacheClientsLayout = std::mem::zeroed();
    let mut polled_upstream: *mut structs::Pw_item = std::ptr::null_mut();
    let mut polled_log: *mut structs::Pw_item;
    let mut events = [std::mem::zeroed::<libc::epoll_event>(); REF_CACHE_MAX_EVENTS as usize];
    let mut log_buf: RefCacheLogBufferLayout = std::mem::zeroed();
    let mut log_can_write = 0;
    let mut running = 1;

    log_buf.in_ = 0;
    log_buf.out = 0;

    if ref_cache_misc_h_40_setnonblock(log_fd) != 0 {
        libc::perror(c"Setting nonblocking on log pipe".as_ptr());
        return -1;
    }

    if ref_cache_server_c_107_init_clients(&mut clients) != 0 {
        libc::perror(c"Initializing clients array".as_ptr());
        return -1;
    }

    let pw = ref_cache_poll_wrap_epoll_c_49_pw_init(((*opts_l).verbosity > 2) as c_int);
    if pw.is_null() {
        libc::perror(c"Initializing poller".as_ptr());
        return -1;
    }

    let polled_listeners =
        super::listener::ref_cache_listener_c_242_register_listener_pollers(lsocks.cast(), pw);
    if polled_listeners.is_null() {
        return -1;
    }

    if upstream != -1 {
        polled_upstream = ref_cache_poll_wrap_epoll_c_78_pw_register(
            pw,
            upstream,
            REF_CACHE_SV_UPSTREAM,
            (libc::EPOLLIN | libc::EPOLLERR | libc::EPOLLHUP) as u32,
            std::ptr::null_mut::<c_void>(),
        );
        if polled_upstream.is_null() {
            libc::perror(c"Adding upstream socket to poller".as_ptr());
            return -1;
        }
    }

    polled_log = ref_cache_poll_wrap_epoll_c_78_pw_register(
        pw,
        log_fd,
        REF_CACHE_SV_LOG,
        (libc::EPOLLOUT | libc::EPOLLERR | libc::EPOLLHUP) as u32,
        std::ptr::null_mut::<c_void>(),
    );
    if polled_log.is_null() {
        libc::perror(c"Adding log pipe to poller".as_ptr());
        return -1;
    }

    while running != 0 || clients.count > 0 {
        let nevts = ref_cache_poll_wrap_epoll_c_120_pw_wait(
            pw,
            events.as_mut_ptr().cast(),
            REF_CACHE_MAX_EVENTS,
            if clients.can_read.is_null() && clients.can_write.is_null() {
                -1
            } else {
                0
            },
        );
        if nevts == -1 {
            if *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EINTR {
                continue;
            }
            libc::perror(c"Error from poller".as_ptr());
            return -1;
        }

        let mut e = 0;
        while e < nevts {
            let evts = events[e as usize].events;
            let item = events[e as usize].u64 as *mut structs::Pw_item;
            assert!(!item.is_null());
            let item_l = item.cast::<RefCachePwItemLayout>();

            match (*item_l).fd_type {
                REF_CACHE_SV_LISTENER => {
                    if running != 0 {
                        ref_cache_server_c_268_handle_incoming(
                            opts,
                            (*item_l).fd,
                            upstream,
                            pw,
                            &mut clients,
                        );
                    }
                }
                REF_CACHE_SV_UPSTREAM => {
                    ref_cache_server_c_362_handle_upstream(
                        upstream,
                        &mut clients,
                        (*opts_l).verbosity as c_int,
                    );
                }
                REF_CACHE_SV_LOG => {
                    if ref_cache_poll_wrap_epoll_c_106_pw_mod(
                        pw,
                        item,
                        (libc::EPOLLERR | libc::EPOLLHUP) as u32,
                    ) != 0
                    {
                        libc::perror(c"Disarming log poller".as_ptr());
                        return -1;
                    }
                    if (evts & (libc::EPOLLHUP | libc::EPOLLERR) as u32) != 0 {
                        running = 0;
                    } else {
                        log_can_write = 1;
                    }
                }
                REF_CACHE_SV_CLIENT => {
                    ref_cache_server_c_509_handle_client_events(&mut clients, opts, pw, item, evts);
                }
                _ => {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Unexpected polled item type %d\n".as_ptr(),
                        (*item_l).fd_type,
                    );
                    return -1;
                }
            }
            e += 1;
        }

        ref_cache_server_c_565_do_reads(&mut clients, opts, pw);
        ref_cache_server_c_642_do_writes(
            &mut clients,
            opts,
            pw,
            (&mut log_buf as *mut RefCacheLogBufferLayout).cast(),
        );

        if log_can_write != 0 && log_buf.in_ != log_buf.out {
            let res = ref_cache_server_c_470_send_to_log(
                (&mut log_buf as *mut RefCacheLogBufferLayout).cast(),
                log_fd,
            );
            if res == REF_CACHE_WRITE_ERROR {
                return -1;
            }
            if res == REF_CACHE_WRITE_BLOCKED {
                if ref_cache_poll_wrap_epoll_c_106_pw_mod(
                    pw,
                    polled_log,
                    (libc::EPOLLOUT | libc::EPOLLERR | libc::EPOLLHUP) as u32,
                ) != 0
                {
                    libc::perror(c"Re-arming log poller".as_ptr());
                    return -1;
                }
                log_can_write = 0;
            }
        }
    }
    0
}

// original: client_host (htslib/ref_cache/server.c:840)
pub unsafe fn ref_cache_server_c_840_client_host(client: *mut structs::Client) -> *const c_char {
    let host = (*(client.cast::<ClientLayout>())).host;
    if host.is_null() {
        c"<NULL>".as_ptr()
    } else {
        host
    }
}
