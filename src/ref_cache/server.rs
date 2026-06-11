use super::http_parser::{
    ref_cache_http_parser_c_111_init_http_parser, ref_cache_http_parser_c_131_cleanup_http_parser,
    ref_cache_http_parser_c_601_parser_read_data, HttpParser,
};
use super::listener::Listeners;
use super::misc::ref_cache_misc_h_40_setnonblock;
use super::options::Options;
use super::poll_wrap::Pw_fd_type;
use super::poll_wrap_epoll::{
    ref_cache_poll_wrap_epoll_c_106_pw_mod, ref_cache_poll_wrap_epoll_c_120_pw_wait,
    ref_cache_poll_wrap_epoll_c_126_pw_remove, ref_cache_poll_wrap_epoll_c_49_pw_init,
    ref_cache_poll_wrap_epoll_c_78_pw_register, Poll_wrap,
};
use super::transaction::{
    ref_cache_transaction_c_196_free_transaction_list,
    ref_cache_transaction_c_204_switch_to_next_transaction,
    ref_cache_transaction_c_210_transaction_get_client,
    ref_cache_transaction_c_214_transaction_get_keep_alive,
    ref_cache_transaction_c_556_transaction_send_data,
    ref_cache_transaction_c_619_transaction_have_content,
    ref_cache_transaction_c_623_transaction_set_next,
    ref_cache_transaction_c_628_transaction_has_data_to_send,
    ref_cache_transaction_c_680_got_download_started,
    ref_cache_transaction_c_698_got_download_part, ref_cache_transaction_c_715_got_download_clen,
    ref_cache_transaction_c_730_got_download_result, ref_cache_transaction_c_769_make_log_message,
    TransactionId,
};
use super::upstream::{
    ref_cache_upstream_c_215_upstream_recv_msg, Upstream_msg, Upstream_msg_code,
};
use std::ffi::{c_int, c_uchar, c_uint};

const REF_CACHE_ON_READ_LIST: c_uint = 0x01;
const REF_CACHE_ON_WRITE_LIST: c_uint = 0x02;
const REF_CACHE_READ_CLOSED: c_uint = 0x04;
const REF_CACHE_READ_DRAIN: c_uint = 0x08;
const REF_CACHE_CAN_WRITE: c_uint = 0x10;

const REF_CACHE_SV_LISTENER: Pw_fd_type = Pw_fd_type::SV_LISTENER;
const REF_CACHE_SV_UPSTREAM: Pw_fd_type = Pw_fd_type::SV_UPSTREAM;
const REF_CACHE_SV_LOG: Pw_fd_type = Pw_fd_type::SV_LOG;
const REF_CACHE_SV_CLIENT: Pw_fd_type = Pw_fd_type::SV_CLIENT;

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

// A sentinel meaning "no client" for the index-based intrusive lists that
// replace the original C raw-pointer doubly-linked lists.
const NO_CLIENT: usize = usize::MAX;

// original: Client (htslib/ref_cache/server.c:71)
//
// The original C `Client` was an intrusive node in two doubly-linked lists
// (read + write) using raw `prev`/`next` pointers, allocated from an arena
// pool. We restructure it into an owned arena entry held in a `Vec`, where the
// links are `usize` indices into that arena (NO_CLIENT == null). `host` and the
// HTTP parser are now owned by value, and the request pipeline (`transact` /
// `last_transact`) is referenced by `TransactionId` (an index into the
// transaction arena) rather than by `*mut Transaction`.
pub struct Client {
    // None until the client is registered with the poller; the index is into
    // the poller's own arena.
    polled: Option<usize>,
    // The client's socket fd, cached here so do_reads/do_writes don't have to
    // round-trip through the poller arena to recover it.
    fd: c_int,
    prev: usize,
    next: usize,
    prev_write: usize,
    next_write: usize,
    host: Vec<u8>,
    transact: Option<TransactionId>,
    last_transact: Option<TransactionId>,
    parser: HttpParser,
    flags: c_uint,
    // True when this arena slot is occupied by a live client.
    in_use: bool,
}

impl Client {
    fn empty() -> Self {
        Self {
            polled: None,
            fd: -1,
            prev: NO_CLIENT,
            next: NO_CLIENT,
            prev_write: NO_CLIENT,
            next_write: NO_CLIENT,
            host: Vec::new(),
            transact: None,
            last_transact: None,
            parser: ref_cache_http_parser_c_111_init_http_parser(-1),
            flags: 0,
            in_use: false,
        }
    }
}

// original: clients container (the C arena pool plus the read/write list heads)
pub struct RefCacheClientsLayout {
    // Owned arena of clients; indices into this Vec replace raw pointers.
    arena: Vec<Client>,
    // Free arena slots available for reuse.
    free_slots: Vec<usize>,
    can_read: usize,
    can_write: usize,
    count: usize,
}

impl Default for RefCacheClientsLayout {
    fn default() -> Self {
        Self {
            arena: Vec::new(),
            free_slots: Vec::new(),
            can_read: NO_CLIENT,
            can_write: NO_CLIENT,
            count: 0,
        }
    }
}

pub struct RefCacheLogBufferLayout {
    buffer: Vec<u8>,
    in_: usize,
    out: usize,
}

