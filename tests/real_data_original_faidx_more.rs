use htslib_rs::{
    bgzf::{bgzf_close, bgzf_index_build_init, bgzf_index_dump, bgzf_open, bgzf_write},
    fai_adjust_region, fai_build3, fai_destroy, fai_fetch, fai_fetch64, fai_fetchqual,
    fai_fetchqual64, fai_line_length, fai_load3_format, fai_parse_region, faidx_fetch_qual64,
    faidx_fetch_seq64, faidx_has_seq, faidx_iseq, faidx_nseq, faidx_seq_len, faidx_seq_len64,
    hts_pos_t, FAI_CREATE, FAI_FASTA, FAI_FASTQ, FAI_NONE,
};
use std::ffi::{CStr, CString};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn load_fai(
    path: &str,
    index: &str,
    format: htslib_rs::fai_format_options,
) -> *mut htslib_rs::faidx_t {
    let input = c_fixture(path);
    let index = c_fixture(index);
    let fai = fai_load3_format(input.as_ptr(), index.as_ptr(), std::ptr::null(), 0, format);
    assert!(!fai.is_null());
    fai
}

fn unique_temp_path(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "htslib_rs-faidx-{name}-{}-{nanos}",
        std::process::id()
    ))
}

unsafe fn bgzip_fixture_with_gzi(
    input: &str,
    name: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let source =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(input)).unwrap();
    let bgz_path = unique_temp_path(name).with_extension("fa.gz");
    let gzi_path = bgz_path.with_extension("fa.gz.gzi");
    let bgz_c = CString::new(bgz_path.to_string_lossy().as_bytes()).unwrap();
    let gzi_c = CString::new(gzi_path.to_string_lossy().as_bytes()).unwrap();

    let fp = bgzf_open(bgz_c.as_ptr(), c"w".as_ptr());
    assert!(!fp.is_null());
    assert_eq!(bgzf_index_build_init(fp), 0);
    assert_eq!(
        bgzf_write(fp, source.as_ptr().cast(), source.len()),
        source.len() as isize
    );
    assert_eq!(bgzf_index_dump(fp, gzi_c.as_ptr(), std::ptr::null()), 0);
    assert_eq!(bgzf_close(fp), 0);

    (bgz_path, gzi_path)
}

#[derive(Debug)]
struct ExpectedIndexRow {
    name: String,
    len: i64,
    line_blen: i64,
}

fn expected_index_rows(text: &str) -> Vec<ExpectedIndexRow> {
    text.lines()
        .map(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            assert!(
                fields.len() == 5 || fields.len() == 6,
                "bad fai row: {line}"
            );
            ExpectedIndexRow {
                name: fields[0].to_string(),
                len: fields[1].parse().unwrap(),
                line_blen: fields[3].parse().unwrap(),
            }
        })
        .collect()
}

unsafe fn fetched_string(ptr: *mut libc::c_char, len: hts_pos_t) -> String {
    assert!(!ptr.is_null());
    assert!(len >= 0);
    let text = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    assert_eq!(text.len(), len as usize);
    libc::free(ptr.cast());
    text
}

fn append_wrapped(out: &mut String, text: &str) {
    for chunk in text.as_bytes().chunks(50) {
        out.push_str(std::str::from_utf8(chunk).unwrap());
        out.push('\n');
    }
}

