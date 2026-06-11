use crate::htslib_rs::{hts, sam, thread_pool};

#[repr(C)]
struct QTaskOrderedData {
    count: i32,
    maxsize: i32,
    bamarray: *mut *mut sam::bam1_t,
    next: *mut QTaskOrderedData,
}

#[repr(C)]
struct QTaskOrderedDataCache {
    // RECONVERGE: shared with thread pool — pthread mutex kept as libc type
    lock: libc::pthread_mutex_t,
    list: *mut QTaskOrderedData,
}

#[repr(C)]
struct QTaskOrderedWrite {
    outfile: *mut hts::htsFile,
    samhdr: *mut sam::sam_hdr_t,
    queue: *mut thread_pool::hts_tpool_process,
    cache: *mut QTaskOrderedDataCache,
    result: i32,
}

// original: print_usage (htslib/samples/qtask_ordered.c:61)
pub unsafe fn samples_qtask_ordered_c_61_print_usage() {
    eprintln!(
        "Usage: qtask_ordered infile threadcount outdir [chunksize]\nCalculates GC ratio - sum(G,C) / sum(A,T,C,G) - and adds to each alignment\nas xr:f aux tag. Output is saved in outdir.\nchunksize [4096] sets the number of alignments clubbed together to process."
    );
}

// original: getbamstorage (htslib/samples/qtask_ordered.c:75)
pub unsafe extern "C" fn samples_qtask_ordered_c_75_getbamstorage(
    chunk: i32,
    bamcache: *mut (),
) -> *mut () {
    let bamcache = bamcache.cast::<QTaskOrderedDataCache>();
    if bamcache.is_null() {
        return std::ptr::null_mut();
    }
    // RECONVERGE: shared thread-pool mutex — libc pthread call
    if libc::pthread_mutex_lock(&mut (*bamcache).lock) != 0 {
        return std::ptr::null_mut();
    }

    let bamdata: *mut QTaskOrderedData;
    if !(*bamcache).list.is_null() {
        bamdata = (*bamcache).list;
        (*bamcache).list = (*bamdata).next;
        (*bamdata).next = std::ptr::null_mut();
        (*bamdata).count = 0;
    } else {
        bamdata = Box::into_raw(Box::new(QTaskOrderedData {
            count: 0,
            maxsize: 0,
            bamarray: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
        }));
        if !bamdata.is_null() {
            let mut array: Vec<*mut sam::bam1_t> = Vec::with_capacity(chunk as usize);
            for _ in 0..chunk {
                array.push(sam::bam_init1());
            }
            (*bamdata).bamarray = array.leak().as_mut_ptr();
            (*bamdata).maxsize = chunk;
            (*bamdata).count = 0;
            (*bamdata).next = std::ptr::null_mut();
        }
    }

    // RECONVERGE: shared thread-pool mutex — libc pthread call
    libc::pthread_mutex_unlock(&mut (*bamcache).lock);
    bamdata.cast()
}

// original: cleanup_bamstorage (htslib/samples/qtask_ordered.c:121)
pub unsafe extern "C" fn samples_qtask_ordered_c_121_cleanup_bamstorage(
    arg: *mut std::ffi::c_void,
) {
    let bamdata = arg.cast::<QTaskOrderedData>();
    if bamdata.is_null() {
        return;
    }
    if !(*bamdata).bamarray.is_null() {
        let array = Vec::from_raw_parts(
            (*bamdata).bamarray,
            (*bamdata).maxsize as usize,
            (*bamdata).maxsize as usize,
        );
        for &bam in &array {
            sam::bam_destroy1(bam);
        }
        drop(array);
    }
    drop(Box::from_raw(bamdata));
}

// original: thread_ordered_proc (htslib/samples/qtask_ordered.c:143)
pub unsafe extern "C" fn samples_qtask_ordered_c_143_thread_ordered_proc(
    args: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
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
            b"xr\0".as_ptr().cast(),
            b'f' as u8,
            std::mem::size_of::<f32>() as i32,
            (&gcratio as *const f32).cast::<u8>(),
        ) < 0
        {
            eprintln!(
                "Failed to add aux tag xr, errno: {}",
                *libc::__errno_location()
            );
            break;
        }
    }
    bamdata.cast()
}

// original: threadfn_orderedwrite (htslib/samples/qtask_ordered.c:176)
pub extern "C" fn samples_qtask_ordered_c_176_threadfn_orderedwrite(
    args: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
    unsafe {
        let tdata = args.cast::<QTaskOrderedWrite>();
        (*tdata).result = 0;

        while (*tdata).result == 0 {
            let r = thread_pool::hts_tpool_next_result_wait((*tdata).queue);
            if r.is_null() {
                break;
            }
            let bamdata = thread_pool::hts_tpool_result_data(&mut *r).cast::<QTaskOrderedData>();
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
                    eprintln!("Failed to write output data");
                    (*tdata).result = -1;
                    break;
                }
            }
            thread_pool::hts_tpool_delete_result(r, 0);

            // RECONVERGE: shared thread-pool mutex — libc pthread calls
            libc::pthread_mutex_lock(&mut (*(*tdata).cache).lock);
            (*bamdata).next = (*(*tdata).cache).list;
            (*(*tdata).cache).list = bamdata;
            libc::pthread_mutex_unlock(&mut (*(*tdata).cache).lock);
        }

        thread_pool::hts_tpool_process_shutdown((*tdata).queue);
    }
    std::ptr::null_mut()
}

