use crate::htslib_rs::{
    bgzf::{
        bgzf_check_EOF, bgzf_close, bgzf_compression, bgzf_dopen, bgzf_flush, bgzf_getc,
        bgzf_getline, bgzf_hopen, bgzf_index_build_init, bgzf_index_dump, bgzf_index_load, bgzf_mt,
        bgzf_open, bgzf_read_small, bgzf_seek, bgzf_set_cache_size, bgzf_useek, bgzf_write_small,
    },
    hfile::{hclose_abruptly, hfile_oflags, hopen},
    hts::{hts_get_log_level, hts_set_log_level, ks_release, kstring_t, BGZF, HTS_LOG_OFF},
};
use std::ptr;

pub const BGZF_SUFFIX: &[u8] = b".gz\0";
pub const IDX_SUFFIX: &[u8] = b".gzi\0";
pub const TMP_SUFFIX: &[u8] = b".tmp\0";

const BUFSZ: usize = 32768;

pub struct Files {
    pub src_plain: Vec<u8>,
    pub src_bgzf: Vec<u8>,
    pub src_idx: Vec<u8>,
    pub tmp_bgzf: Vec<u8>,
    pub tmp_idx: Vec<u8>,
    pub f_plain: *mut libc::FILE,
    pub f_bgzf: *mut libc::FILE,
    pub f_idx: *mut libc::FILE,
    pub text: Vec<u8>,
    pub ltext: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum Open_method {
    USE_BGZF_OPEN,
    USE_BGZF_DOPEN,
    USE_BGZF_HOPEN,
}

// original: try_fopen (htslib/test/test_bgzf.c:68)
pub unsafe fn test_test_bgzf_c_68_try_fopen(
    name: *const u8,
    mode: *const u8,
) -> *mut libc::FILE {
    let f = libc::fopen(name.cast(), mode.cast());
    if f.is_null() {
        eprintln!(
            "Couldn't open {:?} : errno {}",
            name,
            *libc::__errno_location(),
        );
        return ptr::null_mut();
    }
    f
}

// original: try_fclose (htslib/test/test_bgzf.c:77)
pub unsafe fn test_test_bgzf_c_77_try_fclose(
    file: *mut *mut libc::FILE,
    name: *const u8,
    func: &[u8],
) -> i32 {
    let to_close = *file;
    *file = ptr::null_mut();
    if libc::fclose(to_close) != 0 {
        eprintln!(
            "{:?} : Error on closing {:?} : errno {}",
            func,
            name,
            *libc::__errno_location(),
        );
        return -1;
    }

    0
}

// original: try_fread (htslib/test/test_bgzf.c:89)
pub unsafe fn test_test_bgzf_c_89_try_fread(
    in_: *mut libc::FILE,
    buf: *mut (),
    len: usize,
    func: &[u8],
    fname: *const u8,
) -> isize {
    let got = libc::fread(buf.cast(), 1, len, in_);
    if got == 0 && libc::ferror(in_) != 0 {
        eprintln!(
            "{:?} : Error reading from {:?} : errno {}",
            func,
            fname,
            *libc::__errno_location(),
        );
        return -1;
    }
    got as isize
}

// original: try_fseek_start (htslib/test/test_bgzf.c:100)
pub unsafe fn test_test_bgzf_c_100_try_fseek_start(
    f: *mut libc::FILE,
    name: *const u8,
    func: &[u8],
) -> i32 {
    if libc::fseek(f, 0, libc::SEEK_SET) != 0 {
        eprintln!(
            "{:?} : Couldn't seek on {:?} : errno {}",
            func,
            name,
            *libc::__errno_location(),
        );
        return -1;
    }
    0
}

// original: try_bgzf_open (htslib/test/test_bgzf.c:109)
pub unsafe fn test_test_bgzf_c_109_try_bgzf_open(
    name: *const u8,
    mode: *const u8,
    func: &[u8],
) -> *mut BGZF {
    let bgz = bgzf_open(name.cast(), mode.cast());
    if bgz.is_null() {
        eprintln!(
            "{:?} : Couldn't bgzf_open {:?} with mode {:?} : errno {}",
            func,
            name,
            mode,
            *libc::__errno_location(),
        );
        return ptr::null_mut();
    }
    bgz
}

// original: try_bgzf_dopen (htslib/test/test_bgzf.c:120)
pub unsafe fn test_test_bgzf_c_120_try_bgzf_dopen(
    name: *const u8,
    mode: *const u8,
    func: &[u8],
) -> *mut BGZF {
    let fd = libc::open(name.cast(), hfile_oflags(mode.cast()), 0o666);
    if fd < 0 {
        eprintln!(
            "{:?} : Failed to open {:?} with mode {:?} : errno {}",
            func,
            name,
            mode,
            *libc::__errno_location(),
        );
        return ptr::null_mut();
    }

    let bgz = bgzf_dopen(fd, mode.cast());
    if bgz.is_null() {
        eprintln!(
            "{:?} : bgzf_dopen failed on {:?} mode {:?} : errno {}",
            func,
            name,
            mode,
            *libc::__errno_location(),
        );
        libc::close(fd);
        return ptr::null_mut();
    }

    bgz
}

// original: try_bgzf_hopen (htslib/test/test_bgzf.c:141)
pub unsafe fn test_test_bgzf_c_141_try_bgzf_hopen(
    name: *const u8,
    mode: *const u8,
    func: &[u8],
) -> *mut BGZF {
    let hfp = hopen(name.cast(), mode.cast());
    if hfp.is_null() {
        eprintln!(
            "{:?} : hopen failed on {:?} mode {:?} : errno {}",
            func,
            name,
            mode,
            *libc::__errno_location(),
        );
        return ptr::null_mut();
    }

    let bgz = bgzf_hopen(hfp, mode.cast());
    if bgz.is_null() {
        eprintln!(
            "{:?} : bgzf_hopen failed on {:?} mode {:?} : errno {}",
            func,
            name,
            mode,
            *libc::__errno_location(),
        );
        hclose_abruptly(hfp);
        return ptr::null_mut();
    }

    bgz
}

// original: try_bgzf_close (htslib/test/test_bgzf.c:163)
pub unsafe fn test_test_bgzf_c_163_try_bgzf_close(
    bgz: *mut *mut BGZF,
    name: *const u8,
    func: &[u8],
    expected_fail: i32,
) -> i32 {
    let to_close = *bgz;
    *bgz = ptr::null_mut();
    if bgzf_close(to_close) != 0 {
        if expected_fail == 0 {
            let errno = *libc::__errno_location();
            eprintln!(
                "{:?} : bgzf_close failed on {:?} : errno {}",
                func,
                name,
                errno,
            );
        }
        return -1;
    } else if expected_fail != 0 {
        eprintln!(
            "{:?} : bgzf_close worked on {:?}, but expected failure",
            func,
            name,
        );
    }
    0
}

// original: try_bgzf_read (htslib/test/test_bgzf.c:180)
pub unsafe fn test_test_bgzf_c_180_try_bgzf_read(
    fp: *mut BGZF,
    data: *mut (),
    length: usize,
    name: *const u8,
    func: &[u8],
) -> isize {
    let got = bgzf_read_small(fp, data.cast(), length);
    if got < 0 {
        eprintln!(
            "{:?} : Error from bgzf_read {:?} : errno {}",
            func,
            name,
            *libc::__errno_location(),
        );
    }
    got as isize
}

// original: try_bgzf_write (htslib/test/test_bgzf.c:190)
pub unsafe fn test_test_bgzf_c_190_try_bgzf_write(
    fp: *mut BGZF,
    data: *const (),
    length: usize,
    name: *const u8,
    func: &[u8],
) -> isize {
    let put = bgzf_write_small(fp, data.cast(), length);
    if put < length as isize {
        eprintln!(
            "{:?} : {} {:?} : errno {}",
            func,
            if put < 0 {
                "Error writing to"
            } else {
                "Short write on"
            },
            name,
            *libc::__errno_location(),
        );
        return -1;
    }

    put as isize
}

// original: try_bgzf_compression (htslib/test/test_bgzf.c:203)
pub unsafe fn test_test_bgzf_c_203_try_bgzf_compression(
    fp: *mut BGZF,
    expect: i32,
    name: *const u8,
    func: &[u8],
) -> i32 {
    let res = bgzf_compression(fp);
    if res != expect {
        eprintln!(
            "{:?} : Unexpected result {} from bgzf_compression on {:?}; expected {}",
            func,
            res,
            name,
            expect,
        );
        return -1;
    }
    0
}

// original: try_bgzf_mt (htslib/test/test_bgzf.c:216)
pub unsafe fn test_test_bgzf_c_216_try_bgzf_mt(
    bgz: *mut BGZF,
    nthreads: i32,
    func: &[u8],
) -> i32 {
    if bgzf_mt(bgz, nthreads, 64) != 0 {
        eprintln!(
            "{:?} : Error from bgzf_mt : errno {}",
            func,
            *libc::__errno_location(),
        );
        return -1;
    }
    0
}

// original: try_bgzf_index_build_init (htslib/test/test_bgzf.c:225)
pub unsafe fn test_test_bgzf_c_225_try_bgzf_index_build_init(
    bgz: *mut BGZF,
    name: *const u8,
    func: &[u8],
) -> i32 {
    if bgzf_index_build_init(bgz) != 0 {
        eprintln!(
            "{:?} : Error from bgzf_index_build_init on {:?} : errno {}",
            func,
            name,
            *libc::__errno_location(),
        );
        return -1;
    }
    0
}

// original: try_bgzf_index_load (htslib/test/test_bgzf.c:235)
pub unsafe fn test_test_bgzf_c_235_try_bgzf_index_load(
    fp: *mut BGZF,
    bname: *const u8,
    suffix: *const u8,
    func: &[u8],
) -> i32 {
    if bgzf_index_load(fp, bname.cast(), suffix.cast()) != 0 {
        eprintln!(
            "{:?} : Couldn't bgzf_index_load {:?}{:?} : errno {}",
            func,
            bname,
            suffix,
            *libc::__errno_location(),
        );
        return -1;
    }
    0
}

// original: try_bgzf_index_dump (htslib/test/test_bgzf.c:245)
pub unsafe fn test_test_bgzf_c_245_try_bgzf_index_dump(
    fp: *mut BGZF,
    bname: *const u8,
    suffix: *const u8,
    func: &[u8],
) -> i32 {
    if bgzf_index_dump(fp, bname.cast(), suffix.cast()) != 0 {
        eprintln!(
            "{:?} : Couldn't bgzf_index_dump {:?}{:?} : errno {}",
            func,
            bname,
            suffix,
            *libc::__errno_location(),
        );
        return -1;
    }
    0
}

// original: try_bgzf_tell (htslib/test/test_bgzf.c:255)
pub unsafe fn test_test_bgzf_c_255_try_bgzf_tell(
    fp: *mut BGZF,
    name: *const u8,
    func: &[u8],
) -> i64 {
    let told = (((*fp).block_address as u64) << 16 | ((*fp).block_offset as u64 & 0xffff)) as i64;
    if told < 0 {
        eprintln!(
            "{:?} : Error telling in {:?} : errno {}",
            func,
            name,
            *libc::__errno_location(),
        );
        return -1;
    }
    told
}

// original: try_bgzf_tell_expect (htslib/test/test_bgzf.c:267)
pub unsafe fn test_test_bgzf_c_267_try_bgzf_tell_expect(
    fp: *mut BGZF,
    expected: i64,
    name: *const u8,
    func: &[u8],
) -> i64 {
    let told = test_test_bgzf_c_255_try_bgzf_tell(fp, name, func);
    if told != expected {
        eprintln!(
            "{:?} : Unexpected value ({}) from bgzf_tell on {:?}; expected {}",
            func,
            told,
            name,
            expected,
        );
        return -1;
    }
    told
}

// original: try_bgzf_seek (htslib/test/test_bgzf.c:278)
pub unsafe fn test_test_bgzf_c_278_try_bgzf_seek(
    fp: *mut BGZF,
    pos: i64,
    whence: i32,
    name: *const u8,
    func: &[u8],
) -> i32 {
    if bgzf_seek(fp, pos, whence) < 0 {
        eprintln!(
            "{:?} : Error from bgzf_seek({:?}, {}, {}) : errno {}",
            func,
            name,
            pos,
            whence,
            *libc::__errno_location(),
        );
        return -1;
    }
    0
}

// original: try_bgzf_useek (htslib/test/test_bgzf.c:288)
pub unsafe fn test_test_bgzf_c_288_try_bgzf_useek(
    fp: *mut BGZF,
    uoffset: i64,
    where_: i32,
    name: *const u8,
    func: &[u8],
) -> i32 {
    if bgzf_useek(fp, uoffset, where_) < 0 {
        eprintln!(
            "{:?} : Error from bgzf_useek({:?}, {}, {}) : errno {}",
            func,
            name,
            uoffset,
            where_,
            *libc::__errno_location(),
        );
        return -1;
    }
    0
}

// original: try_bgzf_getc (htslib/test/test_bgzf.c:298)
pub unsafe fn test_test_bgzf_c_298_try_bgzf_getc(
    fp: *mut BGZF,
    pos: usize,
    expected: i32,
    name: *const u8,
    func: &[u8],
) -> i32 {
    let c = bgzf_getc(fp);
    if c != expected {
        eprintln!(
            "{:?} : Unexpected value ({}) from bgzf_getc on {:?} pos {}; expected {}",
            func,
            c,
            name,
            pos,
            expected,
        );
        return -1;
    }
    c
}

// original: try_skip (htslib/test/test_bgzf.c:311)
pub unsafe fn test_test_bgzf_c_311_try_skip(
    fp: *mut BGZF,
    count: usize,
    name: *const u8,
    func: &[u8],
) -> i32 {
    for _ in 0..count {
        let c = bgzf_getc(fp);
        if c < 0 {
            eprintln!(
                "{:?} : Error from bgzf_getc on {:?}",
                func,
                name,
            );
            return -1;
        }
    }
    0
}

// original: compare_buffers (htslib/test/test_bgzf.c:327)
pub unsafe fn test_test_bgzf_c_327_compare_buffers(
    b1: *const u8,
    b2: *const u8,
    l1: usize,
    l2: usize,
    name1: *const u8,
    name2: *const u8,
    func: &[u8],
) -> i32 {
    if l1 != l2 {
        eprintln!(
            "{:?} : EOF on {:?}",
            func,
            if l1 < l2 { name1 } else { name2 },
        );
        return -1;
    }
    if std::slice::from_raw_parts(b1, l1) != std::slice::from_raw_parts(b2, l2) {
        eprintln!(
            "{:?} : difference between {:?} and {:?}",
            func,
            name1,
            name2,
        );
        return -1;
    }

    0
}

// original: cleanup (htslib/test/test_bgzf.c:344)
pub unsafe fn test_test_bgzf_c_344_cleanup(f: *mut Files, retval: i32) {
    if retval == libc::EXIT_SUCCESS {
        libc::unlink((*f).tmp_bgzf.as_ptr().cast());
        libc::unlink((*f).tmp_idx.as_ptr().cast());
    }
    if !(*f).f_plain.is_null() {
        libc::fclose((*f).f_plain);
    }
    if !(*f).f_bgzf.is_null() {
        libc::fclose((*f).f_bgzf);
    }
    if !(*f).f_idx.is_null() {
        libc::fclose((*f).f_idx);
    }
    // src_plain and text are owned Vec<u8>; dropped automatically with `f`.
}

// original: setup (htslib/test/test_bgzf.c:357)
pub unsafe fn test_test_bgzf_c_357_setup(src: *const u8, f: *mut Files) -> i32 {
    // Read the NUL-terminated `src` argument into a byte slice (without trailing NUL).
    let mut src_len = 0usize;
    while *src.add(src_len) != 0 {
        src_len += 1;
    }
    let src_bytes = std::slice::from_raw_parts(src, src_len);

    // Suffix constants carry a trailing NUL; strip it for concatenation.
    let bgzf_suffix = &BGZF_SUFFIX[..BGZF_SUFFIX.len() - 1];
    let idx_suffix = &IDX_SUFFIX[..IDX_SUFFIX.len() - 1];
    let tmp_suffix = &TMP_SUFFIX[..TMP_SUFFIX.len() - 1];

    let max = 50000u32;
    let text_sz = max as usize * 8 + 1;

    // Build the five NUL-terminated path names as owned Vec<u8>.
    let mut src_plain = src_bytes.to_vec();
    src_plain.push(0);

    let mut src_bgzf = src_bytes.to_vec();
    src_bgzf.extend_from_slice(bgzf_suffix);
    src_bgzf.push(0);

    let mut src_idx = src_bytes.to_vec();
    src_idx.extend_from_slice(bgzf_suffix);
    src_idx.extend_from_slice(idx_suffix);
    src_idx.push(0);

    let mut tmp_bgzf = src_bytes.to_vec();
    tmp_bgzf.extend_from_slice(tmp_suffix);
    tmp_bgzf.extend_from_slice(bgzf_suffix);
    tmp_bgzf.push(0);

    let mut tmp_idx = src_bytes.to_vec();
    tmp_idx.extend_from_slice(tmp_suffix);
    tmp_idx.extend_from_slice(bgzf_suffix);
    tmp_idx.extend_from_slice(idx_suffix);
    tmp_idx.push(0);

    (*f).src_plain = src_plain;
    (*f).src_bgzf = src_bgzf;
    (*f).src_idx = src_idx;
    (*f).tmp_bgzf = tmp_bgzf;
    (*f).tmp_idx = tmp_idx;

    // Build the text buffer: max lines of "%07u\n" plus a trailing NUL.
    let mut text = Vec::with_capacity(text_sz);
    for i in 0..max {
        text.extend_from_slice(format!("{:07}\n", i).as_bytes());
    }
    text.push(0);
    (*f).text = text;
    (*f).ltext = text_sz - 1;

    (*f).f_plain = test_test_bgzf_c_68_try_fopen((*f).src_plain.as_ptr(), b"rb\0".as_ptr());
    if (*f).f_plain.is_null() {
        return -1;
    }
    (*f).f_bgzf = test_test_bgzf_c_68_try_fopen((*f).src_bgzf.as_ptr(), b"rb\0".as_ptr());
    if (*f).f_bgzf.is_null() {
        return -1;
    }
    (*f).f_idx = test_test_bgzf_c_68_try_fopen((*f).src_idx.as_ptr(), b"rb\0".as_ptr());
    if (*f).f_idx.is_null() {
        return -1;
    }

    0
}

// original: test_read (htslib/test/test_bgzf.c:403)
pub unsafe fn test_test_bgzf_c_403_test_read(f: *mut Files) -> i32 {
    let mut bg_buf = [0u8; BUFSZ];
    let mut f_buf = [0u8; BUFSZ];

    *libc::__errno_location() = 0;
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).src_bgzf.as_ptr(), b"r\0".as_ptr(), b"test_read");
    if bgz.is_null() {
        return -1;
    }

    loop {
        let bg_got = test_test_bgzf_c_180_try_bgzf_read(
            bgz,
            bg_buf.as_mut_ptr().cast(),
            BUFSZ,
            (*f).src_bgzf.as_ptr(),
            b"test_read",
        );
        if bg_got < 0 {
            bgzf_close(bgz);
            return -1;
        }

        let f_got = test_test_bgzf_c_89_try_fread(
            (*f).f_plain,
            f_buf.as_mut_ptr().cast(),
            BUFSZ,
            b"test_read",
            (*f).src_plain.as_ptr(),
        );
        if f_got < 0 {
            bgzf_close(bgz);
            return -1;
        }

        if test_test_bgzf_c_327_compare_buffers(
            f_buf.as_ptr(),
            bg_buf.as_ptr(),
            f_got as usize,
            bg_got as usize,
            (*f).src_plain.as_ptr(),
            (*f).src_bgzf.as_ptr(),
            b"test_read",
        ) != 0
        {
            bgzf_close(bgz);
            return -1;
        }

        if !(bg_got > 0 && f_got > 0) {
            break;
        }
    }

    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).src_bgzf.as_ptr(), b"test_read", 0) != 0 {
        return -1;
    }
    if test_test_bgzf_c_100_try_fseek_start((*f).f_plain, (*f).src_plain.as_ptr(), b"test_read")
        != 0
    {
        return -1;
    }

    0
}

