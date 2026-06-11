// original: make_scm_rights_cmsg (htslib/ref_cache/cmsg_wrap.c:46)
pub unsafe fn ref_cache_cmsg_wrap_c_46_make_scm_rights_cmsg(
    msg: &mut libc::msghdr,
    fd: i32,
    buf: &mut [u8],
) -> i32 {
    let needed = libc::CMSG_SPACE(std::mem::size_of_val(&fd) as u32) as usize;
    if needed > buf.len() {
        return -1;
    } else {
        msg.msg_control = buf.as_mut_ptr().cast();
    }

    msg.msg_controllen = needed;
    buf[..needed].fill(0);

    let cmsg = libc::CMSG_FIRSTHDR(msg);
    (*cmsg).cmsg_level = libc::SOL_SOCKET;
    (*cmsg).cmsg_type = libc::SCM_RIGHTS;
    (*cmsg).cmsg_len = libc::CMSG_LEN(std::mem::size_of_val(&fd) as u32) as usize;

    let fdptr = libc::CMSG_DATA(cmsg);
    let fd_bytes = fd.to_ne_bytes();
    std::slice::from_raw_parts_mut(fdptr, fd_bytes.len()).copy_from_slice(&fd_bytes);
    0
}

// original: get_scm_rights_fd (htslib/ref_cache/cmsg_wrap.c:67)
pub unsafe fn ref_cache_cmsg_wrap_c_67_get_scm_rights_fd(msg: &mut libc::msghdr) -> i32 {
    let mut cmsg = libc::CMSG_FIRSTHDR(msg);
    let mut fd = -1;

    while let Some(c) = cmsg.as_ref() {
        if c.cmsg_level == libc::SOL_SOCKET && c.cmsg_type == libc::SCM_RIGHTS {
            let fdp = libc::CMSG_DATA(cmsg);
            let mut fd_bytes = [0u8; std::mem::size_of::<i32>()];
            let fd_bytes_len = fd_bytes.len();
            fd_bytes.copy_from_slice(std::slice::from_raw_parts(fdp, fd_bytes_len));
            fd = i32::from_ne_bytes(fd_bytes);
            break;
        }
        cmsg = libc::CMSG_NXTHDR(msg, cmsg);
    }

    fd
}
