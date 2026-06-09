// Functions translated from htslib/cram/open_trace_file.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int};
use std::ptr::NonNull;

use super::*;

type MfilePtr = NonNull<mFILE>;
type HfilePtr = NonNull<crate::htslib_rs::hts::hFILE>;

unsafe fn c_ptr_bytes<'a>(ptr: *const c_char) -> Option<&'a [u8]> {
    NonNull::new(ptr.cast_mut()).map(|ptr| {
        let len = libc::strlen(ptr.as_ptr());
        std::slice::from_raw_parts(ptr.as_ptr().cast::<u8>(), len)
    })
}

unsafe fn getenv_bytes(name: &[u8]) -> Option<&'static [u8]> {
    c_ptr_bytes(libc::getenv(name.as_ptr().cast()))
}

fn nul_terminated(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() + 1);
    out.extend_from_slice(bytes);
    out.push(0);
    out
}

fn open_trace_is_file(fn_: &[u8]) -> c_int {
    let fn_ = nul_terminated(fn_);
    let mut buf = std::mem::MaybeUninit::<libc::stat>::uninit();
    if unsafe { libc::stat(fn_.as_ptr().cast(), buf.as_mut_ptr()) } != 0 {
        return 0;
    }
    let buf = unsafe { buf.assume_init() };
    crate::htslib_rs::c_compat::stat_mode_matches(buf.st_mode, libc::S_IFMT, libc::S_IFREG) as c_int
}

pub unsafe fn cram_open_trace_file_c_90_is_file(fn_: *mut c_char) -> c_int {
    c_ptr_bytes(fn_).map_or(0, open_trace_is_file)
}

fn alloc_c_bytes(bytes: &[u8]) -> *mut c_char {
    unsafe {
        let out = malloc(bytes.len() as u64).cast::<c_char>();
        if !out.is_null() {
            std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), out, bytes.len());
        }
        out
    }
}

fn open_trace_tokenise_search_path(searchpath: Option<&[u8]>) -> Vec<u8> {
    let path_sep = if cfg!(windows) { b';' } else { b':' };
    let searchpath = searchpath.unwrap_or(b"");
    let len = searchpath.len();
    let mut newsearch = Vec::with_capacity(len + 5);
    let mut i = 0usize;
    while i < len {
        let cur = searchpath[i];
        if i + 1 < len && cur == b':' && searchpath[i + 1] == b':' {
            newsearch.push(b':');
            i += 2;
            continue;
        }

        if path_sep == b':'
            && (i == 0 || searchpath[i - 1] == b':')
            && (searchpath[i..].starts_with(b"http:")
                || searchpath[i..].starts_with(b"https:")
                || searchpath[i..].starts_with(b"ftp:")
                || searchpath[i..].starts_with(b"|http:")
                || searchpath[i..].starts_with(b"|https:")
                || searchpath[i..].starts_with(b"|ftp:")
                || searchpath[i..].starts_with(b"URL=http:")
                || searchpath[i..].starts_with(b"URL=https:")
                || searchpath[i..].starts_with(b"URL=ftp:"))
        {
            while i < len {
                let was_colon = searchpath[i] == b':';
                newsearch.push(searchpath[i]);
                i += 1;
                if was_colon {
                    break;
                }
            }
            if i < len && searchpath[i] == b':' {
                i += 1;
            }
            if i < len && searchpath[i] == b'/' {
                newsearch.push(searchpath[i]);
                i += 1;
            }
            if i < len && searchpath[i] == b'/' {
                newsearch.push(searchpath[i]);
                i += 1;
            }
            while i < len {
                newsearch.push(searchpath[i]);
                i += 1;
                if i >= len || searchpath[i] == b':' || searchpath[i] == b'/' {
                    break;
                }
            }
            if i < len {
                newsearch.push(searchpath[i]);
                i += 1;
            }
            if i < len && searchpath[i] == b':' {
                i += 1;
            }
        }

        if i < len {
            if searchpath[i] == path_sep {
                if !newsearch.is_empty() && *newsearch.last().unwrap() != 0 {
                    newsearch.push(0);
                }
            } else {
                newsearch.push(searchpath[i]);
            }
            i += 1;
        }
    }

    if !newsearch.is_empty() {
        newsearch.push(0);
    }
    newsearch.extend_from_slice(b"./\0\0");

    newsearch
}

pub unsafe fn cram_open_trace_file_c_108_tokenise_search_path(
    searchpath: *const c_char,
) -> *mut c_char {
    alloc_c_bytes(&open_trace_tokenise_search_path(c_ptr_bytes(searchpath)))
}

struct HfileHandle(HfilePtr);

impl HfileHandle {
    unsafe fn open_read(path: &[u8]) -> Option<Self> {
        let path = nul_terminated(path);
        NonNull::new(crate::htslib_rs::hfile::hopen(
            path.as_ptr().cast(),
            b"r\0".as_ptr().cast(),
        ))
        .map(Self)
    }

