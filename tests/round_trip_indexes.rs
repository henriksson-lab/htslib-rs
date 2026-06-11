// Round-trip integration tests for index files (BAI / TBI / CSI / CRAI).
//
// For each pair (data file, index format) we:
//   1. Build the data file (or reuse a shipped fixture for CRAM).
//   2. Build an index for it via the production native code paths.
//   3. Re-open the data file and query a region using that index.
//   4. Compare the result against a reference computed via a straightforward
//      linear scan of the underlying text (no external tooling required).
//
// This validates "index build -> index load -> region query returns the
// expected records" end to end.

use htslib_rs::{
    bam_destroy1, bam_endpos, bam_get_qname, bam_init1, bcf_destroy, bcf_hdr_destroy,
    bcf_hdr_name2id, bcf_hdr_write, bcf_index_build, bcf_index_load2, bcf_init, bcf_readrec,
    bcf_write, hts_close, hts_get_bgzfp, hts_idx_destroy, hts_itr_destroy, hts_itr_next,
    hts_itr_query, hts_open, hts_pos_t, hts_set_fai_filename, ks_free, kstring_t,
    sam_c_4553_sam_write1, sam_hdr_destroy, sam_hdr_read, sam_hdr_write, sam_index_build,
    sam_index_load, sam_itr_next, sam_itr_querys, sam_read1, tbx_conf_vcf, tbx_destroy,
    tbx_index_build2, tbx_index_load2, tbx_itr_querys1, vcf_hdr_read, vcf_read, vcf_write, BGZF,
};
use std::path::{Path, PathBuf};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

// Build a NUL-terminated byte buffer for production callees that still take a
// raw `*const c_char` at the C boundary.
fn c_path(path: &Path) -> Vec<u8> {
    let mut v = path.to_string_lossy().as_bytes().to_vec();
    v.push(0);
    v
}

fn tmp_path(label: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "htslib_rs-round-trip-idx-{}-{}.{}",
        std::process::id(),
        label,
        ext
    ))
}

fn cleanup(path: &Path) {
    let _ = std::fs::remove_file(path);
}

fn cleanup_with_sidecar(path: &Path, sidecar_suffix: &str) {
    cleanup(path);
    let mut sidecar = path.to_path_buf();
    let new = format!(
        "{}{sidecar_suffix}",
        sidecar.file_name().unwrap().to_string_lossy()
    );
    sidecar.set_file_name(new);
    cleanup(&sidecar);
}

// --------------------------------------------------------------------------
// helpers: build BAM / BCF / bgzipped VCF from in-tree text fixtures
// --------------------------------------------------------------------------