// original: test_write_read (htslib/test/test_bgzf.c:435)
pub unsafe fn test_test_bgzf_c_435_test_write_read(
    f: *mut Files,
    mode: *const u8,
    method: Open_method,
    nthreads: i32,
    expected_compression: i32,
) -> i32 {
    let mut pos = 0usize;
    let mut bg_buf = [0u8; BUFSZ];

    let mut bgz = match method {
        Open_method::USE_BGZF_DOPEN => {
            test_test_bgzf_c_120_try_bgzf_dopen((*f).tmp_bgzf.as_ptr(), mode, b"test_write_read")
        }
        Open_method::USE_BGZF_HOPEN => {
            test_test_bgzf_c_141_try_bgzf_hopen((*f).tmp_bgzf.as_ptr(), mode, b"test_write_read")
        }
        Open_method::USE_BGZF_OPEN => {
            test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf.as_ptr(), mode, b"test_write_read")
        }
    };
    if bgz.is_null() {
        return -1;
    }

    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, b"test_write_read") != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    let bg_put = test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        (*f).text.as_ptr().cast(),
        (*f).ltext,
        (*f).tmp_bgzf.as_ptr(),
        b"test_write_read",
    );
    if bg_put < 0 {
        bgzf_close(bgz);
        return -1;
    }

    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf.as_ptr(), b"test_write_read", 0)
        != 0
    {
        return -1;
    }

    bgz = match method {
        Open_method::USE_BGZF_DOPEN => test_test_bgzf_c_120_try_bgzf_dopen(
            (*f).tmp_bgzf.as_ptr(),
            b"r\0".as_ptr(),
            b"test_write_read",
        ),
        Open_method::USE_BGZF_HOPEN => test_test_bgzf_c_141_try_bgzf_hopen(
            (*f).tmp_bgzf.as_ptr(),
            b"r\0".as_ptr(),
            b"test_write_read",
        ),
        Open_method::USE_BGZF_OPEN => test_test_bgzf_c_109_try_bgzf_open(
            (*f).tmp_bgzf.as_ptr(),
            b"r\0".as_ptr(),
            b"test_write_read",
        ),
    };
    if bgz.is_null() {
        return -1;
    }

    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, b"test_write_read") != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    if test_test_bgzf_c_203_try_bgzf_compression(
        bgz,
        expected_compression,
        (*f).tmp_bgzf.as_ptr(),
        b"test_write_read",
    ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    loop {
        let bg_got = test_test_bgzf_c_180_try_bgzf_read(
            bgz,
            bg_buf.as_mut_ptr().cast(),
            BUFSZ,
            (*f).tmp_bgzf.as_ptr(),
            b"test_write_read",
        );
        if bg_got < 0 {
            bgzf_close(bgz);
            return -1;
        }

        if pos < (*f).ltext {
            let cmp_len = if pos + (bg_got as usize) < (*f).ltext {
                bg_got as usize
            } else {
                (*f).ltext - pos
            };
            if std::slice::from_raw_parts((*f).text.as_ptr().add(pos), cmp_len)
                != std::slice::from_raw_parts(bg_buf.as_ptr(), cmp_len)
            {
                eprintln!(
                    "{:?} : Got wrong data from {:?}, pos {}",
                    b"test_write_read",
                    (*f).tmp_bgzf.as_ptr(),
                    pos,
                );
                bgzf_close(bgz);
                return -1;
            }
        }
        pos += bg_got as usize;

        if bg_got <= 0 {
            break;
        }
    }

    if pos != bg_put as usize {
        eprintln!(
            "{:?} : bgzf_read got {} bytes; expected {}",
            b"test_write_read",
            pos,
            bg_put,
        );
        bgzf_close(bgz);
        return -1;
    }

    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf.as_ptr(), b"test_write_read", 0)
        != 0
    {
        return -1;
    }

    0
}

