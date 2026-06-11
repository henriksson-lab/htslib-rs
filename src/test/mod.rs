//! Test-module-shared isolation primitives.
//!
//! Many of the `htslib-rs` test files contain `#[test]` functions whose names
//! start with `original_` and which invoke a translated C `main()` entry point
//! (`test_test_NAME_c_*_main`, `test_sam_c_*_*`, etc.). These tests faithfully
//! reproduce what the C `main()` does, and because they execute inside the same
//! Rust test process, they share a single set of PROCESS-GLOBAL state — most
//! notably:
//!
//!   * the C library `getopt` globals (`optarg`, `optind`, `optopt`,
//!     `optreset`),
//!   * the process current working directory (`chdir`),
//!   * the `TEST_SAM_STATUS` / similar global accumulators used by the
//!     C-style test harnesses,
//!   * `errno`, signal disposition, and other libc globals.
//!
//! Per-file `Mutex`es (`GETOPT_LOCK`, `TEST_SAM_LOCK`, `TEST_VCF_API_LOCK`) keep
//! the tests *within* a single file serialized, but they DO NOT serialize
//! tests across files: when `cargo test --tests --test-threads=N` runs, the
//! `original_*_main` tests in file A can race the `original_*_main` tests in
//! file B because each file holds its own mutex. The symptom is occasional
//! flakes at high thread counts (e.g. an "invalid option" failure in
//! `test-bcf-sr` while `test_mod` is concurrently parsing its own argv).
//!
//! [`ORIGINAL_MAIN_LOCK`] is a single PROCESS-WIDE `Mutex<()>` that every
//! `original_*_main` (and every test that calls a translated C `main()`)
//! acquires on entry. Combined with the per-file locks, this guarantees:
//!
//!   * mutual exclusion across the entire crate's `original_*_main` tests,
//!   * no race against the C `getopt` globals,
//!   * no interleaved cwd churn,
//!   * no shared-state accumulator (`TEST_SAM_STATUS`) read-modify-write race.
//!
//! Lock ordering rule (deadlock-free):
//!   1. Acquire [`ORIGINAL_MAIN_LOCK`] FIRST.
//!   2. Then acquire any per-file lock (`GETOPT_LOCK`, `TEST_SAM_LOCK`, etc.).
//!
//! All locks are taken with `.lock().unwrap_or_else(|e| e.into_inner())` so a
//! single panicking test cannot cascade-poison the lock and break every later
//! test that acquires it.
//!
//! [`CwdGuard`] captures the process cwd on construction and restores it on
//! `Drop`. Combined with the lock it makes tests resilient to BOTH stale cwd
//! AND lock poisoning. Use:
//!
//! ```ignore
//! let _cwd = CwdGuard::new();
//! let _lock = ORIGINAL_MAIN_LOCK
//!     .lock()
//!     .unwrap_or_else(|e| e.into_inner());
//! ```
//!
//! Tests that ONLY call a C `main()` in a forked child are theoretically safe
//! (the child has its own private copy of the globals), but they still take
//! the lock because the PARENT often does pre-work in the shared address space
//! (file setup, htslib library calls) that also wants serialization, and
//! because making the rule uniform avoids future foot-guns.
//!
//! ## Cross-binary serialization (file lock)
//!
//! `cargo test --tests --test-threads=N` runs the lib binary AND each
//! integration test binary as separate PROCESSES, each with its own private
//! copy of [`ORIGINAL_MAIN_LOCK`]'s in-process `Mutex<()>`. The mutex therefore
//! cannot serialize `original_*_main`-style tests ACROSS test binaries — only
//! WITHIN one. To close that gap, every `ORIGINAL_MAIN_LOCK.lock()` ALSO
//! acquires a `libc::flock(LOCK_EX)` on a stable lock-file
//! (`<temp_dir>/htslib-rs-original-main.lock`), which IS shared by every
//! process. The result is a single serialization point across the whole
//! `cargo test --tests` invocation. The file lock is released automatically
//! when the returned guard drops.
//!
//! The file lock costs microseconds per acquire and never blocks unless
//! another test binary is already inside the critical section — exactly the
//! case we want to serialize.

/// Process-wide guard combining the in-binary `Mutex<()>` with a cross-binary
/// `libc::flock` file lock. See module-level docs.
///
/// Held via `let _g = ORIGINAL_MAIN_LOCK.lock().unwrap_or_else(|e| e.into_inner());`
/// just like a `std::sync::MutexGuard`. Dropping the guard releases the file
/// lock and then the mutex, in that order.
pub(crate) struct OriginalMainGuard {
    // File lock first, mutex second — declared in this order so Drop runs
    // file-lock-release BEFORE mutex-release (Rust drops fields top-to-bottom).
    // This keeps the lock-acquire order (mutex first, then flock) symmetric on
    // release (flock first, then mutex) and avoids briefly holding the cross-
    // binary lock with no in-binary mutex protection.
    _file_lock: FileLockGuard,
    _mutex: std::sync::MutexGuard<'static, ()>,
}

/// RAII wrapper around a `libc::flock(LOCK_EX)` acquisition. The file handle
/// is kept open for the lifetime of the guard so the kernel preserves the
/// lock; explicit `LOCK_UN` on drop is belt-and-braces (closing the file
/// would release the lock anyway).
struct FileLockGuard {
    fd: i32,
    // Keeps the underlying fd alive; closed on drop AFTER LOCK_UN below.
    _file: std::fs::File,
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        // Best-effort unlock. Ignore the return code: even on failure the
        // kernel releases the lock once the fd is closed, and Drop must not
        // panic.
        unsafe {
            libc::flock(self.fd, libc::LOCK_UN);
        }
    }
}

