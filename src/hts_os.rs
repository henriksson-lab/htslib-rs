use std::ffi::c_long;
use std::sync::Mutex;

const RAND48_SEED_0: u16 = 0x330e;
const RAND48_SEED_1: u16 = 0xabcd;
const RAND48_SEED_2: u16 = 0x1234;
const RAND48_MULT_0: u16 = 0xe66d;
const RAND48_MULT_1: u16 = 0xdeec;
const RAND48_MULT_2: u16 = 0x0005;
const RAND48_ADD: u16 = 0x000b;

// Holds the full drand48 emulator state: the rolling 48-bit seed, the
// 48-bit multiplier, and the 16-bit additive constant. The C original in
// htslib/hts_os.c keeps these as separate `static` arrays and is NOT
// reentrant -- concurrent callers race on these globals (the v1.23
// `hts_drand48`/`hts_lrand48`/`hts_srand48` have the same data race the C
// API has always had). Glibc protects its real drand48 with an internal
// mutex; the htslib C version does not. We wrap our copy in a single
// `Mutex` so the Rust port is safer than the C original while preserving
// bit-identical LCG output.
#[derive(Clone, Copy)]
struct Rand48State {
    seed: [u16; 3],
    mult: [u16; 3],
    add_state: u16,
}

impl Rand48State {
    const INITIAL: Self = Self {
        seed: [RAND48_SEED_0, RAND48_SEED_1, RAND48_SEED_2],
        mult: [RAND48_MULT_0, RAND48_MULT_1, RAND48_MULT_2],
        add_state: RAND48_ADD,
    };

    // Linear-congruential 48-bit step, computed as three 16-bit limbs
    // exactly like htslib/hts_os.c's `_dorand48`. `xseed` is the caller-owned
    // (or globally-owned) seed buffer being advanced; `self` supplies the
    // multiplier/adder. No locking is performed here -- callers must already
    // hold the `RAND48` mutex.
    fn step(&self, xseed: &mut [u16; 3]) {
        let mut accu: u64 = self.mult[0] as u64 * xseed[0] as u64 + self.add_state as u64;
        let temp0 = accu as u16;
        accu >>= std::mem::size_of::<u16>() * 8;
        accu += self.mult[0] as u64 * xseed[1] as u64 + self.mult[1] as u64 * xseed[0] as u64;
        let temp1 = accu as u16;
        accu >>= std::mem::size_of::<u16>() * 8;
        accu += self.mult[0] as u64 * xseed[2] as u64
            + self.mult[1] as u64 * xseed[1] as u64
            + self.mult[2] as u64 * xseed[0] as u64;
        xseed[0] = temp0;
        xseed[1] = temp1;
        xseed[2] = accu as u16;
    }

    // Helper to compute the [0,1) result of an erand48-style draw from a
    // freshly-advanced seed triple.
    fn erand_value(xseed: &[u16; 3]) -> f64 {
        (xseed[0] as f64) * 2f64.powi(-48)
            + (xseed[1] as f64) * 2f64.powi(-32)
            + (xseed[2] as f64) * 2f64.powi(-16)
    }
}

// Single shared, lock-protected drand48 state. `Mutex::new` is const so the
// initialization is a true `static` (no `OnceLock`/lazy init required).
static RAND48: Mutex<Rand48State> = Mutex::new(Rand48State::INITIAL);

// NOTE on the `unsafe fn` signatures: the four public entry points keep
// their original `unsafe` markings only because we faithfully mirror the C
// API surface (raw `*mut u16` for `hts_erand48`/`_dorand48`). The internal
// global-state access is now synchronized via `RAND48`, so concurrent calls
// from multiple threads are race-free -- a strict improvement over the C
// v1.23 source, whose `hts_drand48`/`hts_lrand48`/`hts_srand48` are
// non-reentrant.

pub unsafe fn _dorand48(xseed: *mut u16) {
    let state = *RAND48.lock().expect("hts_os rand48 mutex poisoned");
    let mut buf = [*xseed.add(0), *xseed.add(1), *xseed.add(2)];
    state.step(&mut buf);
    *xseed.add(0) = buf[0];
    *xseed.add(1) = buf[1];
    *xseed.add(2) = buf[2];
}

pub unsafe fn hts_srand48(seed: c_long) {
    let mut guard = RAND48.lock().expect("hts_os rand48 mutex poisoned");
    guard.seed[0] = RAND48_SEED_0;
    guard.seed[1] = seed as u16;
    guard.seed[2] = (seed >> 16) as u16;
    guard.mult[0] = RAND48_MULT_0;
    guard.mult[1] = RAND48_MULT_1;
    guard.mult[2] = RAND48_MULT_2;
    guard.add_state = RAND48_ADD;
}