// original: test_embed_eof (htslib/test/test_bgzf.c:511)
pub unsafe fn test_test_bgzf_c_511_test_embed_eof(
    f: *mut Files,
    mode: *const u8,
    nthreads: i32,
) -> i32 {
    let mut pos = 0usize;
    let half = if BUFSZ < (*f).ltext {
        BUFSZ
    } else {
        (*f).ltext / 2
    };
    let mut append_mode = [0u8; 16];
    let mut bg_buf = [0u8; BUFSZ];

    while pos < append_mode.len() - 1 && *mode.add(pos) != 0 {
        append_mode[pos] = if *mode.add(pos) == b'w' {
            b'a'
        } else {
            *mode.add(pos)
        };
        pos += 1;
    }
    append_mode[pos] = 0;

    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf.as_ptr(), mode, b"test_embed_eof");
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, b"test_embed_eof") != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        (*f).text.as_ptr().cast(),
        half,
        (*f).tmp_bgzf.as_ptr(),
        b"test_embed_eof",
    ) < 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf.as_ptr(), b"test_embed_eof", 0)
        != 0
    {
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf.as_ptr(),
        append_mode.as_ptr(),
        b"test_embed_eof",
    );
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, b"test_embed_eof") != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        (*f).text.as_ptr().add(half).cast(),
        (*f).ltext - half,
        (*f).tmp_bgzf.as_ptr(),
        b"test_embed_eof",
    ) < 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf.as_ptr(), b"test_embed_eof", 0)
        != 0
    {
        return -1;
    }

    pos = 0;
    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf.as_ptr(),
        b"r\0".as_ptr(),
        b"test_embed_eof",
    );
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, b"test_embed_eof") != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    loop {
        let bg_got = test_test_bgzf_c_180_try_bgzf_read(
            bgz,
            bg_buf.as_mut_ptr().cast(),
            BUFSZ,
            (*f).tmp_bgzf.as_ptr(),
            b"test_embed_eof",
        );
        if bg_got < 0 {
            bgzf_close(bgz);
            return -1;
        }
        if pos < (*f).ltext {
            let cmp_len = if pos + (bg_got as usize) < (*f).ltext {
                bg_got as usize
            } else {
                (*f).ltext - pos
            };
            if std::slice::from_raw_parts((*f).text.as_ptr().add(pos), cmp_len)
                != std::slice::from_raw_parts(bg_buf.as_ptr(), cmp_len)
            {
                eprintln!(
                    "{:?} : Got wrong data from {:?}, pos {}",
                    b"test_embed_eof",
                    (*f).tmp_bgzf.as_ptr(),
                    pos,
                );
                bgzf_close(bgz);
                return -1;
            }
        }
        pos += bg_got as usize;
        if bg_got <= 0 {
            break;
        }
    }

    if pos != (*f).ltext {
        eprintln!(
            "{:?} : bgzf_read got {} bytes; expected {}",
            b"test_embed_eof",
            pos,
            (*f).ltext,
        );
        bgzf_close(bgz);
        return -1;
    }

    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf.as_ptr(), b"test_embed_eof", 0)
        != 0
    {
        return -1;
    }

    0
}

