use std::collections::VecDeque;
use std::ffi::c_void;
use std::mem;
use std::ptr;
use std::ptr::NonNull;

use crate::htslib_rs::c_compat;

// Native thread pool types. The public names alias the native structs defined
// below; the implementation in this module operates on them directly (the
// previous opaque C-pointer casts become identity casts). These were the only
// references to the C library in this module, so it is now C-library-free.
// original: hts_tpool (htslib/thread_pool_internal.h:136)
pub type hts_tpool = HtsTpool;
// original: hts_tpool_process (htslib/thread_pool_internal.h:100)
pub type hts_tpool_process = HtsTpoolProcess;
// original: hts_tpool_result (htslib/thread_pool_internal.h:71)
pub type hts_tpool_result = HtsTpoolResult;

pub type hts_tpool_worker = Option<unsafe extern "C" fn(arg: *mut c_void) -> *mut c_void>;
pub type hts_tpool_cleanup = Option<unsafe extern "C" fn(arg: *mut c_void)>;

// original: HTS_MIN_THREAD_STACK (htslib/thread_pool.c:44)
const HTS_MIN_THREAD_STACK: usize = 3 * 1024 * 1024;

// original: hts_tpool_job (htslib/thread_pool_internal.h:56)
struct HtsTpoolJob {
    func: hts_tpool_worker,
    arg: Option<NonNull<c_void>>,
    job_cleanup: hts_tpool_cleanup,
    result_cleanup: hts_tpool_cleanup,
    p: NonNull<HtsTpool>,
    q: NonNull<HtsTpoolProcess>,
    serial: u64,
}

// original: hts_tpool_result (htslib/thread_pool_internal.h:71)
pub struct HtsTpoolResult {
    result_cleanup: hts_tpool_cleanup,
    serial: u64,
    data: Option<NonNull<c_void>>,
}

// original: hts_tpool_worker (htslib/thread_pool_internal.h:83)
struct HtsTpoolWorker {
    p: Option<NonNull<HtsTpool>>,
    idx: usize,
    tid: crate::htslib_rs::c_compat::pthread_t,
    pending_c: crate::htslib_rs::c_compat::pthread_cond_t,
}

// original: hts_tpool_process (htslib/thread_pool_internal.h:100)
pub struct HtsTpoolProcess {
    p: Option<NonNull<HtsTpool>>,
    input: VecDeque<Box<HtsTpoolJob>>,
    output: VecDeque<Box<HtsTpoolResult>>,
    qsize: i32,
    next_serial: u64,
    curr_serial: u64,
    no_more_input: bool,
    n_input: i32,
    n_output: i32,
    n_processing: i32,
    shutdown: i32,
    in_only: bool,
    wake_dispatch: bool,
    ref_count: i32,
    output_avail_c: crate::htslib_rs::c_compat::pthread_cond_t,
    input_not_full_c: crate::htslib_rs::c_compat::pthread_cond_t,
    input_empty_c: crate::htslib_rs::c_compat::pthread_cond_t,
    none_processing_c: crate::htslib_rs::c_compat::pthread_cond_t,
    next: Option<NonNull<HtsTpoolProcess>>,
    prev: Option<NonNull<HtsTpoolProcess>>,
}

// original: hts_tpool (htslib/thread_pool_internal.h:136)
pub struct HtsTpool {
    nwaiting: i32,
    njobs: i32,
    shutdown: bool,
    q_head: Option<NonNull<HtsTpoolProcess>>,
    tsize: usize,
    t: Vec<HtsTpoolWorker>,
    t_stack: Vec<bool>,
    t_stack_top: Option<usize>,
    pool_m: crate::htslib_rs::c_compat::pthread_mutex_t,
    n_count: i32,
    n_running: i32,
    total_time: i64,
    wait_time: i64,
}

unsafe fn pool_mut<'a>(p: *mut hts_tpool) -> Option<&'a mut HtsTpool> {
    unsafe { p.cast::<HtsTpool>().as_mut() }
}

unsafe fn pool_ref<'a>(p: *const hts_tpool) -> Option<&'a HtsTpool> {
    unsafe { p.cast::<HtsTpool>().as_ref() }
}

unsafe fn process_mut<'a>(q: *mut hts_tpool_process) -> Option<&'a mut HtsTpoolProcess> {
    unsafe { q.cast::<HtsTpoolProcess>().as_mut() }
}

unsafe fn process_ref<'a>(q: *const hts_tpool_process) -> Option<&'a HtsTpoolProcess> {
    unsafe { q.cast::<HtsTpoolProcess>().as_ref() }
}

unsafe fn result_mut<'a>(r: *mut hts_tpool_result) -> Option<&'a mut HtsTpoolResult> {
    unsafe { r.cast::<HtsTpoolResult>().as_mut() }
}

unsafe fn q_pool_mut<'a>(q: &HtsTpoolProcess) -> Option<&'a mut HtsTpool> {
    unsafe { q.p.map(|mut p| p.as_mut()) }
}

// original: hts_tpool_worker_id (htslib/thread_pool.c:56)
pub unsafe fn hts_tpool_worker_id(p: *mut hts_tpool) -> i32 {
    let Some(p) = (unsafe { pool_ref(p) }) else {
        return -1;
    };
    let s = unsafe { crate::htslib_rs::c_compat::pthread_self() };
    for worker in &p.t {
        if unsafe { crate::htslib_rs::c_compat::pthread_equal(s, worker.tid) } != 0 {
            return worker.idx as i32;
        }
    }
    -1
}

unsafe fn make_process(
    p: Option<NonNull<HtsTpool>>,
    qsize: i32,
    in_only: bool,
) -> Box<HtsTpoolProcess> {
    unsafe {
        Box::new(HtsTpoolProcess {
            p,
            input: VecDeque::new(),
            output: VecDeque::new(),
            qsize,
            next_serial: 0,
            curr_serial: 0,
            no_more_input: false,
            n_input: 0,
            n_output: 0,
            n_processing: 0,
            shutdown: 0,
            in_only,
            wake_dispatch: false,
            ref_count: 1,
            output_avail_c: mem::zeroed(),
            input_not_full_c: mem::zeroed(),
            input_empty_c: mem::zeroed(),
            none_processing_c: mem::zeroed(),
            next: None,
            prev: None,
        })
    }
}

unsafe fn make_pool(n: usize) -> Box<HtsTpool> {
    unsafe {
        Box::new(HtsTpool {
            nwaiting: 0,
            njobs: 0,
            shutdown: false,
            q_head: None,
            tsize: n,
            t: Vec::new(),
            t_stack: Vec::new(),
            t_stack_top: None,
            pool_m: mem::zeroed(),
            n_count: 0,
            n_running: 0,
            total_time: 0,
            wait_time: 0,
        })
    }
}

unsafe fn make_worker(p: NonNull<HtsTpool>, idx: usize) -> HtsTpoolWorker {
    unsafe {
        HtsTpoolWorker {
            p: Some(p),
            idx,
            tid: mem::zeroed(),
            pending_c: mem::zeroed(),
        }
    }
}

// original: hts_tpool_add_result (htslib/thread_pool.c:95)
unsafe fn hts_tpool_add_result(j: Box<HtsTpoolJob>, data: Option<NonNull<c_void>>) -> i32 {
    let q = unsafe { j.q.as_ptr().as_mut().unwrap() };
    let p = unsafe { q_pool_mut(q).unwrap() };
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m)) };

    unsafe {
        q.n_processing -= 1;
        if q.n_processing == 0 {
            crate::htslib_rs::c_compat::pthread_cond_signal(ptr::addr_of_mut!(q.none_processing_c));
        }

        if q.in_only {
            crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
            return 0;
        }
    }

    unsafe {
        let r = Box::new(HtsTpoolResult {
            data,
            result_cleanup: j.result_cleanup,
            serial: j.serial,
        });
        let serial = r.serial;

        q.n_output += 1;
        q.output.push_back(r);

        assert!(serial >= q.next_serial || q.next_serial == i32::MAX as u64);
        if serial == q.next_serial {
            crate::htslib_rs::c_compat::pthread_cond_broadcast(ptr::addr_of_mut!(q.output_avail_c));
        }

        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
    }

    0
}

// original: hts_tpool_next_result_locked (htslib/thread_pool.c:149)
unsafe fn hts_tpool_next_result_locked(q: &mut HtsTpoolProcess) -> Option<Box<HtsTpoolResult>> {
    if q.shutdown != 0 {
        return None;
    }

    let result_idx = q.output.iter().position(|r| r.serial == q.next_serial);

    if let Some(result_idx) = result_idx {
        unsafe {
            let r = q.output.remove(result_idx).unwrap();

            q.next_serial += 1;
            q.n_output -= 1;

            if q.qsize != 0 && q.n_output < q.qsize {
                if q.n_input < q.qsize {
                    crate::htslib_rs::c_compat::pthread_cond_signal(ptr::addr_of_mut!(
                        q.input_not_full_c
                    ));
                }
                if q.shutdown == 0 {
                    wake_next_worker(q, true);
                }
            }
            return Some(r);
        }
    }

    None
}

// original: hts_tpool_next_result (htslib/thread_pool.c:200)
pub unsafe fn hts_tpool_next_result(q: *mut hts_tpool_process) -> *mut hts_tpool_result {
    let Some(q) = (unsafe { process_mut(q) }) else {
        return ptr::null_mut();
    };
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return ptr::null_mut();
    };
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m)) };
    let r = unsafe { hts_tpool_next_result_locked(q) };
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m)) };
    r.map_or(ptr::null_mut(), Box::into_raw).cast()
}

