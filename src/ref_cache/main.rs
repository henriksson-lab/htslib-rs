/*  main.c -- ref-cache main entry point

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

use super::{
    listener::{self, Listeners},
    log_files::{self, Logfiles},
    misc, options,
    options::{DaemonType, MatchAddr, Options},
    ping,
    poll_wrap::{Pw_fd_type, Pw_item, PW_ERR, PW_HUP, PW_IN},
    poll_wrap_epoll as poll_impl,
    server::ref_cache_server_c_721_run_poll_loop,
};
use std::ffi::{c_char, c_int, c_ulong, c_void};

type PollWrap = poll_impl::Poll_wrap;
type PwEvents = poll_impl::Pw_events;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Child_type {
    CHLD_SERVER = 0,
    CHLD_UPSTREAM = 1,
}

#[repr(C)]
pub struct Child_proc {
    type_: Child_type,
    pid: libc::pid_t,
    polled_rd: *mut Pw_item,
    log_rd: c_int,
    log_wr: c_int,
    upstream: c_int,
}

static mut upstream: *mut c_int = std::ptr::null_mut();
static mut kids: *mut Child_proc = std::ptr::null_mut();
static mut nkids: usize = 0;
static mut sig_fds: [c_int; 2] = [0; 2];
pub static mut polled_sig: *mut Pw_item = std::ptr::null_mut();

static mut got_chld: crate::htslib_rs::ref_cache::compat::sig_atomic_t = 0;

static mut argv0: *mut c_char = std::ptr::null_mut();
static mut argv0_len: usize = 0;

const NI_MAXHOST: usize = 1025;
const MAX_EVENTS: c_int = 16;

unsafe extern "C" {
    fn run_upstream_handler(opts: *mut Options, upstream: *mut c_int, liveness_fd: c_int) -> c_int;
}

// original: change_name (htslib/ref_cache/main.c:99)
pub unsafe fn ref_cache_main_c_99_change_name(name: *mut c_char) {
    if argv0_len < libc::strlen(name) {
        return;
    }
    libc::strncpy(argv0, name, argv0_len);
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        libc::prctl(libc::PR_SET_NAME, name as c_ulong, 0, 0, 0);
    }
}

// original: init_children (htslib/ref_cache/main.c:107)
pub unsafe fn ref_cache_main_c_107_init_children(opts: *mut Options) -> c_int {
    upstream = libc::malloc(((*opts).max_kids as usize + 1) * std::mem::size_of::<c_int>()).cast();
    if upstream.is_null() {
        return -1;
    }

    kids = libc::malloc(((*opts).max_kids as usize + 1) * std::mem::size_of::<Child_proc>()).cast();
    if kids.is_null() {
        return -1;
    }

    let mut k = 0;
    while k <= (*opts).max_kids as c_int {
        (*kids.add(k as usize)).type_ = Child_type::CHLD_SERVER;
        (*kids.add(k as usize)).pid = 0;
        (*kids.add(k as usize)).polled_rd = std::ptr::null_mut();
        (*kids.add(k as usize)).log_rd = -1;
        (*kids.add(k as usize)).log_wr = -1;
        (*kids.add(k as usize)).upstream = -1;
        *upstream.add(k as usize) = -1;
        k += 1;
    }

    (*kids.add((*opts).max_kids as usize)).type_ = Child_type::CHLD_UPSTREAM;

    k = 0;
    while k < (*opts).max_kids as c_int {
        let mut sv = [0; 2];
        let mut pipefd = [0; 2];

        /* Make a pipe for the log file */
        if libc::pipe(pipefd.as_mut_ptr()) != 0 {
            libc::perror(c"pipe".as_ptr());
            break;
        }

        (*kids.add(k as usize)).log_rd = pipefd[0];
        (*kids.add(k as usize)).log_wr = pipefd[1];

        /* Sockets for upstream */
        if !(*opts).upstream_url.is_null() {
            if libc::socketpair(libc::AF_UNIX, libc::SOCK_DGRAM, 0, sv.as_mut_ptr()) != 0 {
                libc::perror(c"socketpair".as_ptr());
                break;
            }

            (*kids.add(k as usize)).upstream = sv[0];
            *upstream.add(k as usize) = sv[1];
        }
        k += 1;
    }
    if k < (*opts).max_kids as c_int {
        let mut i = 0;
        while i < (*opts).max_kids as c_int {
            if (*kids.add(i as usize)).log_rd >= 0 {
                libc::close((*kids.add(i as usize)).log_rd);
            }
            if (*kids.add(i as usize)).log_wr >= 0 {
                libc::close((*kids.add(i as usize)).log_wr);
            }
            if (*kids.add(i as usize)).upstream >= 0 {
                libc::close((*kids.add(i as usize)).upstream);
            }
            if *upstream.add(i as usize) >= 0 {
                libc::close(*upstream.add(i as usize));
            }
            i += 1;
        }
        return -1;
    }

    /* Pipe for signals */
    if libc::pipe(std::ptr::addr_of_mut!(sig_fds).cast::<c_int>()) != 0 {
        libc::perror(c"pipe".as_ptr());
        let mut i = 0;
        while i < (*opts).max_kids as c_int {
            if (*kids.add(i as usize)).log_rd >= 0 {
                libc::close((*kids.add(i as usize)).log_rd);
            }
            if (*kids.add(i as usize)).log_wr >= 0 {
                libc::close((*kids.add(i as usize)).log_wr);
            }
            if (*kids.add(i as usize)).upstream >= 0 {
                libc::close((*kids.add(i as usize)).upstream);
            }
            if *upstream.add(i as usize) >= 0 {
                libc::close(*upstream.add(i as usize));
            }
            i += 1;
        }
        return -1;
    }

    0
}

