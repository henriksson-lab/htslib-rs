use crate::htslib_mini_rs::{
    bgzf::{
        bgzf_check_EOF, bgzf_close, bgzf_compression, bgzf_dopen, bgzf_flush, bgzf_getc,
        bgzf_getline, bgzf_hopen, bgzf_index_build_init, bgzf_index_dump, bgzf_index_load, bgzf_mt,
        bgzf_open, bgzf_read_small, bgzf_seek, bgzf_set_cache_size, bgzf_useek, bgzf_write_small,
    },
    hfile::{hclose_abruptly, hfile_oflags, hopen},
    hts::{hts_get_log_level, hts_set_log_level, ks_release, kstring_t, BGZF, HTS_LOG_OFF},
};
use std::{
    ffi::{c_char, c_int, c_void},
    ptr,
};

pub const BGZF_SUFFIX: *const c_char = c".gz".as_ptr();
pub const IDX_SUFFIX: *const c_char = c".gzi".as_ptr();
pub const TMP_SUFFIX: *const c_char = c".tmp".as_ptr();

const BUFSZ: usize = 32768;

#[repr(C)]
pub struct Files {
    pub src_plain: *mut c_char,
    pub src_bgzf: *mut c_char,
    pub src_idx: *mut c_char,
    pub tmp_bgzf: *mut c_char,
    pub tmp_idx: *mut c_char,
    pub f_plain: *mut libc::FILE,
    pub f_bgzf: *mut libc::FILE,
    pub f_idx: *mut libc::FILE,
    pub text: *const u8,
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
    name: *const c_char,
    mode: *const c_char,
) -> *mut libc::FILE {
    let f = libc::fopen(name, mode);
    if f.is_null() {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Couldn't open %s : %s\n".as_ptr(),
            name,
            libc::strerror(*libc::__errno_location()),
        );
        return ptr::null_mut();
    }
    f
}

// original: try_fclose (htslib/test/test_bgzf.c:77)
pub unsafe fn test_test_bgzf_c_77_try_fclose(
    file: *mut *mut libc::FILE,
    name: *const c_char,
    func: *const c_char,
) -> c_int {
    let to_close = *file;
    *file = ptr::null_mut();
    if libc::fclose(to_close) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Error on closing %s : %s\n".as_ptr(),
            func,
            name,
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }

    0
}

// original: try_fread (htslib/test/test_bgzf.c:89)
pub unsafe fn test_test_bgzf_c_89_try_fread(
    in_: *mut libc::FILE,
    buf: *mut c_void,
    len: usize,
    func: *const c_char,
    fname: *const c_char,
) -> libc::ssize_t {
    let got = libc::fread(buf, 1, len, in_);
    if got == 0 && libc::ferror(in_) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Error reading from %s : %s\n".as_ptr(),
            func,
            fname,
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }
    got as libc::ssize_t
}

// original: try_fseek_start (htslib/test/test_bgzf.c:100)
pub unsafe fn test_test_bgzf_c_100_try_fseek_start(
    f: *mut libc::FILE,
    name: *const c_char,
    func: *const c_char,
) -> c_int {
    if libc::fseek(f, 0, libc::SEEK_SET) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Couldn't seek on %s : %s\n".as_ptr(),
            func,
            name,
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }
    0
}

// original: try_bgzf_open (htslib/test/test_bgzf.c:109)
pub unsafe fn test_test_bgzf_c_109_try_bgzf_open(
    name: *const c_char,
    mode: *const c_char,
    func: *const c_char,
) -> *mut BGZF {
    let bgz = bgzf_open(name, mode);
    if bgz.is_null() {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Couldn't bgzf_open %s with mode %s : %s\n".as_ptr(),
            func,
            name,
            mode,
            libc::strerror(*libc::__errno_location()),
        );
        return ptr::null_mut();
    }
    bgz
}

// original: try_bgzf_dopen (htslib/test/test_bgzf.c:120)
pub unsafe fn test_test_bgzf_c_120_try_bgzf_dopen(
    name: *const c_char,
    mode: *const c_char,
    func: *const c_char,
) -> *mut BGZF {
    let fd = libc::open(name, hfile_oflags(mode), 0o666);
    if fd < 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Failed to open %s with mode %s : %s\n".as_ptr(),
            func,
            name,
            mode,
            libc::strerror(*libc::__errno_location()),
        );
        return ptr::null_mut();
    }

    let bgz = bgzf_dopen(fd, mode);
    if bgz.is_null() {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : bgzf_dopen failed on %s mode %s : %s\n".as_ptr(),
            func,
            name,
            mode,
            libc::strerror(*libc::__errno_location()),
        );
        libc::close(fd);
        return ptr::null_mut();
    }

    bgz
}

// original: try_bgzf_hopen (htslib/test/test_bgzf.c:141)
pub unsafe fn test_test_bgzf_c_141_try_bgzf_hopen(
    name: *const c_char,
    mode: *const c_char,
    func: *const c_char,
) -> *mut BGZF {
    let hfp = hopen(name, mode);
    if hfp.is_null() {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : hopen failed on %s mode %s : %s\n".as_ptr(),
            func,
            name,
            mode,
            libc::strerror(*libc::__errno_location()),
        );
        return ptr::null_mut();
    }

    let bgz = bgzf_hopen(hfp, mode);
    if bgz.is_null() {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : bgzf_hopen failed on %s mode %s : %s\n".as_ptr(),
            func,
            name,
            mode,
            libc::strerror(*libc::__errno_location()),
        );
        hclose_abruptly(hfp);
        return ptr::null_mut();
    }

    bgz
}

// original: try_bgzf_close (htslib/test/test_bgzf.c:163)
pub unsafe fn test_test_bgzf_c_163_try_bgzf_close(
    bgz: *mut *mut BGZF,
    name: *const c_char,
    func: *const c_char,
    expected_fail: c_int,
) -> c_int {
    let to_close = *bgz;
    *bgz = ptr::null_mut();
    if bgzf_close(to_close) != 0 {
        if expected_fail == 0 {
            let errno = *libc::__errno_location();
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"%s : bgzf_close failed on %s%s%s\n".as_ptr(),
                func,
                name,
                if errno != 0 {
                    c" : ".as_ptr()
                } else {
                    c"".as_ptr()
                },
                if errno != 0 {
                    libc::strerror(errno)
                } else {
                    c"".as_ptr()
                },
            );
        }
        return -1;
    } else if expected_fail != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : bgzf_close worked on %s, but expected failure\n".as_ptr(),
            func,
            name,
        );
    }
    0
}

