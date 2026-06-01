//! Translation of htscodecs `htscodecs.c` + `htscodecs.h`.
//!
//! Library version reporting.

/// ```c
/// #define HTSCODECS_VERSION 100606
/// ```
/// Version X.Y.Z encoded as XYYYZZ.
// htscodecs.h:46
pub const HTSCODECS_VERSION: i32 = 100606;

/// ```c
/// const char *htscodecs_version(void);
/// ```
/// A const string form of the HTSCODECS_VERSION define.
// htscodecs.c:42 (also htscodecs.h:53)
//
// The C source: `return HTSCODECS_VERSION_TEXT;` where
// HTSCODECS_VERSION_TEXT is defined in version.h (bundled htscodecs at v1.6.6).
pub fn htscodecs_version() -> &'static str {
    "1.6.6-1-g295a940"
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn htscodecs_version_matches_bundled_text() {
        assert_eq!(htscodecs_version(), "1.6.6-1-g295a940");
    }
    // NOTE: no parity test against the linked C `htscodecs_version()`. Our
    // vendored `third_party/hts-sys/build.rs:248` overrides the build-time
    // macro `HTSCODECS_VERSION_TEXT` to the string `"rust-htslib"` (a hts-sys
    // packaging artifact, not the real htscodecs version). Native faithfully
    // returns the upstream `version.h` value; the linked C returns the build
    // override. A byte-parity test would encode the hts-sys override as a
    // test invariant, so it is intentionally omitted.
}