    unsafe fn read(&mut self, buf: &mut [u8]) -> libc::ssize_t {
        crate::htslib_rs::hfile::htslib_hfile_h_247_hread_ref(self.0.as_mut(), buf)
    }

    unsafe fn close(self) -> c_int {
        let ptr = self.0.as_ptr();
        std::mem::forget(self);
        crate::htslib_rs::hfile::hclose(ptr)
    }

    unsafe fn close_abruptly(self) {
        let ptr = self.0.as_ptr();
        std::mem::forget(self);
        crate::htslib_rs::hfile::hclose_abruptly(ptr);
    }
}

impl Drop for HfileHandle {
    fn drop(&mut self) {
        unsafe {
            crate::htslib_rs::hfile::hclose_abruptly(self.0.as_ptr());
        }
    }
}

struct OwnedMfile(MfilePtr);

impl OwnedMfile {
    unsafe fn create_empty() -> Option<Self> {
        NonNull::new(cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0)).map(Self)
    }

    unsafe fn write_all(&mut self, bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return true;
        }
        cram_mFILE_c_527_mfwrite(
            bytes.as_ptr().cast_mut().cast(),
            bytes.len(),
            1,
            self.0.as_ptr(),
        ) != 0
    }

    unsafe fn rewind(&mut self) {
        cram_mFILE_c_475_mrewind(self.0.as_ptr());
    }

    fn into_ptr(self) -> MfilePtr {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }
}

impl Drop for OwnedMfile {
    fn drop(&mut self) {
        unsafe {
            cram_mFILE_c_408_mfdestroy(self.0.as_ptr());
        }
    }
}

unsafe fn open_trace_find_file_url(file: &[u8], url: &[u8]) -> Option<MfilePtr> {
    let path = open_trace_expand_path(file, url, 1);
    let mut hf = HfileHandle::open_read(&path)?;
    let mut mf = OwnedMfile::create_empty()?;

    let mut buf = [0u8; 8192];
    loop {
        let len = hf.read(&mut buf);
        if len <= 0 {
            if hf.close() < 0 || len < 0 {
                return None;
            }
            break;
        }
        if !mf.write_all(&buf[..len as usize]) {
            hf.close_abruptly();
            return None;
        }
    }

    mf.rewind();
    Some(mf.into_ptr())
}

pub unsafe fn cram_open_trace_file_c_182_find_file_url(
    file: *const c_char,
    url: *mut c_char,
) -> *mut mFILE {
    let (Some(file), Some(url)) = (c_ptr_bytes(file), c_ptr_bytes(url)) else {
        return std::ptr::null_mut();
    };
    open_trace_find_file_url(file, url).map_or(std::ptr::null_mut(), MfilePtr::as_ptr)
}

