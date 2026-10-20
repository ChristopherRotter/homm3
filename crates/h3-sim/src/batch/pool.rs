//! The batch's threads (ADR-0025 point 3; `docs/epics/E05-log.md` §1): T std threads, parked
//! between calls, each running one contiguous shard of the envs per call; with T = 0 the caller's
//! thread runs the only shard. Env i is in shard `i * T / N`; T is at most N. What an env does
//! never depends on the shard it is in: a shard's part of a call touches only its own envs and
//! rows.

use std::any::Any;
use std::fmt;
use std::ops::Range;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};

/// A call's work: run by every worker with its index.
type Job<'a> = dyn Fn(usize) + Sync + 'a;

/// The workers of a batch (none for T = 0).
pub(crate) struct Pool {
    shared: Arc<Shared>,
    workers: Vec<JoinHandle<()>>,
}

impl fmt::Debug for Pool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pool")
            .field("threads", &self.workers.len())
            .finish()
    }
}

/// What the caller and the workers share.
struct Shared {
    state: Mutex<State>,
    /// Wakes the workers: a new job, or the pool's end.
    start: Condvar,
    /// Wakes the caller: the last worker finished the job.
    finished: Condvar,
}

#[derive(Default)]
struct State {
    /// The current job, its lifetime erased ([`Pool::run`] says why that is sound).
    job: Option<&'static Job<'static>>,
    /// The jobs started so far: a worker runs each one once.
    generation: u64,
    /// The workers still running the current job.
    running: usize,
    /// The first panic of a worker in the current job, resumed in the caller.
    panic: Option<Box<dyn Any + Send>>,
    /// The pool is dropped: the workers end.
    quit: bool,
}

/// Locks `mutex`; a poisoned lock is taken as it is (no code panics while holding one).
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Waits on `condvar` with `guard`.
fn wait<'a, T>(condvar: &Condvar, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
    condvar.wait(guard).unwrap_or_else(PoisonError::into_inner)
}

impl Pool {
    /// A pool of `threads` workers.
    ///
    /// # Errors
    /// The system's refusal to start a thread (the workers already started end).
    pub(crate) fn new(threads: usize) -> std::io::Result<Self> {
        let mut pool = Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State::default()),
                start: Condvar::new(),
                finished: Condvar::new(),
            }),
            workers: Vec::with_capacity(threads),
        };
        for index in 0..threads {
            let shared = pool.shared.clone();
            let worker = thread::Builder::new()
                .name(format!("h3-batch-{index}"))
                .spawn(move || work(&shared, index))?;
            pool.workers.push(worker);
        }
        Ok(pool)
    }

    /// The number of shards: one per worker, or one in the caller's thread.
    pub(crate) fn shards(&self) -> usize {
        self.workers.len().max(1)
    }

    /// Runs `work` on each of `parts` (one per shard, [`shards`](Self::shards) of them), part k in
    /// worker k, and returns when every part is done. A worker's panic is resumed here once every
    /// worker has finished. What a panic leaves in the parts depends on the thread count (on the
    /// caller's thread the one part stops where it panicked; workers finish the other parts), and
    /// of several panics the one resumed on the scheduling: the batch is then broken (`h3-ffi`:
    /// destroy it).
    pub(crate) fn run<T: Send>(&mut self, parts: Vec<T>, work: impl Fn(T) + Sync) {
        if self.workers.is_empty() {
            parts.into_iter().for_each(work);
            return;
        }
        let parts: Vec<Mutex<Option<T>>> = parts.into_iter().map(|p| Mutex::new(Some(p))).collect();
        let job = |worker: usize| {
            let part = parts.get(worker).and_then(|p| lock(p).take());
            if let Some(part) = part {
                work(part);
            }
        };
        let job: &Job<'_> = &job;
        // SAFETY: `job` borrows `parts` and `work`, which outlive this call. The workers use the
        // erased reference only while running the current job, and this function returns (or
        // resumes a worker's panic) only after every worker has finished it (`running == 0`,
        // counted down after `catch_unwind`, so a panicking job counts too) and the reference was
        // cleared. Nothing between storing it and that wait can unwind. `&mut self` keeps a
        // second job from starting meanwhile.
        let job = unsafe { std::mem::transmute::<&Job<'_>, &'static Job<'static>>(job) };
        let mut state = lock(&self.shared.state);
        state.job = Some(job);
        state.generation = state.generation.wrapping_add(1);
        state.running = self.workers.len();
        self.shared.start.notify_all();
        while state.running > 0 {
            state = wait(&self.shared.finished, state);
        }
        state.job = None;
        let panic = state.panic.take();
        drop(state);
        if let Some(payload) = panic {
            panic::resume_unwind(payload);
        }
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        lock(&self.shared.state).quit = true;
        self.shared.start.notify_all();
        for worker in self.workers.drain(..) {
            // A worker never panics outside a job (a job's panic is caught and resumed in `run`).
            let _ = worker.join();
        }
    }
}

