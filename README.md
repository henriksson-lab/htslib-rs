# htslib_rs

This is a translation of htslib, git version `c9e7a96448b7a307d2acf24705491f1a4ab8dea6`

**Most Rust developers should not use this crate**. This crate has the benefit of making Rust translation of code that depends on htslib easy, and sometimes it offers more performance.
But causual developers should first see if the noodles (https://docs.rs/noodles/latest/noodles/) Rust library fulfills their needs. Noodles is designed to be idiomatic safe Rust,
while this crate merely aims to provide an unfiltered path into the original C htslib. But because the original htslib is in C and our translations are 1-1 for auditability,
this crate makes little attempt to prevent to prevent the user from doing something harmful. Thus, be careful when using this crate.


**This translation is incomplete, features are still missing**
* 2026-06-02: Possibly feature complete; speed on par
* 2026-05-29: Translation still ongoing
* 2026-05-14: Minimal testing performed 


## This is an LLM-mediated faithful (hopefully) translation, not the original code! 

Most users should probably first see if the existing original code works for them, unless they have reason otherwise. The original source
may have newer features and it has had more love in terms of fixing bugs. In fact, we aim to replicate bugs if they are present, for the
sake of reproducibility! (but then we might have added a few more in the process)

There are however cases when you might prefer this Rust version. We generally agree with [this manifesto](https://rewrites.bio/) but more specifically:
* We have had many issues with ensuring that our software works using existing containers (Docker, PodMan, Singularity). One size does not fit all and it eats our resources trying to keep up with every way of delivering software
* Common package managers do not work well. It was great when we had a few Linux distributions with stable procedures, but now there are just too many ecosystems (Homebrew, Conda). Conda has an NP-complete resolver which does not scale. Homebrew is only so-stable. And our dependencies in Python still break. These can no longer be considered professional serious options. Meanwhile, Cargo enables multiple versions of packages to be available, even within the same program(!)
* The future is the web. We deploy software in the web browser, and until now that has meant Javascript. This is a language where even the == operator is broken. Typescript is one step up, but a game changer is the ability to compile Rust code into webassembly, enabling performance and sharing of code with the backend. Translating code to Rust enables new ways of deployment and running code in the browser has especial benefits for science - researchers do not have deep pockets to run servers, so pushing compute to the user enables deployment that otherwise would be impossible
* Old CLI-based utilities are bad for the environment(!). A large amount of compute resources are spent creating and communicating via small files, which we can bypass by using code as libraries. Even better, we can avoid frequent reloading of databases by hoisting this stage, with up to 100x speedups in some cases. Less compute means faster compute and less electricity wasted
* LLM-mediated translations may actually be safer to use than the original code. This article shows that [running the same code on different operating systems can give somewhat different answers](https://doi.org/10.1038/nbt.3820). This is a gap that Rust+Cargo can reduce. Typesafe interfaces also reduce coding mistakes and error handling, as opposed to typical command-line scripting

But:

* **This approach should still be considered experimental**. The LLM technology is immature and has sharp corners. But there are opportunities to reap, and the genie is not going back into the bottle. This translation is as much aimed to learn how to improve the technology and get feedback on the results.
* Translations are not endorsed by the original authors unless otherwise noted. **Do not send bug reports to the original developers**. Use our Github issues page instead.
* **Do not trust the benchmarks on this page**. They are used to help evaluate the translation. If you want improved performance, you generally have to use this code as a library, and use the additional tricks it offers. We generally accept performance losses in order to reduce our dependency issues
* **Check the original Github pages for information about the package**. This README is kept sparse on purpose. It is not meant to be the primary source of information
* **If you are the author of the original code and wish to move to Rust, you can obtain ownership of this repository and crate**. Until then, our commitment is to offer an as-faithful-as-possible translation of a snapshot of your code. If we find serious bugs, we will report them to you. Otherwise we will just replicate them, to ensure comparability across studies that claim to use package XYZ v.666. Think of this like a fancy Ubuntu .deb-package of your software - that is how we treat it

This blurb might be out of date. Go to [this page](https://github.com/henriksson-lab/rustification) for the latest information and further information about how we approach translation

## Source layout

Rust files mirror the htslib C source 1:1 by canonical mapping (lowercased filename, hyphens → underscores). For example `htslib/annot-tsv.c` → `src/annot_tsv.rs`, `htslib/cram/cram_io.c` → `src/cram/cram_io.rs`, `htslib/htscodecs/htscodecs/rANS_static.c` → `src/htscodecs/rans_static.rs`.

SIMD variants (`rANS_static32x16pr_avx2.c`, `_avx512.c`, `_neon.c`, `_sse4.c`) are intentionally not mapped — Rust uses LLVM auto-vectorization.

## Real-data performance comparisons

`tools/compare-real-data-performance.sh` compares translated release binaries with the checked-out original htslib binaries on larger local real-data workloads. It writes timing CSVs and per-run output hashes under `/tmp/htslib-rs-real-data-perf` by default, avoiding large outputs in the repository.

The default workloads cover ordinary gzip FASTQ viewing, BAM viewing, BAM record counting, bgzip decompression of a large BGZF-compressed reference, and bgzip compression of a large FASTA. Override `FASTQ_GZ`, `BAM`, `BAM_REGION`, `REF_GZ`, `FASTA`, `OUT_DIR`, or `RUNS` to use different data:

```bash
RUNS=3 tools/compare-real-data-performance.sh
```

## Known issues

### `bam_to_cram` byte parity vs C

Encoding BAM → CRAM in Rust produces a different byte stream than C, even though every record round-trips identically (Rust BAM → Rust CRAM → Rust BAM gives the original 5 000 000 records; Rust decoding a C-produced CRAM is byte-identical to C decoding it). The CRAM file is also ~177 KB smaller in Rust on the gex_chr22_5m.bam test (145.66 MB vs C 145.84 MB), so this is encoder-behaviour divergence, not a correctness bug.

What's known:

- Bytes 0 – 2 711 are byte-identical to C (CRAM file def + first container header + gzipped SAM-header block).
- First diff is at byte 2 712 — the **length field of container 2** (the first data container). Rust's length is 516 bytes smaller. The container header itself then matches for ~16 bytes before the compression-header block content diverges.
- The CRC32 of the compression-header block differs (`79 c7 73 9e` Rust vs `04 55 16 26` C). That CRC just reflects the differing block content.
- The preservation_map (`RN`, `SM`, `TD`, `AP`, `RR`) IS correctly populated and iterated in the same hash-bucket order as C — verified via probe. So the divergence is **not** in preservation-map content.

Suspected cause (not yet confirmed):

1. **`td_blk` (tag dictionary) content** differs — the byte layout for the same input tags is slightly different.
2. **Codec choices differ** for one or more data series (`BF`/`CF`/`RL`/...). The compression metrics in `cram_compress_block` pick a codec per data series; if Rust's metrics pick e.g. `RANS_PR128` where C picks `RANS_PR0` for the same series, the encoded codec descriptor bytes (and downstream block sizes) shift.

Confirming which requires dumping the uncompressed compression-header block from both files and diffing the codec map entry-by-entry. Then chase whichever data series is different up through `cram_encode_compression_header` (`src/cram/cram_encode.rs:28`) and the per-block codec selection (`cram_cram_io_c_1913_cram_compress_block3` in `src/cram/mod.rs`).

Working pieces (already correct, **don't revert**):

- `src/cram/mod.rs:cram_cram_io_c_1222_zlib_mem_deflate` uses system zlib directly with `deflateInit2_(level, Z_DEFLATED, 15|16, 9, strat)` to match C's gzip output bit-for-bit. This is what got us from divergence at byte 12 to byte 2712.
- `bgzf::system_zlib` / `bgzf::z_stream` / `bgzf::ZlibFns` are `pub(crate)` so the cram path can share them.

Doesn't need fixing right now — it's cosmetic. A future pass can audit it.

## License

* Files outside `htslib/cram/` are under the MIT/Expat license
* files within `htslib/cram/` are under the modified 3-clause BSD license.

are under the modified 3-clause BSD license.

## Citation

Bonfield JK, Marshall J, Danecek P, Li H, Ohan V, Whitwham A, Keane T, Davies RM. HTSlib: C library for reading/writing high-throughput sequencing data. GigaScience. 2021;10(2):giab007.
https://doi.org/10.1093/gigascience/giab007