impl Default for RefCacheLogBufferLayout {
    fn default() -> Self {
        Self {
            buffer: vec![0u8; REF_CACHE_LOG_BUF_SZ],
            in_: 0,
            out: 0,
        }
    }
}

// original: Log_buffer (htslib/ref_cache/server.c:99)
pub type Log_buffer = RefCacheLogBufferLayout;

// original: SocketAddress (htslib/ref_cache/server.c:100)
#[repr(C)]
pub struct SocketAddress {
    _private: [u8; 0],
}

// original: init_clients (htslib/ref_cache/server.c:107)
pub fn ref_cache_server_c_107_init_clients(clients: &mut RefCacheClientsLayout) -> c_int {
    clients.arena = Vec::new();
    clients.free_slots = Vec::new();
    clients.can_read = NO_CLIENT;
    clients.can_write = NO_CLIENT;
    clients.count = 0;
    0
}

// original: read_stack_push (htslib/ref_cache/server.c:115)
pub fn ref_cache_server_c_115_read_stack_push(
    clients: &mut RefCacheClientsLayout,
    client: usize,
    stack: &mut usize,
) {
    if (clients.arena[client].flags & REF_CACHE_ON_READ_LIST) != 0 {
        return;
    }
    clients.arena[client].flags |= REF_CACHE_ON_READ_LIST;
    if *stack != NO_CLIENT {
        clients.arena[*stack].prev = client;
    }
    clients.arena[client].next = *stack;
    clients.arena[client].prev = NO_CLIENT;
    *stack = client;
}

// original: read_stack_pop (htslib/ref_cache/server.c:125)
pub fn ref_cache_server_c_125_read_stack_pop(clients: &mut RefCacheClientsLayout) -> usize {
    let can_read = clients.can_read;
    if can_read == NO_CLIENT {
        return NO_CLIENT;
    }
    clients.can_read = clients.arena[can_read].next;
    if clients.can_read != NO_CLIENT {
        let next = clients.can_read;
        clients.arena[next].prev = NO_CLIENT;
    }
    clients.arena[can_read].prev = NO_CLIENT;
    clients.arena[can_read].next = NO_CLIENT;
    assert!((clients.arena[can_read].flags & REF_CACHE_ON_READ_LIST) != 0);
    clients.arena[can_read].flags &= !REF_CACHE_ON_READ_LIST;
    can_read
}

// original: read_stack_remove (htslib/ref_cache/server.c:137)
pub fn ref_cache_server_c_137_read_stack_remove(
    clients: &mut RefCacheClientsLayout,
    client: usize,
) {
    clients.arena[client].flags &= !REF_CACHE_ON_READ_LIST;
    if clients.can_read == client {
        clients.can_read = clients.arena[client].next;
        if clients.can_read != NO_CLIENT {
            let next = clients.can_read;
            clients.arena[next].prev = NO_CLIENT;
        }
    } else {
        let prev = clients.arena[client].prev;
        let next = clients.arena[client].next;
        clients.arena[prev].next = next;
        if next != NO_CLIENT {
            clients.arena[next].prev = prev;
        }
    }
    clients.arena[client].prev = NO_CLIENT;
    clients.arena[client].next = NO_CLIENT;
}

// original: write_stack_push (htslib/ref_cache/server.c:151)
pub fn ref_cache_server_c_151_write_stack_push(
    clients: &mut RefCacheClientsLayout,
    client: usize,
    stack: &mut usize,
) {
    if (clients.arena[client].flags & REF_CACHE_ON_WRITE_LIST) != 0 {
        return;
    }
    clients.arena[client].flags |= REF_CACHE_ON_WRITE_LIST;
    if *stack != NO_CLIENT {
        clients.arena[*stack].prev_write = client;
    }
    clients.arena[client].next_write = *stack;
    clients.arena[client].prev_write = NO_CLIENT;
    *stack = client;
}

// original: write_stack_pop (htslib/ref_cache/server.c:161)
pub fn ref_cache_server_c_161_write_stack_pop(clients: &mut RefCacheClientsLayout) -> usize {
    let can_write = clients.can_write;
    if can_write == NO_CLIENT {
        return NO_CLIENT;
    }
    clients.can_write = clients.arena[can_write].next_write;
    if clients.can_write != NO_CLIENT {
        let next = clients.can_write;
        clients.arena[next].prev_write = NO_CLIENT;
    }
    clients.arena[can_write].prev_write = NO_CLIENT;
    clients.arena[can_write].next_write = NO_CLIENT;
    assert!((clients.arena[can_write].flags & REF_CACHE_ON_WRITE_LIST) != 0);
    clients.arena[can_write].flags &= !REF_CACHE_ON_WRITE_LIST;
    can_write
}