// original: set_up_child (htslib/ref_cache/main.c:168)
pub unsafe fn ref_cache_main_c_168_set_up_child(
    opts: *mut Options,
    mut k: c_int,
    is_upstream: c_int,
    pw: *mut PollWrap,
) -> c_int {
    let mut sigact: libc::sigaction = std::mem::zeroed();

    if is_upstream != 0 {
        k = (*opts).max_kids as c_int;
    }

    /* Restore default signal handler */
    sigact.sa_sigaction = libc::SIG_DFL;
    libc::sigemptyset(&mut sigact.sa_mask);
    sigact.sa_flags = 0;
    if libc::sigaction(libc::SIGCHLD, &sigact, std::ptr::null_mut()) != 0 {
        libc::perror(c"Resetting SIGCHLD handler".as_ptr());
        return -1;
    }

    assert!(!kids.is_null());

    /* Close all the file descriptors we don't need */
    let mut i = 0;
    while i < (*opts).max_kids as c_int {
        libc::close((*kids.add(i as usize)).log_rd);
        if is_upstream == 0 && *upstream.add(i as usize) >= 0 {
            libc::close(*upstream.add(i as usize));
        }
        if i != k {
            libc::close((*kids.add(i as usize)).log_wr);
            if (*kids.add(i as usize)).upstream >= 0 {
                libc::close((*kids.add(i as usize)).upstream);
            }
        }
        i += 1;
    }
    libc::close(sig_fds[0]);
    libc::close(sig_fds[1]);

    if !pw.is_null() {
        poll_impl::ref_cache_poll_wrap_epoll_c_72_pw_close(pw);
    }

    if (*opts).log != crate::htslib_rs::ref_cache::compat::stdout() {
        libc::close(libc::fileno((*opts).log));
    }

    libc::free(kids.cast());
    if is_upstream == 0 {
        libc::free(upstream.cast());
    }

    ref_cache_main_c_99_change_name(if is_upstream != 0 {
        c"refc[dl]".as_ptr().cast_mut()
    } else {
        c"refc[svr]".as_ptr().cast_mut()
    });

    0
}

// original: make_new_child (htslib/ref_cache/main.c:211)
pub unsafe fn ref_cache_main_c_211_make_new_child(
    opts: *mut Options,
    lsocks: *mut Listeners,
    pw: *mut PollWrap,
) -> c_int {
    assert!(!kids.is_null());

    /* Find a free slot */
    let mut k = 0;
    while k < (*opts).max_kids as c_int {
        if (*kids.add(k as usize)).pid == 0 {
            break;
        }
        k += 1;
    }
    assert!(k < (*opts).max_kids as c_int);

    /* Start the child process */
    let pid = libc::fork();
    if pid < 0 {
        libc::perror(c"fork".as_ptr());
        return -1;
    }

    if pid == 0 {
        /* Copy file descriptors as set_up_child frees kids[] */
        let upstr = (*kids.add(k as usize)).upstream;
        let log_wr = (*kids.add(k as usize)).log_wr;
        if ref_cache_main_c_168_set_up_child(opts, k, 0, pw) != 0 {
            libc::_exit(libc::EXIT_FAILURE);
        }
        let res = ref_cache_server_c_721_run_poll_loop(opts, lsocks, upstr, log_wr);
        libc::_exit(if res == 0 {
            libc::EXIT_SUCCESS
        } else {
            libc::EXIT_FAILURE
        });
    }

    (*kids.add(k as usize)).pid = pid;
    nkids += 1;
    0
}

// original: start_upstream (htslib/ref_cache/main.c:245)
pub unsafe fn ref_cache_main_c_245_start_upstream(opts: *mut Options, pw: *mut PollWrap) -> c_int {
    let mut liveness_pipe = [-1, -1];

    // Make pipe so child can detect parent going away
    if libc::pipe(liveness_pipe.as_mut_ptr()) < 0 {
        libc::perror(c"Opening pipe".as_ptr());
        return -1;
    }

    let upstream_pid = libc::fork();
    if upstream_pid == -1 {
        libc::perror(c"start_upstream couldn't fork".as_ptr());
        libc::close(liveness_pipe[0]);
        libc::close(liveness_pipe[1]);
        return -1;
    }

    if upstream_pid == 0 {
        libc::close(liveness_pipe[0]);
        if ref_cache_main_c_168_set_up_child(opts, 0, 1, pw) != 0 {
            libc::_exit(libc::EXIT_FAILURE);
        }

        let res = run_upstream_handler(opts, upstream, liveness_pipe[1]);

        libc::_exit(if res == 0 {
            libc::EXIT_SUCCESS
        } else {
            libc::EXIT_FAILURE
        });
    }

    assert!(!kids.is_null());

    libc::close(liveness_pipe[1]);
    (*kids.add((*opts).max_kids as usize)).pid = upstream_pid;
    (*kids.add((*opts).max_kids as usize)).type_ = Child_type::CHLD_UPSTREAM;
    (*kids.add((*opts).max_kids as usize)).log_rd = liveness_pipe[0];

    0
}

// original: sig_handler (htslib/ref_cache/main.c:283)
pub unsafe extern "C" fn ref_cache_main_c_283_sig_handler(signal: c_int) {
    match signal {
        libc::SIGCHLD => {
            if got_chld != 0 {
                return; /* save writing repeatedly to the pipe */
            }
            got_chld = 1;
        }
        _ => return,
    }

    if sig_fds[1] < 0 {
        return;
    }
    let c = b'*' as c_char;
    let mut bytes;
    loop {
        bytes = libc::write(sig_fds[1], (&c as *const c_char).cast(), 1);
        if !(bytes < 0 && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR) {
            break;
        }
    }
    if bytes < 0
        && *crate::htslib_rs::c_compat::__errno_location() != libc::EAGAIN
        && *crate::htslib_rs::c_compat::__errno_location() != libc::EWOULDBLOCK
    {
        libc::close(sig_fds[1]); /* Should get the attention of the other end... */
    }
}

// original: handle_sigchld (htslib/ref_cache/main.c:306)
pub unsafe fn ref_cache_main_c_306_handle_sigchld(opts: *mut Options, pw: *mut PollWrap) -> c_int {
    let mut buffer = [0 as c_char; 16];
    let mut pid;

    /* Drain the pipe */
    let mut bytes;
    loop {
        bytes = libc::read(sig_fds[0], buffer.as_mut_ptr().cast(), buffer.len());
        if !(bytes < 0 && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR) {
            break;
        }
    }
    if bytes <= 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"%s on signal fd #%d\n".as_ptr(),
            if bytes == 0 {
                c"EOF".as_ptr()
            } else {
                libc::strerror(*crate::htslib_rs::c_compat::__errno_location())
            },
            sig_fds[0],
        );
        return -1;
    }

    got_chld = 0; /* Allow more writes to the pipe */

    /* Reap the children */
    loop {
        let mut status = 0;
        pid = libc::waitpid(-1, &mut status, libc::WNOHANG);
        if pid < 0 {
            if *crate::htslib_rs::c_compat::__errno_location() == libc::ECHILD
                || *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR
            {
                continue;
            }
            libc::perror(c"waitpid".as_ptr());
            return -1;
        }

        if pid > 0 {
            if libc::WIFEXITED(status) {
                libc::fprintf(
                    crate::htslib_rs::ref_cache::compat::stderr(),
                    c"Child PID %d exited with status %d.\n".as_ptr(),
                    pid,
                    libc::WEXITSTATUS(status),
                );
            } else if libc::WIFSIGNALED(status) {
                libc::fprintf(
                    crate::htslib_rs::ref_cache::compat::stderr(),
                    c"Child PID %d terminated by signal %d.\n".as_ptr(),
                    pid,
                    libc::WTERMSIG(status),
                );
            } else {
                libc::fprintf(
                    crate::htslib_rs::ref_cache::compat::stderr(),
                    c"Child PID %d terminated\n".as_ptr(),
                    pid,
                );
            }
            let mut i = 0;
            while i < (*opts).max_kids as c_int + 1 {
                if (*kids.add(i as usize)).pid == pid {
                    (*kids.add(i as usize)).pid = 0;
                    if (*kids.add(i as usize)).type_ == Child_type::CHLD_UPSTREAM {
                        if ref_cache_main_c_245_start_upstream(opts, pw) != 0 {
                            return -1;
                        }
                    } else {
                        nkids -= 1;
                    }
                    break;
                }
                i += 1;
            }
        }
        if pid <= 0 {
            break;
        }
    }

    0
}

