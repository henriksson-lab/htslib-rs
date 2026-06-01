#!/usr/bin/env bash
#
# libhts cut-over ratchet.
#
# Goal: production code must not depend on the C HTSlib. The crate builds today
# with the default `parity` feature (which pulls in hts-sys); the authoritative
# "are we done" signal is that it also builds WITHOUT it:
#
#     cargo build --no-default-features      # must eventually compile AND link
#
# There are TWO forms of C dependency, and BOTH must reach zero:
#   1. `hts_sys::` references  — caught at COMPILE time by the no-parity check.
#   2. `#[link_name = "..."]` externs to libhts symbols — caught only at LINK
#      time (so the compile-only check above misses them). Benign system
#      symbols (zlib: deflate/inflate/crc32/...) are excluded.
#
# This ratchet tracks both counts and fails if either increased vs baseline.
#
# Usage:
#   scripts/check-hts-sys.sh           # check against baselines (CI gate)
#   scripts/check-hts-sys.sh --update  # record current counts as baselines
#
set -euo pipefail
cd "$(dirname "$0")/.."

BASELINE_FILE="scripts/hts_sys_baseline.txt"
LINK_BASELINE_FILE="scripts/libhts_linkname_baseline.txt"

# (1) hts_sys:: references — compile errors in the no-parity build (excludes
# #[cfg(test)] parity code).
current="$(cargo check --no-default-features --message-format=short 2>&1 \
            | grep -c 'hts_sys' || true)"
baseline="$(cat "$BASELINE_FILE" 2>/dev/null || echo 999999)"

# (2) libhts #[link_name] externs in production code (exclude tests and the
# benign zlib/system symbols).
# Count libhts #[link_name] externs in code ACTUALLY COMPILED in the production
# (no-test, no-parity) build. Exclusions:
#  - `src/cram/` is the `cram_mirror` module, gated `#[cfg(any())]` (never
#    compiled) — its externs never link.
#  - `tests.rs` files and everything from the first `#[cfg(test)]` onward in each
#    file (test modules are conventionally at file end; their parity-comparison
#    externs are test-only).
#  - benign external libs that are NOT the htslib-C dependency: zlib
#    (deflate/inflate/crc32/...), libcurl (curl_*), libc globals
#    (optarg/optind, stdin/stdout/stderr — libc-rs doesn't expose these as
#    safe items, so the extern decl is the canonical way to reach them).
link_current="$({ for f in $(grep -rl 'link_name' src/ | grep -vE '/test/|/cram/|tests\.rs'); do
                   awk '/#\[cfg\(test\)\]/{exit} /link_name/{print}' "$f"
                 done \
                 | grep -vE 'deflate|inflate|crc32|zlib|adler|compress|uncompress|curl_|optarg|optind|"stdin"|"stdout"|"stderr"' \
                 || true; } \
                 | wc -l | tr -d ' ')"
link_baseline="$(cat "$LINK_BASELINE_FILE" 2>/dev/null || echo 999999)"

echo "production hts_sys:: references (no-parity compile): current=$current baseline=$baseline"
echo "production libhts #[link_name] externs (link-time):  current=$link_current baseline=$link_baseline"

if [ "${1:-}" = "--update" ]; then
  printf '%s\n' "$current" > "$BASELINE_FILE"
  printf '%s\n' "$link_current" > "$LINK_BASELINE_FILE"
  echo "baselines updated: hts_sys=$current libhts_linkname=$link_current"
  exit 0
fi

rc=0
if [ "$current" -gt "$baseline" ]; then
  echo "FAIL: hts_sys:: usage increased ($baseline -> $current). Translate natively or gate behind #[cfg(feature=\"parity\")]."
  rc=1
fi
if [ "$link_current" -gt "$link_baseline" ]; then
  echo "FAIL: libhts #[link_name] externs increased ($link_baseline -> $link_current)."
  rc=1
fi
[ "$current"      -lt "$baseline" ]      && echo "Progress (hts_sys::): $baseline -> $current."
[ "$link_current" -lt "$link_baseline" ] && echo "Progress (link_name):  $link_baseline -> $link_current."
if [ "$current" -eq 0 ] && [ "$link_current" -eq 0 ]; then
  echo "DONE: production code is libhts-free. hts-sys can move to [dev-dependencies]."
fi
[ "$rc" -eq 0 ] && echo "OK"
exit "$rc"
