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

use std::ffi::OsString;
use std::ptr::NonNull;

const PLUGINPATH: &[u8] = b"";
const PLUGIN_EXT: &[u8] = b".so";
const HTS_PATH_SEPARATOR: u8 = b':';
const DISABLED_PLUGINS_MESSAGE: &[u8] = b"external plugins are disabled\0";

pub(crate) struct PluginPathItr<'a> {
    path: Vec<u8>,
    entry: Vec<u8>,
    dirv: Option<NonNull<()>>,
    pathdir: Option<NonNull<u8>>,
    prefix: &'a [u8],
    suffix: &'a [u8],
    entry_dir_len: usize,
}

impl Default for PluginPathItr<'_> {
    fn default() -> Self {
        Self {
            path: Vec::new(),
            entry: Vec::new(),
            dirv: None,
            pathdir: None,
            prefix: b"",
            suffix: PLUGIN_EXT,
            entry_dir_len: 0,
        }
    }
}

#[cfg(unix)]
fn os_string_into_bytes(value: OsString) -> Vec<u8> {
    use std::os::unix::ffi::OsStringExt;

    value.into_vec()
}

#[cfg(not(unix))]
fn os_string_into_bytes(value: OsString) -> Vec<u8> {
    value.to_string_lossy().into_owned().into_bytes()
}

fn env_path_bytes(name: &str) -> Option<Vec<u8>> {
    std::env::var_os(name).map(os_string_into_bytes)
}

// original: open_nextdir (htslib/plugin.c:42)
fn plugin_c_42_open_nextdir(itr: &mut PluginPathItr<'_>) -> Option<NonNull<()>> {
    let _ = itr;
    None
}

// original: hts_path_itr_setup (htslib/plugin.c:69)
pub(crate) fn plugin_c_69_hts_path_itr_setup<'a>(
    itr: &mut PluginPathItr<'a>,
    path: Option<&[u8]>,
    builtin_path: Option<&[u8]>,
    prefix: &'a [u8],
    _prefix_len: usize,
    suffix: Option<&'a [u8]>,
    _suffix_len: usize,
) {
    itr.prefix = prefix;
    itr.suffix = suffix.unwrap_or(PLUGIN_EXT);
    itr.path.clear();
    itr.entry.clear();
    itr.entry_dir_len = 0;

    let env_path = path.is_none().then(|| env_path_bytes("HTS_PATH")).flatten();
    let path = path.unwrap_or(env_path.as_deref().unwrap_or(b""));
    let builtin_path = builtin_path.unwrap_or(PLUGINPATH);

    for component in path.split(|byte| *byte == HTS_PATH_SEPARATOR) {
        if component.is_empty() {
            itr.path.extend_from_slice(builtin_path);
        } else {
            itr.path.extend_from_slice(component);
        }
        itr.path.push(HTS_PATH_SEPARATOR);
    }

    // Note that ':' now terminates entries rather than separates them
    itr.pathdir = NonNull::new(itr.path.as_mut_ptr());
    itr.dirv = plugin_c_42_open_nextdir(itr);
}

// original: hts_path_itr_next (htslib/plugin.c:104)
pub(crate) fn plugin_c_104_hts_path_itr_next(itr: &mut PluginPathItr<'_>) -> *const i8 {
    itr.pathdir = None;
    itr.path.clear();
    itr.entry.clear();
    std::ptr::null()
}

// original: load_plugin (htslib/plugin.c:135)
pub fn plugin_c_135_load_plugin(
    pluginp: Option<&mut Option<NonNull<()>>>,
    _filename: &[u8],
    _symbol: &[u8],
) -> Option<NonNull<()>> {
    if let Some(pluginp) = pluginp {
        *pluginp = None;
    }
    None
}

// original: plugin_sym (htslib/plugin.c:172)
pub fn plugin_c_172_plugin_sym(
    _plugin: Option<NonNull<()>>,
    _name: &[u8],
    errmsg: Option<&mut *const i8>,
) -> Option<NonNull<()>> {
    if let Some(errmsg) = errmsg {
        *errmsg = DISABLED_PLUGINS_MESSAGE.as_ptr().cast();
    }
    None
}

// original: plugin_func (htslib/plugin.c:179)
pub fn plugin_c_179_plugin_func(
    plugin: Option<NonNull<()>>,
    name: &[u8],
    errmsg: Option<&mut *const i8>,
) -> Option<NonNull<()>> {
    plugin_c_172_plugin_sym(plugin, name, errmsg)
}

// original: close_plugin (htslib/plugin.c:186)
pub fn plugin_c_186_close_plugin(_plugin: Option<NonNull<()>>) {
    // Runtime dynamic plugins are disabled; supported handlers are statically linked.
}

// original: hts_plugin_path (htslib/plugin.c:195)
pub fn plugin_c_195_hts_plugin_path() -> *const i8 {
    // ENABLE_PLUGINS is not defined for this translation build.
    std::ptr::null()
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn path_bytes(path: &Path) -> &[u8] {
        path.as_os_str().as_bytes()
    }

    #[test]
    fn hts_path_iterator_filters_hfile_shared_objects_across_path_entries() {
        let first = temp_dir("first");
        let second = temp_dir("second");
        std::fs::write(first.join("hfile_alpha.so"), b"").unwrap();
        std::fs::write(first.join("not_hfile.so"), b"").unwrap();
        std::fs::write(second.join("hfile_beta.so"), b"").unwrap();
        std::fs::write(second.join("hfile_gamma.dylib"), b"").unwrap();

        let mut path = Vec::new();
        path.push(b':');
        path.extend_from_slice(path_bytes(&first));
        path.push(b':');
        path.extend_from_slice(path_bytes(&second));

        let mut itr = PluginPathItr::default();
        plugin_c_69_hts_path_itr_setup(
            &mut itr,
            Some(&path),
            Some(path_bytes(&first)),
            b"hfile_",
            6,
            None,
            0,
        );

        assert!(plugin_c_104_hts_path_itr_next(&mut itr).is_null());

        let _ = std::fs::remove_dir_all(first);
        let _ = std::fs::remove_dir_all(second);
    }
}
