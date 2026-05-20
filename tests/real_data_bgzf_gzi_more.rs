use htslib_mini_rs::{
    bgzf::{
        bgzf_close, bgzf_index_destroy, bgzf_index_load, bgzf_open, bgzf_raw_read, bgzf_read,
        bgzf_seek, bgzf_utell, bgzidx_t,
    },
    BGZF,
};
use std::ffi::CString;

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn open_bgziptest() -> *mut BGZF {
    let path = c_fixture("htslib/test/bgziptest.txt.gz");
    let fp = bgzf_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null());
    fp
}

#[test]
fn real_bgzf_gzi_loads_exact_block_index_offsets() {
    unsafe {
        let fp = open_bgziptest();
        let gzi = c_fixture("htslib/test/bgziptest.txt.gz.gzi");

        assert_eq!(bgzf_index_load(fp, gzi.as_ptr(), std::ptr::null()), 0);
        assert!(!(*fp).idx.is_null());

        let idx = (*fp).idx.cast::<bgzidx_t>();
        assert_eq!((*idx).noffs, 6);
        assert_eq!((*idx).moffs, 6);

        let expected: &[(u64, u64)] = &[(0, 0), (29, 1), (59, 3), (90, 6), (122, 10), (153, 15)];
        for (i, &(caddr, uaddr)) in expected.iter().enumerate() {
            let off = (*idx).offs.add(i);
            assert_eq!(((*off).caddr, (*off).uaddr), (caddr, uaddr));
        }

        bgzf_index_destroy(fp);
        assert_eq!(bgzf_close(fp), 0);
    }
}

#[test]
fn real_bgzf_reads_advance_through_gzi_block_boundaries() {
    unsafe {
        let fp = open_bgziptest();

        let chunks: &[(usize, &[u8], i64, i64)] = &[
            (1, b"1".as_slice(), 1, 29),
            (2, b"22".as_slice(), 3, 59),
            (3, b"333".as_slice(), 6, 90),
            (4, b"4444".as_slice(), 10, 122),
            (5, b"55555".as_slice(), 15, 153),
        ];
        for &(len, bytes, logical_offset, next_block_address) in chunks {
            let mut buf = [0u8; 5];
            assert_eq!(bgzf_read(fp, buf.as_mut_ptr().cast(), len), len as isize);
            assert_eq!(&buf[..len], bytes);
            assert_eq!(bgzf_utell(fp), logical_offset);
            assert_eq!((*fp).block_address, next_block_address);
            assert_eq!((*fp).block_offset, 0);
            assert_eq!((*fp).block_length, 0);
        }

        let mut eof = [0u8; 1];
        assert_eq!(bgzf_read(fp, eof.as_mut_ptr().cast(), eof.len()), 0);
        assert_eq!(bgzf_utell(fp), 15);

        assert_eq!(bgzf_close(fp), 0);
    }
}

#[test]
fn real_bgzf_virtual_seek_replays_blocks_from_gzi_offsets() {
    unsafe {
        let fp = open_bgziptest();

        let seek_cases: &[(i64, &[u8])] = &[
            (29, b"22".as_slice()),
            (59, b"333".as_slice()),
            (90, b"4444".as_slice()),
            (122, b"55555".as_slice()),
        ];
        for &(compressed_address, expected) in seek_cases {
            let virtual_offset = compressed_address << 16;
            assert_eq!(bgzf_seek(fp, virtual_offset, libc::SEEK_SET), 0);
            assert_eq!((*fp).block_address, compressed_address);
            assert_eq!((*fp).block_offset, 0);
            assert_eq!((*fp).block_length, 0);

            let mut buf = [0u8; 5];
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
fn real_bgzf_raw_read_exposes_compressed_blocks_and_terminal_eof_block() {
    unsafe {
        let fp = open_bgziptest();

        let mut raw = [0u8; 181];
        assert_eq!(
            bgzf_raw_read(fp, raw.as_mut_ptr().cast(), raw.len()),
            raw.len() as isize
        );

        assert_eq!(
            &raw[..18],
            &[31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 28, 0]
        );
        assert_eq!(
            &raw[153..],
            &[
                31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 27, 0, 3, 0, 0, 0, 0, 0, 0,
                0, 0, 0,
            ]
        );

        assert_eq!(bgzf_close(fp), 0);
    }
}