// original: init_poller (htslib/ref_cache/main.c:362)
pub unsafe fn ref_cache_main_c_362_init_poller(opts: *mut Options) -> *mut PollWrap {
    let pw = poll_impl::ref_cache_poll_wrap_epoll_c_49_pw_init(((*opts).verbosity > 2) as c_int);
    let mut k = 0usize;

    if pw.is_null() {
        return std::ptr::null_mut();
    }

    while k < (*opts).max_kids as usize {
        (*kids.add(k)).polled_rd = poll_impl::ref_cache_poll_wrap_epoll_c_78_pw_register(
            pw,
            (*kids.add(k)).log_rd,
            Pw_fd_type::MAIN_LOG_RD,
            (PW_IN | PW_ERR | PW_HUP) as u32,
            kids.add(k).cast(),
        );
        if (*kids.add(k)).polled_rd.is_null() {
            libc::perror(c"Registering log pipe with poller".as_ptr());
            break;
        }
        k += 1;
    }
    if k < (*opts).max_kids as usize {
        let mut j = 0usize;
        while j < nkids {
            if !(*kids.add(j)).polled_rd.is_null() {
                poll_impl::ref_cache_poll_wrap_epoll_c_126_pw_remove(
                    pw,
                    (*kids.add(j)).polled_rd,
                    0,
                );
            }
            j += 1;
        }
        return std::ptr::null_mut();
    }

    polled_sig = poll_impl::ref_cache_poll_wrap_epoll_c_78_pw_register(
        pw,
        sig_fds[0],
        Pw_fd_type::MAIN_SIG,
        (PW_IN | PW_HUP | PW_ERR) as u32,
        std::ptr::null_mut(),
    );
    if polled_sig.is_null() {
        libc::perror(c"Registering signal pipe with poller".as_ptr());
        let mut j = 0usize;
        while j < nkids {
            if !(*kids.add(j)).polled_rd.is_null() {
                poll_impl::ref_cache_poll_wrap_epoll_c_126_pw_remove(
                    pw,
                    (*kids.add(j)).polled_rd,
                    0,
                );
            }
            j += 1;
        }
        return std::ptr::null_mut();
    }

    pw
}

// original: run_server_population (htslib/ref_cache/main.c:393)
pub unsafe fn ref_cache_main_c_393_run_server_population(
    opts: *mut Options,
    lsocks: *mut Listeners,
    logfiles: *mut Logfiles,
) -> c_int {
    let mut sigact: libc::sigaction = std::mem::zeroed();
    let mut logged = 0;

    /* Set up signal handler */
    sigact.sa_sigaction = ref_cache_main_c_283_sig_handler as usize;
    libc::sigemptyset(&mut sigact.sa_mask);
    sigact.sa_flags = libc::SA_NOCLDSTOP;
    if libc::sigaction(libc::SIGCHLD, &sigact, std::ptr::null_mut()) != 0 {
        libc::perror(c"Setting up SIGCHLD handler".as_ptr());
        return -1;
    }

    let pw = ref_cache_main_c_362_init_poller(opts);
    if pw.is_null() {
        return -1;
    }

    /* Run poll loop */
    loop {
        let mut events: [PwEvents; MAX_EVENTS as usize] = std::mem::zeroed();

        /* Make new children if necessary */
        while nkids < (*opts).max_kids as usize {
            if ref_cache_main_c_211_make_new_child(opts, lsocks, pw) != 0 {
                if nkids == 0 {
                    libc::fprintf(
                        crate::htslib_rs::ref_cache::compat::stderr(),
                        c"Unable to make server processes, giving up.\n".as_ptr(),
                    );
                    return -1;
                } else {
                    break;
                }
            }
        }

        /* Wait for events */
        let ret = poll_impl::ref_cache_poll_wrap_epoll_c_120_pw_wait(
            pw,
            events.as_mut_ptr(),
            MAX_EVENTS,
            if logged != 0 { 100 } else { -1 },
        );
        if ret < 0 {
            if *crate::htslib_rs::c_compat::__errno_location() != libc::EINTR {
                libc::perror(c"Waiting for poller".as_ptr());
                return -1;
            }
            continue;
        }

        if ret == 0 && logged != 0 {
            /* Flush the log */
            libc::fflush((*opts).log);
            logged = 0;
            continue;
        }

        let mut e = 0;
        while e < ret {
            let item = (*events.as_ptr().add(e as usize)).u64 as *mut Pw_item;
            assert!(!item.is_null());

            match (*item).fd_type {
                Pw_fd_type::MAIN_SIG => {
                    /* Got a signal */
                    if ref_cache_main_c_306_handle_sigchld(opts, pw) != 0 {
                        return -1;
                    }
                }
                Pw_fd_type::MAIN_LOG_RD => {
                    /* Deal with log messages */
                    let mut buffer = [0 as c_char; 65536];
                    let kid = (*item).userp.cast::<Child_proc>();
                    assert!(!kid.is_null());
                    assert!((*kid).pid != 0);

                    let mut bytes;
                    loop {
                        bytes = libc::read((*kid).log_rd, buffer.as_mut_ptr().cast(), buffer.len());
                        if !(bytes < 0
                            && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR)
                        {
                            break;
                        }
                    }

                    if bytes <= 0 {
                        libc::fprintf(
                            crate::htslib_rs::ref_cache::compat::stderr(),
                            c"%s reading log fd #%d\n".as_ptr(),
                            if bytes == 0 {
                                c"EOF".as_ptr()
                            } else {
                                libc::strerror(*crate::htslib_rs::c_compat::__errno_location())
                            },
                            (*kid).log_rd,
                        );
                        libc::close((*kid).log_rd);
                        (*kid).log_rd = -1;
                    } else if (*opts).no_log == 0 {
                        if log_files::ref_cache_log_files_c_266_write_to_log(
                            logfiles,
                            opts,
                            buffer.as_ptr().cast(),
                            bytes as usize,
                        ) < 0
                        {
                            return -1;
                        }
                        if !(*opts).log_dir.is_null() {
                            logged = 1;
                        }
                    }
                }
                _ => {
                    libc::fprintf(
                        crate::htslib_rs::ref_cache::compat::stderr(),
                        c"Unexpected item type %x from poll\n".as_ptr(),
                        (*item).fd_type as c_int,
                    );
                    libc::abort();
                }
            }
            e += 1;
        }
    }
}

