use flate2::{write::GzEncoder, Compression};
use htslib_rs::{
    hclose, hgets, hopen, hpeek, hseek, htslib_hfile_h_155_htell, htslib_hfile_h_247_hread,
};
use std::io::Write;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_fixture(path: &str) -> Vec<u8> {
    let mut bytes = fixture(path).to_string_lossy().as_bytes().to_vec();
    bytes.push(0);
    bytes
}

fn temp_path(label: &str, extension: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "htslib_rs-hfile-real-{label}-{}.{extension}",
        std::process::id()
    ))
}

#[test]
fn hfile_peek_read_and_seek_match_real_fixture_bytes() {
    unsafe {
        let expected = std::fs::read(fixture("htslib/test/index.vcf")).unwrap();
        assert!(expected.len() > 512);

        let path = c_fixture("htslib/test/index.vcf");
        let fp = hopen(path.as_ptr().cast(), b"r\0".as_ptr().cast());
        assert!(!fp.is_null());

        let mut peeked = [0u8; 17];
        assert_eq!(
            hpeek(fp, peeked.as_mut_ptr().cast(), peeked.len()),
            peeked.len() as isize
        );
        assert_eq!(&peeked, &expected[..peeked.len()]);
        assert_eq!(htslib_hfile_h_155_htell(fp), 0);

        let mut first = [0u8; 31];
        assert_eq!(
            htslib_hfile_h_247_hread(fp, first.as_mut_ptr().cast(), first.len()),
            first.len() as isize
        );
        assert_eq!(&first, &expected[..first.len()]);
        assert_eq!(htslib_hfile_h_155_htell(fp), first.len() as i64);

        assert_eq!(hseek(fp, 123, libc::SEEK_SET), 123);
        let mut middle = [0u8; 64];
        assert_eq!(
            htslib_hfile_h_247_hread(fp, middle.as_mut_ptr().cast(), middle.len()),
            middle.len() as isize
        );
        assert_eq!(&middle, &expected[123..123 + middle.len()]);

        assert_eq!(hseek(fp, -37, libc::SEEK_END), (expected.len() - 37) as i64);
        let mut tail = [0u8; 37];
        assert_eq!(
            htslib_hfile_h_247_hread(fp, tail.as_mut_ptr().cast(), tail.len()),
            tail.len() as isize
        );
        assert_eq!(&tail, &expected[expected.len() - tail.len()..]);

        assert_eq!(hclose(fp), 0);
    }
}

#[test]
fn hfile_hgets_reads_real_vcf_header_lines_like_std_io() {
    unsafe {
        let expected = std::fs::read_to_string(fixture("htslib/test/index.vcf")).unwrap();
        let mut expected_lines = expected.lines();

        let path = c_fixture("htslib/test/index.vcf");
        let fp = hopen(path.as_ptr().cast(), b"r\0".as_ptr().cast());
        assert!(!fp.is_null());

        let mut buf = [0u8; 256];
        for _ in 0..4 {
            assert_eq!(
                hgets(buf.as_mut_ptr().cast(), buf.len() as i32, fp),
                buf.as_mut_ptr().cast()
            );
            let nul = buf.iter().position(|&b| b == 0).unwrap();
            let got = String::from_utf8_lossy(&buf[..nul]).into_owned();
            let expected_line = expected_lines.next().unwrap();
            assert_eq!(got, format!("{expected_line}\n"));
        }

        assert_eq!(hclose(fp), 0);
    }
}

#[test]
fn htsfile_raw_copy_preserves_real_text_fixture_bytes() {
    unsafe {
        let src = fixture("htslib/test/index.vcf");
        let dst = temp_path("copy-index-vcf", "vcf");
        let _ = std::fs::remove_file(&dst);

        let argv = vec![
            b"htsfile".to_vec(),
            b"-C".to_vec(),
            src.to_string_lossy().as_bytes().to_vec(),
            dst.to_string_lossy().as_bytes().to_vec(),
        ];
        assert_eq!(htslib_rs::htsfile::htsfile_c_227_main(argv), 0);

        assert_eq!(std::fs::read(&dst).unwrap(), std::fs::read(&src).unwrap());
        let _ = std::fs::remove_file(dst);
    }
}

#[test]
fn htsfile_raw_copy_preserves_real_compressed_and_index_bytes() {
    unsafe {
        for (label, fixture_path, extension) in [
            ("bgzf", "htslib/test/bgziptest.txt.gz", "gz"),
            ("gzi", "htslib/test/bgziptest.txt.gz.gzi", "gzi"),
            ("tbi", "htslib/test/index.vcf.gz.tbi", "tbi"),
        ] {
            let src = fixture(fixture_path);
            let dst = temp_path(label, extension);
            let _ = std::fs::remove_file(&dst);

            let argv = vec![
                b"htsfile".to_vec(),
                b"-C".to_vec(),
                src.to_string_lossy().as_bytes().to_vec(),
                dst.to_string_lossy().as_bytes().to_vec(),
            ];
            assert_eq!(htslib_rs::htsfile::htsfile_c_227_main(argv), 0);

            assert_eq!(
                std::fs::read(&dst).unwrap(),
                std::fs::read(&src).unwrap(),
                "{fixture_path}"
            );
            let _ = std::fs::remove_file(dst);
        }
    }
}

#[test]
fn htsfile_view_real_gz_fastq_exits_cleanly_after_redirected_stdout() {
    let src = fixture("htslib/test/fastq/single.fq");
    let gz = temp_path("single-fastq", "fq.gz");
    let out = temp_path("single-fastq-view", "sam");
    let _ = std::fs::remove_file(&gz);
    let _ = std::fs::remove_file(&out);

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&std::fs::read(&src).unwrap()).unwrap();
    std::fs::write(&gz, encoder.finish().unwrap()).unwrap();

    let stdout = std::fs::File::create(&out).unwrap();
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_perf_htsfile"))
        .arg("-c")
        .arg(&gz)
        .stdout(stdout)
        .status()
        .unwrap();
    assert!(status.success(), "perf_htsfile exited with {status}");

    let rendered = std::fs::read_to_string(&out).unwrap();
    assert!(rendered.contains("\t4\t*\t0\t0\t*\t*\t0\t0\t"));

    let _ = std::fs::remove_file(gz);
    let _ = std::fs::remove_file(out);
}
