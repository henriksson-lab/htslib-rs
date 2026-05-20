use std::ffi::{c_int, c_void};
use std::mem;
use std::ptr;

use crate::htslib_mini_rs::c_compat;

// original: hts_tpool (htslib/thread_pool_internal.h:136)
pub type hts_tpool = hts_sys::hts_tpool;
// original: hts_tpool_process (htslib/thread_pool_internal.h:100)
pub type hts_tpool_process = hts_sys::hts_tpool_process;
// original: hts_tpool_result (htslib/thread_pool_internal.h:71)
pub type hts_tpool_result = hts_sys::hts_tpool_result;

pub type hts_tpool_worker = Option<unsafe extern "C" fn(arg: *mut c_void) -> *mut c_void>;
pub type hts_tpool_cleanup = Option<unsafe extern "C" fn(arg: *mut c_void)>;

// original: HTS_MIN_THREAD_STACK (htslib/thread_pool.c:44)
const HTS_MIN_THREAD_STACK: usize = 3 * 1024 * 1024;

// original: hts_tpool_job (htslib/thread_pool_internal.h:56)
#[repr(C)]
struct HtsTpoolJob {
    func: hts_tpool_worker,
    arg: *mut c_void,
    job_cleanup: hts_tpool_cleanup,
    result_cleanup: hts_tpool_cleanup,
    next: *mut HtsTpoolJob,
    p: *mut HtsTpool,
    q: *mut HtsTpoolProcess,
    serial: u64,
}

// original: hts_tpool_result (htslib/thread_pool_internal.h:71)
#[repr(C)]
struct HtsTpoolResult {
    next: *mut HtsTpoolResult,
    result_cleanup: hts_tpool_cleanup,
    serial: u64,
    data: *mut c_void,
}

// original: hts_tpool_worker (htslib/thread_pool_internal.h:83)
#[repr(C)]
struct HtsTpoolWorker {
    p: *mut HtsTpool,
    idx: c_int,
    tid: libc::pthread_t,
    pending_c: libc::pthread_cond_t,
}

// original: hts_tpool_process (htslib/thread_pool_internal.h:100)
#[repr(C)]
struct HtsTpoolProcess {
    p: *mut HtsTpool,
    input_head: *mut HtsTpoolJob,
    input_tail: *mut HtsTpoolJob,
    output_head: *mut HtsTpoolResult,
    output_tail: *mut HtsTpoolResult,
    qsize: c_int,
    next_serial: u64,
    curr_serial: u64,
    no_more_input: c_int,
    n_input: c_int,
    n_output: c_int,
    n_processing: c_int,
    shutdown: c_int,
    in_only: c_int,
    wake_dispatch: c_int,
    ref_count: c_int,
    output_avail_c: libc::pthread_cond_t,
    input_not_full_c: libc::pthread_cond_t,
    input_empty_c: libc::pthread_cond_t,
    none_processing_c: libc::pthread_cond_t,
    next: *mut HtsTpoolProcess,
    prev: *mut HtsTpoolProcess,
}

// original: hts_tpool (htslib/thread_pool_internal.h:136)
#[repr(C)]
struct HtsTpool {
    nwaiting: c_int,
    njobs: c_int,
    shutdown: c_int,
    q_head: *mut HtsTpoolProcess,
    tsize: c_int,
    t: *mut HtsTpoolWorker,
    t_stack: *mut c_int,
    t_stack_top: c_int,
    pool_m: libc::pthread_mutex_t,
    n_count: c_int,
    n_running: c_int,
    total_time: i64,
    wait_time: i64,
}

unsafe fn pool(p: *mut hts_tpool) -> *mut HtsTpool {
    p.cast()
}

unsafe fn process(q: *mut hts_tpool_process) -> *mut HtsTpoolProcess {
    q.cast()
}

unsafe fn result(r: *mut hts_tpool_result) -> *mut HtsTpoolResult {
    r.cast()
}

unsafe fn xmalloc<T>() -> *mut T {
    unsafe { libc::malloc(mem::size_of::<T>()).cast() }
}

unsafe fn xmalloc_array<T>(n: c_int) -> *mut T {
    let Ok(n) = usize::try_from(n) else {
        unsafe {
            *c_compat::__errno_location() = libc::ENOMEM;
        }
        return ptr::null_mut();
    };
    let Some(size) = n.checked_mul(mem::size_of::<T>()) else {
        unsafe {
            *c_compat::__errno_location() = libc::ENOMEM;
        }
        return ptr::null_mut();
    };
    unsafe { libc::malloc(size).cast() }
}

// original: hts_tpool_worker_id (htslib/thread_pool.c:56)
pub unsafe fn hts_tpool_worker_id(p: *mut hts_tpool) -> c_int {
    if p.is_null() {
        return -1;
    }
    let p = unsafe { pool(p) };
    let s = unsafe { libc::pthread_self() };
    for i in 0..unsafe { (*p).tsize } {
        if unsafe { libc::pthread_equal(s, (*(*p).t.add(i as usize)).tid) } != 0 {
            return i;
        }
    }
    -1
}

// original: hts_tpool_add_result (htslib/thread_pool.c:95)
unsafe fn hts_tpool_add_result(j: *mut HtsTpoolJob, data: *mut c_void) -> c_int {
    let q = unsafe { (*j).q };
    unsafe { libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m)) };

    unsafe {
        (*q).n_processing -= 1;
        if (*q).n_processing == 0 {
            libc::pthread_cond_signal(ptr::addr_of_mut!((*q).none_processing_c));
        }

        if (*q).in_only != 0 {
            libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));
            return 0;
        }
    }

    let r = unsafe { xmalloc::<HtsTpoolResult>() };
    if r.is_null() {
        unsafe { libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
        unsafe { hts_tpool_process_shutdown(q.cast()) };
        return -1;
    }

    unsafe {
        (*r).next = ptr::null_mut();
        (*r).data = data;
        (*r).result_cleanup = (*j).result_cleanup;
        (*r).serial = (*j).serial;

        (*q).n_output += 1;
        if !(*q).output_tail.is_null() {
            (*(*q).output_tail).next = r;
            (*q).output_tail = r;
        } else {
            (*q).output_head = r;
            (*q).output_tail = r;
        }

        assert!((*r).serial >= (*q).next_serial || (*q).next_serial == c_int::MAX as u64);
        if (*r).serial == (*q).next_serial {
            libc::pthread_cond_broadcast(ptr::addr_of_mut!((*q).output_avail_c));
        }

        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));
    }

    0
}

// original: hts_tpool_next_result_locked (htslib/thread_pool.c:149)
unsafe fn hts_tpool_next_result_locked(q: *mut HtsTpoolProcess) -> *mut HtsTpoolResult {
    if unsafe { (*q).shutdown } != 0 {
        return ptr::null_mut();
    }

    let mut last = ptr::null_mut();
    let mut r = unsafe { (*q).output_head };
    while !r.is_null() {
        if unsafe { (*r).serial == (*q).next_serial } {
            break;
        }
        last = r;
        r = unsafe { (*r).next };
    }

    if !r.is_null() {
        unsafe {
            if (*q).output_head == r {
                (*q).output_head = (*r).next;
            } else {
                (*last).next = (*r).next;
            }

            if (*q).output_tail == r {
                (*q).output_tail = last;
            }

            if (*q).output_head.is_null() {
                (*q).output_tail = ptr::null_mut();
            }

            (*q).next_serial += 1;
            (*q).n_output -= 1;

            if (*q).qsize != 0 && (*q).n_output < (*q).qsize {
                if (*q).n_input < (*q).qsize {
                    libc::pthread_cond_signal(ptr::addr_of_mut!((*q).input_not_full_c));
                }
                if (*q).shutdown == 0 {
                    wake_next_worker(q, 1);
                }
            }
        }
    }

    r
}

