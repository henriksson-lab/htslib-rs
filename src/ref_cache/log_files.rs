use super::options::Options;
use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr::NonNull;

const LOG_NAME_LEN: usize = 80;

pub struct Logfile {
    name: CString,
    size: libc::off_t,
}

struct DirHandle(NonNull<libc::DIR>);

impl DirHandle {
    unsafe fn open(path: *const c_char) -> Option<Self> {
        NonNull::new(libc::opendir(path)).map(Self)
    }

    fn as_ptr(&self) -> *mut libc::DIR {
        self.0.as_ptr()
    }

    unsafe fn fd(&self) -> c_int {
        libc::dirfd(self.as_ptr())
    }
}

impl Drop for DirHandle {
    fn drop(&mut self) {
        unsafe {
            libc::closedir(self.as_ptr());
        }
    }
}

enum LogDestination {
    Unopened,
    Stdout,
    File(NonNull<libc::FILE>),
}

impl LogDestination {
    fn as_ptr(&self) -> *mut libc::FILE {
        match self {
            Self::Unopened => std::ptr::null_mut(),
            Self::Stdout => unsafe { crate::htslib_rs::ref_cache::compat::stdout() },
            Self::File(file) => file.as_ptr(),
        }
    }
}

impl Drop for LogDestination {
    fn drop(&mut self) {
        if let Self::File(file) = self {
            unsafe {
                libc::fclose(file.as_ptr());
            }
        }
    }
}

// original: Logfiles (htslib/ref_cache/log_files.c:53)
pub struct Logfiles {
    dir_handle: Option<DirHandle>,
    curr_log: LogDestination,
    logs: Vec<Logfile>,
    log_dir_fd: c_int,
}

// original: rotate_logs (htslib/ref_cache/log_files.c:69)
pub unsafe fn ref_cache_log_files_c_69_rotate_logs(
    logfiles: *mut Logfiles,
    opts: *const Options,
) -> c_int {
    let now = libc::time(std::ptr::null_mut());
    let (year, month, day, hour, minute, second, _) =
        crate::htslib_rs::c_compat::unix_time_utc_parts(now);
    let mut log_fd = -1;
    let mut name = CString::new("ref_cache.log").unwrap();

    for i in 0..99u32 {
        name = CString::new(format!(
            "ref_cache_{year:04}{month:02}{day:02}{hour:02}{minute:02}{second:02}_{i:02}.log"
        ))
        .unwrap();
        debug_assert!(name.as_bytes_with_nul().len() <= LOG_NAME_LEN);

        loop {
            log_fd = libc::openat(
                (*logfiles).log_dir_fd,
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
                0o644,
            );
            if !(log_fd < 0 && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR) {
                break;
            }
        }
        if log_fd >= 0 {
            break;
        }
        if *crate::htslib_rs::c_compat::__errno_location() != libc::EEXIST {
            break;
        }
    }

    if log_fd < 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"Couldn't open %s/%s for writing: %s\n".as_ptr(),
            (*opts).log_dir,
            name.as_ptr(),
            libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
        );
        return -1;
    }

    let file = libc::fdopen(log_fd, c"w".as_ptr());
    let Some(file) = NonNull::new(file) else {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"Couldn't fdopen %s/%s : %s\n".as_ptr(),
            (*opts).log_dir,
            name.as_ptr(),
            libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
        );
        libc::close(log_fd);
        libc::unlinkat((*logfiles).log_dir_fd, name.as_ptr(), 0);
        return -1;
    };

    debug_assert!((*opts).nlogs > 0);
    debug_assert!((*logfiles).logs.capacity() > (*opts).nlogs as usize);
    let keep = (*opts).nlogs as usize - 1;
    for old_log in (*logfiles).logs.iter().skip(keep) {
        if libc::unlinkat((*logfiles).log_dir_fd, old_log.name.as_ptr(), 0) != 0 {
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Warning: Couldn't remove old log file %s/%s: %s\n".as_ptr(),
                (*opts).log_dir,
                old_log.name.as_ptr(),
                libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
            );
        }
    }

    (*logfiles).logs.truncate(keep);
    (*logfiles).logs.insert(0, Logfile { name, size: 0 });
    (*logfiles).curr_log = LogDestination::File(file);

    0
}

// original: close_logs (htslib/ref_cache/log_files.c:134)
pub unsafe fn ref_cache_log_files_c_134_close_logs(logfiles: *mut Logfiles) {
    if logfiles.is_null() {
        return;
    }

    drop(Box::from_raw(logfiles));
}