// original: try_bgzf_read (htslib/test/test_bgzf.c:180)
pub unsafe fn test_test_bgzf_c_180_try_bgzf_read(
    fp: *mut BGZF,
    data: *mut c_void,
    length: usize,
    name: *const c_char,
    func: *const c_char,
) -> libc::ssize_t {
    let got = bgzf_read_small(fp, data, length);
    if got < 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Error from bgzf_read %s : %s\n".as_ptr(),
            func,
            name,
            libc::strerror(*libc::__errno_location()),
        );
    }
    got as libc::ssize_t
}

// original: try_bgzf_write (htslib/test/test_bgzf.c:190)
pub unsafe fn test_test_bgzf_c_190_try_bgzf_write(
    fp: *mut BGZF,
    data: *const c_void,
    length: usize,
    name: *const c_char,
    func: *const c_char,
) -> libc::ssize_t {
    let put = bgzf_write_small(fp, data, length);
    if put < length as isize {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : %s %s : %s\n".as_ptr(),
            func,
            if put < 0 {
                c"Error writing to".as_ptr()
            } else {
                c"Short write on".as_ptr()
            },
            name,
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }

    put as libc::ssize_t
}

// original: try_bgzf_compression (htslib/test/test_bgzf.c:203)
pub unsafe fn test_test_bgzf_c_203_try_bgzf_compression(
    fp: *mut BGZF,
    expect: c_int,
    name: *const c_char,
    func: *const c_char,
) -> c_int {
    let res = bgzf_compression(fp);
    if res != expect {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Unexpected result %d from bgzf_compression on %s; expected %d\n".as_ptr(),
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
    nthreads: c_int,
    func: *const c_char,
) -> c_int {
    if bgzf_mt(bgz, nthreads, 64) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Error from bgzf_mt : %s\n".as_ptr(),
            func,
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }
    0
}

// original: try_bgzf_index_build_init (htslib/test/test_bgzf.c:225)
pub unsafe fn test_test_bgzf_c_225_try_bgzf_index_build_init(
    bgz: *mut BGZF,
    name: *const c_char,
    func: *const c_char,
) -> c_int {
    if bgzf_index_build_init(bgz) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Error from bgzf_index_build_init on %s : %s\n".as_ptr(),
            func,
            name,
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }
    0
}

// original: try_bgzf_index_load (htslib/test/test_bgzf.c:235)
pub unsafe fn test_test_bgzf_c_235_try_bgzf_index_load(
    fp: *mut BGZF,
    bname: *const c_char,
    suffix: *const c_char,
    func: *const c_char,
) -> c_int {
    if bgzf_index_load(fp, bname, suffix) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Couldn't bgzf_index_load %s%s : %s\n".as_ptr(),
            func,
            bname,
            if suffix.is_null() {
                c"".as_ptr()
            } else {
                suffix
            },
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }
    0
}

// original: try_bgzf_index_dump (htslib/test/test_bgzf.c:245)
pub unsafe fn test_test_bgzf_c_245_try_bgzf_index_dump(
    fp: *mut BGZF,
    bname: *const c_char,
    suffix: *const c_char,
    func: *const c_char,
) -> c_int {
    if bgzf_index_dump(fp, bname, suffix) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Couldn't bgzf_index_dump %s%s : %s\n".as_ptr(),
            func,
            bname,
            if suffix.is_null() {
                c"".as_ptr()
            } else {
                suffix
            },
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }
    0
}

// original: try_bgzf_tell (htslib/test/test_bgzf.c:255)
pub unsafe fn test_test_bgzf_c_255_try_bgzf_tell(
    fp: *mut BGZF,
    name: *const c_char,
    func: *const c_char,
) -> i64 {
    let told = (((*fp).block_address as u64) << 16 | ((*fp).block_offset as u64 & 0xffff)) as i64;
    if told < 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : %s %s : %s\n".as_ptr(),
            func,
            c"Error telling in".as_ptr(),
            name,
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }
    told
}

// original: try_bgzf_tell_expect (htslib/test/test_bgzf.c:267)
pub unsafe fn test_test_bgzf_c_267_try_bgzf_tell_expect(
    fp: *mut BGZF,
    expected: i64,
    name: *const c_char,
    func: *const c_char,
) -> i64 {
    let told = test_test_bgzf_c_255_try_bgzf_tell(fp, name, func);
    if told != expected {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Unexpected value (%ld) from bgzf_tell on %s; expected %ld\n".as_ptr(),
            func,
            told as libc::c_long,
            name,
            expected as libc::c_long,
        );
        return -1;
    }
    told
}

// original: try_bgzf_seek (htslib/test/test_bgzf.c:278)
pub unsafe fn test_test_bgzf_c_278_try_bgzf_seek(
    fp: *mut BGZF,
    pos: i64,
    whence: c_int,
    name: *const c_char,
    func: *const c_char,
) -> c_int {
    if bgzf_seek(fp, pos, whence) < 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Error from bgzf_seek(%s, %ld, %d) : %s\n".as_ptr(),
            func,
            name,
            pos as libc::c_long,
            whence,
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }
    0
}

// original: try_bgzf_useek (htslib/test/test_bgzf.c:288)
pub unsafe fn test_test_bgzf_c_288_try_bgzf_useek(
    fp: *mut BGZF,
    uoffset: libc::c_long,
    where_: c_int,
    name: *const c_char,
    func: *const c_char,
) -> c_int {
    if bgzf_useek(fp, uoffset as i64, where_) < 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Error from bgzf_useek(%s, %ld, %d) : %s\n".as_ptr(),
            func,
            name,
            uoffset,
            where_,
            libc::strerror(*libc::__errno_location()),
        );
        return -1;
    }
    0
}

// original: try_bgzf_getc (htslib/test/test_bgzf.c:298)
pub unsafe fn test_test_bgzf_c_298_try_bgzf_getc(
    fp: *mut BGZF,
    pos: usize,
    expected: c_int,
    name: *const c_char,
    func: *const c_char,
) -> c_int {
    let c = bgzf_getc(fp);
    if c != expected {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Unexpected value (%d) from bgzf_getc on %s pos %zu; expected %d\n".as_ptr(),
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
    name: *const c_char,
    func: *const c_char,
) -> c_int {
    for _ in 0..count {
        let c = bgzf_getc(fp);
        if c < 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"%s : Error from bgzf_getc on %s\n".as_ptr(),
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
    name1: *const c_char,
    name2: *const c_char,
    func: *const c_char,
) -> c_int {
    if l1 != l2 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : EOF on %s\n".as_ptr(),
            func,
            if l1 < l2 { name1 } else { name2 },
        );
        return -1;
    }
    if libc::memcmp(b1.cast(), b2.cast(), l1) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : difference between %s and %s\n".as_ptr(),
            func,
            name1,
            name2,
        );
        return -1;
    }

    0
}