// original: write_stack_remove (htslib/ref_cache/server.c:173)
pub fn ref_cache_server_c_173_write_stack_remove(
    clients: &mut RefCacheClientsLayout,
    client: usize,
) {
    clients.arena[client].flags &= !REF_CACHE_ON_WRITE_LIST;
    if clients.can_write == client {
        clients.can_write = clients.arena[client].next_write;
        if clients.can_write != NO_CLIENT {
            let next = clients.can_write;
            clients.arena[next].prev_write = NO_CLIENT;
        }
    } else {
        let prev = clients.arena[client].prev_write;
        let next = clients.arena[client].next_write;
        clients.arena[prev].next_write = next;
        if next != NO_CLIENT {
            clients.arena[next].prev_write = prev;
        }
    }
    clients.arena[client].prev_write = NO_CLIENT;
    clients.arena[client].next_write = NO_CLIENT;
}

// original: get_free_client (htslib/ref_cache/server.c:187)
pub fn ref_cache_server_c_187_get_free_client(clients: &mut RefCacheClientsLayout) -> usize {
    let idx = if let Some(slot) = clients.free_slots.pop() {
        clients.arena[slot] = Client::empty();
        slot
    } else {
        clients.arena.push(Client::empty());
        clients.arena.len() - 1
    };
    clients.arena[idx].in_use = true;
    clients.count += 1;
    idx
}

// original: close_client (htslib/ref_cache/server.c:194)
pub unsafe fn ref_cache_server_c_194_close_client(
    clients: &mut RefCacheClientsLayout,
    client: usize,
    pw: &mut Poll_wrap,
) {
    assert!(clients.count > 0);
    if let Some(polled) = clients.arena[client].polled.take() {
        ref_cache_poll_wrap_epoll_c_126_pw_remove(pw, polled, true);
    }

    if (clients.arena[client].flags & REF_CACHE_ON_READ_LIST) != 0 {
        ref_cache_server_c_137_read_stack_remove(clients, client);
    }

    if (clients.arena[client].flags & REF_CACHE_ON_WRITE_LIST) != 0 {
        ref_cache_server_c_173_write_stack_remove(clients, client);
    }

    ref_cache_http_parser_c_131_cleanup_http_parser(&mut clients.arena[client].parser);

    ref_cache_transaction_c_196_free_transaction_list(clients.arena[client].transact);

    // Returning the slot to the arena and dropping its owned contents replaces
    // the C arena `pool_free`; Drop handles `host` and the parser buffers.
    clients.arena[client] = Client::empty();
    clients.free_slots.push(client);
    clients.count -= 1;
}

// original: check_addr_allowed (htslib/ref_cache/server.c:222)
pub unsafe fn ref_cache_server_c_222_check_addr_allowed(
    opts: &Options,
    mut family: libc::sa_family_t,
    addr: &SocketAddress,
    addrlen: libc::socklen_t,
) -> c_int {
    let start;
    let end;
    let mut addrp: *const c_uchar;

    if opts.match_addrs_storage.is_empty() {
        return 0;
    }

    let addr = addr as *const SocketAddress;
    if family == libc::AF_INET as libc::sa_family_t
        && addrlen as usize == std::mem::size_of::<libc::sockaddr_in>()
    {
        let in4 = addr.cast::<libc::sockaddr_in>();
        addrp = (&(*in4).sin_addr as *const libc::in_addr).cast();
        start = 0usize;
        end = opts.first_ip6;
    } else if family == libc::AF_INET6 as libc::sa_family_t
        && addrlen as usize == std::mem::size_of::<libc::sockaddr_in6>()
    {
        let v4mapped_prefix: [c_uchar; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff];
        let in6 = addr.cast::<libc::sockaddr_in6>();
        addrp = (&(*in6).sin6_addr as *const libc::in6_addr).cast();
        let prefix = std::slice::from_raw_parts(addrp, v4mapped_prefix.len());
        if prefix == v4mapped_prefix {
            family = libc::AF_INET as libc::sa_family_t;
            addrp = addrp.add(12);
            start = 0;
            end = opts.first_ip6;
        } else {
            start = opts.first_ip6;
            end = opts.match_addrs_storage.len();
        }
    } else {
        return -1;
    }

    for ma in &opts.match_addrs_storage[start..end] {
        if family != ma.family {
            continue;
        }
        let mask_bytes = ma.mask_bytes as usize;
        let addr_prefix = std::slice::from_raw_parts(addrp, mask_bytes);
        if addr_prefix == &ma.addr[..mask_bytes]
            && (*addrp.add(mask_bytes) & ma.mask) == (ma.addr[mask_bytes] & ma.mask)
        {
            return 0;
        }
    }
    -1
}

