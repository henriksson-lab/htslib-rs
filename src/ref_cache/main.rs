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
    poll_wrap::{Pw_fd_type, PW_ERR, PW_HUP, PW_IN},
    poll_wrap_epoll as poll_impl,
    server::ref_cache_server_c_721_run_poll_loop,
    upstream,
};
use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::ffi::{c_char, c_int, c_ulong};

type PollWrap = poll_impl::Poll_wrap;
type PwEvents = poll_impl::Pw_events;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Child_type {
    CHLD_SERVER = 0,
    CHLD_UPSTREAM = 1,
}

#[derive(Clone, Copy)]
pub struct Child_proc {
    type_: Child_type,
    pid: libc::pid_t,
    // Arena handle returned by the poller (an index into its item arena), or None
    // when this child's log pipe is not currently registered.
    polled_rd: Option<usize>,
    log_rd: c_int,
    log_wr: c_int,
    upstream: c_int,
}

struct ProcessVec<T>(UnsafeCell<Option<Vec<T>>>);

unsafe impl<T> Sync for ProcessVec<T> {}

static UPSTREAM: ProcessVec<c_int> = ProcessVec(UnsafeCell::new(None));
static KIDS: ProcessVec<Child_proc> = ProcessVec(UnsafeCell::new(None));
static mut nkids: usize = 0;
static mut sig_fds: [c_int; 2] = [0; 2];
// Arena handle for the signal pipe registration (index into the poller's arena).
pub static mut polled_sig: Option<usize> = None;

static mut got_chld: crate::htslib_rs::ref_cache::compat::sig_atomic_t = 0;

// Points into the process's own argv[0] memory, so overwriting it changes the
// name shown by `ps`. This is genuinely an OS-level in-place buffer.
static mut argv0: *mut u8 = std::ptr::null_mut();
static mut argv0_len: usize = 0;

const NI_MAXHOST: usize = 1025;
const MAX_EVENTS: c_int = 16;

// original: change_name (htslib/ref_cache/main.c:99)
pub unsafe fn ref_cache_main_c_99_change_name(name: &[u8]) {
    if argv0_len < name.len() {
        return;
    }
    // Overwrite argv[0] in place (strncpy semantics: copy then NUL-pad to argv0_len).
    let dst = std::slice::from_raw_parts_mut(argv0, argv0_len);
    dst[..name.len()].copy_from_slice(name);
    dst[name.len()..].fill(0);
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        // PR_SET_NAME wants a NUL-terminated string.
        let mut name_c = name.to_vec();
        name_c.push(0);
        libc::prctl(libc::PR_SET_NAME, name_c.as_ptr() as c_ulong, 0, 0, 0);
    }
}

// original: init_children (htslib/ref_cache/main.c:107)
pub unsafe fn ref_cache_main_c_107_init_children(opts: &Options) -> c_int {
    let max_kids = opts.max_kids as usize;
    *UPSTREAM.0.get() = Some(vec![-1; max_kids + 1]);
    *KIDS.0.get() = Some(vec![
        Child_proc {
            type_: Child_type::CHLD_SERVER,
            pid: 0,
            polled_rd: None,
            log_rd: -1,
            log_wr: -1,
            upstream: -1,
        };
        max_kids + 1
    ]);

    let upstream_vec = (*UPSTREAM.0.get()).as_mut().unwrap();
    let kids_vec = (*KIDS.0.get()).as_mut().unwrap();

    let mut k = 0;
    while k <= opts.max_kids as c_int {
        kids_vec[k as usize].type_ = Child_type::CHLD_SERVER;
        kids_vec[k as usize].pid = 0;
        kids_vec[k as usize].polled_rd = None;
        kids_vec[k as usize].log_rd = -1;
        kids_vec[k as usize].log_wr = -1;
        kids_vec[k as usize].upstream = -1;
        upstream_vec[k as usize] = -1;
        k += 1;
    }

    kids_vec[max_kids].type_ = Child_type::CHLD_UPSTREAM;

    k = 0;
    while k < opts.max_kids as c_int {
        let mut sv = [0; 2];
        let mut pipefd = [0; 2];

        /* Make a pipe for the log file */
        if libc::pipe(pipefd.as_mut_ptr()) != 0 {
            eprintln!("pipe: {}", std::io::Error::last_os_error());
            break;
        }

        kids_vec[k as usize].log_rd = pipefd[0];
        kids_vec[k as usize].log_wr = pipefd[1];

        /* Sockets for upstream */
        if opts.upstream_url.is_some() {
            if libc::socketpair(libc::AF_UNIX, libc::SOCK_DGRAM, 0, sv.as_mut_ptr()) != 0 {
                eprintln!("socketpair: {}", std::io::Error::last_os_error());
                break;
            }

            kids_vec[k as usize].upstream = sv[0];
            upstream_vec[k as usize] = sv[1];
        }
        k += 1;
    }
    if k < opts.max_kids as c_int {
        let mut i = 0;
        while i < opts.max_kids as c_int {
            if kids_vec[i as usize].log_rd >= 0 {
                libc::close(kids_vec[i as usize].log_rd);
            }
            if kids_vec[i as usize].log_wr >= 0 {
                libc::close(kids_vec[i as usize].log_wr);
            }
            if kids_vec[i as usize].upstream >= 0 {
                libc::close(kids_vec[i as usize].upstream);
            }
            if upstream_vec[i as usize] >= 0 {
                libc::close(upstream_vec[i as usize]);
            }
            i += 1;
        }
        *KIDS.0.get() = None;
        *UPSTREAM.0.get() = None;
        return -1;
    }

    /* Pipe for signals */
    if libc::pipe(std::ptr::addr_of_mut!(sig_fds).cast::<c_int>()) != 0 {
        eprintln!("pipe: {}", std::io::Error::last_os_error());
        let mut i = 0;
        while i < opts.max_kids as c_int {
            if kids_vec[i as usize].log_rd >= 0 {
                libc::close(kids_vec[i as usize].log_rd);
            }
            if kids_vec[i as usize].log_wr >= 0 {
                libc::close(kids_vec[i as usize].log_wr);
            }
            if kids_vec[i as usize].upstream >= 0 {
                libc::close(kids_vec[i as usize].upstream);
            }
            if upstream_vec[i as usize] >= 0 {
                libc::close(upstream_vec[i as usize]);
            }
            i += 1;
        }
        *KIDS.0.get() = None;
        *UPSTREAM.0.get() = None;
        return -1;
    }

    0
}

