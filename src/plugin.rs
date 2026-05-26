/*  plugin.c -- low-level path parsing and plugin functions.

    Copyright (C) 2015-2016, 2020 Genome Research Ltd.

    Author: John Marshall <jm18@sanger.ac.uk>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.  */

use crate::htslib_rs::{
    c_compat::__errno_location,
    hts::{hts_verbose, kputc, kputs, kputsn, kstring_t},
};
use std::ffi::{c_char, c_int, c_void};

const PLUGINPATH: *const c_char = c"".as_ptr();
const PLUGIN_EXT: *const c_char = c".so".as_ptr();
const PLUGIN_EXT_LEN: usize = 3;
const HTS_PATH_SEPARATOR_CHAR: c_int = b':' as c_int;
const HTS_PATH_SEPARATOR_STR: *const c_char = c":".as_ptr();

#[repr(C)]
pub(crate) struct PluginPathItr {
    path: kstring_t,
    entry: kstring_t,
    dirv: *mut c_void,
    pathdir: *const c_char,
    prefix: *const c_char,
    suffix: *const c_char,
    prefix_len: usize,
    suffix_len: usize,
    entry_dir_l: usize,
}

// original: open_nextdir (htslib/plugin.c:42)
unsafe fn plugin_c_42_open_nextdir(itr: *mut PluginPathItr) -> *mut c_void {
    loop {
        let colon = libc::strchr((*itr).pathdir, HTS_PATH_SEPARATOR_CHAR);
        if colon.is_null() {
            return std::ptr::null_mut();
        }

        (*itr).entry.l = 0;
        kputsn(
            (*itr).pathdir,
            colon.offset_from((*itr).pathdir) as usize,
            &mut (*itr).entry,
        );
        (*itr).pathdir = colon.add(1);
        if (*itr).entry.l == 0 {
            continue;
        }

        let dir = libc::opendir((*itr).entry.s);
        if !dir.is_null() {
            if *(*itr).entry.s.add((*itr).entry.l - 1) != b'/' as c_char {
                kputc(b'/' as c_int, &mut (*itr).entry);
            }
            (*itr).entry_dir_l = (*itr).entry.l;
            return dir.cast();
        }

        if hts_verbose >= 4 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"[W::hts_path_itr] can't scan directory \"%s\": %s\n".as_ptr(),
                (*itr).entry.s,
                libc::strerror(*__errno_location()),
            );
        }
    }
}

// original: hts_path_itr_setup (htslib/plugin.c:69)
pub unsafe fn plugin_c_69_hts_path_itr_setup(
    itr: *mut c_void,
    mut path: *const c_char,
    mut builtin_path: *const c_char,
    prefix: *const c_char,
    prefix_len: usize,
    suffix: *const c_char,
    suffix_len: usize,
) {
    let itr = itr.cast::<PluginPathItr>();
    (*itr).prefix = prefix;
    (*itr).prefix_len = prefix_len;

    if !suffix.is_null() {
        (*itr).suffix = suffix;
        (*itr).suffix_len = suffix_len;
    } else {
        (*itr).suffix = PLUGIN_EXT;
        (*itr).suffix_len = PLUGIN_EXT_LEN;
    }

    (*itr).path.l = 0;
    (*itr).path.m = 0;
    (*itr).path.s = std::ptr::null_mut();
    (*itr).entry.l = 0;
    (*itr).entry.m = 0;
    (*itr).entry.s = std::ptr::null_mut();

    if builtin_path.is_null() {
        builtin_path = PLUGINPATH;
    }
    if path.is_null() {
        path = libc::getenv(c"HTS_PATH".as_ptr());
        if path.is_null() {
            path = c"".as_ptr();
        }
    }

    loop {
        let len = libc::strcspn(path, HTS_PATH_SEPARATOR_STR);
        if len == 0 {
            kputs(builtin_path, &mut (*itr).path);
        } else {
            kputsn(path, len, &mut (*itr).path);
        }
        kputc(HTS_PATH_SEPARATOR_CHAR, &mut (*itr).path);

        path = path.add(len);
        if *path == HTS_PATH_SEPARATOR_CHAR as c_char {
            path = path.add(1);
        } else {
            break;
        }
    }

    // Note that ':' now terminates entries rather than separates them
    (*itr).pathdir = (*itr).path.s;
    (*itr).dirv = plugin_c_42_open_nextdir(itr);
}

// original: hts_path_itr_next (htslib/plugin.c:104)
pub unsafe fn plugin_c_104_hts_path_itr_next(itr: *mut c_void) -> *const c_char {
    let itr = itr.cast::<PluginPathItr>();
    while !(*itr).dirv.is_null() {
        loop {
            let e = libc::readdir((*itr).dirv.cast());
            if e.is_null() {
                break;
            }

            let d_name = (*e).d_name.as_ptr();
            let d_name_len = libc::strlen(d_name);
            if libc::strncmp(d_name, (*itr).prefix, (*itr).prefix_len) == 0
                && d_name_len >= (*itr).suffix_len
                && libc::strncmp(
                    d_name.add(d_name_len - (*itr).suffix_len),
                    (*itr).suffix,
                    (*itr).suffix_len,
                ) == 0
            {
                (*itr).entry.l = (*itr).entry_dir_l;
                kputs(d_name, &mut (*itr).entry);
                return (*itr).entry.s;
            }
        }

        libc::closedir((*itr).dirv.cast());
        (*itr).dirv = plugin_c_42_open_nextdir(itr);
    }

    (*itr).pathdir = std::ptr::null();
    libc::free((*itr).path.s.cast());
    (*itr).path.s = std::ptr::null_mut();
    libc::free((*itr).entry.s.cast());
    (*itr).entry.s = std::ptr::null_mut();
    std::ptr::null()
}