// original: handle_incoming (htslib/ref_cache/server.c:268)
pub unsafe fn ref_cache_server_c_268_handle_incoming(
    opts: &Options,
    listener: c_int,
    upstream: c_int,
    pw: &mut Poll_wrap,
    clients: &mut RefCacheClientsLayout,
) {
    let mut addr: libc::sockaddr_storage = std::mem::zeroed();
    let mut addrlen = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    // Host name buffer; resolved name lands here as raw bytes (no NUL handling).
    let mut host_buf = [0u8; REF_CACHE_NI_MAXHOST];

    loop {
        // Genuine syscall: accept(2). Buffers around it are owned/sliced.
        let incoming = libc::accept(
            listener,
            (&mut addr as *mut libc::sockaddr_storage).cast(),
            &mut addrlen,
        );
        if incoming < 0 {
            break;
        }

        // Genuine syscall: getnameinfo(3). It writes into our owned buffer.
        let res = libc::getnameinfo(
            (&addr as *const libc::sockaddr_storage).cast(),
            addrlen,
            host_buf.as_mut_ptr().cast(),
            host_buf.len() as libc::socklen_t,
            std::ptr::null_mut(),
            0,
            libc::NI_NUMERICHOST,
        );
        // Resolved host as a borrowed byte slice (up to first NUL).
        let host_len = host_buf.iter().position(|&b| b == 0).unwrap_or(host_buf.len());
        let mut host: Vec<u8> = host_buf[..host_len].to_vec();
        if res != 0 {
            let msg = std::ffi::CStr::from_ptr(libc::gai_strerror(res)).to_string_lossy();
            eprintln!("Couldn't resolve host for incoming client: {}", msg);
            host = b"<unknown>".to_vec();
        }

        if ref_cache_server_c_222_check_addr_allowed(
            opts,
            addr.ss_family,
            &*(&addr as *const libc::sockaddr_storage as *const SocketAddress),
            addrlen,
        ) != 0
        {
            if opts.verbosity > 1 {
                eprintln!(
                    "Rejected incoming client from {}",
                    String::from_utf8_lossy(&host)
                );
            }
            libc::close(incoming);
            continue;
        }

        if opts.verbosity > 1 {
            eprintln!(
                "Accepted incoming client from {} on fd #{}",
                String::from_utf8_lossy(&host),
                incoming
            );
        }

        if ref_cache_misc_h_40_setnonblock(incoming) != 0 {
            libc::close(incoming);
            return;
        }

        let client = ref_cache_server_c_187_get_free_client(clients);
        // get_free_client always yields a slot now; no null path.

        let events =
            (libc::EPOLLIN | libc::EPOLLOUT | libc::EPOLLERR | libc::EPOLLHUP | libc::EPOLLET)
                as u32;
        // The parser is freshly initialised when the arena slot is taken
        // (Client::empty); re-init it here with the correct upstream fd.
        clients.arena[client].parser = ref_cache_http_parser_c_111_init_http_parser(upstream);
        let polled = ref_cache_poll_wrap_epoll_c_78_pw_register(
            pw,
            incoming,
            REF_CACHE_SV_CLIENT,
            events,
            client,
        );
        let Some(polled) = polled else {
            eprintln!("Registering client with poll: {}", std::io::Error::last_os_error());
            ref_cache_server_c_194_close_client(clients, client, pw);
            return;
        };
        clients.arena[client].polled = Some(polled);
        clients.arena[client].fd = incoming;
        clients.arena[client].flags = 0;
        clients.arena[client].host = host;
        clients.arena[client].transact = None;
        clients.arena[client].last_transact = None;
    }

    match std::io::Error::last_os_error().raw_os_error().unwrap_or(0) {
        libc::EAGAIN
        | libc::EINTR
        | libc::ENETDOWN
        | libc::EPROTO
        | libc::ENOPROTOOPT
        | libc::EHOSTUNREACH
        | libc::EOPNOTSUPP
        | libc::ENETUNREACH => {}
        _ => eprintln!(
            "Error while accepting incoming client: {}",
            std::io::Error::last_os_error()
        ),
    }
}

// original: client_add_transaction (htslib/ref_cache/server.c:352)
pub unsafe fn ref_cache_server_c_352_client_add_transaction(
    clients: &mut RefCacheClientsLayout,
    client: usize,
    transact: TransactionId,
) {
    if clients.arena[client].transact.is_none() {
        assert!(clients.arena[client].last_transact.is_none());
        clients.arena[client].transact = Some(transact);
        clients.arena[client].last_transact = Some(transact);
    } else {
        let last = clients.arena[client]
            .last_transact
            .expect("last_transact set when transact set");
        ref_cache_transaction_c_623_transaction_set_next(last, transact);
        clients.arena[client].last_transact = Some(transact);
    }
}

// original: handle_upstream (htslib/ref_cache/server.c:362)
pub unsafe fn ref_cache_server_c_362_handle_upstream(
    upstream: c_int,
    clients: &mut RefCacheClientsLayout,
    verbosity: c_int,
) {
    let mut msg = Upstream_msg {
        id: 0,
        code: Upstream_msg_code::US_START,
        val: 0,
    };
    let mut fd = -1;

    let res = ref_cache_upstream_c_215_upstream_recv_msg(upstream, &mut msg, &mut fd);
    if res <= 0 {
        return;
    }

    if verbosity > 2 {
        eprintln!(
            "Message from upstream: {}, {:?}, {}",
            msg.id, msg.code, msg.val
        );
    }

    // The download-* handlers manipulate the write stack head; move it into a
    // local so the handlers can take `&mut clients` and a separate `&mut head`
    // without aliasing, then write it back.
    let mut can_write = clients.can_write;
    match msg.code {
        Upstream_msg_code::US_START => {
            ref_cache_transaction_c_680_got_download_started(
                clients,
                msg.id,
                msg.val,
                fd,
                &mut can_write,
            );
        }
        Upstream_msg_code::US_PARTIAL_LENGTH => {
            ref_cache_transaction_c_698_got_download_part(clients, msg.id, msg.val, &mut can_write);
        }
        Upstream_msg_code::US_CONTENT_LENGTH => {
            ref_cache_transaction_c_715_got_download_clen(clients, msg.id, msg.val, &mut can_write);
        }
        Upstream_msg_code::US_RESULT => {
            ref_cache_transaction_c_730_got_download_result(clients, msg.id, msg.val, &mut can_write);
        }
    }
    clients.can_write = can_write;
}