// original: daemonise (htslib/ref_cache/main.c:493)
pub unsafe fn ref_cache_main_c_493_daemonise(daemon_fds: *mut c_int, opts: *mut Options) -> c_int {
    let error_log_file = (*opts).error_log_file;
    let mut fd_limit: libc::rlimit = std::mem::zeroed();
    let mut sigact: libc::sigaction = std::mem::zeroed();
    let mut all_sigs: libc::sigset_t = std::mem::zeroed();

    if libc::getrlimit(libc::RLIMIT_NOFILE, &mut fd_limit) != 0 {
        libc::perror(c"Getting max file descriptor count".as_ptr());
        return -1;
    }

    // Close any open file descriptors above 3 (apart from the pipe passed in)
    let save_errno = *crate::htslib_rs::c_compat::__errno_location();
    let mut i = 3;
    while i < fd_limit.rlim_cur as c_int {
        if i != *daemon_fds.add(0) && i != *daemon_fds.add(1) {
            libc::close(i);
        }
        i += 1;
    }

    // Reset signal handlers to default
    // Unfortunately there's no portable way to count them, so take
    // a punt that it's going to be less than sizeof(sigset_t) * 8
    libc::sigfillset(&mut all_sigs);
    libc::sigdelset(&mut all_sigs, libc::SIGKILL); // Can't be changed
    libc::sigdelset(&mut all_sigs, libc::SIGSTOP); // Can't be changed
    i = 1;
    while i < (std::mem::size_of_val(&all_sigs) * 8) as c_int {
        if libc::sigismember(&all_sigs, i) == 0 {
            i += 1;
            continue;
        }
        libc::memset(
            (&mut sigact as *mut libc::sigaction).cast(),
            0,
            std::mem::size_of_val(&sigact),
        );
        sigact.sa_sigaction = libc::SIG_DFL;
        libc::sigemptyset(&mut sigact.sa_mask);
        if libc::sigaction(i, &sigact, std::ptr::null_mut()) != 0 {
            // Ideally, sigfillset() would only fill valid signals,
            // but sadly on Linux, at least, that doesn't appear to be
            // the case so we should expect to get EINVAL
            if *crate::htslib_rs::c_compat::__errno_location() != libc::EINVAL {
                libc::fprintf(
                    crate::htslib_rs::ref_cache::compat::stderr(),
                    c"Resetting signal handler %d: %s\n".as_ptr(),
                    i,
                    libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
                );
                return -1;
            } else {
                break;
            }
        }
        i += 1;
    }
    *crate::htslib_rs::c_compat::__errno_location() = save_errno;

    // Unblock all signals
    libc::sigemptyset(&mut all_sigs);
    if libc::sigprocmask(libc::SIG_SETMASK, &all_sigs, std::ptr::null_mut()) != 0 {
        libc::perror(c"Setting signal mask".as_ptr());
        return -1;
    }

    let pid1 = libc::fork();
    if pid1 < 0 {
        libc::perror(c"Couldn't fork".as_ptr());
        return -1;
    }
    if pid1 != 0 {
        // Check that the daemon started up successfully, by waiting for
        // a message from the pipe in daemon_fds[].
        let mut msg = [0 as c_char, 0 as c_char];

        // Ensure a clean exit from initial process
        libc::free((*opts).match_addrs.cast());

        libc::close(*daemon_fds.add(1)); // Writing end is for the daemon process
        let res = misc::ref_cache_misc_h_72_do_read_all(
            *daemon_fds.add(0),
            msg.as_mut_ptr().cast(),
            msg.len(),
        );
        libc::close(*daemon_fds.add(0));
        if res < 0 || !(msg[0] == b'o' as c_char && msg[1] == b'k' as c_char) {
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Daemon failed to start, sorry.\n".as_ptr(),
            );
            libc::_exit(libc::EXIT_FAILURE);
        }
        if (*opts).error_log_file.is_null() && (*opts).no_log == 0 {
            // Last chance to tell the user...
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Error messages will be unavailable after this one.\n".as_ptr(),
            );
        }
        libc::_exit(libc::EXIT_SUCCESS);
    }

    // Close the reading end of the pipe.  Writing end will be closed on
    // successful start.
    libc::close(*daemon_fds.add(0));
    *daemon_fds.add(0) = -1;

    // Become session leader
    if libc::setsid() < 0 {
        libc::perror(c"Couldn't become session leader".as_ptr());
        libc::_exit(libc::EXIT_FAILURE);
    }

    // Fork again to make the daemon
    let pid2 = libc::fork();
    if pid2 < 0 {
        libc::perror(c"Couldn't fork".as_ptr());
        libc::_exit(libc::EXIT_FAILURE);
    }
    if pid2 != 0 {
        // Exit so the daemon is inherited by PID 1
        libc::exit(libc::EXIT_SUCCESS);
    }

    // Redirect stdin and stdout to /dev/null
    let devnull = libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
    if devnull < 0 {
        libc::perror(c"Opening /dev/null".as_ptr());
        libc::exit(libc::EXIT_FAILURE);
    }
    if libc::dup2(devnull, 0) < 0 || libc::dup2(devnull, 1) < 0 || libc::dup2(devnull, 2) < 0 {
        libc::perror(c"Redirecting stdin and stdout to /dev/null".as_ptr());
        libc::exit(libc::EXIT_FAILURE);
    }

    // Redirect stderr to error_log_file, or /dev/null if not set
    if !error_log_file.is_null() {
        if libc::freopen(
            error_log_file,
            c"a".as_ptr(),
            crate::htslib_rs::ref_cache::compat::stderr(),
        )
        .is_null()
        {
            /* Does it still exist? */
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Couldn't redirect stderr to %s: %s\n".as_ptr(),
                error_log_file,
                libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
            );
            libc::exit(libc::EXIT_FAILURE);
        }
    } else if libc::dup2(devnull, 2) < 0 {
        libc::perror(c"Redirecting stderr to /dev/null".as_ptr());
        libc::exit(libc::EXIT_FAILURE);
    }

    if devnull > 2 {
        libc::close(devnull);
    }

    // Reset umask
    libc::umask(0);

    0
}

