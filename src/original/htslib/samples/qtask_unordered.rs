use std::ffi::{c_char, c_int, c_void};

use crate::htslib_mini_rs::{hts, sam, thread_pool};

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
pub unsafe extern "C" fn samples_qtask_unordered_c_76_getbamstorage(
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
pub unsafe extern "C" fn samples_qtask_unordered_c_128_cleanup_bamstorage(arg: *mut c_void) {
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
pub unsafe extern "C" fn samples_qtask_unordered_c_148_thread_unordered_proc(
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
pub unsafe fn samples_qtask_unordered_c_181_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut ret = libc::EXIT_FAILURE;
    let mut infile = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();
    let mut pool = std::ptr::null_mut();
    let mut queue = std::ptr::null_mut();
    let mut tpool = hts::htsThreadPool {
        pool: std::ptr::null_mut(),
        qsize: 0,
    };
    let mut bamdata: *mut QTaskUnorderedData = std::ptr::null_mut();
    let mut gccount = [0_u64; 16];
    let mut bamcache: QTaskUnorderedDataCache = std::mem::zeroed();
    libc::pthread_mutex_init(&mut bamcache.lock, std::ptr::null());

    if argc != 3 && argc != 4 {
        samples_qtask_unordered_c_62_print_usage(hts_sys::stdout.cast());
    } else {
        let inname = *argv.add(1);
        let mut cnt = libc::atoi(*argv.add(2));
        let mut chunk = if argc == 4 {
            libc::atoi(*argv.add(3))
        } else {
            0
        };
        if cnt < 1 {
            cnt = 1;
        }
        if chunk < 1 {
            chunk = 4096;
        }

        pool = thread_pool::hts_tpool_init(cnt);
        if pool.is_null() {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Failed to create thread pool\n".as_ptr(),
            );
        } else {
            tpool.pool = pool;
            queue = thread_pool::hts_tpool_process_init(pool, cnt * 2, 1);
            if queue.is_null() {
                libc::fprintf(hts_sys::stderr.cast(), c"Failed to create queue\n".as_ptr());
            } else {
                infile = hts::hts_open(inname, c"r".as_ptr());
                if infile.is_null() {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Could not open %s\n".as_ptr(),
                        inname,
                    );
                } else if hts::hts_set_thread_pool(infile, &mut tpool) < 0 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Failed to set threads to i/o files\n".as_ptr(),
                    );
                } else {
                    in_samhdr = sam::sam_hdr_read(infile);
                    if in_samhdr.is_null() {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Failed to read header from file!\n".as_ptr(),
                        );
                    } else {
                        let mut c = 0;
                        while c >= 0 {
                            bamdata = samples_qtask_unordered_c_76_getbamstorage(
                                chunk,
                                gccount.as_mut_ptr().cast(),
                                (&mut bamcache as *mut QTaskUnorderedDataCache).cast(),
                            )
                            .cast();
                            if bamdata.is_null() {
                                libc::fprintf(
                                    hts_sys::stderr.cast(),
                                    c"Failed to allocate memory\n".as_ptr(),
                                );
                                break;
                            }
                            cnt = 0;
                            while cnt < (*bamdata).maxsize {
                                c = sam::sam_read1(
                                    infile,
                                    in_samhdr,
                                    *(*bamdata).bamarray.add(cnt as usize),
                                );
                                if c < 0 {
                                    break;
                                }
                                cnt += 1;
                            }
                            if c >= -1 {
                                (*bamdata).count = cnt;
                                if thread_pool::hts_tpool_dispatch3(
                                    pool,
                                    queue,
                                    Some(samples_qtask_unordered_c_148_thread_unordered_proc),
                                    bamdata.cast(),
                                    Some(samples_qtask_unordered_c_128_cleanup_bamstorage),
                                    Some(samples_qtask_unordered_c_128_cleanup_bamstorage),
                                    0,
                                ) == -1
                                {
                                    libc::fprintf(
                                        hts_sys::stderr.cast(),
                                        c"Failed to schedule processing\n".as_ptr(),
                                    );
                                    break;
                                }
                                bamdata = std::ptr::null_mut();
                            } else {
                                libc::fprintf(
                                    hts_sys::stderr.cast(),
                                    c"Error in reading data\n".as_ptr(),
                                );
                                break;
                            }
                        }

                        if c == -1 {
                            if thread_pool::hts_tpool_process_flush(queue) == -1 {
                                libc::fprintf(
                                    hts_sys::stderr.cast(),
                                    c"Failed to flush queue\n".as_ptr(),
                                );
                            } else {
                                libc::fprintf(
                                    hts_sys::stdout.cast(),
                                    c"GCratio: %f\nBase counts:\n".as_ptr(),
                                    (gccount[2] + gccount[4]) as f64
                                        / (gccount[1] + gccount[8] + gccount[2] + gccount[4])
                                            as f64,
                                );
                                for i in 0..16 {
                                    libc::fprintf(
                                        hts_sys::stdout.cast(),
                                        c"%c: %llu\n".as_ptr(),
                                        SEQ_NT16_STR[i] as c_int,
                                        gccount[i] as libc::c_ulonglong,
                                    );
                                }
                                ret = libc::EXIT_SUCCESS;
                            }
                        }
                    }
                }
            }
        }
    }

    if !queue.is_null() {
        thread_pool::hts_tpool_process_destroy(queue);
    }
    if !in_samhdr.is_null() {
        sam::sam_hdr_destroy(in_samhdr);
    }
    if !infile.is_null() && hts::hts_close(infile) != 0 {
        ret = libc::EXIT_FAILURE;
    }
    if !bamdata.is_null() {
        samples_qtask_unordered_c_128_cleanup_bamstorage(bamdata.cast());
    }

    libc::pthread_mutex_lock(&mut bamcache.lock);
    while !bamcache.list.is_null() {
        let tmp = bamcache.list;
        bamcache.list = (*bamcache.list).next;
        samples_qtask_unordered_c_128_cleanup_bamstorage(tmp.cast());
    }
    libc::pthread_mutex_unlock(&mut bamcache.lock);
    libc::pthread_mutex_destroy(&mut bamcache.lock);

    if !pool.is_null() {
        thread_pool::hts_tpool_destroy(pool);
    }
    ret
}
