use flate2::{write::GzEncoder, Compression};
use htslib_rs::{
    bgzf::{
        bgzf_block_write, bgzf_c_1942_lazy_flush, bgzf_check_EOF, bgzf_close, bgzf_compression,
        bgzf_getc, bgzf_getline, bgzf_index_destroy, bgzf_index_load, bgzf_is_bgzf, bgzf_open,
        bgzf_peek, bgzf_raw_read, bgzf_read, bgzf_seek, bgzf_useek, bgzf_utell, bgzf_write,
    },
    bgzip::bgzip_c_217_main,
    kstring_t,
    test::test_bgzf::{
        test_test_bgzf_c_1073_main, test_test_bgzf_c_344_cleanup, test_test_bgzf_c_357_setup,
        test_test_bgzf_c_403_test_read, test_test_bgzf_c_435_test_write_read,
        test_test_bgzf_c_511_test_embed_eof, test_test_bgzf_c_584_test_index_load_dump,
        test_test_bgzf_c_622_test_check_EOF, test_test_bgzf_c_638_test_index_useek_getc,
        test_test_bgzf_c_730_test_tell_seek_getc, test_test_bgzf_c_831_test_tell_read,
        test_test_bgzf_c_881_test_useek_read_small, test_test_bgzf_c_924_test_bgzf_getline,
        test_test_bgzf_c_981_test_bgzf_getline_on_truncated_file, Files, Open_method,
    },
    BGZF,
};
use std::io::Write;
use std::process::Command;