// original: queue_transaction_write (htslib/ref_cache/server.c:395)
pub unsafe fn ref_cache_server_c_395_queue_transaction_write(
    clients: &mut RefCacheClientsLayout,
    transact: TransactionId,
    write_stack: &mut usize,
) {
    let Some(client) = ref_cache_transaction_c_210_transaction_get_client(transact) else {
        return;
    };

    if (clients.arena[client].flags & REF_CACHE_ON_WRITE_LIST) != 0 {
        return;
    }
    if clients.arena[client].transact != Some(transact) {
        return;
    }
    if ref_cache_transaction_c_628_transaction_has_data_to_send(transact) == 0 {
        return;
    }
    ref_cache_server_c_151_write_stack_push(clients, client, write_stack);
}

// original: eat_input (htslib/ref_cache/server.c:422)
pub unsafe fn ref_cache_server_c_422_eat_input(fd: c_int) -> c_int {
    let mut buffer = vec![0u8; 65536];
    let mut bytes;

    loop {
        // Genuine syscall: read(2) into an owned buffer.
        bytes = libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len());
        if !(bytes < 0
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR))
        {
            break;
        }
    }
    if bytes < 0 {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
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
    log_buf: &mut RefCacheLogBufferLayout,
    client_host: &[u8],
    transact: TransactionId,
) {
    // make_log_message now returns the formatted line as owned bytes; the client
    // hostname is resolved by the caller (which owns the client arena).
    let buf = ref_cache_transaction_c_769_make_log_message(transact, client_host);
    let mut bytes = buf.len();
    let cap = 128 + REF_CACHE_MAX_UA_LEN + REF_CACHE_MAX_REFERRER_LEN + REF_CACHE_MAX_REQUEST_LEN + 1025;
    debug_assert!(bytes < cap);
    if bytes >= cap {
        bytes = cap - 1;
    }

    let space = if log_buf.in_ >= log_buf.out {
        REF_CACHE_LOG_BUF_SZ - log_buf.in_ + log_buf.out - 1
    } else {
        log_buf.out - log_buf.in_ - 1
    };
    if bytes > space {
        return;
    }

    // Offset into the produced message as we copy it into the ring buffer.
    let mut src_off = 0usize;
    if log_buf.in_ >= log_buf.out {
        let mut l = REF_CACHE_LOG_BUF_SZ - log_buf.in_ - if log_buf.out == 0 { 1 } else { 0 };
        if l > bytes {
            l = bytes;
        }
        let in_ = log_buf.in_;
        log_buf.buffer[in_..in_ + l].copy_from_slice(&buf[src_off..src_off + l]);
        src_off += l;
        bytes -= l;
        log_buf.in_ = (log_buf.in_ + l) & REF_CACHE_LOG_BUF_MASK;
    }
    if bytes > 0 && log_buf.in_ < log_buf.out {
        assert!(bytes < log_buf.out - log_buf.in_ - 1);
        let in_ = log_buf.in_;
        log_buf.buffer[in_..in_ + bytes].copy_from_slice(&buf[src_off..src_off + bytes]);
        log_buf.in_ += bytes;
    }
    assert!(log_buf.in_ < REF_CACHE_LOG_BUF_SZ);
    assert!(log_buf.in_ != log_buf.out);
}

// original: send_to_log (htslib/ref_cache/server.c:470)
pub unsafe fn ref_cache_server_c_470_send_to_log(
    log_buf: &mut RefCacheLogBufferLayout,
    log_fd: c_int,
) -> c_int {
    let mut bytes;

    if log_buf.out == log_buf.in_ {
        return REF_CACHE_WRITE_MORE;
    }
    if log_buf.out > log_buf.in_ {
        let len = REF_CACHE_LOG_BUF_SZ - log_buf.out;
        loop {
            // Genuine syscall: write(2) of an owned-buffer slice.
            bytes = libc::write(log_fd, log_buf.buffer[log_buf.out..].as_ptr().cast(), len);
            if !(bytes < 0
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR))
            {
                break;
            }
        }

        if bytes < 0 {
            let err = std::io::Error::last_os_error();
            let errno = err.raw_os_error().unwrap_or(0);
            if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                return REF_CACHE_WRITE_BLOCKED;
            }
            eprintln!("Couldn't write to log: {}", err);
            return REF_CACHE_WRITE_ERROR;
        }

        log_buf.out = (log_buf.out + bytes as usize) & REF_CACHE_LOG_BUF_MASK;
        if (bytes as usize) < len {
            return REF_CACHE_WRITE_BLOCKED;
        }
    }

    if log_buf.out == log_buf.in_ {
        return REF_CACHE_WRITE_MORE;
    }
    assert!(log_buf.out < log_buf.in_);

    let len = log_buf.in_ - log_buf.out;
    loop {
        bytes = libc::write(log_fd, log_buf.buffer[log_buf.out..].as_ptr().cast(), len);
        if !(bytes < 0
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR))
        {
            break;
        }
    }

    if bytes < 0 {
        let err = std::io::Error::last_os_error();
        let errno = err.raw_os_error().unwrap_or(0);
        if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
            return REF_CACHE_WRITE_BLOCKED;
        }
        eprintln!("Couldn't write to log: {}", err);
        return REF_CACHE_WRITE_ERROR;
    }

    log_buf.out += bytes as usize;
    assert!(log_buf.out < REF_CACHE_LOG_BUF_SZ);
    if (bytes as usize) < len {
        REF_CACHE_WRITE_BLOCKED
    } else {
        REF_CACHE_WRITE_MORE
    }
}