fn open_trace_expand_path(file: &[u8], dirname: &[u8], max_s_digits: c_int) -> Vec<u8> {
    let mut file_remaining = file;
    let mut dirname = dirname;
    while dirname.len() > 1 && dirname[dirname.len() - 1] == b'/' {
        dirname = &dirname[..dirname.len() - 1];
    }

    if file_remaining.first() == Some(&b'/') || dirname == b"." {
        file.to_vec()
    } else {
        let mut path = Vec::with_capacity(dirname.len() + file_remaining.len() + 2);
        while let Some(cp) = dirname.iter().position(|&ch| ch == b'%') {
            let digit_start = cp + 1;
            let mut digit_end = digit_start;
            while digit_end < dirname.len() && dirname[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            let digits = &dirname[digit_start..digit_end];
            let l = std::str::from_utf8(digits)
                .ok()
                .and_then(|digits| {
                    if digits.is_empty() {
                        Some(0usize)
                    } else {
                        digits.parse::<usize>().ok()
                    }
                })
                .unwrap_or(usize::MAX);
            let valid = digit_end < dirname.len()
                && dirname[digit_end] == b's'
                && l != usize::MAX
                && usize::try_from(max_s_digits)
                    .is_ok_and(|max_s_digits| digits.len() <= max_s_digits);
            if !valid {
                let end = std::cmp::min(digit_end + 1, dirname.len());
                path.extend_from_slice(&dirname[..end]);
                dirname = &dirname[end..];
                continue;
            }

            path.extend_from_slice(&dirname[..cp]);
            let to_copy = if l == 0 {
                file_remaining.len()
            } else {
                std::cmp::min(file_remaining.len(), l)
            };
            path.extend_from_slice(&file_remaining[..to_copy]);
            file_remaining = &file_remaining[to_copy..];
            dirname = &dirname[digit_end + 1..];
        }

        path.extend_from_slice(dirname);

        if !file_remaining.is_empty() {
            if !path.is_empty() && *path.last().unwrap() != b'/' {
                path.push(b'/');
            }
            path.extend_from_slice(file_remaining);
        }
        path
    }
}

pub unsafe fn cram_open_trace_file_c_230_expand_path(
    file: *const c_char,
    dirname: *const c_char,
    max_s_digits: c_int,
) -> *mut c_char {
    let (Some(file), Some(dirname)) = (c_ptr_bytes(file), c_ptr_bytes(dirname)) else {
        return std::ptr::null_mut();
    };
    let path = open_trace_expand_path(file, dirname, max_s_digits);
    alloc_c_bytes(&nul_terminated(&path))
}

fn open_trace_find_path(file: &[u8], path: Option<&[u8]>) -> Option<Vec<u8>> {
    let newsearch = open_trace_tokenise_search_path(path);

    for ele in newsearch
        .split(|&ch| ch == 0)
        .take_while(|ele| !ele.is_empty())
    {
        let ele2 = ele.strip_prefix(b"|").unwrap_or(ele);

        if !ele2.starts_with(b"URL=")
            && !ele2.starts_with(b"http:")
            && !ele2.starts_with(b"https:")
            && !ele2.starts_with(b"ftp:")
        {
            let outpath = open_trace_expand_path(file, ele2, c_int::MAX);
            if open_trace_is_file(&outpath) != 0 {
                return Some(outpath);
            }
        }
    }

    None
}

pub unsafe fn cram_open_trace_file_c_433_find_path(
    file: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    let Some(file) = c_ptr_bytes(file) else {
        return std::ptr::null_mut();
    };
    let path = if path.is_null() {
        getenv_bytes(b"RAWDATA\0")
    } else {
        c_ptr_bytes(path)
    };
    open_trace_find_path(file, path)
        .map(|path| alloc_c_bytes(&nul_terminated(&path)))
        .unwrap_or(std::ptr::null_mut())
}

unsafe fn open_trace_find_file_dir(file: &[u8], dirname: &[u8]) -> Option<MfilePtr> {
    let path = open_trace_expand_path(file, dirname, c_int::MAX);
    if open_trace_is_file(&path) != 0 {
        let path = nul_terminated(&path);
        NonNull::new(cram_mFILE_c_347_mfopen(
            path.as_ptr().cast(),
            b"rbm\0".as_ptr().cast(),
        ))
    } else {
        None
    }
}

pub unsafe fn cram_open_trace_file_c_314_find_file_dir(
    file: *const c_char,
    dirname: *mut c_char,
) -> *mut mFILE {
    let (Some(file), Some(dirname)) = (c_ptr_bytes(file), c_ptr_bytes(dirname)) else {
        return std::ptr::null_mut();
    };
    open_trace_find_file_dir(file, dirname).map_or(std::ptr::null_mut(), MfilePtr::as_ptr)
}

struct OpenPathMfile {
    mf: MfilePtr,
    local: bool,
}

unsafe fn open_trace_open_path_mfile(
    file: &[u8],
    path: Option<&[u8]>,
    relative_to: Option<&[u8]>,
) -> Option<OpenPathMfile> {
    let newsearch = open_trace_tokenise_search_path(path);

    for ele in newsearch
        .split(|&ch| ch == 0)
        .take_while(|ele| !ele.is_empty())
    {
        let ele2 = ele.strip_prefix(b"|").unwrap_or(ele);

        if let Some(url) = ele2.strip_prefix(b"URL=") {
            if let Some(mf) = open_trace_find_file_url(file, url) {
                return Some(OpenPathMfile {
                    mf,
                    local: url.starts_with(b"file:"),
                });
            }
        } else if is_remote_path(ele2) {
            if let Some(mf) = open_trace_find_file_url(file, ele2) {
                return Some(OpenPathMfile { mf, local: false });
            }
        } else if let Some(mf) = open_trace_find_file_dir(file, ele2) {
            return Some(OpenPathMfile { mf, local: true });
        }
    }

    if let Some(relative_to) = relative_to {
        let relative_path = relative_to
            .iter()
            .rposition(|&ch| ch == b'/')
            .map_or(relative_to, |slash| &relative_to[..slash]);
        if let Some(mf) = open_trace_find_file_dir(file, relative_path) {
            return Some(OpenPathMfile { mf, local: true });
        }
    }

    None
}

pub unsafe fn cram_open_trace_file_c_352_open_path_mfile(
    file: *const c_char,
    path: *mut c_char,
    relative_to: *mut c_char,
    local: *mut c_int,
) -> *mut mFILE {
    let Some(file) = c_ptr_bytes(file) else {
        return std::ptr::null_mut();
    };
    let mut local = NonNull::new(local);
    if let Some(local) = local.as_mut() {
        *local.as_ptr() = 1;
    }

    let path = if path.is_null() {
        getenv_bytes(b"RAWDATA\0")
    } else {
        c_ptr_bytes(path)
    };
    let relative_to = c_ptr_bytes(relative_to);
    let Some(opened) = open_trace_open_path_mfile(file, path, relative_to) else {
        return std::ptr::null_mut();
    };
    if let Some(local) = local.as_mut() {
        *local.as_ptr() = opened.local as c_int;
    }
    opened.mf.as_ptr()
}

fn is_remote_path(path: &[u8]) -> bool {
    let path = nul_terminated(path);
    unsafe { crate::htslib_rs::hfile::hisremote(path.as_ptr().cast()) != 0 }
}