// original: test_index_load_dump (htslib/test/test_bgzf.c:584)
pub unsafe fn test_test_bgzf_c_584_test_index_load_dump(f: *mut Files) -> i32 {
    let mut bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).src_bgzf.as_ptr(),
        b"r\0".as_ptr(),
        b"test_index_load_dump",
    );
    let mut buf_src = [0u8; BUFSZ];
    let mut buf_dest = [0u8; BUFSZ];
    if bgz.is_null() {
        return -1;
    }
    if test_test_bgzf_c_235_try_bgzf_index_load(
        bgz,
        (*f).src_bgzf.as_ptr(),
        IDX_SUFFIX.as_ptr(),
        b"test_index_load_dump",
    ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_245_try_bgzf_index_dump(
        bgz,
        (*f).tmp_bgzf.as_ptr(),
        IDX_SUFFIX.as_ptr(),
        b"test_index_load_dump",
    ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    let mut fdest = test_test_bgzf_c_68_try_fopen((*f).tmp_idx.as_ptr(), b"r\0".as_ptr());
    loop {
        let got_src = test_test_bgzf_c_89_try_fread(
            (*f).f_idx,
            buf_src.as_mut_ptr().cast(),
            BUFSZ,
            b"test_index_load_dump",
            (*f).src_idx.as_ptr(),
        );
        if got_src < 0 {
            libc::fclose(fdest);
            bgzf_close(bgz);
            return -1;
        }
        let got_dest = test_test_bgzf_c_89_try_fread(
            fdest,
            buf_dest.as_mut_ptr().cast(),
            BUFSZ,
            b"test_index_load_dump",
            (*f).tmp_idx.as_ptr(),
        );
        if got_dest < 0 {
            libc::fclose(fdest);
            bgzf_close(bgz);
            return -1;
        }
        if test_test_bgzf_c_327_compare_buffers(
            buf_src.as_ptr(),
            buf_dest.as_ptr(),
            got_src as usize,
            got_dest as usize,
            (*f).src_idx.as_ptr(),
            (*f).tmp_idx.as_ptr(),
            b"test_index_load_dump",
        ) != 0
        {
            libc::fclose(fdest);
            bgzf_close(bgz);
            return -1;
        }
        if !(got_src > 0 && got_dest > 0) {
            break;
        }
    }
    if test_test_bgzf_c_77_try_fclose(&mut fdest, (*f).tmp_idx.as_ptr(), b"test_index_load_dump")
        != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).src_bgzf.as_ptr(),
        b"test_index_load_dump",
        0,
    ) != 0
    {
        return -1;
    }
    0
}

