#!/usr/bin/env python3
"""Summarise the perf-comparison CSV from compare-real-data-performance.sh.

Reports per-workload median time, max RSS, and parity status (sha256
match between rust and c outputs) for each impl.

Usage: tools/perf_report.py [path/to/results.csv]
"""
from __future__ import annotations

import csv
import statistics
import sys
from collections import defaultdict
from pathlib import Path


def fmt_time(s: str) -> str:
    if not s:
        return "?"
    try:
        return f"{float(s):.2f}s"
    except ValueError:
        return s


def fmt_rss(kb: str) -> str:
    if not kb:
        return "?"
    try:
        mb = float(kb) / 1024.0
        return f"{mb:.0f}MB"
    except ValueError:
        return kb


def main(csv_path: Path) -> int:
    rows = list(csv.DictReader(csv_path.open()))
    if not rows:
        print(f"No data in {csv_path}", file=sys.stderr)
        return 1

    # workload -> impl -> list of (real_s, max_rss_kb, sha256)
    grouped: dict[str, dict[str, list[tuple[float, float, str]]]] = defaultdict(
        lambda: defaultdict(list)
    )
    input_bytes_by_workload: dict[str, int] = {}
    for r in rows:
        if r["status"] != "0":
            continue
        try:
            real_s = float(r["real_s"])
            rss_kb = float(r["max_rss_kb"])
        except (ValueError, KeyError):
            continue
        grouped[r["workload"]][r["impl"]].append((real_s, rss_kb, r["sha256"]))
        input_bytes_by_workload[r["workload"]] = int(r["input_bytes"])

    workloads = sorted(grouped)
    if not workloads:
        print("No successful runs found.", file=sys.stderr)
        return 1

    # Column widths
    w_workload = max(28, max(len(w) for w in workloads))

    print("=" * (w_workload + 78))
    print(
        f"{'workload':<{w_workload}}  "
        f"{'impl':<5}  "
        f"{'real_s':>10}  "
        f"{'max_rss':>10}  "
        f"{'speedup':>8}  "
        f"{'parity':>10}"
    )
    print("-" * (w_workload + 78))

    for workload in workloads:
        impls = grouped[workload]
        median_real = {impl: statistics.median(t[0] for t in runs) for impl, runs in impls.items()}
        median_rss = {impl: statistics.median(t[1] for t in runs) for impl, runs in impls.items()}
        # Take last sha256 per impl as representative (all runs should match)
        sha = {impl: runs[-1][2] for impl, runs in impls.items()}

        rust_t = median_real.get("rust")
        c_t = median_real.get("c")
        speedup = (c_t / rust_t) if (rust_t and c_t) else None

        rust_sha = sha.get("rust")
        c_sha = sha.get("c")
        if rust_sha and c_sha:
            parity = "MATCH" if rust_sha == c_sha else "DIFFER"
        else:
            parity = "?"

        # Print one row per impl
        for impl in ("rust", "c"):
            if impl not in impls:
                continue
            extra_speedup = ""
            extra_parity = ""
            if impl == "rust":
                if speedup is not None:
                    extra_speedup = f"{speedup:.2f}x"
                extra_parity = parity
            print(
                f"{workload if impl == 'rust' else '':<{w_workload}}  "
                f"{impl:<5}  "
                f"{fmt_time(str(median_real[impl])):>10}  "
                f"{fmt_rss(str(median_rss[impl])):>10}  "
                f"{extra_speedup:>8}  "
                f"{extra_parity:>10}"
            )

    print("=" * (w_workload + 78))
    print(f"speedup = (c real_s) / (rust real_s); >1 means rust is faster")
    return 0


if __name__ == "__main__":
    csv_path = Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/htslib-rs-perf/results.csv")
    sys.exit(main(csv_path))