// original: get_systemd_listen_fds (htslib/ref_cache/main.c:633)
pub unsafe fn ref_cache_main_c_633_get_systemd_listen_fds(opts: *mut Options) -> c_int {
    let mut fd_limit: libc::rlimit = std::mem::zeroed();
    let our_pid = libc::getpid();
    let env_listen_pid = libc::getenv(c"LISTEN_PID".as_ptr());
    let env_listen_fds = libc::getenv(c"LISTEN_FDS".as_ptr());
    let mut end = env_listen_pid;
    if env_listen_pid.is_null() {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"LISTEN_PID environment variable is not set\n".as_ptr(),
        );
        return -1;
    }
    if env_listen_fds.is_null() {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"LISTEN_FDS environment variable is not set\n".as_ptr(),
        );
        return -1;
    }
    if libc::getrlimit(libc::RLIMIT_NOFILE, &mut fd_limit) != 0 {
        libc::perror(c"Getting max file descriptor count".as_ptr());
        return -1;
    }

    let listen_pid = libc::strtoll(env_listen_pid, &mut end, 10);
    if *env_listen_pid == 0 || *end != 0 || listen_pid < 0 || listen_pid != our_pid as i64 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"LISTEN_PID is incorrect\n".as_ptr(),
        );
        return -1;
    }
    end = env_listen_fds;
    let listen_fds = libc::strtol(env_listen_fds, &mut end, 10);
    if *env_listen_fds == 0
        || *end != 0
        || listen_fds <= 0
        || listen_fds > c_int::MAX as libc::c_long
        || listen_fds
            > fd_limit.rlim_cur as libc::c_long - options::FIRST_SD_LISTEN_FD as libc::c_long
    {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"LISTEN_FDS is not valid\n".as_ptr(),
        );
        return -1;
    }
    (*opts).listen_fds = listen_fds as c_int;
    libc::unsetenv(c"LISTEN_PID".as_ptr());
    libc::unsetenv(c"LISTEN_FDS".as_ptr());
    libc::unsetenv(c"LISTEN_FDNAMES".as_ptr());
    0
}

// original: add_match_addr (htslib/ref_cache/main.c:677)
pub unsafe fn ref_cache_main_c_677_add_match_addr(
    opts: *mut Options,
    addr_list: *const c_char,
) -> c_int {
    let mut host = [0 as c_char; NI_MAXHOST + 1];
    let addr_list_len = libc::strlen(addr_list);
    let mut host_start = 0usize;
    let mut addrs: *mut libc::addrinfo;
    let mut hints: libc::addrinfo = std::mem::zeroed();

    while host_start < addr_list_len {
        // Look for the IP address part
        let mut p = host_start;
        let host_len = libc::strcspn(addr_list.add(p), c"/,".as_ptr());
        if host_len >= NI_MAXHOST {
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"IP address \"%.20s...\" too long\n".as_ptr(),
                addr_list.add(host_start),
            );
            return -1;
        }
        libc::memcpy(host.as_mut_ptr().cast(), addr_list.add(p).cast(), host_len);
        host[host_len] = 0;
        p += host_len;

        // Check for CIDR-notation netmask
        let (mut netmask_bits, mut netmask_end) = if *addr_list.add(p) == b'/' as c_char {
            let netmask_len = libc::strcspn(addr_list.add(p), c",".as_ptr());
            let nm = addr_list.add(p + 1);
            let mut endp: *mut c_char = std::ptr::null_mut();
            let netmask_bits = libc::strtoul(nm, &mut endp, 10);
            if endp == nm.cast_mut() {
                libc::fprintf(
                    crate::htslib_rs::ref_cache::compat::stderr(),
                    c"Empty netmask for host \"%s\"\n".as_ptr(),
                    host.as_ptr(),
                );
                return -1;
            }
            if netmask_bits > 128 {
                libc::fprintf(
                    crate::htslib_rs::ref_cache::compat::stderr(),
                    c"Netmask \"%s/%lu\" too large\n".as_ptr(),
                    host.as_ptr(),
                    netmask_bits,
                );
                return -1;
            }
            (netmask_bits, p + netmask_len)
        } else {
            (128, p)
        };
        if netmask_end < addr_list_len && *addr_list.add(netmask_end) == b',' as c_char {
            netmask_end += 1;
        }

        // Use getaddrinfo() to convert the address to a struct sockaddr
        addrs = std::ptr::null_mut();
        libc::memset(
            (&mut hints as *mut libc::addrinfo).cast(),
            0,
            std::mem::size_of_val(&hints),
        );
        hints.ai_family = libc::AF_UNSPEC;
        hints.ai_socktype = libc::SOCK_STREAM;
        hints.ai_protocol = 0;
        hints.ai_canonname = std::ptr::null_mut();
        hints.ai_addr = std::ptr::null_mut();
        hints.ai_next = std::ptr::null_mut();
        hints.ai_flags = libc::AI_NUMERICHOST;
        let res = libc::getaddrinfo(host.as_ptr(), std::ptr::null(), &hints, &mut addrs);
        if res != 0 {
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Couldn't resolve IP address \"%s\" : %s\n".as_ptr(),
                host.as_ptr(),
                libc::gai_strerror(res),
            );
            return -1;
        }
        // In theory there could be more than one address, so iterate just
        // in case.
        let mut addr = addrs;
        while !addr.is_null() {
            // Ignore unexpected types
            if (*addr).ai_family != libc::AF_INET && (*addr).ai_family != libc::AF_INET6 {
                addr = (*addr).ai_next;
                continue;
            }

            // Grow output array if necessary
            if (*opts).match_addrs_size == (*opts).num_match_addrs {
                let new_size = if (*opts).match_addrs_size > 0 {
                    (*opts).match_addrs_size * 2
                } else {
                    16
                };
                let new_addrs = libc::realloc(
                    (*opts).match_addrs.cast(),
                    new_size * std::mem::size_of::<MatchAddr>(),
                )
                .cast::<MatchAddr>();
                if new_addrs.is_null() {
                    libc::perror(c"Allocating address match list".as_ptr());
                    libc::freeaddrinfo(addrs);
                    return -1;
                }
                (*opts).match_addrs_size = new_size;
                (*opts).match_addrs = new_addrs;
            }

            let ma = (*opts).match_addrs.add((*opts).num_match_addrs);
            libc::memset(ma.cast(), 0, std::mem::size_of::<MatchAddr>());
            (*ma).family = (*(*addr).ai_addr).sa_family;

            let (alen, addrp) = if (*addr).ai_family == libc::AF_INET
                && (*addr).ai_addrlen as usize == std::mem::size_of::<libc::sockaddr_in>()
            {
                (
                    std::mem::size_of::<libc::in_addr>(),
                    (&(*((*addr).ai_addr.cast::<libc::sockaddr_in>())).sin_addr
                        as *const libc::in_addr)
                        .cast::<u8>(),
                )
            } else if (*addr).ai_family == libc::AF_INET6
                && (*addr).ai_addrlen as usize == std::mem::size_of::<libc::sockaddr_in6>()
            {
                (
                    std::mem::size_of::<libc::in6_addr>(),
                    (&(*((*addr).ai_addr.cast::<libc::sockaddr_in6>())).sin6_addr
                        as *const libc::in6_addr)
                        .cast::<u8>(),
                )
            } else {
                libc::fprintf(
                    crate::htslib_rs::ref_cache::compat::stderr(),
                    c"Unexpected address type/length! Got %u/%zd expected %u/%zd or %u/%zd\n"
                        .as_ptr(),
                    (*(*addr).ai_addr).sa_family as libc::c_uint,
                    (*addr).ai_addrlen as libc::size_t,
                    libc::AF_INET as libc::c_uint,
                    std::mem::size_of::<libc::sockaddr_in>(),
                    libc::AF_INET6 as libc::c_uint,
                    std::mem::size_of::<libc::sockaddr_in6>(),
                );
                libc::freeaddrinfo(addrs);
                return -1;
            };
            let max_mask_bits = (alen * 8) as c_ulong;

            // Store netmask
            if netmask_bits > max_mask_bits {
                netmask_bits = max_mask_bits;
            }

            (*ma).mask_bytes = (netmask_bits / 8) as u8;
            if (*ma).mask_bytes as usize == alen {
                // This ensures ma->addr[ma->mask_bytes] is always valid
                // which makes checking easier
                (*ma).mask_bytes -= 1;
                (*ma).mask = 0xff;
            } else {
                (*ma).mask = ((0xff00u32 >> (netmask_bits & 7)) & 0xff) as u8;
            }

            // Copy masked IP address
            libc::memcpy(
                (*ma).addr.as_mut_ptr().cast(),
                addrp.cast(),
                (*ma).mask_bytes as usize,
            );
            (*ma).addr[(*ma).mask_bytes as usize] =
                *addrp.add((*ma).mask_bytes as usize) & (*ma).mask;

            (*opts).num_match_addrs += 1;
            addr = (*addr).ai_next;
        }
        libc::freeaddrinfo(addrs);
        host_start = netmask_end;
    }
    0
}

