use htslib_rs::{
    fai_adjust_region, fai_destroy, fai_fetch64, fai_line_length, fai_load3_format,
    fai_parse_region, faidx_fetch_qual64, faidx_fetch_seq64, faidx_has_seq, faidx_iseq,
    faidx_seq_len, faidx_seq_len64, hts_pos_t, FAI_FASTA, FAI_FASTQ,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn c_fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let mut bytes = path.to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

fn unique_temp_path(name: &str, extension: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "htslib_rs-faidx-edge-{name}-{}-{nanos}.{extension}",
        std::process::id()
    ))
}

unsafe fn load_fastq_fai() -> *mut htslib_rs::faidx_t {
    let input = c_fixture("htslib/test/faidx/fastqs.fq");
    let index = c_fixture("htslib/test/faidx/fastqs.fq.expected.fai");
    let fai = fai_load3_format(
        input.as_ptr().cast(),
        index.as_ptr().cast(),
        std::ptr::null(),
        0,
        FAI_FASTQ,
    );
    assert!(!fai.is_null());
    fai
}

unsafe fn load_fasta_fai() -> *mut htslib_rs::faidx_t {
    let input = c_fixture("htslib/test/faidx/faidx.fa");
    let index = c_fixture("htslib/test/faidx/faidx.fa.expected.fai");
    let fai = fai_load3_format(
        input.as_ptr().cast(),
        index.as_ptr().cast(),
        std::ptr::null(),
        0,
        FAI_FASTA,
    );
    assert!(!fai.is_null());
    fai
}

unsafe fn fetched_string(ptr: *mut u8, len: hts_pos_t) -> String {
    assert!(!ptr.is_null());
    assert!(len >= 0);
    let bytes = std::slice::from_raw_parts(ptr, len as usize).to_vec();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    assert_eq!(text.len(), len as usize);
    text
}

fn append_wrapped(out: &mut String, text: &str) {
    for chunk in text.as_bytes().chunks(50) {
        out.push_str(std::str::from_utf8(chunk).unwrap());
        out.push('\n');
    }
}