// original: cleanup (htslib/test/test_bgzf.c:344)
pub unsafe fn test_test_bgzf_c_344_cleanup(f: *mut Files, retval: c_int) {
    if retval == libc::EXIT_SUCCESS {
        libc::unlink((*f).tmp_bgzf);
        libc::unlink((*f).tmp_idx);
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
    libc::free((*f).src_plain.cast());
    libc::free((*f).text.cast_mut().cast());
}

// original: setup (htslib/test/test_bgzf.c:357)
pub unsafe fn test_test_bgzf_c_357_setup(src: *const c_char, f: *mut Files) -> c_int {
    let len = libc::strlen(src)
        + libc::strlen(BGZF_SUFFIX)
        + libc::strlen(IDX_SUFFIX)
        + libc::strlen(TMP_SUFFIX)
        + 8;
    let max = 50000u32;
    let text_sz = max as usize * 8 + 1;

    let mem = libc::calloc(5, len).cast::<c_char>();
    if mem.is_null() {
        libc::perror(c"test_test_bgzf_c_357_setup".as_ptr());
        return -1;
    }

    libc::snprintf(mem, len, c"%s".as_ptr(), src);
    libc::snprintf(mem.add(len), len, c"%s%s".as_ptr(), src, BGZF_SUFFIX);
    libc::snprintf(
        mem.add(len * 2),
        len,
        c"%s%s%s".as_ptr(),
        src,
        BGZF_SUFFIX,
        IDX_SUFFIX,
    );
    libc::snprintf(
        mem.add(len * 3),
        len,
        c"%s%s%s".as_ptr(),
        src,
        TMP_SUFFIX,
        BGZF_SUFFIX,
    );
    libc::snprintf(
        mem.add(len * 4),
        len,
        c"%s%s%s%s".as_ptr(),
        src,
        TMP_SUFFIX,
        BGZF_SUFFIX,
        IDX_SUFFIX,
    );

    (*f).src_plain = mem;
    (*f).src_bgzf = mem.add(len);
    (*f).src_idx = mem.add(len * 2);
    (*f).tmp_bgzf = mem.add(len * 3);
    (*f).tmp_idx = mem.add(len * 4);

    let text = libc::malloc(text_sz).cast::<c_char>();
    if text.is_null() {
        libc::perror(c"test_test_bgzf_c_357_setup".as_ptr());
        return -1;
    }
    for i in 0..max {
        libc::snprintf(
            text.add(i as usize * 8),
            text_sz - i as usize * 8,
            c"%07u\n".as_ptr(),
            i,
        );
    }
    (*f).text = text.cast();
    (*f).ltext = text_sz - 1;

    (*f).f_plain = test_test_bgzf_c_68_try_fopen((*f).src_plain, c"rb".as_ptr());
    if (*f).f_plain.is_null() {
        return -1;
    }
    (*f).f_bgzf = test_test_bgzf_c_68_try_fopen((*f).src_bgzf, c"rb".as_ptr());
    if (*f).f_bgzf.is_null() {
        return -1;
    }
    (*f).f_idx = test_test_bgzf_c_68_try_fopen((*f).src_idx, c"rb".as_ptr());
    if (*f).f_idx.is_null() {
        return -1;
    }

    0
}

// original: test_read (htslib/test/test_bgzf.c:403)
pub unsafe fn test_test_bgzf_c_403_test_read(f: *mut Files) -> c_int {
    let mut bg_buf = [0u8; BUFSZ];
    let mut f_buf = [0u8; BUFSZ];

    *libc::__errno_location() = 0;
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).src_bgzf, c"r".as_ptr(), c"test_read".as_ptr());
    if bgz.is_null() {
        return -1;
    }

    loop {
        let bg_got = test_test_bgzf_c_180_try_bgzf_read(
            bgz,
            bg_buf.as_mut_ptr().cast(),
            BUFSZ,
            (*f).src_bgzf,
            c"test_read".as_ptr(),
        );
        if bg_got < 0 {
            bgzf_close(bgz);
            return -1;
        }

        let f_got = test_test_bgzf_c_89_try_fread(
            (*f).f_plain,
            f_buf.as_mut_ptr().cast(),
            BUFSZ,
            c"test_read".as_ptr(),
            (*f).src_plain,
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
            (*f).src_plain,
            (*f).src_bgzf,
            c"test_read".as_ptr(),
        ) != 0
        {
            bgzf_close(bgz);
            return -1;
        }

        if !(bg_got > 0 && f_got > 0) {
            break;
        }
    }

    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).src_bgzf, c"test_read".as_ptr(), 0) != 0 {
        return -1;
    }
    if test_test_bgzf_c_100_try_fseek_start((*f).f_plain, (*f).src_plain, c"test_read".as_ptr())
        != 0
    {
        return -1;
    }

    0
}