// original: test_check_EOF (htslib/test/test_bgzf.c:622)
pub unsafe fn test_test_bgzf_c_622_test_check_EOF(name: *mut u8, expected: i32) -> i32 {
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open(name, b"r\0".as_ptr(), b"test_check_EOF");
    if bgz.is_null() {
        return -1;
    }
    let eof = bgzf_check_EOF(bgz);
    if eof != expected {
        eprintln!(
            "{:?} : Unexpected result {} from bgzf_check_EOF on {:?}; expected {}",
            b"test_check_EOF",
            eof,
            name,
            expected,
        );
        bgzf_close(bgz);
        return -1;
    }
    test_test_bgzf_c_163_try_bgzf_close(&mut bgz, name, b"test_check_EOF", 0)
}

// original: test_index_useek_getc (htslib/test/test_bgzf.c:638)
pub unsafe fn test_test_bgzf_c_638_test_index_useek_getc(
    f: *mut Files,
    mode: *const u8,
    cache_size: i32,
    nthreads: i32,
) -> i32 {
    let mut bgz: *mut BGZF =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf.as_ptr(), mode, b"test_index_useek_getc");
    let iskip = (*f).ltext / 10;
    let is_uncompressed = !libc::strchr(mode.cast(), b'u' as i32).is_null();
    let offsets = [0usize, 100, 50];
    if bgz.is_null() {
        return -1;
    }
    if test_test_bgzf_c_225_try_bgzf_index_build_init(
        bgz,
        (*f).tmp_bgzf.as_ptr(),
        b"test_index_useek_getc",
    ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, b"test_index_useek_getc") != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        (*f).text.as_ptr().cast(),
        (*f).ltext,
        (*f).tmp_bgzf.as_ptr(),
        b"test_index_useek_getc",
    ) < 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if !is_uncompressed
        && test_test_bgzf_c_245_try_bgzf_index_dump(
            bgz,
            (*f).tmp_idx.as_ptr(),
            ptr::null(),
            b"test_index_useek_getc",
        ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf.as_ptr(),
        b"test_index_useek_getc",
        0,
    ) != 0
    {
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf.as_ptr(),
        b"r\0".as_ptr(),
        b"test_index_useek_getc",
    );
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, b"test_index_useek_getc") != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if !is_uncompressed
        && test_test_bgzf_c_235_try_bgzf_index_load(
            bgz,
            (*f).tmp_bgzf.as_ptr(),
            IDX_SUFFIX.as_ptr(),
            b"test_index_useek_getc",
        ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    let mut i = 0usize;
    while i < (*f).ltext {
        for &o in &offsets {
            if test_test_bgzf_c_288_try_bgzf_useek(
                bgz,
                (i + o) as i64,
                libc::SEEK_SET,
                (*f).tmp_bgzf.as_ptr(),
                b"test_index_useek_getc",
            ) != 0
            {
                bgzf_close(bgz);
                return -1;
            }
            let mut j = 0usize;
            while j < 16 && i + o + j < (*f).ltext {
                if test_test_bgzf_c_298_try_bgzf_getc(
                    bgz,
                    i + o + j,
                    *(*f).text.as_ptr().add(i + o + j) as i32,
                    (*f).tmp_bgzf.as_ptr(),
                    b"test_index_useek_getc",
                ) < 0
                {
                    bgzf_close(bgz);
                    return -1;
                }
                j += 1;
            }
        }
        i += iskip;
    }

    if test_test_bgzf_c_288_try_bgzf_useek(
        bgz,
        0,
        libc::SEEK_SET,
        (*f).tmp_bgzf.as_ptr(),
        b"test_index_useek_getc",
    ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    let mut j = 0usize;
    while j < 70000 && j < (*f).ltext {
        if test_test_bgzf_c_298_try_bgzf_getc(
            bgz,
            j,
            *(*f).text.as_ptr().add(j) as i32,
            (*f).tmp_bgzf.as_ptr(),
            b"test_index_useek_getc",
        ) < 0
        {
            bgzf_close(bgz);
            return -1;
        }
        j += 1;
    }

    if cache_size > 0 {
        let mid = (*f).ltext / 2;
        bgzf_set_cache_size(bgz, cache_size);
        for _ in 0..10 {
            if test_test_bgzf_c_288_try_bgzf_useek(
                bgz,
                0,
                libc::SEEK_SET,
                (*f).tmp_bgzf.as_ptr(),
                b"test_index_useek_getc",
            ) != 0
            {
                bgzf_close(bgz);
                return -1;
            }
            j = 0;
            while j < 64 && j < (*f).ltext {
                if test_test_bgzf_c_298_try_bgzf_getc(
                    bgz,
                    j,
                    *(*f).text.as_ptr().add(j) as i32,
                    (*f).tmp_bgzf.as_ptr(),
                    b"test_index_useek_getc",
                ) < 0
                {
                    bgzf_close(bgz);
                    return -1;
                }
                j += 1;
            }
            if test_test_bgzf_c_288_try_bgzf_useek(
                bgz,
                mid as i64,
                libc::SEEK_SET,
                (*f).tmp_bgzf.as_ptr(),
                b"test_index_useek_getc",
            ) != 0
            {
                bgzf_close(bgz);
                return -1;
            }
            j = 0;
            while j < 64 && j + mid < (*f).ltext {
                if test_test_bgzf_c_298_try_bgzf_getc(
                    bgz,
                    j + mid,
                    *(*f).text.as_ptr().add(j + mid) as i32,
                    (*f).tmp_bgzf.as_ptr(),
                    b"test_index_useek_getc",
                ) < 0
                {
                    bgzf_close(bgz);
                    return -1;
                }
                j += 1;
            }
        }
    }

    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf.as_ptr(),
        b"test_index_useek_getc",
        0,
    ) != 0
    {
        return -1;
    }
    0
}