unsafe fn write_bam_from_sam(sam_path: &Path, bam_path: &Path) {
    let sam_c = c_path(sam_path);
    let bam_c = c_path(bam_path);

    let in_fp = hts_open(sam_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {}", sam_path.display());
    let hdr = sam_hdr_read(in_fp);
    assert!(!hdr.is_null());

    let out_fp = hts_open(bam_c.as_ptr().cast(), c"wb".as_ptr());
    assert!(!out_fp.is_null(), "failed to create {}", bam_path.display());
    assert_eq!(sam_hdr_write(out_fp, hdr), 0);

    let rec = bam_init1();
    assert!(!rec.is_null());
    loop {
        let ret = sam_read1(in_fp, hdr, rec);
        if ret < 0 {
            break;
        }
        assert!(sam_c_4553_sam_write1(out_fp, hdr, rec) > 0);
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(hts_close(out_fp), 0);
}

unsafe fn write_bcf_from_vcf(vcf_path: &Path, bcf_path: &Path) {
    let vcf_c = c_path(vcf_path);
    let bcf_c = c_path(bcf_path);

    let in_fp = hts_open(vcf_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {}", vcf_path.display());
    let hdr = vcf_hdr_read(in_fp);
    assert!(!hdr.is_null());

    let out_fp = hts_open(bcf_c.as_ptr().cast(), c"wb".as_ptr());
    assert!(!out_fp.is_null(), "failed to create {}", bcf_path.display());
    assert_eq!(bcf_hdr_write(out_fp, hdr), 0);

    let rec = bcf_init();
    assert!(!rec.is_null());
    while vcf_read(in_fp, hdr, rec) >= 0 {
        assert_eq!(bcf_write(out_fp, hdr, rec), 0);
    }

    bcf_destroy(rec);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(hts_close(out_fp), 0);
}

unsafe fn write_bgz_vcf_from_vcf(vcf_path: &Path, bgz_path: &Path) {
    let vcf_c = c_path(vcf_path);
    let bgz_c = c_path(bgz_path);

    let in_fp = hts_open(vcf_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {}", vcf_path.display());
    let hdr = vcf_hdr_read(in_fp);
    assert!(!hdr.is_null());

    let out_fp = hts_open(bgz_c.as_ptr().cast(), c"wz".as_ptr());
    assert!(!out_fp.is_null(), "failed to create {}", bgz_path.display());
    assert_eq!(bcf_hdr_write(out_fp, hdr), 0);

    let rec = bcf_init();
    assert!(!rec.is_null());
    while vcf_read(in_fp, hdr, rec) >= 0 {
        assert_eq!(vcf_write(out_fp, hdr, rec), 0);
    }

    bcf_destroy(rec);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(hts_close(out_fp), 0);
}

// --------------------------------------------------------------------------
// query helpers
// --------------------------------------------------------------------------

unsafe fn bam_region_qnames(bam_path: &Path, region: &[u8]) -> Vec<Vec<u8>> {
    let path_c = c_path(bam_path);
    let fp = hts_open(path_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", bam_path.display());
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let idx = sam_index_load(fp, path_c.as_ptr().cast());
    assert!(
        !idx.is_null(),
        "failed to load index for {}",
        bam_path.display()
    );
    let itr = sam_itr_querys(idx, hdr, region.as_ptr().cast());
    assert!(!itr.is_null(), "failed to build iterator for {:?}", region);

    let rec = bam_init1();
    assert!(!rec.is_null());
    let mut out = Vec::new();
    loop {
        let ret = sam_itr_next(fp, itr, rec);
        if ret < 0 {
            break;
        }
        let qname = bam_get_qname(rec);
        let len = (0..).take_while(|&i| *qname.add(i) != 0).count();
        out.push(std::slice::from_raw_parts(qname.cast::<u8>(), len).to_vec());
    }

    bam_destroy1(rec);
    hts_itr_destroy(itr);
    hts_idx_destroy(idx);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

unsafe fn bam_linear_scan_qnames_in_region(
    bam_path: &Path,
    contig_name: &str,
    start: i64,
    end_exclusive: i64,
) -> Vec<Vec<u8>> {
    let path_c = c_path(bam_path);
    let fp = hts_open(path_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", bam_path.display());
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());

    let mut target_tid: i32 = -1;
    let target_name = contig_name.as_bytes();
    for i in 0..(*hdr).n_targets {
        let name_ptr = (*(*hdr).target_name.add(i as usize)).cast::<u8>();
        let len = (0..).take_while(|&j| *name_ptr.add(j) != 0).count();
        let name = std::slice::from_raw_parts(name_ptr, len);
        if name == target_name {
            target_tid = i;
            break;
        }
    }
    assert!(target_tid >= 0, "contig {contig_name} not in header");

    let rec = bam_init1();
    assert!(!rec.is_null());
    let mut out = Vec::new();
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        if (*rec).core.tid != target_tid {
            continue;
        }
        let pos = (*rec).core.pos;
        let endpos = bam_endpos(rec);
        if endpos > start && pos < end_exclusive {
            let qname = bam_get_qname(rec);
            let len = (0..).take_while(|&i| *qname.add(i) != 0).count();
            out.push(std::slice::from_raw_parts(qname.cast::<u8>(), len).to_vec());
        }
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

unsafe extern "C" fn bcf_readrec_adapter(
    fp: *mut BGZF,
    data: *mut std::os::raw::c_void,
    rec: *mut std::os::raw::c_void,
    tid: *mut i32,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> i32 {
    unsafe { bcf_readrec(fp, data, rec, tid, beg, end) }
}

unsafe fn bcf_csi_region_count(
    bcf_path: &Path,
    csi_path: &Path,
    chrom: &[u8],
    beg: hts_pos_t,
    end: hts_pos_t,
) -> usize {
    let bcf_c = c_path(bcf_path);
    let csi_c = c_path(csi_path);

    let fp = hts_open(bcf_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", bcf_path.display());
    let hdr = vcf_hdr_read(fp);
    assert!(!hdr.is_null());
    let tid = bcf_hdr_name2id(hdr, chrom.as_ptr().cast());
    assert!(tid >= 0, "{:?} not in header", chrom);

    let idx = bcf_index_load2(bcf_c.as_ptr().cast(), csi_c.as_ptr().cast());
    assert!(!idx.is_null(), "failed to load CSI {}", csi_path.display());
    let itr = hts_itr_query(idx, tid, beg, end, Some(bcf_readrec_adapter));
    assert!(!itr.is_null());

    let rec = bcf_init();
    assert!(!rec.is_null());
    let mut count = 0usize;
    loop {
        let ret = hts_itr_next(hts_get_bgzfp(fp), itr, rec.cast(), std::ptr::null_mut());
        if ret < 0 {
            break;
        }
        count += 1;
    }

    bcf_destroy(rec);
    hts_itr_destroy(itr);
    hts_idx_destroy(idx);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    count
}

unsafe fn vcf_tabix_region_count(bgz_path: &Path, tbi_path: &Path, region: &[u8]) -> usize {
    let bgz_c = c_path(bgz_path);
    let tbi_c = c_path(tbi_path);

    let fp = hts_open(bgz_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", bgz_path.display());

    let tbx = tbx_index_load2(&bgz_c, Some(&tbi_c));
    assert!(!tbx.is_null(), "failed to load TBI {}", tbi_path.display());

    let itr = tbx_itr_querys1(tbx, region);
    assert!(!itr.is_null(), "tbx_itr_querys1 failed for {:?}", region);

    let mut line: kstring_t = kstring_t::default();
    let mut count = 0usize;
    loop {
        let ret = hts_itr_next(
            hts_get_bgzfp(fp),
            itr,
            (&mut line as *mut kstring_t).cast::<std::os::raw::c_void>(),
            tbx.cast::<std::os::raw::c_void>(),
        );
        if ret < 0 {
            break;
        }
        count += 1;
    }

    ks_free(&mut line);
    hts_itr_destroy(itr);
    tbx_destroy(tbx);
    assert_eq!(hts_close(fp), 0);
    count
}

/// Count records in a text VCF that fall inside the inclusive POS range
/// `[start, end]` on contig `chrom`.  POS is 1-based.
fn vcf_linear_scan_count(vcf_path: &Path, chrom: &str, start: i64, end: i64) -> usize {
    let mut count = 0usize;
    for line in std::fs::read_to_string(vcf_path).unwrap().lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        let line_chrom = fields.next().unwrap_or("");
        let pos: i64 = fields.next().unwrap_or("0").parse().unwrap_or(0);
        if line_chrom == chrom && pos >= start && pos <= end {
            count += 1;
        }
    }
    count
}

// --------------------------------------------------------------------------
// 1. BAM + BAI
// --------------------------------------------------------------------------

#[test]
fn round_trip_bam_bai_region_query_matches_linear_scan() {
    unsafe {
        let sam = fixture("htslib/test/index.sam");
        let bam = tmp_path("bam-bai", "bam");
        cleanup_with_sidecar(&bam, ".bai");

        write_bam_from_sam(&sam, &bam);
        let bam_c = c_path(&bam);
        assert_eq!(sam_index_build(bam_c.as_ptr().cast(), 0), 0);
        let bai = bam.with_extension("bam.bai");
        assert!(bai.exists(), "BAI not created at {}", bai.display());

        // Half-open [start, end_excl) — sam_itr_querys uses
        // 1-based inclusive in the region string.
        let region = b"CHROMOSOME_I:999900-1000100\0";
        let query = bam_region_qnames(&bam, region);
        let scan = bam_linear_scan_qnames_in_region(&bam, "CHROMOSOME_I", 999_900 - 1, 1_000_100);

        let mut query_sorted = query.clone();
        let mut scan_sorted = scan.clone();
        query_sorted.sort();
        scan_sorted.sort();
        assert_eq!(
            query_sorted, scan_sorted,
            "indexed query and linear scan disagree on overlapping records",
        );
        assert!(!query.is_empty(), "expected at least one overlapping read");

        cleanup_with_sidecar(&bam, ".bai");
    }
}

// --------------------------------------------------------------------------
// 2. VCF + TBI
// --------------------------------------------------------------------------

#[test]
fn round_trip_vcf_tbi_region_query_matches_linear_scan() {
    unsafe {
        let vcf = fixture("htslib/test/index.vcf");
        let bgz = tmp_path("vcf-tbi", "vcf.gz");
        let tbi = bgz.with_extension("gz.tbi");
        cleanup(&bgz);
        cleanup(&tbi);

        write_bgz_vcf_from_vcf(&vcf, &bgz);

        let bgz_c = c_path(&bgz);
        let tbi_c = c_path(&tbi);
        let conf = tbx_conf_vcf();
        let ret = tbx_index_build2(&bgz_c, Some(&tbi_c), 0, &conf);
        assert_eq!(ret, 0, "tbx_index_build2 failed");
        assert!(tbi.exists());

        // index.vcf records on contig "1" cluster around POS=10_000_000.
        let region = b"1:9999000-10001000\0";
        let count_indexed = vcf_tabix_region_count(&bgz, &tbi, region);
        let count_scan = vcf_linear_scan_count(&vcf, "1", 9_999_000, 10_001_000);

        assert_eq!(
            count_indexed, count_scan,
            "VCF TBI region query returned {count_indexed}, linear scan found {count_scan}",
        );
        assert!(count_indexed > 0, "expected records in 1:9999000-10001000");

        cleanup(&bgz);
        cleanup(&tbi);
    }
}

// --------------------------------------------------------------------------
// 3. BCF + CSI
// --------------------------------------------------------------------------

#[test]
fn round_trip_bcf_csi_region_query_matches_linear_scan() {
    unsafe {
        let vcf = fixture("htslib/test/index.vcf");
        let bcf = tmp_path("bcf-csi", "bcf");
        let csi = bcf.with_extension("bcf.csi");
        cleanup(&bcf);
        cleanup(&csi);

        write_bcf_from_vcf(&vcf, &bcf);
        let bcf_c = c_path(&bcf);
        assert_eq!(bcf_index_build(bcf_c.as_ptr().cast(), 14), 0);
        assert!(csi.exists(), "CSI not created at {}", csi.display());

        // bcf_hdr_name2id + hts_itr_query uses 0-based half-open [beg, end).
        // 1:9999000-10001000 in VCF coords -> POS in [9_999_000, 10_001_000]
        // -> in 0-based half-open: [9_998_999, 10_001_000).
        let count_indexed = bcf_csi_region_count(&bcf, &csi, b"1\0", 9_998_999, 10_001_000);
        let count_scan = vcf_linear_scan_count(&vcf, "1", 9_999_000, 10_001_000);

        assert_eq!(
            count_indexed, count_scan,
            "BCF CSI region query returned {count_indexed}, linear scan found {count_scan}",
        );
        assert!(count_indexed > 0);

        cleanup(&bcf);
        cleanup(&csi);
    }
}

// --------------------------------------------------------------------------
// 4. CRAM + CRAI
//
// CRAM indexing differs structurally from BAI/CSI: a CRAI is normally built
// alongside the CRAM (via `sam_idx_init` + `sam_idx_save` during writing),
// not post-hoc.  Building a fresh CRAI by re-reading a finished CRAM via
// `sam_index_build3` is supported but the read path needs careful reference
// setup, and the test corpus's `range.cram` already ships a `.crai` we can
// validate against.  We therefore exercise the "load index -> region query
// -> expected count" half of the round trip with the shipped CRAI and
// verify the records returned match a linear scan of the CRAM over the same
// region.  This proves the load+query path end to end.
// --------------------------------------------------------------------------

#[test]
fn round_trip_cram_crai_region_query_matches_linear_scan() {
    unsafe {
        let cram = fixture("htslib/test/range.cram");
        let ref_path = fixture("htslib/test/ce.fa");
        let cram_c = c_path(&cram);
        let ref_c = c_path(&ref_path);
        let region = b"CHROMOSOME_II:2976-3070\0";

        let indexed = collect_cram_region_qnames(&cram_c, &ref_c, region);
        assert!(!indexed.is_empty(), "shipped CRAI returned no records");

        let scan =
            cram_linear_scan_qnames_in_region(&cram_c, &ref_c, "CHROMOSOME_II", 2_976 - 1, 3_070);

        let mut indexed_sorted = indexed.clone();
        let mut scan_sorted = scan.clone();
        indexed_sorted.sort();
        scan_sorted.sort();
        assert_eq!(
            indexed_sorted, scan_sorted,
            "CRAI region query disagrees with linear scan",
        );
    }
}

unsafe fn collect_cram_region_qnames(cram_c: &[u8], ref_c: &[u8], region: &[u8]) -> Vec<Vec<u8>> {
    let fp = hts_open(cram_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!fp.is_null());
    assert_eq!(hts_set_fai_filename(fp, ref_c.as_ptr().cast()), 0);
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());

    let idx = sam_index_load(fp, cram_c.as_ptr().cast());
    assert!(!idx.is_null(), "failed to load CRAI for {:?}", cram_c);

    let itr = sam_itr_querys(idx, hdr, region.as_ptr().cast());
    assert!(!itr.is_null());
    let rec = bam_init1();
    let mut out = Vec::new();
    while sam_itr_next(fp, itr, rec) >= 0 {
        let qname = bam_get_qname(rec);
        let len = (0..).take_while(|&i| *qname.add(i) != 0).count();
        out.push(std::slice::from_raw_parts(qname.cast::<u8>(), len).to_vec());
    }

    bam_destroy1(rec);
    hts_itr_destroy(itr);
    hts_idx_destroy(idx);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

unsafe fn cram_linear_scan_qnames_in_region(
    cram_c: &[u8],
    ref_c: &[u8],
    contig_name: &str,
    start: i64,
    end_exclusive: i64,
) -> Vec<Vec<u8>> {
    let fp = hts_open(cram_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!fp.is_null());
    assert_eq!(hts_set_fai_filename(fp, ref_c.as_ptr().cast()), 0);
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());

    let target_name = contig_name.as_bytes();
    let mut target_tid: i32 = -1;
    for i in 0..(*hdr).n_targets {
        let name_ptr = (*(*hdr).target_name.add(i as usize)).cast::<u8>();
        let len = (0..).take_while(|&j| *name_ptr.add(j) != 0).count();
        let name = std::slice::from_raw_parts(name_ptr, len);
        if name == target_name {
            target_tid = i;
            break;
        }
    }
    assert!(target_tid >= 0, "contig {contig_name} not in CRAM header");

    let rec = bam_init1();
    assert!(!rec.is_null());
    let mut out = Vec::new();
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        if (*rec).core.tid != target_tid {
            continue;
        }
        let pos = (*rec).core.pos;
        let endpos = bam_endpos(rec);
        if endpos > start && pos < end_exclusive {
            let qname = bam_get_qname(rec);
            let len = (0..).take_while(|&i| *qname.add(i) != 0).count();
            out.push(std::slice::from_raw_parts(qname.cast::<u8>(), len).to_vec());
        }
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}