pub unsafe fn hts_erand48(xseed: *mut u16) -> f64 {
    // Snapshot the (essentially immutable) multiplier/adder under the lock,
    // then advance the caller's external buffer with no further locking
    // required -- the buffer is owned by the caller, not by us.
    let state = *RAND48.lock().expect("hts_os rand48 mutex poisoned");
    let mut buf = [*xseed.add(0), *xseed.add(1), *xseed.add(2)];
    state.step(&mut buf);
    *xseed.add(0) = buf[0];
    *xseed.add(1) = buf[1];
    *xseed.add(2) = buf[2];
    Rand48State::erand_value(&buf)
}

pub unsafe fn hts_drand48() -> f64 {
    let mut guard = RAND48.lock().expect("hts_os rand48 mutex poisoned");
    let snapshot = *guard;
    let mut buf = guard.seed;
    snapshot.step(&mut buf);
    guard.seed = buf;
    Rand48State::erand_value(&buf)
}

pub unsafe fn hts_lrand48() -> c_long {
    let mut guard = RAND48.lock().expect("hts_os rand48 mutex poisoned");
    let snapshot = *guard;
    let mut buf = guard.seed;
    snapshot.step(&mut buf);
    guard.seed = buf;
    ((buf[2] as c_long) << 15) + ((buf[1] as c_long) >> 1)
}

// original: hts_srand48 (htslib/hts_os.c:35)
pub unsafe fn hts_os_c_35_hts_srand48(seed: c_long) {
    hts_srand48(seed);
}

// original: hts_erand48 (htslib/hts_os.c:45)
pub unsafe fn hts_os_c_45_hts_erand48(xseed: *mut u16) -> f64 {
    hts_erand48(xseed)
}

// original: hts_drand48 (htslib/hts_os.c:48)
pub unsafe fn hts_os_c_48_hts_drand48() -> f64 {
    hts_drand48()
}

// original: hts_lrand48 (htslib/hts_os.c:51)
pub unsafe fn hts_os_c_51_hts_lrand48() -> c_long {
    hts_lrand48()
}