// original: hts_tpool_next_result (htslib/thread_pool.c:200)
pub unsafe fn hts_tpool_next_result(q: *mut hts_tpool_process) -> *mut hts_tpool_result {
    let q = unsafe { process(q) };
    unsafe { libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
    let r = unsafe { hts_tpool_next_result_locked(q) };
    unsafe { libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
    r.cast()
}

// original: hts_tpool_next_result_wait (htslib/thread_pool.c:224)
pub unsafe fn hts_tpool_next_result_wait(q: *mut hts_tpool_process) -> *mut hts_tpool_result {
    let q = unsafe { process(q) };
    unsafe { libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m)) };

    loop {
        let r = unsafe { hts_tpool_next_result_locked(q) };
        if !r.is_null() {
            unsafe { libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
            return r.cast();
        }

        let mut now: libc::timeval = unsafe { mem::zeroed() };
        let mut timeout: libc::timespec = unsafe { mem::zeroed() };
        unsafe {
            libc::gettimeofday(&mut now, ptr::null_mut());
            timeout.tv_sec = now.tv_sec + 10;
            timeout.tv_nsec = now.tv_usec * 1000;

            (*q).ref_count += 1;
            if (*q).shutdown != 0 {
                (*q).ref_count -= 1;
                let rc = (*q).ref_count;
                libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));
                if rc == 0 {
                    hts_tpool_process_destroy(q.cast());
                }
                return ptr::null_mut();
            }
            libc::pthread_cond_timedwait(
                ptr::addr_of_mut!((*q).output_avail_c),
                ptr::addr_of_mut!((*(*q).p).pool_m),
                &timeout,
            );
            (*q).ref_count -= 1;
        }
    }
}

// original: hts_tpool_process_empty (htslib/thread_pool.c:258)
pub unsafe fn hts_tpool_process_empty(q: *mut hts_tpool_process) -> c_int {
    let q = unsafe { process(q) };
    unsafe { libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
    let empty =
        unsafe { ((*q).n_input == 0 && (*q).n_processing == 0 && (*q).n_output == 0) as c_int };
    unsafe { libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
    empty
}

// original: hts_tpool_process_ref_incr (htslib/thread_pool.c:268)
pub unsafe fn hts_tpool_process_ref_incr(q: *mut hts_tpool_process) {
    let q = unsafe { process(q) };
    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m));
        (*q).ref_count += 1;
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));
    }
}

// original: hts_tpool_process_ref_decr (htslib/thread_pool.c:274)
pub unsafe fn hts_tpool_process_ref_decr(q: *mut hts_tpool_process) {
    let q = unsafe { process(q) };
    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m));
        (*q).ref_count -= 1;
        if (*q).ref_count <= 0 {
            libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));
            hts_tpool_process_destroy(q.cast());
            return;
        }
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));
    }
}

// original: hts_tpool_process_len (htslib/thread_pool.c:289)
pub unsafe fn hts_tpool_process_len(q: *mut hts_tpool_process) -> c_int {
    let q = unsafe { process(q) };
    unsafe { libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
    let len = unsafe { (*q).n_output };
    unsafe { libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
    len
}

// original: hts_tpool_process_sz (htslib/thread_pool.c:303)
pub unsafe fn hts_tpool_process_sz(q: *mut hts_tpool_process) -> c_int {
    let q = unsafe { process(q) };
    unsafe { libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
    let len = unsafe { (*q).n_output + (*q).n_input + (*q).n_processing };
    unsafe { libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
    len
}

// original: hts_tpool_process_shutdown_locked (htslib/thread_pool.c:319)
unsafe fn hts_tpool_process_shutdown_locked(q: *mut HtsTpoolProcess) {
    unsafe {
        (*q).shutdown = 1;
        libc::pthread_cond_broadcast(ptr::addr_of_mut!((*q).output_avail_c));
        libc::pthread_cond_broadcast(ptr::addr_of_mut!((*q).input_not_full_c));
        libc::pthread_cond_broadcast(ptr::addr_of_mut!((*q).input_empty_c));
        libc::pthread_cond_broadcast(ptr::addr_of_mut!((*q).none_processing_c));
    }
}

// original: hts_tpool_process_shutdown (htslib/thread_pool.c:327)
pub unsafe fn hts_tpool_process_shutdown(q: *mut hts_tpool_process) {
    let q = unsafe { process(q) };
    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m));
        hts_tpool_process_shutdown_locked(q);
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));
    }
}

// original: hts_tpool_process_is_shutdown (htslib/thread_pool.c:333)
pub unsafe fn hts_tpool_process_is_shutdown(q: *mut hts_tpool_process) -> c_int {
    let q = unsafe { process(q) };
    unsafe { libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
    let r = unsafe { (*q).shutdown };
    unsafe { libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m)) };
    r
}

// original: hts_tpool_delete_result (htslib/thread_pool.c:344)
pub unsafe fn hts_tpool_delete_result(r: *mut hts_tpool_result, free_data: c_int) {
    if r.is_null() {
        return;
    }
    let r = unsafe { result(r) };
    unsafe {
        if free_data != 0 && !(*r).data.is_null() {
            libc::free((*r).data);
        }
        libc::free(r.cast());
    }
}

// original: hts_tpool_result_data (htslib/thread_pool.c:358)
pub unsafe fn hts_tpool_result_data(r: *mut hts_tpool_result) -> *mut c_void {
    unsafe { (*result(r)).data }
}

// original: hts_tpool_process_init (htslib/thread_pool.c:372)
pub unsafe fn hts_tpool_process_init(
    p: *mut hts_tpool,
    qsize: c_int,
    in_only: c_int,
) -> *mut hts_tpool_process {
    let q = unsafe { xmalloc::<HtsTpoolProcess>() };
    if q.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        ptr::write_bytes(q, 0, 1);
        libc::pthread_cond_init(ptr::addr_of_mut!((*q).output_avail_c), ptr::null());
        libc::pthread_cond_init(ptr::addr_of_mut!((*q).input_not_full_c), ptr::null());
        libc::pthread_cond_init(ptr::addr_of_mut!((*q).input_empty_c), ptr::null());
        libc::pthread_cond_init(ptr::addr_of_mut!((*q).none_processing_c), ptr::null());

        (*q).p = pool(p);
        (*q).qsize = qsize;
        (*q).in_only = in_only;
        (*q).ref_count = 1;

        hts_tpool_process_attach(p, q.cast());
    }

    q.cast()
}

// original: hts_tpool_process_destroy (htslib/thread_pool.c:410)
pub unsafe fn hts_tpool_process_destroy(q: *mut hts_tpool_process) {
    if q.is_null() {
        return;
    }
    let q = unsafe { process(q) };

    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m));
        (*q).no_more_input = 1;
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));

        hts_tpool_process_reset(q.cast(), 0);

        libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m));
        hts_tpool_process_detach_locked((*q).p, q);
        hts_tpool_process_shutdown_locked(q);

        (*q).ref_count -= 1;
        if (*q).ref_count > 0 {
            libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));
            return;
        }

        libc::pthread_cond_destroy(ptr::addr_of_mut!((*q).output_avail_c));
        libc::pthread_cond_destroy(ptr::addr_of_mut!((*q).input_not_full_c));
        libc::pthread_cond_destroy(ptr::addr_of_mut!((*q).input_empty_c));
        libc::pthread_cond_destroy(ptr::addr_of_mut!((*q).none_processing_c));
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));
        libc::free(q.cast());
    }
}

