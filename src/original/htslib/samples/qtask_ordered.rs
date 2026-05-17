use std::ffi::{c_char, c_int, c_void};

use crate::htslib_mini_rs::sam;

#[repr(C)]
struct QTaskOrderedData {
    count: c_int,
    maxsize: c_int,
    bamarray: *mut *mut sam::bam1_t,
    next: *mut QTaskOrderedData,
}

#[repr(C)]
struct QTaskOrderedDataCache {
    lock: libc::pthread_mutex_t,
    list: *mut QTaskOrderedData,
}

// original: print_usage (htslib/samples/qtask_ordered.c:61)
pub unsafe fn samples_qtask_ordered_c_61_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: qtask_ordered infile threadcount outdir [chunksize]\nCalculates GC ratio - sum(G,C) / sum(A,T,C,G) - and adds to each alignment\nas xr:f aux tag. Output is saved in outdir.\nchunksize [4096] sets the number of alignments clubbed together to process.\n".as_ptr(),
    );
}

// original: getbamstorage (htslib/samples/qtask_ordered.c:75)
pub unsafe fn samples_qtask_ordered_c_75_getbamstorage(
    chunk: c_int,
    bamcache: *mut c_void,
) -> *mut c_void {
    let bamcache = bamcache.cast::<QTaskOrderedDataCache>();
    if bamcache.is_null() {
        return std::ptr::null_mut();
    }
    if libc::pthread_mutex_lock(&mut (*bamcache).lock) != 0 {
        return std::ptr::null_mut();
    }

    let mut bamdata: *mut QTaskOrderedData;
    if !(*bamcache).list.is_null() {
        bamdata = (*bamcache).list;
        (*bamcache).list = (*bamdata).next;
        (*bamdata).next = std::ptr::null_mut();
        (*bamdata).count = 0;
    } else {
        bamdata = libc::malloc(std::mem::size_of::<QTaskOrderedData>()).cast::<QTaskOrderedData>();
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
            }
        }
    }

    libc::pthread_mutex_unlock(&mut (*bamcache).lock);
    bamdata.cast()
}

// original: cleanup_bamstorage (htslib/samples/qtask_ordered.c:121)
pub unsafe fn samples_qtask_ordered_c_121_cleanup_bamstorage(arg: *mut c_void) {
    let bamdata = arg.cast::<QTaskOrderedData>();
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

// original: thread_ordered_proc (htslib/samples/qtask_ordered.c:143)
pub unsafe fn samples_qtask_ordered_c_143_thread_ordered_proc(args: *mut c_void) -> *mut c_void {
    let bamdata = args.cast::<QTaskOrderedData>();
    if bamdata.is_null() {
        return std::ptr::null_mut();
    }

    for i in 0..(*bamdata).count {
        let bam = *(*bamdata).bamarray.add(i as usize);
        let seq = sam::bam_get_seq(bam);
        let mut count = [0_u64; 16];
        for pos in 0..(*bam).core.l_qseq as usize {
            count[sam::bam_seqi(seq, pos) as usize] += 1;
        }

        let denom = (count[1] + count[8] + count[2] + count[4]) as f32;
        let gcratio = (count[2] + count[4]) as f32 / denom;
        if sam::bam_aux_append(
            bam,
            c"xr".as_ptr(),
            b'f' as c_char,
            std::mem::size_of::<f32>() as c_int,
            (&gcratio as *const f32).cast::<u8>(),
        ) < 0
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Failed to add aux tag xr, errno: %d\n".as_ptr(),
                *libc::__errno_location(),
            );
            break;
        }
    }
    bamdata.cast()
}

// original: threadfn_orderedwrite (htslib/samples/qtask_ordered.c:176)
pub unsafe fn samples_qtask_ordered_c_176_threadfn_orderedwrite() {
    todo!("translate HTSlib threadfn_orderedwrite from htslib/samples/qtask_ordered.c:176");
}

// original: main (htslib/samples/qtask_ordered.c:223)
pub unsafe fn samples_qtask_ordered_c_223_main() {
    todo!("translate HTSlib main from htslib/samples/qtask_ordered.c:223");
}