// original: open_logs (htslib/ref_cache/log_files.c:148)
pub unsafe fn ref_cache_log_files_c_148_open_logs(opts: *const Options) -> *mut Logfiles {
    let mut logfiles = Box::new(Logfiles {
        dir_handle: None,
        curr_log: LogDestination::Unopened,
        logs: Vec::new(),
        log_dir_fd: -1,
    });

    if (*opts).log_dir.is_null() {
        logfiles.curr_log = LogDestination::Stdout;
        return Box::into_raw(logfiles);
    }

    debug_assert!((*opts).nlogs > 0);

    logfiles.dir_handle = DirHandle::open((*opts).log_dir);

    if logfiles.dir_handle.is_none() {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"Couldn't open directory %s: %s\n".as_ptr(),
            (*opts).log_dir,
            libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
        );
        return std::ptr::null_mut();
    }

    logfiles.logs = Vec::with_capacity((*opts).nlogs as usize + 1);
    logfiles.log_dir_fd = logfiles.dir_handle.as_ref().unwrap().fd();
    if logfiles.log_dir_fd < 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"Couldn't get descriptor for %s : %s".as_ptr(),
            (*opts).log_dir,
            libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
        );
        return std::ptr::null_mut();
    }

    *crate::htslib_rs::c_compat::__errno_location() = 0;
    loop {
        let ent = libc::readdir(logfiles.dir_handle.as_ref().unwrap().as_ptr());
        if ent.is_null() {
            break;
        }
        let mut dt = [0 as c_char; 15];
        let mut idx = 0u32;
        let mut suff = [0 as c_char; 8];
        let mut st: libc::stat = std::mem::zeroed();

        if libc::sscanf(
            (*ent).d_name.as_ptr(),
            c"ref_cache_%14[0-9]_%u.%7s".as_ptr(),
            dt.as_mut_ptr(),
            &mut idx as *mut u32,
            suff.as_mut_ptr(),
        ) != 0
            && (libc::strcmp(suff.as_ptr(), c"log".as_ptr()) == 0
                || libc::strcmp(suff.as_ptr(), c"log.gz".as_ptr()) == 0)
        {
            let entry_name = CStr::from_ptr((*ent).d_name.as_ptr());
            if entry_name.to_bytes_with_nul().len() > LOG_NAME_LEN {
                libc::abort();
            }
            if libc::fstatat(
                logfiles.log_dir_fd,
                entry_name.as_ptr(),
                &mut st,
                libc::AT_SYMLINK_NOFOLLOW,
            ) != 0
            {
                libc::fprintf(
                    crate::htslib_rs::ref_cache::compat::stderr(),
                    c"Warning: Couldn't stat %s/%s : %s\n".as_ptr(),
                    (*opts).log_dir,
                    entry_name.as_ptr(),
                    libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
                );
                continue;
            }
            if (st.st_mode & libc::S_IFMT) == libc::S_IFREG {
                logfiles.logs.push(Logfile {
                    name: entry_name.to_owned(),
                    size: st.st_size,
                });
            }
        }
    }

    if !logfiles.logs.is_empty() {
        logfiles
            .logs
            .sort_by(|a, b| b.name.as_bytes().cmp(a.name.as_bytes()));
    }

    let logfiles = Box::into_raw(logfiles);
    if ref_cache_log_files_c_69_rotate_logs(logfiles, opts) < 0 {
        ref_cache_log_files_c_134_close_logs(logfiles);
        return std::ptr::null_mut();
    }

    logfiles
}

const fn build_needs_escape() -> [u8; 256] {
    let mut vals = [0; 256];
    let mut i = 0;
    while i < 256 {
        vals[i] = if (i < 32 && i != b'\n' as usize) || i == b'\\' as usize || i >= 127 {
            1
        } else {
            0
        };
        i += 1;
    }
    vals
}

static NEEDS_ESCAPE: [u8; 256] = build_needs_escape();

// original: write_to_log (htslib/ref_cache/log_files.c:266)
pub unsafe fn ref_cache_log_files_c_266_write_to_log(
    logfiles: *mut Logfiles,
    opts: *const Options,
    msg: *const c_char,
    len: usize,
) -> c_int {
    let mut written = 0usize;
    let curr_log = (*logfiles).curr_log.as_ptr();
    if curr_log.is_null() {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"No log file is open\n".as_ptr(),
        );
        return -1;
    }

    while written < len {
        let mut ok = written;
        while ok < len && NEEDS_ESCAPE[*msg.add(ok) as u8 as usize] == 0 {
            ok += 1;
        }

        if ok > written {
            let wrote = libc::fwrite(msg.add(written).cast(), 1, ok - written, curr_log);
            if wrote != ok - written {
                break;
            }
        }
        if ok < len && NEEDS_ESCAPE[*msg.add(ok) as u8 as usize] != 0 {
            if *msg.add(ok) as u8 == b'\\' {
                if libc::fprintf(curr_log, c"\\\\".as_ptr()) < 0 {
                    break;
                }
            } else if libc::fprintf(curr_log, c"\\x%02x".as_ptr(), *msg.add(ok) as u8 as c_int) < 0
            {
                break;
            }
            ok += 1;
        }
        written = ok;
    }

    if written < len {
        if !(*opts).log_dir.is_null() {
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Error writing to %s/%s : %s\n".as_ptr(),
                (*opts).log_dir,
                (*logfiles)
                    .logs
                    .first()
                    .map_or(c"".as_ptr(), |log| log.name.as_ptr()),
                libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
            );
        } else {
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Error writing to stdout: %s\n".as_ptr(),
                libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
            );
        }
        return -1;
    }
    if !(*opts).log_dir.is_null() {
        if let Some(current_log) = (*logfiles).logs.first_mut() {
            current_log.size += written as libc::off_t;
            if current_log.size > (*opts).max_log_sz
                && ref_cache_log_files_c_69_rotate_logs(logfiles, opts) != 0
            {
                return -1;
            }
        }
    }
    0
}
