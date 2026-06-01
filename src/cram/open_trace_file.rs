// Functions translated from htslib/cram/open_trace_file.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int};

use super::*;

pub unsafe fn cram_open_trace_file_c_90_is_file(fn_: *mut c_char) -> c_int {
    let mut buf = std::mem::MaybeUninit::<libc::stat>::uninit();
    if libc::stat(fn_, buf.as_mut_ptr()) != 0 {
        return 0;
    }
    let buf = buf.assume_init();
    ((buf.st_mode & libc::S_IFMT) == libc::S_IFREG) as c_int
}

pub unsafe fn cram_open_trace_file_c_108_tokenise_search_path(
    searchpath: *const c_char,
) -> *mut c_char {
    let path_sep = if cfg!(windows) { b';' } else { b':' };
    let searchpath = if searchpath.is_null() {
        c"".as_ptr()
    } else {
        searchpath
    };
    let len = libc::strlen(searchpath);
    let newsearch = malloc((len + 5) as u64).cast::<c_char>();
    if newsearch.is_null() {
        return std::ptr::null_mut();
    }

    let mut i = 0usize;
    let mut j = 0usize;
    while i < len {
        let cur = *searchpath.add(i) as u8;
        if i < len - 1 && cur == b':' && *searchpath.add(i + 1) as u8 == b':' {
            *newsearch.add(j) = b':' as c_char;
            j += 1;
            i += 2;
            continue;
        }

        if path_sep == b':'
            && (i == 0 || *searchpath.add(i - 1) as u8 == b':')
            && (libc::strncmp(searchpath.add(i), c"http:".as_ptr(), 5) == 0
                || libc::strncmp(searchpath.add(i), c"https:".as_ptr(), 6) == 0
                || libc::strncmp(searchpath.add(i), c"ftp:".as_ptr(), 4) == 0
                || libc::strncmp(searchpath.add(i), c"|http:".as_ptr(), 6) == 0
                || libc::strncmp(searchpath.add(i), c"|https:".as_ptr(), 7) == 0
                || libc::strncmp(searchpath.add(i), c"|ftp:".as_ptr(), 5) == 0
                || libc::strncmp(searchpath.add(i), c"URL=http:".as_ptr(), 9) == 0
                || libc::strncmp(searchpath.add(i), c"URL=https:".as_ptr(), 10) == 0
                || libc::strncmp(searchpath.add(i), c"URL=ftp:".as_ptr(), 8) == 0)
        {
            loop {
                *newsearch.add(j) = *searchpath.add(i);
                j += 1;
                let was_colon = *searchpath.add(i) as u8 == b':';
                i += 1;
                if i >= len || was_colon {
                    break;
                }
            }
            if *searchpath.add(i) as u8 == b':' {
                i += 1;
            }
            if *searchpath.add(i) as u8 == b'/' {
                *newsearch.add(j) = *searchpath.add(i);
                j += 1;
                i += 1;
            }
            if *searchpath.add(i) as u8 == b'/' {
                *newsearch.add(j) = *searchpath.add(i);
                j += 1;
                i += 1;
            }
            loop {
                *newsearch.add(j) = *searchpath.add(i);
                j += 1;
                i += 1;
                if i >= len || *searchpath.add(i) as u8 == b':' || *searchpath.add(i) as u8 == b'/'
                {
                    break;
                }
            }
            *newsearch.add(j) = *searchpath.add(i);
            j += 1;
            i += 1;
            if *searchpath.add(i) as u8 == b':' {
                i += 1;
            }
        }

        if *searchpath.add(i) as u8 == path_sep {
            if j != 0 && *newsearch.add(j - 1) != 0 {
                *newsearch.add(j) = 0;
                j += 1;
            }
        } else {
            *newsearch.add(j) = *searchpath.add(i);
            j += 1;
        }
        i += 1;
    }

    if j != 0 {
        *newsearch.add(j) = 0;
        j += 1;
    }
    *newsearch.add(j) = b'.' as c_char;
    j += 1;
    *newsearch.add(j) = b'/' as c_char;
    j += 1;
    *newsearch.add(j) = 0;
    j += 1;
    *newsearch.add(j) = 0;

    newsearch
}