const BGZF_EOF_MARKER: [u8; 28] = [
    31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 27, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_fixture(path: &str) -> Vec<u8> {
    let mut bytes = fixture(path).to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

unsafe fn open_bgzf_fixture(path: &str) -> *mut BGZF {
    let path = c_fixture(path);
    let fp = bgzf_open(path.as_ptr().cast(), c"r".as_ptr().cast());
    assert!(!fp.is_null());
    fp
}

fn little_u64(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().unwrap())
}

fn block_sizes(raw: &[u8]) -> Vec<usize> {
    let mut sizes = Vec::new();
    let mut offset = 0;
    while offset < raw.len() {
        assert!(raw.len() - offset >= 18);
        assert_eq!(&raw[offset..offset + 4], &[31, 139, 8, 4]);
        assert_eq!(&raw[offset + 12..offset + 16], b"BC\x02\0");
        let block_size = u16::from_le_bytes([raw[offset + 16], raw[offset + 17]]) as usize + 1;
        assert!(block_size >= 28);
        assert!(offset + block_size <= raw.len());
        sizes.push(block_size);
        offset += block_size;
    }
    sizes
}

fn gzip_member(data: &[u8]) -> Vec<u8> {
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(data).unwrap();
    gz.finish().unwrap()
}

fn with_original_test_bgzf_setup(
    label: &str,
    run: impl FnOnce(&mut Files) -> Vec<(&'static str, i32)>,
) -> Vec<(&'static str, i32)> {
    let workdir = std::env::temp_dir().join(format!(
        "htslib_rs-original-test-bgzf-{label}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&workdir).unwrap();

    let base = workdir.join("bgziptest.txt");
    std::fs::copy(fixture("htslib/test/bgziptest.txt"), &base).unwrap();
    std::fs::copy(
        fixture("htslib/test/bgziptest.txt.gz"),
        workdir.join("bgziptest.txt.gz"),
    )
    .unwrap();
    std::fs::copy(
        fixture("htslib/test/bgziptest.txt.gz.gzi"),
        workdir.join("bgziptest.txt.gz.gzi"),
    )
    .unwrap();

    let mut base_c = base.to_string_lossy().into_owned().into_bytes();
    base_c.push(0);
    let results = unsafe {
        let mut files = Files {
            src_plain: Vec::new(),
            src_bgzf: Vec::new(),
            src_idx: Vec::new(),
            tmp_bgzf: Vec::new(),
            tmp_idx: Vec::new(),
            f_plain: std::ptr::null_mut(),
            f_bgzf: std::ptr::null_mut(),
            f_idx: std::ptr::null_mut(),
            text: Vec::new(),
            ltext: 0,
        };

        assert_eq!(test_test_bgzf_c_357_setup(base_c.as_ptr().cast(), &mut files), 0);
        let results = run(&mut files);
        let retval = if results.iter().all(|(_, result)| *result == 0) {
            libc::EXIT_SUCCESS
        } else {
            libc::EXIT_FAILURE
        };
        test_test_bgzf_c_344_cleanup(&mut files, retval);
        results
    };

    let _ = std::fs::remove_dir_all(workdir);
    results
}

#[test]
fn lazy_flush_non_mt_writes_current_partial_block_without_closing_stream() {
    let out = std::env::temp_dir().join(format!(
        "htslib_rs-lazy-flush-non-mt-{}-{}.gz",
        std::process::id(),
        "split"
    ));
    let mut out_c = out.to_string_lossy().into_owned().into_bytes();
    out_c.push(0);
    let first = b"first partial block\n";
    let second = b"second partial block\n";

    unsafe {
        let fp = bgzf_open(out_c.as_ptr().cast(), c"w".as_ptr().cast());
        assert!(!fp.is_null());
        assert_eq!(
            bgzf_write(fp, first.as_ptr().cast(), first.len()),
            first.len() as isize
        );
        assert_eq!(bgzf_c_1942_lazy_flush(fp), 0);
        assert_eq!((*fp).block_offset, 0);
        let after_lazy_flush = (*fp).block_address;
        assert!(after_lazy_flush > 0);

        assert_eq!(
            bgzf_write(fp, second.as_ptr().cast(), second.len()),
            second.len() as isize
        );
        assert_eq!((*fp).block_address, after_lazy_flush);
        assert_eq!(bgzf_close(fp), 0);
    }

    let raw = std::fs::read(&out).unwrap();
    let sizes = block_sizes(&raw);
    assert_eq!(sizes.len(), 3);
    assert_eq!(sizes[2], BGZF_EOF_MARKER.len());
    assert_eq!(&raw[raw.len() - BGZF_EOF_MARKER.len()..], BGZF_EOF_MARKER);

    unsafe {
        let fp = bgzf_open(out_c.as_ptr().cast(), c"r".as_ptr().cast());
        assert!(!fp.is_null());
        let mut observed = vec![0_u8; first.len() + second.len()];
        assert_eq!(
            bgzf_read(fp, observed.as_mut_ptr().cast(), observed.len()),
            observed.len() as isize
        );
        assert_eq!(&observed[..first.len()], first);
        assert_eq!(&observed[first.len()..], second);
        assert_eq!(bgzf_close(fp), 0);
    }

    let _ = std::fs::remove_file(out);
}

#[test]
fn bgzip_stdout_compresses_real_sam_to_stdout_not_literal_dash_file() {
    let input = fixture("htslib/test/ce#large_seq.sam");
    let expected_plain = std::fs::read(&input).unwrap();
    let workdir = std::env::temp_dir().join(format!(
        "htslib_rs-bgzip-stdout-{}-{}",
        std::process::id(),
        "real-sam"
    ));
    std::fs::create_dir_all(&workdir).unwrap();

    let mut pipefd = [-1; 2];
    unsafe {
        assert_eq!(libc::pipe(pipefd.as_mut_ptr()), 0);
        let pid = libc::fork();
        assert!(pid >= 0);

        if pid == 0 {
            libc::close(pipefd[0]);
            if libc::dup2(pipefd[1], libc::STDOUT_FILENO) < 0 {
                libc::_exit(120);
            }
            libc::close(pipefd[1]);

            let mut cwd = workdir.to_string_lossy().into_owned().into_bytes();
            cwd.push(0);
            if libc::chdir(cwd.as_ptr().cast()) < 0 {
                libc::_exit(121);
            }

            let mut argv0 = b"bgzip\0".to_vec();
            let mut arg1 = b"-c\0".to_vec();
            let mut arg2 = input.to_string_lossy().into_owned().into_bytes();
            arg2.push(0);
            let mut argv = [
                argv0.as_mut_ptr().cast(),
                arg1.as_mut_ptr().cast(),
                arg2.as_mut_ptr().cast(),
                std::ptr::null_mut(),
            ];
            let ret = bgzip_c_217_main(3, argv.as_mut_ptr());
            libc::_exit(ret);
        }

        libc::close(pipefd[1]);
        let mut compressed = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = libc::read(pipefd[0], buf.as_mut_ptr().cast(), buf.len());
            assert!(n >= 0);
            if n == 0 {
                break;
            }
            compressed.extend_from_slice(&buf[..n as usize]);
        }
        libc::close(pipefd[0]);

        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);

        assert!(!compressed.is_empty());
        assert!(!workdir.join("-").exists());

        let out = workdir.join("stdout.sam.gz");
        std::fs::write(&out, &compressed).unwrap();
        let mut out_c = out.to_string_lossy().into_owned().into_bytes();
        out_c.push(0);
        let fp = bgzf_open(out_c.as_ptr().cast(), c"r".as_ptr().cast());
        assert!(!fp.is_null());
        let mut actual = Vec::new();
        loop {
            let n = bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len());
            assert!(n >= 0);
            if n == 0 {
                break;
            }
            actual.extend_from_slice(&buf[..n as usize]);
        }
        assert_eq!(bgzf_close(fp), 0);
        assert_eq!(actual, expected_plain);
    }

    let _ = std::fs::remove_dir_all(workdir);
}

#[test]
fn bgzip_thread_options_round_trip_with_synchronous_bgzf_backend() {
    let input = fixture("htslib/test/ce#large_seq.sam");
    let expected_input = std::fs::read(&input).unwrap();
    let compressed = fixture("htslib/test/bgziptest.txt.gz");
    let expected_decompressed = std::fs::read(fixture("htslib/test/bgziptest.txt")).unwrap();
    let cases = [
        (
            vec![
                "-@".to_string(),
                "2".to_string(),
                "-c".to_string(),
                input.to_string_lossy().into_owned(),
            ],
            Some(expected_input.as_slice()),
        ),
        (
            vec![
                "--threads".to_string(),
                "2".to_string(),
                "-c".to_string(),
                input.to_string_lossy().into_owned(),
            ],
            Some(expected_input.as_slice()),
        ),
        (
            vec![
                "-d".to_string(),
                "-@".to_string(),
                "2".to_string(),
                "-c".to_string(),
                compressed.to_string_lossy().into_owned(),
            ],
            None,
        ),
    ];

    for (args, expected_compressed_payload) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_perf_bgzip"))
            .args(&args)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "args failed: {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        if let Some(expected) = expected_compressed_payload {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-bgzip-thread-option-{}-{}.gz",
                std::process::id(),
                output.stdout.len()
            ));
            std::fs::write(&path, &output.stdout).unwrap();
            let mut path_c = path.to_string_lossy().into_owned().into_bytes();
            path_c.push(0);
            unsafe {
                let fp = bgzf_open(path_c.as_ptr().cast(), c"r".as_ptr().cast());
                assert!(!fp.is_null());
                let mut actual = vec![0_u8; expected.len()];
                assert_eq!(
                    bgzf_read(fp, actual.as_mut_ptr().cast(), actual.len()),
                    actual.len() as isize
                );
                assert_eq!(bgzf_close(fp), 0);
                assert_eq!(actual, expected);
            }
            let _ = std::fs::remove_file(path);
        } else {
            assert_eq!(output.stdout, expected_decompressed);
        }
    }
}

