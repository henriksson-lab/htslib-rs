use htslib_rs::{
    bgzf::{
        bgzf_check_EOF, bgzf_close, bgzf_compression, bgzf_getline, bgzf_is_bgzf, bgzf_open,
        bgzf_read,
    },
    fai_destroy, fai_fetch, fai_fetchqual, fai_line_length, fai_load3_format,
    faidx_fetch_qual64, faidx_fetch_seq64, faidx_free_returned_c_bytes, faidx_has_seq, faidx_iseq,
    faidx_nseq, faidx_seq_len, hclose, hopen, htsFormat, hts_detect_format2, kstring_t, FAI_FASTA,
    FAI_FASTQ, HTS_COMPRESSION_NO_COMPRESSION, HTS_FORMAT_FAI_FORMAT, HTS_FORMAT_FASTA_FORMAT,
    HTS_FORMAT_FASTQ_FORMAT, HTS_FORMAT_FQI_FORMAT, HTS_FORMAT_SEQUENCE_DATA,
    HTS_FORMAT_TEXT_FORMAT,
};

/// Build a NUL-terminated byte buffer for a fixture path so it can be passed to
/// production APIs that still take a raw `*const c_char` (they convert in parallel).
fn c_fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let mut bytes = path.to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

fn detect_fixture_format(path: &str) -> htsFormat {
    detect_fixture_format_with_name(path, path)
}

fn detect_fixture_format_with_name(path: &str, format_name: &str) -> htsFormat {
    unsafe {
        let path = c_fixture(path);
        let format_name = c_fixture(format_name);
        let fp = hopen(path.as_ptr().cast(), c"r".as_ptr());
        assert!(!fp.is_null());

        let mut fmt: htsFormat = std::mem::zeroed();
        assert_eq!(
            hts_detect_format2(fp, format_name.as_ptr().cast(), &mut fmt),
            0
        );
        assert_eq!(hclose(fp), 0);
        fmt
    }
}

unsafe fn load_indexed_fasta(path: &str, index: &str) -> *mut htslib_rs::faidx_t {
    let fasta = c_fixture(path);
    let fai_path = c_fixture(index);
    let fai = fai_load3_format(
        fasta.as_ptr().cast(),
        fai_path.as_ptr().cast(),
        std::ptr::null(),
        0,
        FAI_FASTA,
    );
    assert!(!fai.is_null());
    fai
}

unsafe fn load_indexed_fastq(path: &str, index: &str) -> *mut htslib_rs::faidx_t {
    let fastq = c_fixture(path);
    let fai_path = c_fixture(index);
    let fai = fai_load3_format(
        fastq.as_ptr().cast(),
        fai_path.as_ptr().cast(),
        std::ptr::null(),
        0,
        FAI_FASTQ,
    );
    assert!(!fai.is_null());
    fai
}

unsafe fn fetch_text(fai: *const htslib_rs::faidx_t, region: &[u8]) -> String {
    let region = [region, &[0]].concat();
    let mut len = 0;
    let seq = fai_fetch(fai, region.as_ptr().cast(), &mut len);
    assert!(!seq.is_null());
    let bytes = std::slice::from_raw_parts(seq.cast::<u8>(), len as usize);
    let text = String::from_utf8_lossy(bytes).into_owned();
    assert_eq!(text.len(), len as usize);
    faidx_free_returned_c_bytes(seq);
    text
}

unsafe fn fetch_qual_text(fai: *const htslib_rs::faidx_t, region: &[u8]) -> String {
    let region = [region, &[0]].concat();
    let mut len = 0;
    let qual = fai_fetchqual(fai, region.as_ptr().cast(), &mut len);
    assert!(!qual.is_null());
    let bytes = std::slice::from_raw_parts(qual.cast::<u8>(), len as usize);
    let text = String::from_utf8_lossy(bytes).into_owned();
    assert_eq!(text.len(), len as usize);
    faidx_free_returned_c_bytes(qual);
    text
}

unsafe fn fetch_seq64_text(
    fai: *const htslib_rs::faidx_t,
    name: &[u8],
    beg: htslib_rs::hts_pos_t,
    end: htslib_rs::hts_pos_t,
) -> String {
    let name = [name, &[0]].concat();
    let mut len = 0;
    let seq = faidx_fetch_seq64(fai, name.as_ptr().cast(), beg, end, &mut len);
    assert!(!seq.is_null());
    let bytes = std::slice::from_raw_parts(seq.cast::<u8>(), len as usize);
    let text = String::from_utf8_lossy(bytes).into_owned();
    assert_eq!(text.len(), len as usize);
    faidx_free_returned_c_bytes(seq);
    text
}