// original: hts_tpool_next_result_wait (htslib/thread_pool.c:224)
pub unsafe fn hts_tpool_next_result_wait(q: *mut hts_tpool_process) -> *mut hts_tpool_result {
    let Some(q) = (unsafe { process_mut(q) }) else {
        return ptr::null_mut();
    };
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return ptr::null_mut();
    };
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m)) };

    loop {
        let r = unsafe { hts_tpool_next_result_locked(q) };
        if let Some(r) = r {
            unsafe {
                crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m))
            };
            return Box::into_raw(r).cast();
        }

        let mut now: libc::timeval = unsafe { mem::zeroed() };
        let mut timeout: libc::timespec = unsafe { mem::zeroed() };
        unsafe {
            crate::htslib_rs::c_compat::gettimeofday(&mut now, ptr::null_mut());
            timeout.tv_sec = (now.tv_sec + 10) as _;
            timeout.tv_nsec = (now.tv_usec * 1000) as _;

            q.ref_count += 1;
            if q.shutdown != 0 {
                q.ref_count -= 1;
                let rc = q.ref_count;
                crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
                if rc == 0 {
                    hts_tpool_process_destroy(ptr::from_mut(q).cast());
                }
                return ptr::null_mut();
            }
            crate::htslib_rs::c_compat::pthread_cond_timedwait(
                ptr::addr_of_mut!(q.output_avail_c),
                ptr::addr_of_mut!(p.pool_m),
                &timeout,
            );
            q.ref_count -= 1;
        }
    }
}

// original: hts_tpool_process_empty (htslib/thread_pool.c:258)
pub unsafe fn hts_tpool_process_empty(q: &mut HtsTpoolProcess) -> i32 {
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return 1;
    };
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m)) };
    let empty = (q.n_input == 0 && q.n_processing == 0 && q.n_output == 0) as i32;
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m)) };
    empty
}

// original: hts_tpool_process_ref_incr (htslib/thread_pool.c:268)
pub unsafe fn hts_tpool_process_ref_incr(q: *mut hts_tpool_process) {
    let Some(q) = (unsafe { process_mut(q) }) else {
        return;
    };
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return;
    };
    unsafe {
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        q.ref_count += 1;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
    }
}

// original: hts_tpool_process_ref_decr (htslib/thread_pool.c:274)
pub unsafe fn hts_tpool_process_ref_decr(q: *mut hts_tpool_process) {
    let Some(q) = (unsafe { process_mut(q) }) else {
        return;
    };
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return;
    };
    unsafe {
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        q.ref_count -= 1;
        if q.ref_count <= 0 {
            crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
            hts_tpool_process_destroy(ptr::from_mut(q).cast());
            return;
        }
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
    }
}

// original: hts_tpool_process_len (htslib/thread_pool.c:289)
pub unsafe fn hts_tpool_process_len(q: &mut HtsTpoolProcess) -> i32 {
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return 0;
    };
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m)) };
    let len = q.n_output;
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m)) };
    len
}

// original: hts_tpool_process_sz (htslib/thread_pool.c:303)
pub unsafe fn hts_tpool_process_sz(q: &mut HtsTpoolProcess) -> i32 {
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return 0;
    };
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m)) };
    let len = q.n_output + q.n_input + q.n_processing;
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m)) };
    len
}

// original: hts_tpool_process_shutdown_locked (htslib/thread_pool.c:319)
unsafe fn hts_tpool_process_shutdown_locked(q: &mut HtsTpoolProcess) {
    unsafe {
        q.shutdown = 1;
        crate::htslib_rs::c_compat::pthread_cond_broadcast(ptr::addr_of_mut!(q.output_avail_c));
        crate::htslib_rs::c_compat::pthread_cond_broadcast(ptr::addr_of_mut!(q.input_not_full_c));
        crate::htslib_rs::c_compat::pthread_cond_broadcast(ptr::addr_of_mut!(q.input_empty_c));
        crate::htslib_rs::c_compat::pthread_cond_broadcast(ptr::addr_of_mut!(q.none_processing_c));
    }
}

// original: hts_tpool_process_shutdown (htslib/thread_pool.c:327)
pub unsafe fn hts_tpool_process_shutdown(q: *mut hts_tpool_process) {
    let Some(q) = (unsafe { process_mut(q) }) else {
        return;
    };
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return;
    };
    unsafe {
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        hts_tpool_process_shutdown_locked(q);
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
    }
}

// original: hts_tpool_process_is_shutdown (htslib/thread_pool.c:333)
pub unsafe fn hts_tpool_process_is_shutdown(q: *mut hts_tpool_process) -> i32 {
    let Some(q) = (unsafe { process_mut(q) }) else {
        return 0;
    };
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return 0;
    };
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m)) };
    let r = q.shutdown;
    unsafe { crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m)) };
    r
}

// original: hts_tpool_delete_result (htslib/thread_pool.c:344)
pub unsafe fn hts_tpool_delete_result(r: *mut hts_tpool_result, _free_data: i32) {
    let Some(r) = NonNull::new(r.cast::<HtsTpoolResult>()) else {
        return;
    };
    let _ = unsafe { Box::from_raw(r.as_ptr()) };
}

// original: hts_tpool_result_data (htslib/thread_pool.c:358)
pub fn hts_tpool_result_data(r: &mut HtsTpoolResult) -> *mut c_void {
    r.data.map_or(ptr::null_mut(), NonNull::as_ptr)
}

// original: hts_tpool_process_init (htslib/thread_pool.c:372)
pub unsafe fn hts_tpool_process_init(
    p: *mut hts_tpool,
    qsize: i32,
    in_only: i32,
) -> *mut hts_tpool_process {
    let Some(mut p_nn) = NonNull::new(p.cast::<HtsTpool>()) else {
        return ptr::null_mut();
    };
    let mut q = unsafe { make_process(Some(p_nn), qsize, in_only != 0) };
    let q_ptr = ptr::addr_of_mut!(*q);

    unsafe {
        crate::htslib_rs::c_compat::pthread_cond_init(
            ptr::addr_of_mut!((*q_ptr).output_avail_c),
            ptr::null(),
        );
        crate::htslib_rs::c_compat::pthread_cond_init(
            ptr::addr_of_mut!((*q_ptr).input_not_full_c),
            ptr::null(),
        );
        crate::htslib_rs::c_compat::pthread_cond_init(
            ptr::addr_of_mut!((*q_ptr).input_empty_c),
            ptr::null(),
        );
        crate::htslib_rs::c_compat::pthread_cond_init(
            ptr::addr_of_mut!((*q_ptr).none_processing_c),
            ptr::null(),
        );

        hts_tpool_process_attach(p_nn.as_mut(), q.as_mut());
    }

    Box::into_raw(q).cast()
}

// original: hts_tpool_process_destroy (htslib/thread_pool.c:410)
pub unsafe fn hts_tpool_process_destroy(q: *mut hts_tpool_process) {
    let Some(q_nn) = NonNull::new(q.cast::<HtsTpoolProcess>()) else {
        return;
    };
    let q = unsafe { q_nn.as_ptr().as_mut().unwrap() };
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return;
    };

    unsafe {
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        q.no_more_input = true;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));

        hts_tpool_process_reset(q, 0);

        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        hts_tpool_process_detach_locked(p, q);
        hts_tpool_process_shutdown_locked(q);

        q.ref_count -= 1;
        if q.ref_count > 0 {
            crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
            return;
        }

        crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(q.output_avail_c));
        crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(q.input_not_full_c));
        crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(q.input_empty_c));
        crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(q.none_processing_c));
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
        drop(Box::from_raw(q_nn.as_ptr()));
    }
}

// original: hts_tpool_process_attach (htslib/thread_pool.c:456)
pub unsafe fn hts_tpool_process_attach(p: &mut hts_tpool, q: &mut hts_tpool_process) {
    let p = p as &mut HtsTpool;
    let q = q as &mut HtsTpoolProcess;
    let q_nn = NonNull::from(&mut *q);
    unsafe {
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        if let Some(head) = p.q_head {
            (*q).next = Some(head);
            (*q).prev = (*head.as_ptr()).prev;
            if let Some(prev) = (*head.as_ptr()).prev {
                (*prev.as_ptr()).next = Some(q_nn);
            }
            (*head.as_ptr()).prev = Some(q_nn);
        } else {
            (*q).next = Some(q_nn);
            (*q).prev = Some(q_nn);
        }
        p.q_head = Some(q_nn);
        assert!(p.q_head.is_some() && q.prev.is_some() && q.next.is_some());
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
    }
}

// original: hts_tpool_process_detach_locked (htslib/thread_pool.c:472)
unsafe fn hts_tpool_process_detach_locked(p: &mut HtsTpool, q: &mut HtsTpoolProcess) {
    unsafe {
        if p.q_head.is_none() || q.prev.is_none() || q.next.is_none() {
            return;
        }

        let first = p.q_head.unwrap();
        let mut curr = first;
        loop {
            if std::ptr::eq(curr.as_ptr(), q) {
                if let Some(next) = q.next {
                    (*next.as_ptr()).prev = q.prev;
                }
                if let Some(prev) = q.prev {
                    (*prev.as_ptr()).next = q.next;
                }
                p.q_head = q.next;
                q.next = None;
                q.prev = None;

                if p.q_head.is_some_and(|head| std::ptr::eq(head.as_ptr(), q)) {
                    p.q_head = None;
                }
                break;
            }

            let Some(next) = (*curr.as_ptr()).next else {
                break;
            };
            curr = next;
            if curr == first {
                break;
            }
        }
    }
}