// original: hts_tpool_process_attach (htslib/thread_pool.c:456)
pub unsafe fn hts_tpool_process_attach(p: *mut hts_tpool, q: *mut hts_tpool_process) {
    let p = unsafe { pool(p) };
    let q = unsafe { process(q) };
    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*p).pool_m));
        if !(*p).q_head.is_null() {
            (*q).next = (*p).q_head;
            (*q).prev = (*(*p).q_head).prev;
            (*(*(*p).q_head).prev).next = q;
            (*(*p).q_head).prev = q;
        } else {
            (*q).next = q;
            (*q).prev = q;
        }
        (*p).q_head = q;
        assert!(
            !(*p).q_head.is_null()
                && !(*(*p).q_head).prev.is_null()
                && !(*(*p).q_head).next.is_null()
        );
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
    }
}

// original: hts_tpool_process_detach_locked (htslib/thread_pool.c:472)
unsafe fn hts_tpool_process_detach_locked(p: *mut HtsTpool, q: *mut HtsTpoolProcess) {
    unsafe {
        if (*p).q_head.is_null() || (*q).prev.is_null() || (*q).next.is_null() {
            return;
        }

        let first = (*p).q_head;
        let mut curr = first;
        loop {
            if curr == q {
                (*(*q).next).prev = (*q).prev;
                (*(*q).prev).next = (*q).next;
                (*p).q_head = (*q).next;
                (*q).next = ptr::null_mut();
                (*q).prev = ptr::null_mut();

                if (*p).q_head == q {
                    (*p).q_head = ptr::null_mut();
                }
                break;
            }

            curr = (*curr).next;
            if curr == first {
                break;
            }
        }
    }
}

// original: hts_tpool_process_detach (htslib/thread_pool.c:495)
pub unsafe fn hts_tpool_process_detach(p: *mut hts_tpool, q: *mut hts_tpool_process) {
    let p = unsafe { pool(p) };
    let q = unsafe { process(q) };
    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*p).pool_m));
        hts_tpool_process_detach_locked(p, q);
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
    }
}

// original: tpool_worker (htslib/thread_pool.c:518)
extern "C" fn tpool_worker(arg: *mut c_void) -> *mut c_void {
    let w = arg.cast::<HtsTpoolWorker>();
    let p = unsafe { (*w).p };

    unsafe { libc::pthread_mutex_lock(ptr::addr_of_mut!((*p).pool_m)) };
    while unsafe { (*p).shutdown == 0 } {
        assert!(unsafe {
            (*p).q_head.is_null()
                || (!(*(*p).q_head).prev.is_null() && !(*(*p).q_head).next.is_null())
        });

        let mut work_to_do = 0;
        let first = unsafe { (*p).q_head };
        let mut q = first;
        if !q.is_null() {
            loop {
                unsafe {
                    if !(*q).input_head.is_null()
                        && (*q).qsize - (*q).n_output > (*q).n_processing
                        && (*q).shutdown == 0
                    {
                        work_to_do = 1;
                        break;
                    }
                    q = (*q).next;
                }
                if q.is_null() || q == first {
                    break;
                }
            }
        }

        if work_to_do == 0 {
            unsafe {
                (*p).nwaiting += 1;
                if (*p).t_stack_top == -1 || (*p).t_stack_top > (*w).idx {
                    (*p).t_stack_top = (*w).idx;
                }
                *(*p).t_stack.add((*w).idx as usize) = 1;
                libc::pthread_cond_wait(
                    ptr::addr_of_mut!((*w).pending_c),
                    ptr::addr_of_mut!((*p).pool_m),
                );
                *(*p).t_stack.add((*w).idx as usize) = 0;

                (*p).t_stack_top = -1;
                for i in 0..(*p).tsize {
                    if *(*p).t_stack.add(i as usize) != 0 {
                        (*p).t_stack_top = i;
                        break;
                    }
                }

                (*p).nwaiting -= 1;
            }
            continue;
        }

        unsafe {
            (*q).ref_count += 1;
            while !(*q).input_head.is_null() && (*q).qsize - (*q).n_output > (*q).n_processing {
                if (*p).shutdown != 0 {
                    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
                    return ptr::null_mut();
                }

                if (*q).shutdown != 0 {
                    break;
                }

                let j = (*q).input_head;
                assert!((*j).p == p);

                (*q).input_head = (*j).next;
                if (*q).input_head.is_null() {
                    (*q).input_tail = ptr::null_mut();
                }

                (*q).n_processing += 1;
                let old_n_input = (*q).n_input;
                (*q).n_input -= 1;
                if old_n_input >= (*q).qsize {
                    libc::pthread_cond_broadcast(ptr::addr_of_mut!((*q).input_not_full_c));
                }

                if (*q).n_input == 0 {
                    libc::pthread_cond_signal(ptr::addr_of_mut!((*q).input_empty_c));
                }

                (*p).njobs -= 1;

                libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));

                let data = (*j).func.unwrap_unchecked()((*j).arg);
                if hts_tpool_add_result(j, data) < 0 {
                    libc::pthread_mutex_lock(ptr::addr_of_mut!((*p).pool_m));
                    let first = (*p).q_head;
                    let mut q = first;
                    if !q.is_null() {
                        loop {
                            hts_tpool_process_shutdown_locked(q);
                            (*q).shutdown = 2;
                            q = (*q).next;
                            if q == first {
                                break;
                            }
                        }
                    }
                    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
                    return ptr::null_mut();
                }
                libc::free(j.cast());

                libc::pthread_mutex_lock(ptr::addr_of_mut!((*p).pool_m));
            }

            (*q).ref_count -= 1;
            if (*q).ref_count == 0 {
                hts_tpool_process_destroy(q.cast());
            } else if !(*p).q_head.is_null() {
                (*p).q_head = (*(*p).q_head).next;
            }
        }
    }

    unsafe { libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m)) };
    ptr::null_mut()
}

// original: wake_next_worker (htslib/thread_pool.c:654)
unsafe fn wake_next_worker(q: *mut HtsTpoolProcess, locked: c_int) {
    if q.is_null() {
        return;
    }
    let p = unsafe { (*q).p };
    unsafe {
        if locked == 0 {
            libc::pthread_mutex_lock(ptr::addr_of_mut!((*p).pool_m));
        }

        assert!(!(*q).prev.is_null() && !(*q).next.is_null());
        (*p).q_head = q;

        assert!((*p).njobs >= (*q).n_input);

        let sig = (*p).t_stack_top >= 0
            && (*p).njobs > (*p).tsize - (*p).nwaiting
            && (*q).n_processing < (*q).qsize - (*q).n_output;

        if sig {
            libc::pthread_cond_signal(ptr::addr_of_mut!(
                (*(*p).t.add((*p).t_stack_top as usize)).pending_c
            ));
        }

        if locked == 0 {
            libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
        }
    }
}