// original: cmp_match_addrs (htslib/ref_cache/main.c:816)
pub unsafe extern "C" fn ref_cache_main_c_816_cmp_match_addrs(
    av: *const c_void,
    bv: *const c_void,
) -> c_int {
    let a = av.cast::<MatchAddr>();
    let b = bv.cast::<MatchAddr>();
    let ip_order = if libc::AF_INET < libc::AF_INET6 {
        1
    } else {
        -1
    };

    if (*a).family != (*b).family {
        return ip_order * if (*a).family < (*b).family { -1 } else { 1 };
    }
    libc::memcmp(
        (*a).addr.as_ptr().cast(),
        (*b).addr.as_ptr().cast(),
        (*a).addr.len(),
    )
}

// original: sort_match_addrs (htslib/ref_cache/main.c:826)
pub unsafe fn ref_cache_main_c_826_sort_match_addrs(opts: *mut Options) {
    let mut i;
    let mut j;
    let mut ip6_seen = 0;

    if (*opts).match_addrs.is_null() {
        return;
    }

    libc::qsort(
        (*opts).match_addrs.cast(),
        (*opts).num_match_addrs,
        std::mem::size_of::<MatchAddr>(),
        Some(ref_cache_main_c_816_cmp_match_addrs),
    );

    i = 0;
    j = 0;
    while j < (*opts).num_match_addrs {
        if ip6_seen == 0 && (*(*opts).match_addrs.add(i)).family as c_int == libc::AF_INET6 {
            (*opts).first_ip6 = i;
            ip6_seen = 1;
        }
        loop {
            j += 1;
            if !(j < (*opts).num_match_addrs
                && libc::memcmp(
                    (*opts).match_addrs.add(i).cast(),
                    (*opts).match_addrs.add(j).cast(),
                    std::mem::size_of::<MatchAddr>(),
                ) == 0)
            {
                break;
            }
        }
        i += 1;
        if j < (*opts).num_match_addrs && i < j {
            libc::memcpy(
                (*opts).match_addrs.add(i).cast(),
                (*opts).match_addrs.add(j).cast(),
                std::mem::size_of::<MatchAddr>(),
            );
        }
    }
    (*opts).num_match_addrs = i;
    if ip6_seen == 0 {
        (*opts).first_ip6 = i;
    }
}

// original: usage (htslib/ref_cache/main.c:857)
pub unsafe fn ref_cache_main_c_857_usage(prog: *const c_char, help: c_int, opts: *const Options) {
    libc::fprintf(
        crate::htslib_rs::ref_cache::compat::stderr(),
        c"Usage: %s [options] -d <dir>\n".as_ptr(),
        prog,
    );
    if help != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"Options:\n  -b         Run in background as a daemon\n  -d <dir>   Directory for cached reference files\n  -h         Show help\n  -l <dir>   Directory for log files.  Log to stdout if not set and running in\n             foreground\n  -L         Don't log\n  -m <list>  Only respond to connections from these networks\n  -n <1-4>   Number of server processes to run [%u]\n  -p <num>   Port number to listen on [%u]\n  -r <num>   Number of request log files to keep [%u]\n  -R <num>   Maximum size of a single request log file (MiB) [%lld]\n  -s         Run as a systemd socket service\n  -u <url>   URL for upstream server%s%s%s\n  -U         Only serve local files, turn off upstream\n  -v         Turn on debugging output\n"
                .as_ptr(),
            (*opts).max_kids as libc::c_uint,
            (*opts).port as libc::c_uint,
            (*opts).nlogs as libc::c_uint,
            ((*opts).max_log_sz >> 20) as libc::c_longlong,
            if !(*opts).upstream_url.is_null() {
                if (*opts).upstream_url_len > 40 {
                    c"\n             [".as_ptr()
                } else {
                    c" [".as_ptr()
                }
            } else {
                c"".as_ptr()
            },
            if !(*opts).upstream_url.is_null() {
                (*opts).upstream_url
            } else {
                c"".as_ptr()
            },
            if !(*opts).upstream_url.is_null() {
                c"]".as_ptr()
            } else {
                c"".as_ptr()
            },
        );
    }
}

