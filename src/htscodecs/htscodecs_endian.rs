//! Translation of htscodecs `htscodecs_endian.h` (header-only).
//!
//! Endianness checking. This header contains only preprocessor `#define`s set
//! according to the target architecture. In Rust the equivalent information is
//! available via `cfg!(target_endian = ...)`, so these are modelled as
//! constants whose values the translator should derive from the build target.

/// ```c
/// #define HTSCODECS_LITTLE_ENDIAN
/// ```
/// Defined when the target is detected as little endian.
// htscodecs_endian.h:76
pub const HTSCODECS_LITTLE_ENDIAN: bool = cfg!(target_endian = "little");

/// ```c
/// #define HTSCODECS_BIG_ENDIAN
/// ```
/// Defined when the target is detected as big endian.
// htscodecs_endian.h:87
pub const HTSCODECS_BIG_ENDIAN: bool = cfg!(target_endian = "big");

/// ```c
/// #define HTSCODECS_ENDIAN_KNOWN
/// ```
/// Defined when the system endianness was detected (always true for Rust
/// `cfg!(target_endian)`).
// htscodecs_endian.h:77 / :88
pub const HTSCODECS_ENDIAN_KNOWN: bool = true;
