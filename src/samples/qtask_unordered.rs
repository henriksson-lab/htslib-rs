use crate::htslib_rs::{hts, sam, thread_pool};
use std::io::Write;

#[repr(C)]
struct QTaskUnorderedData {
    count: i32,
    maxsize: i32,
    bamarray: *mut *mut sam::bam1_t,
    cache: *mut QTaskUnorderedDataCache,
    bases: *mut (),
    next: *mut QTaskUnorderedData,
}

#[repr(C)]
struct QTaskUnorderedDataCache {
    lock: crate::htslib_rs::c_compat::pthread_mutex_t,
    list: *mut QTaskUnorderedData,
}

// original: print_usage (htslib/samples/qtask_unordered.c:62)
pub unsafe fn samples_qtask_unordered_c_62_print_usage() {
    let mut __out = std::io::stdout();
    write!(
        __out,
        "Usage: qtask_unordered infile threadcount [chunksize]\nShows the base counts and calculates GC ratio - sum(G,C) / sum(A,T,C,G)\nchunksize [4096] sets the number of alignments clubbed together to process.\n"
    )
    .unwrap();
    __out.flush().unwrap();
}

// original: getbamstorage (htslib/samples/qtask_unordered.c:76)
pub unsafe extern "C" fn samples_qtask_unordered_c_76_getbamstorage(
    chunk: i32,
    bases: *mut (),
    bamcache: *mut (),
) -> *mut () {
    let bamcache = bamcache.cast::<QTaskUnorderedDataCache>();
    if bamcache.is_null() || bases.is_null() {
        return std::ptr::null_mut();
    }
    if crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*bamcache).lock) != 0 {
        return std::ptr::null_mut();
    }

    let bamdata: *mut QTaskUnorderedData;
    if !(*bamcache).list.is_null() {
        bamdata = (*bamcache).list;
        (*bamcache).list = (*bamdata).next;
        (*bamdata).next = std::ptr::null_mut();
        (*bamdata).count = 0;
        (*bamdata).bases = bases;
        (*bamdata).cache = bamcache;
    } else {
        bamdata = Box::into_raw(Box::new(std::mem::zeroed::<QTaskUnorderedData>()));
        {
            let mut array = Vec::<*mut sam::bam1_t>::with_capacity(chunk as usize);
            for _ in 0..chunk {
                array.push(sam::bam_init1());
            }
            (*bamdata).bamarray = array.leak().as_mut_ptr();
            (*bamdata).maxsize = chunk;
            (*bamdata).count = 0;
            (*bamdata).next = std::ptr::null_mut();
            (*bamdata).bases = bases;
            (*bamdata).cache = bamcache;
        }
    }

    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*bamcache).lock);
    bamdata.cast()
}

// original: cleanup_bamstorage (htslib/samples/qtask_unordered.c:128)
pub unsafe extern "C" fn samples_qtask_unordered_c_128_cleanup_bamstorage(
    arg: *mut std::ffi::c_void,
) {
    let bamdata = arg.cast::<QTaskUnorderedData>();
    if bamdata.is_null() {
        return;
    }
    if !(*bamdata).bamarray.is_null() {
        let array = Vec::from_raw_parts(
            (*bamdata).bamarray,
            (*bamdata).maxsize as usize,
            (*bamdata).maxsize as usize,
        );
        for bam in &array {
            sam::bam_destroy1(*bam);
        }
        drop(array);
    }
    drop(Box::from_raw(bamdata));
}

// original: thread_unordered_proc (htslib/samples/qtask_unordered.c:148)
pub unsafe extern "C" fn samples_qtask_unordered_c_148_thread_unordered_proc(
    args: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
    let bamdata = args.cast::<QTaskUnorderedData>();
    let mut counts = [0_u64; 16];

    for i in 0..(*bamdata).count {
        let bam = *(*bamdata).bamarray.add(i as usize);
        let seq = sam::bam_get_seq(bam);
        for pos in 0..(*bam).core.l_qseq as usize {
            counts[sam::bam_seqi(seq, pos) as usize] += 1;
        }
    }

    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*(*bamdata).cache).lock);
    let bases = (*bamdata).bases.cast::<u64>();
    for (i, count) in counts.iter().enumerate() {
        *bases.add(i) += *count;
    }
    (*bamdata).next = (*(*bamdata).cache).list;
    (*(*bamdata).cache).list = bamdata;
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*(*bamdata).cache).lock);

    std::ptr::null_mut()
}

