//! `h3-server-journals [--map <file>] [--packs-dir packs] [--days N] [--per-turn N] [--marks a,b,…] [--seed N]
//! [--agent-seed N] [--checkpoint-every N] [--data <dir>]`: the long journals' measurement
//! (`h3_server::measure`, E16 M5 T23): a random game on a map, its journal as a game of
//! connections' at each mark's day, its reading, its re-run as a restart hosts it and its replay's
//! verification timed, the heap counted by this binary's allocator. One line a mark, tab-separated.
// An adapter's measurement: the wall time it prints.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};

use h3_server::measure::{self, Meter, Options};

const USAGE: &str = "usage: h3-server-journals [--map <file>] [--packs-dir packs] [--days N] \
                     [--per-turn N] [--marks a,b,…] [--seed N] [--agent-seed N] [--checkpoint-every N] \
                     [--data <dir>]";

/// The heap counted: the system's allocator, each allocation's size added and each release's
/// taken away, the most at once kept.
struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

impl Counting {
    fn added(size: usize) {
        let live = LIVE.fetch_add(size, Ordering::Relaxed) + size;
        PEAK.fetch_max(live, Ordering::Relaxed);
    }
}

// SAFETY: every call is the system allocator's with the caller's own arguments; the counters
// only add and take away the sizes it was asked for.
#[allow(unsafe_code)]
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's contract is `System`'s.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            Self::added(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: as `alloc`.
        unsafe { System.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: as `alloc`.
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            Self::added(layout.size());
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: as `alloc`.
        let new = unsafe { System.realloc(ptr, layout, new_size) };
        if !new.is_null() {
            LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
            Self::added(new_size);
        }
        new
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// The binary's allocator as the measurement's meter.
struct Counted;

impl Meter for Counted {
    fn live(&self) -> usize {
        LIVE.load(Ordering::Relaxed)
    }

    fn peak(&self) -> usize {
        PEAK.load(Ordering::Relaxed)
    }

    fn reset_peak(&self) {
        PEAK.store(LIVE.load(Ordering::Relaxed), Ordering::Relaxed);
    }
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / f64::from(1u32 << 20)
}

fn run(args: &[String]) -> Result<(), String> {
    let mut options = Options {
        map: PathBuf::from("fixtures/maps/e08-economy-144.h3m"),
        packs: PathBuf::from("packs"),
        seed: 1,
        agent_seed: 1,
        days: 112,
        per_turn: 0,
        marks: vec![7, 14, 28, 56, 84, 112],
        checkpoint_every: h3_server::config::Server::default().whole_checkpoint_every,
        data: std::env::temp_dir().join("h3-server-journals"),
    };
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let value = iter
            .next()
            .ok_or_else(|| format!("{flag} needs a value\n{USAGE}"))?;
        let number = |value: &str| value.parse().map_err(|e| format!("{flag} {value}: {e}"));
        match flag.as_str() {
            "--map" => options.map = PathBuf::from(value),
            "--packs-dir" => options.packs = PathBuf::from(value),
            "--days" => options.days = number(value)?,
            "--per-turn" => options.per_turn = number(value)?,
            "--marks" => {
                options.marks = (value.split(',').map(&number)).collect::<Result<_, _>>()?;
            }
            "--seed" => options.seed = u64::from(number(value)?),
            "--agent-seed" => options.agent_seed = u64::from(number(value)?),
            "--checkpoint-every" => options.checkpoint_every = number(value)?,
            "--data" => options.data = PathBuf::from(value),
            _ => return Err(format!("unknown option {flag}\n{USAGE}")),
        }
    }
    let (played, marks) = measure::run(&options, &Counted).map_err(|e| e.0)?;
    println!(
        "map {}: {} players, {} days, {} commands ({} the system's, {} battle commands); \
         played in {:.1}s; the build {:.3}s, {:.1} MiB",
        options.map.display(),
        played.players,
        played.day - 1,
        played.commands,
        played.system,
        played.battle,
        played.play_secs,
        played.build_secs,
        mib(played.built_bytes),
    );
    println!(
        "day\tcommands\tentries\tjournal MiB\tapply s\tstate MiB\tread s\tread peak MiB\t\
         read MiB\treopen s\treopen peak MiB\tsession MiB\tcopy MiB\tverify s"
    );
    for mark in marks {
        println!(
            "{}\t{}\t{}\t{:.2}\t{:.3}\t{:.1}\t{:.3}\t{:.1}\t{:.1}\t{:.3}\t{:.1}\t{:.1}\t{:.2}\t{:.3}",
            mark.day,
            mark.commands,
            mark.entries,
            mark.bytes as f64 / f64::from(1u32 << 20),
            mark.apply_secs,
            mib(mark.state_bytes),
            mark.read_secs,
            mib(mark.read_peak),
            mib(mark.read_bytes),
            mark.reopen_secs,
            mib(mark.reopen_peak),
            mib(mark.session_bytes),
            mib(mark.copy_bytes),
            mark.verify_secs,
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("h3-server-journals: {e}");
            ExitCode::FAILURE
        }
    }
}