// original: main (htslib/samples/qtask_ordered.c:223)
pub unsafe fn samples_qtask_ordered_c_223_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut ret = 1;
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
    // RECONVERGE: shared with thread pool — pthread thread handle kept as libc type
    let mut thread: libc::pthread_t = std::mem::zeroed();
    let mut twritedata: QTaskOrderedWrite = std::mem::zeroed();
    // RECONVERGE: shared thread-pool mutex — libc pthread call
    libc::pthread_mutex_init(&mut bamcache.lock, std::ptr::null());

    if argc != 4 && argc != 5 {
        samples_qtask_ordered_c_61_print_usage();
    } else {
        let inname = *argv.add(1);
        let mut cnt = libc::atoi(*argv.add(2).cast());
        let outdir = *argv.add(3);
        let mut chunk = if argc == 5 {
            libc::atoi(*argv.add(4).cast())
        } else {
            0
        };
        if cnt < 1 {
            cnt = 1;
        }
        if chunk < 1 {
            chunk = 4096;
        }

        // Build "<outdir>/out.bam" as a NUL-terminated byte buffer to pass to the raw C-ABI hts_open.
        let outdir_bytes = std::ffi::CStr::from_ptr(outdir.cast()).to_bytes();
        let mut file: Vec<u8> = Vec::new();
        file.extend_from_slice(outdir_bytes);
        file.extend_from_slice(b"/out.bam\0");
        {
            pool = thread_pool::hts_tpool_init(cnt);
            if pool.is_null() {
                eprintln!("Failed to create thread pool");
            } else {
                tpool.pool = pool;
                queue = thread_pool::hts_tpool_process_init(pool, cnt * 2, 0);
                if queue.is_null() {
                    eprintln!("Failed to create queue");
                } else {
                    infile = hts::hts_open(inname.cast(), b"r\0".as_ptr().cast());
                    if infile.is_null() {
                        eprintln!(
                            "Could not open {}",
                            String::from_utf8_lossy(
                                std::ffi::CStr::from_ptr(inname.cast()).to_bytes()
                            )
                        );
                    } else {
                        outfile = hts::hts_open(file.as_ptr().cast(), b"wb\0".as_ptr().cast());
                        if outfile.is_null() {
                            eprintln!("Could not open output file");
                        } else if hts::hts_set_thread_pool(infile, &mut tpool) < 0
                            || hts::hts_set_thread_pool(outfile, &mut tpool) < 0
                        {
                            eprintln!("Failed to set threads to i/o files");
                        } else {
                            in_samhdr = sam::sam_hdr_read(infile);
                            if in_samhdr.is_null() {
                                eprintln!("Failed to read header from file!");
                            } else if sam::sam_hdr_write(outfile, in_samhdr) == -1 {
                                eprintln!("Failed to write header");
                            } else {
                                twritedata.outfile = outfile;
                                twritedata.samhdr = in_samhdr;
                                twritedata.result = 0;
                                twritedata.queue = queue;
                                twritedata.cache = &mut bamcache;
                                // RECONVERGE: shared thread-pool thread — libc pthread call
                                if libc::pthread_create(
                                    &mut thread,
                                    std::ptr::null(),
                                    samples_qtask_ordered_c_176_threadfn_orderedwrite,
                                    (&mut twritedata as *mut QTaskOrderedWrite).cast(),
                                ) != 0
                                {
                                    eprintln!("Failed to create writer thread");
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
                                            eprintln!("Failed to allocate memory");
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
                                                eprintln!("Failed to schedule processing");
                                                dispatch_failed = true;
                                                break;
                                            }
                                            bamdata = std::ptr::null_mut();
                                        } else {
                                            eprintln!("Error in reading data");
                                            break;
                                        }
                                    }
                                    if !dispatch_failed {
                                        ret = 0;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            drop(file);
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
                eprintln!("Failed to schedule sentinel job");
                ret = 1;
            }
        } else {
            thread_pool::hts_tpool_process_shutdown(queue);
        }
    }

    if started_thread != 0 {
        // RECONVERGE: shared thread-pool thread — libc pthread call
        libc::pthread_join(thread, std::ptr::null_mut());
        if twritedata.result != 0 {
            ret = 1;
        }
    }

    if !queue.is_null() {
        thread_pool::hts_tpool_process_destroy(queue);
    }
    if !in_samhdr.is_null() {
        sam::sam_hdr_destroy(in_samhdr);
    }
    if !infile.is_null() && hts::hts_close(infile) != 0 {
        ret = 1;
    }
    if !outfile.is_null() && hts::hts_close(outfile) != 0 {
        ret = 1;
    }
    if !bamdata.is_null() {
        samples_qtask_ordered_c_121_cleanup_bamstorage(bamdata.cast());
    }

    // RECONVERGE: shared thread-pool mutex — libc pthread calls
    libc::pthread_mutex_lock(&mut bamcache.lock);
    while !bamcache.list.is_null() {
        let tmp = bamcache.list;
        bamcache.list = (*bamcache.list).next;
        samples_qtask_ordered_c_121_cleanup_bamstorage(tmp.cast());
    }
    libc::pthread_mutex_unlock(&mut bamcache.lock);
    libc::pthread_mutex_destroy(&mut bamcache.lock);

    if !pool.is_null() {
        thread_pool::hts_tpool_destroy(pool);
    }
    ret
}