#[test]
fn bgzf_open_reads_ordinary_gzip_stream_like_upstream() {
    let plain = std::fs::read(fixture("htslib/test/bgziptest.txt")).unwrap();
    let out = std::env::temp_dir().join(format!(
        "htslib_rs-real-gzip-{}-{}.gz",
        std::process::id(),
        "bgzf-api"
    ));
    {
        let file = std::fs::File::create(&out).unwrap();
        let mut gz = GzEncoder::new(file, Compression::default());
        gz.write_all(&plain).unwrap();
        gz.finish().unwrap();
    }
    let mut out_c = out.to_string_lossy().into_owned().into_bytes();
    out_c.push(0);

    unsafe {
        let fp = bgzf_open(out_c.as_ptr().cast(), c"r".as_ptr().cast());
        assert!(!fp.is_null());
        assert_eq!(bgzf_compression(fp), 1);

        let mut actual = vec![0u8; plain.len()];
        assert_eq!(
            bgzf_read(fp, actual.as_mut_ptr().cast(), actual.len()),
            plain.len() as isize
        );
        assert_eq!(actual, plain);
        assert_eq!(bgzf_read(fp, actual.as_mut_ptr().cast(), actual.len()), 0);
        assert_eq!(bgzf_close(fp), 0);
    }

    let _ = std::fs::remove_file(out);
}