// original: test_tell_seek_getc (htslib/test/test_bgzf.c:730)
pub unsafe fn test_test_bgzf_c_730_test_tell_seek_getc(
    f: *mut Files,
    mode: *const u8,
    cache_size: i32,
    nthreads: i32,
) -> i32 {
    let num_points = 10usize;
    let iskip = (*f).ltext / num_points;
    let offsets = [0usize, 100, 50];
    let mut points = vec![0usize; num_points];
    let mut point_vos = vec![0i64; num_points];
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf.as_ptr(), mode, b"test_tell_seek_getc");
    if bgz.is_null() {
        return -1;
    }
    for (i, (point, point_vo)) in points.iter_mut().zip(point_vos.iter_mut()).enumerate() {
        *point_vo =
            test_test_bgzf_c_255_try_bgzf_tell(bgz, (*f).tmp_bgzf.as_ptr(), b"test_tell_seek_getc");
        if *point_vo < 0 {
            bgzf_close(bgz);
            return -1;
        }
        *point = i * iskip;
        if test_test_bgzf_c_190_try_bgzf_write(
            bgz,
            (*f).text.as_ptr().add(i * iskip).cast(),
            iskip,
            (*f).tmp_bgzf.as_ptr(),
            b"test_tell_seek_getc",
        ) < 0
        {
            bgzf_close(bgz);
            return -1;
        }
    }
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf.as_ptr(),
        b"test_tell_seek_getc",
        0,
    ) != 0
    {
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf.as_ptr(),
        b"r\0".as_ptr(),
        b"test_tell_seek_getc",
    );
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, b"test_tell_seek_getc") != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    let mut i = 0usize;
    while i < (*f).ltext {
        for &o in &offsets {
            if test_test_bgzf_c_278_try_bgzf_seek(
                bgz,
                point_vos[i / iskip],
                libc::SEEK_SET,
                (*f).tmp_bgzf.as_ptr(),
                b"test_tell_seek_getc",
            ) != 0
                || test_test_bgzf_c_267_try_bgzf_tell_expect(
                    bgz,
                    point_vos[i / iskip],
                    (*f).tmp_bgzf.as_ptr(),
                    b"test_tell_seek_getc",
                ) < 0
                || test_test_bgzf_c_311_try_skip(
                    bgz,
                    o,
                    (*f).tmp_bgzf.as_ptr(),
                    b"test_tell_seek_getc",
                ) != 0
            {
                bgzf_close(bgz);
                return -1;
            }
            let mut j = 0usize;
            while j < 16 && i + o + j < (*f).ltext {
                if test_test_bgzf_c_298_try_bgzf_getc(
                    bgz,
                    i + o + j,
                    *(*f).text.as_ptr().add(i + o + j) as i32,
                    (*f).tmp_bgzf.as_ptr(),
                    b"test_tell_seek_getc",
                ) < 0
                {
                    bgzf_close(bgz);
                    return -1;
                }
                j += 1;
            }
        }
        i += iskip;
    }

    if test_test_bgzf_c_278_try_bgzf_seek(
        bgz,
        0,
        libc::SEEK_SET,
        (*f).tmp_bgzf.as_ptr(),
        b"test_tell_seek_getc",
    ) != 0
        || test_test_bgzf_c_267_try_bgzf_tell_expect(
            bgz,
            0,
            (*f).tmp_bgzf.as_ptr(),
            b"test_tell_seek_getc",
        ) < 0
    {
        bgzf_close(bgz);
        return -1;
    }
    let mut j = 0usize;
    while j < 70000 && j < (*f).ltext {
        if test_test_bgzf_c_298_try_bgzf_getc(
            bgz,
            j,
            *(*f).text.as_ptr().add(j) as i32,
            (*f).tmp_bgzf.as_ptr(),
            b"test_tell_seek_getc",
        ) < 0
        {
            bgzf_close(bgz);
            return -1;
        }
        j += 1;
    }

    if cache_size > 0 {
        let mid = points[num_points / 2];
        let mid_vo = point_vos[num_points / 2];
        bgzf_set_cache_size(bgz, cache_size);
        for _ in 0..10 {
            if test_test_bgzf_c_278_try_bgzf_seek(
                bgz,
                0,
                libc::SEEK_SET,
                (*f).tmp_bgzf.as_ptr(),
                b"test_tell_seek_getc",
            ) != 0
                || test_test_bgzf_c_267_try_bgzf_tell_expect(
                    bgz,
                    0,
                    (*f).tmp_bgzf.as_ptr(),
                    b"test_tell_seek_getc",
                ) < 0
            {
                bgzf_close(bgz);
                return -1;
            }
            j = 0;
            while j < 64 && j < (*f).ltext {
                if test_test_bgzf_c_298_try_bgzf_getc(
                    bgz,
                    j,
                    *(*f).text.as_ptr().add(j) as i32,
                    (*f).tmp_bgzf.as_ptr(),
                    b"test_tell_seek_getc",
                ) < 0
                {
                    bgzf_close(bgz);
                    return -1;
                }
                j += 1;
            }
            if test_test_bgzf_c_278_try_bgzf_seek(
                bgz,
                mid_vo,
                libc::SEEK_SET,
                (*f).tmp_bgzf.as_ptr(),
                b"test_tell_seek_getc",
            ) != 0
                || test_test_bgzf_c_267_try_bgzf_tell_expect(
                    bgz,
                    mid_vo,
                    (*f).tmp_bgzf.as_ptr(),
                    b"test_tell_seek_getc",
                ) < 0
            {
                bgzf_close(bgz);
                return -1;
            }
            j = 0;
            while j < 64 && j + mid < (*f).ltext {
                if test_test_bgzf_c_298_try_bgzf_getc(
                    bgz,
                    j + mid,
                    *(*f).text.as_ptr().add(j + mid) as i32,
                    (*f).tmp_bgzf.as_ptr(),
                    b"test_tell_seek_getc",
                ) < 0
                {
                    bgzf_close(bgz);
                    return -1;
                }
                j += 1;
            }
        }
    }

    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf.as_ptr(),
        b"test_tell_seek_getc",
        0,
    ) != 0
    {
        return -1;
    }
    0
}

// original: test_tell_read (htslib/test/test_bgzf.c:831)
pub unsafe fn test_test_bgzf_c_831_test_tell_read(f: *mut Files, mode: *const u8) -> i32 {
    let num_points = 10usize;
    let iskip = (*f).ltext / num_points;
    let mut point_vos = vec![0i64; num_points];
    let mut bg_buf = vec![0u8; iskip + 1];
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf.as_ptr(), mode, b"test_tell_read");
    if bgz.is_null() {
        return -1;
    }

    for (i, point_vo) in point_vos.iter_mut().enumerate() {
        *point_vo =
            test_test_bgzf_c_255_try_bgzf_tell(bgz, (*f).tmp_bgzf.as_ptr(), b"test_tell_read");
        if *point_vo < 0
            || test_test_bgzf_c_190_try_bgzf_write(
                bgz,
                (*f).text.as_ptr().add(i * iskip).cast(),
                iskip,
                (*f).tmp_bgzf.as_ptr(),
                b"test_tell_read",
            ) < 0
        {
            bgzf_close(bgz);
            return -1;
        }
    }
    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf.as_ptr(), b"test_tell_read", 0)
        != 0
    {
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf.as_ptr(),
        b"r\0".as_ptr(),
        b"test_tell_read",
    );
    if bgz.is_null() {
        return -1;
    }

    let mut i = 0usize;
    while i < (*f).ltext {
        if test_test_bgzf_c_267_try_bgzf_tell_expect(
            bgz,
            point_vos[i / iskip],
            (*f).tmp_bgzf.as_ptr(),
            b"test_tell_read",
        ) < 0
            || test_test_bgzf_c_180_try_bgzf_read(
                bgz,
                bg_buf.as_mut_ptr().cast(),
                iskip,
                (*f).tmp_bgzf.as_ptr(),
                b"test_tell_read",
            ) < 0
            || test_test_bgzf_c_327_compare_buffers(
                (*f).text.as_ptr().add(i),
                bg_buf.as_ptr(),
                iskip,
                iskip,
                (*f).tmp_bgzf.as_ptr(),
                (*f).tmp_bgzf.as_ptr(),
                b"test_tell_read",
            ) != 0
        {
            eprintln!("{:?}: failed", b"test_tell_read");
            bgzf_close(bgz);
            return -1;
        }
        i += iskip;
    }
    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf.as_ptr(), b"test_tell_read", 0)
        != 0
    {
        return -1;
    }
    0
}

