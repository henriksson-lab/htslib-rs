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

/// Process-wide mutex protecting every test that emulates a translated
/// C `main()`. See the module docs for the full rationale. Always acquire
/// with `.lock().unwrap_or_else(|e| e.into_inner())` so test panics do not
/// poison the lock.
pub(crate) static ORIGINAL_MAIN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