// original: hts_tpool_process_detach (htslib/thread_pool.c:495)
pub unsafe fn hts_tpool_process_detach(p: *mut hts_tpool, q: *mut hts_tpool_process) {
    let Some(p) = (unsafe { pool_mut(p) }) else {
        return;
    };
    let Some(q) = (unsafe { process_mut(q) }) else {
        return;
    };
    unsafe {
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        hts_tpool_process_detach_locked(p, q);
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
    }
}

// original: tpool_worker (htslib/thread_pool.c:518)
extern "C" fn tpool_worker(arg: *mut c_void) -> *mut c_void {
    let Some(w) = (unsafe { arg.cast::<HtsTpoolWorker>().as_mut() }) else {
        return ptr::null_mut();
    };
    let Some(mut p_nn) = w.p else {
        return ptr::null_mut();
    };
    let p = unsafe { p_nn.as_mut() };

    unsafe { crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m)) };
    while !p.shutdown {
        assert!(unsafe {
            p.q_head.is_none()
                || ((*p.q_head.unwrap().as_ptr()).prev.is_some()
                    && (*p.q_head.unwrap().as_ptr()).next.is_some())
        });

        let mut work_to_do = 0;
        let first = p.q_head;
        let mut q = first;
        if let Some(first_q) = q {
            let _ = first_q;
            loop {
                let q_ptr = q.unwrap().as_ptr();
                unsafe {
                    if !(*q_ptr).input.is_empty()
                        && (*q_ptr).qsize - (*q_ptr).n_output > (*q_ptr).n_processing
                        && (*q_ptr).shutdown == 0
                    {
                        work_to_do = 1;
                        break;
                    }
                    q = (*q_ptr).next;
                }
                if q.is_none() || q == first {
                    break;
                }
            }
        }

        if work_to_do == 0 {
            unsafe {
                p.nwaiting += 1;
                if p.t_stack_top.is_none_or(|top| top > w.idx) {
                    p.t_stack_top = Some(w.idx);
                }
                p.t_stack[w.idx] = true;
                crate::htslib_rs::c_compat::pthread_cond_wait(
                    ptr::addr_of_mut!(w.pending_c),
                    ptr::addr_of_mut!(p.pool_m),
                );
                p.t_stack[w.idx] = false;

                p.t_stack_top = None;
                for i in 0..p.tsize {
                    if p.t_stack[i] {
                        p.t_stack_top = Some(i);
                        break;
                    }
                }

                p.nwaiting -= 1;
            }
            continue;
        }

        unsafe {
            let mut q_nn = q.unwrap();
            let q = q_nn.as_mut();
            q.ref_count += 1;
            while !q.input.is_empty() && q.qsize - q.n_output > q.n_processing {
                if p.shutdown {
                    crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
                    return ptr::null_mut();
                }

                if q.shutdown != 0 {
                    break;
                }

                let j = q.input.pop_front().unwrap();
                assert!(std::ptr::eq(j.p.as_ptr(), p));

                q.n_processing += 1;
                let old_n_input = q.n_input;
                q.n_input -= 1;
                if old_n_input >= q.qsize {
                    crate::htslib_rs::c_compat::pthread_cond_broadcast(ptr::addr_of_mut!(
                        q.input_not_full_c
                    ));
                }

                if q.n_input == 0 {
                    crate::htslib_rs::c_compat::pthread_cond_signal(ptr::addr_of_mut!(
                        q.input_empty_c
                    ));
                }

                p.njobs -= 1;

                crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));

                let data =
                    j.func.unwrap_unchecked()(j.arg.map_or(ptr::null_mut(), NonNull::as_ptr));
                if hts_tpool_add_result(j, NonNull::new(data)) < 0 {
                    crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
                    let first = p.q_head;
                    let mut q = first;
                    if q.is_some() {
                        loop {
                            let q_ptr = q.unwrap().as_ptr();
                            hts_tpool_process_shutdown_locked(&mut *q_ptr);
                            (*q_ptr).shutdown = 2;
                            q = (*q_ptr).next;
                            if q == first {
                                break;
                            }
                        }
                    }
                    crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
                    return ptr::null_mut();
                }

                crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
            }

            q.ref_count -= 1;
            if q.ref_count == 0 {
                hts_tpool_process_destroy(q_nn.as_ptr().cast());
            } else if let Some(head) = p.q_head {
                p.q_head = (*head.as_ptr()).next;
            }
        }
    }

    unsafe { crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m)) };
    ptr::null_mut()
}

// original: wake_next_worker (htslib/thread_pool.c:654)
unsafe fn wake_next_worker(q: &mut HtsTpoolProcess, locked: bool) {
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return;
    };
    unsafe {
        if !locked {
            crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        }

        assert!(q.prev.is_some() && q.next.is_some());
        p.q_head = Some(NonNull::from(&mut *q));

        assert!(p.njobs >= q.n_input);

        let sig = p.t_stack_top.is_some()
            && (p.njobs as usize) > p.tsize.saturating_sub(p.nwaiting as usize)
            && q.n_processing < q.qsize - q.n_output;

        if let (true, Some(top)) = (sig, p.t_stack_top) {
            crate::htslib_rs::c_compat::pthread_cond_signal(ptr::addr_of_mut!(p.t[top].pending_c));
        }

        if !locked {
            crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
        }
    }
}

// original: hts_tpool_init (htslib/thread_pool.c:725)
pub unsafe fn hts_tpool_init(n: i32) -> *mut hts_tpool {
    let mut t_idx = 0usize;
    let mut stack_size: usize = 0;
    let mut pattr: crate::htslib_rs::c_compat::pthread_attr_t = unsafe { mem::zeroed() };
    let mut pattr_init_done = false;
    let Ok(n_usize) = usize::try_from(n) else {
        unsafe {
            *c_compat::__errno_location() = c_compat::ENOMEM;
        }
        return ptr::null_mut();
    };
    let mut p_box = unsafe { make_pool(n_usize) };
    p_box.t = Vec::with_capacity(n_usize);
    p_box.t_stack = vec![false; n_usize];
    let p_ptr = Box::into_raw(p_box);

    unsafe {
        let mut attr: crate::htslib_rs::c_compat::pthread_mutexattr_t = mem::zeroed();
        crate::htslib_rs::c_compat::pthread_mutexattr_init(&mut attr);
        crate::htslib_rs::c_compat::pthread_mutexattr_settype(
            &mut attr,
            crate::htslib_rs::c_compat::PTHREAD_MUTEX_RECURSIVE,
        );
        crate::htslib_rs::c_compat::pthread_mutex_init(ptr::addr_of_mut!((*p_ptr).pool_m), &attr);
        crate::htslib_rs::c_compat::pthread_mutexattr_destroy(&mut attr);

        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!((*p_ptr).pool_m));

        if crate::htslib_rs::c_compat::pthread_attr_init(&mut pattr) != 0 {
            goto_cleanup(&mut *p_ptr, t_idx, &mut pattr, pattr_init_done);
            return ptr::null_mut();
        }
        pattr_init_done = true;
        if crate::htslib_rs::c_compat::pthread_attr_getstacksize(&pattr, &mut stack_size) != 0 {
            goto_cleanup(&mut *p_ptr, t_idx, &mut pattr, pattr_init_done);
            return ptr::null_mut();
        }
        if stack_size < HTS_MIN_THREAD_STACK
            && crate::htslib_rs::c_compat::pthread_attr_setstacksize(
                &mut pattr,
                HTS_MIN_THREAD_STACK,
            ) != 0
        {
            goto_cleanup(&mut *p_ptr, t_idx, &mut pattr, pattr_init_done);
            return ptr::null_mut();
        }

        while t_idx < n_usize {
            (*p_ptr)
                .t
                .push(make_worker(NonNull::new_unchecked(p_ptr), t_idx));
            let w = ptr::addr_of_mut!((&mut (*p_ptr).t)[t_idx]);
            crate::htslib_rs::c_compat::pthread_cond_init(
                ptr::addr_of_mut!((*w).pending_c),
                ptr::null(),
            );
            if crate::htslib_rs::c_compat::pthread_create(
                ptr::addr_of_mut!((*w).tid),
                &pattr,
                tpool_worker,
                w.cast(),
            ) != 0
            {
                goto_cleanup(&mut *p_ptr, t_idx, &mut pattr, pattr_init_done);
                return ptr::null_mut();
            }
            t_idx += 1;
        }

        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!((*p_ptr).pool_m));
        crate::htslib_rs::c_compat::pthread_attr_destroy(&mut pattr);
    }

    p_ptr.cast()
}

unsafe fn goto_cleanup(
    p: &mut HtsTpool,
    t_idx: usize,
    pattr: &mut crate::htslib_rs::c_compat::pthread_attr_t,
    pattr_init_done: bool,
) {
    let save_errno = unsafe { *c_compat::__errno_location() };
    unsafe {
        p.shutdown = true;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
        for j in 0..t_idx {
            crate::htslib_rs::c_compat::pthread_join(p.t[j].tid, ptr::null_mut());
            crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(p.t[j].pending_c));
        }
        crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(p.pool_m));
        if pattr_init_done {
            crate::htslib_rs::c_compat::pthread_attr_destroy(pattr);
        }
        drop(Box::from_raw(ptr::from_mut(p)));
        *c_compat::__errno_location() = save_errno;
    }
}

// original: hts_tpool_size (htslib/thread_pool.c:819)
pub unsafe fn hts_tpool_size(p: *mut hts_tpool) -> i32 {
    unsafe { pool_ref(p).map_or(0, |p| p.tsize as i32) }
}

// original: hts_tpool_dispatch (htslib/thread_pool.c:829)
pub unsafe fn hts_tpool_dispatch(
    p: *mut hts_tpool,
    q: *mut hts_tpool_process,
    func: hts_tpool_worker,
    arg: *mut c_void,
) -> i32 {
    unsafe { hts_tpool_dispatch3(p, q, func, arg, None, None, 0) }
}