#[test]
fn bgzf_open_reads_concatenated_ordinary_gzip_members_like_upstream() {
    let first = std::fs::read(fixture("htslib/test/bgziptest.txt")).unwrap();
    let second = b"second member after an empty gzip member\n".to_vec();
    let mut plain = Vec::new();
    plain.extend_from_slice(&first);
    plain.extend_from_slice(&second);

    let out = std::env::temp_dir().join(format!(
        "htslib_rs-concat-real-gzip-{}-{}.gz",
        std::process::id(),
        "bgzf-api"
    ));
    let mut concatenated = gzip_member(&[]);
    concatenated.extend_from_slice(&gzip_member(&first));
    concatenated.extend_from_slice(&gzip_member(&second));
    std::fs::write(&out, concatenated).unwrap();
    let mut out_c = out.to_string_lossy().into_owned().into_bytes();
    out_c.push(0);

    unsafe {
        let fp = bgzf_open(out_c.as_ptr().cast(), c"r".as_ptr().cast());
        assert!(!fp.is_null());
        assert_eq!(bgzf_compression(fp), 1);

        let mut actual = vec![0u8; plain.len()];
        assert_eq!(
            bgzf_read(fp, actual.as_mut_ptr().cast(), actual.len()),
            plain.len() as isize
        );
        assert_eq!(actual, plain);
        assert_eq!(bgzf_read(fp, actual.as_mut_ptr().cast(), actual.len()), 0);
        assert_eq!(bgzf_close(fp), 0);
    }

    let _ = std::fs::remove_file(out);
}

#[test]
fn bgzf_open_reads_large_ordinary_gzip_stream_across_internal_blocks() {
    let motif = std::fs::read(fixture("htslib/test/bgziptest.txt")).unwrap();
    let mut plain = Vec::new();
    while plain.len() < 150_000 {
        plain.extend_from_slice(&motif);
    }

    let out = std::env::temp_dir().join(format!(
        "htslib_rs-large-real-gzip-{}-{}.gz",
        std::process::id(),
        "bgzf-api"
    ));
    {
        let file = std::fs::File::create(&out).unwrap();
        let mut gz = GzEncoder::new(file, Compression::default());
        gz.write_all(&plain).unwrap();
        gz.finish().unwrap();
    }
    let mut out_c = out.to_string_lossy().into_owned().into_bytes();
    out_c.push(0);

    unsafe {
        let fp = bgzf_open(out_c.as_ptr().cast(), c"r".as_ptr().cast());
        assert!(!fp.is_null());
        assert_eq!(bgzf_compression(fp), 1);

        let mut actual = Vec::new();
        let mut buf = vec![0u8; 4096];
        loop {
            let n = bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len());
            assert!(n >= 0, "bgzf_read returned {n}");
            if n == 0 {
                break;
            }
            actual.extend_from_slice(&buf[..n as usize]);
        }
        assert_eq!(actual, plain);
        assert_eq!(bgzf_close(fp), 0);
    }

    let _ = std::fs::remove_file(out);
}