// original: test_write_read (htslib/test/test_bgzf.c:435)
pub unsafe fn test_test_bgzf_c_435_test_write_read(
    f: *mut Files,
    mode: *const c_char,
    method: Open_method,
    nthreads: c_int,
    expected_compression: c_int,
) -> c_int {
    let mut bgz: *mut BGZF = ptr::null_mut();
    let mut pos = 0usize;
    let mut bg_buf = [0u8; BUFSZ];

    bgz = match method {
        Open_method::USE_BGZF_DOPEN => {
            test_test_bgzf_c_120_try_bgzf_dopen((*f).tmp_bgzf, mode, c"test_write_read".as_ptr())
        }
        Open_method::USE_BGZF_HOPEN => {
            test_test_bgzf_c_141_try_bgzf_hopen((*f).tmp_bgzf, mode, c"test_write_read".as_ptr())
        }
        Open_method::USE_BGZF_OPEN => {
            test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf, mode, c"test_write_read".as_ptr())
        }
    };
    if bgz.is_null() {
        return -1;
    }

    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, c"test_write_read".as_ptr()) != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    let bg_put = test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        (*f).text.cast(),
        (*f).ltext,
        (*f).tmp_bgzf,
        c"test_write_read".as_ptr(),
    );
    if bg_put < 0 {
        bgzf_close(bgz);
        return -1;
    }

    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf, c"test_write_read".as_ptr(), 0)
        != 0
    {
        return -1;
    }

    bgz = match method {
        Open_method::USE_BGZF_DOPEN => test_test_bgzf_c_120_try_bgzf_dopen(
            (*f).tmp_bgzf,
            c"r".as_ptr(),
            c"test_write_read".as_ptr(),
        ),
        Open_method::USE_BGZF_HOPEN => test_test_bgzf_c_141_try_bgzf_hopen(
            (*f).tmp_bgzf,
            c"r".as_ptr(),
            c"test_write_read".as_ptr(),
        ),
        Open_method::USE_BGZF_OPEN => test_test_bgzf_c_109_try_bgzf_open(
            (*f).tmp_bgzf,
            c"r".as_ptr(),
            c"test_write_read".as_ptr(),
        ),
    };
    if bgz.is_null() {
        return -1;
    }

    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, c"test_write_read".as_ptr()) != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    if test_test_bgzf_c_203_try_bgzf_compression(
        bgz,
        expected_compression,
        (*f).tmp_bgzf,
        c"test_write_read".as_ptr(),
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
            (*f).tmp_bgzf,
            c"test_write_read".as_ptr(),
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
            if libc::memcmp((*f).text.add(pos).cast(), bg_buf.as_ptr().cast(), cmp_len) != 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"%s : Got wrong data from %s, pos %zu\n".as_ptr(),
                    c"test_write_read".as_ptr(),
                    (*f).tmp_bgzf,
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
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : bgzf_read got %zd bytes; expected %zd\n".as_ptr(),
            c"test_write_read".as_ptr(),
            pos,
            bg_put,
        );
        bgzf_close(bgz);
        return -1;
    }

    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf, c"test_write_read".as_ptr(), 0)
        != 0
    {
        return -1;
    }

    0
}

// original: test_embed_eof (htslib/test/test_bgzf.c:511)
pub unsafe fn test_test_bgzf_c_511_test_embed_eof(
    f: *mut Files,
    mode: *const c_char,
    nthreads: c_int,
) -> c_int {
    let mut bgz: *mut BGZF = ptr::null_mut();
    let mut pos = 0usize;
    let half = if BUFSZ < (*f).ltext {
        BUFSZ
    } else {
        (*f).ltext / 2
    };
    let mut append_mode = [0 as c_char; 16];
    let mut bg_buf = [0u8; BUFSZ];

    while pos < append_mode.len() - 1 && *mode.add(pos) != 0 {
        append_mode[pos] = if *mode.add(pos) == b'w' as c_char {
            b'a' as c_char
        } else {
            *mode.add(pos)
        };
        pos += 1;
    }
    append_mode[pos] = 0;

    bgz = test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf, mode, c"test_embed_eof".as_ptr());
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, c"test_embed_eof".as_ptr()) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        (*f).text.cast(),
        half,
        (*f).tmp_bgzf,
        c"test_embed_eof".as_ptr(),
    ) < 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf, c"test_embed_eof".as_ptr(), 0)
        != 0
    {
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf,
        append_mode.as_ptr(),
        c"test_embed_eof".as_ptr(),
    );
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, c"test_embed_eof".as_ptr()) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        (*f).text.add(half).cast(),
        (*f).ltext - half,
        (*f).tmp_bgzf,
        c"test_embed_eof".as_ptr(),
    ) < 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf, c"test_embed_eof".as_ptr(), 0)
        != 0
    {
        return -1;
    }

    pos = 0;
    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf,
        c"r".as_ptr(),
        c"test_embed_eof".as_ptr(),
    );
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, c"test_embed_eof".as_ptr()) != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    loop {
        let bg_got = test_test_bgzf_c_180_try_bgzf_read(
            bgz,
            bg_buf.as_mut_ptr().cast(),
            BUFSZ,
            (*f).tmp_bgzf,
            c"test_embed_eof".as_ptr(),
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
            if libc::memcmp((*f).text.add(pos).cast(), bg_buf.as_ptr().cast(), cmp_len) != 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"%s : Got wrong data from %s, pos %zu\n".as_ptr(),
                    c"test_embed_eof".as_ptr(),
                    (*f).tmp_bgzf,
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
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : bgzf_read got %zd bytes; expected %zd\n".as_ptr(),
            c"test_embed_eof".as_ptr(),
            pos,
            (*f).ltext,
        );
        bgzf_close(bgz);
        return -1;
    }

    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf, c"test_embed_eof".as_ptr(), 0)
        != 0
    {
        return -1;
    }

    0
}

// original: test_index_load_dump (htslib/test/test_bgzf.c:584)
pub unsafe fn test_test_bgzf_c_584_test_index_load_dump(f: *mut Files) -> c_int {
    let mut bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).src_bgzf,
        c"r".as_ptr(),
        c"test_index_load_dump".as_ptr(),
    );
    let mut fdest: *mut libc::FILE = ptr::null_mut();
    let mut buf_src = [0u8; BUFSZ];
    let mut buf_dest = [0u8; BUFSZ];
    if bgz.is_null() {
        return -1;
    }
    if test_test_bgzf_c_235_try_bgzf_index_load(
        bgz,
        (*f).src_bgzf,
        IDX_SUFFIX,
        c"test_index_load_dump".as_ptr(),
    ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_245_try_bgzf_index_dump(
        bgz,
        (*f).tmp_bgzf,
        IDX_SUFFIX,
        c"test_index_load_dump".as_ptr(),
    ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    fdest = test_test_bgzf_c_68_try_fopen((*f).tmp_idx, c"r".as_ptr());
    loop {
        let got_src = test_test_bgzf_c_89_try_fread(
            (*f).f_idx,
            buf_src.as_mut_ptr().cast(),
            BUFSZ,
            c"test_index_load_dump".as_ptr(),
            (*f).src_idx,
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
            c"test_index_load_dump".as_ptr(),
            (*f).tmp_idx,
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
            (*f).src_idx,
            (*f).tmp_idx,
            c"test_index_load_dump".as_ptr(),
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
    if test_test_bgzf_c_77_try_fclose(&mut fdest, (*f).tmp_idx, c"test_index_load_dump".as_ptr())
        != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).src_bgzf,
        c"test_index_load_dump".as_ptr(),
        0,
    ) != 0
    {
        return -1;
    }
    0
}

// original: test_check_EOF (htslib/test/test_bgzf.c:622)
pub unsafe fn test_test_bgzf_c_622_test_check_EOF(name: *mut c_char, expected: c_int) -> c_int {
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open(name, c"r".as_ptr(), c"test_check_EOF".as_ptr());
    if bgz.is_null() {
        return -1;
    }
    let eof = bgzf_check_EOF(bgz);
    if eof != expected {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s : Unexpected result %d from bgzf_check_EOF on %s; expected %d\n".as_ptr(),
            c"test_check_EOF".as_ptr(),
            eof,
            name,
            expected,
        );
        bgzf_close(bgz);
        return -1;
    }
    test_test_bgzf_c_163_try_bgzf_close(&mut bgz, name, c"test_check_EOF".as_ptr(), 0)
}