// original: hts_tpool_dispatch2 (htslib/thread_pool.c:841)
pub unsafe fn hts_tpool_dispatch2(
    p: *mut hts_tpool,
    q: *mut hts_tpool_process,
    func: hts_tpool_worker,
    arg: *mut c_void,
    nonblock: i32,
) -> i32 {
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
    nonblock: i32,
) -> i32 {
    let Some(p) = (unsafe { pool_mut(p) }) else {
        return -1;
    };
    let Some(q) = (unsafe { process_mut(q) }) else {
        return -1;
    };
    let p_nn = NonNull::from(&mut *p);
    let q_nn = NonNull::from(&mut *q);
    unsafe {
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));

        if (q.no_more_input || q.n_input >= q.qsize) && nonblock == 1 {
            crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
            *c_compat::__errno_location() = c_compat::EAGAIN;
            return -1;
        }

        let j = Box::new(HtsTpoolJob {
            func: exec_func,
            arg: NonNull::new(arg),
            job_cleanup,
            result_cleanup,
            p: p_nn,
            q: q_nn,
            serial: q.curr_serial,
        });
        q.curr_serial += 1;

        if nonblock == 0 {
            while (q.no_more_input || q.n_input >= q.qsize) && q.shutdown == 0 && !q.wake_dispatch {
                crate::htslib_rs::c_compat::pthread_cond_wait(
                    ptr::addr_of_mut!(q.input_not_full_c),
                    ptr::addr_of_mut!(p.pool_m),
                );
            }
            if q.no_more_input || q.shutdown != 0 {
                crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
                return -1;
            }
            if q.wake_dispatch {
                q.wake_dispatch = false;
            }
        }

        p.njobs += 1;
        q.n_input += 1;
        q.input.push_back(j);

        if q.shutdown == 0 {
            wake_next_worker(q, true);
        }

        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
    }
    0
}

// original: hts_tpool_wake_dispatch (htslib/thread_pool.c:923)
pub unsafe fn hts_tpool_wake_dispatch(q: *mut hts_tpool_process) {
    let Some(q) = (unsafe { process_mut(q) }) else {
        return;
    };
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return;
    };
    unsafe {
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        q.wake_dispatch = true;
        crate::htslib_rs::c_compat::pthread_cond_signal(ptr::addr_of_mut!(q.input_not_full_c));
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
    }
}

// original: hts_tpool_process_flush (htslib/thread_pool.c:941)
pub unsafe fn hts_tpool_process_flush(q: &mut HtsTpoolProcess) -> i32 {
    let Some(p) = (unsafe { q_pool_mut(q) }) else {
        return -1;
    };
    unsafe {
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));

        for i in 0..p.tsize {
            if p.t_stack[i] {
                crate::htslib_rs::c_compat::pthread_cond_signal(ptr::addr_of_mut!(
                    p.t[i].pending_c
                ));
            }
        }

        if q.qsize < q.n_output + q.n_input + q.n_processing {
            q.qsize = q.n_output + q.n_input + q.n_processing;
        }

        if q.shutdown != 0 {
            while q.n_processing != 0 {
                crate::htslib_rs::c_compat::pthread_cond_wait(
                    ptr::addr_of_mut!(q.none_processing_c),
                    ptr::addr_of_mut!(p.pool_m),
                );
            }
        }

        while q.shutdown == 0 && (q.n_input != 0 || q.n_processing != 0) {
            let mut now: libc::timeval = mem::zeroed();
            let mut timeout: libc::timespec = mem::zeroed();

            while q.n_input != 0 && q.shutdown == 0 {
                crate::htslib_rs::c_compat::gettimeofday(&mut now, ptr::null_mut());
                timeout.tv_sec = (now.tv_sec + 1) as _;
                timeout.tv_nsec = (now.tv_usec * 1000) as _;
                crate::htslib_rs::c_compat::pthread_cond_timedwait(
                    ptr::addr_of_mut!(q.input_empty_c),
                    ptr::addr_of_mut!(p.pool_m),
                    &timeout,
                );
            }

            while q.n_processing != 0 {
                crate::htslib_rs::c_compat::gettimeofday(&mut now, ptr::null_mut());
                timeout.tv_sec = (now.tv_sec + 1) as _;
                timeout.tv_nsec = (now.tv_usec * 1000) as _;
                crate::htslib_rs::c_compat::pthread_cond_timedwait(
                    ptr::addr_of_mut!(q.none_processing_c),
                    ptr::addr_of_mut!(p.pool_m),
                    &timeout,
                );
            }
            if q.shutdown != 0 {
                break;
            }
        }

        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
    }
    0
}

// original: hts_tpool_process_reset (htslib/thread_pool.c:1013)
pub unsafe fn hts_tpool_process_reset(q: &mut HtsTpoolProcess, _free_results: i32) -> i32 {
    let (mut input, mut output) = unsafe {
        let Some(p) = q_pool_mut(q) else {
            return -1;
        };
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        q.next_serial = i32::MAX as u64;

        let input = mem::take(&mut q.input);
        q.n_input = 0;

        let output = mem::take(&mut q.output);
        q.n_output = 0;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));
        (input, output)
    };

    unsafe {
        while let Some(j) = input.pop_front() {
            if let Some(cleanup) = j.job_cleanup {
                cleanup(j.arg.map_or(ptr::null_mut(), NonNull::as_ptr));
            }
        }

        while let Some(mut r) = output.pop_front() {
            if let Some(cleanup) = r.result_cleanup {
                cleanup(r.data.map_or(ptr::null_mut(), NonNull::as_ptr));
                r.data = None;
            }
        }

        if hts_tpool_process_flush(q) != 0 {
            return -1;
        }

        let Some(p) = q_pool_mut(q) else {
            return -1;
        };
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        output = mem::take(&mut q.output);
        q.n_output = 0;
        q.next_serial = 0;
        q.curr_serial = 0;
        crate::htslib_rs::c_compat::pthread_cond_signal(ptr::addr_of_mut!(q.input_not_full_c));
        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));

        while let Some(mut r) = output.pop_front() {
            if let Some(cleanup) = r.result_cleanup {
                cleanup(r.data.map_or(ptr::null_mut(), NonNull::as_ptr));
                r.data = None;
            }
        }
    }
    0
}

// original: hts_tpool_process_qsize (htslib/thread_pool.c:1081)
pub unsafe fn hts_tpool_process_qsize(q: *mut hts_tpool_process) -> i32 {
    unsafe { process_ref(q).map_or(0, |q| q.qsize) }
}

// original: hts_tpool_destroy (htslib/thread_pool.c:1089)
pub unsafe fn hts_tpool_destroy(p: *mut hts_tpool) {
    let Some(p_nn) = NonNull::new(p.cast::<HtsTpool>()) else {
        return;
    };
    let p = unsafe { &mut *p_nn.as_ptr() };
    unsafe {
        crate::htslib_rs::c_compat::pthread_mutex_lock(ptr::addr_of_mut!(p.pool_m));
        p.shutdown = true;

        for i in 0..p.tsize {
            crate::htslib_rs::c_compat::pthread_cond_signal(ptr::addr_of_mut!(p.t[i].pending_c));
        }

        crate::htslib_rs::c_compat::pthread_mutex_unlock(ptr::addr_of_mut!(p.pool_m));

        for i in 0..p.tsize {
            crate::htslib_rs::c_compat::pthread_join(p.t[i].tid, ptr::null_mut());
        }

        crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(p.pool_m));
        for i in 0..p.tsize {
            crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(p.t[i].pending_c));
        }

        drop(Box::from_raw(p_nn.as_ptr()));
    }
}