/// Static handle exposing the process-wide [`OriginalMainGuard`]. Existing
/// call sites use:
///
/// ```ignore
/// let _g = ORIGINAL_MAIN_LOCK
///     .lock()
///     .unwrap_or_else(|e| e.into_inner());
/// ```
///
/// — that pattern keeps working because [`Self::lock`] returns the same
/// `Result<Guard, PoisonError<Guard>>` shape as `std::sync::Mutex::lock`.
pub(crate) struct OriginalMainLock {
    inner: std::sync::Mutex<()>,
}

impl OriginalMainLock {
    const fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(()),
        }
    }

    /// Acquire the combined in-binary mutex + cross-binary file lock.
    ///
    /// Returns `Result<OriginalMainGuard, PoisonError<OriginalMainGuard>>`
    /// mirroring `std::sync::Mutex::lock`, so call sites can keep using
    /// `.lock().unwrap_or_else(|e| e.into_inner())` for poison tolerance.
    /// The file lock is acquired AFTER the mutex (lock-order: in-binary
    /// before cross-binary) and released BEFORE the mutex on drop.
    pub(crate) fn lock(
        &'static self,
    ) -> Result<OriginalMainGuard, std::sync::PoisonError<OriginalMainGuard>> {
        match self.inner.lock() {
            Ok(mutex_guard) => {
                let file_lock = acquire_file_lock();
                Ok(OriginalMainGuard {
                    _file_lock: file_lock,
                    _mutex: mutex_guard,
                })
            }
            Err(poison) => {
                // Mutex was poisoned. Recover the inner guard, take the file
                // lock anyway, and re-wrap into a fresh PoisonError so callers
                // get the same `.unwrap_or_else(|e| e.into_inner())` ergonomics.
                let mutex_guard = poison.into_inner();
                let file_lock = acquire_file_lock();
                Err(std::sync::PoisonError::new(OriginalMainGuard {
                    _file_lock: file_lock,
                    _mutex: mutex_guard,
                }))
            }
        }
    }
}

/// Open (or create) the shared lock-file at `<temp_dir>/htslib-rs-original-main.lock`
/// and acquire `LOCK_EX` (blocking). Used by every `original_*_main`-style
/// test so multiple `cargo test` binaries serialize against each other.
fn acquire_file_lock() -> FileLockGuard {
    use std::os::fd::AsRawFd;

    let path = std::env::temp_dir().join("htslib-rs-original-main.lock");
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap_or_else(|e| {
            panic!(
                "ORIGINAL_MAIN_LOCK: cannot open cross-binary lock-file {}: {}",
                path.display(),
                e
            )
        });
    let fd = file.as_raw_fd();
    // Blocking exclusive lock. EINTR is retried; any other error is fatal
    // because losing serialization here defeats the whole point of the lock.
    loop {
        let rc = unsafe { libc::flock(fd, libc::LOCK_EX) };
        if rc == 0 {
            break;
        }
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        panic!(
            "ORIGINAL_MAIN_LOCK: flock(LOCK_EX) on {} failed: {}",
            path.display(),
            err
        );
    }
    FileLockGuard { fd, _file: file }
}

/// Process-wide lock protecting every test that emulates a translated
/// C `main()`. See the module docs for the full rationale. Always acquire
/// with `.lock().unwrap_or_else(|e| e.into_inner())` so test panics do not
/// poison the lock. The acquired guard transparently holds BOTH an in-binary
/// `Mutex<()>` AND a cross-binary `libc::flock` file lock, so serialization
/// holds across every `cargo test --tests` binary in the workspace.
pub(crate) static ORIGINAL_MAIN_LOCK: OriginalMainLock = OriginalMainLock::new();

/// RAII guard that captures the process cwd on construction and restores it
/// on drop, so a test that calls `chdir` cannot leak its cwd into a later
/// test. Construct BEFORE acquiring [`ORIGINAL_MAIN_LOCK`] is fine — the
/// guard does no locking itself. Failures during restoration are intentionally
/// ignored (drop must not panic).
pub(crate) struct CwdGuard {
    original: Option<std::path::PathBuf>,
}

impl CwdGuard {
    /// Capture the current working directory.
    pub(crate) fn new() -> Self {
        Self {
            original: std::env::current_dir().ok(),
        }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        if let Some(path) = self.original.take() {
            let _ = std::env::set_current_dir(&path);
        }
    }
}

pub mod fieldarith;
pub mod fuzz;
pub mod hfile;
pub mod hts_endian;
pub mod pileup;
pub mod pileup_mod;
#[cfg(any())]
#[path = "plugins-dlhts.rs"]
pub mod plugins_dlhts;
pub mod sam;
#[path = "test-bcf_set_variant_type.rs"]
pub mod test_bcf_set_variant_type;
#[path = "test-bcf-sr.rs"]
pub mod test_bcf_sr;
pub mod test_bgzf;
pub mod test_expr;
pub mod test_faidx;
pub mod test_hfile_libcurl;
pub mod test_index;
pub mod test_introspection;
pub mod test_kfunc;
pub mod test_khash;
pub mod test_kstring;
pub mod test_mod;
pub mod test_nibbles;
#[path = "test-parse-reg.rs"]
pub mod test_parse_reg;
pub mod test_realn;
#[path = "test-regidx.rs"]
pub mod test_regidx;
pub mod test_str2int;
pub mod test_time_funcs;
#[path = "test-vcf-api.rs"]
pub mod test_vcf_api;
#[path = "test-vcf-sweep.rs"]
pub mod test_vcf_sweep;
pub mod test_view;
pub mod thrash_threads1;
pub mod thrash_threads2;
pub mod thrash_threads3;
pub mod thrash_threads4;
pub mod thrash_threads5;
pub mod thrash_threads6;
pub mod thrash_threads7;