// original: handle_client_events (htslib/ref_cache/server.c:509)
pub unsafe fn ref_cache_server_c_509_handle_client_events(
    clients: &mut RefCacheClientsLayout,
    opts: &Options,
    pw: &mut Poll_wrap,
    client: usize,
    evts: c_uint,
) {
    let fd = clients.arena[client].fd;

    if opts.verbosity > 2 {
        eprintln!("Events {:04x} for fd #{}", evts, fd);
    }
    if (evts & libc::EPOLLHUP as c_uint) != 0 {
        if opts.verbosity > 1 {
            eprintln!("Got hangup on fd#{}", fd);
        }
        ref_cache_server_c_194_close_client(clients, client, pw);
        return;
    } else if (evts & (libc::EPOLLIN | libc::EPOLLERR) as c_uint) != 0 {
        if (clients.arena[client].flags & REF_CACHE_READ_CLOSED) == 0 {
            let mut can_read = clients.can_read;
            ref_cache_server_c_115_read_stack_push(clients, client, &mut can_read);
            clients.can_read = can_read;
        } else {
            assert!((clients.arena[client].flags & REF_CACHE_ON_READ_LIST) == 0);
        }
    }

    if (evts & libc::EPOLLOUT as c_uint) != 0 {
        clients.arena[client].flags |= REF_CACHE_CAN_WRITE;
        if (clients.arena[client].flags & REF_CACHE_ON_WRITE_LIST) == 0 {
            if let Some(transact) = clients.arena[client].transact {
                if ref_cache_transaction_c_619_transaction_have_content(transact) != 0 {
                    let mut can_write = clients.can_write;
                    ref_cache_server_c_151_write_stack_push(clients, client, &mut can_write);
                    clients.can_write = can_write;
                }
            }
        }
    }
}

// original: do_reads (htslib/ref_cache/server.c:565)
pub unsafe fn ref_cache_server_c_565_do_reads(
    clients: &mut RefCacheClientsLayout,
    opts: &Options,
    pw: &mut Poll_wrap,
) {
    let mut can_read: usize = NO_CLIENT;

    loop {
        let client = ref_cache_server_c_125_read_stack_pop(clients);
        if client == NO_CLIENT {
            break;
        }
        let fd = clients.arena[client].fd;

        if opts.verbosity > 2 {
            eprintln!("Reading fd #{}", fd);
        }

        let res = if (clients.arena[client].flags & REF_CACHE_READ_DRAIN) == 0 {
            // Borrow the parser out of the client slot so the parser and the
            // client arena can both be mutated; put it back afterwards.
            let mut parser = std::mem::replace(
                &mut clients.arena[client].parser,
                ref_cache_http_parser_c_111_init_http_parser(-1),
            );
            let res = ref_cache_http_parser_c_601_parser_read_data(
                opts, clients, client, &mut parser, fd,
            );
            clients.arena[client].parser = parser;
            if opts.verbosity > 2 {
                eprintln!("parser_read_data returned {}", res);
            }
            if (clients.arena[client].flags & (REF_CACHE_CAN_WRITE | REF_CACHE_ON_WRITE_LIST))
                == REF_CACHE_CAN_WRITE
            {
                if let Some(transact) = clients.arena[client].transact {
                    if ref_cache_transaction_c_619_transaction_have_content(transact) != 0 {
                        let mut can_write = clients.can_write;
                        ref_cache_server_c_151_write_stack_push(clients, client, &mut can_write);
                        clients.can_write = can_write;
                    }
                }
            }
            res
        } else {
            let res = ref_cache_server_c_422_eat_input(fd);
            if opts.verbosity > 2 {
                eprintln!("eat_input returned {}", res);
            }
            res
        };

        match res {
            REF_CACHE_READ_MORE => {
                if opts.verbosity > 2 {
                    eprintln!("fd #{} has more data...", fd);
                }
                ref_cache_server_c_115_read_stack_push(clients, client, &mut can_read);
            }
            REF_CACHE_READ_EOF => {
                clients.arena[client].flags |= REF_CACHE_READ_CLOSED;
                let no_content = match clients.arena[client].transact {
                    None => true,
                    Some(t) => ref_cache_transaction_c_619_transaction_have_content(t) == 0,
                };
                if ((clients.arena[client].flags & (REF_CACHE_READ_CLOSED | REF_CACHE_READ_DRAIN))
                    != 0)
                    && no_content
                {
                    if opts.verbosity > 1 {
                        eprintln!("Closing fd #{} (EOF)", fd);
                    }
                    assert!((clients.arena[client].flags & REF_CACHE_ON_READ_LIST) == 0);
                    ref_cache_server_c_194_close_client(clients, client, pw);
                }
            }
            REF_CACHE_READ_BLOCKED => {
                let no_content = match clients.arena[client].transact {
                    None => true,
                    Some(t) => ref_cache_transaction_c_619_transaction_have_content(t) == 0,
                };
                if ((clients.arena[client].flags & (REF_CACHE_READ_CLOSED | REF_CACHE_READ_DRAIN))
                    != 0)
                    && no_content
                {
                    if opts.verbosity > 1 {
                        eprintln!("Closing fd #{} (EOF)", fd);
                    }
                    assert!((clients.arena[client].flags & REF_CACHE_ON_READ_LIST) == 0);
                    ref_cache_server_c_194_close_client(clients, client, pw);
                }
            }
            REF_CACHE_READ_ERROR => {
                if opts.verbosity > 1 {
                    eprintln!("Closing fd #{} (read error)", fd);
                }
                assert!((clients.arena[client].flags & REF_CACHE_ON_READ_LIST) == 0);
                ref_cache_server_c_194_close_client(clients, client, pw);
            }
            _ => {}
        }
    }
    clients.can_read = can_read;
}

