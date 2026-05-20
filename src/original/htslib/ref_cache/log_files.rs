use super::options::Options;
use std::ffi::{c_char, c_int, c_void};

const LOG_NAME_LEN: usize = 80;

#[repr(C)]
pub struct Logfile {
    pub name: [c_char; LOG_NAME_LEN],
    pub size: libc::off_t,
}

// original: Logfiles (htslib/ref_cache/log_files.c:53)
#[repr(C)]
pub struct Logfiles {
    pub dir_handle: *mut libc::DIR,
    pub curr_log: *mut libc::FILE,
    pub logs: *mut Logfile,
    pub nlogs: usize,
    pub sz: usize,
    pub log_dir_fd: c_int,
}

// original: log_compare (htslib/ref_cache/log_files.c:62)
pub unsafe extern "C" fn ref_cache_log_files_c_62_log_compare(
    av: *const c_void,
    bv: *const c_void,
) -> c_int {
    let a = av.cast::<Logfile>();
    let b = bv.cast::<Logfile>();
    libc::strcmp((*b).name.as_ptr(), (*a).name.as_ptr())
}

// original: rotate_logs (htslib/ref_cache/log_files.c:69)
pub unsafe fn ref_cache_log_files_c_69_rotate_logs(
    logfiles: *mut Logfiles,
    opts: *const Options,
) -> c_int {
    let mut name = [0 as c_char; LOG_NAME_LEN];
    let mut now = libc::time(std::ptr::null_mut());
    let gmt = libc::gmtime(&mut now);
    let mut log_fd = -1;
    let mut file: *mut libc::FILE = std::ptr::null_mut();

    for i in 0..99u32 {
        libc::snprintf(
            name.as_mut_ptr(),
            LOG_NAME_LEN,
            b"ref_cache_%04d%02d%02d%02d%02d%02d_%02u.log\0"
                .as_ptr()
                .cast(),
            (*gmt).tm_year + 1900,
            (*gmt).tm_mon + 1,
            (*gmt).tm_mday,
            (*gmt).tm_hour,
            (*gmt).tm_min,
            (*gmt).tm_sec,
            i,
        );

        loop {
            log_fd = libc::openat(
                (*logfiles).log_dir_fd,
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
                0o644,
            );
            if !(log_fd < 0 && *libc::__errno_location() == libc::EINTR) {
                break;
            }
        }
        if log_fd >= 0 {
            break;
        }
        if *libc::__errno_location() != libc::EEXIST {
            break;
        }
    }

    if log_fd < 0 {
        libc::fprintf(
            libc::stderr,
            b"Couldn't open %s/%s for writing: %s\n\0".as_ptr().cast(),
            (*opts).log_dir,
            name.as_ptr(),
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }

    file = libc::fdopen(log_fd, b"w\0".as_ptr().cast());
    if file.is_null() {
        libc::fprintf(
            libc::stderr,
            b"Couldn't fdopen %s/%s : %s\n\0".as_ptr().cast(),
            (*opts).log_dir,
            name.as_ptr(),
            libc::strerror(*libc::__errno_location()),
        );
        libc::close(log_fd);
        libc::unlinkat((*logfiles).log_dir_fd, name.as_ptr(), 0);
        return -1;
    }

    debug_assert!((*opts).nlogs > 0);
    debug_assert!((*logfiles).sz > (*opts).nlogs as usize);
    let mut i = (*opts).nlogs as usize - 1;
    while i < (*logfiles).nlogs {
        if libc::unlinkat(
            (*logfiles).log_dir_fd,
            (*(*logfiles).logs.add(i)).name.as_ptr(),
            0,
        ) != 0
        {
            libc::fprintf(
                libc::stderr,
                b"Warning: Couldn't remove old log file %s/%s: %s\n\0"
                    .as_ptr()
                    .cast(),
                (*opts).log_dir,
                (*(*logfiles).logs.add(i)).name.as_ptr(),
                libc::strerror(*libc::__errno_location()),
            );
        }
        i += 1;
    }

    if (*logfiles).nlogs > (*opts).nlogs as usize - 1 {
        (*logfiles).nlogs = (*opts).nlogs as usize - 1;
    }

    if (*logfiles).nlogs > 0 {
        libc::memmove(
            (*logfiles).logs.add(1).cast(),
            (*logfiles).logs.cast(),
            (*logfiles).nlogs * std::mem::size_of::<Logfile>(),
        );
    }
    libc::memcpy(
        (*(*logfiles).logs).name.as_mut_ptr().cast(),
        name.as_ptr().cast(),
        LOG_NAME_LEN,
    );
    (*(*logfiles).logs).size = 0;
    (*logfiles).nlogs += 1;

    if !(*logfiles).curr_log.is_null() && (*logfiles).curr_log != libc::stdout {
        libc::fclose((*logfiles).curr_log);
    }
    (*logfiles).curr_log = file;

    0
}

// original: close_logs (htslib/ref_cache/log_files.c:134)
pub unsafe fn ref_cache_log_files_c_134_close_logs(logfiles: *mut Logfiles) {
    if logfiles.is_null() {
        return;
    }

    if !(*logfiles).dir_handle.is_null() {
        libc::closedir((*logfiles).dir_handle);
    }
    if !(*logfiles).curr_log.is_null() && (*logfiles).curr_log != libc::stdout {
        libc::fclose((*logfiles).curr_log);
    }
    libc::free((*logfiles).logs.cast());
    libc::free(logfiles.cast());
}

// original: open_logs (htslib/ref_cache/log_files.c:148)
pub unsafe fn ref_cache_log_files_c_148_open_logs(opts: *const Options) -> *mut Logfiles {
    let logfiles = libc::calloc(1, std::mem::size_of::<Logfiles>()).cast::<Logfiles>();

    if logfiles.is_null() {
        libc::perror(b"Allocating logfiles\0".as_ptr().cast());
        return std::ptr::null_mut();
    }

    (*logfiles).logs = std::ptr::null_mut();

    if (*opts).log_dir.is_null() {
        (*logfiles).dir_handle = std::ptr::null_mut();
        (*logfiles).curr_log = libc::stdout;
        return logfiles;
    }

    debug_assert!((*opts).nlogs > 0);

    (*logfiles).curr_log = std::ptr::null_mut();
    (*logfiles).dir_handle = libc::opendir((*opts).log_dir);

    if (*logfiles).dir_handle.is_null() {
        libc::fprintf(
            libc::stderr,
            b"Couldn't open directory %s: %s\n\0".as_ptr().cast(),
            (*opts).log_dir,
            libc::strerror(*libc::__errno_location()),
        );
        ref_cache_log_files_c_134_close_logs(logfiles);
        return std::ptr::null_mut();
    }

    (*logfiles).sz = (*opts).nlogs as usize + 1;
    (*logfiles).logs =
        libc::calloc((*logfiles).sz, std::mem::size_of::<Logfile>()).cast::<Logfile>();
    if (*logfiles).logs.is_null() {
        libc::perror(std::ptr::null());
        ref_cache_log_files_c_134_close_logs(logfiles);
        return std::ptr::null_mut();
    }

    (*logfiles).log_dir_fd = libc::dirfd((*logfiles).dir_handle);
    if (*logfiles).log_dir_fd < 0 {
        libc::fprintf(
            libc::stderr,
            b"Couldn't get descriptor for %s : %s\0".as_ptr().cast(),
            (*opts).log_dir,
            libc::strerror(*libc::__errno_location()),
        );
        ref_cache_log_files_c_134_close_logs(logfiles);
        return std::ptr::null_mut();
    }

    *libc::__errno_location() = 0;
    loop {
        let ent = libc::readdir((*logfiles).dir_handle);
        if ent.is_null() {
            break;
        }
        let mut dt = [0 as c_char; 15];
        let mut idx = 0u32;
        let mut suff = [0 as c_char; 8];
        let mut st: libc::stat = std::mem::zeroed();

        if libc::sscanf(
            (*ent).d_name.as_ptr(),
            b"ref_cache_%14[0-9]_%u.%7s\0".as_ptr().cast(),
            dt.as_mut_ptr(),
            &mut idx as *mut u32,
            suff.as_mut_ptr(),
        ) != 0
            && (libc::strcmp(suff.as_ptr(), b"log\0".as_ptr().cast()) == 0
                || libc::strcmp(suff.as_ptr(), b"log.gz\0".as_ptr().cast()) == 0)
        {
            if (*logfiles).nlogs == (*logfiles).sz {
                let new_sz = (*logfiles).sz * 2;
                let new_logs = libc::realloc(
                    (*logfiles).logs.cast(),
                    new_sz * std::mem::size_of::<Logfile>(),
                )
                .cast::<Logfile>();
                if new_logs.is_null() {
                    libc::perror(std::ptr::null());
                    ref_cache_log_files_c_134_close_logs(logfiles);
                    return std::ptr::null_mut();
                }
                libc::memset(
                    new_logs.add((*logfiles).nlogs).cast(),
                    0,
                    (new_sz - (*logfiles).nlogs) * std::mem::size_of::<Logfile>(),
                );
                (*logfiles).logs = new_logs;
                (*logfiles).sz = new_sz;
            }

            let l = libc::snprintf(
                (*(*logfiles).logs.add((*logfiles).nlogs)).name.as_mut_ptr(),
                LOG_NAME_LEN,
                b"%s\0".as_ptr().cast(),
                (*ent).d_name.as_ptr(),
            );
            if l >= LOG_NAME_LEN as c_int {
                libc::abort();
            }
            if libc::fstatat(
                (*logfiles).log_dir_fd,
                (*ent).d_name.as_ptr(),
                &mut st,
                libc::AT_SYMLINK_NOFOLLOW,
            ) != 0
            {
                libc::fprintf(
                    libc::stderr,
                    b"Warning: Couldn't stat %s/%s : %s\n\0".as_ptr().cast(),
                    (*opts).log_dir,
                    (*ent).d_name.as_ptr(),
                    libc::strerror(*libc::__errno_location()),
                );
                continue;
            }
            if (st.st_mode & libc::S_IFMT) == libc::S_IFREG {
                (*(*logfiles).logs.add((*logfiles).nlogs)).size = st.st_size;
                (*logfiles).nlogs += 1;
            }
        }
    }

    if (*logfiles).nlogs > 0 {
        libc::qsort(
            (*logfiles).logs.cast(),
            (*logfiles).nlogs,
            std::mem::size_of::<Logfile>(),
            Some(ref_cache_log_files_c_62_log_compare),
        );
    }

    if ref_cache_log_files_c_69_rotate_logs(logfiles, opts) < 0 {
        ref_cache_log_files_c_134_close_logs(logfiles);
        return std::ptr::null_mut();
    }

    logfiles
}

static NEEDS_ESCAPE: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

// original: write_to_log (htslib/ref_cache/log_files.c:266)
pub unsafe fn ref_cache_log_files_c_266_write_to_log(
    logfiles: *mut Logfiles,
    opts: *const Options,
    msg: *const c_char,
    len: usize,
) -> c_int {
    let mut written = 0usize;
    while written < len {
        let mut ok = written;
        while ok < len && NEEDS_ESCAPE[*msg.add(ok) as u8 as usize] == 0 {
            ok += 1;
        }

        if ok > written {
            let wrote = libc::fwrite(
                msg.add(written).cast(),
                1,
                ok - written,
                (*logfiles).curr_log,
            );
            if wrote != ok - written {
                break;
            }
        }
        if ok < len && NEEDS_ESCAPE[*msg.add(ok) as u8 as usize] != 0 {
            if *msg.add(ok) as u8 == b'\\' {
                if libc::fprintf((*logfiles).curr_log, b"\\\\\0".as_ptr().cast()) < 0 {
                    break;
                }
            } else if libc::fprintf(
                (*logfiles).curr_log,
                b"\\x%02x\0".as_ptr().cast(),
                *msg.add(ok) as u8 as c_int,
            ) < 0
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
                libc::stderr,
                b"Error writing to %s/%s : %s\n\0".as_ptr().cast(),
                (*opts).log_dir,
                (*(*logfiles).logs).name.as_ptr(),
                libc::strerror(*libc::__errno_location()),
            );
        } else {
            libc::fprintf(
                libc::stderr,
                b"Error writing to stdout: %s\n\0".as_ptr().cast(),
                libc::strerror(*libc::__errno_location()),
            );
        }
        return -1;
    }
    if !(*opts).log_dir.is_null() {
        (*(*logfiles).logs).size += written as libc::off_t;
        if (*(*logfiles).logs).size > (*opts).max_log_sz {
            if ref_cache_log_files_c_69_rotate_logs(logfiles, opts) != 0 {
                return -1;
            }
        }
    }
    0
}