#[test]
fn bgziptest_rebgzip_reconstructs_exact_original_bgzf_stream() {
    let plain = std::fs::read(fixture("htslib/test/bgziptest.txt")).unwrap();
    let expected = std::fs::read(fixture("htslib/test/bgziptest.txt.gz")).unwrap();
    let out = std::env::temp_dir().join(format!(
        "htslib_rs-rebgzip-{}-{}.gz",
        std::process::id(),
        "exact"
    ));
    let mut out_c = out.to_string_lossy().into_owned().into_bytes();
    out_c.push(0);
    let gzi = c_fixture("htslib/test/bgziptest.txt.gz.gzi");

    unsafe {
        let fp = bgzf_open(out_c.as_ptr().cast(), c"w".as_ptr().cast());
        assert!(!fp.is_null());
        assert_eq!(bgzf_index_load(fp, gzi.as_ptr().cast(), std::ptr::null()), 0);
        assert_eq!(
            bgzf_block_write(fp, plain.as_ptr().cast(), plain.len()),
            plain.len() as isize
        );
        assert_eq!(bgzf_close(fp), 0);
    }

    let actual = std::fs::read(&out).unwrap();
    assert_eq!(actual, expected);

    let _ = std::fs::remove_file(out);
}

#[test]
fn bgziptest_gzi_sidecar_has_exact_original_little_endian_layout() {
    let gzi = std::fs::read(fixture("htslib/test/bgziptest.txt.gz.gzi")).unwrap();
    assert_eq!(gzi.len(), 88);
    assert_eq!(little_u64(&gzi[0..8]), 5);

    let expected_pairs = [(29, 1), (59, 3), (90, 6), (122, 10), (153, 15)];
    for (i, &(compressed_offset, uncompressed_offset)) in expected_pairs.iter().enumerate() {
        let base = 8 + i * 16;
        assert_eq!(little_u64(&gzi[base..base + 8]), compressed_offset);
        assert_eq!(little_u64(&gzi[base + 8..base + 16]), uncompressed_offset);
    }
}

#[test]
fn bgziptest_gzi_load_adds_original_implicit_zero_offset() {
    unsafe {
        let fp = open_bgzf_fixture("htslib/test/bgziptest.txt.gz");
        let gzi = c_fixture("htslib/test/bgziptest.txt.gz.gzi");

        assert_eq!(bgzf_index_load(fp, gzi.as_ptr().cast(), std::ptr::null()), 0);
        assert!((*fp).idx.is_some());

        let idx = (*fp).idx.as_ref().unwrap();
        assert_eq!(idx.noffs, 6);
        assert_eq!(idx.moffs, 6);
        assert_eq!(idx.ublock_addr, 0);

        let expected_pairs = [(0, 0), (29, 1), (59, 3), (90, 6), (122, 10), (153, 15)];
        for (i, &(compressed_offset, uncompressed_offset)) in expected_pairs.iter().enumerate() {
            let off = idx.offs.add(i);
            assert_eq!((*off).caddr, compressed_offset);
            assert_eq!((*off).uaddr, uncompressed_offset);
        }

        bgzf_index_destroy(fp);
        assert_eq!(bgzf_close(fp), 0);
    }
}

#[test]
fn bgziptest_virtual_seek_inside_blocks_reads_exact_suffix_bytes() {
    unsafe {
        let fp = open_bgzf_fixture("htslib/test/bgziptest.txt.gz");

        let cases: &[(i64, &[u8])] = &[
            ((29 << 16) | 1, b"2"),
            ((59 << 16) | 1, b"33"),
            ((90 << 16) | 2, b"44"),
            ((122 << 16) | 3, b"55"),
        ];
        for &(virtual_offset, expected) in cases {
            assert_eq!(bgzf_seek(fp, virtual_offset, libc::SEEK_SET), 0);
            assert_eq!((*fp).block_address, virtual_offset >> 16);
            assert_eq!((*fp).block_offset, (virtual_offset & 0xffff) as i32);

            let mut buf = [0u8; 3];
            assert_eq!(
                bgzf_read(fp, buf.as_mut_ptr().cast(), expected.len()),
                expected.len() as isize
            );
            assert_eq!(&buf[..expected.len()], expected);
        }

        assert_eq!(bgzf_close(fp), 0);
    }
}

