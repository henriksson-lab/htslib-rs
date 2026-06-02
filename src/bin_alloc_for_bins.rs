// Shared global-allocator override for the bundled binaries.
//
// When the crate is built with `--features bin-mimalloc`, each `perf_*`
// and sample binary that does `include!("bin_alloc.rs")` near its top
// switches its global allocator to mimalloc. Library consumers and the
// default `cargo build` are unaffected — the override is a binary-side
// choice, not a library requirement.

#[cfg(feature = "bin-mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
