#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

OUT_DIR="${OUT_DIR:-/tmp/htslib-rs-real-data-perf}"
RUNS="${RUNS:-1}"
SKIP_BUILD="${SKIP_BUILD:-0}"

FASTQ_GZ="${FASTQ_GZ:-/husky/henriksson/for_claude/minimap/testdata/hg002_hifi_20370reads.fq.gz}"
BAM="${BAM:-/husky/henriksson/for_claude/cellsnp/gex_chr22_5m.bam}"
BAM_REGION="${BAM_REGION:-}"
REF_GZ="${REF_GZ:-/husky/henriksson/for_claude/minibwa/t2t_chm13/chm13v2.0.fa.gz}"
FASTA="${FASTA:-/husky/henriksson/for_claude/blast/inputs/chr1.fa}"

RUST_HTSFILE="${RUST_HTSFILE:-$ROOT/target/release/perf_htsfile}"
RUST_BGZIP="${RUST_BGZIP:-$ROOT/target/release/perf_bgzip}"
RUST_COUNT_BAM="${RUST_COUNT_BAM:-$ROOT/target/release/perf_count_bam}"
C_HTSFILE="${C_HTSFILE:-$ROOT/htslib/htsfile}"
C_BGZIP="${C_BGZIP:-$ROOT/htslib/bgzip}"
C_COUNT_BAM="${C_COUNT_BAM:-$OUT_DIR/bin/count_bam_c}"

CSV="$OUT_DIR/results.csv"

require_file() {
    local path="$1"
    if [[ ! -f "$path" ]]; then
        echo "missing required file: $path" >&2
        exit 1
    fi
}

input_size() {
    stat -c '%s' "$1"
}

run_one() {
    local workload="$1"
    local impl="$2"
    local bytes="$3"
    local run="$4"
    local command="$5"
    local time_file="$OUT_DIR/${workload}.${impl}.${run}.time"
    local hash_file="$OUT_DIR/${workload}.${impl}.${run}.sha256"
    local status_file="$OUT_DIR/${workload}.${impl}.${run}.status"

    rm -f "$time_file" "$hash_file" "$status_file"
    printf 'running %-28s %-4s run %s\n' "$workload" "$impl" "$run" >&2

    set +e
    /usr/bin/time -f '%e,%U,%S' -o "$time_file" \
        bash -o pipefail -c "$command | sha256sum | awk '{print \$1}' > '$hash_file'"
    local status=$?
    set -e

    printf '%s\n' "$status" > "$status_file"

    local timing=",,"
    local hash=""
    if [[ -s "$time_file" ]]; then
        timing="$(cat "$time_file")"
    fi
    if [[ -s "$hash_file" ]]; then
        hash="$(cat "$hash_file")"
    fi

    printf '%s,%s,%s,%s,%s,%s,%s\n' \
        "$workload" "$impl" "$bytes" "$run" "$timing" "$hash" "$status" >> "$CSV"

    if [[ "$status" != 0 ]]; then
        echo "command failed for $workload $impl run $run with status $status" >&2
        return "$status"
    fi
}

run_pair() {
    local workload="$1"
    local input="$2"
    local rust_command="$3"
    local c_command="$4"
    local bytes
    bytes="$(input_size "$input")"

    for run in $(seq 1 "$RUNS"); do
        run_one "$workload" rust "$bytes" "$run" "$rust_command"
        run_one "$workload" c "$bytes" "$run" "$c_command"
    done
}

mkdir -p "$OUT_DIR"

require_file "$FASTQ_GZ"
require_file "$BAM"
require_file "$REF_GZ"
require_file "$FASTA"
require_file "$C_HTSFILE"
require_file "$C_BGZIP"

if [[ "$SKIP_BUILD" != 1 ]]; then
    (cd "$ROOT" && cargo build --release --bin perf_htsfile --bin perf_bgzip --bin perf_count_bam)
fi

require_file "$RUST_HTSFILE"
require_file "$RUST_BGZIP"
require_file "$RUST_COUNT_BAM"

mkdir -p "$OUT_DIR/bin"
cc -O3 -I "$ROOT/htslib" -I "$ROOT/htslib/htslib" \
    "$ROOT/tools/count_bam_c.c" \
    -L "$ROOT/htslib" -Wl,-rpath,"$ROOT/htslib" \
    -lhts -lz -lm -lbz2 -llzma -lcurl -lpthread \
    -o "$C_COUNT_BAM"
require_file "$C_COUNT_BAM"

printf 'workload,impl,input_bytes,run,real_s,user_s,sys_s,sha256,status\n' > "$CSV"

run_pair \
    htsfile_fastq_gz_view \
    "$FASTQ_GZ" \
    "'$RUST_HTSFILE' -c '$FASTQ_GZ'" \
    "'$C_HTSFILE' -c '$FASTQ_GZ'"

run_pair \
    htsfile_bam_view \
    "$BAM" \
    "'$RUST_HTSFILE' -c '$BAM'" \
    "'$C_HTSFILE' -c '$BAM'"

if [[ -n "$BAM_REGION" ]]; then
    run_pair \
        count_bam_region \
        "$BAM" \
        "'$RUST_COUNT_BAM' '$BAM' '$BAM_REGION'" \
        "'$C_COUNT_BAM' '$BAM' '$BAM_REGION'"
else
    run_pair \
        count_bam_full \
        "$BAM" \
        "'$RUST_COUNT_BAM' '$BAM'" \
        "'$C_COUNT_BAM' '$BAM'"
fi

run_pair \
    bgzip_decompress_ref_gz \
    "$REF_GZ" \
    "'$RUST_BGZIP' -d -c '$REF_GZ'" \
    "'$C_BGZIP' -d -c '$REF_GZ'"

run_pair \
    bgzip_compress_fasta \
    "$FASTA" \
    "'$RUST_BGZIP' -c '$FASTA'" \
    "'$C_BGZIP' -c '$FASTA'"

echo "wrote $CSV"