unsafe fn format_adjusted_fastq_records(
    fai: *const htslib_rs::faidx_t,
    regions: &[&'static [u8]],
) -> String {
    let mut out = String::new();
    for &region in regions {
        let mut tid = 0;
        let mut beg = 0;
        let mut end = 0;
        assert!(
            !fai_parse_region(fai, region.as_ptr().cast(), &mut tid, &mut beg, &mut end, 0)
                .is_null()
        );
        assert!(fai_adjust_region(&*fai, tid, &mut beg, &mut end) >= 0);

        let mut name = faidx_iseq(&*fai, tid).unwrap().to_vec();
        name.push(0);
        let mut seq_len = 0;
        let seq = fetched_string(
            faidx_fetch_seq64(fai, name.as_ptr().cast(), beg, end - 1, &mut seq_len).cast(),
            seq_len,
        );
        let mut qual_len = 0;
        let qual = fetched_string(
            faidx_fetch_qual64(fai, name.as_ptr().cast(), beg, end - 1, &mut qual_len).cast(),
            qual_len,
        );
        assert_eq!(seq_len, qual_len);

        out.push('@');
        out.push_str(std::str::from_utf8(&region[..region.len() - 1]).unwrap());
        out.push_str(" length: ");
        out.push_str(&seq_len.to_string());
        out.push('\n');
        append_wrapped(&mut out, &seq);
        out.push_str("+\n");
        append_wrapped(&mut out, &qual);
    }
    out
}

unsafe fn format_adjusted_fasta_records(
    fai: *const htslib_rs::faidx_t,
    regions: &[&'static [u8]],
) -> String {
    let mut out = String::new();
    for &region in regions {
        let mut tid = 0;
        let mut beg = 0;
        let mut end = 0;
        assert!(
            !fai_parse_region(fai, region.as_ptr().cast(), &mut tid, &mut beg, &mut end, 0)
                .is_null()
        );
        assert!(fai_adjust_region(&*fai, tid, &mut beg, &mut end) >= 0);

        let mut name = faidx_iseq(&*fai, tid).unwrap().to_vec();
        name.push(0);
        let mut seq_len = 0;
        let seq = fetched_string(
            faidx_fetch_seq64(fai, name.as_ptr().cast(), beg, end - 1, &mut seq_len).cast(),
            seq_len,
        );

        out.push('>');
        out.push_str(std::str::from_utf8(&region[..region.len() - 1]).unwrap());
        out.push_str(" length: ");
        out.push_str(&seq_len.to_string());
        out.push('\n');
        append_wrapped(&mut out, &seq);
    }
    out
}

unsafe fn fetch_region_len(fai: *const htslib_rs::faidx_t, region: &'static [u8]) -> hts_pos_t {
    let mut len = 0;
    let seq = fai_fetch64(fai, region.as_ptr().cast(), &mut len);
    let _ = fetched_string(seq.cast(), len);
    len
}

#[test]
fn original_fastq_faidx_tst_named_helper_edges_match_expected_index() {
    unsafe {
        let fai = load_fastq_fai();

        assert_eq!(fai_line_length(fai, b"FAKE0005_3\0".as_ptr().cast()), 63);
        assert_eq!(
            fai_line_length(fai, b"SRR014849.203935_3\0".as_ptr().cast()),
            144
        );
        assert_eq!(
            faidx_has_seq(fai, b"SRR014849.203935_3\0".as_ptr().cast()),
            1
        );
        assert_eq!(faidx_has_seq(fai, b"absent\0".as_ptr().cast()), 0);
        assert_eq!(faidx_iseq(&*fai, 0).unwrap(), b"FAKE0005_1");
        assert_eq!(
            faidx_seq_len(fai, b"FSRRS4401CM938_1\0".as_ptr().cast()),
            453
        );
        assert_eq!(
            faidx_seq_len64(fai, b"FSRRS4401AOV6A_4\0".as_ptr().cast()),
            309
        );

        fai_destroy(fai);
    }
}

#[test]
fn original_fastq_faidx_adjust_region_matches_expected_fastq_output() {
    const EXPECTED_FASTQ: &str = include_str!("../htslib/test/faidx/fastqs.1.expected.fq");
    const REGIONS: &[&[u8]] = &[
        b"FAKE0006_1:4-12\0",
        b"FSRRS4401BE7HA_1:81-120\0",
        b"FAKE0010_2\0",
        b"SRR014849.50939_3:71-90\0",
    ];

    unsafe {
        let fai = load_fastq_fai();
        assert_eq!(format_adjusted_fastq_records(fai, REGIONS), EXPECTED_FASTQ);
        fai_destroy(fai);
    }
}

#[test]
fn original_fastq_faidx_adjust_region_matches_expected_fasta_output() {
    const EXPECTED_FASTA: &str = include_str!("../htslib/test/faidx/fastqs.2.expected.fa");
    const REGIONS: &[&[u8]] = &[
        b"FAKE0006_1:4-12\0",
        b"FSRRS4401BE7HA_1:81-120\0",
        b"FAKE0010_2\0",
        b"SRR014849.50939_3:71-90\0",
    ];

    unsafe {
        let fai = load_fastq_fai();
        assert_eq!(format_adjusted_fasta_records(fai, REGIONS), EXPECTED_FASTA);
        fai_destroy(fai);
    }
}

#[test]
fn original_fasta_faidx_adjust_region_matches_expected_fasta_output() {
    const EXPECTED_FASTA: &str = include_str!("../htslib/test/faidx/faidx.1.expected.fa");
    const REGIONS: &[&[u8]] = &[
        b"trailingblank2:28-33\0",
        b"trailingblank3:4-5\0",
        b"bar:4-5\0",
    ];

    unsafe {
        let fai = load_fasta_fai();
        assert_eq!(format_adjusted_fasta_records(fai, REGIONS), EXPECTED_FASTA);
        fai_destroy(fai);
    }
}

#[test]
fn original_fastq_faidx_whole_record_and_slice_lengths_match_expected_regions() {
    unsafe {
        let fai = load_fastq_fai();

        assert_eq!(fetch_region_len(fai, b"FAKE0006_1:4-12\0"), 9);
        assert_eq!(fetch_region_len(fai, b"FSRRS4401BE7HA_1:81-120\0"), 40);
        assert_eq!(fetch_region_len(fai, b"FAKE0010_2\0"), 30);
        assert_eq!(fetch_region_len(fai, b"SRR014849.50939_3:71-90\0"), 20);

        fai_destroy(fai);
    }
}

#[test]
fn original_fasta_faidx_handles_empty_name_and_crlf_line_metadata() {
    unsafe {
        let input = c_fixture("htslib/test/faidx/faidx.fa");
        let index = c_fixture("htslib/test/faidx/faidx.fa.expected.fai");
        let fai = fai_load3_format(
            input.as_ptr().cast(),
            index.as_ptr().cast(),
            std::ptr::null(),
            0,
            FAI_FASTA,
        );
        assert!(!fai.is_null());

        assert_eq!(faidx_has_seq(fai, b"\0".as_ptr().cast()), 1);
        assert_eq!(faidx_seq_len64(fai, b"\0".as_ptr().cast()), 4);
        assert_eq!(faidx_iseq(&*fai, 0).unwrap(), b"");

        let mut len = 0;
        let seq = faidx_fetch_seq64(fai, b"\0".as_ptr().cast(), 0, 3, &mut len);
        assert_eq!(fetched_string(seq.cast(), len), "ATGC");

        assert_eq!(faidx_seq_len64(fai, b"trailingblank3\0".as_ptr().cast()), 5);
        assert_eq!(fai_line_length(fai, b"trailingblank3\0".as_ptr().cast()), 4);
        let seq = fai_fetch64(fai, b"trailingblank3:1-5\0".as_ptr().cast(), &mut len);
        assert_eq!(fetched_string(seq.cast(), len), "ACGTA");

        fai_destroy(fai);
    }
}

#[test]
fn original_fai_read_matches_sscanf_trailing_final_field_junk() {
    unsafe {
        let fasta = unique_temp_path("sscanf-fasta", "fa");
        let fasta_fai = fasta.with_extension("fa.fai");
        std::fs::write(&fasta, b">chr1\nACGT\n").unwrap();
        std::fs::write(&fasta_fai, b"chr1\t4\t6\t4\t5junk\n").unwrap();

        let mut fasta_c = fasta.to_string_lossy().into_owned().into_bytes();
        fasta_c.push(0);
        let fai = fai_load3_format(
            fasta_c.as_ptr().cast(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            FAI_FASTA,
        );
        assert!(!fai.is_null());
        assert_eq!(faidx_seq_len64(fai, b"chr1\0".as_ptr().cast()), 4);
        fai_destroy(fai);

        let fastq = unique_temp_path("sscanf-fastq", "fq");
        let fastq_fai = fastq.with_extension("fq.fai");
        std::fs::write(&fastq, b"@r1\nACGT\n+\n!!!!\n").unwrap();
        std::fs::write(&fastq_fai, b"r1\t4\t4\t4\t5\t11junk\n").unwrap();

        let mut fastq_c = fastq.to_string_lossy().into_owned().into_bytes();
        fastq_c.push(0);
        let fai = fai_load3_format(
            fastq_c.as_ptr().cast(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            FAI_FASTQ,
        );
        assert!(!fai.is_null());
        assert_eq!(faidx_seq_len64(fai, b"r1\0".as_ptr().cast()), 4);
        fai_destroy(fai);

        let _ = std::fs::remove_file(fasta);
        let _ = std::fs::remove_file(fasta_fai);
        let _ = std::fs::remove_file(fastq);
        let _ = std::fs::remove_file(fastq_fai);
    }
}

#[test]
fn original_fai_read_rejects_trailing_junk_before_required_fields() {
    unsafe {
        let fasta = unique_temp_path("sscanf-bad-middle", "fa");
        let fasta_fai = fasta.with_extension("fa.fai");
        std::fs::write(&fasta, b">chr1\nACGT\n").unwrap();
        std::fs::write(&fasta_fai, b"chr1\t4\t6junk\t4\t5\n").unwrap();

        let mut fasta_c = fasta.to_string_lossy().into_owned().into_bytes();
        fasta_c.push(0);
        let fai = fai_load3_format(
            fasta_c.as_ptr().cast(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            FAI_FASTA,
        );
        assert!(fai.is_null());

        let _ = std::fs::remove_file(fasta);
        let _ = std::fs::remove_file(fasta_fai);
    }
}
