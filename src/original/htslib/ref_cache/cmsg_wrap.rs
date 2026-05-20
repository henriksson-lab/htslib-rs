use std::ffi::{c_char, c_int, c_uint};

// original: make_scm_rights_cmsg (htslib/ref_cache/cmsg_wrap.c:46)
pub unsafe fn ref_cache_cmsg_wrap_c_46_make_scm_rights_cmsg(
    msg: *mut libc::msghdr,
    fd: c_int,
    buf: *mut c_char,
    buf_sz: usize,
) -> c_int {
    let needed = libc::CMSG_SPACE(std::mem::size_of_val(&fd) as c_uint) as usize;
    if needed > buf_sz {
        return -1;
    } else {
        (*msg).msg_control = buf.cast();
    }

    (*msg).msg_controllen = needed;
    libc::memset((*msg).msg_control, 0, needed);

    let cmsg = libc::CMSG_FIRSTHDR(msg);
    (*cmsg).cmsg_level = libc::SOL_SOCKET;
    (*cmsg).cmsg_type = libc::SCM_RIGHTS;
    (*cmsg).cmsg_len = libc::CMSG_LEN(std::mem::size_of_val(&fd) as c_uint) as usize;

    let fdptr = libc::CMSG_DATA(cmsg);
    libc::memcpy(
        fdptr.cast(),
        (&fd as *const c_int).cast(),
        std::mem::size_of_val(&fd),
    );
    0
}

// original: get_scm_rights_fd (htslib/ref_cache/cmsg_wrap.c:67)
pub unsafe fn ref_cache_cmsg_wrap_c_67_get_scm_rights_fd(msg: *mut libc::msghdr) -> c_int {
    let mut cmsg = libc::CMSG_FIRSTHDR(msg);
    let mut fd = -1;

    while !cmsg.is_null() {
        if (*cmsg).cmsg_level == libc::SOL_SOCKET && (*cmsg).cmsg_type == libc::SCM_RIGHTS {
            let fdp = libc::CMSG_DATA(cmsg);
            libc::memcpy(
                (&mut fd as *mut c_int).cast(),
                fdp.cast(),
                std::mem::size_of::<c_int>(),
            );
            break;
        }
        cmsg = libc::CMSG_NXTHDR(msg, cmsg);
    }

    fd
}