pub unsafe fn cram_open_trace_file_c_182_find_file_url(
    file: *const c_char,
    url: *mut c_char,
) -> *mut mFILE {
    let path = cram_open_trace_file_c_230_expand_path(file, url.cast_const(), 1);
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let hf = crate::htslib_rs::hfile::hopen(path, c"r".as_ptr());
    if hf.is_null() {
        free(path.cast());
        return std::ptr::null_mut();
    }

    let mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
    if mf.is_null() {
        crate::htslib_rs::hfile::hclose_abruptly(hf);
        free(path.cast());
        return std::ptr::null_mut();
    }

    let mut buf = [0u8; 8192];
    loop {
        let len = crate::htslib_rs::hfile::hread2(hf, buf.as_mut_ptr().cast(), buf.len(), 0);
        if len <= 0 {
            if crate::htslib_rs::hfile::hclose(hf) < 0 || len < 0 {
                cram_mFILE_c_408_mfdestroy(mf);
                free(path.cast());
                return std::ptr::null_mut();
            }
            break;
        }
        if cram_mFILE_c_527_mfwrite(buf.as_mut_ptr().cast(), len as usize, 1, mf) == 0 {
            crate::htslib_rs::hfile::hclose_abruptly(hf);
            cram_mFILE_c_408_mfdestroy(mf);
            free(path.cast());
            return std::ptr::null_mut();
        }
    }

    free(path.cast());
    cram_mFILE_c_475_mrewind(mf);
    mf
}

pub unsafe fn cram_open_trace_file_c_230_expand_path(
    mut file: *const c_char,
    mut dirname: *const c_char,
    max_s_digits: c_int,
) -> *mut c_char {
    let mut len = libc::strlen(dirname);
    let mut lenf = libc::strlen(file);
    let mut end_dirname = dirname.add(len);
    let path = malloc((len + lenf + 2) as u64).cast::<c_char>();
    if path.is_null() {
        return std::ptr::null_mut();
    }

    while len > 1 && *dirname.add(len - 1) as u8 == b'/' {
        len -= 1;
        end_dirname = end_dirname.sub(1);
    }

    if *file as u8 == b'/' || (len == 1 && *dirname as u8 == b'.') {
        memcpy(path.cast(), file.cast(), (lenf + 1) as u64);
    } else {
        let mut path_end = path;
        loop {
            let cp = libc::strchr(dirname, b'%' as c_int);
            if cp.is_null() {
                break;
            }

            let mut endp: *mut c_char = std::ptr::null_mut();
            let l = libc::strtol(cp.add(1), &mut endp, 10);
            if *endp as u8 != b's' || l < 0 || endp.offset_from(cp) - 1 > max_s_digits as isize {
                let mut e = endp.add(1).cast_const();
                if e > end_dirname {
                    e = end_dirname;
                }
                let n = e.offset_from(dirname) as usize;
                memcpy(path_end.cast(), dirname.cast(), n as u64);
                path_end = path_end.add(n);
                dirname = e;
                continue;
            }

            let n = cp.cast_const().offset_from(dirname) as usize;
            memcpy(path_end.cast(), dirname.cast(), n as u64);
            path_end = path_end.add(n);

            let to_copy = if l > 0 {
                std::cmp::min(lenf, l as usize)
            } else {
                lenf
            };
            memcpy(path_end.cast(), file.cast(), to_copy as u64);
            path_end = path_end.add(to_copy);
            file = file.add(to_copy);
            lenf -= to_copy;

            dirname = endp.add(1);
        }

        if dirname < end_dirname {
            let n = end_dirname.offset_from(dirname) as usize;
            memcpy(path_end.cast(), dirname.cast(), n as u64);
            path_end = path_end.add(n);
        }

        if *file != 0 {
            if path_end > path && *path_end.sub(1) as u8 != b'/' {
                *path_end = b'/' as c_char;
                path_end = path_end.add(1);
            }
            memcpy(path_end.cast(), file.cast(), lenf as u64);
            path_end = path_end.add(lenf);
        }
        *path_end = 0;
    }

    path
}