// original: do_writes (htslib/ref_cache/server.c:642)
pub unsafe fn ref_cache_server_c_642_do_writes(
    clients: &mut RefCacheClientsLayout,
    opts: &Options,
    pw: &mut Poll_wrap,
    log_buf: &mut RefCacheLogBufferLayout,
) {
    let mut can_write: usize = NO_CLIENT;

    loop {
        let client = ref_cache_server_c_161_write_stack_pop(clients);
        if client == NO_CLIENT {
            break;
        }
        let fd = clients.arena[client].fd;

        if opts.verbosity > 1 {
            eprintln!("Writing to fd #{}", fd);
        }
        let cur_transact = clients.arena[client]
            .transact
            .expect("write stack only holds clients with a current transaction");
        let res = ref_cache_transaction_c_556_transaction_send_data(cur_transact, fd);
        if opts.verbosity > 2 {
            eprintln!("transact_send_data returned {}", res);
        }

        match res {
            REF_CACHE_WRITE_BLOCKED => {
                clients.arena[client].flags &= !REF_CACHE_CAN_WRITE;
            }
            REF_CACHE_WRITE_BLOCKED_UPSTREAM => {}
            REF_CACHE_WRITE_COMPLETE => {
                let host = clients.arena[client].host.clone();
                ref_cache_server_c_436_queue_log_msg(log_buf, &host, cur_transact);
                if ref_cache_transaction_c_214_transaction_get_keep_alive(cur_transact) != 0 {
                    clients.arena[client].transact =
                        ref_cache_transaction_c_204_switch_to_next_transaction(cur_transact);
                } else {
                    ref_cache_transaction_c_196_free_transaction_list(Some(cur_transact));
                    clients.arena[client].transact = None;
                    clients.arena[client].flags |= REF_CACHE_READ_DRAIN;
                }

                match clients.arena[client].transact {
                    None => {
                        clients.arena[client].last_transact = None;
                        if (clients.arena[client].flags
                            & (REF_CACHE_READ_CLOSED | REF_CACHE_READ_DRAIN))
                            != 0
                        {
                            if opts.verbosity > 2 {
                                eprintln!("Shutting down fd #{}", fd);
                            }
                            // Genuine syscall: shutdown(2).
                            if libc::shutdown(fd, libc::SHUT_WR) == -1 {
                                eprintln!(
                                    "Error from shutdown({}, SHUT_WR) : {}",
                                    fd,
                                    std::io::Error::last_os_error()
                                );
                                ref_cache_server_c_194_close_client(clients, client, pw);
                            }
                        }
                    }
                    Some(next) => {
                        if ref_cache_transaction_c_619_transaction_have_content(next) != 0 {
                            if opts.verbosity > 2 {
                                eprintln!("fd #{} can send more data...", fd);
                            }
                            ref_cache_server_c_151_write_stack_push(clients, client, &mut can_write);
                        }
                    }
                }
            }
            REF_CACHE_WRITE_MORE => {
                if opts.verbosity > 2 {
                    eprintln!("fd #{} can send more data...", fd);
                }
                ref_cache_server_c_151_write_stack_push(clients, client, &mut can_write);
            }
            REF_CACHE_WRITE_EOF | REF_CACHE_WRITE_ERROR => {
                if opts.verbosity > 1 {
                    eprintln!("Closing fd #{}", fd);
                }
                ref_cache_server_c_194_close_client(clients, client, pw);
            }
            _ => {}
        }
    }
    clients.can_write = can_write;
}