// original: test_useek_read_small (htslib/test/test_bgzf.c:881)
pub unsafe fn test_test_bgzf_c_881_test_useek_read_small(
    f: *mut Files,
    mode: *const u8,
) -> i32 {
    let mut bg_buf = [0u8; 99];
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf.as_ptr(), mode, b"test_useek_read_small");
    if bgz.is_null() {
        return -1;
    }
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        b"#>Hello, World!\n".as_ptr().cast(),
        16,
        (*f).tmp_bgzf.as_ptr(),
        b"test_useek_read_small",
    ) != 16
        || test_test_bgzf_c_163_try_bgzf_close(
            &mut bgz,
            (*f).tmp_bgzf.as_ptr(),
            b"test_useek_read_small",
            0,
        ) != 0
    {
        if !bgz.is_null() {
            bgzf_close(bgz);
        }
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf.as_ptr(),
        b"r\0".as_ptr(),
        b"test_useek_read_small",
    );
    if bgz.is_null() {
        return -1;
    }
    if test_test_bgzf_c_298_try_bgzf_getc(
        bgz,
        0,
        b'#' as i32,
        (*f).tmp_bgzf.as_ptr(),
        b"test_useek_read_small",
    ) < 0
        || test_test_bgzf_c_298_try_bgzf_getc(
            bgz,
            1,
            b'>' as i32,
            (*f).tmp_bgzf.as_ptr(),
            b"test_useek_read_small",
        ) < 0
        || test_test_bgzf_c_180_try_bgzf_read(
            bgz,
            bg_buf.as_mut_ptr().cast(),
            5,
            (*f).tmp_bgzf.as_ptr(),
            b"test_useek_read_small",
        ) != 5
        || bg_buf[..5] != b"Hello"[..]
        || test_test_bgzf_c_288_try_bgzf_useek(
            bgz,
            9,
            libc::SEEK_SET,
            (*f).tmp_bgzf.as_ptr(),
            b"test_useek_read_small",
        ) < 0
        || test_test_bgzf_c_180_try_bgzf_read(
            bgz,
            bg_buf.as_mut_ptr().cast(),
            5,
            (*f).tmp_bgzf.as_ptr(),
            b"test_useek_read_small",
        ) != 5
        || bg_buf[..5] != b"World"[..]
    {
        eprintln!("{:?}: failed", b"test_useek_read_small");
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf.as_ptr(),
        b"test_useek_read_small",
        0,
    ) != 0
    {
        return -1;
    }
    0
}

// original: test_bgzf_getline (htslib/test/test_bgzf.c:924)
pub unsafe fn test_test_bgzf_c_924_test_bgzf_getline(
    f: *mut Files,
    mode: *const u8,
    nthreads: i32,
) -> i32 {
    let mut str_: kstring_t = kstring_t::default();
    let text = (*f).text.as_ptr();
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf.as_ptr(), mode, b"test_bgzf_getline");
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, b"test_bgzf_getline") != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        (*f).text.as_ptr().cast(),
        (*f).ltext,
        (*f).tmp_bgzf.as_ptr(),
        b"test_bgzf_getline",
    ) < 0
        || test_test_bgzf_c_163_try_bgzf_close(
            &mut bgz,
            (*f).tmp_bgzf.as_ptr(),
            b"test_bgzf_getline",
            0,
        ) != 0
    {
        if !bgz.is_null() {
            bgzf_close(bgz);
        }
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf.as_ptr(),
        b"r\0".as_ptr(),
        b"test_bgzf_getline",
    );
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, b"test_bgzf_getline") != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    let mut pos = 0usize;
    while pos < (*f).ltext {
        let end = libc::strchr(text.add(pos).cast(), b'\n' as i32).cast::<u8>();
        let l = if end.is_null() {
            (*f).ltext - pos
        } else {
            end.offset_from(text.add(pos)) as usize
        };
        let res = bgzf_getline(bgz, b'\n' as i32, &mut str_);
        if res < 0 {
            eprintln!(
                "{:?} : {} from bgzf_getline on {:?} : errno {}",
                b"test_bgzf_getline",
                if res < -1 {
                    "Error"
                } else {
                    "Unexpected EOF"
                },
                (*f).tmp_bgzf.as_ptr(),
                *libc::__errno_location(),
            );
            bgzf_close(bgz);
            ks_release(&mut str_);
            return -1;
        }
        if str_.data.len() != l
            || std::slice::from_raw_parts(text.add(pos), l) != &str_.data[..l]
        {
            eprintln!(
                "{:?} : Unexpected data from bgzf_getline on {:?}\nExpected : {:?}\nGot      : {:?}",
                b"test_bgzf_getline",
                (*f).tmp_bgzf.as_ptr(),
                std::slice::from_raw_parts((*f).text.as_ptr().add(pos), l),
                &str_.data[..],
            );
            bgzf_close(bgz);
            ks_release(&mut str_);
            return -1;
        }
        pos += l + 1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf.as_ptr(),
        b"test_bgzf_getline",
        0,
    ) != 0
    {
        ks_release(&mut str_);
        return -1;
    }
    ks_release(&mut str_);
    0
}

// original: test_bgzf_getline_on_truncated_file (htslib/test/test_bgzf.c:981)
pub unsafe fn test_test_bgzf_c_981_test_bgzf_getline_on_truncated_file(
    f: *mut Files,
    mode: *const u8,
    nthreads: i32,
) -> i32 {
    let mut str_: kstring_t = kstring_t::default();
    let text = (*f).text.as_ptr();
    let lvl = hts_get_log_level();
    hts_set_log_level(HTS_LOG_OFF);

    let mut bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf.as_ptr(),
        mode,
        b"test_bgzf_getline_on_truncated_file",
    );
    if bgz.is_null() {
        hts_set_log_level(lvl);
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(
            bgz,
            nthreads,
            b"test_bgzf_getline_on_truncated_file",
        ) != 0
    {
        hts_set_log_level(lvl);
        bgzf_close(bgz);
        return -1;
    }
    let text_line2 = libc::strchr(text.cast(), b'\n' as i32).cast::<u8>().add(1);
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        text.cast(),
        text_line2.offset_from(text) as usize,
        (*f).tmp_bgzf.as_ptr(),
        b"test_bgzf_getline_on_truncated_file",
    ) < 0
        || bgzf_flush(bgz) < 0
    {
        hts_set_log_level(lvl);
        bgzf_close(bgz);
        return -1;
    }
    let block2_start = (*bgz).block_address;
    let text_line3 = libc::strchr(text_line2.cast(), b'\n' as i32).cast::<u8>().add(1);
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        text_line2.cast(),
        text_line3.offset_from(text_line2) as usize,
        (*f).tmp_bgzf.as_ptr(),
        b"test_bgzf_getline_on_truncated_file",
    ) < 0
        || bgzf_flush(bgz) < 0
    {
        hts_set_log_level(lvl);
        bgzf_close(bgz);
        return -1;
    }
    let block3_start = (*bgz).block_address;
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf.as_ptr(),
        b"test_bgzf_getline_on_truncated_file",
        0,
    ) != 0
    {
        hts_set_log_level(lvl);
        return -1;
    }

    let mut newsize = block3_start - 1;
    while newsize > block2_start {
        if libc::truncate((*f).tmp_bgzf.as_ptr().cast(), newsize) != 0 {
            hts_set_log_level(lvl);
            ks_release(&mut str_);
            return -1;
        }
        bgz = test_test_bgzf_c_109_try_bgzf_open(
            (*f).tmp_bgzf.as_ptr(),
            b"r\0".as_ptr(),
            b"test_bgzf_getline_on_truncated_file",
        );
        if bgz.is_null() {
            hts_set_log_level(lvl);
            ks_release(&mut str_);
            return -1;
        }
        if nthreads > 0
            && test_test_bgzf_c_216_try_bgzf_mt(
                bgz,
                nthreads,
                b"test_bgzf_getline_on_truncated_file",
            ) != 0
        {
            hts_set_log_level(lvl);
            bgzf_close(bgz);
            ks_release(&mut str_);
            return -1;
        }

        let mut pos = 0usize;
        while pos < (*f).ltext {
            let end = libc::strchr(text.add(pos).cast(), b'\n' as i32).cast::<u8>();
            let l = if end.is_null() {
                (*f).ltext - pos
            } else {
                end.offset_from(text.add(pos)) as usize
            };
            let res = bgzf_getline(bgz, b'\n' as i32, &mut str_);
            if res < -1 {
                break;
            } else if res == -1 {
                eprintln!(
                    "{:?} : {} from bgzf_getline on {:?}",
                    b"test_bgzf_getline_on_truncated_file",
                    "Unexpected EOF",
                    (*f).tmp_bgzf.as_ptr(),
                );
                hts_set_log_level(lvl);
                bgzf_close(bgz);
                ks_release(&mut str_);
                return -1;
            }
            if str_.data.len() != l
                || std::slice::from_raw_parts(text.add(pos), l) != &str_.data[..l]
            {
                eprintln!(
                    "{:?} : Unexpected data from bgzf_getline on {:?}\nExpected : {:?}\nGot      : {:?}",
                    b"test_bgzf_getline_on_truncated_file",
                    (*f).tmp_bgzf.as_ptr(),
                    std::slice::from_raw_parts((*f).text.as_ptr().add(pos), l),
                    &str_.data[..],
                );
                hts_set_log_level(lvl);
                bgzf_close(bgz);
                ks_release(&mut str_);
                return -1;
            }
            pos += l + 1;
        }

        for _ in 0..3 {
            let res = bgzf_getline(bgz, b'\n' as i32, &mut str_);
            if res > -2 {
                eprintln!(
                    "{:?} : unexpected bgzf_getline result {}",
                    b"test_bgzf_getline_on_truncated_file",
                    res,
                );
                hts_set_log_level(lvl);
                bgzf_close(bgz);
                ks_release(&mut str_);
                return -1;
            }
        }
        if test_test_bgzf_c_163_try_bgzf_close(
            &mut bgz,
            (*f).tmp_bgzf.as_ptr(),
            b"test_bgzf_getline_on_truncated_file",
            1,
        ) == 0
        {
            hts_set_log_level(lvl);
            ks_release(&mut str_);
            return -1;
        }
        newsize -= 1;
    }
    ks_release(&mut str_);
    hts_set_log_level(lvl);
    0
}

