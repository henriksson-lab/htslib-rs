use std::ffi::{c_int, c_void};

pub type hts_tpool = hts_sys::hts_tpool;
pub type hts_tpool_process = hts_sys::hts_tpool_process;
pub type hts_tpool_result = hts_sys::hts_tpool_result;

pub type hts_tpool_worker = Option<unsafe extern "C" fn(arg: *mut c_void) -> *mut c_void>;
pub type hts_tpool_cleanup = Option<unsafe extern "C" fn(arg: *mut c_void)>;

unsafe extern "C" {
    #[link_name = "hts_tpool_worker_id"]
    fn htslib_hts_tpool_worker_id(p: *mut hts_tpool) -> c_int;
    #[link_name = "hts_tpool_process_is_shutdown"]
    fn htslib_hts_tpool_process_is_shutdown(q: *mut hts_tpool_process) -> c_int;
}

pub unsafe fn hts_tpool_worker_id(p: *mut hts_tpool) -> c_int {
    unsafe { htslib_hts_tpool_worker_id(p) }
}

pub unsafe fn hts_tpool_init(n: c_int) -> *mut hts_tpool {
    hts_sys::hts_tpool_init(n)
}

pub unsafe fn hts_tpool_size(p: *mut hts_tpool) -> c_int {
    hts_sys::hts_tpool_size(p)
}

pub unsafe fn hts_tpool_dispatch(
    p: *mut hts_tpool,
    q: *mut hts_tpool_process,
    func: hts_tpool_worker,
    arg: *mut c_void,
) -> c_int {
    hts_sys::hts_tpool_dispatch(p, q, func, arg)
}

pub unsafe fn hts_tpool_dispatch2(
    p: *mut hts_tpool,
    q: *mut hts_tpool_process,
    func: hts_tpool_worker,
    arg: *mut c_void,
    nonblock: c_int,
) -> c_int {
    hts_sys::hts_tpool_dispatch2(p, q, func, arg, nonblock)
}

pub unsafe fn hts_tpool_dispatch3(
    p: *mut hts_tpool,
    q: *mut hts_tpool_process,
    exec_func: hts_tpool_worker,
    arg: *mut c_void,
    job_cleanup: hts_tpool_cleanup,
    result_cleanup: hts_tpool_cleanup,
    nonblock: c_int,
) -> c_int {
    hts_sys::hts_tpool_dispatch3(p, q, exec_func, arg, job_cleanup, result_cleanup, nonblock)
}

pub unsafe fn hts_tpool_wake_dispatch(q: *mut hts_tpool_process) {
    hts_sys::hts_tpool_wake_dispatch(q)
}

pub unsafe fn hts_tpool_process_flush(q: *mut hts_tpool_process) -> c_int {
    hts_sys::hts_tpool_process_flush(q)
}

pub unsafe fn hts_tpool_process_reset(q: *mut hts_tpool_process, free_results: c_int) -> c_int {
    hts_sys::hts_tpool_process_reset(q, free_results)
}

pub unsafe fn hts_tpool_process_qsize(q: *mut hts_tpool_process) -> c_int {
    hts_sys::hts_tpool_process_qsize(q)
}

pub unsafe fn hts_tpool_destroy(p: *mut hts_tpool) {
    hts_sys::hts_tpool_destroy(p)
}

pub unsafe fn hts_tpool_kill(p: *mut hts_tpool) {
    hts_sys::hts_tpool_kill(p)
}

pub unsafe fn hts_tpool_next_result(q: *mut hts_tpool_process) -> *mut hts_tpool_result {
    hts_sys::hts_tpool_next_result(q)
}

pub unsafe fn hts_tpool_next_result_wait(q: *mut hts_tpool_process) -> *mut hts_tpool_result {
    hts_sys::hts_tpool_next_result_wait(q)
}

pub unsafe fn hts_tpool_delete_result(r: *mut hts_tpool_result, free_data: c_int) {
    hts_sys::hts_tpool_delete_result(r, free_data)
}

pub unsafe fn hts_tpool_result_data(r: *mut hts_tpool_result) -> *mut c_void {
    hts_sys::hts_tpool_result_data(r)
}

pub unsafe fn hts_tpool_process_init(
    p: *mut hts_tpool,
    qsize: c_int,
    in_only: c_int,
) -> *mut hts_tpool_process {
    hts_sys::hts_tpool_process_init(p, qsize, in_only)
}

pub unsafe fn hts_tpool_process_destroy(q: *mut hts_tpool_process) {
    hts_sys::hts_tpool_process_destroy(q)
}

pub unsafe fn hts_tpool_process_empty(q: *mut hts_tpool_process) -> c_int {
    hts_sys::hts_tpool_process_empty(q)
}

pub unsafe fn hts_tpool_process_len(q: *mut hts_tpool_process) -> c_int {
    hts_sys::hts_tpool_process_len(q)
}

pub unsafe fn hts_tpool_process_sz(q: *mut hts_tpool_process) -> c_int {
    hts_sys::hts_tpool_process_sz(q)
}

pub unsafe fn hts_tpool_process_shutdown(q: *mut hts_tpool_process) {
    hts_sys::hts_tpool_process_shutdown(q)
}

pub unsafe fn hts_tpool_process_is_shutdown(q: *mut hts_tpool_process) -> c_int {
    unsafe { htslib_hts_tpool_process_is_shutdown(q) }
}

pub unsafe fn hts_tpool_process_attach(p: *mut hts_tpool, q: *mut hts_tpool_process) {
    hts_sys::hts_tpool_process_attach(p, q)
}

pub unsafe fn hts_tpool_process_detach(p: *mut hts_tpool, q: *mut hts_tpool_process) {
    hts_sys::hts_tpool_process_detach(p, q)
}

pub unsafe fn hts_tpool_process_ref_incr(q: *mut hts_tpool_process) {
    hts_sys::hts_tpool_process_ref_incr(q)
}

pub unsafe fn hts_tpool_process_ref_decr(q: *mut hts_tpool_process) {
    hts_sys::hts_tpool_process_ref_decr(q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_pool_wrappers_create_process_and_report_sizes() {
        unsafe {
            let pool = hts_tpool_init(1);
            assert!(!pool.is_null());
            assert_eq!(hts_tpool_size(pool), 1);

            let queue = hts_tpool_process_init(pool, 2, 0);
            assert!(!queue.is_null());
            assert_eq!(hts_tpool_process_empty(queue), 1);
            assert_eq!(hts_tpool_process_len(queue), 0);
            assert_eq!(hts_tpool_process_qsize(queue), 2);

            hts_tpool_process_destroy(queue);
            hts_tpool_destroy(pool);
        }
    }
}
