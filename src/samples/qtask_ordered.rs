use std::ffi::{c_char, c_int, c_void};

use crate::htslib_rs::{hts, sam, thread_pool};

#[repr(C)]
struct QTaskOrderedData {
    count: c_int,
    maxsize: c_int,
    bamarray: *mut *mut sam::bam1_t,
    next: *mut QTaskOrderedData,
}

#[repr(C)]
struct QTaskOrderedDataCache {
    lock: crate::htslib_rs::c_compat::pthread_mutex_t,
    list: *mut QTaskOrderedData,
}

#[repr(C)]
struct QTaskOrderedWrite {
    outfile: *mut hts::htsFile,
    samhdr: *mut sam::sam_hdr_t,
    queue: *mut thread_pool::hts_tpool_process,
    cache: *mut QTaskOrderedDataCache,
    result: c_int,
}

// original: print_usage (htslib/samples/qtask_ordered.c:61)
pub unsafe fn samples_qtask_ordered_c_61_print_usage(fp: *mut libc::FILE) {
    libc::fprintf(
        fp,
        c"Usage: qtask_ordered infile threadcount outdir [chunksize]\nCalculates GC ratio - sum(G,C) / sum(A,T,C,G) - and adds to each alignment\nas xr:f aux tag. Output is saved in outdir.\nchunksize [4096] sets the number of alignments clubbed together to process.\n".as_ptr(),
    );
}

// original: getbamstorage (htslib/samples/qtask_ordered.c:75)
pub unsafe extern "C" fn samples_qtask_ordered_c_75_getbamstorage(
    chunk: c_int,
    bamcache: *mut c_void,
) -> *mut c_void {
    let bamcache = bamcache.cast::<QTaskOrderedDataCache>();
    if bamcache.is_null() {
        return std::ptr::null_mut();
    }
    if crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*bamcache).lock) != 0 {
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

    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*bamcache).lock);
    bamdata.cast()
}

// original: cleanup_bamstorage (htslib/samples/qtask_ordered.c:121)
pub unsafe extern "C" fn samples_qtask_ordered_c_121_cleanup_bamstorage(arg: *mut c_void) {
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
pub unsafe extern "C" fn samples_qtask_ordered_c_143_thread_ordered_proc(
    args: *mut c_void,
) -> *mut c_void {
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
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Failed to add aux tag xr, errno: %d\n".as_ptr(),
                *crate::htslib_rs::c_compat::__errno_location(),
            );
            break;
        }
    }
    bamdata.cast()
}

// original: threadfn_orderedwrite (htslib/samples/qtask_ordered.c:176)
pub extern "C" fn samples_qtask_ordered_c_176_threadfn_orderedwrite(
    args: *mut c_void,
) -> *mut c_void {
    unsafe {
        let tdata = args.cast::<QTaskOrderedWrite>();
        (*tdata).result = 0;

        while (*tdata).result == 0 {
            let r = thread_pool::hts_tpool_next_result_wait((*tdata).queue);
            if r.is_null() {
                break;
            }
            let bamdata = thread_pool::hts_tpool_result_data(r).cast::<QTaskOrderedData>();
            if bamdata.is_null() {
                thread_pool::hts_tpool_delete_result(r, 0);
                break;
            }

            for i in 0..(*bamdata).count {
                if sam::sam_c_4553_sam_write1(
                    (*tdata).outfile,
                    (*tdata).samhdr,
                    *(*bamdata).bamarray.add(i as usize),
                ) < 0
                {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Failed to write output data\n".as_ptr(),
                    );
                    (*tdata).result = -1;
                    break;
                }
            }
            thread_pool::hts_tpool_delete_result(r, 0);

            crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*(*tdata).cache).lock);
            (*bamdata).next = (*(*tdata).cache).list;
            (*(*tdata).cache).list = bamdata;
            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*(*tdata).cache).lock);
        }

        thread_pool::hts_tpool_process_shutdown((*tdata).queue);
    }
    std::ptr::null_mut()
}

