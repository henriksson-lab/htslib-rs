use std::ffi::{c_int, c_void};

use crate::htslib_mini_rs::sam;

#[repr(C)]
struct QTaskUnorderedData {
    count: c_int,
    maxsize: c_int,
    bamarray: *mut *mut sam::bam1_t,
    cache: *mut QTaskUnorderedDataCache,
    bases: *mut c_void,
    next: *mut QTaskUnorderedData,
}

#[repr(C)]
struct QTaskUnorderedDataCache {
    lock: libc::pthread_mutex_t,
    list: *mut QTaskUnorderedData,
}

// original: print_usage (htslib/samples/qtask_unordered.c:62)
pub unsafe fn samples_qtask_unordered_c_62_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: qtask_unordered infile threadcount [chunksize]\nShows the base counts and calculates GC ratio - sum(G,C) / sum(A,T,C,G)\nchunksize [4096] sets the number of alignments clubbed together to process.\n".as_ptr(),
    );
}

// original: getbamstorage (htslib/samples/qtask_unordered.c:76)
pub unsafe fn samples_qtask_unordered_c_76_getbamstorage(
    chunk: c_int,
    bases: *mut c_void,
    bamcache: *mut c_void,
) -> *mut c_void {
    let bamcache = bamcache.cast::<QTaskUnorderedDataCache>();
    if bamcache.is_null() || bases.is_null() {
        return std::ptr::null_mut();
    }
    if libc::pthread_mutex_lock(&mut (*bamcache).lock) != 0 {
        return std::ptr::null_mut();
    }

    let mut bamdata: *mut QTaskUnorderedData;
    if !(*bamcache).list.is_null() {
        bamdata = (*bamcache).list;
        (*bamcache).list = (*bamdata).next;
        (*bamdata).next = std::ptr::null_mut();
        (*bamdata).count = 0;
        (*bamdata).bases = bases;
        (*bamdata).cache = bamcache;
    } else {
        bamdata =
            libc::malloc(std::mem::size_of::<QTaskUnorderedData>()).cast::<QTaskUnorderedData>();
        if !bamdata.is_null() {
            (*bamdata).bamarray =
                libc::malloc(chunk as usize * std::mem::size_of::<*mut sam::bam1_t>()).cast();
            if (*bamdata).bamarray.is_null() {
                libc::free(bamdata.cast());
                bamdata = std::ptr::null_mut();
            } else {
                for i in 0..chunk {
                    *(*bamdata).bamarray.add(i as usize) = sam::bam_init1();
                }
                (*bamdata).maxsize = chunk;
                (*bamdata).count = 0;
                (*bamdata).next = std::ptr::null_mut();
                (*bamdata).bases = bases;
                (*bamdata).cache = bamcache;
            }
        }
    }

    libc::pthread_mutex_unlock(&mut (*bamcache).lock);
    bamdata.cast()
}

// original: cleanup_bamstorage (htslib/samples/qtask_unordered.c:128)
pub unsafe fn samples_qtask_unordered_c_128_cleanup_bamstorage(arg: *mut c_void) {
    let bamdata = arg.cast::<QTaskUnorderedData>();
    if bamdata.is_null() {
        return;
    }
    if !(*bamdata).bamarray.is_null() {
        for i in 0..(*bamdata).maxsize {
            sam::bam_destroy1(*(*bamdata).bamarray.add(i as usize));
        }
        libc::free((*bamdata).bamarray.cast());
    }
    libc::free(bamdata.cast());
}

// original: thread_unordered_proc (htslib/samples/qtask_unordered.c:148)
pub unsafe fn samples_qtask_unordered_c_148_thread_unordered_proc(
    args: *mut c_void,
) -> *mut c_void {
    let bamdata = args.cast::<QTaskUnorderedData>();
    let mut counts = [0_u64; 16];

    for i in 0..(*bamdata).count {
        let bam = *(*bamdata).bamarray.add(i as usize);
        let seq = sam::bam_get_seq(bam);
        for pos in 0..(*bam).core.l_qseq as usize {
            counts[sam::bam_seqi(seq, pos) as usize] += 1;
        }
    }

    libc::pthread_mutex_lock(&mut (*(*bamdata).cache).lock);
    let bases = (*bamdata).bases.cast::<u64>();
    for i in 0..16 {
        *bases.add(i) += counts[i];
    }
    (*bamdata).next = (*(*bamdata).cache).list;
    (*(*bamdata).cache).list = bamdata;
    libc::pthread_mutex_unlock(&mut (*(*bamdata).cache).lock);

    std::ptr::null_mut()
}

// original: main (htslib/samples/qtask_unordered.c:181)
pub unsafe fn samples_qtask_unordered_c_181_main() {
    todo!("translate HTSlib main from htslib/samples/qtask_unordered.c:181");
}