// original: set_up_child (htslib/ref_cache/main.c:168)
pub unsafe fn ref_cache_main_c_168_set_up_child(
    opts: &Options,
    mut k: c_int,
    is_upstream: c_int,
    pw: Option<&mut PollWrap>,
) -> c_int {
    let mut sigact: libc::sigaction = std::mem::zeroed();

    if is_upstream != 0 {
        k = opts.max_kids as c_int;
    }

    /* Restore default signal handler */
    sigact.sa_sigaction = libc::SIG_DFL;
    libc::sigemptyset(&mut sigact.sa_mask);
    sigact.sa_flags = 0;
    if libc::sigaction(libc::SIGCHLD, &sigact, std::ptr::null_mut()) != 0 {
        eprintln!("Resetting SIGCHLD handler: {}", std::io::Error::last_os_error());
        return -1;
    }

    {
        let kids_vec = (*KIDS.0.get()).as_mut().unwrap();
        let upstream_vec = (*UPSTREAM.0.get()).as_mut().unwrap();

        /* Close all the file descriptors we don't need */
        let mut i = 0;
        while i < opts.max_kids as c_int {
            libc::close(kids_vec[i as usize].log_rd);
            if is_upstream == 0 && upstream_vec[i as usize] >= 0 {
                libc::close(upstream_vec[i as usize]);
            }
            if i != k {
                libc::close(kids_vec[i as usize].log_wr);
                if kids_vec[i as usize].upstream >= 0 {
                    libc::close(kids_vec[i as usize].upstream);
                }
            }
            i += 1;
        }
    }
    libc::close(sig_fds[0]);
    libc::close(sig_fds[1]);

    // In the forked child we no longer need the parent's poller. We only hold a
    // borrow here (the parent retains ownership of the `Box`), and the child
    // promptly runs its own loop / `_exit`s, so the inherited epoll fd is
    // reclaimed at exit. (The owning side calls pw_close on the real `Box`.)
    let _ = &pw;

    if opts.log != crate::htslib_rs::ref_cache::compat::stdout() {
        libc::close(libc::fileno(opts.log));
    }

    *KIDS.0.get() = None;
    if is_upstream == 0 {
        *UPSTREAM.0.get() = None;
    }

    ref_cache_main_c_99_change_name(if is_upstream != 0 {
        b"refc[dl]"
    } else {
        b"refc[svr]"
    });

    0
}

// original: make_new_child (htslib/ref_cache/main.c:211)
pub unsafe fn ref_cache_main_c_211_make_new_child(
    opts: &Options,
    lsocks: &mut Listeners,
    pw: Option<&mut PollWrap>,
) -> c_int {
    let kids_vec = (*KIDS.0.get()).as_mut().unwrap();

    /* Find a free slot */
    let mut k = 0;
    while k < opts.max_kids as c_int {
        if kids_vec[k as usize].pid == 0 {
            break;
        }
        k += 1;
    }
    assert!(k < opts.max_kids as c_int);

    /* Start the child process */
    let pid = libc::fork();
    if pid < 0 {
        eprintln!("fork: {}", std::io::Error::last_os_error());
        return -1;
    }

    if pid == 0 {
        /* Copy file descriptors as set_up_child frees kids[] */
        let upstr = kids_vec[k as usize].upstream;
        let log_wr = kids_vec[k as usize].log_wr;
        if ref_cache_main_c_168_set_up_child(opts, k, 0, pw) != 0 {
            libc::_exit(1);
        }
        let res = ref_cache_server_c_721_run_poll_loop(opts, lsocks, upstr, log_wr);
        // _exit (not exit) is required in the forked child to skip atexit handlers.
        libc::_exit(if res == 0 { 0 } else { 1 });
    }

    let kids_vec = (*KIDS.0.get()).as_mut().unwrap();
    kids_vec[k as usize].pid = pid;
    nkids += 1;
    0
}

