use super::options::Options;
use std::io::Write;
use std::ptr::NonNull;

const LOG_NAME_LEN: usize = 80;

pub struct Logfile {
    name: Vec<u8>,
    size: i64,
}

struct DirHandle(NonNull<libc::DIR>);

impl DirHandle {
    unsafe fn open(path: &[u8]) -> Option<Self> {
        let mut path_c = path.to_vec();
        path_c.push(0);
        NonNull::new(libc::opendir(path_c.as_ptr().cast())).map(Self)
    }

    fn as_ptr(&self) -> *mut libc::DIR {
        self.0.as_ptr()
    }

    unsafe fn fd(&self) -> i32 {
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
    File(std::fs::File),
}

impl LogDestination {
    /// Write a raw slice to the current destination, returning the number of
    /// bytes written (or an I/O error).
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Unopened => Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "no log destination",
            )),
            Self::Stdout => {
                let stdout = std::io::stdout();
                let mut lock = stdout.lock();
                let n = lock.write(buf)?;
                lock.flush()?;
                Ok(n)
            }
            Self::File(file) => {
                let n = file.write(buf)?;
                file.flush()?;
                Ok(n)
            }
        }
    }

    fn is_open(&self) -> bool {
        !matches!(self, Self::Unopened)
    }
}

// original: Logfiles (htslib/ref_cache/log_files.c:53)
pub struct Logfiles {
    dir_handle: Option<DirHandle>,
    curr_log: LogDestination,
    logs: Vec<Logfile>,
    log_dir_fd: i32,
}

