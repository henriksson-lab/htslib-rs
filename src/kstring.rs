use crate::htslib_rs::hts::{
    kgets_func, kgets_func2, kputc, kputsn, ks_free, ks_resize, ks_tokaux_t, kstring_t,
};
use std::ffi::{c_char, c_int, c_void, CStr};

// original: kvsprintf (htslib/kstring.c:142)
pub unsafe fn kvsprintf(
    s: &mut kstring_t,
    fmt: &[u8],
    ap: &mut crate::htslib_rs::c_compat::__va_list_tag,
) -> c_int {
    let mut args = std::mem::MaybeUninit::<crate::htslib_rs::c_compat::__va_list_tag>::uninit();
    std::ptr::copy_nonoverlapping(ap as *mut _, args.as_mut_ptr(), 1);
    let mut args = args.assume_init();

    let mut out = Vec::new();
    let mut i = 0usize;
    while i < fmt.len() {
        if fmt[i] != b'%' {
            out.push(fmt[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= fmt.len() {
            return -1;
        }
        match fmt[i] {
            b'%' => out.push(b'%'),
            b'd' => out.extend_from_slice(kstring_va_arg_int(&mut args).to_string().as_bytes()),
            b's' => {
                let p = kstring_va_arg_word(&mut args) as *const c_char;
                if p.is_null() {
                    return -1;
                }
                out.extend_from_slice(CStr::from_ptr(p).to_bytes());
            }
            b'g' => {
                let d = kstring_va_arg_f64(&mut args);
                let Some(tmp) = kputd_bytes(d) else {
                    return -1;
                };
                out.extend_from_slice(&tmp);
            }
            _ => return -1,
        }
        i += 1;
    }

    if out.len() > c_int::MAX as usize {
        return -1;
    }
    if kputsn(&out, out.len(), s) < 0 {
        -1
    } else {
        out.len() as c_int
    }
}

unsafe fn kstring_va_arg_word(args: &mut crate::htslib_rs::c_compat::__va_list_tag) -> usize {
    if args.gp_offset <= 40 {
        let p = args.reg_save_area.cast::<u8>().add(args.gp_offset as usize);
        args.gp_offset += 8;
        std::ptr::read_unaligned(p.cast::<usize>())
    } else {
        let p = args.overflow_arg_area.cast::<u8>();
        args.overflow_arg_area = p.add(8).cast();
        std::ptr::read_unaligned(p.cast::<usize>())
    }
}

unsafe fn kstring_va_arg_int(args: &mut crate::htslib_rs::c_compat::__va_list_tag) -> c_int {
    kstring_va_arg_word(args) as u32 as c_int
}

unsafe fn kstring_va_arg_f64(args: &mut crate::htslib_rs::c_compat::__va_list_tag) -> f64 {
    if args.fp_offset <= 160 {
        let p = args.reg_save_area.cast::<u8>().add(args.fp_offset as usize);
        args.fp_offset += 16;
        std::ptr::read_unaligned(p.cast::<f64>())
    } else {
        let p = args.overflow_arg_area.cast::<u8>();
        args.overflow_arg_area = p.add(8).cast();
        std::ptr::read_unaligned(p.cast::<f64>())
    }
}

pub enum KsPrintfArg {
    Int(c_int),
    Str(*const c_char),
}

// Stable Rust cannot define C-variadic functions.  This helper keeps the
// translated local callers usable without relying on nightly-only syntax.
pub unsafe fn ksprintf(
    s: &mut kstring_t,
    fmt: &[u8],
    args: &[KsPrintfArg],
) -> c_int {
    let mut out = Vec::new();
    let mut arg_i = 0usize;
    let mut i = 0usize;
    while i < fmt.len() {
        if fmt[i] != b'%' {
            out.push(fmt[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= fmt.len() {
            return -1;
        }
        match fmt[i] {
            b'%' => out.push(b'%'),
            b'd' => {
                let Some(KsPrintfArg::Int(v)) = args.get(arg_i) else {
                    return -1;
                };
                out.extend_from_slice(v.to_string().as_bytes());
                arg_i += 1;
            }
            b's' => {
                let Some(KsPrintfArg::Str(v)) = args.get(arg_i) else {
                    return -1;
                };
                if v.is_null() {
                    return -1;
                }
                out.extend_from_slice(CStr::from_ptr(*v).to_bytes());
                arg_i += 1;
            }
            _ => return -1,
        }
        i += 1;
    }

    if out.len() > c_int::MAX as usize {
        return -1;
    }
    kputsn(&out, out.len(), s)
}

// original: main (htslib/kstring.c:531)
pub unsafe fn kstring_c_531_main() -> c_int {
    let mut s = kstring_t::default();
    let mut aux: ks_tokaux_t = std::mem::zeroed();

    ksprintf(&mut s, b" abcdefg:    %d ", &[KsPrintfArg::Int(100)]);
    println!("'{}'", String::from_utf8_lossy(&s.data));

    let fields = ksplit_vec(&mut s, 0).unwrap_or_default();
    for (i, &offset) in fields.iter().enumerate() {
        // ksplit writes NUL separators into the buffer; read up to the next one.
        let start = offset as usize;
        let end = s.data[start..]
            .iter()
            .position(|&b| b == 0)
            .map(|p| start + p)
            .unwrap_or(s.data.len());
        println!(
            "field[{}] = '{}'",
            i,
            String::from_utf8_lossy(&s.data[start..end])
        );
    }

    s.data.clear();
    let mut p = kstrtok(Some(b"ab:cde:fg/hij::k"), Some(b":/"), &mut aux);
    while !p.is_null() {
        let len = aux.p.offset_from(p) as usize;
        kputsn(std::slice::from_raw_parts(p.cast::<u8>(), len), len, &mut s);
        kputc(b'\n' as c_int, &mut s);
        p = kstrtok(None, None, &mut aux);
    }
    print!("{}", String::from_utf8_lossy(&s.data));

    ks_free(&mut s);

    {
        let haystack = b"abcdefgcdgcagtcakcdcd";
        let pat = b"cd";
        let mut from = 0usize;
        let mut prep = None;
        while let Some(rel) = kstrstr_boxed_prep(&haystack[from..], pat, Some(&mut prep)) {
            let at = from + rel;
            println!("match: {}", String::from_utf8_lossy(&haystack[at..]));
            from = at + prep.as_deref().unwrap()[0] as usize;
        }
    }

    0
}

// ----------------------------------------------------------------------
// Functions translated from htslib/kstring.c (moved from src/hts.rs).
// ----------------------------------------------------------------------

pub fn ksplit_core(buf: &mut [u8], delimiter: c_int, write_nuls: bool) -> Option<Vec<c_int>> {
    let mut offsets = Vec::new();
    let mut last_char = 0u8 as c_int;
    let mut last_start = 0;
    for (i, ch) in buf.iter_mut().enumerate() {
        let signed_ch = *ch as i8 as c_int;
        let unsigned_ch = *ch as c_int;
        // isspace(c) for the C locale: ' ', '\t', '\n', '\v', '\f', '\r'.
        let is_space = |c: c_int| matches!(c, 0x20 | 0x09 | 0x0a | 0x0b | 0x0c | 0x0d);
        // isgraph(c) for the C locale: any printable character except space.
        let is_graph = |c: c_int| (0x21..=0x7e).contains(&c);
        if delimiter == 0 {
            if is_space(unsigned_ch) || *ch == 0 {
                if is_graph(last_char) {
                    offsets.try_reserve(1).ok()?;
                    if write_nuls {
                        *ch = 0;
                        offsets.push(last_start);
                    } else {
                        offsets.push(0);
                    }
                }
            } else if is_space(last_char) || last_char == 0 {
                last_start = i as c_int;
            }
        } else if signed_ch == delimiter || *ch == 0 {
            if last_char != 0 && last_char != delimiter {
                offsets.try_reserve(1).ok()?;
                if write_nuls {
                    *ch = 0;
                    offsets.push(last_start);
                } else {
                    offsets.push(0);
                }
            }
        } else if last_char == delimiter || last_char == 0 {
            last_start = i as c_int;
        }
        last_char = unsigned_ch;
    }

    Some(offsets)
}

pub unsafe fn ksplit(s: &mut kstring_t, delimiter: c_int, n: &mut c_int) -> *mut c_int {
    if s.data.is_empty() {
        *n = 0;
        return std::ptr::null_mut();
    }

    let Some(fields) = ksplit_vec(s, delimiter) else {
        *n = 0;
        return std::ptr::null_mut();
    };
    if fields.len() > c_int::MAX as usize {
        *n = 0;
        return std::ptr::null_mut();
    }
    let count = fields.len() as c_int;
    let mut max = 0;
    let Some(offsets) = copy_offsets_to_libc_buffer(fields, &mut max, std::ptr::null_mut()) else {
        *n = 0;
        return std::ptr::null_mut();
    };
    *n = count;
    offsets
}

pub unsafe fn ksplit_vec(s: &mut kstring_t, delimiter: c_int) -> Option<Vec<c_int>> {
    if s.data.is_empty() {
        return None;
    }

    // The split scans one past the content (where the old NUL terminator lived)
    // to flush the final field; append a temporary 0 then drop it again so the
    // in-place NUL separators it writes persist in the owned buffer.
    s.data.push(0);
    let result = ksplit_core(&mut s.data, delimiter, true);
    s.data.pop();
    result
}

unsafe fn copy_offsets_to_libc_buffer(
    fields: Vec<c_int>,
    max: &mut c_int,
    offsets: *mut c_int,
) -> Option<*mut c_int> {
    let needed = fields.len();
    if needed == 0 {
        return Some(offsets);
    }
    let current = (*max).max(0) as usize;
    let target = if current >= needed {
        current
    } else {
        let mut target = current.max(2);
        while target < needed {
            target = target.checked_mul(2)?;
        }
        target
    };
    if target > c_int::MAX as usize {
        return None;
    }
    let bytes = target.checked_mul(std::mem::size_of::<c_int>())?;
    let out = crate::htslib_rs::c_compat::realloc(offsets.cast(), bytes as u64).cast::<c_int>();
    if out.is_null() {
        return None;
    }
    std::ptr::copy_nonoverlapping(fields.as_ptr(), out, needed);
    *max = target as c_int;
    Some(out)
}

pub unsafe fn kgetline(s: &mut kstring_t, fgets_fn: kgets_func, fp: *mut c_void) -> c_int {
    let l0 = s.data.len();
    while s.data.len() == l0 || s.data[s.data.len() - 1] != b'\n' {
        // Keep at least 200 bytes of spare capacity (plus room for the NUL the
        // C fgets boundary writes), then let fgets fill the raw tail in place.
        if s.data.capacity() - s.data.len() < 200 {
            ks_resize(s, s.data.capacity() + 200);
        }
        let len = s.data.len();
        let avail = s.data.capacity() - len;
        let ret = fgets_fn.unwrap_unchecked()(s.data.as_mut_ptr().add(len).cast(), avail as c_int, fp);
        if ret.is_null() {
            break;
        }
        // fgets wrote a NUL-terminated string into the spare capacity; adopt it.
        let written = CStr::from_ptr(s.data.as_ptr().add(len).cast()).to_bytes().len();
        s.data.set_len(len + written);
    }

    if s.data.len() == l0 {
        return -1;
    }

    if s.data.len() > l0 && s.data[s.data.len() - 1] == b'\n' {
        s.data.pop();
        if s.data.len() > l0 && s.data[s.data.len() - 1] == b'\r' {
            s.data.pop();
        }
    }
    0
}

pub unsafe extern "C" fn fgets_wrapper(
    buffer: *mut c_char,
    size: c_int,
    stream: *mut c_void,
) -> *mut c_char {
    libc::fgets(buffer, size, stream.cast::<libc::FILE>())
}

pub unsafe fn kfgetline(s: *mut kstring_t, fp: *mut libc::FILE) -> c_int {
    if s.is_null() || fp.is_null() {
        return -1;
    }
    kgetline(&mut *s, Some(fgets_wrapper), fp.cast())
}

pub unsafe fn kgetline2(s: &mut kstring_t, fgets_fn: kgets_func2, fp: *mut c_void) -> c_int {
    let l0 = s.data.len();
    while s.data.len() == l0 || s.data[s.data.len() - 1] != b'\n' {
        if s.data.capacity() - s.data.len() < 200 {
            ks_resize(s, s.data.capacity() + 200);
        }
        let len = s.data.len();
        let avail = s.data.capacity() - len;
        let written = fgets_fn.unwrap_unchecked()(s.data.as_mut_ptr().add(len).cast(), avail, fp);
        if written <= 0 {
            break;
        }
        s.data.set_len(len + written as usize);
    }

    if s.data.len() == l0 {
        return -1;
    }

    if s.data.len() > l0 && s.data[s.data.len() - 1] == b'\n' {
        s.data.pop();
        if s.data.len() > l0 && s.data[s.data.len() - 1] == b'\r' {
            s.data.pop();
        }
    }
    0
}

pub unsafe fn kstrtok(
    str_: Option<&[u8]>,
    sep_in: Option<&[u8]>,
    aux: &mut ks_tokaux_t,
) -> *mut c_char {
    if let Some(sep) = sep_in {
        if str_.is_none() && aux.finished != 0 {
            return std::ptr::null_mut();
        }
        aux.finished = 0;
        if sep.len() > 1 {
            aux.sep = -1;
            aux.tab = [0; 4];
            for &ch in sep {
                aux.tab[(ch >> 6) as usize] |= 1u64 << (ch & 0x3f);
            }
        } else {
            aux.sep = sep.first().copied().unwrap_or(0) as c_int;
        }
    }
    if aux.finished != 0 {
        return std::ptr::null_mut();
    }
    let start_and_len = if let Some(str_) = str_ {
        aux.finished = 0;
        Some((str_.as_ptr(), str_.len()))
    } else {
        None
    };
    let start = start_and_len
        .map(|(start, _)| start)
        .unwrap_or_else(|| aux.p.add(1).cast::<u8>());

    let p = if let Some((start, len)) = start_and_len {
        let bytes = std::slice::from_raw_parts(start, len);
        start.add(kstrtok_find_in_slice(bytes, aux))
    } else if aux.sep < 0 {
        let mut p = start;
        while *p != 0 {
            if ((aux.tab[(*p >> 6) as usize] >> (*p & 0x3f)) & 1) != 0 {
                break;
            }
            p = p.add(1);
        }
        p
    } else {
        let mut p = start;
        while *p != 0 && *p as c_int != aux.sep {
            p = p.add(1);
        }
        p
    };
    aux.p = p.cast::<c_char>();
    if start_and_len
        .map(|(start, len)| p == start.add(len))
        .unwrap_or_else(|| *p == 0)
    {
        aux.finished = 1;
    }
    start.cast::<c_char>().cast_mut()
}

fn kstrtok_find_in_slice(bytes: &[u8], aux: &ks_tokaux_t) -> usize {
    if aux.sep < 0 {
        bytes
            .iter()
            .position(|&ch| ((aux.tab[(ch >> 6) as usize] >> (ch & 0x3f)) & 1) != 0)
            .unwrap_or(bytes.len())
    } else {
        bytes
            .iter()
            .position(|&ch| ch as c_int == aux.sep)
            .unwrap_or(bytes.len())
    }
}

pub fn fast_exp(mut x: u64, mut n: u64) -> u64 {
    let mut y = 1u64;
    if n == 0 {
        return 1;
    }
    while n > 1 {
        if (n & 1) != 0 {
            y = y.wrapping_mul(x);
        }
        x = x.wrapping_mul(x);
        n >>= 1;
    }
    y.wrapping_mul(x)
}

pub fn karp_rabin(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let b = 31u64;
    let mut hash_pat = 0u64;
    let mut hash_str = 0u64;
    let m = needle.len();
    let b_to_m = fast_exp(b, m as u64);
    let mut mismatch = 0u8;

    if m > haystack.len() {
        return None;
    }

    let mut i = 0usize;
    while i < m {
        mismatch |= haystack[i] ^ needle[i];
        hash_pat = hash_pat
            .wrapping_mul(b)
            .wrapping_add(needle[i] as u64)
            .wrapping_add(1);
        hash_str = hash_str
            .wrapping_mul(b)
            .wrapping_add(haystack[i] as u64)
            .wrapping_add(1);
        i += 1;
    }

    if mismatch == 0 {
        return Some(0);
    }

    while i < haystack.len() {
        hash_str = hash_str
            .wrapping_mul(b)
            .wrapping_add(haystack[i] as u64)
            .wrapping_add(1)
            .wrapping_sub(b_to_m.wrapping_mul((haystack[i - m] as u64).wrapping_add(1)));
        let start = i + 1 - m;
        if hash_str == hash_pat && &haystack[start..start + m] == needle {
            return Some(start);
        }
        i += 1;
    }
    None
}

pub unsafe fn ksBM_prep(pat: *const u8, m: c_int) -> *mut c_int {
    let Ok(m) = usize::try_from(m) else {
        return std::ptr::null_mut();
    };
    let Some(pat) = ptr_len_to_u8_slice(pat, m) else {
        return std::ptr::null_mut();
    };
    let Some(prep_box) = ksbm_prep_box(pat) else {
        return std::ptr::null_mut();
    };
    copy_prep_to_libc_buffer(&prep_box)
}

fn copy_prep_to_libc_buffer(prep_box: &[c_int]) -> *mut c_int {
    let len = prep_box.len();
    let Some(bytes) = len.checked_mul(std::mem::size_of::<c_int>()) else {
        return std::ptr::null_mut();
    };
    let prep = unsafe { crate::htslib_rs::c_compat::malloc(bytes as u64).cast::<c_int>() };
    if prep.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(prep_box.as_ptr(), prep, len);
    }

    prep
}

fn ksbm_prep_box(pat: &[u8]) -> Option<Box<[c_int]>> {
    if pat.is_empty() || pat.len() > c_int::MAX as usize {
        return None;
    }
    let len = pat.len() + 256;

    let Some(mut prep_vec) = zeroed_c_int_vec(len) else {
        return None;
    };
    if !ksbm_fill_prep(pat, &mut prep_vec) {
        return None;
    }

    Some(prep_vec.into_boxed_slice())
}

fn zeroed_c_int_vec(len: usize) -> Option<Vec<c_int>> {
    let mut values = Vec::new();
    values.try_reserve_exact(len).ok()?;
    values.resize(len, 0);
    Some(values)
}

fn ksbm_fill_prep(pat: &[u8], prep: &mut [c_int]) -> bool {
    let m = pat.len() as c_int;
    if prep.len() < pat.len() + 256 {
        return false;
    }

    let (bm_gs, bm_bc) = prep.split_at_mut(pat.len());

    let mut i = 0;
    while i < 256 {
        bm_bc[i as usize] = m;
        i += 1;
    }
    i = 0;
    while i < m - 1 {
        bm_bc[pat[i as usize] as usize] = m - i - 1;
        i += 1;
    }

    let Some(mut suff) = zeroed_c_int_vec(pat.len()) else {
        return false;
    };

    let mut f = 0;
    suff[(m - 1) as usize] = m;
    let mut g = m - 1;
    i = m - 2;
    while i >= 0 {
        if i > g && suff[(i + m - 1 - f) as usize] < i - g {
            suff[i as usize] = suff[(i + m - 1 - f) as usize];
        } else {
            if i < g {
                g = i;
            }
            f = i;
            while g >= 0 && pat[g as usize] == pat[(g + m - 1 - f) as usize] {
                g -= 1;
            }
            suff[i as usize] = f - g;
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }

    let mut j = 0;
    i = 0;
    while i < m {
        bm_gs[i as usize] = m;
        i += 1;
    }
    i = m - 1;
    while i >= 0 {
        if suff[i as usize] == i + 1 {
            while j < m - 1 - i {
                if bm_gs[j as usize] == m {
                    bm_gs[j as usize] = m - 1 - i;
                }
                j += 1;
            }
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    i = 0;
    while i <= m - 2 {
        bm_gs[(m - 1 - suff[i as usize]) as usize] = m - 1 - i;
        i += 1;
    }

    true
}

unsafe fn ptr_len_to_u8_slice<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
    if ptr.is_null() {
        None
    } else {
        Some(std::slice::from_raw_parts(ptr, len))
    }
}

pub fn boyer_moore(
    haystack: &[u8],
    needle: &[u8],
    stored_prep_ptr: Option<&mut *mut c_int>,
) -> Option<usize> {
    let Some(prep_len) = ksbm_search_prep_len(haystack, needle) else {
        return ksbm_search_without_prep(haystack, needle);
    };

    match stored_prep_ptr {
        Some(stored_prep) if !(*stored_prep).is_null() => {
            let prep = unsafe { std::slice::from_raw_parts(*stored_prep, prep_len) };
            boyer_moore_prepped(haystack, needle, prep)
        }
        Some(stored_prep) => {
            let Some(prep_box) = ksbm_prep_box(needle) else {
                return karp_rabin(haystack, needle);
            };
            let prep = copy_prep_to_libc_buffer(&prep_box);
            if prep.is_null() {
                return karp_rabin(haystack, needle);
            }
            *stored_prep = prep;
            boyer_moore_prepped(haystack, needle, &prep_box)
        }
        None => {
            let Some(local_prep) = ksbm_prep_box(needle) else {
                return karp_rabin(haystack, needle);
            };
            boyer_moore_prepped(haystack, needle, &local_prep)
        }
    }
}

pub fn boyer_moore_boxed_prep(
    haystack: &[u8],
    needle: &[u8],
    mut stored_prep: Option<&mut Option<Box<[c_int]>>>,
) -> Option<usize> {
    if ksbm_search_prep_len(haystack, needle).is_none() {
        return ksbm_search_without_prep(haystack, needle);
    }

    if let Some(stored_prep) = stored_prep.as_deref_mut() {
        if stored_prep.is_none() {
            let Some(prep) = ksbm_prep_box(needle) else {
                return karp_rabin(haystack, needle);
            };
            *stored_prep = Some(prep);
        }
        let prep = stored_prep.as_deref().unwrap();
        return boyer_moore_prepped(haystack, needle, prep);
    }

    let Some(local_prep) = ksbm_prep_box(needle) else {
        return karp_rabin(haystack, needle);
    };
    boyer_moore_prepped(haystack, needle, &local_prep)
}

fn ksbm_search_prep_len(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.len() <= 1 || haystack.len() < needle.len() || needle.len() > c_int::MAX as usize {
        None
    } else {
        Some(needle.len() + 256)
    }
}

fn ksbm_search_without_prep(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if haystack.len() < needle.len() {
        return None;
    }
    if needle.len() == 1 {
        return haystack.iter().position(|&ch| ch == needle[0]);
    }

    karp_rabin(haystack, needle)
}

fn boyer_moore_prepped(haystack: &[u8], needle: &[u8], prep: &[c_int]) -> Option<usize> {
    if prep.len() < needle.len() + 256 || needle.len() > c_int::MAX as usize {
        return None;
    }

    let m = needle.len() as c_int;
    let (bm_gs, bm_bc) = prep.split_at(needle.len());

    let mut j = 0usize;
    while j <= haystack.len() - needle.len() {
        let mut i = m - 1;
        while i >= 0 && needle[i as usize] == haystack[i as usize + j] {
            if i == 0 {
                i = -1;
                break;
            }
            i -= 1;
        }
        if i >= 0 {
            let mut max = bm_bc[haystack[i as usize + j] as usize] - m + 1 + i;
            if max < bm_gs[i as usize] {
                max = bm_gs[i as usize];
            }
            j += max as usize;
        } else {
            return Some(j);
        }
    }

    None
}

pub unsafe fn kmemmem(
    str_: *const c_void,
    n: c_int,
    pat: *const c_void,
    m: c_int,
    prep: *mut *mut c_int,
) -> *mut c_void {
    let n = if n >= 0 { n as usize } else { 0 };
    let Some(haystack) = ptr_len_to_u8_slice(str_.cast::<u8>(), n) else {
        return str_.cast_mut();
    };
    let Ok(m) = usize::try_from(m) else {
        return std::ptr::null_mut();
    };
    let Some(needle) = ptr_len_to_u8_slice(pat.cast::<u8>(), m) else {
        return str_.cast_mut();
    };
    let Some(offset) = boyer_moore(
        haystack,
        needle,
        if prep.is_null() {
            None
        } else {
            Some(&mut *prep)
        },
    ) else {
        return std::ptr::null_mut();
    };
    haystack.as_ptr().add(offset).cast::<c_void>().cast_mut()
}

pub fn kstrstr(
    haystack: &[u8],
    needle: &[u8],
    prep: Option<&mut *mut c_int>,
) -> Option<usize> {
    // C-string semantics: search only up to the first NUL terminator (no-op for
    // NUL-free input). Matches the pre-refactor `&CStr`-based behavior.
    let end = haystack.iter().position(|&b| b == 0).unwrap_or(haystack.len());
    let haystack = &haystack[..end];
    if needle.len() <= c_int::MAX as usize {
        boyer_moore(haystack, needle, prep)
    } else {
        karp_rabin(haystack, needle)
    }
}

pub fn kstrstr_boxed_prep(
    haystack: &[u8],
    needle: &[u8],
    prep: Option<&mut Option<Box<[c_int]>>>,
) -> Option<usize> {
    // C-string semantics: bound the search at the first NUL (no-op for NUL-free input).
    let end = haystack.iter().position(|&b| b == 0).unwrap_or(haystack.len());
    let haystack = &haystack[..end];
    if needle.len() <= c_int::MAX as usize {
        boyer_moore_boxed_prep(haystack, needle, prep)
    } else {
        karp_rabin(haystack, needle)
    }
}

pub fn kstrnstr(
    haystack: &[u8],
    needle: &[u8],
    n: usize,
    prep: Option<&mut *mut c_int>,
) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    let n = n.min(haystack.len());
    // C-string semantics (matches the pre-refactor `&CStr` behavior): the haystack
    // is treated as NUL-terminated, so the search is bounded at the first NUL within
    // the first n bytes. No-op for NUL-free input.
    let n = haystack[..n].iter().position(|&b| b == 0).unwrap_or(n);
    if needle.len() > n {
        return None;
    }
    boyer_moore(&haystack[..n], needle, prep)
}

pub unsafe fn kputd(d: f64, s: &mut kstring_t) -> c_int {
    let Some(text) = kputd_bytes(d) else {
        return -1;
    };
    if text.len() > c_int::MAX as usize {
        return -1;
    }
    if kputsn(&text, text.len(), s) < 0 {
        return -1;
    }
    text.len() as c_int
}

fn kputd_bytes(d: f64) -> Option<Vec<u8>> {
    if d == 0.0 {
        return Some(if d.is_sign_negative() {
            b"-0".to_vec()
        } else {
            b"0".to_vec()
        });
    }

    let mut d = d;
    let mut out = Vec::new();
    if d < 0.0 {
        out.push(b'-');
        d = -d;
    }

    if !(0.0001..=999999.0).contains(&d) {
        let mut buf = [0 as c_char; 64];
        let len = unsafe {
            // Keep htslib's %g formatting for exponent cases while avoiding
            // temporary kstring allocation.
            libc::snprintf(
                buf.as_mut_ptr(),
                buf.len() as libc::size_t,
                c"%g".as_ptr(),
                d,
            )
        };
        if len < 0 || len as usize >= buf.len() {
            return None;
        }
        let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), len as usize) };
        out.extend_from_slice(bytes);
        return Some(out);
    }

    let decimals = if d < 0.001 {
        9
    } else if d < 0.01 {
        8
    } else if d < 0.1 {
        7
    } else if d < 1.0 {
        6
    } else if d < 10.0 {
        5
    } else if d < 100.0 {
        4
    } else if d < 1000.0 {
        3
    } else if d < 10000.0 {
        2
    } else if d < 100000.0 {
        1
    } else {
        0
    };
    let text = format!("{d:.decimals$}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    out.extend_from_slice(text.as_bytes());
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ksprintf_allocates_and_appends_to_kstring() {
        unsafe {
            let mut s = kstring_t::default();

            assert_eq!(
                ksprintf(&mut s, b"sample-%d", &[KsPrintfArg::Int(42)]),
                9
            );
            assert_eq!(s.data.len(), 9);
            assert_eq!(s.data.as_slice(), b"sample-42");

            assert_eq!(
                ksprintf(
                    &mut s,
                    b":%s",
                    &[KsPrintfArg::Str(c"ok".as_ptr())],
                ),
                3
            );
            assert_eq!(s.data.len(), 12);
            assert_eq!(s.data.as_slice(), b"sample-42:ok");

            ks_free(&mut s);
        }
    }

    #[test]
    fn kvsprintf_formats_synthetic_va_list_without_c_vsnprintf() {
        unsafe {
            let mut s = kstring_t::default();
            let mut reg_save = [0usize; 24];
            reg_save[0] = 7;
            reg_save[1] = c"abc".as_ptr() as usize;
            let float_slot = reg_save.as_mut_ptr().cast::<u8>().add(48);
            std::ptr::write_unaligned(float_slot.cast::<f64>(), 3.5);
            let mut overflow = [0usize; 1];
            let mut args = crate::htslib_rs::c_compat::__va_list_tag {
                gp_offset: 0,
                fp_offset: 48,
                overflow_arg_area: overflow.as_mut_ptr().cast(),
                reg_save_area: reg_save.as_mut_ptr().cast(),
            };

            assert_eq!(
                kvsprintf(&mut s, b"%d:%s:%g:%%", &mut args),
                11
            );
            assert_eq!(s.data.as_slice(), b"7:abc:3.5:%");

            ks_free(&mut s);
        }
    }
}