// original: hts_tpool_init (htslib/thread_pool.c:725)
pub unsafe fn hts_tpool_init(n: c_int) -> *mut hts_tpool {
    let mut t_idx = 0;
    let mut stack_size: usize = 0;
    let mut pattr: libc::pthread_attr_t = unsafe { mem::zeroed() };
    let mut pattr_init_done = 0;
    let p = unsafe { xmalloc::<HtsTpool>() };
    if p.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        ptr::write_bytes(p, 0, 1);
        (*p).tsize = n;
        (*p).t_stack_top = -1;
        (*p).t = xmalloc_array::<HtsTpoolWorker>(n);
        if (*p).t.is_null() {
            libc::free(p.cast());
            return ptr::null_mut();
        }
        (*p).t_stack = xmalloc_array::<c_int>(n);
        if (*p).t_stack.is_null() {
            libc::free((*p).t.cast());
            libc::free(p.cast());
            return ptr::null_mut();
        }

        let mut attr: libc::pthread_mutexattr_t = mem::zeroed();
        libc::pthread_mutexattr_init(&mut attr);
        libc::pthread_mutexattr_settype(&mut attr, libc::PTHREAD_MUTEX_RECURSIVE);
        libc::pthread_mutex_init(ptr::addr_of_mut!((*p).pool_m), &attr);
        libc::pthread_mutexattr_destroy(&mut attr);

        libc::pthread_mutex_lock(ptr::addr_of_mut!((*p).pool_m));

        if libc::pthread_attr_init(&mut pattr) != 0 {
            goto_cleanup(p, t_idx, &mut pattr, pattr_init_done);
            return ptr::null_mut();
        }
        pattr_init_done = 1;
        if libc::pthread_attr_getstacksize(&pattr, &mut stack_size) != 0 {
            goto_cleanup(p, t_idx, &mut pattr, pattr_init_done);
            return ptr::null_mut();
        }
        if stack_size < HTS_MIN_THREAD_STACK
            && libc::pthread_attr_setstacksize(&mut pattr, HTS_MIN_THREAD_STACK) != 0
        {
            goto_cleanup(p, t_idx, &mut pattr, pattr_init_done);
            return ptr::null_mut();
        }

        while t_idx < n {
            let w = (*p).t.add(t_idx as usize);
            ptr::write_bytes(w, 0, 1);
            *(*p).t_stack.add(t_idx as usize) = 0;
            (*w).p = p;
            (*w).idx = t_idx;
            libc::pthread_cond_init(ptr::addr_of_mut!((*w).pending_c), ptr::null());
            if libc::pthread_create(ptr::addr_of_mut!((*w).tid), &pattr, tpool_worker, w.cast())
                != 0
            {
                goto_cleanup(p, t_idx, &mut pattr, pattr_init_done);
                return ptr::null_mut();
            }
            t_idx += 1;
        }

        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
        libc::pthread_attr_destroy(&mut pattr);
    }

    p.cast()
}

unsafe fn goto_cleanup(
    p: *mut HtsTpool,
    t_idx: c_int,
    pattr: *mut libc::pthread_attr_t,
    pattr_init_done: c_int,
) {
    let save_errno = unsafe { *c_compat::__errno_location() };
    unsafe {
        (*p).shutdown = 1;
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
        for j in 0..t_idx {
            libc::pthread_join((*(*p).t.add(j as usize)).tid, ptr::null_mut());
            libc::pthread_cond_destroy(ptr::addr_of_mut!((*(*p).t.add(j as usize)).pending_c));
        }
        libc::pthread_mutex_destroy(ptr::addr_of_mut!((*p).pool_m));
        if pattr_init_done != 0 {
            libc::pthread_attr_destroy(pattr);
        }
        libc::free((*p).t_stack.cast());
        libc::free((*p).t.cast());
        libc::free(p.cast());
        *c_compat::__errno_location() = save_errno;
    }
}

// original: hts_tpool_size (htslib/thread_pool.c:819)
pub unsafe fn hts_tpool_size(p: *mut hts_tpool) -> c_int {
    unsafe { (*pool(p)).tsize }
}

// original: hts_tpool_dispatch (htslib/thread_pool.c:829)
pub unsafe fn hts_tpool_dispatch(
    p: *mut hts_tpool,
    q: *mut hts_tpool_process,
    func: hts_tpool_worker,
    arg: *mut c_void,
) -> c_int {
    unsafe { hts_tpool_dispatch3(p, q, func, arg, None, None, 0) }
}

// original: hts_tpool_dispatch2 (htslib/thread_pool.c:841)
pub unsafe fn hts_tpool_dispatch2(
    p: *mut hts_tpool,
    q: *mut hts_tpool_process,
    func: hts_tpool_worker,
    arg: *mut c_void,
    nonblock: c_int,
) -> c_int {
    unsafe { hts_tpool_dispatch3(p, q, func, arg, None, None, nonblock) }
}

// original: hts_tpool_dispatch3 (htslib/thread_pool.c:846)
pub unsafe fn hts_tpool_dispatch3(
    p: *mut hts_tpool,
    q: *mut hts_tpool_process,
    exec_func: hts_tpool_worker,
    arg: *mut c_void,
    job_cleanup: hts_tpool_cleanup,
    result_cleanup: hts_tpool_cleanup,
    nonblock: c_int,
) -> c_int {
    let p = unsafe { pool(p) };
    let q = unsafe { process(q) };
    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*p).pool_m));

        if ((*q).no_more_input != 0 || (*q).n_input >= (*q).qsize) && nonblock == 1 {
            libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
            *c_compat::__errno_location() = libc::EAGAIN;
            return -1;
        }

        let j = xmalloc::<HtsTpoolJob>();
        if j.is_null() {
            libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
            return -1;
        }

        (*j).func = exec_func;
        (*j).arg = arg;
        (*j).job_cleanup = job_cleanup;
        (*j).result_cleanup = result_cleanup;
        (*j).next = ptr::null_mut();
        (*j).p = p;
        (*j).q = q;
        (*j).serial = (*q).curr_serial;
        (*q).curr_serial += 1;

        if nonblock == 0 {
            while ((*q).no_more_input != 0 || (*q).n_input >= (*q).qsize)
                && (*q).shutdown == 0
                && (*q).wake_dispatch == 0
            {
                libc::pthread_cond_wait(
                    ptr::addr_of_mut!((*q).input_not_full_c),
                    ptr::addr_of_mut!((*(*q).p).pool_m),
                );
            }
            if (*q).no_more_input != 0 || (*q).shutdown != 0 {
                libc::free(j.cast());
                libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
                return -1;
            }
            if (*q).wake_dispatch != 0 {
                (*q).wake_dispatch = 0;
            }
        }

        (*p).njobs += 1;
        (*q).n_input += 1;

        if !(*q).input_tail.is_null() {
            (*(*q).input_tail).next = j;
            (*q).input_tail = j;
        } else {
            (*q).input_head = j;
            (*q).input_tail = j;
        }

        if (*q).shutdown == 0 {
            wake_next_worker(q, 1);
        }

        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
    }
    0
}

// original: hts_tpool_wake_dispatch (htslib/thread_pool.c:923)
pub unsafe fn hts_tpool_wake_dispatch(q: *mut hts_tpool_process) {
    let q = unsafe { process(q) };
    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m));
        (*q).wake_dispatch = 1;
        libc::pthread_cond_signal(ptr::addr_of_mut!((*q).input_not_full_c));
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));
    }
}

