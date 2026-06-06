// Functions translated from htslib/cram/string_alloc.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::c_char;

use super::*;

unsafe fn take_string_table(a_str: &mut cram_string_alloc_t) -> Vec<cram_string_alloc_string_t> {
    if a_str.strings.is_null() {
        Vec::new()
    } else {
        Vec::from_raw_parts(a_str.strings, a_str.nstrings, a_str.max_strings)
    }
}

fn install_string_table(
    a_str: &mut cram_string_alloc_t,
    mut strings: Vec<cram_string_alloc_string_t>,
) {
    a_str.nstrings = strings.len();
    a_str.max_strings = strings.capacity();
    a_str.strings = if strings.capacity() == 0 {
        std::ptr::null_mut()
    } else {
        strings.as_mut_ptr()
    };
    std::mem::forget(strings);
}

pub unsafe fn cram_string_alloc_c_55_string_pool_create(
    mut max_length: usize,
) -> *mut cram_string_alloc_t {
    if max_length < CRAM_STRING_ALLOC_MIN_STR_SIZE {
        max_length = CRAM_STRING_ALLOC_MIN_STR_SIZE;
    }

    Box::into_raw(Box::new(cram_string_alloc_t {
        max_length,
        nstrings: 0,
        max_strings: 0,
        strings: std::ptr::null_mut(),
    }))
}

pub unsafe fn cram_string_alloc_c_75_new_string_pool(
    a_str: *mut cram_string_alloc_t,
) -> *mut cram_string_alloc_string_t {
    let Some(a_str) = a_str.as_mut() else {
        return std::ptr::null_mut();
    };

    let mut strings = take_string_table(a_str);

    if strings.len() == strings.capacity() {
        let new_max = (strings.capacity() | (strings.capacity() >> 2)) + 1;
        if strings
            .try_reserve_exact(new_max.saturating_sub(strings.capacity()))
            .is_err()
        {
            install_string_table(a_str, strings);
            return std::ptr::null_mut();
        }
    }

    let slab = malloc(a_str.max_length as u64).cast::<c_char>();
    if slab.is_null() {
        install_string_table(a_str, strings);
        return std::ptr::null_mut();
    }

    strings.push(cram_string_alloc_string_t {
        str_: slab,
        used: 0,
    });

    let str_ = strings.as_mut_ptr().add(strings.len() - 1);
    install_string_table(a_str, strings);
    str_
}

pub unsafe fn cram_string_alloc_c_103_string_pool_destroy(a_str: *mut cram_string_alloc_t) {
    let Some(a_str) = a_str.as_mut() else {
        return;
    };

    let strings = take_string_table(a_str);
    for str_ in &strings {
        free(str_.str_.cast());
    }

    a_str.strings = std::ptr::null_mut();
    a_str.nstrings = 0;
    a_str.max_strings = 0;
    drop(strings);
    drop(Box::from_raw(a_str));
}

pub unsafe fn cram_string_alloc_c_117_string_alloc(
    a_str: *mut cram_string_alloc_t,
    length: usize,
) -> *mut c_char {
    if length == 0 {
        return std::ptr::null_mut();
    }

    let Some(a_str_ref) = a_str.as_mut() else {
        return std::ptr::null_mut();
    };

    if a_str_ref.nstrings != 0 {
        let str_ = a_str_ref.strings.add(a_str_ref.nstrings - 1);

        if (*str_).used + length < a_str_ref.max_length {
            let ret = (*str_).str_.add((*str_).used);
            (*str_).used += length;
            return ret;
        }
    }

    if length > a_str_ref.max_length {
        a_str_ref.max_length = length;
    }

    let str_ = cram_string_alloc_c_75_new_string_pool(a_str);
    if str_.is_null() {
        return std::ptr::null_mut();
    }

    (*str_).used = length;
    (*str_).str_
}

pub unsafe fn cram_string_alloc_c_149_string_dup(
    a_str: *mut cram_string_alloc_t,
    instr: *const c_char,
) -> *mut c_char {
    cram_string_alloc_c_153_string_ndup(a_str, instr, libc::strlen(instr))
}

pub unsafe fn cram_string_alloc_c_153_string_ndup(
    a_str: *mut cram_string_alloc_t,
    instr: *const c_char,
    len: usize,
) -> *mut c_char {
    let str_ = cram_string_alloc_c_117_string_alloc(a_str, len + 1);
    if str_.is_null() {
        return std::ptr::null_mut();
    }

    memcpy(str_.cast(), instr.cast(), len as u64);
    *str_.add(len) = 0;

    str_
}