unsafe fn fetch_qual64_text(
    fai: *const htslib_rs::faidx_t,
    name: &[u8],
    beg: htslib_rs::hts_pos_t,
    end: htslib_rs::hts_pos_t,
) -> String {
    let name = [name, &[0]].concat();
    let mut len = 0;
    let qual = faidx_fetch_qual64(fai, name.as_ptr().cast(), beg, end, &mut len);
    assert!(!qual.is_null());
    let bytes = std::slice::from_raw_parts(qual.cast::<u8>(), len as usize);
    let text = String::from_utf8_lossy(bytes).into_owned();
    assert_eq!(text.len(), len as usize);
    faidx_free_returned_c_bytes(qual);
    text
}

unsafe fn render_fasta_queries(fai: *const htslib_rs::faidx_t, queries: &[&[u8]]) -> String {
    let mut out = String::new();
    for query in queries {
        let seq = fetch_text(fai, query);
        out.push('>');
        out.push_str(&String::from_utf8_lossy(query));
        out.push_str(" length: ");
        out.push_str(&seq.len().to_string());
        out.push('\n');
        for chunk in seq.as_bytes().chunks(60) {
            out.push_str(std::str::from_utf8(chunk).unwrap());
            out.push('\n');
        }
    }
    out
}

unsafe fn render_fastq_queries(fai: *const htslib_rs::faidx_t, queries: &[&[u8]]) -> String {
    let mut out = String::new();
    for query in queries {
        let seq = fetch_text(fai, query);
        let qual = fetch_qual_text(fai, query);
        assert_eq!(seq.len(), qual.len());
        out.push('@');
        out.push_str(&String::from_utf8_lossy(query));
        out.push_str(" length: ");
        out.push_str(&seq.len().to_string());
        out.push('\n');
        out.push_str(&seq);
        out.push_str("\n+\n");
        out.push_str(&qual);
        out.push('\n');
    }
    out
}

#[test]
fn detects_sequence_and_index_fixture_formats() {
    let fasta = detect_fixture_format("htslib/test/faidx/faidx.fa");
    assert_eq!(fasta.category, HTS_FORMAT_SEQUENCE_DATA);
    assert_eq!(fasta.format, HTS_FORMAT_FASTA_FORMAT);
    assert_eq!(fasta.compression, HTS_COMPRESSION_NO_COMPRESSION);

    let fastq = detect_fixture_format("htslib/test/faidx/fastqs.fq");
    assert_eq!(fastq.category, HTS_FORMAT_SEQUENCE_DATA);
    assert_eq!(fastq.format, HTS_FORMAT_FASTQ_FORMAT);
    assert_eq!(fastq.compression, HTS_COMPRESSION_NO_COMPRESSION);

    let fai = detect_fixture_format("htslib/test/faidx/faidx.fa.expected.fai");
    assert_eq!(fai.format, HTS_FORMAT_FAI_FORMAT);

    let fqi = detect_fixture_format_with_name(
        "htslib/test/faidx/fastqs.fq.expected.fai",
        "htslib/test/faidx/fastqs.fq.expected.fqi",
    );
    assert_eq!(fqi.format, HTS_FORMAT_FQI_FORMAT);

    let plain_bgzip_input = detect_fixture_format("htslib/test/bgziptest.txt");
    assert_eq!(plain_bgzip_input.format, HTS_FORMAT_TEXT_FORMAT);
}

#[test]
fn reads_realn_fasta_indexes_with_exact_lengths_and_substrings() {
    unsafe {
        let realn01 = load_indexed_fasta("htslib/test/realn01.fa", "htslib/test/realn01.fa.fai");
        assert_eq!(faidx_nseq(realn01), 1);
        assert_eq!(faidx_iseq(&*realn01, 0), Some(b"000000F".as_slice()));
        assert_eq!(
            faidx_seq_len(realn01, [b"000000F".as_slice(), &[0]].concat().as_ptr().cast()),
            686
        );
        assert_eq!(fetch_text(realn01, b"000000F:1-12"), "CAGACAAACATA");
        assert_eq!(fetch_text(realn01, b"000000F:675-686"), "GATATTTGTTAT");
        fai_destroy(realn01);

        let realn02 = load_indexed_fasta("htslib/test/realn02.fa", "htslib/test/realn02.fa.fai");
        assert_eq!(faidx_nseq(realn02), 1);
        assert_eq!(faidx_iseq(&*realn02, 0), Some(b"17".as_slice()));
        assert_eq!(
            faidx_seq_len(realn02, [b"17".as_slice(), &[0]].concat().as_ptr().cast()),
            4200
        );
        assert_eq!(fetch_text(realn02, b"17:1-14"), "AAGCTTCTCACCCT");
        fai_destroy(realn02);

        let realn03 = load_indexed_fasta("htslib/test/realn03.fa", "htslib/test/realn03.fa.fai");
        assert_eq!(faidx_nseq(realn03), 1);
        assert_eq!(faidx_iseq(&*realn03, 0), Some(b"MX".as_slice()));
        assert_eq!(
            faidx_seq_len(realn03, [b"MX".as_slice(), &[0]].concat().as_ptr().cast()),
            11
        );
        assert_eq!(fetch_text(realn03, b"MX:1-11"), "CGTCTACTACG");
        fai_destroy(realn03);
    }
}