// original: hts_tpool_process_flush (htslib/thread_pool.c:941)
pub unsafe fn hts_tpool_process_flush(q: *mut hts_tpool_process) -> c_int {
    let q = unsafe { process(q) };
    let p = unsafe { (*q).p };
    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*p).pool_m));

        for i in 0..(*p).tsize {
            if *(*p).t_stack.add(i as usize) != 0 {
                libc::pthread_cond_signal(ptr::addr_of_mut!((*(*p).t.add(i as usize)).pending_c));
            }
        }

        if (*q).qsize < (*q).n_output + (*q).n_input + (*q).n_processing {
            (*q).qsize = (*q).n_output + (*q).n_input + (*q).n_processing;
        }

        if (*q).shutdown != 0 {
            while (*q).n_processing != 0 {
                libc::pthread_cond_wait(
                    ptr::addr_of_mut!((*q).none_processing_c),
                    ptr::addr_of_mut!((*p).pool_m),
                );
            }
        }

        while (*q).shutdown == 0 && ((*q).n_input != 0 || (*q).n_processing != 0) {
            let mut now: libc::timeval = mem::zeroed();
            let mut timeout: libc::timespec = mem::zeroed();

            while (*q).n_input != 0 && (*q).shutdown == 0 {
                libc::gettimeofday(&mut now, ptr::null_mut());
                timeout.tv_sec = now.tv_sec + 1;
                timeout.tv_nsec = now.tv_usec * 1000;
                libc::pthread_cond_timedwait(
                    ptr::addr_of_mut!((*q).input_empty_c),
                    ptr::addr_of_mut!((*p).pool_m),
                    &timeout,
                );
            }

            while (*q).n_processing != 0 {
                libc::gettimeofday(&mut now, ptr::null_mut());
                timeout.tv_sec = now.tv_sec + 1;
                timeout.tv_nsec = now.tv_usec * 1000;
                libc::pthread_cond_timedwait(
                    ptr::addr_of_mut!((*q).none_processing_c),
                    ptr::addr_of_mut!((*p).pool_m),
                    &timeout,
                );
            }
            if (*q).shutdown != 0 {
                break;
            }
        }

        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));
    }
    0
}

// original: hts_tpool_process_reset (htslib/thread_pool.c:1013)
pub unsafe fn hts_tpool_process_reset(q: *mut hts_tpool_process, free_results: c_int) -> c_int {
    let q = unsafe { process(q) };
    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m));
        (*q).next_serial = c_int::MAX as u64;

        let mut j = (*q).input_head;
        (*q).input_head = ptr::null_mut();
        (*q).input_tail = ptr::null_mut();
        (*q).n_input = 0;

        let mut r = (*q).output_head;
        (*q).output_head = ptr::null_mut();
        (*q).output_tail = ptr::null_mut();
        (*q).n_output = 0;
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));

        while !j.is_null() {
            let jn = (*j).next;
            if let Some(cleanup) = (*j).job_cleanup {
                cleanup((*j).arg);
            }
            libc::free(j.cast());
            j = jn;
        }

        while !r.is_null() {
            let rn = (*r).next;
            if let Some(cleanup) = (*r).result_cleanup {
                cleanup((*r).data);
                (*r).data = ptr::null_mut();
            }
            hts_tpool_delete_result(r.cast(), free_results);
            r = rn;
        }

        if hts_tpool_process_flush(q.cast()) != 0 {
            return -1;
        }

        libc::pthread_mutex_lock(ptr::addr_of_mut!((*(*q).p).pool_m));
        r = (*q).output_head;
        (*q).output_head = ptr::null_mut();
        (*q).output_tail = ptr::null_mut();
        (*q).n_output = 0;
        (*q).next_serial = 0;
        (*q).curr_serial = 0;
        libc::pthread_cond_signal(ptr::addr_of_mut!((*q).input_not_full_c));
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*(*q).p).pool_m));

        while !r.is_null() {
            let rn = (*r).next;
            if let Some(cleanup) = (*r).result_cleanup {
                cleanup((*r).data);
                (*r).data = ptr::null_mut();
            }
            hts_tpool_delete_result(r.cast(), free_results);
            r = rn;
        }
    }
    0
}

// original: hts_tpool_process_qsize (htslib/thread_pool.c:1081)
pub unsafe fn hts_tpool_process_qsize(q: *mut hts_tpool_process) -> c_int {
    unsafe { (*process(q)).qsize }
}

// original: hts_tpool_destroy (htslib/thread_pool.c:1089)
pub unsafe fn hts_tpool_destroy(p: *mut hts_tpool) {
    let p = unsafe { pool(p) };
    unsafe {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*p).pool_m));
        (*p).shutdown = 1;

        for i in 0..(*p).tsize {
            libc::pthread_cond_signal(ptr::addr_of_mut!((*(*p).t.add(i as usize)).pending_c));
        }

        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*p).pool_m));

        for i in 0..(*p).tsize {
            libc::pthread_join((*(*p).t.add(i as usize)).tid, ptr::null_mut());
        }

        libc::pthread_mutex_destroy(ptr::addr_of_mut!((*p).pool_m));
        for i in 0..(*p).tsize {
            libc::pthread_cond_destroy(ptr::addr_of_mut!((*(*p).t.add(i as usize)).pending_c));
        }

        if !(*p).t_stack.is_null() {
            libc::free((*p).t_stack.cast());
        }
        libc::free((*p).t.cast());
        libc::free(p.cast());
    }
}

