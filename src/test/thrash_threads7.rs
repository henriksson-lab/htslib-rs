// original: job (htslib/test/thrash_threads7.c:43)
pub unsafe extern "C" fn test_thrash_threads7_c_43_job(v: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    let usecs = v.cast::<u32>();
    crate::htslib_rs::hts::hts_usleep(*usecs as i64);
    std::ptr::null_mut()
}

// original: main (htslib/test/thrash_threads7.c:49)
pub unsafe fn test_thrash_threads7_c_49_main(_argc: i32, _argv: *mut *mut u8) -> i32 {
    let run_for_secs: libc::time_t = 120;
    let num_threads = 8;
    let num_jobs = 8;
    let mut count = 0;
    let n_proc = 8;
    let mut end: libc::timeval = std::mem::zeroed();
    let mut now: libc::timeval = std::mem::zeroed();
    let mut q = [std::ptr::null_mut(); 8];

    let p = crate::htslib_rs::thread_pool::hts_tpool_init(num_threads);
    if p.is_null() {
        libc::perror(c"hts_tpool_init".as_ptr());
        libc::exit(libc::EXIT_FAILURE);
    }

    for slot in q.iter_mut().take(n_proc) {
        *slot = crate::htslib_rs::thread_pool::hts_tpool_process_init(p, 10, 1);
        if slot.is_null() {
            libc::perror(c"hts_tpool_process_init".as_ptr());
            libc::exit(libc::EXIT_FAILURE);
        }
    }

    if libc::gettimeofday(&mut end, std::ptr::null_mut()) != 0 {
        libc::perror(c"gettimeofday".as_ptr());
        libc::exit(libc::EXIT_FAILURE);
    }
    end.tv_sec += run_for_secs;

    loop {
        let qnum = (libc::rand() % n_proc as i32) as usize;
        let mut t = vec![0_u32; num_jobs];

        count += 1;
        if (count & 15) == 1 {
            eprint!("\r{} ", count);
            libc::alarm(10);
        }

        for i in 0..num_jobs {
            t[i] = 1000;
            if crate::htslib_rs::thread_pool::hts_tpool_dispatch(
                p,
                q[qnum],
                Some(test_thrash_threads7_c_43_job),
                (&mut t[i] as *mut u32).cast(),
            ) < 0
            {
                libc::perror(c"hts_tpool_dispatch".as_ptr());
                libc::exit(libc::EXIT_FAILURE);
            }
        }

        crate::htslib_rs::thread_pool::hts_tpool_process_flush(&mut *q[qnum]);
        crate::htslib_rs::thread_pool::hts_tpool_process_destroy(q[qnum]);
        drop(t);
        q[qnum] = crate::htslib_rs::thread_pool::hts_tpool_process_init(p, 10, 1);
        if q[qnum].is_null() {
            libc::perror(c"hts_tpool_process_init".as_ptr());
            libc::exit(libc::EXIT_FAILURE);
        }

        if libc::gettimeofday(&mut now, std::ptr::null_mut()) != 0 {
            libc::perror(c"gettimeofday".as_ptr());
            libc::exit(libc::EXIT_FAILURE);
        }
        if now.tv_sec > end.tv_sec || (now.tv_sec == end.tv_sec && now.tv_usec >= end.tv_usec) {
            break;
        }
    }

    for queue in q.iter().take(n_proc) {
        crate::htslib_rs::thread_pool::hts_tpool_process_flush(&mut **queue);
        crate::htslib_rs::thread_pool::hts_tpool_process_destroy(*queue);
    }
    crate::htslib_rs::thread_pool::hts_tpool_destroy(p);
    eprint!("\n");

    libc::EXIT_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_thrash_threads7_recreates_process_queues() {
        unsafe {
            let pool = crate::htslib_rs::thread_pool::hts_tpool_init(3);
            assert!(!pool.is_null());

            let mut queues = [std::ptr::null_mut(); 3];
            for queue in queues.iter_mut() {
                *queue = crate::htslib_rs::thread_pool::hts_tpool_process_init(pool, 4, 1);
                assert!(!queue.is_null());
            }

            let queue_order = [0_usize, 2, 1, 0, 2, 1];
            let mut delays = [[100_u32; 4]; 6];
            for (cycle, &queue_index) in queue_order.iter().enumerate() {
                for delay in delays[cycle].iter_mut() {
                    assert_eq!(
                        crate::htslib_rs::thread_pool::hts_tpool_dispatch(
                            pool,
                            queues[queue_index],
                            Some(test_thrash_threads7_c_43_job),
                            delay as *mut u32 as *mut std::ffi::c_void,
                        ),
                        0
                    );
                }

                assert_eq!(
                    crate::htslib_rs::thread_pool::hts_tpool_process_flush(
                        &mut *queues[queue_index]
                    ),
                    0
                );
                assert_eq!(
                    crate::htslib_rs::thread_pool::hts_tpool_process_empty(
                        &mut *queues[queue_index]
                    ),
                    1
                );
                crate::htslib_rs::thread_pool::hts_tpool_process_destroy(queues[queue_index]);

                queues[queue_index] =
                    crate::htslib_rs::thread_pool::hts_tpool_process_init(pool, 4, 1);
                assert!(!queues[queue_index].is_null());
            }

            for queue in queues {
                assert_eq!(
                    crate::htslib_rs::thread_pool::hts_tpool_process_flush(&mut *queue),
                    0
                );
                crate::htslib_rs::thread_pool::hts_tpool_process_destroy(queue);
            }
            crate::htslib_rs::thread_pool::hts_tpool_destroy(pool);
        }
    }
}