// original: test_index_useek_getc (htslib/test/test_bgzf.c:638)
pub unsafe fn test_test_bgzf_c_638_test_index_useek_getc(
    f: *mut Files,
    mode: *const c_char,
    cache_size: c_int,
    nthreads: c_int,
) -> c_int {
    let mut bgz: *mut BGZF =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf, mode, c"test_index_useek_getc".as_ptr());
    let iskip = (*f).ltext / 10;
    let is_uncompressed = !libc::strchr(mode, b'u' as c_int).is_null();
    let offsets = [0usize, 100, 50];
    if bgz.is_null() {
        return -1;
    }
    if test_test_bgzf_c_225_try_bgzf_index_build_init(
        bgz,
        (*f).tmp_bgzf,
        c"test_index_useek_getc".as_ptr(),
    ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, c"test_index_useek_getc".as_ptr()) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        (*f).text.cast(),
        (*f).ltext,
        (*f).tmp_bgzf,
        c"test_index_useek_getc".as_ptr(),
    ) < 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if !is_uncompressed
        && test_test_bgzf_c_245_try_bgzf_index_dump(
            bgz,
            (*f).tmp_idx,
            ptr::null(),
            c"test_index_useek_getc".as_ptr(),
        ) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf,
        c"test_index_useek_getc".as_ptr(),
        0,
    ) != 0
    {
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf,
        c"r".as_ptr(),
        c"test_index_useek_getc".as_ptr(),
    );
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, c"test_index_useek_getc".as_ptr()) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if !is_uncompressed
        && test_test_bgzf_c_235_try_bgzf_index_load(
            bgz,
            (*f).tmp_bgzf,
            IDX_SUFFIX,
            c"test_index_useek_getc".as_ptr(),
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
                (i + o) as libc::c_long,
                libc::SEEK_SET,
                (*f).tmp_bgzf,
                c"test_index_useek_getc".as_ptr(),
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
                    *(*f).text.add(i + o + j) as c_int,
                    (*f).tmp_bgzf,
                    c"test_index_useek_getc".as_ptr(),
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
        (*f).tmp_bgzf,
        c"test_index_useek_getc".as_ptr(),
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
            *(*f).text.add(j) as c_int,
            (*f).tmp_bgzf,
            c"test_index_useek_getc".as_ptr(),
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
                (*f).tmp_bgzf,
                c"test_index_useek_getc".as_ptr(),
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
                    *(*f).text.add(j) as c_int,
                    (*f).tmp_bgzf,
                    c"test_index_useek_getc".as_ptr(),
                ) < 0
                {
                    bgzf_close(bgz);
                    return -1;
                }
                j += 1;
            }
            if test_test_bgzf_c_288_try_bgzf_useek(
                bgz,
                mid as libc::c_long,
                libc::SEEK_SET,
                (*f).tmp_bgzf,
                c"test_index_useek_getc".as_ptr(),
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
                    *(*f).text.add(j + mid) as c_int,
                    (*f).tmp_bgzf,
                    c"test_index_useek_getc".as_ptr(),
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
        (*f).tmp_bgzf,
        c"test_index_useek_getc".as_ptr(),
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
    mode: *const c_char,
    cache_size: c_int,
    nthreads: c_int,
) -> c_int {
    let num_points = 10usize;
    let iskip = (*f).ltext / num_points;
    let offsets = [0usize, 100, 50];
    let mut points = vec![0usize; num_points];
    let mut point_vos = vec![0i64; num_points];
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf, mode, c"test_tell_seek_getc".as_ptr());
    if bgz.is_null() {
        return -1;
    }
    for i in 0..num_points {
        point_vos[i] =
            test_test_bgzf_c_255_try_bgzf_tell(bgz, (*f).tmp_bgzf, c"test_tell_seek_getc".as_ptr());
        if point_vos[i] < 0 {
            bgzf_close(bgz);
            return -1;
        }
        points[i] = i * iskip;
        if test_test_bgzf_c_190_try_bgzf_write(
            bgz,
            (*f).text.add(i * iskip).cast(),
            iskip,
            (*f).tmp_bgzf,
            c"test_tell_seek_getc".as_ptr(),
        ) < 0
        {
            bgzf_close(bgz);
            return -1;
        }
    }
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf,
        c"test_tell_seek_getc".as_ptr(),
        0,
    ) != 0
    {
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf,
        c"r".as_ptr(),
        c"test_tell_seek_getc".as_ptr(),
    );
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, c"test_tell_seek_getc".as_ptr()) != 0
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
                (*f).tmp_bgzf,
                c"test_tell_seek_getc".as_ptr(),
            ) != 0
                || test_test_bgzf_c_267_try_bgzf_tell_expect(
                    bgz,
                    point_vos[i / iskip],
                    (*f).tmp_bgzf,
                    c"test_tell_seek_getc".as_ptr(),
                ) < 0
                || test_test_bgzf_c_311_try_skip(
                    bgz,
                    o,
                    (*f).tmp_bgzf,
                    c"test_tell_seek_getc".as_ptr(),
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
                    *(*f).text.add(i + o + j) as c_int,
                    (*f).tmp_bgzf,
                    c"test_tell_seek_getc".as_ptr(),
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
        (*f).tmp_bgzf,
        c"test_tell_seek_getc".as_ptr(),
    ) != 0
        || test_test_bgzf_c_267_try_bgzf_tell_expect(
            bgz,
            0,
            (*f).tmp_bgzf,
            c"test_tell_seek_getc".as_ptr(),
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
            *(*f).text.add(j) as c_int,
            (*f).tmp_bgzf,
            c"test_tell_seek_getc".as_ptr(),
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
                (*f).tmp_bgzf,
                c"test_tell_seek_getc".as_ptr(),
            ) != 0
                || test_test_bgzf_c_267_try_bgzf_tell_expect(
                    bgz,
                    0,
                    (*f).tmp_bgzf,
                    c"test_tell_seek_getc".as_ptr(),
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
                    *(*f).text.add(j) as c_int,
                    (*f).tmp_bgzf,
                    c"test_tell_seek_getc".as_ptr(),
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
                (*f).tmp_bgzf,
                c"test_tell_seek_getc".as_ptr(),
            ) != 0
                || test_test_bgzf_c_267_try_bgzf_tell_expect(
                    bgz,
                    mid_vo,
                    (*f).tmp_bgzf,
                    c"test_tell_seek_getc".as_ptr(),
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
                    *(*f).text.add(j + mid) as c_int,
                    (*f).tmp_bgzf,
                    c"test_tell_seek_getc".as_ptr(),
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
        (*f).tmp_bgzf,
        c"test_tell_seek_getc".as_ptr(),
        0,
    ) != 0
    {
        return -1;
    }
    0
}