// original: hts_tpool_kill (htslib/thread_pool.c:1128)
pub unsafe fn hts_tpool_kill(p: *mut hts_tpool) {
    let Some(p_nn) = NonNull::new(p.cast::<HtsTpool>()) else {
        return;
    };
    let p = unsafe { &mut *p_nn.as_ptr() };
    unsafe {
        for i in 0..p.tsize {
            crate::htslib_rs::c_compat::pthread_kill(p.t[i].tid, libc::SIGINT);
        }

        crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(p.pool_m));
        for i in 0..p.tsize {
            crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(p.t[i].pending_c));
        }

        drop(Box::from_raw(p_nn.as_ptr()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static JOB_CLEANUPS: AtomicUsize = AtomicUsize::new(0);
    static RESULT_CLEANUPS: AtomicUsize = AtomicUsize::new(0);
    static SQUAREB_DISPATCH_FAILURES: AtomicUsize = AtomicUsize::new(0);
    static UNORDERED_SQUARE_JOBS: AtomicUsize = AtomicUsize::new(0);
    static WORKER_ID_FAILURES: AtomicUsize = AtomicUsize::new(0);
    static TEST_FAILURE_MARKER: u8 = 0;
    static TEST_MARKER_01: u8 = 0;
    static TEST_MARKER_02: u8 = 0;
    static TEST_MARKER_11: u8 = 0;
    static TEST_MARKER_22: u8 = 0;
    static TEST_MARKER_55: u8 = 0;
    static TEST_MARKER_77: u8 = 0;
    static TEST_MARKER_1234: u8 = 0;

    fn static_marker_ptr(marker: &'static u8) -> *mut c_void {
        std::ptr::from_ref(marker).cast::<c_void>().cast_mut()
    }

    fn test_failure_return() -> *mut c_void {
        static_marker_ptr(&TEST_FAILURE_MARKER)
    }

    fn test_marker_ptr(value: usize) -> *mut c_void {
        match value {
            0x01 => static_marker_ptr(&TEST_MARKER_01),
            0x02 => static_marker_ptr(&TEST_MARKER_02),
            0x11 => static_marker_ptr(&TEST_MARKER_11),
            0x22 => static_marker_ptr(&TEST_MARKER_22),
            0x55 => static_marker_ptr(&TEST_MARKER_55),
            0x77 => static_marker_ptr(&TEST_MARKER_77),
            0x1234 => static_marker_ptr(&TEST_MARKER_1234),
            _ => panic!("unknown test marker {value:#x}"),
        }
    }

    unsafe fn test_pool() -> HtsTpool {
        *unsafe { make_pool(0) }
    }

    unsafe fn test_process(pool: &mut HtsTpool, qsize: i32, in_only: i32) -> HtsTpoolProcess {
        *unsafe { make_process(Some(NonNull::from(pool)), qsize, in_only != 0) }
    }

    fn opt_ptr<T>(ptr: *mut T) -> Option<NonNull<T>> {
        NonNull::new(ptr)
    }

    #[repr(C)]
    struct SquareBOpt {
        pool: *mut hts_tpool,
        queue: *mut hts_tpool_process,
        n: i32,
    }

    #[repr(C)]
    struct PipeOpt {
        pool: *mut hts_tpool,
        q1: *mut hts_tpool_process,
        q2: *mut hts_tpool_process,
        q3: *mut hts_tpool_process,
        n: i32,
        failures: AtomicUsize,
        output_next: AtomicUsize,
        output_checksum: AtomicUsize,
    }

    #[repr(C)]
    struct PipeJob {
        opt: *mut PipeOpt,
        x: u32,
        eof: i32,
    }

    #[repr(C)]
    struct TestWorkerArg {
        pool: *mut hts_tpool,
        value: i32,
        delay_us: u32,
    }

    #[derive(Debug, Eq, PartialEq)]
    enum TestMainMode {
        Usage,
        Unknown,
        Unordered(i32),
        OrderedNonblocking(i32),
        OrderedDispatchThread(i32),
        Pipe(i32),
    }

    fn classify_test_main_args(args: &[&str]) -> TestMainMode {
        if args.len() < 3 {
            return TestMainMode::Usage;
        }

        let nthreads = args[2].parse::<i32>().unwrap_or(0);
        match args[1] {
            "unordered" => TestMainMode::Unordered(nthreads),
            "ordered1" => TestMainMode::OrderedNonblocking(nthreads),
            "ordered2" => TestMainMode::OrderedDispatchThread(nthreads),
            "pipe" => TestMainMode::Pipe(nthreads),
            _ => TestMainMode::Unknown,
        }
    }

    unsafe fn init_test_mutex(mutex: *mut crate::htslib_rs::c_compat::pthread_mutex_t) {
        let mut attr: crate::htslib_rs::c_compat::pthread_mutexattr_t = unsafe { mem::zeroed() };
        unsafe {
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutexattr_init(&mut attr),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutexattr_settype(
                    &mut attr,
                    crate::htslib_rs::c_compat::PTHREAD_MUTEX_RECURSIVE
                ),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_init(mutex, &attr),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutexattr_destroy(&mut attr),
                0
            );
        }
    }

    unsafe fn init_test_process_conds(queue: &mut HtsTpoolProcess) {
        unsafe {
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_cond_init(
                    ptr::addr_of_mut!(queue.output_avail_c),
                    ptr::null()
                ),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_cond_init(
                    ptr::addr_of_mut!(queue.input_not_full_c),
                    ptr::null()
                ),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_cond_init(
                    ptr::addr_of_mut!(queue.input_empty_c),
                    ptr::null()
                ),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_cond_init(
                    ptr::addr_of_mut!(queue.none_processing_c),
                    ptr::null()
                ),
                0
            );
        }
    }

    unsafe fn destroy_test_process_conds(queue: &mut HtsTpoolProcess) {
        unsafe {
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(
                    queue.output_avail_c
                )),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(
                    queue.input_not_full_c
                )),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(
                    queue.input_empty_c
                )),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_cond_destroy(ptr::addr_of_mut!(
                    queue.none_processing_c
                )),
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
        let arg = unsafe { Box::from_raw(arg) };
        let pool = arg.pool;
        let value = arg.value;
        let delay_us = arg.delay_us;

        if unsafe { hts_tpool_worker_id(pool) } < 0 {
            WORKER_ID_FAILURES.fetch_add(1, Ordering::SeqCst);
        }
        if delay_us != 0 {
            unsafe { crate::htslib_rs::c_compat::usleep(delay_us) };
        }

        Box::into_raw(Box::new(value * value)).cast()
    }

    unsafe extern "C" fn unordered_square_in_only(arg: *mut c_void) -> *mut c_void {
        let value = unsafe { *Box::from_raw(arg.cast::<i32>()) };
        let square = value * value;
        assert!(square >= 0);
        UNORDERED_SQUARE_JOBS.fetch_add(1, Ordering::SeqCst);
        ptr::null_mut()
    }

    unsafe extern "C" fn ordered_square_with_slow_serial(arg: *mut c_void) -> *mut c_void {
        let value = unsafe { *Box::from_raw(arg.cast::<i32>()) };
        if value & 31 == 31 {
            unsafe { crate::htslib_rs::c_compat::usleep(50_000) };
        }

        Box::into_raw(Box::new(if value < 0 {
            -(value * value)
        } else {
            value * value
        }))
        .cast()
    }

    extern "C" fn test_squareb_dispatcher(arg: *mut c_void) -> *mut c_void {
        let opt = arg.cast::<SquareBOpt>();
        let pool = unsafe { (*opt).pool };
        let queue = unsafe { (*opt).queue };
        let n = unsafe { (*opt).n };

        for value in 0..n {
            let job_arg = Box::into_raw(Box::new(value));
            if unsafe {
                hts_tpool_dispatch(
                    pool,
                    queue,
                    Some(ordered_square_with_slow_serial),
                    job_arg.cast(),
                )
            } != 0
            {
                unsafe { drop(Box::from_raw(job_arg)) };
                SQUAREB_DISPATCH_FAILURES.fetch_add(1, Ordering::SeqCst);
                return ptr::null_mut();
            }
        }

        let sentinel = Box::into_raw(Box::new(-1));
        if unsafe {
            hts_tpool_dispatch(
                pool,
                queue,
                Some(ordered_square_with_slow_serial),
                sentinel.cast(),
            )
        } != 0
        {
            unsafe { drop(Box::from_raw(sentinel)) };
            SQUAREB_DISPATCH_FAILURES.fetch_add(1, Ordering::SeqCst);
        }

        ptr::null_mut()
    }

    extern "C" fn pipe_input_thread(arg: *mut c_void) -> *mut c_void {
        let opt = arg.cast::<PipeOpt>();
        let n = unsafe { (*opt).n };

        for value in 1..=n {
            let job = Box::into_raw(Box::new(PipeJob {
                opt,
                x: value as u32,
                eof: i32::from(value == n),
            }));

            unsafe {
                if hts_tpool_dispatch((*opt).pool, (*opt).q1, Some(pipe_stage1), job.cast()) != 0 {
                    drop(Box::from_raw(job));
                    (*opt).failures.fetch_add(1, Ordering::SeqCst);
                    return test_failure_return();
                }
            }
        }

        ptr::null_mut()
    }

    unsafe extern "C" fn pipe_stage1(arg: *mut c_void) -> *mut c_void {
        let job = arg.cast::<PipeJob>();
        unsafe {
            (*job).x <<= 8;
            crate::htslib_rs::c_compat::usleep(((*job).x & 3) * 1_000);
        }
        arg
    }

    extern "C" fn pipe_stage1to2(arg: *mut c_void) -> *mut c_void {
        let opt = arg.cast::<PipeOpt>();

        loop {
            let result = unsafe { hts_tpool_next_result_wait((*opt).q1) };
            if result.is_null() {
                return ptr::null_mut();
            }

            let job = unsafe { hts_tpool_result_data(&mut *result).cast::<PipeJob>() };
            unsafe {
                hts_tpool_delete_result(result, 0);
                if hts_tpool_dispatch((*(*job).opt).pool, (*opt).q2, Some(pipe_stage2), job.cast())
                    != 0
                {
                    drop(Box::from_raw(job));
                    (*opt).failures.fetch_add(1, Ordering::SeqCst);
                    return test_failure_return();
                }
                if (*job).eof != 0 {
                    return ptr::null_mut();
                }
            }
        }
    }

    unsafe extern "C" fn pipe_stage2(arg: *mut c_void) -> *mut c_void {
        let job = arg.cast::<PipeJob>();
        unsafe {
            (*job).x <<= 8;
            crate::htslib_rs::c_compat::usleep(((*job).x & 7) * 1_000);
        }
        arg
    }

    extern "C" fn pipe_stage2to3(arg: *mut c_void) -> *mut c_void {
        let opt = arg.cast::<PipeOpt>();

        loop {
            let result = unsafe { hts_tpool_next_result_wait((*opt).q2) };
            if result.is_null() {
                return ptr::null_mut();
            }

            let job = unsafe { hts_tpool_result_data(&mut *result).cast::<PipeJob>() };
            unsafe {
                hts_tpool_delete_result(result, 0);
                if hts_tpool_dispatch((*(*job).opt).pool, (*opt).q3, Some(pipe_stage3), job.cast())
                    != 0
                {
                    drop(Box::from_raw(job));
                    (*opt).failures.fetch_add(1, Ordering::SeqCst);
                    return test_failure_return();
                }
                if (*job).eof != 0 {
                    return ptr::null_mut();
                }
            }
        }
    }

    unsafe extern "C" fn pipe_stage3(arg: *mut c_void) -> *mut c_void {
        let job = arg.cast::<PipeJob>();
        unsafe {
            crate::htslib_rs::c_compat::usleep(((*job).x & 3) * 1_000);
            (*job).x <<= 8;
        }
        arg
    }

    extern "C" fn pipe_output_thread(arg: *mut c_void) -> *mut c_void {
        let opt = arg.cast::<PipeOpt>();

        loop {
            let result = unsafe { hts_tpool_next_result_wait((*opt).q3) };
            if result.is_null() {
                return ptr::null_mut();
            }

            let job = unsafe { hts_tpool_result_data(&mut *result).cast::<PipeJob>() };
            let (x, eof) = unsafe { ((*job).x, (*job).eof != 0) };
            let expected = unsafe { (*opt).output_next.fetch_add(1, Ordering::SeqCst) + 1 };
            if x != ((expected as u32) << 24) {
                unsafe { (*opt).failures.fetch_add(1, Ordering::SeqCst) };
            }
            unsafe {
                (*opt)
                    .output_checksum
                    .fetch_add(x as usize, Ordering::SeqCst);
                drop(Box::from_raw(job));
                hts_tpool_delete_result(result, 0);
            }

            if eof {
                return ptr::null_mut();
            }
        }
    }

    unsafe fn alloc_worker_arg(
        pool: *mut hts_tpool,
        value: i32,
        delay_us: u32,
    ) -> *mut c_void {
        Box::into_raw(Box::new(TestWorkerArg {
            pool,
            value,
            delay_us,
        }))
        .cast()
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
        let mut result = Box::new(HtsTpoolResult {
            result_cleanup: Some(count_result_cleanup),
            serial: 0,
            data: NonNull::new(test_marker_ptr(0x1234)),
        });

        RESULT_CLEANUPS.store(0, Ordering::SeqCst);
        assert_eq!(
            hts_tpool_result_data(&mut result),
            test_marker_ptr(0x1234)
        );
        assert_eq!(RESULT_CLEANUPS.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn thread_pool_wrappers_create_process_and_report_sizes() {
        unsafe {
            let pool = hts_tpool_init(1);
            assert!(!pool.is_null());
            assert_eq!(hts_tpool_size(pool), 1);

            let queue = hts_tpool_process_init(pool, 2, 0);
            assert!(!queue.is_null());
            assert_eq!(hts_tpool_process_empty(&mut *queue), 1);
            assert_eq!(hts_tpool_process_len(&mut *queue), 0);
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
            assert_eq!(*c_compat::__errno_location(), c_compat::ENOMEM);
        }
    }

    #[test]
    fn tpool_kill_releases_zero_worker_pool_without_signalling_threads() {
        unsafe {
            let pool = Box::into_raw(make_pool(0));
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_init(
                    ptr::addr_of_mut!((*pool).pool_m),
                    ptr::null()
                ),
                0
            );

            hts_tpool_kill(pool.cast());
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
                let data = hts_tpool_result_data(&mut *result).cast::<i32>();
                assert_eq!(*data, value * value);
                drop(Box::from_raw(data));
                hts_tpool_delete_result(result, 0);
            }

            assert_eq!(WORKER_ID_FAILURES.load(Ordering::SeqCst), 0);
            assert_eq!(hts_tpool_process_empty(&mut *queue), 1);

            hts_tpool_process_destroy(queue);
            hts_tpool_destroy(pool);
        }
    }

    #[test]
    fn original_test_square_nonblocking_dispatch_drains_ordered_results() {
        const TASK_SIZE: i32 = 96;

        unsafe {
            let pool = hts_tpool_init(4);
            assert!(!pool.is_null());
            let queue = hts_tpool_process_init(pool, hts_tpool_size(pool) * 2, 0);
            assert!(!queue.is_null());

            let mut next_result_value = 0;
            for value in 0..TASK_SIZE {
                let arg = Box::into_raw(Box::new(value));

                loop {
                    let dispatch_rc = hts_tpool_dispatch2(
                        pool,
                        queue,
                        Some(ordered_square_with_slow_serial),
                        arg.cast(),
                        1,
                    );

                    let result = hts_tpool_next_result(queue);
                    if !result.is_null() {
                        let data = hts_tpool_result_data(&mut *result).cast::<i32>();
                        assert_eq!(*data, next_result_value * next_result_value);
                        next_result_value += 1;
                        drop(Box::from_raw(data));
                        hts_tpool_delete_result(result, 0);
                    }

                    if dispatch_rc == 0 {
                        break;
                    }
                    assert_eq!(dispatch_rc, -1);
                    assert_eq!(*c_compat::__errno_location(), c_compat::EAGAIN);
                    crate::htslib_rs::c_compat::usleep(1_000);
                }
            }

            assert_eq!(hts_tpool_process_flush(&mut *queue), 0);

            loop {
                let result = hts_tpool_next_result(queue);
                if result.is_null() {
                    break;
                }
                let data = hts_tpool_result_data(&mut *result).cast::<i32>();
                assert_eq!(*data, next_result_value * next_result_value);
                next_result_value += 1;
                drop(Box::from_raw(data));
                hts_tpool_delete_result(result, 0);
            }

            assert_eq!(next_result_value, TASK_SIZE);
            assert_eq!(hts_tpool_process_empty(&mut *queue), 1);

            hts_tpool_process_destroy(queue);
            hts_tpool_destroy(pool);
        }
    }

    #[test]
    fn original_test_square_u_in_only_jobs_flush_before_destroy() {
        const TASK_SIZE: i32 = 96;

        unsafe {
            UNORDERED_SQUARE_JOBS.store(0, Ordering::SeqCst);

            let pool = hts_tpool_init(4);
            assert!(!pool.is_null());
            let queue = hts_tpool_process_init(pool, 8, 1);
            assert!(!queue.is_null());

            for value in 0..TASK_SIZE {
                let arg = Box::into_raw(Box::new(value));
                assert_eq!(
                    hts_tpool_dispatch(pool, queue, Some(unordered_square_in_only), arg.cast()),
                    0
                );
            }

            assert_eq!(hts_tpool_process_flush(&mut *queue), 0);
            assert_eq!(
                UNORDERED_SQUARE_JOBS.load(Ordering::SeqCst),
                TASK_SIZE as usize
            );
            assert_eq!(hts_tpool_process_empty(&mut *queue), 1);

            hts_tpool_process_destroy(queue);
            hts_tpool_destroy(pool);
        }
    }

    #[test]
    fn original_test_squareb_dispatch_thread_consumes_until_sentinel() {
        const TASK_SIZE: i32 = 96;

        unsafe {
            SQUAREB_DISPATCH_FAILURES.store(0, Ordering::SeqCst);

            let pool = hts_tpool_init(4);
            assert!(!pool.is_null());
            let queue = hts_tpool_process_init(pool, hts_tpool_size(pool) * 2, 0);
            assert!(!queue.is_null());

            let mut opt = SquareBOpt {
                pool,
                queue,
                n: TASK_SIZE,
            };
            let mut tid: crate::htslib_rs::c_compat::pthread_t = mem::zeroed();
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_create(
                    ptr::addr_of_mut!(tid),
                    ptr::null(),
                    test_squareb_dispatcher,
                    ptr::addr_of_mut!(opt).cast(),
                ),
                0
            );

            let mut next_result_value = 0;
            loop {
                let result = hts_tpool_next_result_wait(queue);
                assert!(!result.is_null());
                let data = hts_tpool_result_data(&mut *result).cast::<i32>();
                let value = *data;
                drop(Box::from_raw(data));
                hts_tpool_delete_result(result, 0);

                if value == -1 {
                    break;
                }
                assert_eq!(value, next_result_value * next_result_value);
                next_result_value += 1;
            }

            assert_eq!(next_result_value, TASK_SIZE);
            assert_eq!(hts_tpool_process_flush(&mut *queue), 0);
            assert!(hts_tpool_next_result(queue).is_null());

            hts_tpool_process_destroy(queue);
            hts_tpool_destroy(pool);
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_join(tid, ptr::null_mut()),
                0
            );
            assert_eq!(SQUAREB_DISPATCH_FAILURES.load(Ordering::SeqCst), 0);
        }
    }

    #[test]
    fn original_test_pipe_runs_three_ordered_stages_to_eof() {
        const TASK_SIZE: i32 = 24;

        unsafe {
            let pool = hts_tpool_init(4);
            assert!(!pool.is_null());
            let qsize = hts_tpool_size(pool) * 2;
            let q1 = hts_tpool_process_init(pool, qsize, 0);
            let q2 = hts_tpool_process_init(pool, qsize, 0);
            let q3 = hts_tpool_process_init(pool, qsize, 0);
            assert!(!q1.is_null());
            assert!(!q2.is_null());
            assert!(!q3.is_null());

            let mut opt = PipeOpt {
                pool,
                q1,
                q2,
                q3,
                n: TASK_SIZE,
                failures: AtomicUsize::new(0),
                output_next: AtomicUsize::new(0),
                output_checksum: AtomicUsize::new(0),
            };

            let mut input_tid: crate::htslib_rs::c_compat::pthread_t = mem::zeroed();
            let mut stage1_tid: crate::htslib_rs::c_compat::pthread_t = mem::zeroed();
            let mut stage2_tid: crate::htslib_rs::c_compat::pthread_t = mem::zeroed();
            let mut output_tid: crate::htslib_rs::c_compat::pthread_t = mem::zeroed();
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_create(
                    ptr::addr_of_mut!(input_tid),
                    ptr::null(),
                    pipe_input_thread,
                    ptr::addr_of_mut!(opt).cast(),
                ),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_create(
                    ptr::addr_of_mut!(stage1_tid),
                    ptr::null(),
                    pipe_stage1to2,
                    ptr::addr_of_mut!(opt).cast(),
                ),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_create(
                    ptr::addr_of_mut!(stage2_tid),
                    ptr::null(),
                    pipe_stage2to3,
                    ptr::addr_of_mut!(opt).cast(),
                ),
                0
            );
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_create(
                    ptr::addr_of_mut!(output_tid),
                    ptr::null(),
                    pipe_output_thread,
                    ptr::addr_of_mut!(opt).cast(),
                ),
                0
            );

            let mut retv: *mut c_void = ptr::null_mut();
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_join(input_tid, ptr::addr_of_mut!(retv)),
                0
            );
            assert!(retv.is_null());
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_join(stage1_tid, ptr::addr_of_mut!(retv)),
                0
            );
            assert!(retv.is_null());
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_join(stage2_tid, ptr::addr_of_mut!(retv)),
                0
            );
            assert!(retv.is_null());
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_join(output_tid, ptr::addr_of_mut!(retv)),
                0
            );
            assert!(retv.is_null());

            assert_eq!(opt.failures.load(Ordering::SeqCst), 0);
            assert_eq!(opt.output_next.load(Ordering::SeqCst), TASK_SIZE as usize);
            let expected_checksum = ((TASK_SIZE as usize) * ((TASK_SIZE + 1) as usize) / 2) << 24;
            assert_eq!(
                opt.output_checksum.load(Ordering::SeqCst),
                expected_checksum
            );
            assert_eq!(hts_tpool_process_empty(&mut *q1), 1);
            assert_eq!(hts_tpool_process_empty(&mut *q2), 1);
            assert_eq!(hts_tpool_process_empty(&mut *q3), 1);

            hts_tpool_process_destroy(q1);
            hts_tpool_process_destroy(q2);
            hts_tpool_process_destroy(q3);
            hts_tpool_destroy(pool);
        }
    }

    #[test]
    fn original_thread_pool_test_main_policy_routes_only_demo_modes() {
        assert_eq!(
            classify_test_main_args(&["thread_pool"]),
            TestMainMode::Usage
        );
        assert_eq!(
            classify_test_main_args(&["thread_pool", "unknown", "4"]),
            TestMainMode::Unknown
        );
        assert_eq!(
            classify_test_main_args(&["thread_pool", "unordered", "2"]),
            TestMainMode::Unordered(2)
        );
        assert_eq!(
            classify_test_main_args(&["thread_pool", "ordered1", "3"]),
            TestMainMode::OrderedNonblocking(3)
        );
        assert_eq!(
            classify_test_main_args(&["thread_pool", "ordered2", "4"]),
            TestMainMode::OrderedDispatchThread(4)
        );
        assert_eq!(
            classify_test_main_args(&["thread_pool", "pipe", "5"]),
            TestMainMode::Pipe(5)
        );
        assert_eq!(
            classify_test_main_args(&["thread_pool", "pipe", "not-a-number"]),
            TestMainMode::Pipe(0)
        );
    }

    #[test]
    fn process_accounting_wrappers_report_combined_queue_state_and_refcount() {
        unsafe {
            let mut pool = test_pool();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));

            let mut queue = test_process(&mut pool, 7, 0);
            queue.qsize = 7;
            queue.n_input = 2;
            queue.n_processing = 3;
            queue.n_output = 4;
            queue.ref_count = 1;

            assert_eq!(hts_tpool_process_empty(&mut queue), 0);
            assert_eq!(hts_tpool_process_len(&mut queue), 4);
            assert_eq!(hts_tpool_process_sz(&mut queue), 9);
            assert_eq!(hts_tpool_process_qsize(ptr::addr_of_mut!(queue).cast()), 7);

            hts_tpool_process_ref_incr(ptr::addr_of_mut!(queue).cast());
            assert_eq!(queue.ref_count, 2);
            hts_tpool_process_ref_decr(ptr::addr_of_mut!(queue).cast());
            assert_eq!(queue.ref_count, 1);

            queue.n_input = 0;
            queue.n_processing = 0;
            queue.n_output = 0;
            assert_eq!(hts_tpool_process_empty(&mut queue), 1);
            assert_eq!(hts_tpool_process_sz(&mut queue), 0);

            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn process_attach_and_detach_maintain_circular_queue_ring() {
        unsafe {
            let mut pool = test_pool();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));

            let mut first = test_process(&mut pool, 0, 0);
            let mut second = test_process(&mut pool, 0, 0);
            let first_ptr = ptr::addr_of_mut!(first);
            let second_ptr = ptr::addr_of_mut!(second);

            hts_tpool_process_attach(&mut pool, &mut first);
            assert_eq!(pool.q_head, opt_ptr(first_ptr));
            assert_eq!(first.next, opt_ptr(first_ptr));
            assert_eq!(first.prev, opt_ptr(first_ptr));

            hts_tpool_process_attach(&mut pool, &mut second);
            assert_eq!(pool.q_head, opt_ptr(second_ptr));
            assert_eq!(second.next, opt_ptr(first_ptr));
            assert_eq!(second.prev, opt_ptr(first_ptr));
            assert_eq!(first.next, opt_ptr(second_ptr));
            assert_eq!(first.prev, opt_ptr(second_ptr));

            hts_tpool_process_detach(ptr::addr_of_mut!(pool).cast(), first_ptr.cast());
            assert_eq!(pool.q_head, opt_ptr(second_ptr));
            assert_eq!(second.next, opt_ptr(second_ptr));
            assert_eq!(second.prev, opt_ptr(second_ptr));
            assert!(first.next.is_none());
            assert!(first.prev.is_none());

            hts_tpool_process_detach(ptr::addr_of_mut!(pool).cast(), first_ptr.cast());
            assert_eq!(pool.q_head, opt_ptr(second_ptr));
            assert_eq!(second.next, opt_ptr(second_ptr));
            assert_eq!(second.prev, opt_ptr(second_ptr));

            hts_tpool_process_detach(ptr::addr_of_mut!(pool).cast(), second_ptr.cast());
            assert!(pool.q_head.is_none());
            assert!(second.next.is_none());
            assert!(second.prev.is_none());

            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn process_destroy_detaches_and_shutdowns_queue_with_outstanding_reference() {
        unsafe {
            let mut pool = test_pool();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));

            let mut first = test_process(&mut pool, 4, 0);
            init_test_process_conds(&mut first);
            first.ref_count = 2;

            let mut second = test_process(&mut pool, 4, 0);
            second.ref_count = 1;

            let first_ptr = ptr::addr_of_mut!(first);
            let second_ptr = ptr::addr_of_mut!(second);
            hts_tpool_process_attach(&mut pool, &mut first);
            hts_tpool_process_attach(&mut pool, &mut second);
            assert_eq!(pool.q_head, opt_ptr(second_ptr));

            hts_tpool_process_destroy(first_ptr.cast());

            assert!(first.no_more_input);
            assert_eq!(first.shutdown, 1);
            assert_eq!(first.ref_count, 1);
            assert!(first.next.is_none());
            assert!(first.prev.is_none());
            assert_eq!(pool.q_head, opt_ptr(second_ptr));
            assert_eq!(second.next, opt_ptr(second_ptr));
            assert_eq!(second.prev, opt_ptr(second_ptr));

            destroy_test_process_conds(&mut first);
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn next_result_locked_returns_results_in_serial_order() {
        unsafe {
            let mut pool = test_pool();
            let mut queue = test_process(&mut pool, 0, 0);
            queue.next_serial = 0;
            queue.n_output = 2;

            let mut later = Box::new(HtsTpoolResult {
                result_cleanup: None,
                serial: 1,
                data: NonNull::new(test_marker_ptr(0x11)),
            });
            let later_ptr = ptr::addr_of_mut!(*later);
            let mut first = Box::new(HtsTpoolResult {
                result_cleanup: None,
                serial: 0,
                data: NonNull::new(test_marker_ptr(0x22)),
            });
            let first_ptr = ptr::addr_of_mut!(*first);
            queue.output.push_back(first);
            queue.output.push_back(later);

            let r0 = hts_tpool_next_result_locked(&mut queue).unwrap();
            assert_eq!(ptr::from_ref(&*r0).cast_mut(), first_ptr);
            assert_eq!(queue.next_serial, 1);
            assert_eq!(queue.n_output, 1);
            assert_eq!(queue.output.front().map(|r| r.serial), Some(1));
            drop(r0);

            let r1 = hts_tpool_next_result_locked(&mut queue).unwrap();
            assert_eq!(ptr::from_ref(&*r1).cast_mut(), later_ptr);
            assert_eq!(queue.next_serial, 2);
            assert_eq!(queue.n_output, 0);
            assert!(queue.output.is_empty());
            drop(r1);
        }
    }

    #[test]
    fn next_result_locked_waits_for_missing_serial_and_honors_shutdown() {
        unsafe {
            let mut pool = test_pool();
            let mut queue = test_process(&mut pool, 0, 0);
            queue.next_serial = 0;
            queue.n_output = 1;

            let later = Box::new(HtsTpoolResult {
                result_cleanup: None,
                serial: 1,
                data: None,
            });
            let later_ptr = ptr::from_ref(&*later).cast_mut();
            queue.output.push_back(later);

            assert!(hts_tpool_next_result_locked(&mut queue).is_none());
            assert_eq!(queue.next_serial, 0);
            assert_eq!(queue.n_output, 1);
            assert_eq!(
                queue.output.front().map(|r| ptr::from_ref(&**r).cast_mut()),
                Some(later_ptr)
            );

            queue.shutdown = 1;
            assert!(hts_tpool_next_result_locked(&mut queue).is_none());
            assert_eq!(queue.next_serial, 0);
            assert_eq!(queue.n_output, 1);
        }
    }

    #[test]
    fn next_result_locked_removes_matching_serial_from_middle() {
        unsafe {
            let mut pool = test_pool();
            let mut queue = test_process(&mut pool, 0, 0);
            queue.next_serial = 1;
            queue.n_output = 3;

            let tail = Box::new(HtsTpoolResult {
                result_cleanup: None,
                serial: 2,
                data: NonNull::new(test_marker_ptr(0x22)),
            });
            let tail_ptr = ptr::from_ref(&*tail).cast_mut();
            let mut middle = Box::new(HtsTpoolResult {
                result_cleanup: None,
                serial: 1,
                data: NonNull::new(test_marker_ptr(0x11)),
            });
            let middle_ptr = ptr::addr_of_mut!(*middle);
            let head = Box::new(HtsTpoolResult {
                result_cleanup: None,
                serial: 0,
                data: None,
            });
            queue.output.push_back(head);
            queue.output.push_back(middle);
            queue.output.push_back(tail);

            let r = hts_tpool_next_result_locked(&mut queue).unwrap();
            assert_eq!(ptr::from_ref(&*r).cast_mut(), middle_ptr);
            assert_eq!(queue.next_serial, 2);
            assert_eq!(queue.n_output, 2);
            assert_eq!(queue.output.front().map(|r| r.serial), Some(0));
            assert_eq!(
                queue.output.back().map(|r| ptr::from_ref(&**r).cast_mut()),
                Some(tail_ptr)
            );
            drop(r);
        }
    }

    #[test]
    fn process_shutdown_sets_flag_and_blocks_pending_output_delivery() {
        unsafe {
            let mut pool = test_pool();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));

            let mut queue = test_process(&mut pool, 0, 0);
            init_test_process_conds(&mut queue);
            queue.next_serial = 0;
            queue.n_output = 1;

            let result = Box::new(HtsTpoolResult {
                result_cleanup: None,
                serial: 0,
                data: NonNull::new(test_marker_ptr(0x55)),
            });
            let result_ptr = ptr::from_ref(&*result).cast_mut();
            queue.output.push_back(result);

            hts_tpool_process_shutdown(ptr::addr_of_mut!(queue).cast());
            assert_eq!(
                hts_tpool_process_is_shutdown(ptr::addr_of_mut!(queue).cast()),
                1
            );
            assert!(hts_tpool_next_result(ptr::addr_of_mut!(queue).cast()).is_null());
            assert_eq!(queue.next_serial, 0);
            assert_eq!(queue.n_output, 1);
            assert_eq!(
                queue.output.front().map(|r| ptr::from_ref(&**r).cast_mut()),
                Some(result_ptr)
            );

            destroy_test_process_conds(&mut queue);
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn add_result_in_only_queue_discards_data_after_finishing_job() {
        unsafe {
            let mut pool = test_pool();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));

            let mut queue = test_process(&mut pool, 1, 1);
            init_test_process_conds(&mut queue);
            queue.n_processing = 1;

            let job = HtsTpoolJob {
                func: None,
                arg: None,
                job_cleanup: None,
                result_cleanup: Some(count_result_cleanup),
                p: NonNull::new(ptr::addr_of_mut!(pool)).unwrap(),
                q: NonNull::new(ptr::addr_of_mut!(queue)).unwrap(),
                serial: 0,
            };

            RESULT_CLEANUPS.store(0, Ordering::SeqCst);
            assert_eq!(
                hts_tpool_add_result(Box::new(job), NonNull::new(test_marker_ptr(0x77))),
                0
            );
            assert_eq!(queue.n_processing, 0);
            assert_eq!(queue.n_output, 0);
            assert!(queue.output.is_empty());
            assert_eq!(RESULT_CLEANUPS.load(Ordering::SeqCst), 0);

            destroy_test_process_conds(&mut queue);
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn dispatch3_nonblock_full_queue_sets_eagain_without_state_change() {
        unsafe {
            let mut pool = test_pool();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));

            let mut queue = test_process(&mut pool, 1, 0);
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
            assert_eq!(*c_compat::__errno_location(), c_compat::EAGAIN);
            assert_eq!(queue.curr_serial, 7);
            assert_eq!(queue.n_input, 1);
            assert!(queue.input.is_empty());
            assert_eq!(pool.njobs, 0);

            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn dispatch3_wake_dispatch_allows_one_blocking_enqueue_on_full_queue() {
        unsafe {
            let mut pool = test_pool();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));

            let mut queue = test_process(&mut pool, 1, 0);
            init_test_process_conds(&mut queue);
            queue.n_input = 1;
            queue.curr_serial = 41;
            queue.wake_dispatch = true;
            queue.next = opt_ptr(ptr::addr_of_mut!(queue));
            queue.prev = opt_ptr(ptr::addr_of_mut!(queue));
            pool.q_head = opt_ptr(ptr::addr_of_mut!(queue));
            pool.njobs = queue.n_input;
            let p_nn = NonNull::new(ptr::addr_of_mut!(pool)).unwrap();
            let q_nn = NonNull::new(ptr::addr_of_mut!(queue)).unwrap();

            queue.input.push_back(Box::new(HtsTpoolJob {
                func: None,
                arg: NonNull::new(test_marker_ptr(0x01)),
                job_cleanup: None,
                result_cleanup: None,
                p: p_nn,
                q: q_nn,
                serial: 40,
            }));

            assert_eq!(
                hts_tpool_dispatch3(
                    ptr::addr_of_mut!(pool).cast(),
                    ptr::addr_of_mut!(queue).cast(),
                    None,
                    test_marker_ptr(0x02),
                    None,
                    Some(count_result_cleanup),
                    0,
                ),
                0
            );

            assert!(!queue.wake_dispatch);
            assert_eq!(queue.curr_serial, 42);
            assert_eq!(queue.n_input, 2);
            assert_eq!(pool.njobs, 2);
            assert_eq!(queue.input.front().unwrap().serial, 40);
            assert_eq!(queue.input.back().unwrap().serial, 41);
            assert!(queue.input.back().unwrap().result_cleanup.is_some());

            assert_eq!(
                hts_tpool_process_reset(&mut queue, 0),
                0
            );
            destroy_test_process_conds(&mut queue);
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn dispatch3_blocking_shutdown_after_allocation_returns_without_enqueue() {
        unsafe {
            let mut pool = test_pool();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));

            let mut queue = test_process(&mut pool, 1, 0);
            queue.n_input = 1;
            queue.shutdown = 1;
            queue.curr_serial = 7;

            JOB_CLEANUPS.store(0, Ordering::SeqCst);
            RESULT_CLEANUPS.store(0, Ordering::SeqCst);
            let ret = hts_tpool_dispatch3(
                ptr::addr_of_mut!(pool).cast(),
                ptr::addr_of_mut!(queue).cast(),
                None,
                test_marker_ptr(0x55),
                Some(count_job_cleanup),
                Some(count_result_cleanup),
                0,
            );

            assert_eq!(ret, -1);
            assert_eq!(queue.curr_serial, 8);
            assert_eq!(queue.n_input, 1);
            assert!(queue.input.is_empty());
            assert_eq!(pool.njobs, 0);
            assert_eq!(JOB_CLEANUPS.load(Ordering::SeqCst), 0);
            assert_eq!(RESULT_CLEANUPS.load(Ordering::SeqCst), 0);

            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }

    #[test]
    fn process_reset_cleans_queued_work_and_resets_serial_state() {
        unsafe {
            let job_cleanups = AtomicUsize::new(0);
            let result_cleanups = AtomicUsize::new(0);
            let job_counter = ptr::addr_of!(job_cleanups).cast::<c_void>().cast_mut();
            let result_counter = ptr::addr_of!(result_cleanups).cast::<c_void>().cast_mut();

            let mut pool = test_pool();
            init_test_mutex(ptr::addr_of_mut!(pool.pool_m));

            let mut queue = test_process(&mut pool, 4, 0);
            init_test_process_conds(&mut queue);
            queue.next_serial = 12;
            queue.curr_serial = 14;
            queue.n_input = 2;
            queue.n_output = 1;
            let p_nn = NonNull::new(ptr::addr_of_mut!(pool)).unwrap();
            let q_nn = NonNull::new(ptr::addr_of_mut!(queue)).unwrap();

            queue.input.push_back(Box::new(HtsTpoolJob {
                func: None,
                arg: NonNull::new(job_counter),
                job_cleanup: Some(count_pointed_atomic),
                result_cleanup: None,
                p: p_nn,
                q: q_nn,
                serial: 12,
            }));
            queue.input.push_back(Box::new(HtsTpoolJob {
                func: None,
                arg: NonNull::new(job_counter),
                job_cleanup: Some(count_pointed_atomic),
                result_cleanup: None,
                p: p_nn,
                q: q_nn,
                serial: 13,
            }));
            queue.output.push_back(Box::new(HtsTpoolResult {
                result_cleanup: Some(count_pointed_atomic),
                serial: 11,
                data: NonNull::new(result_counter),
            }));

            assert_eq!(
                hts_tpool_process_reset(&mut queue, 1),
                0
            );

            assert!(queue.input.is_empty());
            assert!(queue.output.is_empty());
            assert_eq!(queue.n_input, 0);
            assert_eq!(queue.n_output, 0);
            assert_eq!(queue.next_serial, 0);
            assert_eq!(queue.curr_serial, 0);
            assert_eq!(job_cleanups.load(Ordering::SeqCst), 2);
            assert_eq!(result_cleanups.load(Ordering::SeqCst), 1);

            destroy_test_process_conds(&mut queue);
            assert_eq!(
                crate::htslib_rs::c_compat::pthread_mutex_destroy(ptr::addr_of_mut!(pool.pool_m)),
                0
            );
        }
    }
}