// original: start_upstream (htslib/ref_cache/main.c:245)
pub unsafe fn ref_cache_main_c_245_start_upstream(
    opts: &Options,
    pw: Option<&mut PollWrap>,
) -> c_int {
    let mut liveness_pipe = [-1, -1];

    // Make pipe so child can detect parent going away
    if libc::pipe(liveness_pipe.as_mut_ptr()) < 0 {
        eprintln!("Opening pipe: {}", std::io::Error::last_os_error());
        return -1;
    }

    let upstream_pid = libc::fork();
    if upstream_pid == -1 {
        eprintln!(
            "start_upstream couldn't fork: {}",
            std::io::Error::last_os_error()
        );
        libc::close(liveness_pipe[0]);
        libc::close(liveness_pipe[1]);
        return -1;
    }

    if upstream_pid == 0 {
        libc::close(liveness_pipe[0]);
        if ref_cache_main_c_168_set_up_child(opts, 0, 1, pw) != 0 {
            libc::_exit(1);
        }

        let upstream_sockets = (*UPSTREAM.0.get()).as_ref().unwrap().as_slice();
        let res = upstream::ref_cache_upstream_c_1157_run_upstream_handler(
            opts,
            upstream_sockets,
            liveness_pipe[1],
        );

        // _exit (not exit) is required in the forked child to skip atexit handlers.
        libc::_exit(if res == 0 { 0 } else { 1 });
    }

    let kids_vec = (*KIDS.0.get()).as_mut().unwrap();

    libc::close(liveness_pipe[1]);
    kids_vec[opts.max_kids as usize].pid = upstream_pid;
    kids_vec[opts.max_kids as usize].type_ = Child_type::CHLD_UPSTREAM;
    kids_vec[opts.max_kids as usize].log_rd = liveness_pipe[0];

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
    let c: u8 = b'*';
    let mut bytes;
    loop {
        // write() is a genuine syscall and is async-signal-safe (unlike Rust I/O).
        bytes = libc::write(sig_fds[1], (&c as *const u8).cast(), 1);
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
pub unsafe fn ref_cache_main_c_306_handle_sigchld(
    opts: &Options,
    mut pw: Option<&mut PollWrap>,
) -> c_int {
    let mut buffer = [0u8; 16];
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
        if bytes == 0 {
            eprintln!("EOF on signal fd #{}", sig_fds[0]);
        } else {
            eprintln!(
                "{} on signal fd #{}",
                std::io::Error::last_os_error(),
                sig_fds[0]
            );
        }
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
            eprintln!("waitpid: {}", std::io::Error::last_os_error());
            return -1;
        }

        if pid > 0 {
            let mut restart_upstream = false;
            if libc::WIFEXITED(status) {
                eprintln!(
                    "Child PID {} exited with status {}.",
                    pid,
                    libc::WEXITSTATUS(status)
                );
            } else if libc::WIFSIGNALED(status) {
                eprintln!(
                    "Child PID {} terminated by signal {}.",
                    pid,
                    libc::WTERMSIG(status)
                );
            } else {
                eprintln!("Child PID {pid} terminated");
            }
            {
                let mut i = 0;
                let kids_vec = (*KIDS.0.get()).as_mut().unwrap();
                while i < opts.max_kids as c_int + 1 {
                    if kids_vec[i as usize].pid == pid {
                        kids_vec[i as usize].pid = 0;
                        if kids_vec[i as usize].type_ == Child_type::CHLD_UPSTREAM {
                            restart_upstream = true;
                        } else {
                            nkids -= 1;
                        }
                        break;
                    }
                    i += 1;
                }
            }
            if restart_upstream
                && ref_cache_main_c_245_start_upstream(opts, pw.as_deref_mut()) != 0
            {
                return -1;
            }
        }
        if pid <= 0 {
            break;
        }
    }

    0
}

// original: init_poller (htslib/ref_cache/main.c:362)
// Returns an owned poller (`Box`), or `None` on failure.
pub unsafe fn ref_cache_main_c_362_init_poller(opts: &Options) -> Option<Box<PollWrap>> {
    let mut pw = poll_impl::ref_cache_poll_wrap_epoll_c_49_pw_init(opts.verbosity > 2)?;
    let mut k = 0usize;
    let kids_vec = (*KIDS.0.get()).as_mut().unwrap();

    while k < opts.max_kids as usize {
        let log_rd = kids_vec[k].log_rd;
        kids_vec[k].polled_rd = poll_impl::ref_cache_poll_wrap_epoll_c_78_pw_register(
            &mut pw,
            log_rd,
            Pw_fd_type::MAIN_LOG_RD,
            (PW_IN | PW_ERR | PW_HUP) as u32,
            0,
        );
        if kids_vec[k].polled_rd.is_none() {
            eprintln!(
                "Registering log pipe with poller: {}",
                std::io::Error::last_os_error()
            );
            break;
        }
        k += 1;
    }
    if k < opts.max_kids as usize {
        let mut j = 0usize;
        while j < nkids {
            if let Some(idx) = kids_vec[j].polled_rd {
                poll_impl::ref_cache_poll_wrap_epoll_c_126_pw_remove(&mut pw, idx, false);
            }
            j += 1;
        }
        return None;
    }

    polled_sig = poll_impl::ref_cache_poll_wrap_epoll_c_78_pw_register(
        &mut pw,
        sig_fds[0],
        Pw_fd_type::MAIN_SIG,
        (PW_IN | PW_HUP | PW_ERR) as u32,
        0,
    );
    if polled_sig.is_none() {
        eprintln!(
            "Registering signal pipe with poller: {}",
            std::io::Error::last_os_error()
        );
        let mut j = 0usize;
        while j < nkids {
            if let Some(idx) = kids_vec[j].polled_rd {
                poll_impl::ref_cache_poll_wrap_epoll_c_126_pw_remove(&mut pw, idx, false);
            }
            j += 1;
        }
        return None;
    }

    Some(pw)
}