unsafe fn format_fasta_records(
    fai: *const htslib_rs::faidx_t,
    regions: &[&'static CStr],
    fetch: unsafe fn(*const htslib_rs::faidx_t, &'static CStr) -> (String, hts_pos_t),
) -> String {
    let mut out = String::new();
    for &region in regions {
        let (seq, len) = fetch(fai, region);
        out.push('>');
        out.push_str(region.to_str().unwrap());
        out.push_str(" length: ");
        out.push_str(&len.to_string());
        out.push('\n');
        append_wrapped(&mut out, &seq);
    }
    out
}

unsafe fn format_fastq_records(
    fai: *const htslib_rs::faidx_t,
    regions: &[&'static CStr],
    fetch: unsafe fn(*const htslib_rs::faidx_t, &'static CStr) -> (String, String, hts_pos_t),
) -> String {
    let mut out = String::new();
    for &region in regions {
        let (seq, qual, len) = fetch(fai, region);
        assert_eq!(seq.len(), qual.len());
        out.push('@');
        out.push_str(region.to_str().unwrap());
        out.push_str(" length: ");
        out.push_str(&len.to_string());
        out.push('\n');
        append_wrapped(&mut out, &seq);
        out.push_str("+\n");
        append_wrapped(&mut out, &qual);
    }
    out
}

unsafe fn fetch64_record(
    fai: *const htslib_rs::faidx_t,
    region: &'static CStr,
) -> (String, hts_pos_t) {
    let mut len = 0;
    let seq = fai_fetch64(fai, region.as_ptr(), &mut len);
    (fetched_string(seq, len), len)
}

unsafe fn fetch32_record(
    fai: *const htslib_rs::faidx_t,
    region: &'static CStr,
) -> (String, hts_pos_t) {
    let mut len = 0;
    let seq = fai_fetch(fai, region.as_ptr(), &mut len);
    (fetched_string(seq, len as hts_pos_t), len as hts_pos_t)
}

unsafe fn adjusted_fetch_seq64_record(
    fai: *const htslib_rs::faidx_t,
    region: &'static CStr,
) -> (String, hts_pos_t) {
    let mut tid = 0;
    let mut beg = 0;
    let mut end = 0;
    assert!(!fai_parse_region(fai, region.as_ptr(), &mut tid, &mut beg, &mut end, 0).is_null());

    let orig_beg = beg;
    let orig_end = end;
    let adjusted = fai_adjust_region(&*fai, tid, &mut beg, &mut end);
    assert!(adjusted >= 0);
    assert_eq!((adjusted & 1) != 0, beg != orig_beg);
    assert_eq!((adjusted & 2) != 0, end != orig_end);

    let mut len = 0;
    let name = CString::new(faidx_iseq(&*fai, tid).unwrap()).unwrap();
    let seq = faidx_fetch_seq64(fai, name.as_ptr(), beg, end - 1, &mut len);
    (fetched_string(seq, len), len)
}

unsafe fn fastq_fetch64_record(
    fai: *const htslib_rs::faidx_t,
    region: &'static CStr,
) -> (String, String, hts_pos_t) {
    let mut seq_len = 0;
    let seq = fetched_string(fai_fetch64(fai, region.as_ptr(), &mut seq_len), seq_len);

    let mut qual_len = 0;
    let qual = fetched_string(
        fai_fetchqual64(fai, region.as_ptr(), &mut qual_len),
        qual_len,
    );
    assert_eq!(seq_len, qual_len);

    (seq, qual, seq_len)
}

unsafe fn fastq_fetch32_record(
    fai: *const htslib_rs::faidx_t,
    region: &'static CStr,
) -> (String, String, hts_pos_t) {
    let mut seq_len = 0;
    let seq = fetched_string(
        fai_fetch(fai, region.as_ptr(), &mut seq_len),
        seq_len as hts_pos_t,
    );

    let mut qual_len = 0;
    let qual = fetched_string(
        fai_fetchqual(fai, region.as_ptr(), &mut qual_len),
        qual_len as hts_pos_t,
    );
    assert_eq!(seq_len, qual_len);

    (seq, qual, seq_len as hts_pos_t)
}

unsafe fn fastq_fetch_seq64_record(
    fai: *const htslib_rs::faidx_t,
    region: &'static CStr,
) -> (String, String, hts_pos_t) {
    let mut tid = 0;
    let mut beg = 0;
    let mut end = 0;
    assert!(!fai_parse_region(fai, region.as_ptr(), &mut tid, &mut beg, &mut end, 0).is_null());

    let name = CString::new(faidx_iseq(&*fai, tid).unwrap()).unwrap();
    let mut seq_len = 0;
    let seq = fetched_string(
        faidx_fetch_seq64(fai, name.as_ptr(), beg, end - 1, &mut seq_len),
        seq_len,
    );

    let mut qual_len = 0;
    let qual = fetched_string(
        faidx_fetch_qual64(fai, name.as_ptr(), beg, end - 1, &mut qual_len),
        qual_len,
    );
    assert_eq!(seq_len, qual_len);

    (seq, qual, seq_len)
}

fn generated_fastq_path(record_count: usize, label: &str) -> std::path::PathBuf {
    let path = unique_temp_path(label).with_extension("fq");
    let bases = b"ACGTN";
    let quals = b"BCDEF";
    let mut out = String::with_capacity(record_count * 180);
    for i in 0..record_count {
        out.push_str(&format!("@read_{i:06}\n"));
        for j in 0..96 {
            out.push(bases[(i + j) % bases.len()] as char);
        }
        out.push_str("\n+\n");
        for j in 0..96 {
            out.push(quals[(i + j) % quals.len()] as char);
        }
        out.push('\n');
    }
    std::fs::write(&path, out).unwrap();
    path
}

fn run_read_fast_index(path: &std::path::Path, fmt: &str, multi: &str, regions: &str) -> Vec<u8> {
    let output = Command::new(env!("CARGO_BIN_EXE_perf_read_fast_index"))
        .arg(path)
        .arg(fmt)
        .arg(multi)
        .arg(regions)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "read_fast_index failed with {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

unsafe fn expected_read_fast_index_multi_fastq(path: &std::path::Path, regions: &str) -> Vec<u8> {
    const HTS_PARSE_LIST: i32 = 4;

    let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
    let fai_path = CString::new(format!("{}.fai", path.display())).unwrap();
    let fai = fai_load3_format(
        path_c.as_ptr(),
        fai_path.as_ptr(),
        std::ptr::null(),
        0,
        FAI_FASTQ,
    );
    assert!(!fai.is_null());

    let mut region_storage = CString::new(regions).unwrap().into_bytes_with_nul();
    let mut region = region_storage.as_mut_ptr().cast::<libc::c_char>();
    let mut expected = Vec::new();
    loop {
        let mut tid = -1;
        let mut beg = 0;
        let mut end = 0;
        let remaining = fai_parse_region(fai, region, &mut tid, &mut beg, &mut end, HTS_PARSE_LIST);
        if remaining.is_null() {
            break;
        }
        assert_eq!(fai_adjust_region(&*fai, tid, &mut beg, &mut end), 0);

        let name = CString::new(faidx_iseq(&*fai, tid).unwrap()).unwrap();
        let mut len = 0;
        let seq = faidx_fetch_seq64(fai, name.as_ptr(), beg, end, &mut len);
        assert!(!seq.is_null());
        expected.extend_from_slice(
            format!("Data: {len} {}\n", CStr::from_ptr(seq).to_string_lossy()).as_bytes(),
        );
        libc::free(seq.cast());

        let qual = faidx_fetch_qual64(fai, name.as_ptr(), beg, end, &mut len);
        assert!(!qual.is_null());
        expected.extend_from_slice(
            format!("Qual: {len} {}\n", CStr::from_ptr(qual).to_string_lossy()).as_bytes(),
        );
        libc::free(qual.cast());

        region = remaining.cast_mut();
    }

    fai_destroy(fai);
    expected
}

#[test]
fn fastq_expected_index_metadata_matches_faidx_apis() {
    unsafe {
        let fai = load_fai(
            "htslib/test/faidx/fastqs.fq",
            "htslib/test/faidx/fastqs.fq.expected.fai",
            FAI_FASTQ,
        );
        let rows = expected_index_rows(include_str!("../htslib/test/faidx/fastqs.fq.expected.fai"));

        assert_eq!(faidx_nseq(fai), rows.len() as i32);
        for (i, row) in rows.iter().enumerate() {
            let name = CString::new(row.name.as_bytes()).unwrap();
            assert_eq!(faidx_iseq(&*fai, i as i32).unwrap(), row.name.as_bytes());
            assert_eq!(faidx_has_seq(fai, name.as_ptr()), 1);
            assert_eq!(faidx_seq_len(fai, name.as_ptr()), row.len as i32);
            assert_eq!(faidx_seq_len64(fai, name.as_ptr()), row.len);
            assert_eq!(fai_line_length(fai, name.as_ptr()), row.line_blen);
        }
        assert_eq!(faidx_has_seq(fai, c"absent".as_ptr()), 0);

        fai_destroy(fai);
    }
}

#[test]
fn fai_none_format_matches_original_existing_index_reading_rules() {
    unsafe {
        let fastq = c_fixture("htslib/test/faidx/fastqs.fq");
        let fastq_index = c_fixture("htslib/test/faidx/fastqs.fq.expected.fai");
        let fai = fai_load3_format(
            fastq.as_ptr(),
            fastq_index.as_ptr(),
            std::ptr::null(),
            0,
            FAI_NONE,
        );
        assert!(!fai.is_null());
        assert_eq!(faidx_nseq(fai), 105);
        assert_eq!(faidx_iseq(&*fai, 0).unwrap(), c"FAKE0005_1".to_bytes());
        fai_destroy(fai);

        let fasta = c_fixture("htslib/test/faidx/faidx.fa");
        let fasta_index = c_fixture("htslib/test/faidx/faidx.fa.expected.fai");
        let fai = fai_load3_format(
            fasta.as_ptr(),
            fasta_index.as_ptr(),
            std::ptr::null(),
            0,
            FAI_NONE,
        );
        assert!(fai.is_null());
    }
}

#[test]
fn fasta_retrieval_expected_output_matches_original_ce_fixture() {
    const EXPECTED: &str = include_str!("../htslib/test/faidx/ce.1.expected.fa");
    const REGIONS: &[&CStr] = &[c"CHROMOSOME_I:5001-5125", c"CHROMOSOME_X:101-225"];

    unsafe {
        let fai = load_fai("htslib/test/ce.fa", "htslib/test/ce.fa.fai", FAI_FASTA);

        assert_eq!(format_fasta_records(fai, REGIONS, fetch64_record), EXPECTED);
        assert_eq!(format_fasta_records(fai, REGIONS, fetch32_record), EXPECTED);
        assert_eq!(
            format_fasta_records(fai, REGIONS, adjusted_fetch_seq64_record),
            EXPECTED
        );

        fai_destroy(fai);
    }
}

#[test]
fn bgzipped_fasta_retrieval_uses_gzi_virtual_offsets_for_ce_fixture() {
    const EXPECTED: &str = include_str!("../htslib/test/faidx/ce.1.expected.fa");
    const REGIONS: &[&CStr] = &[c"CHROMOSOME_I:5001-5125", c"CHROMOSOME_X:101-225"];

    unsafe {
        let (bgz_path, gzi_path) = bgzip_fixture_with_gzi("htslib/test/ce.fa", "ce");
        let bgz_c = CString::new(bgz_path.to_string_lossy().as_bytes()).unwrap();
        let fai_c = c_fixture("htslib/test/ce.fa.fai");
        let gzi_c = CString::new(gzi_path.to_string_lossy().as_bytes()).unwrap();
        let fai = fai_load3_format(bgz_c.as_ptr(), fai_c.as_ptr(), gzi_c.as_ptr(), 0, FAI_FASTA);
        assert!(!fai.is_null());

        assert_eq!(format_fasta_records(fai, REGIONS, fetch64_record), EXPECTED);
        assert_eq!(format_fasta_records(fai, REGIONS, fetch32_record), EXPECTED);
        assert_eq!(
            format_fasta_records(fai, REGIONS, adjusted_fetch_seq64_record),
            EXPECTED
        );

        fai_destroy(fai);
        let _ = std::fs::remove_file(bgz_path);
        let _ = std::fs::remove_file(gzi_path);
    }
}

#[test]
fn bgzipped_fasta_existing_fai_missing_gzi_rebuilds_with_create_flag() {
    unsafe {
        let source = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/test/faidx/faidx.fa"),
        )
        .unwrap();
        let bgz_path = unique_temp_path("faidx-missing-gzi").with_extension("fa.gz");
        let fai_path = std::path::PathBuf::from(format!("{}.fai", bgz_path.to_string_lossy()));
        let gzi_path = std::path::PathBuf::from(format!("{}.gzi", bgz_path.to_string_lossy()));
        let bgz_c = CString::new(bgz_path.to_string_lossy().as_bytes()).unwrap();

        let fp = bgzf_open(bgz_c.as_ptr(), c"w".as_ptr());
        assert!(!fp.is_null());
        assert_eq!(
            bgzf_write(fp, source.as_ptr().cast(), source.len()),
            source.len() as isize
        );
        assert_eq!(bgzf_close(fp), 0);

        let fai = fai_load3_format(
            bgz_c.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            FAI_CREATE,
            FAI_FASTA,
        );
        assert!(!fai.is_null());
        fai_destroy(fai);
        assert!(fai_path.exists());
        assert!(gzi_path.exists());

        std::fs::remove_file(&gzi_path).unwrap();
        let fai = fai_load3_format(
            bgz_c.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            FAI_CREATE,
            FAI_FASTA,
        );
        assert!(!fai.is_null());
        assert!(gzi_path.exists());

        let mut len = 0;
        let seq = faidx_fetch_seq64(fai, c"foo".as_ptr(), 0, 7, &mut len);
        assert_eq!(fetched_string(seq, len), "TGCATGCA");

        fai_destroy(fai);
        let _ = std::fs::remove_file(bgz_path);
        let _ = std::fs::remove_file(fai_path);
        let _ = std::fs::remove_file(gzi_path);
    }
}

#[test]
fn fastq_retrieval_expected_output_matches_sequence_and_quality_apis() {
    const EXPECTED_FASTQ: &str = include_str!("../htslib/test/faidx/fastqs.1.expected.fq");
    const EXPECTED_FASTA: &str = include_str!("../htslib/test/faidx/fastqs.2.expected.fa");
    const REGIONS: &[&CStr] = &[
        c"FAKE0006_1:4-12",
        c"FSRRS4401BE7HA_1:81-120",
        c"FAKE0010_2",
        c"SRR014849.50939_3:71-90",
    ];

    unsafe {
        let fai = load_fai(
            "htslib/test/faidx/fastqs.fq",
            "htslib/test/faidx/fastqs.fq.expected.fai",
            FAI_FASTQ,
        );

        assert_eq!(
            format_fastq_records(fai, REGIONS, fastq_fetch64_record),
            EXPECTED_FASTQ
        );
        assert_eq!(
            format_fastq_records(fai, REGIONS, fastq_fetch32_record),
            EXPECTED_FASTQ
        );
        assert_eq!(
            format_fastq_records(fai, REGIONS, fastq_fetch_seq64_record),
            EXPECTED_FASTQ
        );
        assert_eq!(
            format_fasta_records(fai, REGIONS, fetch64_record),
            EXPECTED_FASTA
        );

        fai_destroy(fai);
    }
}

#[test]
fn generated_large_fastq_read_fast_index_cli_matches_faidx_api() {
    unsafe {
        let path = generated_fastq_path(20_000, "large-read-fast-index");
        let fai_path = std::path::PathBuf::from(format!("{}.fai", path.display()));
        let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let fai_c = CString::new(fai_path.to_string_lossy().as_bytes()).unwrap();
        assert_eq!(
            fai_build3(path_c.as_ptr(), fai_c.as_ptr(), std::ptr::null()),
            0
        );

        let regions = "read_000000:1-96,read_010000:11-70,read_019999:30-96";
        let actual = run_read_fast_index(&path, "Q", "1", regions);
        let expected = expected_read_fast_index_multi_fastq(&path, regions);
        assert_eq!(actual, expected);

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(fai_path);
    }
}