// original: test_tell_read (htslib/test/test_bgzf.c:831)
pub unsafe fn test_test_bgzf_c_831_test_tell_read(f: *mut Files, mode: *const c_char) -> c_int {
    let num_points = 10usize;
    let iskip = (*f).ltext / num_points;
    let mut point_vos = vec![0i64; num_points];
    let bg_buf = libc::calloc(iskip + 1, 1).cast::<u8>();
    if bg_buf.is_null() {
        return -1;
    }
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf, mode, c"test_tell_read".as_ptr());
    if bgz.is_null() {
        libc::free(bg_buf.cast());
        return -1;
    }

    for i in 0..num_points {
        point_vos[i] =
            test_test_bgzf_c_255_try_bgzf_tell(bgz, (*f).tmp_bgzf, c"test_tell_read".as_ptr());
        if point_vos[i] < 0
            || test_test_bgzf_c_190_try_bgzf_write(
                bgz,
                (*f).text.add(i * iskip).cast(),
                iskip,
                (*f).tmp_bgzf,
                c"test_tell_read".as_ptr(),
            ) < 0
        {
            bgzf_close(bgz);
            libc::free(bg_buf.cast());
            return -1;
        }
    }
    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf, c"test_tell_read".as_ptr(), 0)
        != 0
    {
        libc::free(bg_buf.cast());
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf,
        c"r".as_ptr(),
        c"test_tell_read".as_ptr(),
    );
    if bgz.is_null() {
        libc::free(bg_buf.cast());
        return -1;
    }

    let mut i = 0usize;
    while i < (*f).ltext {
        if test_test_bgzf_c_267_try_bgzf_tell_expect(
            bgz,
            point_vos[i / iskip],
            (*f).tmp_bgzf,
            c"test_tell_read".as_ptr(),
        ) < 0
            || test_test_bgzf_c_180_try_bgzf_read(
                bgz,
                bg_buf.cast(),
                iskip,
                (*f).tmp_bgzf,
                c"test_tell_read".as_ptr(),
            ) < 0
            || test_test_bgzf_c_327_compare_buffers(
                (*f).text.add(i),
                bg_buf,
                iskip,
                iskip,
                (*f).tmp_bgzf,
                (*f).tmp_bgzf,
                c"test_tell_read".as_ptr(),
            ) != 0
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"%s: failed\n".as_ptr(),
                c"test_tell_read".as_ptr(),
            );
            bgzf_close(bgz);
            libc::free(bg_buf.cast());
            return -1;
        }
        i += iskip;
    }
    if test_test_bgzf_c_163_try_bgzf_close(&mut bgz, (*f).tmp_bgzf, c"test_tell_read".as_ptr(), 0)
        != 0
    {
        libc::free(bg_buf.cast());
        return -1;
    }
    libc::free(bg_buf.cast());
    0
}

// original: test_useek_read_small (htslib/test/test_bgzf.c:881)
pub unsafe fn test_test_bgzf_c_881_test_useek_read_small(
    f: *mut Files,
    mode: *const c_char,
) -> c_int {
    let mut bg_buf = [0 as c_char; 99];
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf, mode, c"test_useek_read_small".as_ptr());
    if bgz.is_null() {
        return -1;
    }
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        c"#>Hello, World!\n".as_ptr().cast(),
        16,
        (*f).tmp_bgzf,
        c"test_useek_read_small".as_ptr(),
    ) != 16
        || test_test_bgzf_c_163_try_bgzf_close(
            &mut bgz,
            (*f).tmp_bgzf,
            c"test_useek_read_small".as_ptr(),
            0,
        ) != 0
    {
        if !bgz.is_null() {
            bgzf_close(bgz);
        }
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf,
        c"r".as_ptr(),
        c"test_useek_read_small".as_ptr(),
    );
    if bgz.is_null() {
        return -1;
    }
    if test_test_bgzf_c_298_try_bgzf_getc(
        bgz,
        0,
        b'#' as c_int,
        (*f).tmp_bgzf,
        c"test_useek_read_small".as_ptr(),
    ) < 0
        || test_test_bgzf_c_298_try_bgzf_getc(
            bgz,
            1,
            b'>' as c_int,
            (*f).tmp_bgzf,
            c"test_useek_read_small".as_ptr(),
        ) < 0
        || test_test_bgzf_c_180_try_bgzf_read(
            bgz,
            bg_buf.as_mut_ptr().cast(),
            5,
            (*f).tmp_bgzf,
            c"test_useek_read_small".as_ptr(),
        ) != 5
        || libc::memcmp(bg_buf.as_ptr().cast(), c"Hello".as_ptr().cast(), 5) != 0
        || test_test_bgzf_c_288_try_bgzf_useek(
            bgz,
            9,
            libc::SEEK_SET,
            (*f).tmp_bgzf,
            c"test_useek_read_small".as_ptr(),
        ) < 0
        || test_test_bgzf_c_180_try_bgzf_read(
            bgz,
            bg_buf.as_mut_ptr().cast(),
            5,
            (*f).tmp_bgzf,
            c"test_useek_read_small".as_ptr(),
        ) != 5
        || libc::memcmp(bg_buf.as_ptr().cast(), c"World".as_ptr().cast(), 5) != 0
    {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%s: failed\n".as_ptr(),
            c"test_useek_read_small".as_ptr(),
        );
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf,
        c"test_useek_read_small".as_ptr(),
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
    mode: *const c_char,
    nthreads: c_int,
) -> c_int {
    let mut str_: kstring_t = std::mem::zeroed();
    let text = (*f).text.cast::<c_char>();
    let mut bgz =
        test_test_bgzf_c_109_try_bgzf_open((*f).tmp_bgzf, mode, c"test_bgzf_getline".as_ptr());
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, c"test_bgzf_getline".as_ptr()) != 0
    {
        bgzf_close(bgz);
        return -1;
    }
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        (*f).text.cast(),
        (*f).ltext,
        (*f).tmp_bgzf,
        c"test_bgzf_getline".as_ptr(),
    ) < 0
        || test_test_bgzf_c_163_try_bgzf_close(
            &mut bgz,
            (*f).tmp_bgzf,
            c"test_bgzf_getline".as_ptr(),
            0,
        ) != 0
    {
        if !bgz.is_null() {
            bgzf_close(bgz);
        }
        return -1;
    }

    bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf,
        c"r".as_ptr(),
        c"test_bgzf_getline".as_ptr(),
    );
    if bgz.is_null() {
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(bgz, nthreads, c"test_bgzf_getline".as_ptr()) != 0
    {
        bgzf_close(bgz);
        return -1;
    }

    let mut pos = 0usize;
    while pos < (*f).ltext {
        let end = libc::strchr(text.add(pos), b'\n' as c_int);
        let l = if end.is_null() {
            (*f).ltext - pos
        } else {
            end.offset_from(text.add(pos)) as usize
        };
        let res = bgzf_getline(bgz, b'\n' as c_int, &mut str_);
        if res < 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"%s : %s from bgzf_getline on %s : %s\n".as_ptr(),
                c"test_bgzf_getline".as_ptr(),
                if res < -1 {
                    c"Error".as_ptr()
                } else {
                    c"Unexpected EOF".as_ptr()
                },
                (*f).tmp_bgzf,
                if res < -1 {
                    libc::strerror(*libc::__errno_location())
                } else {
                    c"EOF".as_ptr()
                },
            );
            bgzf_close(bgz);
            libc::free(ks_release(&mut str_).cast());
            return -1;
        }
        if str_.l != l || libc::memcmp(text.add(pos).cast(), str_.s.cast(), l) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"%s : Unexpected data from bgzf_getline on %s\nExpected : %.*s\nGot      : %.*s\n"
                    .as_ptr(),
                c"test_bgzf_getline".as_ptr(),
                (*f).tmp_bgzf,
                l as c_int,
                (*f).text.add(pos).cast::<c_char>(),
                str_.l as c_int,
                str_.s,
            );
            bgzf_close(bgz);
            libc::free(ks_release(&mut str_).cast());
            return -1;
        }
        pos += l + 1;
    }
    if test_test_bgzf_c_163_try_bgzf_close(
        &mut bgz,
        (*f).tmp_bgzf,
        c"test_bgzf_getline".as_ptr(),
        0,
    ) != 0
    {
        libc::free(ks_release(&mut str_).cast());
        return -1;
    }
    libc::free(ks_release(&mut str_).cast());
    0
}