#[test]
fn bgziptest_useek_rejects_offset_beyond_indexed_terminal_block() {
    unsafe {
        let fp = open_bgzf_fixture("htslib/test/bgziptest.txt.gz");
        let gzi = c_fixture("htslib/test/bgziptest.txt.gz.gzi");

        assert_eq!(bgzf_index_load(fp, gzi.as_ptr().cast(), std::ptr::null()), 0);
        assert_eq!(bgzf_useek(fp, 16, libc::SEEK_SET), -1);
        assert_eq!(bgzf_utell(fp), 0);

        bgzf_index_destroy(fp);
        assert_eq!(bgzf_close(fp), -1);
    }
}

#[test]
fn real_bgzf_probes_and_eof_checks_match_original_fixture_markers() {
    unsafe {
        for path in [
            "htslib/test/bgziptest.txt.gz",
            "htslib/test/bgzf_boundaries/bgzf_boundaries1.bam",
            "htslib/test/bgzf_boundaries/bgzf_boundaries2.bam",
            "htslib/test/bgzf_boundaries/bgzf_boundaries3.bam",
            "htslib/test/mpileup/small.bam",
        ] {
            let c_path = c_fixture(path);
            assert_eq!(bgzf_is_bgzf(c_path.as_ptr().cast()), 1, "{path}");

            let fp = bgzf_open(c_path.as_ptr().cast(), c"r".as_ptr().cast());
            assert!(!fp.is_null(), "{path}");
            assert_eq!(bgzf_check_EOF(fp), 1, "{path}");
            assert_eq!(bgzf_utell(fp), 0, "{path}");
            assert_eq!(bgzf_close(fp), 0, "{path}");

            let raw = std::fs::read(fixture(path)).unwrap();
            assert_eq!(
                &raw[raw.len() - BGZF_EOF_MARKER.len()..],
                BGZF_EOF_MARKER,
                "{path}"
            );
        }

        for path in [
            "htslib/test/bgziptest.txt",
            "htslib/test/bgziptest.txt.gz.gzi",
            "htslib/test/emptyfile",
        ] {
            let c_path = c_fixture(path);
            assert_eq!(bgzf_is_bgzf(c_path.as_ptr().cast()), 0, "{path}");
        }
    }
}

#[test]
fn bgzf_boundary_bam_fixtures_keep_exact_original_block_layouts() {
    let cases: &[(&str, &[usize])] = &[
        (
            "htslib/test/bgzf_boundaries/bgzf_boundaries1.bam",
            &[95, 33, 276, 28],
        ),
        (
            "htslib/test/bgzf_boundaries/bgzf_boundaries2.bam",
            &[95, 46, 263, 28],
        ),
        (
            "htslib/test/bgzf_boundaries/bgzf_boundaries3.bam",
            &[
                95, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 38, 28,
            ],
        ),
        ("htslib/test/mpileup/small.bam", &[115, 1151, 28]),
    ];

    for &(path, expected_sizes) in cases {
        let raw = std::fs::read(fixture(path)).unwrap();
        assert_eq!(block_sizes(&raw), expected_sizes, "{path}");
        assert_eq!(
            &raw[raw.len() - BGZF_EOF_MARKER.len()..],
            BGZF_EOF_MARKER,
            "{path}"
        );
    }
}

#[test]
fn raw_read_returns_exact_compressed_bytes_for_real_bgzf_fixtures() {
    unsafe {
        let cases = [
            ("htslib/test/bgziptest.txt.gz", 181usize),
            ("htslib/test/bgzf_boundaries/bgzf_boundaries3.bam", 897usize),
        ];

        for (path, len) in cases {
            let fp = open_bgzf_fixture(path);
            let expected = std::fs::read(fixture(path)).unwrap();
            assert_eq!(expected.len(), len);

            let mut raw = vec![0u8; len];
            assert_eq!(
                bgzf_raw_read(fp, raw.as_mut_ptr().cast(), raw.len()),
                len as isize
            );
            assert_eq!(raw, expected, "{path}");

            let mut eof = [0u8; 1];
            assert_eq!(bgzf_raw_read(fp, eof.as_mut_ptr().cast(), eof.len()), 0);
            assert_eq!(bgzf_close(fp), 0);
        }
    }
}