#[test]
fn fetches_fastq_sequences_and_quality_slices_from_expected_fai() {
    unsafe {
        let fai = load_indexed_fastq(
            "htslib/test/faidx/fastqs.fq",
            "htslib/test/faidx/fastqs.fq.expected.fai",
        );

        assert_eq!(faidx_nseq(fai), 105);
        assert_eq!(faidx_iseq(&*fai, 0), Some(b"FAKE0005_1".as_slice()));
        assert_eq!(faidx_iseq(&*fai, 1), Some(b"FAKE0006_1".as_slice()));
        assert_eq!(faidx_iseq(&*fai, 8), Some(b"FSRRS4401BE7HA_1".as_slice()));
        assert_eq!(
            faidx_seq_len(fai, [b"FAKE0005_1".as_slice(), &[0]].concat().as_ptr().cast()),
            63
        );
        assert_eq!(
            faidx_seq_len(fai, [b"FAKE0006_1".as_slice(), &[0]].concat().as_ptr().cast()),
            63
        );
        assert_eq!(
            faidx_seq_len(fai, [b"FSRRS4401BE7HA_1".as_slice(), &[0]].concat().as_ptr().cast()),
            395
        );

        assert_eq!(fetch_text(fai, b"FAKE0005_1:1-12"), "ACGTACGTACGT");
        assert_eq!(fetch_qual_text(fai, b"FAKE0005_1:1-12"), "@ABCDEFGHIJK");
        assert_eq!(fetch_text(fai, b"FAKE0006_1:52-63"), "TGCATGCATGCA");
        assert_eq!(fetch_qual_text(fai, b"FAKE0006_1:52-63"), "KJIHGFEDCBA@");
        assert_eq!(
            fetch_text(fai, b"FSRRS4401BE7HA_1:1-16"),
            "tcagTTAAGATGGGAT"
        );
        assert_eq!(
            fetch_qual_text(fai, b"FSRRS4401BE7HA_1:1-16"),
            "eeeccccccc`UUU^U"
        );

        fai_destroy(fai);
    }
}

#[test]
fn fastq_index_direct_fetch_clips_sequence_and_quality_together() {
    unsafe {
        let fai = load_indexed_fastq(
            "htslib/test/faidx/fastqs.fq",
            "htslib/test/faidx/fastqs.fq.expected.fai",
        );

        assert_eq!(fetch_seq64_text(fai, b"FAKE0005_1", -12, 7), "ACGTACGT");
        assert_eq!(fetch_qual64_text(fai, b"FAKE0005_1", -12, 7), "@ABCDEFG");
        assert_eq!(
            fetch_seq64_text(fai, b"SRR014849.50939_3", 130, 999),
            "TTCGG"
        );
        assert_eq!(
            fetch_qual64_text(fai, b"SRR014849.50939_3", 130, 999),
            "^LY\\S"
        );
        assert_eq!(fetch_seq64_text(fai, b"FAKE0010_2", 63, 99), "");
        assert_eq!(fetch_qual64_text(fai, b"FAKE0010_2", 63, 99), "");

        fai_destroy(fai);
    }
}

#[test]
fn faidx_fasta_queries_render_exact_upstream_expected_output() {
    unsafe {
        let fai = load_indexed_fasta(
            "htslib/test/faidx/faidx.fa",
            "htslib/test/faidx/faidx.fa.expected.fai",
        );
        let actual = render_fasta_queries(
            fai,
            &[
                b"trailingblank2:28-33".as_slice(),
                b"trailingblank3:4-5",
                b"bar:4-5",
            ],
        );
        let expected = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("htslib/test/faidx/faidx.1.expected.fa"),
        )
        .unwrap();
        assert_eq!(actual, expected);
        fai_destroy(fai);
    }
}