// original: load_plugin (htslib/plugin.c:135)
pub unsafe fn plugin_c_135_load_plugin(
    pluginp: *mut *mut c_void,
    filename: *const c_char,
    symbol: *const c_char,
) -> *mut c_void {
    let mut lib = libc::dlopen(filename, libc::RTLD_NOW | libc::RTLD_LOCAL);
    if lib.is_null() {
        if hts_verbose >= 4 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"[W::%s] can't load plugin \"%s\": %s\n".as_ptr(),
                c"load_plugin".as_ptr(),
                filename,
                libc::dlerror(),
            );
        }
        return std::ptr::null_mut();
    }

    let mut sym = libc::dlsym(lib, symbol);
    if sym.is_null() {
        // Reopen the plugin with RTLD_GLOBAL and check for uniquified symbol
        let libg = libc::dlopen(
            filename,
            libc::RTLD_NOLOAD | libc::RTLD_NOW | libc::RTLD_GLOBAL,
        );
        if libg.is_null() {
            if hts_verbose >= 4 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"[W::%s] can't load plugin \"%s\": %s\n".as_ptr(),
                    c"load_plugin".as_ptr(),
                    filename,
                    libc::dlerror(),
                );
            }
            libc::dlclose(lib);
            return std::ptr::null_mut();
        }
        libc::dlclose(lib);
        lib = libg;

        let mut symbolg: kstring_t = std::mem::zeroed();
        kputs(symbol, &mut symbolg);
        kputc(b'_' as c_int, &mut symbolg);
        let slash = libc::strrchr(filename, b'/' as c_int);
        let basename = if slash.is_null() {
            filename
        } else {
            slash.add(1)
        };
        kputsn(
            basename,
            libc::strcspn(basename, c".-+".as_ptr()),
            &mut symbolg,
        );

        sym = libc::dlsym(lib, symbolg.s);
        libc::free(symbolg.s.cast());
        if sym.is_null() {
            if hts_verbose >= 4 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"[W::%s] can't load plugin \"%s\": %s\n".as_ptr(),
                    c"load_plugin".as_ptr(),
                    filename,
                    libc::dlerror(),
                );
            }
            libc::dlclose(lib);
            return std::ptr::null_mut();
        }
    }

    *pluginp = lib;
    sym
}

// original: plugin_sym (htslib/plugin.c:172)
pub unsafe fn plugin_c_172_plugin_sym(
    plugin: *mut c_void,
    name: *const c_char,
    errmsg: *mut *const c_char,
) -> *mut c_void {
    let sym = libc::dlsym(plugin, name);
    if sym.is_null() {
        *errmsg = libc::dlerror();
    }
    sym
}

// original: plugin_func (htslib/plugin.c:179)
pub unsafe fn plugin_c_179_plugin_func(
    plugin: *mut c_void,
    name: *const c_char,
    errmsg: *mut *const c_char,
) -> *mut c_void {
    plugin_c_172_plugin_sym(plugin, name, errmsg)
}

// original: close_plugin (htslib/plugin.c:186)
pub unsafe fn plugin_c_186_close_plugin(plugin: *mut c_void) {
    if libc::dlclose(plugin) != 0 && hts_verbose >= 4 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"[W::%s] dlclose() failed: %s\n".as_ptr(),
            c"close_plugin".as_ptr(),
            libc::dlerror(),
        );
    }
}

// original: hts_plugin_path (htslib/plugin.c:195)
pub unsafe fn plugin_c_195_hts_plugin_path() -> *const c_char {
    // ENABLE_PLUGINS is not defined for this translation build.
    std::ptr::null()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "htslib-rs-plugin-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn c_path(path: &Path) -> CString {
        CString::new(path.as_os_str().as_bytes()).unwrap()
    }

    #[test]
    fn hts_path_iterator_filters_hfile_shared_objects_across_path_entries() {
        let first = temp_dir("first");
        let second = temp_dir("second");
        std::fs::write(first.join("hfile_alpha.so"), b"").unwrap();
        std::fs::write(first.join("not_hfile.so"), b"").unwrap();
        std::fs::write(second.join("hfile_beta.so"), b"").unwrap();
        std::fs::write(second.join("hfile_gamma.dylib"), b"").unwrap();

        let first_c = c_path(&first);
        let second_c = c_path(&second);
        let path = CString::new(format!(
            ":{}:{}",
            first_c.to_str().unwrap(),
            second_c.to_str().unwrap()
        ))
        .unwrap();
        let builtin = c_path(&first);

        unsafe {
            let mut itr: PluginPathItr = std::mem::zeroed();
            plugin_c_69_hts_path_itr_setup(
                (&mut itr as *mut PluginPathItr).cast(),
                path.as_ptr(),
                builtin.as_ptr(),
                c"hfile_".as_ptr(),
                6,
                std::ptr::null(),
                0,
            );

            let mut found = Vec::new();
            loop {
                let next = plugin_c_104_hts_path_itr_next((&mut itr as *mut PluginPathItr).cast());
                if next.is_null() {
                    break;
                }
                found.push(
                    CStr::from_ptr(next)
                        .to_string_lossy()
                        .rsplit('/')
                        .next()
                        .unwrap()
                        .to_owned(),
                );
            }
            found.sort();
            assert_eq!(found, ["hfile_alpha.so", "hfile_alpha.so", "hfile_beta.so"]);
        }

        let _ = std::fs::remove_dir_all(first);
        let _ = std::fs::remove_dir_all(second);
    }
}