// original: main (htslib/samples/qtask_ordered.c:223)
pub unsafe fn samples_qtask_ordered_c_223_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut ret = libc::EXIT_FAILURE;
    let mut c = 0;
    let mut started_thread = 0;
    let mut infile = std::ptr::null_mut();
    let mut outfile = std::ptr::null_mut();
    let mut in_samhdr = std::ptr::null_mut();
    let mut pool = std::ptr::null_mut();
    let mut queue = std::ptr::null_mut();
    let mut tpool = hts::htsThreadPool {
        pool: std::ptr::null_mut(),
        qsize: 0,
    };
    let mut bamdata: *mut QTaskOrderedData = std::ptr::null_mut();
    let mut bamcache: QTaskOrderedDataCache = std::mem::zeroed();
    let mut thread: crate::htslib_rs::c_compat::pthread_t = std::mem::zeroed();
    let mut twritedata: QTaskOrderedWrite = std::mem::zeroed();
    crate::htslib_rs::c_compat::pthread_mutex_init(&mut bamcache.lock, std::ptr::null());

    if argc != 4 && argc != 5 {
        samples_qtask_ordered_c_61_print_usage(crate::htslib_rs::c_compat::stdout.cast());
    } else {
        let inname = *argv.add(1);
        let mut cnt = libc::atoi(*argv.add(2));
        let outdir = *argv.add(3);
        let mut chunk = if argc == 5 {
            libc::atoi(*argv.add(4))
        } else {
            0
        };
        if cnt < 1 {
            cnt = 1;
        }
        if chunk < 1 {
            chunk = 4096;
        }

        let size = libc::strlen(outdir) + c"/out.bam".to_bytes_with_nul().len() + 1;
        let file = libc::malloc(size).cast::<c_char>();
        if file.is_null() {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Failed to set output path\n".as_ptr(),
            );
        } else {
            libc::snprintf(file, size, c"%s/out.bam".as_ptr(), outdir);
            pool = thread_pool::hts_tpool_init(cnt);
            if pool.is_null() {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Failed to create thread pool\n".as_ptr(),
                );
            } else {
                tpool.pool = pool;
                queue = thread_pool::hts_tpool_process_init(pool, cnt * 2, 0);
                if queue.is_null() {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Failed to create queue\n".as_ptr(),
                    );
                } else {
                    infile = hts::hts_open(inname, c"r".as_ptr());
                    if infile.is_null() {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Could not open %s\n".as_ptr(),
                            inname,
                        );
                    } else {
                        outfile = hts::hts_open(file, c"wb".as_ptr());
                        if outfile.is_null() {
                            libc::fprintf(
                                crate::htslib_rs::c_compat::stderr.cast(),
                                c"Could not open output file\n".as_ptr(),
                            );
                        } else if hts::hts_set_thread_pool(infile, &mut tpool) < 0
                            || hts::hts_set_thread_pool(outfile, &mut tpool) < 0
                        {
                            libc::fprintf(
                                crate::htslib_rs::c_compat::stderr.cast(),
                                c"Failed to set threads to i/o files\n".as_ptr(),
                            );
                        } else {
                            in_samhdr = sam::sam_hdr_read(infile);
                            if in_samhdr.is_null() {
                                libc::fprintf(
                                    crate::htslib_rs::c_compat::stderr.cast(),
                                    c"Failed to read header from file!\n".as_ptr(),
                                );
                            } else if sam::sam_hdr_write(outfile, in_samhdr) == -1 {
                                libc::fprintf(
                                    crate::htslib_rs::c_compat::stderr.cast(),
                                    c"Failed to write header\n".as_ptr(),
                                );
                            } else {
                                twritedata.outfile = outfile;
                                twritedata.samhdr = in_samhdr;
                                twritedata.result = 0;
                                twritedata.queue = queue;
                                twritedata.cache = &mut bamcache;
                                if crate::htslib_rs::c_compat::pthread_create(
                                    &mut thread,
                                    std::ptr::null(),
                                    samples_qtask_ordered_c_176_threadfn_orderedwrite,
                                    (&mut twritedata as *mut QTaskOrderedWrite).cast(),
                                ) != 0
                                {
                                    libc::fprintf(
                                        crate::htslib_rs::c_compat::stderr.cast(),
                                        c"Failed to create writer thread\n".as_ptr(),
                                    );
                                } else {
                                    started_thread = 1;
                                    let mut dispatch_failed = false;
                                    c = 0;
                                    while c >= 0 {
                                        bamdata = samples_qtask_ordered_c_75_getbamstorage(
                                            chunk,
                                            (&mut bamcache as *mut QTaskOrderedDataCache).cast(),
                                        )
                                        .cast();
                                        if bamdata.is_null() {
                                            libc::fprintf(
                                                crate::htslib_rs::c_compat::stderr.cast(),
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
                                                Some(
                                                    samples_qtask_ordered_c_143_thread_ordered_proc,
                                                ),
                                                bamdata.cast(),
                                                Some(
                                                    samples_qtask_ordered_c_121_cleanup_bamstorage,
                                                ),
                                                Some(
                                                    samples_qtask_ordered_c_121_cleanup_bamstorage,
                                                ),
                                                0,
                                            ) == -1
                                            {
                                                libc::fprintf(
                                                    crate::htslib_rs::c_compat::stderr.cast(),
                                                    c"Failed to schedule processing\n".as_ptr(),
                                                );
                                                dispatch_failed = true;
                                                break;
                                            }
                                            bamdata = std::ptr::null_mut();
                                        } else {
                                            libc::fprintf(
                                                crate::htslib_rs::c_compat::stderr.cast(),
                                                c"Error in reading data\n".as_ptr(),
                                            );
                                            break;
                                        }
                                    }
                                    if !dispatch_failed {
                                        ret = libc::EXIT_SUCCESS;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            libc::free(file.cast());
        }
    }

    if !queue.is_null() {
        if c == -1 {
            if thread_pool::hts_tpool_dispatch(
                pool,
                queue,
                Some(samples_qtask_ordered_c_143_thread_ordered_proc),
                std::ptr::null_mut(),
            ) == -1
            {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Failed to schedule sentinel job\n".as_ptr(),
                );
                ret = libc::EXIT_FAILURE;
            }
        } else {
            thread_pool::hts_tpool_process_shutdown(queue);
        }
    }

    if started_thread != 0 {
        crate::htslib_rs::c_compat::pthread_join(thread, std::ptr::null_mut());
        if twritedata.result != 0 {
            ret = libc::EXIT_FAILURE;
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
    if !outfile.is_null() && hts::hts_close(outfile) != 0 {
        ret = libc::EXIT_FAILURE;
    }
    if !bamdata.is_null() {
        samples_qtask_ordered_c_121_cleanup_bamstorage(bamdata.cast());
    }

    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut bamcache.lock);
    while !bamcache.list.is_null() {
        let tmp = bamcache.list;
        bamcache.list = (*bamcache.list).next;
        samples_qtask_ordered_c_121_cleanup_bamstorage(tmp.cast());
    }
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut bamcache.lock);
    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut bamcache.lock);

    if !pool.is_null() {
        thread_pool::hts_tpool_destroy(pool);
    }
    ret
}