// original: get_opt_val (htslib/ref_cache/main.c:889)
pub unsafe fn ref_cache_main_c_889_get_opt_val(
    arg: *const c_char,
    prog: *const c_char,
    opt: *const c_char,
    min: u16,
    max: u16,
    badarg: *mut c_int,
) -> u16 {
    let mut endp: *mut c_char = std::ptr::null_mut();
    let lmin = min as libc::c_long;
    let lmax = max as libc::c_long;
    let val = libc::strtol(arg, &mut endp, 0);
    if *arg == 0 || *endp != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"%s : %s option value \"%s\" is not a number\n".as_ptr(),
            prog,
            opt,
            arg,
        );
        *badarg = 1;
        return 0;
    }
    if val < lmin || val > lmax {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"%s : %s option value \"%s\" should be between %ld and %ld\n".as_ptr(),
            prog,
            opt,
            arg,
            lmin,
            lmax,
        );
        *badarg = 1;
        return 0;
    }
    val as u16
}

// original: main (htslib/ref_cache/main.c:913)
pub unsafe fn ref_cache_main_c_913_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut opts: Options = std::mem::zeroed();
    let mut lsocks: *mut Listeners = std::ptr::null_mut();
    let mut logfiles: *mut Logfiles = std::ptr::null_mut();
    let mut badarg = 0;
    let mut retval = libc::EXIT_FAILURE;
    let mut show_help = 0;
    let mut daemon_pipe = [-1, -1];
    let ip_ranges_all = c"0.0.0.0/0,::/0".as_ptr();
    let ip_ranges_localhost = c"127.0.0.0/8,::1/128".as_ptr();
    let ip_ranges_default = c"10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,fc00::/7,fe80::/10".as_ptr();

    #[allow(clippy::never_loop)]
    'main_body: loop {
        /* Copy argv[0] for change_name() */
        argv0 = *argv.add(0);
        argv0_len = libc::strlen(argv0);

        /* Options */

        libc::memset(
            (&mut opts as *mut Options).cast(),
            0,
            std::mem::size_of_val(&opts),
        );

        opts.port = 8000;
        opts.cache_dir = std::ptr::null();
        opts.cache_fd = -1;
        opts.max_kids = 1;
        opts.verbosity = 0;
        opts.log_dir = std::ptr::null();
        opts.log = crate::htslib_rs::ref_cache::compat::stdout();
        opts.match_addrs = std::ptr::null_mut();
        opts.num_match_addrs = 0;
        opts.match_addrs_size = 0;
        opts.error_log_file = std::ptr::null();
        opts.nlogs = 5;
        opts.max_log_sz = 10 << 20;
        opts.daemon = DaemonType::not_a_daemon;
        opts.no_log = 0;
        opts.upstream_url = c"https://www.ebi.ac.uk/ena/cram/md5/".as_ptr();

        loop {
            let c = crate::htslib_rs::ref_cache::compat::getopt_(
                argc,
                argv,
                c"be:d:hl:Lm:n:p:r:R:su:Uv".as_ptr(),
            );
            if c < 0 {
                break;
            }
            match c as u8 as char {
                'b' => {
                    if opts.daemon != DaemonType::not_a_daemon {
                        libc::fprintf(
                            crate::htslib_rs::ref_cache::compat::stderr(),
                            c"The -b and -s options cannot be used together\n".as_ptr(),
                        );
                        badarg = 1;
                    } else {
                        opts.daemon = DaemonType::sysv_daemon;
                    }
                }
                'e' => opts.error_log_file = crate::htslib_rs::ref_cache::compat::optarg(),
                'd' => opts.cache_dir = crate::htslib_rs::ref_cache::compat::optarg(),
                'h' => show_help = 1,
                'l' => {
                    opts.log_dir = crate::htslib_rs::ref_cache::compat::optarg();
                    opts.no_log = 0;
                }
                'L' => {
                    if opts.log_dir.is_null() {
                        opts.no_log = 1;
                    }
                }
                'm' => {
                    let mut to_add = crate::htslib_rs::ref_cache::compat::optarg();
                    if libc::strcmp(
                        crate::htslib_rs::ref_cache::compat::optarg(),
                        c"all".as_ptr(),
                    ) == 0
                    {
                        to_add = ip_ranges_all;
                    } else if libc::strcmp(
                        crate::htslib_rs::ref_cache::compat::optarg(),
                        c"default".as_ptr(),
                    ) == 0
                    {
                        to_add = ip_ranges_default;
                    } else if libc::strcmp(
                        crate::htslib_rs::ref_cache::compat::optarg(),
                        c"localhost".as_ptr(),
                    ) == 0
                    {
                        to_add = ip_ranges_localhost;
                    }
                    if ref_cache_main_c_677_add_match_addr(&mut opts, to_add) != 0 {
                        break 'main_body;
                    }
                }
                'n' => {
                    opts.max_kids = ref_cache_main_c_889_get_opt_val(
                        crate::htslib_rs::ref_cache::compat::optarg(),
                        *argv.add(0),
                        c"-n".as_ptr(),
                        1,
                        4,
                        &mut badarg,
                    )
                }
                'p' => {
                    opts.port = ref_cache_main_c_889_get_opt_val(
                        crate::htslib_rs::ref_cache::compat::optarg(),
                        *argv.add(0),
                        c"-p".as_ptr(),
                        1,
                        65535,
                        &mut badarg,
                    )
                }
                'r' => {
                    opts.nlogs = ref_cache_main_c_889_get_opt_val(
                        crate::htslib_rs::ref_cache::compat::optarg(),
                        *argv.add(0),
                        c"-r".as_ptr(),
                        1,
                        100,
                        &mut badarg,
                    )
                }
                'R' => {
                    opts.max_log_sz = ref_cache_main_c_889_get_opt_val(
                        crate::htslib_rs::ref_cache::compat::optarg(),
                        *argv.add(0),
                        c"-R".as_ptr(),
                        1,
                        1000,
                        &mut badarg,
                    ) as libc::off_t
                        * (1 << 20)
                }
                's' => {
                    if opts.daemon != DaemonType::not_a_daemon {
                        libc::fprintf(
                            crate::htslib_rs::ref_cache::compat::stderr(),
                            c"The -b and -s options cannot be used together\n".as_ptr(),
                        );
                        badarg = 1;
                    } else {
                        opts.daemon = DaemonType::systemd_socket_service;
                    }
                }
                'u' => opts.upstream_url = crate::htslib_rs::ref_cache::compat::optarg(),
                'U' => opts.upstream_url = std::ptr::null(),
                'v' => opts.verbosity += 1,
                _ => {
                    ref_cache_main_c_857_usage(*argv.add(0), 0, &opts);
                    break 'main_body;
                }
            }
        }

        if show_help != 0 {
            ref_cache_main_c_857_usage(*argv.add(0), 1, &opts);
            retval = libc::EXIT_SUCCESS;
            break 'main_body;
        }

        if badarg != 0 {
            break 'main_body;
        }

        if opts.cache_dir.is_null() {
            ref_cache_main_c_857_usage(*argv.add(0), 0, &opts);
            break 'main_body;
        }

        if opts.match_addrs.is_null()
            && ref_cache_main_c_677_add_match_addr(&mut opts, ip_ranges_default) != 0
        {
            break 'main_body;
        }
        if ref_cache_main_c_677_add_match_addr(&mut opts, ip_ranges_localhost) != 0 {
            break 'main_body;
        }

        ref_cache_main_c_826_sort_match_addrs(&mut opts);

        if opts.daemon != DaemonType::systemd_socket_service {
            /* See if we're already running */
            let res = ping::ref_cache_ping_c_39_check_running(opts.port as c_int);
            if res != 0 {
                retval = if res < 0 {
                    libc::EXIT_FAILURE
                } else {
                    libc::EXIT_SUCCESS
                };
                break 'main_body;
            }
        }

        /* Turn into a daemon, if requested */
        match opts.daemon {
            DaemonType::sysv_daemon => {
                if opts.log_dir.is_null() && opts.no_log == 0 {
                    libc::fprintf(
                    crate::htslib_rs::ref_cache::compat::stderr(),
                    c"Warning: Running as a daemon without setting a request log directory!\nRequest logs will be unavailable.\n".as_ptr(),
                );
                }
                if opts.error_log_file.is_null() && opts.no_log == 0 {
                    libc::fprintf(
                        crate::htslib_rs::ref_cache::compat::stderr(),
                        c"Warning: Running as a daemon without setting an error log file!\n"
                            .as_ptr(),
                    );
                }

                if libc::pipe(daemon_pipe.as_mut_ptr()) < 0 {
                    libc::perror(c"Opening pipe".as_ptr());
                    break 'main_body;
                }
                if ref_cache_main_c_493_daemonise(daemon_pipe.as_mut_ptr(), &mut opts) != 0 {
                    break 'main_body;
                }
            }
            DaemonType::systemd_socket_service => {
                if ref_cache_main_c_633_get_systemd_listen_fds(&mut opts) != 0 {
                    break 'main_body;
                }
            }
            _ => {}
        }

        /* Log files */
        logfiles = log_files::ref_cache_log_files_c_148_open_logs(&opts);
        if logfiles.is_null() {
            break 'main_body;
        }

        /* Open cache directory */
        opts.cache_fd = libc::open(opts.cache_dir, libc::O_RDONLY | libc::O_DIRECTORY);
        if opts.cache_fd < 0 {
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Couldn't open directory %s: %s\n".as_ptr(),
                opts.cache_dir,
                libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
            );
            break 'main_body;
        }

        /* Allocate data structures and plumbing for child processes */

        if ref_cache_main_c_107_init_children(&mut opts) != 0 {
            break 'main_body;
        }

        /* Start the upstream handler if needed */

        if !opts.upstream_url.is_null() {
            opts.upstream_url_len = libc::strlen(opts.upstream_url);
            if ref_cache_main_c_245_start_upstream(&mut opts, std::ptr::null_mut()) != 0 {
                break 'main_body;
            }
        }

        /* Open the socket to listen on */

        match opts.daemon {
            DaemonType::systemd_socket_service => {
                lsocks = listener::ref_cache_listener_c_203_adopt_listen_sockets(
                    options::FIRST_SD_LISTEN_FD,
                    opts.listen_fds,
                );
            }
            _ => {
                lsocks = listener::ref_cache_listener_c_95_get_listen_sockets(opts.port as c_int);
            }
        }
        if lsocks.is_null() {
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Couldn't start up.  Sorry.\n".as_ptr(),
            );
            break 'main_body;
        }

        if opts.daemon == DaemonType::sysv_daemon {
            if misc::ref_cache_misc_h_55_do_write_all(daemon_pipe[1], c"ok".as_ptr().cast(), 2) < 0
            {
                libc::perror(c"Couldn't report successful start back to parent".as_ptr());
                break 'main_body;
            }
            libc::close(daemon_pipe[1]);
            daemon_pipe[1] = -1;
        } else if !opts.error_log_file.is_null()
            && libc::freopen(
                opts.error_log_file,
                c"a".as_ptr(),
                crate::htslib_rs::ref_cache::compat::stderr(),
            )
            .is_null()
        {
            /* Does it still exist? */
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Couldn't redirect stderr to %s: %s\n".as_ptr(),
                opts.error_log_file,
                libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
            );
            break 'main_body;
        }

        /* Run the servers */

        let res = ref_cache_main_c_393_run_server_population(&mut opts, lsocks, logfiles);
        if res == -1 {
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Server died.\n".as_ptr(),
            );
        }
        retval = if res == 0 {
            libc::EXIT_SUCCESS
        } else {
            libc::EXIT_FAILURE
        };

        break 'main_body;
    }

    if daemon_pipe[0] >= 0 {
        libc::close(daemon_pipe[0]);
    }
    if daemon_pipe[1] >= 0 {
        libc::close(daemon_pipe[1]);
    }
    if opts.cache_fd >= 0 {
        libc::close(opts.cache_fd);
    }
    if !lsocks.is_null() {
        listener::ref_cache_listener_c_235_close_listen_sockets(lsocks);
        libc::free(lsocks.cast());
    }
    log_files::ref_cache_log_files_c_134_close_logs(logfiles);
    libc::free(opts.match_addrs.cast());
    retval
}