#[test]
fn bgziptest_getc_peek_and_getline_cross_original_block_boundaries() {
    unsafe {
        let fp = open_bgzf_fixture("htslib/test/bgziptest.txt.gz");

        for expected in b"122" {
            assert_eq!(bgzf_peek(fp), *expected as i32);
            assert_eq!(bgzf_peek(fp), *expected as i32);
            assert_eq!(bgzf_getc(fp), *expected as i32);
        }
        assert_eq!(bgzf_utell(fp), 3);
        assert_eq!((*fp).block_address, 59);
        assert_eq!((*fp).block_offset, 0);
        assert_eq!((*fp).block_length, 0);

        let mut line = kstring_t::default();
        assert_eq!(bgzf_getline(fp, b'4' as i32, &mut line), 3);
        assert_eq!(line.data.as_slice(), b"333");
        assert_eq!(bgzf_utell(fp), 7);
        assert_eq!(bgzf_peek(fp), b'4' as i32);
        assert_eq!(bgzf_getc(fp), b'4' as i32);
        assert_eq!(bgzf_utell(fp), 8);

        assert_eq!(bgzf_close(fp), 0);
    }
}

#[test]
fn original_test_bgzf_read_write_embed_eof_and_index_cases_run_on_fixtures() {
    let results = with_original_test_bgzf_setup("core-original-cases", |files| unsafe {
        vec![
            ("read-existing-bgzf", test_test_bgzf_c_403_test_read(files)),
            (
                "write-read-bgzf-open-compressed",
                test_test_bgzf_c_435_test_write_read(
                    files,
                    c"w".as_ptr().cast(),
                    Open_method::USE_BGZF_OPEN,
                    0,
                    2,
                ),
            ),
            (
                "write-read-bgzf-open-uncompressed",
                test_test_bgzf_c_435_test_write_read(
                    files,
                    c"wu".as_ptr().cast(),
                    Open_method::USE_BGZF_OPEN,
                    0,
                    0,
                ),
            ),
            (
                "embedded-eof-single-thread",
                test_test_bgzf_c_511_test_embed_eof(files, c"w".as_ptr().cast(), 0),
            ),
            (
                "index-load-dump",
                test_test_bgzf_c_584_test_index_load_dump(files),
            ),
        ]
    });

    for (case, result) in results {
        assert_eq!(result, 0, "{case}");
    }
}

#[test]
fn original_test_bgzf_eof_index_useek_and_tell_cases_run_on_fixtures() {
    let results = with_original_test_bgzf_setup("seek-original-cases", |files| unsafe {
        vec![
            (
                "check-eof-source",
                test_test_bgzf_c_622_test_check_EOF(files.src_bgzf.as_mut_ptr(), 1),
            ),
            (
                "index-useek-getc-compressed",
                test_test_bgzf_c_638_test_index_useek_getc(files, c"w".as_ptr().cast(), 1000000, 0),
            ),
            (
                "index-useek-getc-uncompressed",
                test_test_bgzf_c_638_test_index_useek_getc(files, c"wu".as_ptr().cast(), 0, 0),
            ),
            (
                "tell-seek-getc-compressed",
                test_test_bgzf_c_730_test_tell_seek_getc(files, c"w".as_ptr().cast(), 0, 0),
            ),
            (
                "tell-seek-getc-uncompressed",
                test_test_bgzf_c_730_test_tell_seek_getc(files, c"wu".as_ptr().cast(), 0, 0),
            ),
            (
                "tell-read-compressed",
                test_test_bgzf_c_831_test_tell_read(files, c"w".as_ptr().cast()),
            ),
            (
                "tell-read-uncompressed",
                test_test_bgzf_c_831_test_tell_read(files, c"wu".as_ptr().cast()),
            ),
        ]
    });

    for (case, result) in results {
        assert_eq!(result, 0, "{case}");
    }
}

#[test]
fn original_test_bgzf_getline_reads_generated_real_fixture_text() {
    let results = with_original_test_bgzf_setup("getline", |files| unsafe {
        vec![
            (
                "getline-single-thread",
                test_test_bgzf_c_924_test_bgzf_getline(files, c"w".as_ptr().cast(), 0),
            ),
            (
                "getline-thread-option-1",
                test_test_bgzf_c_924_test_bgzf_getline(files, c"w".as_ptr().cast(), 1),
            ),
        ]
    });

    for (case, result) in results {
        assert_eq!(result, 0, "{case}");
    }
}