#[cfg(test)]
pub(crate) fn rand48_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_next(seed: [u16; 3]) -> [u16; 3] {
        let state = seed[0] as u64 | ((seed[1] as u64) << 16) | ((seed[2] as u64) << 32);
        let next = state.wrapping_mul(0x5deece66d).wrapping_add(0x0b) & ((1u64 << 48) - 1);
        [next as u16, (next >> 16) as u16, (next >> 32) as u16]
    }

    fn reference_erand(seed: [u16; 3]) -> f64 {
        (seed[0] as f64) * 2f64.powi(-48)
            + (seed[1] as f64) * 2f64.powi(-32)
            + (seed[2] as f64) * 2f64.powi(-16)
    }

    #[test]
    fn rand48_sequence_matches_known_freebsd_algorithm_values() {
        unsafe {
            let _guard = rand48_test_lock();
            // Reset internal state so this test does not depend on the
            // multiplier/adder being at their default values.
            hts_srand48(0);
            let mut seed = [RAND48_SEED_0, RAND48_SEED_1, RAND48_SEED_2];
            _dorand48(seed.as_mut_ptr());
            assert_eq!(seed, [0x5101, 0xb725, 0x657e]);

            let mut custom = [0x330e, 0x0000, 0x0000];
            let x = hts_erand48(custom.as_mut_ptr());
            assert_eq!(custom, [0x5101, 0x62dc, 0x2bbb]);
            assert_eq!(x.to_bits(), reference_erand(custom).to_bits());

            hts_srand48(1);
            assert_eq!(hts_lrand48(), 89400484);
            hts_srand48(1);
            assert_eq!(
                hts_drand48().to_bits(),
                reference_erand(reference_next([RAND48_SEED_0, 0x0001, 0x0000])).to_bits()
            );
        }
    }

    #[test]
    fn srand48_initializes_seed_words_and_multiplier_state() {
        unsafe {
            let _guard = rand48_test_lock();
            hts_srand48(0x1234_5678);

            // Inspect the locked state directly to verify hts_srand48
            // reinitialized every field exactly like the C original.
            let snapshot = *RAND48.lock().expect("hts_os rand48 mutex poisoned");
            assert_eq!(snapshot.seed[0], RAND48_SEED_0);
            assert_eq!(snapshot.seed[1], 0x5678);
            assert_eq!(snapshot.seed[2], 0x1234);
            assert_eq!(snapshot.mult[0], RAND48_MULT_0);
            assert_eq!(snapshot.mult[1], RAND48_MULT_1);
            assert_eq!(snapshot.mult[2], RAND48_MULT_2);
            assert_eq!(snapshot.add_state, RAND48_ADD);
        }
    }

    #[test]
    fn erand48_updates_all_seed_words_at_upper_boundary() {
        unsafe {
            let _guard = rand48_test_lock();
            // hts_erand48 takes a caller-owned buffer but uses the global
            // multiplier/adder -- reset them so the test result is stable
            // regardless of preceding tests.
            hts_srand48(0);
            let mut seed = [0xffff, 0xffff, 0xffff];
            let value = hts_os_c_45_hts_erand48(seed.as_mut_ptr());

            assert_eq!(seed, [0x199e, 0x2113, 0xfffa]);
            assert_eq!(value.to_bits(), reference_erand(seed).to_bits());
        }
    }

    #[test]
    fn rand48_entrypoint_aliases_share_deterministic_state_progression() {
        unsafe {
            let _guard = rand48_test_lock();
            hts_os_c_35_hts_srand48(0x1234_5678);
            let first_seed = reference_next([RAND48_SEED_0, 0x5678, 0x1234]);
            let first_value = reference_erand(first_seed);
            assert_eq!(hts_os_c_51_hts_lrand48(), 0x5c2a01fa);

            hts_os_c_35_hts_srand48(0x1234_5678);
            assert_eq!(hts_os_c_48_hts_drand48().to_bits(), first_value.to_bits());

            let second_seed = reference_next(first_seed);
            let second_value = reference_erand(second_seed);
            assert_eq!(hts_os_c_48_hts_drand48().to_bits(), second_value.to_bits());
        }
    }

    #[test]
    fn erand48_uses_caller_seed_without_advancing_global_state() {
        unsafe {
            let _guard = rand48_test_lock();
            hts_srand48(0x0000_0001);
            let expected_global_seed = reference_next([RAND48_SEED_0, 0x0001, 0x0000]);
            let expected_global_value = reference_erand(expected_global_seed);

            let mut explicit_seed = [0x330e, 0x5678, 0x1234];
            let expected_explicit_seed = reference_next(explicit_seed);
            let explicit_value = hts_erand48(explicit_seed.as_mut_ptr());
            assert_eq!(explicit_seed, expected_explicit_seed);
            assert_eq!(
                explicit_value.to_bits(),
                reference_erand(expected_explicit_seed).to_bits()
            );

            assert_eq!(hts_drand48().to_bits(), expected_global_value.to_bits());
        }
    }

    #[test]
    fn srand48_reinitializes_after_alias_state_advancement() {
        unsafe {
            let _guard = rand48_test_lock();
            let first_seed = reference_next([RAND48_SEED_0, 0xffff, 0xffff]);
            let first_lrand = ((first_seed[2] as c_long) << 15) + ((first_seed[1] as c_long) >> 1);

            hts_os_c_35_hts_srand48(-1);
            assert_eq!(hts_os_c_51_hts_lrand48(), first_lrand);
            assert_ne!(hts_os_c_51_hts_lrand48(), first_lrand);

            hts_srand48(-1);
            // Inspect the locked state to confirm the seed words were reset
            // exactly to the documented values.
            let snapshot = *RAND48.lock().expect("hts_os rand48 mutex poisoned");
            assert_eq!(snapshot.seed[0], RAND48_SEED_0);
            assert_eq!(snapshot.seed[1], 0xffff);
            assert_eq!(snapshot.seed[2], 0xffff);
            assert_eq!(hts_lrand48(), first_lrand);
        }
    }

    // -------- Concurrency tests for the new Mutex-protected state. --------
    //
    // These tests intentionally stress the global `RAND48` state from
    // multiple threads. None of them rely on a specific *ordering* of
    // concurrent draws (which would be inherently non-deterministic); they
    // verify race-freedom, that the LCG never returns a non-finite value,
    // and that on a single thread the algorithm remains bit-reproducible.

    use std::collections::HashSet;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Barrier,
    };
    use std::thread;

    /// Compute the exact set (in order) of N states the LCG would visit
    /// starting from `start_seed`, advancing once per element. Used to
    /// bound the set of bit-patterns the global generator can possibly
    /// produce across `n` concurrent calls.
    fn forward_states(mut seed: [u16; 3], n: usize) -> Vec<[u16; 3]> {
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            seed = reference_next(seed);
            v.push(seed);
        }
        v
    }

    #[test]
    fn rand48_concurrent_drand48_does_not_panic_or_diverge_from_serial() {
        // Many threads call hts_drand48 simultaneously after a known seed.
        // Even though the interleaving is non-deterministic, every value
        // returned must correspond to *some* state along the deterministic
        // LCG trajectory of length `THREADS * PER_THREAD`. We verify:
        //   - no thread panics,
        //   - every returned f64 lies in [0, 1) and is finite,
        //   - every returned f64's bit-pattern is one of the
        //     `THREADS * PER_THREAD` legal states.
        const THREADS: usize = 8;
        const PER_THREAD: usize = 10_000;

        unsafe {
            let _guard = rand48_test_lock();
            hts_srand48(0xC0FFEE);
            let start_seed = {
                let s = *RAND48.lock().expect("hts_os rand48 mutex poisoned");
                s.seed
            };

            // Build the universe of legal post-step states.
            let trajectory = forward_states(start_seed, THREADS * PER_THREAD);
            let legal_bits: HashSet<u64> = trajectory
                .iter()
                .map(|s| reference_erand(*s).to_bits())
                .collect();

            let barrier = Arc::new(Barrier::new(THREADS));
            let handles: Vec<_> = (0..THREADS)
                .map(|_| {
                    let barrier = barrier.clone();
                    thread::spawn(move || {
                        let mut out = Vec::with_capacity(PER_THREAD);
                        barrier.wait();
                        for _ in 0..PER_THREAD {
                            let v = hts_drand48();
                            out.push(v);
                        }
                        out
                    })
                })
                .collect();

            let mut all_values = Vec::with_capacity(THREADS * PER_THREAD);
            for h in handles {
                let chunk = h.join().expect("worker thread panicked");
                all_values.extend(chunk);
            }

            assert_eq!(all_values.len(), THREADS * PER_THREAD);
            for v in &all_values {
                assert!(v.is_finite(), "hts_drand48 returned a non-finite value");
                assert!(
                    (0.0..1.0).contains(v),
                    "hts_drand48 returned out-of-range value {v}"
                );
                assert!(
                    legal_bits.contains(&v.to_bits()),
                    "hts_drand48 returned an off-trajectory bit pattern {:#x}",
                    v.to_bits()
                );
            }

            // After exactly `THREADS * PER_THREAD` draws the global state
            // must equal the final reference trajectory state (because the
            // mutex serializes every draw, the total step count is exact).
            let final_state = *RAND48.lock().expect("hts_os rand48 mutex poisoned");
            assert_eq!(final_state.seed, *trajectory.last().unwrap());
        }
    }

    #[test]
    fn rand48_srand_reseed_under_contention() {
        // One thread continuously reseeds while three other threads draw.
        // The invariants we check are minimal: no panic, no UB, every draw
        // returns a value in [0, 1). The point is to exercise the writer
        // path (hts_srand48 mutates seed/mult/add_state) interleaved with
        // many readers (hts_drand48) and ensure the Mutex serializes them.
        const READERS: usize = 3;
        const ITERS: usize = 5_000;

        unsafe {
            let _guard = rand48_test_lock();
            hts_srand48(42);

            let stop = Arc::new(AtomicBool::new(false));
            let stop_writer = stop.clone();
            let writer = thread::spawn(move || {
                let mut n: c_long = 0;
                while !stop_writer.load(Ordering::Relaxed) {
                    hts_srand48(n.wrapping_mul(0x9E37_79B9) ^ 42);
                    n = n.wrapping_add(1);
                }
                n
            });

            let readers: Vec<_> = (0..READERS)
                .map(|_| {
                    thread::spawn(|| {
                        for _ in 0..ITERS {
                            let v = hts_drand48();
                            assert!(v.is_finite());
                            assert!((0.0..1.0).contains(&v));
                        }
                    })
                })
                .collect();

            for r in readers {
                r.join().expect("reader thread panicked");
            }
            stop.store(true, Ordering::Relaxed);
            let writes = writer.join().expect("writer thread panicked");
            assert!(writes > 0, "writer made no progress");

            // After the storm, hts_srand48(42) should still produce the
            // documented deterministic sequence -- proving the state is not
            // corrupted by the contention.
            hts_srand48(42);
            let expected = reference_next([RAND48_SEED_0, 42, 0]);
            assert_eq!(hts_drand48().to_bits(), reference_erand(expected).to_bits());
        }
    }

    #[test]
    fn rand48_serial_reproducibility() {
        // Single-threaded reproducibility under the same seed -- the LCG
        // sequence must be bit-identical across two independent reseeds.
        const K: usize = 1024;

        unsafe {
            let _guard = rand48_test_lock();

            hts_srand48(0xDEADBEEF_u32 as c_long);
            let mut first = Vec::with_capacity(K);
            for _ in 0..K {
                first.push(hts_drand48().to_bits());
            }
            let first_lrand = hts_lrand48();

            hts_srand48(0xDEADBEEF_u32 as c_long);
            let mut second = Vec::with_capacity(K);
            for _ in 0..K {
                second.push(hts_drand48().to_bits());
            }
            let second_lrand = hts_lrand48();

            assert_eq!(first, second, "drand48 not bit-reproducible after reseed");
            assert_eq!(
                first_lrand, second_lrand,
                "lrand48 not bit-reproducible after reseed"
            );
        }
    }
}