// original: run_server_population (htslib/ref_cache/main.c:393)
pub unsafe fn ref_cache_main_c_393_run_server_population(
    opts: &Options,
    lsocks: &mut Listeners,
    logfiles: &mut Logfiles,
) -> c_int {
    let mut sigact: libc::sigaction = std::mem::zeroed();
    let mut logged = 0;

    /* Set up signal handler */
    sigact.sa_sigaction = ref_cache_main_c_283_sig_handler as usize;
    libc::sigemptyset(&mut sigact.sa_mask);
    sigact.sa_flags = libc::SA_NOCLDSTOP;
    if libc::sigaction(libc::SIGCHLD, &sigact, std::ptr::null_mut()) != 0 {
        eprintln!(
            "Setting up SIGCHLD handler: {}",
            std::io::Error::last_os_error()
        );
        return -1;
    }

    let mut pw = match ref_cache_main_c_362_init_poller(opts) {
        Some(pw) => pw,
        None => return -1,
    };

    /* Run poll loop */
    loop {
        let mut events: Vec<PwEvents> = vec![std::mem::zeroed(); MAX_EVENTS as usize];

        /* Make new children if necessary */
        while nkids < opts.max_kids as usize {
            if ref_cache_main_c_211_make_new_child(opts, lsocks, Some(&mut pw)) != 0 {
                if nkids == 0 {
                    eprintln!("Unable to make server processes, giving up.");
                    return -1;
                } else {
                    break;
                }
            }
        }

        /* Wait for events */
        let timeout = if logged != 0 { 100 } else { -1 };
        let ret = poll_impl::ref_cache_poll_wrap_epoll_c_120_pw_wait(&mut pw, &mut events, timeout);
        if ret < 0 {
            if *crate::htslib_rs::c_compat::__errno_location() != libc::EINTR {
                eprintln!("Waiting for poller: {}", std::io::Error::last_os_error());
                return -1;
            }
            continue;
        }

        if ret == 0 && logged != 0 {
            /* Flush the log */
            libc::fflush(opts.log);
            logged = 0;
            continue;
        }

        let mut e = 0;
        while e < ret {
            // The poller stashes the arena index of the woken item in `u64`.
            let item_idx = events[e as usize].u64 as usize;

            if polled_sig == Some(item_idx) {
                /* Got a signal */
                if ref_cache_main_c_306_handle_sigchld(opts, Some(&mut pw)) != 0 {
                    return -1;
                }
            } else if let Some(k) = {
                let kids_vec = (*KIDS.0.get()).as_ref().unwrap();
                (0..opts.max_kids as usize).find(|&j| kids_vec[j].polled_rd == Some(item_idx))
            } {
                /* Deal with log messages */
                let kids_vec = (*KIDS.0.get()).as_mut().unwrap();
                let kid = &mut kids_vec[k];
                assert!(kid.pid != 0);

                let mut buffer = [0u8; 65536];
                let mut bytes;
                loop {
                    bytes = libc::read(kid.log_rd, buffer.as_mut_ptr().cast(), buffer.len());
                    if !(bytes < 0
                        && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR)
                    {
                        break;
                    }
                }

                if bytes <= 0 {
                    if bytes == 0 {
                        eprintln!("EOF reading log fd #{}", kid.log_rd);
                    } else {
                        eprintln!(
                            "{} reading log fd #{}",
                            std::io::Error::last_os_error(),
                            kid.log_rd
                        );
                    }
                    libc::close(kid.log_rd);
                    kid.log_rd = -1;
                } else if opts.no_log == 0 {
                    if log_files::ref_cache_log_files_c_266_write_to_log(
                        logfiles,
                        opts,
                        &buffer[..bytes as usize],
                    ) < 0
                    {
                        return -1;
                    }
                    if opts.log_dir.is_some() {
                        logged = 1;
                    }
                }
            } else {
                eprintln!("Unexpected item index {item_idx} from poll");
                std::process::abort();
            }
            e += 1;
        }
    }
}