// original: test_bgzf_getline_on_truncated_file (htslib/test/test_bgzf.c:981)
pub unsafe fn test_test_bgzf_c_981_test_bgzf_getline_on_truncated_file(
    f: *mut Files,
    mode: *const c_char,
    nthreads: c_int,
) -> c_int {
    let mut str_: kstring_t = std::mem::zeroed();
    let text = (*f).text.cast::<c_char>();
    let lvl = hts_get_log_level();
    hts_set_log_level(HTS_LOG_OFF);

    let mut bgz = test_test_bgzf_c_109_try_bgzf_open(
        (*f).tmp_bgzf,
        mode,
        c"test_bgzf_getline_on_truncated_file".as_ptr(),
    );
    if bgz.is_null() {
        hts_set_log_level(lvl);
        return -1;
    }
    if nthreads > 0
        && test_test_bgzf_c_216_try_bgzf_mt(
            bgz,
            nthreads,
            c"test_bgzf_getline_on_truncated_file".as_ptr(),
        ) != 0
    {
        hts_set_log_level(lvl);
        bgzf_close(bgz);
        return -1;
    }
    let text_line2 = libc::strchr(text, b'\n' as c_int).add(1);
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        text.cast(),
        text_line2.offset_from(text) as usize,
        (*f).tmp_bgzf,
        c"test_bgzf_getline_on_truncated_file".as_ptr(),
    ) < 0
        || bgzf_flush(bgz) < 0
    {
        hts_set_log_level(lvl);
        bgzf_close(bgz);
        return -1;
    }
    let block2_start = (*bgz).block_address;
    let text_line3 = libc::strchr(text_line2, b'\n' as c_int).add(1);
    if test_test_bgzf_c_190_try_bgzf_write(
        bgz,
        text_line2.cast(),
        text_line3.offset_from(text_line2) as usize,
        (*f).tmp_bgzf,
        c"test_bgzf_getline_on_truncated_file".as_ptr(),
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
        (*f).tmp_bgzf,
        c"test_bgzf_getline_on_truncated_file".as_ptr(),
        0,
    ) != 0
    {
        hts_set_log_level(lvl);
        return -1;
    }

    let mut newsize = block3_start - 1;
    while newsize > block2_start {
        if libc::truncate((*f).tmp_bgzf, newsize) != 0 {
            hts_set_log_level(lvl);
            libc::free(ks_release(&mut str_).cast());
            return -1;
        }
        bgz = test_test_bgzf_c_109_try_bgzf_open(
            (*f).tmp_bgzf,
            c"r".as_ptr(),
            c"test_bgzf_getline_on_truncated_file".as_ptr(),
        );
        if bgz.is_null() {
            hts_set_log_level(lvl);
            libc::free(ks_release(&mut str_).cast());
            return -1;
        }
        if nthreads > 0
            && test_test_bgzf_c_216_try_bgzf_mt(
                bgz,
                nthreads,
                c"test_bgzf_getline_on_truncated_file".as_ptr(),
            ) != 0
        {
            hts_set_log_level(lvl);
            bgzf_close(bgz);
            libc::free(ks_release(&mut str_).cast());
            return -1;
        }

        let mut pos = 0usize;
        while pos < (*f).ltext {
            let end = libc::strchr(text.add(pos), b'\n' as c_int);
            let l = if end.is_null() {
                (*f).ltext - pos
            } else {
                end.offset_from(text.add(pos)) as usize
            };
            let res = bgzf_getline(bgz, b'\n' as c_int, &mut str_);
            if res < -1 {
                break;
            } else if res == -1 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"%s : %s from bgzf_getline on %s\n".as_ptr(),
                    c"test_bgzf_getline_on_truncated_file".as_ptr(),
                    c"Unexpected EOF".as_ptr(),
                    (*f).tmp_bgzf,
                );
                hts_set_log_level(lvl);
                bgzf_close(bgz);
                libc::free(ks_release(&mut str_).cast());
                return -1;
            }
            if str_.l != l || libc::memcmp(text.add(pos).cast(), str_.s.cast(), l) != 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"%s : Unexpected data from bgzf_getline on %s\nExpected : %.*s\nGot      : %.*s\n".as_ptr(),
                    c"test_bgzf_getline_on_truncated_file".as_ptr(),
                    (*f).tmp_bgzf,
                    l as c_int,
                    (*f).text.add(pos).cast::<c_char>(),
                    str_.l as c_int,
                    str_.s,
                );
                hts_set_log_level(lvl);
                bgzf_close(bgz);
                libc::free(ks_release(&mut str_).cast());
                return -1;
            }
            pos += l + 1;
        }

        for _ in 0..3 {
            let res = bgzf_getline(bgz, b'\n' as c_int, &mut str_);
            if res > -2 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"%s : unexpected bgzf_getline result %d\n".as_ptr(),
                    c"test_bgzf_getline_on_truncated_file".as_ptr(),
                    res,
                );
                hts_set_log_level(lvl);
                bgzf_close(bgz);
                libc::free(ks_release(&mut str_).cast());
                return -1;
            }
        }
        if test_test_bgzf_c_163_try_bgzf_close(
            &mut bgz,
            (*f).tmp_bgzf,
            c"test_bgzf_getline_on_truncated_file".as_ptr(),
            1,
        ) == 0
        {
            hts_set_log_level(lvl);
            libc::free(ks_release(&mut str_).cast());
            return -1;
        }
        newsize -= 1;
    }
    libc::free(ks_release(&mut str_).cast());
    hts_set_log_level(lvl);
    0
}