#[test]
fn original_test_bgzf_getline_reports_truncated_block_errors() {
    let results = with_original_test_bgzf_setup("truncated-getline", |files| unsafe {
        vec![(
            "truncated-getline-single-thread",
            test_test_bgzf_c_981_test_bgzf_getline_on_truncated_file(files, c"w".as_ptr().cast(), 0),
        )]
    });

    for (case, result) in results {
        assert_eq!(result, 0, "{case}");
    }
}

#[test]
fn original_test_bgzf_small_useek_reads_cover_compressed_and_uncompressed_modes() {
    let workdir = std::env::temp_dir().join(format!(
        "htslib_rs-original-test-bgzf-useek-small-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&workdir).unwrap();

    let base = workdir.join("bgziptest.txt");
    std::fs::copy(fixture("htslib/test/bgziptest.txt"), &base).unwrap();
    std::fs::copy(
        fixture("htslib/test/bgziptest.txt.gz"),
        workdir.join("bgziptest.txt.gz"),
    )
    .unwrap();
    std::fs::copy(
        fixture("htslib/test/bgziptest.txt.gz.gzi"),
        workdir.join("bgziptest.txt.gz.gzi"),
    )
    .unwrap();

    let mut base_c = base.to_string_lossy().into_owned().into_bytes();
    base_c.push(0);
    unsafe {
        let mut files = Files {
            src_plain: Vec::new(),
            src_bgzf: Vec::new(),
            src_idx: Vec::new(),
            tmp_bgzf: Vec::new(),
            tmp_idx: Vec::new(),
            f_plain: std::ptr::null_mut(),
            f_bgzf: std::ptr::null_mut(),
            f_idx: std::ptr::null_mut(),
            text: Vec::new(),
            ltext: 0,
        };

        assert_eq!(test_test_bgzf_c_357_setup(base_c.as_ptr().cast(), &mut files), 0);
        let compressed = test_test_bgzf_c_881_test_useek_read_small(&mut files, c"w".as_ptr().cast());
        let uncompressed = test_test_bgzf_c_881_test_useek_read_small(&mut files, c"wu".as_ptr().cast());
        let retval = if compressed == 0 && uncompressed == 0 {
            libc::EXIT_SUCCESS
        } else {
            libc::EXIT_FAILURE
        };
        test_test_bgzf_c_344_cleanup(&mut files, retval);

        assert_eq!(compressed, 0);
        assert_eq!(uncompressed, 0);
    }

    let _ = std::fs::remove_dir_all(workdir);
}

#[test]
fn original_test_bgzf_main_runs_against_original_fixture() {
    unsafe {
        let workdir = std::env::temp_dir().join(format!(
            "htslib_rs-original-test-bgzf-main-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&workdir).unwrap();

        let base = workdir.join("bgziptest.txt");
        std::fs::copy(fixture("htslib/test/bgziptest.txt"), &base).unwrap();
        std::fs::copy(
            fixture("htslib/test/bgziptest.txt.gz"),
            workdir.join("bgziptest.txt.gz"),
        )
        .unwrap();
        std::fs::copy(
            fixture("htslib/test/bgziptest.txt.gz.gzi"),
            workdir.join("bgziptest.txt.gz.gzi"),
        )
        .unwrap();

        let mut arg_bytes = base.to_string_lossy().into_owned().into_bytes();
        arg_bytes.push(0);
        let mut args = [b"test_bgzf\0".to_vec(), arg_bytes];
        let mut argv = args
            .iter_mut()
            .map(|arg| arg.as_mut_ptr())
            .collect::<Vec<_>>();
        assert_eq!(
            test_test_bgzf_c_1073_main(argv.len() as i32, argv.as_mut_ptr()),
            libc::EXIT_SUCCESS
        );

        let _ = std::fs::remove_dir_all(workdir);
    }
}