// original: daemonise (htslib/ref_cache/main.c:493)
pub unsafe fn ref_cache_main_c_493_daemonise(
    daemon_fds: &mut [c_int; 2],
    opts: &Options,
) -> c_int {
    let error_log_file = opts.error_log_file.as_deref();
    let mut fd_limit: libc::rlimit = std::mem::zeroed();
    let mut all_sigs: libc::sigset_t = std::mem::zeroed();

    if libc::getrlimit(libc::RLIMIT_NOFILE, &mut fd_limit) != 0 {
        eprintln!(
            "Getting max file descriptor count: {}",
            std::io::Error::last_os_error()
        );
        return -1;
    }

    // Close any open file descriptors above 3 (apart from the pipe passed in)
    let save_errno = *crate::htslib_rs::c_compat::__errno_location();
    let mut i = 3;
    while i < fd_limit.rlim_cur as c_int {
        if i != daemon_fds[0] && i != daemon_fds[1] {
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
        let mut sigact: libc::sigaction = std::mem::zeroed();
        sigact.sa_sigaction = libc::SIG_DFL;
        libc::sigemptyset(&mut sigact.sa_mask);
        if libc::sigaction(i, &sigact, std::ptr::null_mut()) != 0 {
            // Ideally, sigfillset() would only fill valid signals,
            // but sadly on Linux, at least, that doesn't appear to be
            // the case so we should expect to get EINVAL
            if *crate::htslib_rs::c_compat::__errno_location() != libc::EINVAL {
                eprintln!(
                    "Resetting signal handler {}: {}",
                    i,
                    std::io::Error::last_os_error()
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
        eprintln!("Setting signal mask: {}", std::io::Error::last_os_error());
        return -1;
    }

    let pid1 = libc::fork();
    if pid1 < 0 {
        eprintln!("Couldn't fork: {}", std::io::Error::last_os_error());
        return -1;
    }
    if pid1 != 0 {
        // Check that the daemon started up successfully, by waiting for
        // a message from the pipe in daemon_fds[].
        let mut msg = [0u8; 2];

        libc::close(daemon_fds[1]); // Writing end is for the daemon process
        let res = misc::ref_cache_misc_h_72_do_read_all(daemon_fds[0], &mut msg);
        libc::close(daemon_fds[0]);
        if res < 0 || msg != *b"ok" {
            eprintln!("Daemon failed to start, sorry.");
            // _exit (not exit) in the parent of the double-fork.
            libc::_exit(1);
        }
        if opts.error_log_file.is_none() && opts.no_log == 0 {
            // Last chance to tell the user...
            eprintln!("Error messages will be unavailable after this one.");
        }
        libc::_exit(0);
    }

    // Close the reading end of the pipe.  Writing end will be closed on
    // successful start.
    libc::close(daemon_fds[0]);
    daemon_fds[0] = -1;

    // Become session leader
    if libc::setsid() < 0 {
        eprintln!(
            "Couldn't become session leader: {}",
            std::io::Error::last_os_error()
        );
        libc::_exit(1);
    }

    // Fork again to make the daemon
    let pid2 = libc::fork();
    if pid2 < 0 {
        eprintln!("Couldn't fork: {}", std::io::Error::last_os_error());
        libc::_exit(1);
    }
    if pid2 != 0 {
        // Exit so the daemon is inherited by PID 1
        std::process::exit(0);
    }

    // Redirect stdin and stdout to /dev/null
    let devnull = libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
    if devnull < 0 {
        eprintln!("Opening /dev/null: {}", std::io::Error::last_os_error());
        std::process::exit(1);
    }
    if libc::dup2(devnull, 0) < 0 || libc::dup2(devnull, 1) < 0 || libc::dup2(devnull, 2) < 0 {
        eprintln!(
            "Redirecting stdin and stdout to /dev/null: {}",
            std::io::Error::last_os_error()
        );
        std::process::exit(1);
    }

    // Redirect stderr to error_log_file, or /dev/null if not set
    if let Some(path) = error_log_file {
        // freopen() needs a NUL-terminated path at the FFI boundary.
        let mut path_c = path.to_vec();
        path_c.push(0);
        if libc::freopen(
            path_c.as_ptr().cast(),
            c"a".as_ptr(),
            crate::htslib_rs::ref_cache::compat::stderr(),
        )
        .is_null()
        {
            /* Does it still exist? */
            eprintln!(
                "Couldn't redirect stderr to {}: {}",
                String::from_utf8_lossy(path),
                std::io::Error::last_os_error()
            );
            std::process::exit(1);
        }
    } else if libc::dup2(devnull, 2) < 0 {
        eprintln!(
            "Redirecting stderr to /dev/null: {}",
            std::io::Error::last_os_error()
        );
        std::process::exit(1);
    }

    if devnull > 2 {
        libc::close(devnull);
    }

    // Reset umask
    libc::umask(0);

    0
}

// original: get_systemd_listen_fds (htslib/ref_cache/main.c:633)
pub unsafe fn ref_cache_main_c_633_get_systemd_listen_fds(opts: &mut Options) -> c_int {
    let mut fd_limit: libc::rlimit = std::mem::zeroed();
    let our_pid = libc::getpid();
    let env_listen_pid = std::env::var("LISTEN_PID");
    let env_listen_fds = std::env::var("LISTEN_FDS");
    let env_listen_pid = match &env_listen_pid {
        Ok(v) => v.as_str(),
        Err(_) => {
            eprintln!("LISTEN_PID environment variable is not set");
            return -1;
        }
    };
    let env_listen_fds = match &env_listen_fds {
        Ok(v) => v.as_str(),
        Err(_) => {
            eprintln!("LISTEN_FDS environment variable is not set");
            return -1;
        }
    };
    if libc::getrlimit(libc::RLIMIT_NOFILE, &mut fd_limit) != 0 {
        eprintln!(
            "Getting max file descriptor count: {}",
            std::io::Error::last_os_error()
        );
        return -1;
    }

    // The whole value must be a non-empty decimal number (strtol with *end == 0).
    let listen_pid = env_listen_pid.parse::<i64>();
    match listen_pid {
        Ok(pid) if pid >= 0 && pid == our_pid as i64 => {}
        _ => {
            eprintln!("LISTEN_PID is incorrect");
            return -1;
        }
    }
    let listen_fds = match env_listen_fds.parse::<libc::c_long>() {
        Ok(fds)
            if fds > 0
                && fds <= c_int::MAX as libc::c_long
                && fds <= fd_limit.rlim_cur as libc::c_long
                    - options::FIRST_SD_LISTEN_FD as libc::c_long =>
        {
            fds
        }
        _ => {
            eprintln!("LISTEN_FDS is not valid");
            return -1;
        }
    };
    opts.listen_fds = listen_fds as c_int;
    std::env::remove_var("LISTEN_PID");
    std::env::remove_var("LISTEN_FDS");
    std::env::remove_var("LISTEN_FDNAMES");
    0
}

// original: add_match_addr (htslib/ref_cache/main.c:677)
pub unsafe fn ref_cache_main_c_677_add_match_addr(opts: &mut Options, addr_list: &[u8]) -> c_int {
    let addr_list_len = addr_list.len();
    let mut host_start = 0usize;
    let mut addrs: *mut libc::addrinfo;

    while host_start < addr_list_len {
        // Look for the IP address part (up to the first '/' or ',').
        let mut p = host_start;
        let host_len = addr_list[p..]
            .iter()
            .position(|&b| b == b'/' || b == b',')
            .unwrap_or(addr_list_len - p);
        if host_len >= NI_MAXHOST {
            eprintln!(
                "IP address \"{}...\" too long",
                String::from_utf8_lossy(&addr_list[host_start..(host_start + 20).min(addr_list_len)])
            );
            return -1;
        }
        // getaddrinfo() requires a NUL-terminated host string.
        let host = &addr_list[p..p + host_len];
        let mut host_c = host.to_vec();
        host_c.push(0);
        p += host_len;

        // Check for CIDR-notation netmask
        let (mut netmask_bits, mut netmask_end) = if p < addr_list_len && addr_list[p] == b'/' {
            // Length of "/<bits>" up to the next ',' or end.
            let netmask_len = addr_list[p..]
                .iter()
                .position(|&b| b == b',')
                .unwrap_or(addr_list_len - p);
            // Parse the decimal digits after the '/'.
            let nm = &addr_list[p + 1..addr_list_len];
            let digits_len = nm.iter().take_while(|&&b| b.is_ascii_digit()).count();
            if digits_len == 0 {
                eprintln!(
                    "Empty netmask for host \"{}\"",
                    String::from_utf8_lossy(host)
                );
                return -1;
            }
            let netmask_bits: c_ulong = std::str::from_utf8(&nm[..digits_len])
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(c_ulong::MAX);
            if netmask_bits > 128 {
                eprintln!(
                    "Netmask \"{}/{}\" too large",
                    String::from_utf8_lossy(host),
                    netmask_bits
                );
                return -1;
            }
            (netmask_bits, p + netmask_len)
        } else {
            (128, p)
        };
        if netmask_end < addr_list_len && addr_list[netmask_end] == b',' {
            netmask_end += 1;
        }

        // Use getaddrinfo() to convert the address to a struct sockaddr
        addrs = std::ptr::null_mut();
        let mut hints: libc::addrinfo = std::mem::zeroed();
        hints.ai_family = libc::AF_UNSPEC;
        hints.ai_socktype = libc::SOCK_STREAM;
        hints.ai_protocol = 0;
        hints.ai_canonname = std::ptr::null_mut();
        hints.ai_addr = std::ptr::null_mut();
        hints.ai_next = std::ptr::null_mut();
        hints.ai_flags = libc::AI_NUMERICHOST;
        let res = libc::getaddrinfo(host_c.as_ptr().cast(), std::ptr::null(), &hints, &mut addrs);
        if res != 0 {
            let gai = libc::gai_strerror(res);
            let gai_msg = if gai.is_null() {
                String::new()
            } else {
                String::from_utf8_lossy(std::slice::from_raw_parts(
                    gai.cast::<u8>(),
                    libc::strlen(gai),
                ))
                .into_owned()
            };
            eprintln!(
                "Couldn't resolve IP address \"{}\" : {}",
                String::from_utf8_lossy(host),
                gai_msg
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

            let mut ma = MatchAddr {
                family: (*(*addr).ai_addr).sa_family,
                mask_bytes: 0,
                mask: 0,
                addr: [0; 16],
            };

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
                eprintln!(
                    "Unexpected address type/length! Got {}/{} expected {}/{} or {}/{}",
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

            ma.mask_bytes = (netmask_bits / 8) as u8;
            if ma.mask_bytes as usize == alen {
                // This ensures ma->addr[ma->mask_bytes] is always valid
                // which makes checking easier
                ma.mask_bytes -= 1;
                ma.mask = 0xff;
            } else {
                ma.mask = ((0xff00u32 >> (netmask_bits & 7)) & 0xff) as u8;
            }

            // Copy masked IP address
            let mask_bytes = ma.mask_bytes as usize;
            let addr_src = std::slice::from_raw_parts(addrp, mask_bytes + 1);
            ma.addr[..mask_bytes].copy_from_slice(&addr_src[..mask_bytes]);
            ma.addr[mask_bytes] = addr_src[mask_bytes] & ma.mask;

            opts.match_addrs_storage.push(ma);
            addr = (*addr).ai_next;
        }
        libc::freeaddrinfo(addrs);
        host_start = netmask_end;
    }
    0
}

fn compare_match_addrs(a: &MatchAddr, b: &MatchAddr) -> Ordering {
    let ip_order = if libc::AF_INET < libc::AF_INET6 {
        Ordering::Greater
    } else {
        Ordering::Less
    };

    if a.family != b.family {
        return if a.family < b.family {
            ip_order.reverse()
        } else {
            ip_order
        };
    }
    a.addr.cmp(&b.addr)
}

// original: sort_match_addrs (htslib/ref_cache/main.c:826)
pub fn ref_cache_main_c_826_sort_match_addrs(opts: &mut Options) {
    let mut ip6_seen = 0;

    if opts.match_addrs_storage.is_empty() {
        return;
    }

    opts.match_addrs_storage.sort_by(compare_match_addrs);
    opts.match_addrs_storage.dedup();

    for (i, ma) in opts.match_addrs_storage.iter().enumerate() {
        if ip6_seen == 0 && ma.family as c_int == libc::AF_INET6 {
            opts.first_ip6 = i;
            ip6_seen = 1;
            break;
        }
    }
    if ip6_seen == 0 {
        opts.first_ip6 = opts.match_addrs_storage.len();
    }
}

// original: usage (htslib/ref_cache/main.c:857)
pub unsafe fn ref_cache_main_c_857_usage(prog: &[u8], help: c_int, opts: &Options) {
    eprintln!("Usage: {} [options] -d <dir>", String::from_utf8_lossy(prog));
    if help != 0 {
        // Render the optional "[<upstream url>]" suffix as plain Rust strings.
        let (upstream_open, upstream_url, upstream_close) = if let Some(url) = &opts.upstream_url {
            let open = if url.len() > 40 {
                "\n             ["
            } else {
                " ["
            };
            (
                open.to_string(),
                String::from_utf8_lossy(url).into_owned(),
                "]".to_string(),
            )
        } else {
            (String::new(), String::new(), String::new())
        };
        eprint!(
            "Options:\n  -b         Run in background as a daemon\n  -d <dir>   Directory for cached reference files\n  -h         Show help\n  -l <dir>   Directory for log files.  Log to stdout if not set and running in\n             foreground\n  -L         Don't log\n  -m <list>  Only respond to connections from these networks\n  -n <1-4>   Number of server processes to run [{}]\n  -p <num>   Port number to listen on [{}]\n  -r <num>   Number of request log files to keep [{}]\n  -R <num>   Maximum size of a single request log file (MiB) [{}]\n  -s         Run as a systemd socket service\n  -u <url>   URL for upstream server{}{}{}\n  -U         Only serve local files, turn off upstream\n  -v         Turn on debugging output\n",
            opts.max_kids as libc::c_uint,
            opts.port as libc::c_uint,
            opts.nlogs as libc::c_uint,
            (opts.max_log_sz >> 20) as libc::c_longlong,
            upstream_open,
            upstream_url,
            upstream_close,
        );
    }
}

// original: get_opt_val (htslib/ref_cache/main.c:889)
pub unsafe fn ref_cache_main_c_889_get_opt_val(
    arg: &[u8],
    prog: &[u8],
    opt: &[u8],
    min: u16,
    max: u16,
    badarg: &mut c_int,
) -> u16 {
    let lmin = min as libc::c_long;
    let lmax = max as libc::c_long;

    // Parse like strtol(arg, &end, 0): base auto-detected from prefix, and the
    // whole (non-empty) string must be consumed.
    let parsed: Option<libc::c_long> = (|| {
        if arg.is_empty() {
            return None;
        }
        let mut rest = arg;
        let mut neg = false;
        match rest.first() {
            Some(b'+') => rest = &rest[1..],
            Some(b'-') => {
                neg = true;
                rest = &rest[1..];
            }
            _ => {}
        }
        let (digits, radix) = if rest.len() >= 2 && (rest[0] == b'0') && (rest[1] | 0x20 == b'x') {
            (&rest[2..], 16u32)
        } else if rest.len() >= 1 && rest[0] == b'0' {
            (rest, 8u32)
        } else {
            (rest, 10u32)
        };
        if digits.is_empty() {
            return None;
        }
        let s = std::str::from_utf8(digits).ok()?;
        let v = libc::c_long::from_str_radix(s, radix).ok()?;
        Some(if neg { -v } else { v })
    })();

    let val = match parsed {
        Some(v) => v,
        None => {
            eprintln!(
                "{} : {} option value \"{}\" is not a number",
                String::from_utf8_lossy(prog),
                String::from_utf8_lossy(opt),
                String::from_utf8_lossy(arg),
            );
            *badarg = 1;
            return 0;
        }
    };
    if val < lmin || val > lmax {
        eprintln!(
            "{} : {} option value \"{}\" should be between {} and {}",
            String::from_utf8_lossy(prog),
            String::from_utf8_lossy(opt),
            String::from_utf8_lossy(arg),
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
    let mut opts: Options;
    let mut lsocks: Option<Box<Listeners>> = None;
    let mut logfiles: Option<Box<Logfiles>> = None;
    let mut badarg = 0;
    let mut retval = 1; // EXIT_FAILURE
    let mut show_help = 0;
    let mut daemon_pipe = [-1, -1];
    let ip_ranges_all: &[u8] = b"0.0.0.0/0,::/0";
    let ip_ranges_localhost: &[u8] = b"127.0.0.0/8,::1/128";
    let ip_ranges_default: &[u8] = b"10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,fc00::/7,fe80::/10";

    // argv[0] as a byte slice (for usage() and change_name()).
    let prog0 = {
        let p = *argv.add(0);
        std::slice::from_raw_parts(p.cast::<u8>(), libc::strlen(p))
    };

    #[allow(clippy::never_loop)]
    'main_body: loop {
        /* Copy argv[0] for change_name() */
        argv0 = (*argv.add(0)).cast::<u8>();
        argv0_len = prog0.len();

        /* Options */

        opts = Options::default();

        opts.port = 8000;
        opts.cache_dir = None;
        opts.cache_fd = -1;
        opts.max_kids = 1;
        opts.verbosity = 0;
        opts.log_dir = None;
        opts.log = crate::htslib_rs::ref_cache::compat::stdout();
        opts.error_log_file = None;
        opts.nlogs = 5;
        opts.max_log_sz = 10 << 20;
        opts.daemon = DaemonType::not_a_daemon;
        opts.no_log = 0;
        opts.upstream_url = Some(b"https://www.ebi.ac.uk/ena/cram/md5/".to_vec());

        loop {
            let c =
                crate::htslib_rs::ref_cache::compat::getopt_(argc, argv, b"be:d:hl:Lm:n:p:r:R:su:Uv");
            if c < 0 {
                break;
            }
            match c as u8 as char {
                'b' => {
                    if opts.daemon != DaemonType::not_a_daemon {
                        eprintln!("The -b and -s options cannot be used together");
                        badarg = 1;
                    } else {
                        opts.daemon = DaemonType::sysv_daemon;
                    }
                }
                'e' => {
                    opts.error_log_file =
                        crate::htslib_rs::ref_cache::compat::optarg().map(|s| s.to_vec())
                }
                'd' => {
                    opts.cache_dir =
                        crate::htslib_rs::ref_cache::compat::optarg().map(|s| s.to_vec())
                }
                'h' => show_help = 1,
                'l' => {
                    opts.log_dir = crate::htslib_rs::ref_cache::compat::optarg().map(|s| s.to_vec());
                    opts.no_log = 0;
                }
                'L' => {
                    if opts.log_dir.is_none() {
                        opts.no_log = 1;
                    }
                }
                'm' => {
                    let arg = crate::htslib_rs::ref_cache::compat::optarg().unwrap_or(b"");
                    let to_add: &[u8] = match arg {
                        b"all" => ip_ranges_all,
                        b"default" => ip_ranges_default,
                        b"localhost" => ip_ranges_localhost,
                        other => other,
                    };
                    if ref_cache_main_c_677_add_match_addr(&mut opts, to_add) != 0 {
                        break 'main_body;
                    }
                }
                'n' => {
                    opts.max_kids = ref_cache_main_c_889_get_opt_val(
                        crate::htslib_rs::ref_cache::compat::optarg().unwrap_or(b""),
                        prog0,
                        b"-n",
                        1,
                        4,
                        &mut badarg,
                    )
                }
                'p' => {
                    opts.port = ref_cache_main_c_889_get_opt_val(
                        crate::htslib_rs::ref_cache::compat::optarg().unwrap_or(b""),
                        prog0,
                        b"-p",
                        1,
                        65535,
                        &mut badarg,
                    )
                }
                'r' => {
                    opts.nlogs = ref_cache_main_c_889_get_opt_val(
                        crate::htslib_rs::ref_cache::compat::optarg().unwrap_or(b""),
                        prog0,
                        b"-r",
                        1,
                        100,
                        &mut badarg,
                    )
                }
                'R' => {
                    opts.max_log_sz = ref_cache_main_c_889_get_opt_val(
                        crate::htslib_rs::ref_cache::compat::optarg().unwrap_or(b""),
                        prog0,
                        b"-R",
                        1,
                        1000,
                        &mut badarg,
                    ) as libc::off_t
                        * (1 << 20)
                }
                's' => {
                    if opts.daemon != DaemonType::not_a_daemon {
                        eprintln!("The -b and -s options cannot be used together");
                        badarg = 1;
                    } else {
                        opts.daemon = DaemonType::systemd_socket_service;
                    }
                }
                'u' => {
                    opts.upstream_url =
                        crate::htslib_rs::ref_cache::compat::optarg().map(|s| s.to_vec())
                }
                'U' => opts.upstream_url = None,
                'v' => opts.verbosity += 1,
                _ => {
                    ref_cache_main_c_857_usage(prog0, 0, &opts);
                    break 'main_body;
                }
            }
        }

        if show_help != 0 {
            ref_cache_main_c_857_usage(prog0, 1, &opts);
            retval = 0; // EXIT_SUCCESS
            break 'main_body;
        }

        if badarg != 0 {
            break 'main_body;
        }

        if opts.cache_dir.is_none() {
            ref_cache_main_c_857_usage(prog0, 0, &opts);
            break 'main_body;
        }

        if opts.match_addrs_storage.is_empty()
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
                retval = if res < 0 { 1 } else { 0 };
                break 'main_body;
            }
        }

        /* Turn into a daemon, if requested */
        match opts.daemon {
            DaemonType::sysv_daemon => {
                if opts.log_dir.is_none() && opts.no_log == 0 {
                    eprintln!(
                        "Warning: Running as a daemon without setting a request log directory!"
                    );
                    eprintln!("Request logs will be unavailable.");
                }
                if opts.error_log_file.is_none() && opts.no_log == 0 {
                    eprintln!("Warning: Running as a daemon without setting an error log file!");
                }

                if libc::pipe(daemon_pipe.as_mut_ptr()) < 0 {
                    eprintln!("Opening pipe: {}", std::io::Error::last_os_error());
                    break 'main_body;
                }
                if ref_cache_main_c_493_daemonise(&mut daemon_pipe, &opts) != 0 {
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
        if logfiles.is_none() {
            break 'main_body;
        }

        /* Open cache directory */
        let cache_dir = opts.cache_dir.as_deref().unwrap_or(b"");
        // open() needs a NUL-terminated path at the FFI boundary.
        let mut cache_dir_c = cache_dir.to_vec();
        cache_dir_c.push(0);
        opts.cache_fd = libc::open(
            cache_dir_c.as_ptr().cast(),
            libc::O_RDONLY | libc::O_DIRECTORY,
        );
        if opts.cache_fd < 0 {
            eprintln!(
                "Couldn't open directory {}: {}",
                String::from_utf8_lossy(cache_dir),
                std::io::Error::last_os_error()
            );
            break 'main_body;
        }

        /* Allocate data structures and plumbing for child processes */

        if ref_cache_main_c_107_init_children(&opts) != 0 {
            break 'main_body;
        }

        /* Start the upstream handler if needed */

        if opts.upstream_url.is_some() {
            if ref_cache_main_c_245_start_upstream(&opts, None) != 0 {
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
        if lsocks.is_none() {
            eprintln!("Couldn't start up.  Sorry.");
            break 'main_body;
        }

        if opts.daemon == DaemonType::sysv_daemon {
            if misc::ref_cache_misc_h_55_do_write_all(daemon_pipe[1], b"ok") < 0 {
                eprintln!(
                    "Couldn't report successful start back to parent: {}",
                    std::io::Error::last_os_error()
                );
                break 'main_body;
            }
            libc::close(daemon_pipe[1]);
            daemon_pipe[1] = -1;
        } else if let Some(path) = opts.error_log_file.as_deref() {
            // freopen() needs a NUL-terminated path at the FFI boundary.
            let mut path_c = path.to_vec();
            path_c.push(0);
            if libc::freopen(
                path_c.as_ptr().cast(),
                c"a".as_ptr(),
                crate::htslib_rs::ref_cache::compat::stderr(),
            )
            .is_null()
            {
                /* Does it still exist? */
                eprintln!(
                    "Couldn't redirect stderr to {}: {}",
                    String::from_utf8_lossy(path),
                    std::io::Error::last_os_error()
                );
                break 'main_body;
            }
        }

        /* Run the servers */

        let res = ref_cache_main_c_393_run_server_population(
            &opts,
            lsocks.as_deref_mut().unwrap(),
            logfiles.as_deref_mut().unwrap(),
        );
        if res == -1 {
            eprintln!("Server died.");
        }
        retval = if res == 0 { 0 } else { 1 };

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
    if let Some(lsocks) = lsocks {
        listener::ref_cache_listener_c_235_close_listen_sockets(lsocks);
    }
    log_files::ref_cache_log_files_c_134_close_logs(logfiles);
    *KIDS.0.get() = None;
    *UPSTREAM.0.get() = None;
    retval
}