// original: main (htslib/test/test_bgzf.c:1073)
pub unsafe fn test_test_bgzf_c_1073_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut f = Files {
        src_plain: Vec::new(),
        src_bgzf: Vec::new(),
        src_idx: Vec::new(),
        tmp_bgzf: Vec::new(),
        tmp_idx: Vec::new(),
        f_plain: ptr::null_mut(),
        f_bgzf: ptr::null_mut(),
        f_idx: ptr::null_mut(),
        text: Vec::new(),
        ltext: 0,
    };
    let mut retval = libc::EXIT_FAILURE;

    if argc != 2 {
        eprintln!("Usage: {:?} <source file>", *argv);
        return libc::EXIT_FAILURE;
    }

    if test_test_bgzf_c_357_setup(*argv.add(1), &mut f) != 0 {
        test_test_bgzf_c_344_cleanup(&mut f, retval);
        return retval;
    }

    macro_rules! run {
        ($expr:expr) => {
            if $expr != 0 {
                test_test_bgzf_c_344_cleanup(&mut f, retval);
                return retval;
            }
        };
    }

    run!(test_test_bgzf_c_622_test_check_EOF(f.src_bgzf.as_mut_ptr(), 1));
    run!(test_test_bgzf_c_403_test_read(&mut f));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        b"wu\0".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        0,
        0
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf.as_mut_ptr(), 0));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        b"w\0".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        0,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf.as_mut_ptr(), 1));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        b"w0\0".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        0,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf.as_mut_ptr(), 1));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        b"w1\0".as_ptr(),
        Open_method::USE_BGZF_DOPEN,
        0,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf.as_mut_ptr(), 1));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        b"w9\0".as_ptr(),
        Open_method::USE_BGZF_HOPEN,
        0,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf.as_mut_ptr(), 1));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        b"wg\0".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        0,
        1
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf.as_mut_ptr(), 0));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        b"w\0".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        1,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf.as_mut_ptr(), 1));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        b"w\0".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        2,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf.as_mut_ptr(), 1));
    run!(test_test_bgzf_c_511_test_embed_eof(
        &mut f,
        b"w\0".as_ptr(),
        0
    ));
    run!(test_test_bgzf_c_511_test_embed_eof(
        &mut f,
        b"w\0".as_ptr(),
        1
    ));
    run!(test_test_bgzf_c_511_test_embed_eof(
        &mut f,
        b"w\0".as_ptr(),
        2
    ));
    run!(test_test_bgzf_c_584_test_index_load_dump(&mut f));
    run!(test_test_bgzf_c_638_test_index_useek_getc(
        &mut f,
        b"w\0".as_ptr(),
        1000000,
        0
    ));
    run!(test_test_bgzf_c_638_test_index_useek_getc(
        &mut f,
        b"w\0".as_ptr(),
        1000000,
        1
    ));
    run!(test_test_bgzf_c_638_test_index_useek_getc(
        &mut f,
        b"w\0".as_ptr(),
        1000000,
        2
    ));
    run!(test_test_bgzf_c_638_test_index_useek_getc(
        &mut f,
        b"wu\0".as_ptr(),
        0,
        0
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"w\0".as_ptr(),
        0,
        0
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"wu\0".as_ptr(),
        0,
        0
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"w\0".as_ptr(),
        1000000,
        0
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"wu\0".as_ptr(),
        1000000,
        0
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"w\0".as_ptr(),
        0,
        1
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"w\0".as_ptr(),
        0,
        2
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"wu\0".as_ptr(),
        0,
        1
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"wu\0".as_ptr(),
        0,
        2
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"w\0".as_ptr(),
        1000000,
        1
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"w\0".as_ptr(),
        1000000,
        2
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"wu\0".as_ptr(),
        1000000,
        1
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        b"wu\0".as_ptr(),
        1000000,
        2
    ));
    run!(test_test_bgzf_c_831_test_tell_read(&mut f, b"w\0".as_ptr()));
    run!(test_test_bgzf_c_831_test_tell_read(&mut f, b"wu\0".as_ptr()));
    run!(test_test_bgzf_c_881_test_useek_read_small(
        &mut f,
        b"w\0".as_ptr()
    ));
    run!(test_test_bgzf_c_881_test_useek_read_small(
        &mut f,
        b"wu\0".as_ptr()
    ));
    run!(test_test_bgzf_c_924_test_bgzf_getline(
        &mut f,
        b"w\0".as_ptr(),
        0
    ));
    run!(test_test_bgzf_c_924_test_bgzf_getline(
        &mut f,
        b"w\0".as_ptr(),
        1
    ));
    run!(test_test_bgzf_c_924_test_bgzf_getline(
        &mut f,
        b"w\0".as_ptr(),
        2
    ));
    run!(test_test_bgzf_c_981_test_bgzf_getline_on_truncated_file(
        &mut f,
        b"w\0".as_ptr(),
        0
    ));
    run!(test_test_bgzf_c_981_test_bgzf_getline_on_truncated_file(
        &mut f,
        b"w\0".as_ptr(),
        1
    ));
    run!(test_test_bgzf_c_981_test_bgzf_getline_on_truncated_file(
        &mut f,
        b"w\0".as_ptr(),
        2
    ));

    retval = libc::EXIT_SUCCESS;
    test_test_bgzf_c_344_cleanup(&mut f, retval);
    retval
}
