// Functions translated from htslib/cram/string_alloc.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::c_char;

use super::*;

pub unsafe fn cram_string_alloc_c_55_string_pool_create(
    mut max_length: usize,
) -> *mut cram_string_alloc_t {
    let a_str =
        malloc(std::mem::size_of::<cram_string_alloc_t>() as u64) as *mut cram_string_alloc_t;
    if a_str.is_null() {
        return std::ptr::null_mut();
    }

    if max_length < CRAM_STRING_ALLOC_MIN_STR_SIZE {
        max_length = CRAM_STRING_ALLOC_MIN_STR_SIZE;
    }

    (*a_str).nstrings = 0;
    (*a_str).max_strings = 0;
    (*a_str).max_length = max_length;
    (*a_str).strings = std::ptr::null_mut();

    a_str
}

pub unsafe fn cram_string_alloc_c_75_new_string_pool(
    a_str: *mut cram_string_alloc_t,
) -> *mut cram_string_alloc_string_t {
    if (*a_str).nstrings == (*a_str).max_strings {
        let new_max = ((*a_str).max_strings | ((*a_str).max_strings >> 2)) + 1;
        let str_ = realloc(
            (*a_str).strings.cast(),
            (new_max * std::mem::size_of::<cram_string_alloc_string_t>()) as u64,
        ) as *mut cram_string_alloc_string_t;

        if str_.is_null() {
            return std::ptr::null_mut();
        }

        (*a_str).strings = str_;
        (*a_str).max_strings = new_max;
    }

    let str_ = (*a_str).strings.add((*a_str).nstrings);
    (*str_).str_ = malloc((*a_str).max_length as u64).cast();

    if (*str_).str_.is_null() {
        return std::ptr::null_mut();
    }

    (*str_).used = 0;
    (*a_str).nstrings += 1;

    str_
}

pub unsafe fn cram_string_alloc_c_103_string_pool_destroy(a_str: *mut cram_string_alloc_t) {
    for i in 0..(*a_str).nstrings {
        free((*(*a_str).strings.add(i)).str_.cast());
    }

    free((*a_str).strings.cast());
    free(a_str.cast());
}

pub unsafe fn cram_string_alloc_c_117_string_alloc(
    a_str: *mut cram_string_alloc_t,
    length: usize,
) -> *mut c_char {
    if length == 0 {
        return std::ptr::null_mut();
    }

    if (*a_str).nstrings != 0 {
        let str_ = (*a_str).strings.add((*a_str).nstrings - 1);

        if (*str_).used + length < (*a_str).max_length {
            let ret = (*str_).str_.add((*str_).used);
            (*str_).used += length;
            return ret;
        }
    }

    if length > (*a_str).max_length {
        (*a_str).max_length = length;
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