// original: rotate_logs (htslib/ref_cache/log_files.c:69)
pub unsafe fn ref_cache_log_files_c_69_rotate_logs(
    logfiles: &mut Logfiles,
    opts: &Options,
) -> i32 {
    let now = libc::time(std::ptr::null_mut());
    let mut tm: libc::tm = std::mem::zeroed();
    libc::gmtime_r(&now, &mut tm);
    let year = tm.tm_year + 1900;
    let month = tm.tm_mon + 1;
    let day = tm.tm_mday;
    let hour = tm.tm_hour;
    let minute = tm.tm_min;
    let second = tm.tm_sec;
    let mut log_fd = -1;
    let mut name: Vec<u8> = b"ref_cache.log".to_vec();

    for i in 0..99u32 {
        name = format!(
            "ref_cache_{year:04}{month:02}{day:02}{hour:02}{minute:02}{second:02}_{i:02}.log"
        )
        .into_bytes();
        debug_assert!(name.len() + 1 <= LOG_NAME_LEN);

        let mut name_c = name.clone();
        name_c.push(0);
        loop {
            log_fd = libc::openat(
                logfiles.log_dir_fd,
                name_c.as_ptr().cast(),
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
        eprintln!(
            "Couldn't open {}/{} for writing: {}",
            String::from_utf8_lossy(opts.log_dir.as_deref().unwrap_or(b"")),
            String::from_utf8_lossy(&name),
            std::io::Error::last_os_error(),
        );
        return -1;
    }

    // Adopt the descriptor as an owned Rust file handle.
    use std::os::unix::io::FromRawFd;
    let file = std::fs::File::from_raw_fd(log_fd);

    debug_assert!(opts.nlogs > 0);
    debug_assert!(logfiles.logs.capacity() > opts.nlogs as usize);
    let keep = opts.nlogs as usize - 1;
    for old_log in logfiles.logs.iter().skip(keep) {
        let mut name_c = old_log.name.clone();
        name_c.push(0);
        if libc::unlinkat(logfiles.log_dir_fd, name_c.as_ptr().cast(), 0) != 0 {
            eprintln!(
                "Warning: Couldn't remove old log file {}/{}: {}",
                String::from_utf8_lossy(opts.log_dir.as_deref().unwrap_or(b"")),
                String::from_utf8_lossy(&old_log.name),
                std::io::Error::last_os_error(),
            );
        }
    }

    logfiles.logs.truncate(keep);
    logfiles.logs.insert(0, Logfile { name, size: 0 });
    logfiles.curr_log = LogDestination::File(file);

    0
}

// original: close_logs (htslib/ref_cache/log_files.c:134)
pub fn ref_cache_log_files_c_134_close_logs(logfiles: Option<Box<Logfiles>>) {
    // Dropping the owned box releases the directory handle and any open log
    // file via their Drop impls.
    drop(logfiles);
}

// original: open_logs (htslib/ref_cache/log_files.c:148)
pub unsafe fn ref_cache_log_files_c_148_open_logs(opts: &Options) -> Option<Box<Logfiles>> {
    let mut logfiles = Box::new(Logfiles {
        dir_handle: None,
        curr_log: LogDestination::Unopened,
        logs: Vec::new(),
        log_dir_fd: -1,
    });

    let Some(log_dir) = opts.log_dir.as_deref() else {
        logfiles.curr_log = LogDestination::Stdout;
        return Some(logfiles);
    };

    debug_assert!(opts.nlogs > 0);

    logfiles.dir_handle = DirHandle::open(log_dir);

    if logfiles.dir_handle.is_none() {
        eprintln!(
            "Couldn't open directory {}: {}",
            String::from_utf8_lossy(log_dir),
            std::io::Error::last_os_error(),
        );
        return None;
    }

    logfiles.logs = Vec::with_capacity(opts.nlogs as usize + 1);
    logfiles.log_dir_fd = logfiles.dir_handle.as_ref().unwrap().fd();
    if logfiles.log_dir_fd < 0 {
        eprintln!(
            "Couldn't get descriptor for {} : {}",
            String::from_utf8_lossy(log_dir),
            std::io::Error::last_os_error(),
        );
        return None;
    }

    *libc::__errno_location() = 0;
    loop {
        let ent = libc::readdir(logfiles.dir_handle.as_ref().unwrap().as_ptr());
        if ent.is_null() {
            break;
        }
        // Pull the entry name out as an owned byte vector (no trailing NUL).
        let d_name_ptr = (*ent).d_name.as_ptr();
        let mut entry_name: Vec<u8> = Vec::new();
        let mut p = 0isize;
        loop {
            let b = *d_name_ptr.offset(p) as u8;
            if b == 0 {
                break;
            }
            entry_name.push(b);
            p += 1;
        }

        // Parse "ref_cache_<14 digits>_<u32>.<suffix>" by hand, replacing the
        // original sscanf.
        let matched = (|| {
            let rest = entry_name.strip_prefix(b"ref_cache_")?;
            if rest.len() < 15 || rest[14] != b'_' {
                return None;
            }
            if !rest[..14].iter().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let after = &rest[15..];
            let dot = after.iter().position(|&b| b == b'.')?;
            // The digits before the dot are the index; require at least one.
            if dot == 0 || !after[..dot].iter().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let suff = &after[dot + 1..];
            if suff.len() > 7 {
                return None;
            }
            Some(suff.to_vec())
        })();

        let Some(suff) = matched else {
            continue;
        };
        if suff != b"log" && suff != b"log.gz" {
            continue;
        }

        if entry_name.len() + 1 > LOG_NAME_LEN {
            std::process::abort();
        }

        let mut st: libc::stat = std::mem::zeroed();
        let mut name_c = entry_name.clone();
        name_c.push(0);
        if libc::fstatat(
            logfiles.log_dir_fd,
            name_c.as_ptr().cast(),
            &mut st,
            libc::AT_SYMLINK_NOFOLLOW,
        ) != 0
        {
            eprintln!(
                "Warning: Couldn't stat {}/{} : {}",
                String::from_utf8_lossy(log_dir),
                String::from_utf8_lossy(&entry_name),
                std::io::Error::last_os_error(),
            );
            continue;
        }
        if (st.st_mode & libc::S_IFMT) == libc::S_IFREG {
            logfiles.logs.push(Logfile {
                name: entry_name,
                size: st.st_size,
            });
        }
    }

    if !logfiles.logs.is_empty() {
        logfiles.logs.sort_by(|a, b| b.name.cmp(&a.name));
    }

    if ref_cache_log_files_c_69_rotate_logs(&mut logfiles, opts) < 0 {
        ref_cache_log_files_c_134_close_logs(Some(logfiles));
        return None;
    }

    Some(logfiles)
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
    logfiles: &mut Logfiles,
    opts: &Options,
    msg: &[u8],
) -> i32 {
    let len = msg.len();
    let mut written = 0usize;
    if !logfiles.curr_log.is_open() {
        eprintln!("No log file is open");
        return -1;
    }

    while written < len {
        let mut ok = written;
        while ok < len && NEEDS_ESCAPE[msg[ok] as usize] == 0 {
            ok += 1;
        }

        if ok > written {
            match logfiles.curr_log.write(&msg[written..ok]) {
                Ok(wrote) if wrote == ok - written => {}
                _ => break,
            }
        }
        if ok < len && NEEDS_ESCAPE[msg[ok] as usize] != 0 {
            if msg[ok] == b'\\' {
                if logfiles.curr_log.write(b"\\\\").is_err() {
                    break;
                }
            } else {
                let escaped = format!("\\x{:02x}", msg[ok]).into_bytes();
                if logfiles.curr_log.write(&escaped).is_err() {
                    break;
                }
            }
            ok += 1;
        }
        written = ok;
    }

    if written < len {
        let err = std::io::Error::last_os_error();
        if let Some(log_dir) = opts.log_dir.as_deref() {
            let empty = Vec::new();
            let name = logfiles.logs.first().map_or(&empty, |log| &log.name);
            eprintln!(
                "Error writing to {}/{} : {}",
                String::from_utf8_lossy(log_dir),
                String::from_utf8_lossy(name),
                err,
            );
        } else {
            eprintln!("Error writing to stdout: {}", err);
        }
        return -1;
    }
    if opts.log_dir.is_some() {
        if let Some(current_log) = logfiles.logs.first_mut() {
            current_log.size += written as i64;
            if current_log.size > opts.max_log_sz
                && ref_cache_log_files_c_69_rotate_logs(logfiles, opts) != 0
            {
                return -1;
            }
        }
    }
    0
}