// original: main (htslib/samples/qtask_unordered.c:181)
pub unsafe fn samples_qtask_unordered_c_181_main(argc: i32, argv: *mut *mut u8) -> i32 {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut __out = std::io::stdout();
    let mut ret = 1;
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
    crate::htslib_rs::c_compat::pthread_mutex_init(&mut bamcache.lock, std::ptr::null());

    if argc != 3 && argc != 4 {
        samples_qtask_unordered_c_62_print_usage();
    } else {
        let inname = *argv.add(1);
        let mut cnt: i32 = {
            let arg = std::ffi::CStr::from_ptr((*argv.add(2)).cast());
            arg.to_bytes()
                .iter()
                .take_while(|b| b.is_ascii_digit())
                .fold(0i32, |acc, b| acc * 10 + i32::from(b - b'0'))
        };
        let mut chunk: i32 = if argc == 4 {
            let arg = std::ffi::CStr::from_ptr((*argv.add(3)).cast());
            arg.to_bytes()
                .iter()
                .take_while(|b| b.is_ascii_digit())
                .fold(0i32, |acc, b| acc * 10 + i32::from(b - b'0'))
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
            eprint!("Failed to create thread pool\n");
        } else {
            tpool.pool = pool;
            queue = thread_pool::hts_tpool_process_init(pool, cnt * 2, 1);
            if queue.is_null() {
                eprint!("Failed to create queue\n");
            } else {
                infile = hts::hts_open(inname.cast(), c"r".as_ptr());
                if infile.is_null() {
                    eprint!(
                        "Could not open {}\n",
                        String::from_utf8_lossy(std::ffi::CStr::from_ptr(inname.cast()).to_bytes())
                    );
                } else if hts::hts_set_thread_pool(infile, &mut tpool) < 0 {
                    eprint!("Failed to set threads to i/o files\n");
                } else {
                    in_samhdr = sam::sam_hdr_read(infile);
                    if in_samhdr.is_null() {
                        eprint!("Failed to read header from file!\n");
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
                                eprint!("Failed to allocate memory\n");
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
                                    eprint!("Failed to schedule processing\n");
                                    break;
                                }
                                bamdata = std::ptr::null_mut();
                            } else {
                                eprint!("Error in reading data\n");
                                break;
                            }
                        }

                        if c == -1 {
                            if thread_pool::hts_tpool_process_flush(&mut *queue) == -1 {
                                eprint!("Failed to flush queue\n");
                            } else {
                                write!(
                                    __out,
                                    "GCratio: {:.6}\nBase counts:\n",
                                    (gccount[2] + gccount[4]) as f64
                                        / (gccount[1] + gccount[8] + gccount[2] + gccount[4])
                                            as f64,
                                )
                                .unwrap();
                                for (i, count) in gccount.iter().enumerate() {
                                    write!(__out, "{}: {}\n", SEQ_NT16_STR[i] as char, *count)
                                        .unwrap();
                                }
                                ret = 0;
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
        ret = 1;
    }
    if !bamdata.is_null() {
        samples_qtask_unordered_c_128_cleanup_bamstorage(bamdata.cast());
    }

    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut bamcache.lock);
    while !bamcache.list.is_null() {
        let tmp = bamcache.list;
        bamcache.list = (*bamcache.list).next;
        samples_qtask_unordered_c_128_cleanup_bamstorage(tmp.cast());
    }
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut bamcache.lock);
    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut bamcache.lock);

    if !pool.is_null() {
        thread_pool::hts_tpool_destroy(pool);
    }
    __out.flush().unwrap();
    ret
}
