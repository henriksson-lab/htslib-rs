use htslib_mini_rs::{
    bgzf::{
        bgzf_block_write, bgzf_check_EOF, bgzf_close, bgzf_index_destroy, bgzf_index_load,
        bgzf_is_bgzf, bgzf_open, bgzf_raw_read, bgzf_read, bgzf_seek, bgzf_useek, bgzf_utell,
        bgzidx_t,
    },
    BGZF,
};
use std::ffi::CString;

const BGZF_EOF_MARKER: [u8; 28] = [
    31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 27, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_fixture(path: &str) -> CString {
    CString::new(fixture(path).to_string_lossy().as_bytes()).unwrap()
}

unsafe fn open_bgzf_fixture(path: &str) -> *mut BGZF {
    let path = c_fixture(path);
    let fp = bgzf_open(path.as_ptr(), c"r".as_ptr());
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

#[test]
fn bgziptest_rebgzip_reconstructs_exact_original_bgzf_stream() {
    let plain = std::fs::read(fixture("htslib/test/bgziptest.txt")).unwrap();
    let expected = std::fs::read(fixture("htslib/test/bgziptest.txt.gz")).unwrap();
    let out = std::env::temp_dir().join(format!(
        "htslib-mini-rs-rebgzip-{}-{}.gz",
        std::process::id(),
        "exact"
    ));
    let out_c = CString::new(out.to_string_lossy().as_bytes()).unwrap();
    let gzi = c_fixture("htslib/test/bgziptest.txt.gz.gzi");

    unsafe {
        let fp = bgzf_open(out_c.as_ptr(), c"w".as_ptr());
        assert!(!fp.is_null());
        assert_eq!(bgzf_index_load(fp, gzi.as_ptr(), std::ptr::null()), 0);
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

        assert_eq!(bgzf_index_load(fp, gzi.as_ptr(), std::ptr::null()), 0);
        assert!(!(*fp).idx.is_null());

        let idx = (*fp).idx.cast::<bgzidx_t>();
        assert_eq!((*idx).noffs, 6);
        assert_eq!((*idx).moffs, 6);
        assert_eq!((*idx).ublock_addr, 0);

        let expected_pairs = [(0, 0), (29, 1), (59, 3), (90, 6), (122, 10), (153, 15)];
        for (i, &(compressed_offset, uncompressed_offset)) in expected_pairs.iter().enumerate() {
            let off = (*idx).offs.add(i);
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

        assert_eq!(bgzf_index_load(fp, gzi.as_ptr(), std::ptr::null()), 0);
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
            assert_eq!(bgzf_is_bgzf(c_path.as_ptr()), 1, "{path}");

            let fp = bgzf_open(c_path.as_ptr(), c"r".as_ptr());
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
            assert_eq!(bgzf_is_bgzf(c_path.as_ptr()), 0, "{path}");
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