#[test]
fn faidx_fasta_real_fixture_line_lengths_and_clipped_direct_fetches() {
    unsafe {
        let fai = load_indexed_fasta(
            "htslib/test/faidx/faidx.fa",
            "htslib/test/faidx/faidx.fa.expected.fai",
        );

        assert_eq!(fai_line_length(fai, [b"".as_slice(), &[0]].concat().as_ptr().cast()), 4);
        assert_eq!(
            fai_line_length(fai, [b"trailingblank2".as_slice(), &[0]].concat().as_ptr().cast()),
            24
        );
        assert_eq!(
            fai_line_length(fai, [b"trailingblank3".as_slice(), &[0]].concat().as_ptr().cast()),
            4
        );
        assert_eq!(
            fai_line_length(fai, [b"foo".as_slice(), &[0]].concat().as_ptr().cast()),
            6
        );
        assert_eq!(
            fai_line_length(fai, [b"bar".as_slice(), &[0]].concat().as_ptr().cast()),
            8
        );
        assert_eq!(
            fai_line_length(fai, [b"missing".as_slice(), &[0]].concat().as_ptr().cast()),
            -1
        );

        assert_eq!(fetch_text(fai, b"bar:1-999"), "TTTTAAAA");
        assert_eq!(fetch_seq64_text(fai, b"bar", -10, 99), "TTTTAAAA");
        assert_eq!(fetch_seq64_text(fai, b"trailingblank3", 3, 99), "TA");
        assert_eq!(fetch_seq64_text(fai, b"trailingblank2", 70, 99), "TT");

        fai_destroy(fai);
    }
}

#[test]
fn faidx_fastq_queries_render_exact_upstream_expected_outputs() {
    unsafe {
        let fai = load_indexed_fastq(
            "htslib/test/faidx/fastqs.fq",
            "htslib/test/faidx/fastqs.fq.expected.fai",
        );
        let queries: [&[u8]; 4] = [
            b"FAKE0006_1:4-12",
            b"FSRRS4401BE7HA_1:81-120",
            b"FAKE0010_2",
            b"SRR014849.50939_3:71-90",
        ];

        let actual_fastq = render_fastq_queries(fai, &queries);
        let expected_fastq = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("htslib/test/faidx/fastqs.1.expected.fq"),
        )
        .unwrap();
        assert_eq!(actual_fastq, expected_fastq);

        let actual_fasta = render_fasta_queries(fai, &queries);
        let expected_fasta = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("htslib/test/faidx/fastqs.2.expected.fa"),
        )
        .unwrap();
        assert_eq!(actual_fasta, expected_fasta);

        fai_destroy(fai);
    }
}

#[test]
fn fasta_index_presence_checks_use_real_fixture_names() {
    unsafe {
        let fai = load_indexed_fasta("htslib/test/auxf.fa", "htslib/test/auxf.fa.fai");

        assert_eq!(faidx_nseq(fai), 1);
        assert_eq!(faidx_iseq(&*fai, 0), Some(b"Sheila".as_slice()));
        assert_eq!(
            faidx_has_seq(fai, [b"Sheila".as_slice(), &[0]].concat().as_ptr().cast()),
            1
        );
        assert_eq!(
            faidx_has_seq(fai, [b"sheila".as_slice(), &[0]].concat().as_ptr().cast()),
            0
        );
        assert_eq!(
            faidx_seq_len(fai, [b"Sheila".as_slice(), &[0]].concat().as_ptr().cast()),
            20
        );
        assert_eq!(fetch_text(fai, b"Sheila:1-20"), "GCTAGCTCAGAAAAAAAAAA");
        assert_eq!(fetch_text(fai, b"Sheila:8-12"), "CAGAA");

        fai_destroy(fai);
    }
}

#[test]
fn bgzf_fixture_reads_payload_lines_and_reports_eof_marker() {
    unsafe {
        let gz = c_fixture("htslib/test/bgziptest.txt.gz");
        assert_eq!(bgzf_is_bgzf(gz.as_ptr().cast()), 1);

        let fp = bgzf_open(gz.as_ptr().cast(), c"r".as_ptr());
        assert!(!fp.is_null());
        assert_eq!(bgzf_compression(fp), hts_sys::htsCompression_bgzf as i32);

        let mut line: kstring_t = kstring_t::default();
        assert_eq!(bgzf_getline(fp, b'\n' as i32, &mut line), 15);
        assert_eq!(line.data.as_slice(), b"122333444455555");

        let mut tail = [0u8; 1];
        assert_eq!(bgzf_read(fp, tail.as_mut_ptr().cast(), tail.len()), 0);
        assert_eq!(bgzf_check_EOF(fp), 1);
        assert_eq!(bgzf_close(fp), 0);
    }
}

#[test]
fn bgzf_fixture_read_matches_uncompressed_text_bytes() {
    unsafe {
        let gz = c_fixture("htslib/test/bgziptest.txt.gz");
        let fp = bgzf_open(gz.as_ptr().cast(), c"r".as_ptr());
        assert!(!fp.is_null());

        let mut buf = [0u8; 32];
        let n = bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len());
        assert_eq!(n, 15);
        assert_eq!(&buf[..n as usize], b"122333444455555");
        assert_eq!(bgzf_check_EOF(fp), 1);

        assert_eq!(bgzf_close(fp), 0);
    }
}