/// A worker's loop: park until a job starts, run it with `index`, report it done.
fn work(shared: &Shared, index: usize) {
    let mut seen = 0;
    loop {
        let job = {
            let mut state = lock(&shared.state);
            while !state.quit && state.generation == seen {
                state = wait(&shared.start, state);
            }
            if state.quit {
                return;
            }
            seen = state.generation;
            state.job
        };
        let outcome = job.map(|job| panic::catch_unwind(AssertUnwindSafe(|| job(index))));
        let mut state = lock(&shared.state);
        if let Some(Err(payload)) = outcome {
            state.panic.get_or_insert(payload);
        }
        state.running -= 1;
        if state.running == 0 {
            shared.finished.notify_one();
        }
    }
}

/// The envs of each of `shards` shards of `n` envs, in order: env i is in shard `i * shards / n`.
pub(crate) fn shard_ranges(n: usize, shards: usize) -> Vec<Range<usize>> {
    let start = |shard: usize| (shard * n).div_ceil(shards);
    (0..shards).map(|s| start(s)..start(s + 1)).collect()
}

/// Splits `rows` (`width` values per env) into the parts of `ranges` (contiguous, from 0; `rows`
/// holds at least all of them).
pub(crate) fn split<'a, T>(
    mut rows: &'a mut [T],
    width: usize,
    ranges: &[Range<usize>],
) -> Vec<&'a mut [T]> {
    ranges
        .iter()
        .map(|r| {
            let (part, rest) = std::mem::take(&mut rows).split_at_mut(r.len() * width);
            rows = rest;
            part
        })
        .collect()
}

/// [`split`] for an optional array: `None` in every part when it is absent.
pub(crate) fn split_optional<'a, T>(
    rows: Option<&'a mut [T]>,
    width: usize,
    ranges: &[Range<usize>],
) -> Vec<Option<&'a mut [T]>> {
    match rows {
        Some(rows) => split(rows, width, ranges).into_iter().map(Some).collect(),
        None => ranges.iter().map(|_| None).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_i_is_in_shard_i_t_over_n() {
        for n in 1..40 {
            for t in 1..=n {
                let ranges = shard_ranges(n, t);
                assert_eq!(ranges.len(), t);
                for (s, r) in ranges.iter().enumerate() {
                    assert!(!r.is_empty(), "n {n}, t {t}");
                    for i in r.clone() {
                        assert_eq!(i * t / n, s, "n {n}, t {t}, env {i}");
                    }
                }
                assert_eq!(ranges.last().map(|r| r.end), Some(n));
            }
        }
    }

    #[test]
    fn parts_run_in_their_workers_and_write_in_place() {
        for threads in 0..5 {
            let mut pool = Pool::new(threads).unwrap();
            assert_eq!(pool.shards(), threads.max(1));
            let n = 23;
            let ranges = shard_ranges(n, pool.shards());
            let mut out = vec![0u64; n * 2];
            for call in 1..4u64 {
                let parts: Vec<_> = split(&mut out, 2, &ranges)
                    .into_iter()
                    .zip(ranges.iter().cloned())
                    .collect();
                pool.run(parts, |(rows, envs): (&mut [u64], Range<usize>)| {
                    for (row, env) in rows.chunks_exact_mut(2).zip(envs) {
                        row[0] = call * 100 + env as u64;
                        row[1] += 1;
                    }
                });
                for (env, row) in out.chunks_exact(2).enumerate() {
                    assert_eq!(row, [call * 100 + env as u64, call], "{threads} threads");
                }
            }
            assert_eq!(
                format!("{pool:?}"),
                format!("Pool {{ threads: {threads} }}")
            );
        }
    }

    #[test]
    fn a_workers_panic_reaches_the_caller_after_every_worker_finished() {
        let mut pool = Pool::new(3).unwrap();
        let ranges = shard_ranges(6, 3);
        let mut out = [0u8; 6];
        let caught = panic::catch_unwind(AssertUnwindSafe(|| {
            pool.run(split(&mut out, 1, &ranges), |rows: &mut [u8]| {
                rows.fill(1);
                panic!("shard");
            });
        }));
        let payload = caught.unwrap_err();
        assert_eq!(payload.downcast_ref::<&str>(), Some(&"shard"));
        // Every part ran to its panic; the pool still works.
        assert_eq!(out, [1; 6]);
        pool.run(split(&mut out, 1, &ranges), |rows: &mut [u8]| rows.fill(2));
        assert_eq!(out, [2; 6]);
    }

    #[test]
    fn optional_rows_split_into_parts_or_nothing() {
        let ranges = shard_ranges(5, 2);
        let mut rows = [0u8; 15];
        let parts = split_optional(Some(&mut rows[..]), 3, &ranges);
        assert_eq!(
            parts
                .iter()
                .map(|p| p.as_ref().map(|r| r.len()))
                .collect::<Vec<_>>(),
            [Some(9), Some(6)]
        );
        assert!(
            split_optional::<u8>(None, 3, &ranges)
                .iter()
                .all(Option::is_none)
        );
    }
}