// original: main (htslib/test/test_bgzf.c:1073)
pub unsafe fn test_test_bgzf_c_1073_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut f = Files {
        src_plain: ptr::null_mut(),
        src_bgzf: ptr::null_mut(),
        src_idx: ptr::null_mut(),
        tmp_bgzf: ptr::null_mut(),
        tmp_idx: ptr::null_mut(),
        f_plain: ptr::null_mut(),
        f_bgzf: ptr::null_mut(),
        f_idx: ptr::null_mut(),
        text: ptr::null(),
        ltext: 0,
    };
    let mut retval = libc::EXIT_FAILURE;

    if argc != 2 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Usage: %s <source file>\n".as_ptr(),
            *argv,
        );
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

    run!(test_test_bgzf_c_622_test_check_EOF(f.src_bgzf, 1));
    run!(test_test_bgzf_c_403_test_read(&mut f));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        c"wu".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        0,
        0
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf, 0));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        c"w".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        0,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf, 1));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        c"w0".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        0,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf, 1));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        c"w1".as_ptr(),
        Open_method::USE_BGZF_DOPEN,
        0,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf, 1));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        c"w9".as_ptr(),
        Open_method::USE_BGZF_HOPEN,
        0,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf, 1));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        c"wg".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        0,
        1
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf, 0));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        c"w".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        1,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf, 1));
    run!(test_test_bgzf_c_435_test_write_read(
        &mut f,
        c"w".as_ptr(),
        Open_method::USE_BGZF_OPEN,
        2,
        2
    ));
    run!(test_test_bgzf_c_622_test_check_EOF(f.tmp_bgzf, 1));
    run!(test_test_bgzf_c_511_test_embed_eof(
        &mut f,
        c"w".as_ptr(),
        0
    ));
    run!(test_test_bgzf_c_511_test_embed_eof(
        &mut f,
        c"w".as_ptr(),
        1
    ));
    run!(test_test_bgzf_c_511_test_embed_eof(
        &mut f,
        c"w".as_ptr(),
        2
    ));
    run!(test_test_bgzf_c_584_test_index_load_dump(&mut f));
    run!(test_test_bgzf_c_638_test_index_useek_getc(
        &mut f,
        c"w".as_ptr(),
        1000000,
        0
    ));
    run!(test_test_bgzf_c_638_test_index_useek_getc(
        &mut f,
        c"w".as_ptr(),
        1000000,
        1
    ));
    run!(test_test_bgzf_c_638_test_index_useek_getc(
        &mut f,
        c"w".as_ptr(),
        1000000,
        2
    ));
    run!(test_test_bgzf_c_638_test_index_useek_getc(
        &mut f,
        c"wu".as_ptr(),
        0,
        0
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"w".as_ptr(),
        0,
        0
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"wu".as_ptr(),
        0,
        0
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"w".as_ptr(),
        1000000,
        0
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"wu".as_ptr(),
        1000000,
        0
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"w".as_ptr(),
        0,
        1
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"w".as_ptr(),
        0,
        2
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"wu".as_ptr(),
        0,
        1
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"wu".as_ptr(),
        0,
        2
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"w".as_ptr(),
        1000000,
        1
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"w".as_ptr(),
        1000000,
        2
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"wu".as_ptr(),
        1000000,
        1
    ));
    run!(test_test_bgzf_c_730_test_tell_seek_getc(
        &mut f,
        c"wu".as_ptr(),
        1000000,
        2
    ));
    run!(test_test_bgzf_c_831_test_tell_read(&mut f, c"w".as_ptr()));
    run!(test_test_bgzf_c_831_test_tell_read(&mut f, c"wu".as_ptr()));
    run!(test_test_bgzf_c_881_test_useek_read_small(
        &mut f,
        c"w".as_ptr()
    ));
    run!(test_test_bgzf_c_881_test_useek_read_small(
        &mut f,
        c"wu".as_ptr()
    ));
    run!(test_test_bgzf_c_924_test_bgzf_getline(
        &mut f,
        c"w".as_ptr(),
        0
    ));
    run!(test_test_bgzf_c_924_test_bgzf_getline(
        &mut f,
        c"w".as_ptr(),
        1
    ));
    run!(test_test_bgzf_c_924_test_bgzf_getline(
        &mut f,
        c"w".as_ptr(),
        2
    ));
    run!(test_test_bgzf_c_981_test_bgzf_getline_on_truncated_file(
        &mut f,
        c"w".as_ptr(),
        0
    ));
    run!(test_test_bgzf_c_981_test_bgzf_getline_on_truncated_file(
        &mut f,
        c"w".as_ptr(),
        1
    ));
    run!(test_test_bgzf_c_981_test_bgzf_getline_on_truncated_file(
        &mut f,
        c"w".as_ptr(),
        2
    ));

    retval = libc::EXIT_SUCCESS;
    test_test_bgzf_c_344_cleanup(&mut f, retval);
    retval
}