pub unsafe fn cram_open_trace_file_c_433_find_path(
    file: *const c_char,
    mut path: *const c_char,
) -> *mut c_char {
    if path.is_null() {
        path = libc::getenv(c"RAWDATA".as_ptr());
    }
    let newsearch = cram_open_trace_file_c_108_tokenise_search_path(path);
    if newsearch.is_null() {
        return std::ptr::null_mut();
    }

    let mut ele = newsearch;
    while *ele != 0 {
        let ele2 = if *ele as u8 == b'|' { ele.add(1) } else { ele };

        if libc::strncmp(ele2, c"URL=".as_ptr(), 4) != 0
            && libc::strncmp(ele2, c"http:".as_ptr(), 5) != 0
            && libc::strncmp(ele2, c"https:".as_ptr(), 6) != 0
            && libc::strncmp(ele2, c"ftp:".as_ptr(), 4) != 0
        {
            let outpath = cram_open_trace_file_c_230_expand_path(file, ele2, c_int::MAX);
            if cram_open_trace_file_c_90_is_file(outpath) != 0 {
                free(newsearch.cast());
                return outpath;
            }
            free(outpath.cast());
        }

        ele = ele.add(libc::strlen(ele) + 1);
    }

    free(newsearch.cast());
    std::ptr::null_mut()
}

pub unsafe fn cram_open_trace_file_c_314_find_file_dir(
    file: *const c_char,
    dirname: *mut c_char,
) -> *mut mFILE {
    let path = cram_open_trace_file_c_230_expand_path(file, dirname.cast_const(), c_int::MAX);
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let mf = if cram_open_trace_file_c_90_is_file(path) != 0 {
        cram_mFILE_c_347_mfopen(path, c"rbm".as_ptr())
    } else {
        std::ptr::null_mut()
    };
    free(path.cast());
    mf
}

pub unsafe fn cram_open_trace_file_c_352_open_path_mfile(
    file: *const c_char,
    mut path: *mut c_char,
    relative_to: *mut c_char,
    local: *mut c_int,
) -> *mut mFILE {
    if !local.is_null() {
        *local = 1;
    }

    if path.is_null() {
        path = libc::getenv(c"RAWDATA".as_ptr());
    }
    let newsearch = cram_open_trace_file_c_108_tokenise_search_path(path);
    if newsearch.is_null() {
        return std::ptr::null_mut();
    }

    let mut ele = newsearch;
    while *ele != 0 {
        let ele2 = if *ele as u8 == b'|' { ele.add(1) } else { ele };

        if libc::strncmp(ele2, c"URL=".as_ptr(), 4) == 0 {
            let fp = cram_open_trace_file_c_182_find_file_url(file, ele2.add(4));
            if !fp.is_null() {
                if !local.is_null() {
                    *local = if libc::strncmp(ele2.add(4), c"file:".as_ptr(), 5) == 0 {
                        1
                    } else {
                        0
                    };
                }
                free(newsearch.cast());
                return fp;
            }
        } else if crate::htslib_rs::hfile::hisremote(ele2) != 0 {
            let fp = cram_open_trace_file_c_182_find_file_url(file, ele2);
            if !fp.is_null() {
                if !local.is_null() {
                    *local = 0;
                }
                free(newsearch.cast());
                return fp;
            }
        } else {
            let fp = cram_open_trace_file_c_314_find_file_dir(file, ele2);
            if !fp.is_null() {
                free(newsearch.cast());
                return fp;
            }
        }

        ele = ele.add(libc::strlen(ele) + 1);
    }

    free(newsearch.cast());

    if !relative_to.is_null() {
        let mut relative_path = [0 as c_char; libc::PATH_MAX as usize + 1];
        libc::strcpy(relative_path.as_mut_ptr(), relative_to);
        let cp = libc::strrchr(relative_path.as_mut_ptr(), b'/' as c_int);
        if !cp.is_null() {
            *cp = 0;
        }
        let fp = cram_open_trace_file_c_314_find_file_dir(file, relative_path.as_mut_ptr());
        if !fp.is_null() {
            return fp;
        }
    }

    std::ptr::null_mut()
}