// original: run_poll_loop (htslib/ref_cache/server.c:721)
pub unsafe fn ref_cache_server_c_721_run_poll_loop(
    opts: &Options,
    lsocks: &mut Listeners,
    upstream: c_int,
    log_fd: c_int,
) -> c_int {
    let mut clients = RefCacheClientsLayout::default();
    let mut events = vec![std::mem::zeroed::<libc::epoll_event>(); REF_CACHE_MAX_EVENTS as usize];
    let mut log_buf = RefCacheLogBufferLayout::default();
    let mut log_can_write = 0;
    let mut running = 1;

    log_buf.in_ = 0;
    log_buf.out = 0;

    if ref_cache_misc_h_40_setnonblock(log_fd) != 0 {
        eprintln!("Setting nonblocking on log pipe: {}", std::io::Error::last_os_error());
        return -1;
    }

    if ref_cache_server_c_107_init_clients(&mut clients) != 0 {
        eprintln!("Initializing clients array: {}", std::io::Error::last_os_error());
        return -1;
    }

    // The poller is now an owned `Box<Poll_wrap>`; registration returns an
    // arena index (`Option<usize>`) used as the handle for mod/remove.
    let Some(mut pw_box) = ref_cache_poll_wrap_epoll_c_49_pw_init(opts.verbosity > 2) else {
        eprintln!("Initializing poller: {}", std::io::Error::last_os_error());
        return -1;
    };
    let pw = &mut *pw_box;

    if super::listener::ref_cache_listener_c_242_register_listener_pollers(lsocks, pw) != 0 {
        return -1;
    }

    if upstream != -1
        && ref_cache_poll_wrap_epoll_c_78_pw_register(
            pw,
            upstream,
            REF_CACHE_SV_UPSTREAM,
            (libc::EPOLLIN | libc::EPOLLERR | libc::EPOLLHUP) as u32,
            0,
        )
        .is_none()
    {
        eprintln!("Adding upstream socket to poller: {}", std::io::Error::last_os_error());
        return -1;
    }

    let Some(polled_log) = ref_cache_poll_wrap_epoll_c_78_pw_register(
        pw,
        log_fd,
        REF_CACHE_SV_LOG,
        (libc::EPOLLOUT | libc::EPOLLERR | libc::EPOLLHUP) as u32,
        0,
    ) else {
        eprintln!("Adding log pipe to poller: {}", std::io::Error::last_os_error());
        return -1;
    };

    while running != 0 || clients.count > 0 {
        let nevts = ref_cache_poll_wrap_epoll_c_120_pw_wait(
            pw,
            &mut events,
            if clients.can_read == NO_CLIENT && clients.can_write == NO_CLIENT {
                -1
            } else {
                0
            },
        );
        if nevts == -1 {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            eprintln!("Error from poller: {}", std::io::Error::last_os_error());
            return -1;
        }

        let mut e = 0;
        while e < nevts {
            let evts = events[e as usize].events;
            // epoll's `u64` carries the poller arena index; recover the item's
            // type/fd/userp through the poller rather than a raw pointer.
            let item_idx = events[e as usize].u64 as usize;
            let Some(item) = pw.item_at(item_idx) else {
                eprintln!("Stale polled item index {}", item_idx);
                return -1;
            };
            let fd_type = item.fd_type;
            let item_fd = item.fd;
            let item_client = item.userp;

            match fd_type {
                REF_CACHE_SV_LISTENER => {
                    if running != 0 {
                        ref_cache_server_c_268_handle_incoming(
                            opts,
                            item_fd,
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
                        opts.verbosity as c_int,
                    );
                }
                REF_CACHE_SV_LOG => {
                    if ref_cache_poll_wrap_epoll_c_106_pw_mod(
                        pw,
                        item_idx,
                        (libc::EPOLLERR | libc::EPOLLHUP) as u32,
                    ) != 0
                    {
                        eprintln!("Disarming log poller: {}", std::io::Error::last_os_error());
                        return -1;
                    }
                    if (evts & (libc::EPOLLHUP | libc::EPOLLERR) as u32) != 0 {
                        running = 0;
                    } else {
                        log_can_write = 1;
                    }
                }
                REF_CACHE_SV_CLIENT => {
                    ref_cache_server_c_509_handle_client_events(
                        &mut clients,
                        opts,
                        pw,
                        item_client,
                        evts,
                    );
                }
                _ => {
                    eprintln!("Unexpected polled item type {}", fd_type as c_int);
                    return -1;
                }
            }
            e += 1;
        }

        ref_cache_server_c_565_do_reads(&mut clients, opts, pw);
        ref_cache_server_c_642_do_writes(&mut clients, opts, pw, &mut log_buf);

        if log_can_write != 0 && log_buf.in_ != log_buf.out {
            let res = ref_cache_server_c_470_send_to_log(&mut log_buf, log_fd);
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
                    eprintln!("Re-arming log poller: {}", std::io::Error::last_os_error());
                    return -1;
                }
                log_can_write = 0;
            }
        }
    }
    0
}

// original: client_host (htslib/ref_cache/server.c:840)
pub fn ref_cache_server_c_840_client_host(clients: &RefCacheClientsLayout, client: usize) -> &[u8] {
    let host = &clients.arena[client].host;
    if host.is_empty() {
        b"<NULL>"
    } else {
        host
    }
}