// original: hts_tpool_kill (htslib/thread_pool.c:1128)
pub unsafe fn hts_tpool_kill(p: *mut hts_tpool) {
    let p = unsafe { pool(p) };
    unsafe {
        for i in 0..(*p).tsize {
            libc::pthread_kill((*(*p).t.add(i as usize)).tid, libc::SIGINT);
        }

        libc::pthread_mutex_destroy(ptr::addr_of_mut!((*p).pool_m));
        for i in 0..(*p).tsize {
            libc::pthread_cond_destroy(ptr::addr_of_mut!((*(*p).t.add(i as usize)).pending_c));
        }

        if !(*p).t_stack.is_null() {
            libc::free((*p).t_stack.cast());
        }
        libc::free((*p).t.cast());
        libc::free(p.cast());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static JOB_CLEANUPS: AtomicUsize = AtomicUsize::new(0);
    static RESULT_CLEANUPS: AtomicUsize = AtomicUsize::new(0);
    static WORKER_ID_FAILURES: AtomicUsize = AtomicUsize::new(0);

    #[repr(C)]
    struct TestWorkerArg {
        pool: *mut hts_tpool,
        value: c_int,
        delay_us: libc::useconds_t,
    }

    unsafe fn init_test_mutex(mutex: *mut libc::pthread_mutex_t) {
        let mut attr: libc::pthread_mutexattr_t = unsafe { mem::zeroed() };
        unsafe {
            assert_eq!(libc::pthread_mutexattr_init(&mut attr), 0);
            assert_eq!(
                libc::pthread_mutexattr_settype(&mut attr, libc::PTHREAD_MUTEX_RECURSIVE),
                0
            );
            assert_eq!(libc::pthread_mutex_init(mutex, &attr), 0);
            assert_eq!(libc::pthread_mutexattr_destroy(&mut attr), 0);
        }
    }

    unsafe fn init_test_process_conds(queue: *mut HtsTpoolProcess) {
        unsafe {
            assert_eq!(
                libc::pthread_cond_init(ptr::addr_of_mut!((*queue).output_avail_c), ptr::null()),
                0
            );
            assert_eq!(
                libc::pthread_cond_init(ptr::addr_of_mut!((*queue).input_not_full_c), ptr::null()),
                0
            );
            assert_eq!(
                libc::pthread_cond_init(ptr::addr_of_mut!((*queue).input_empty_c), ptr::null()),
                0
            );
            assert_eq!(
                libc::pthread_cond_init(ptr::addr_of_mut!((*queue).none_processing_c), ptr::null()),
                0
            );
        }
    }

    unsafe fn destroy_test_process_conds(queue: *mut HtsTpoolProcess) {
        unsafe {
            assert_eq!(
                libc::pthread_cond_destroy(ptr::addr_of_mut!((*queue).output_avail_c)),
                0
            );
            assert_eq!(
                libc::pthread_cond_destroy(ptr::addr_of_mut!((*queue).input_not_full_c)),
                0
            );
            assert_eq!(
                libc::pthread_cond_destroy(ptr::addr_of_mut!((*queue).input_empty_c)),
                0
            );
            assert_eq!(
                libc::pthread_cond_destroy(ptr::addr_of_mut!((*queue).none_processing_c)),
                0
            );
        }
    }

    unsafe extern "C" fn count_job_cleanup(_arg: *mut c_void) {
        JOB_CLEANUPS.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn count_result_cleanup(_arg: *mut c_void) {
        RESULT_CLEANUPS.fetch_add(1, Ordering::SeqCst);
    }

    unsafe extern "C" fn count_pointed_atomic(arg: *mut c_void) {
        unsafe {
            (*(arg.cast::<AtomicUsize>())).fetch_add(1, Ordering::SeqCst);
        }
    }

    unsafe extern "C" fn square_and_record_worker_id(arg: *mut c_void) -> *mut c_void {
        let arg = arg.cast::<TestWorkerArg>();
        let pool = unsafe { (*arg).pool };
        let value = unsafe { (*arg).value };
        let delay_us = unsafe { (*arg).delay_us };

        if unsafe { hts_tpool_worker_id(pool) } < 0 {
            WORKER_ID_FAILURES.fetch_add(1, Ordering::SeqCst);
        }
        if delay_us != 0 {
            unsafe { libc::usleep(delay_us) };
        }

        let result = unsafe { libc::malloc(mem::size_of::<c_int>()).cast::<c_int>() };
        assert!(!result.is_null());
        unsafe {
            *result = value * value;
            libc::free(arg.cast());
        }
        result.cast()
    }

    unsafe fn alloc_worker_arg(
        pool: *mut hts_tpool,
        value: c_int,
        delay_us: libc::useconds_t,
    ) -> *mut c_void {
        let arg = unsafe { libc::malloc(mem::size_of::<TestWorkerArg>()).cast::<TestWorkerArg>() };
        assert!(!arg.is_null());
        unsafe {
            ptr::write(
                arg,
                TestWorkerArg {
                    pool,
                    value,
                    delay_us,
                },
            );
        }
        arg.cast()
    }

    #[test]
    fn worker_id_null_and_delete_null_are_safe_noops() {
        unsafe {
            assert_eq!(hts_tpool_worker_id(ptr::null_mut()), -1);
            hts_tpool_delete_result(ptr::null_mut(), 1);
        }
    }

    #[test]
    fn result_data_returns_payload_and_delete_ignores_cleanup_callback() {
        unsafe {
            let result = xmalloc::<HtsTpoolResult>();
            assert!(!result.is_null());
            ptr::write(
                result,
                HtsTpoolResult {
                    next: ptr::null_mut(),
                    result_cleanup: Some(count_result_cleanup),
                    serial: 0,
                    data: 0x1234usize as *mut c_void,
                },
            );

            RESULT_CLEANUPS.store(0, Ordering::SeqCst);
            assert_eq!(
                hts_tpool_result_data(result.cast()),
                0x1234usize as *mut c_void
            );
            hts_tpool_delete_result(result.cast(), 0);
            assert_eq!(RESULT_CLEANUPS.load(Ordering::SeqCst), 0);
        }
    }

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

    #[test]
    fn thread_pool_init_rejects_negative_thread_count_without_overflowing_allocation() {
        unsafe {
            *c_compat::__errno_location() = 0;
            assert!(hts_tpool_init(-1).is_null());
            assert_eq!(*c_compat::__errno_location(), libc::ENOMEM);
        }
    }

    #[test]
    fn thread_pool_executes_jobs_and_returns_results_in_serial_order() {
        unsafe {
            WORKER_ID_FAILURES.store(0, Ordering::SeqCst);

            let pool = hts_tpool_init(2);
            assert!(!pool.is_null());
            let queue = hts_tpool_process_init(pool, 8, 0);
            assert!(!queue.is_null());

            for value in 0..6 {
                let delay_us = if value == 0 { 20_000 } else { 0 };
                assert_eq!(
                    hts_tpool_dispatch(
                        pool,
                        queue,
                        Some(square_and_record_worker_id),
                        alloc_worker_arg(pool, value, delay_us),
                    ),
                    0
                );
            }

            for value in 0..6 {
                let result = hts_tpool_next_result_wait(queue);
                assert!(!result.is_null());
                let data = hts_tpool_result_data(result).cast::<c_int>();
                assert_eq!(*data, value * value);
                hts_tpool_delete_result(result, 1);
            }

            assert_eq!(WORKER_ID_FAILURES.load(Ordering::SeqCst), 0);
            assert_eq!(hts_tpool_process_empty(queue), 1);

            hts_tpool_process_destroy(queue);
            hts_tpool_destroy(pool);
        }
    }

    #[test]
    fn process_accounting_wrappers_report_combined_queue_state_and_refcount() {
        unsafe {
            let mut pool: HtsTpool = mem::zeroed();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));
            pool.t_stack_top = -1;

            let mut queue: HtsTpoolProcess = mem::zeroed();
            queue.p = ptr::addr_of_mut!(pool);
            queue.qsize = 7;
            queue.n_input = 2;
            queue.n_processing = 3;
            queue.n_output = 4;
            queue.ref_count = 1;

            assert_eq!(hts_tpool_process_empty(ptr::addr_of_mut!(queue).cast()), 0);
            assert_eq!(hts_tpool_process_len(ptr::addr_of_mut!(queue).cast()), 4);
            assert_eq!(hts_tpool_process_sz(ptr::addr_of_mut!(queue).cast()), 9);
            assert_eq!(hts_tpool_process_qsize(ptr::addr_of_mut!(queue).cast()), 7);

            hts_tpool_process_ref_incr(ptr::addr_of_mut!(queue).cast());
            assert_eq!(queue.ref_count, 2);
            hts_tpool_process_ref_decr(ptr::addr_of_mut!(queue).cast());
            assert_eq!(queue.ref_count, 1);

            queue.n_input = 0;
            queue.n_processing = 0;
            queue.n_output = 0;
            assert_eq!(hts_tpool_process_empty(ptr::addr_of_mut!(queue).cast()), 1);
            assert_eq!(hts_tpool_process_sz(ptr::addr_of_mut!(queue).cast()), 0);

            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn process_attach_and_detach_maintain_circular_queue_ring() {
        unsafe {
            let mut pool: HtsTpool = mem::zeroed();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));
            pool.t_stack_top = -1;

            let mut first: HtsTpoolProcess = mem::zeroed();
            let mut second: HtsTpoolProcess = mem::zeroed();
            let first_ptr = ptr::addr_of_mut!(first);
            let second_ptr = ptr::addr_of_mut!(second);

            hts_tpool_process_attach(ptr::addr_of_mut!(pool).cast(), first_ptr.cast());
            assert_eq!(pool.q_head, first_ptr);
            assert_eq!(first.next, first_ptr);
            assert_eq!(first.prev, first_ptr);

            hts_tpool_process_attach(ptr::addr_of_mut!(pool).cast(), second_ptr.cast());
            assert_eq!(pool.q_head, second_ptr);
            assert_eq!(second.next, first_ptr);
            assert_eq!(second.prev, first_ptr);
            assert_eq!(first.next, second_ptr);
            assert_eq!(first.prev, second_ptr);

            hts_tpool_process_detach(ptr::addr_of_mut!(pool).cast(), first_ptr.cast());
            assert_eq!(pool.q_head, second_ptr);
            assert_eq!(second.next, second_ptr);
            assert_eq!(second.prev, second_ptr);
            assert!(first.next.is_null());
            assert!(first.prev.is_null());

            hts_tpool_process_detach(ptr::addr_of_mut!(pool).cast(), first_ptr.cast());
            assert_eq!(pool.q_head, second_ptr);
            assert_eq!(second.next, second_ptr);
            assert_eq!(second.prev, second_ptr);

            hts_tpool_process_detach(ptr::addr_of_mut!(pool).cast(), second_ptr.cast());
            assert!(pool.q_head.is_null());
            assert!(second.next.is_null());
            assert!(second.prev.is_null());

            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn next_result_locked_returns_results_in_serial_order() {
        unsafe {
            let mut pool: HtsTpool = mem::zeroed();
            pool.t_stack_top = -1;
            let mut queue: HtsTpoolProcess = mem::zeroed();
            queue.p = ptr::addr_of_mut!(pool);
            queue.qsize = 0;
            queue.next_serial = 0;
            queue.n_output = 2;

            let mut later = HtsTpoolResult {
                next: ptr::null_mut(),
                result_cleanup: None,
                serial: 1,
                data: 0x11usize as *mut c_void,
            };
            let mut first = HtsTpoolResult {
                next: ptr::addr_of_mut!(later),
                result_cleanup: None,
                serial: 0,
                data: 0x22usize as *mut c_void,
            };
            queue.output_head = ptr::addr_of_mut!(first);
            queue.output_tail = ptr::addr_of_mut!(later);

            let r0 = hts_tpool_next_result_locked(ptr::addr_of_mut!(queue));
            assert_eq!(r0, ptr::addr_of_mut!(first));
            assert_eq!(queue.next_serial, 1);
            assert_eq!(queue.n_output, 1);
            assert_eq!(queue.output_head, ptr::addr_of_mut!(later));
            assert_eq!(queue.output_tail, ptr::addr_of_mut!(later));

            let r1 = hts_tpool_next_result_locked(ptr::addr_of_mut!(queue));
            assert_eq!(r1, ptr::addr_of_mut!(later));
            assert_eq!(queue.next_serial, 2);
            assert_eq!(queue.n_output, 0);
            assert!(queue.output_head.is_null());
            assert!(queue.output_tail.is_null());
        }
    }

    #[test]
    fn next_result_locked_waits_for_missing_serial_and_honors_shutdown() {
        unsafe {
            let mut pool: HtsTpool = mem::zeroed();
            pool.t_stack_top = -1;
            let mut queue: HtsTpoolProcess = mem::zeroed();
            queue.p = ptr::addr_of_mut!(pool);
            queue.qsize = 0;
            queue.next_serial = 0;
            queue.n_output = 1;

            let mut later = HtsTpoolResult {
                next: ptr::null_mut(),
                result_cleanup: None,
                serial: 1,
                data: ptr::null_mut(),
            };
            queue.output_head = ptr::addr_of_mut!(later);
            queue.output_tail = ptr::addr_of_mut!(later);

            assert!(hts_tpool_next_result_locked(ptr::addr_of_mut!(queue)).is_null());
            assert_eq!(queue.next_serial, 0);
            assert_eq!(queue.n_output, 1);
            assert_eq!(queue.output_head, ptr::addr_of_mut!(later));
            assert_eq!(queue.output_tail, ptr::addr_of_mut!(later));

            queue.shutdown = 1;
            assert!(hts_tpool_next_result_locked(ptr::addr_of_mut!(queue)).is_null());
            assert_eq!(queue.next_serial, 0);
            assert_eq!(queue.n_output, 1);
        }
    }

    #[test]
    fn next_result_locked_removes_matching_serial_from_middle() {
        unsafe {
            let mut pool: HtsTpool = mem::zeroed();
            pool.t_stack_top = -1;
            let mut queue: HtsTpoolProcess = mem::zeroed();
            queue.p = ptr::addr_of_mut!(pool);
            queue.qsize = 0;
            queue.next_serial = 1;
            queue.n_output = 3;

            let mut tail = HtsTpoolResult {
                next: ptr::null_mut(),
                result_cleanup: None,
                serial: 2,
                data: 0x22usize as *mut c_void,
            };
            let mut middle = HtsTpoolResult {
                next: ptr::addr_of_mut!(tail),
                result_cleanup: None,
                serial: 1,
                data: 0x11usize as *mut c_void,
            };
            let mut head = HtsTpoolResult {
                next: ptr::addr_of_mut!(middle),
                result_cleanup: None,
                serial: 0,
                data: 0x00usize as *mut c_void,
            };
            queue.output_head = ptr::addr_of_mut!(head);
            queue.output_tail = ptr::addr_of_mut!(tail);

            let r = hts_tpool_next_result_locked(ptr::addr_of_mut!(queue));
            assert_eq!(r, ptr::addr_of_mut!(middle));
            assert_eq!(queue.next_serial, 2);
            assert_eq!(queue.n_output, 2);
            assert_eq!(queue.output_head, ptr::addr_of_mut!(head));
            assert_eq!(head.next, ptr::addr_of_mut!(tail));
            assert_eq!(queue.output_tail, ptr::addr_of_mut!(tail));
        }
    }

    #[test]
    fn process_shutdown_sets_flag_and_blocks_pending_output_delivery() {
        unsafe {
            let mut pool: HtsTpool = mem::zeroed();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));
            pool.t_stack_top = -1;

            let mut queue: HtsTpoolProcess = mem::zeroed();
            init_test_process_conds(ptr::addr_of_mut!(queue));
            queue.p = ptr::addr_of_mut!(pool);
            queue.qsize = 0;
            queue.next_serial = 0;
            queue.n_output = 1;

            let mut result = HtsTpoolResult {
                next: ptr::null_mut(),
                result_cleanup: None,
                serial: 0,
                data: 0x55usize as *mut c_void,
            };
            queue.output_head = ptr::addr_of_mut!(result);
            queue.output_tail = ptr::addr_of_mut!(result);

            hts_tpool_process_shutdown(ptr::addr_of_mut!(queue).cast());
            assert_eq!(
                hts_tpool_process_is_shutdown(ptr::addr_of_mut!(queue).cast()),
                1
            );
            assert!(hts_tpool_next_result(ptr::addr_of_mut!(queue).cast()).is_null());
            assert_eq!(queue.next_serial, 0);
            assert_eq!(queue.n_output, 1);
            assert_eq!(queue.output_head, ptr::addr_of_mut!(result));
            assert_eq!(queue.output_tail, ptr::addr_of_mut!(result));

            destroy_test_process_conds(ptr::addr_of_mut!(queue));
            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn add_result_in_only_queue_discards_data_after_finishing_job() {
        unsafe {
            let mut pool: HtsTpool = mem::zeroed();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));
            pool.t_stack_top = -1;

            let mut queue: HtsTpoolProcess = mem::zeroed();
            init_test_process_conds(ptr::addr_of_mut!(queue));
            queue.p = ptr::addr_of_mut!(pool);
            queue.qsize = 1;
            queue.in_only = 1;
            queue.n_processing = 1;

            let mut job = HtsTpoolJob {
                func: None,
                arg: ptr::null_mut(),
                job_cleanup: None,
                result_cleanup: Some(count_result_cleanup),
                next: ptr::null_mut(),
                p: ptr::addr_of_mut!(pool),
                q: ptr::addr_of_mut!(queue),
                serial: 0,
            };

            RESULT_CLEANUPS.store(0, Ordering::SeqCst);
            assert_eq!(
                hts_tpool_add_result(ptr::addr_of_mut!(job), 0x77usize as *mut c_void),
                0
            );
            assert_eq!(queue.n_processing, 0);
            assert_eq!(queue.n_output, 0);
            assert!(queue.output_head.is_null());
            assert!(queue.output_tail.is_null());
            assert_eq!(RESULT_CLEANUPS.load(Ordering::SeqCst), 0);

            destroy_test_process_conds(ptr::addr_of_mut!(queue));
            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn dispatch3_nonblock_full_queue_sets_eagain_without_state_change() {
        unsafe {
            let mut pool: HtsTpool = mem::zeroed();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));
            pool.t_stack_top = -1;

            let mut queue: HtsTpoolProcess = mem::zeroed();
            queue.p = ptr::addr_of_mut!(pool);
            queue.qsize = 1;
            queue.n_input = 1;
            queue.curr_serial = 7;

            *c_compat::__errno_location() = 0;
            let ret = hts_tpool_dispatch3(
                ptr::addr_of_mut!(pool).cast(),
                ptr::addr_of_mut!(queue).cast(),
                None,
                ptr::null_mut(),
                None,
                None,
                1,
            );

            assert_eq!(ret, -1);
            assert_eq!(*c_compat::__errno_location(), libc::EAGAIN);
            assert_eq!(queue.curr_serial, 7);
            assert_eq!(queue.n_input, 1);
            assert!(queue.input_head.is_null());
            assert!(queue.input_tail.is_null());
            assert_eq!(pool.njobs, 0);

            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn dispatch3_wake_dispatch_allows_one_blocking_enqueue_on_full_queue() {
        unsafe {
            let mut pool: HtsTpool = mem::zeroed();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));
            pool.t_stack_top = -1;

            let mut queue: HtsTpoolProcess = mem::zeroed();
            init_test_process_conds(ptr::addr_of_mut!(queue));
            queue.p = ptr::addr_of_mut!(pool);
            queue.qsize = 1;
            queue.n_input = 1;
            queue.curr_serial = 41;
            queue.wake_dispatch = 1;
            queue.next = ptr::addr_of_mut!(queue);
            queue.prev = ptr::addr_of_mut!(queue);
            pool.q_head = ptr::addr_of_mut!(queue);
            pool.njobs = queue.n_input;

            let first_job = xmalloc::<HtsTpoolJob>();
            assert!(!first_job.is_null());
            ptr::write(
                first_job,
                HtsTpoolJob {
                    func: None,
                    arg: 0x01usize as *mut c_void,
                    job_cleanup: None,
                    result_cleanup: None,
                    next: ptr::null_mut(),
                    p: ptr::addr_of_mut!(pool),
                    q: ptr::addr_of_mut!(queue),
                    serial: 40,
                },
            );
            queue.input_head = first_job;
            queue.input_tail = first_job;

            assert_eq!(
                hts_tpool_dispatch3(
                    ptr::addr_of_mut!(pool).cast(),
                    ptr::addr_of_mut!(queue).cast(),
                    None,
                    0x02usize as *mut c_void,
                    None,
                    Some(count_result_cleanup),
                    0,
                ),
                0
            );

            assert_eq!(queue.wake_dispatch, 0);
            assert_eq!(queue.curr_serial, 42);
            assert_eq!(queue.n_input, 2);
            assert_eq!(pool.njobs, 2);
            assert_eq!((*queue.input_head).serial, 40);
            assert_eq!((*queue.input_tail).serial, 41);
            assert!((*queue.input_tail).result_cleanup.is_some());

            assert_eq!(
                hts_tpool_process_reset(ptr::addr_of_mut!(queue).cast(), 0),
                0
            );
            destroy_test_process_conds(ptr::addr_of_mut!(queue));
            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn dispatch3_blocking_shutdown_after_allocation_returns_without_enqueue() {
        unsafe {
            let mut pool: HtsTpool = mem::zeroed();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));
            pool.t_stack_top = -1;

            let mut queue: HtsTpoolProcess = mem::zeroed();
            queue.p = ptr::addr_of_mut!(pool);
            queue.qsize = 1;
            queue.n_input = 1;
            queue.shutdown = 1;
            queue.curr_serial = 7;

            JOB_CLEANUPS.store(0, Ordering::SeqCst);
            RESULT_CLEANUPS.store(0, Ordering::SeqCst);
            let ret = hts_tpool_dispatch3(
                ptr::addr_of_mut!(pool).cast(),
                ptr::addr_of_mut!(queue).cast(),
                None,
                0x55usize as *mut c_void,
                Some(count_job_cleanup),
                Some(count_result_cleanup),
                0,
            );

            assert_eq!(ret, -1);
            assert_eq!(queue.curr_serial, 8);
            assert_eq!(queue.n_input, 1);
            assert!(queue.input_head.is_null());
            assert!(queue.input_tail.is_null());
            assert_eq!(pool.njobs, 0);
            assert_eq!(JOB_CLEANUPS.load(Ordering::SeqCst), 0);
            assert_eq!(RESULT_CLEANUPS.load(Ordering::SeqCst), 0);

            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn process_reset_cleans_queued_work_and_resets_serial_state() {
        unsafe {
            let job_cleanups = AtomicUsize::new(0);
            let result_cleanups = AtomicUsize::new(0);
            let job_counter = ptr::addr_of!(job_cleanups) as *mut c_void;
            let result_counter = ptr::addr_of!(result_cleanups) as *mut c_void;

            let mut pool: HtsTpool = mem::zeroed();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));
            pool.t_stack_top = -1;

            let mut queue: HtsTpoolProcess = mem::zeroed();
            init_test_process_conds(ptr::addr_of_mut!(queue));
            queue.p = ptr::addr_of_mut!(pool);
            queue.qsize = 4;
            queue.next_serial = 12;
            queue.curr_serial = 14;
            queue.n_input = 2;
            queue.n_output = 1;

            let first_job = xmalloc::<HtsTpoolJob>();
            let second_job = xmalloc::<HtsTpoolJob>();
            let result = xmalloc::<HtsTpoolResult>();
            assert!(!first_job.is_null());
            assert!(!second_job.is_null());
            assert!(!result.is_null());

            ptr::write(
                second_job,
                HtsTpoolJob {
                    func: None,
                    arg: job_counter,
                    job_cleanup: Some(count_pointed_atomic),
                    result_cleanup: None,
                    next: ptr::null_mut(),
                    p: ptr::addr_of_mut!(pool),
                    q: ptr::addr_of_mut!(queue),
                    serial: 13,
                },
            );
            ptr::write(
                first_job,
                HtsTpoolJob {
                    func: None,
                    arg: job_counter,
                    job_cleanup: Some(count_pointed_atomic),
                    result_cleanup: None,
                    next: second_job,
                    p: ptr::addr_of_mut!(pool),
                    q: ptr::addr_of_mut!(queue),
                    serial: 12,
                },
            );
            ptr::write(
                result,
                HtsTpoolResult {
                    next: ptr::null_mut(),
                    result_cleanup: Some(count_pointed_atomic),
                    serial: 11,
                    data: result_counter,
                },
            );
            queue.input_head = first_job;
            queue.input_tail = second_job;
            queue.output_head = result;
            queue.output_tail = result;

            assert_eq!(
                hts_tpool_process_reset(ptr::addr_of_mut!(queue).cast(), 1),
                0
            );

            assert!(queue.input_head.is_null());
            assert!(queue.input_tail.is_null());
            assert!(queue.output_head.is_null());
            assert!(queue.output_tail.is_null());
            assert_eq!(queue.n_input, 0);
            assert_eq!(queue.n_output, 0);
            assert_eq!(queue.next_serial, 0);
            assert_eq!(queue.curr_serial, 0);
            assert_eq!(job_cleanups.load(Ordering::SeqCst), 2);
            assert_eq!(result_cleanups.load(Ordering::SeqCst), 1);

            destroy_test_process_conds(ptr::addr_of_mut!(queue));
            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }
}
