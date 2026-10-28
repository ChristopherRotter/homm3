use super::*;

use alloc::collections::VecDeque;
use alloc::vec;

/// A rate of 100 frames a second: a fade's step is 10 frames, a fade 90.
const RATE: u32 = 100;

/// The tracks the tests know: A 1000 frames, B 500, C 300, `short` 25; `gone` missing, `empty`
/// decoded to nothing, `slow` still being decoded.
fn tracks(name: &str) -> Track {
    match name {
        "A" => Track::Frames(1000),
        "B" => Track::Frames(500),
        "C" => Track::Frames(300),
        "short" => Track::Frames(25),
        "empty" => Track::Frames(0),
        "slow" => Track::Decoding,
        _ => Track::Missing,
    }
}

/// A run as (track, first frame, frames, volume).
type Item<'a> = (&'a str, u64, u64, u8);

/// Items joined where one goes on from the last at the same volume.
fn join<'a>(items: impl IntoIterator<Item = Item<'a>>) -> Vec<Item<'a>> {
    let mut out: Vec<Item<'a>> = Vec::new();
    for item in items {
        match out.last_mut() {
            Some(last) if last.0 == item.0 && last.3 == item.3 && last.1 + last.2 == item.1 => {
                last.2 += item.2;
            }
            _ => out.push(item),
        }
    }
    out
}

/// Runs as items, joined.
fn line(runs: &[Run]) -> Vec<Item<'_>> {
    join(
        runs.iter()
            .map(|r| (r.name.as_str(), r.from, r.frames, r.volume)),
    )
}

/// A fade-in's runs of `name` from frame `from`, one a step of 10 frames.
fn fade_in(name: &str, from: u64) -> Vec<Item<'_>> {
    (0..9)
        .map(|i| {
            (
                name,
                from + 10 * i,
                10,
                FADE_IN[usize::try_from(i).unwrap()],
            )
        })
        .collect()
}

/// A fade-out's runs of `name` from frame `from`, one a step of 10 frames.
fn fade_out(name: &str, from: u64) -> Vec<Item<'_>> {
    (0..9)
        .map(|i| {
            (
                name,
                from + 10 * i,
                10,
                FADE_OUT[usize::try_from(i).unwrap()],
            )
        })
        .collect()
}

/// A model with A asked for and heard for `frames` (its fade-in included).
fn playing_a(frames: u64) -> Music {
    playing_a_with(frames, Bugs::Fixed)
}

/// The same with the original's filter of the requests or the client's (MCL-Q12).
fn playing_a_with(frames: u64, bugs: Bugs) -> Music {
    let mut m = Music::new(RATE, bugs);
    assert_eq!(m.request("A"), Request::Taken);
    let _ = m.advance(frames, &tracks);
    m
}

#[test]
fn a_first_track_fades_in_from_its_start_then_plays_whole() {
    let mut m = Music::new(RATE, Bugs::Fixed);
    assert_eq!(m.playing(), None);
    assert_eq!(m.request("A"), Request::Taken);
    assert_eq!(m.pending(), Some("A"));
    let runs = m.advance(105, &tracks);
    let mut want = fade_in("A", 0);
    want.push(("A", 90, 15, FULL));
    assert_eq!(line(&runs), join(want));
    assert_eq!(m.playing(), Some("A"));
    assert_eq!(m.pending(), None);
    // A step of 100 ms at the stream's rate; at least one frame.
    assert_eq!(Music::new(44_100, Bugs::Fixed).step, 4_410);
    assert_eq!(Music::new(0, Bugs::Fixed).step, 1);
}

#[test]
fn a_track_starts_again_at_its_end() {
    let mut m = Music::new(RATE, Bugs::Fixed);
    m.request("short");
    let runs = m.advance(60, &tracks);
    assert_eq!(
        line(&runs),
        join([
            ("short", 0, 10, 12),
            ("short", 10, 10, 25),
            ("short", 20, 5, 38),
            ("short", 0, 5, 38),
            ("short", 5, 10, 50),
            ("short", 15, 10, 63),
            ("short", 0, 10, 76),
        ])
    );
    // No run passes the track's end.
    assert!(runs.iter().all(|r| r.from + r.frames <= 25));
}

#[test]
fn a_new_track_fades_the_old_one_out_then_fades_in() {
    let mut m = playing_a(200);
    assert_eq!(m.request("B"), Request::Taken);
    let runs = m.advance(200, &tracks);
    let mut want = fade_out("A", 200);
    want.extend(fade_in("B", 0));
    want.push(("B", 90, 20, FULL));
    assert_eq!(line(&runs), join(want));
    assert_eq!(m.playing(), Some("B"));
    // A's place is where its fade began.
    assert_eq!(m.place("A"), Some(200));
}

/// R2-2: a track asked for during a fade-in waits for its end (the thread holds the lock through
/// it), then the stream fades out from the whole, not from the fade-in's volume.
#[test]
fn a_request_during_a_fade_in_waits_for_its_end() {
    let mut m = playing_a(45);
    assert_eq!(m.request("B"), Request::Taken);
    let runs = m.advance(45 + 90 + 20, &tracks);
    let mut want = vec![("A", 45, 5, 63)];
    want.extend(fade_in("A", 0).into_iter().skip(5));
    want.extend(fade_out("A", 90));
    want.extend(fade_in("B", 0).into_iter().take(2));
    assert_eq!(line(&runs), join(want));
    // A's place is the fade-in's end, where its fade-out began.
    assert_eq!(m.place("A"), Some(90));
}

/// R2-5: the old track's place is kept before its fade: it starts again where its fade began.
#[test]
fn a_track_comes_back_where_its_fade_out_began() {
    let mut m = playing_a(300);
    m.request("B");
    let _ = m.advance(90 + 150, &tracks);
    assert_eq!(m.request("A"), Request::Taken);
    let runs = m.advance(90 + 90 + 10, &tracks);
    let mut want = fade_out("B", 150);
    want.extend(fade_in("A", 300));
    want.push(("A", 390, 10, FULL));
    assert_eq!(line(&runs), join(want));
    assert_eq!((m.place("A"), m.place("B")), (Some(300), Some(150)));
}

#[test]
fn the_track_opened_is_the_last_asked_for_during_the_fade_out() {
    let mut m = playing_a(100);
    m.request("B");
    let _ = m.advance(30, &tracks);
    assert_eq!(m.request("C"), Request::Taken);
    assert_eq!(m.pending(), Some("C"));
    let runs = m.advance(70, &tracks);
    let mut want: Vec<_> = fade_out("A", 100).into_iter().skip(3).collect();
    want.extend(fade_in("C", 0).into_iter().take(1));
    assert_eq!(line(&runs), join(want));
    assert_eq!(m.place("B"), None);
}

/// R2-4: `play_music` drops a request for the pending name and for the track opened last while
/// its stream plays, a fading one included (MCL-Q12, the original's).
#[test]
fn a_request_for_the_pending_or_the_playing_track_is_dropped() {
    let mut m = playing_a_with(200, Bugs::Original);
    assert_eq!(m.request("A"), Request::Playing);
    assert_eq!(m.request("B"), Request::Taken);
    assert_eq!(m.request("B"), Request::Pending);
    let _ = m.advance(15, &tracks);
    // A fading out still plays: asked for again, it is dropped and B opens.
    assert_eq!(m.request("A"), Request::Playing);
    let _ = m.advance(75 + 5, &tracks);
    assert_eq!(m.playing(), Some("B"));
    // B fading in plays.
    assert_eq!(m.request("B"), Request::Playing);
    // A, now neither pending nor playing, is taken.
    assert_eq!(m.request("A"), Request::Taken);
}

/// The client's filter (MCL-Q12): the pending name and the track playing with nothing pending
/// are dropped; the track fading out asked for again comes back.
#[test]
fn the_client_drops_only_what_will_play() {
    let mut m = playing_a(200);
    assert_eq!(m.request("A"), Request::Playing);
    assert_eq!(m.request("B"), Request::Taken);
    assert_eq!(m.request("B"), Request::Pending);
    let _ = m.advance(15, &tracks);
    // A fading out, asked for again: it replaces B and opens where its fade began.
    assert_eq!(m.request("A"), Request::Taken);
    assert_eq!(m.pending(), Some("A"));
    assert_eq!(m.request("A"), Request::Pending);
    let runs = m.advance(75 + 20, &tracks);
    let mut want = vec![("A", 215, 5, FADE_OUT[1])];
    want.extend(fade_out("A", 200).into_iter().skip(2));
    want.extend(fade_in("A", 200).into_iter().take(2));
    assert_eq!(line(&runs), join(want));
    assert_eq!(
        (m.playing(), m.pending(), m.place("B")),
        (Some("A"), None, None)
    );
}

/// The two tracks of grass and dirt.
fn grass_and_dirt(name: &str) -> Track {
    match name {
        "Grass" => Track::Frames(1000),
        "Dirt" => Track::Frames(800),
        other => tracks(other),
    }
}

/// Grass, dirt and grass again 150 ms apart (MCL-Q12): the original keeps Dirt.
#[test]
fn grass_dirt_grass_keeps_dirt() {
    let both = grass_and_dirt;
    let mut m2 = Music::new(RATE, Bugs::Original);
    m2.request("Grass");
    let _ = m2.advance(200, &both);
    assert_eq!(m2.request("Dirt"), Request::Taken);
    let _ = m2.advance(15, &both);
    assert_eq!(m2.request("Grass"), Request::Playing);
    let runs = m2.advance(75 + 100, &both);
    let line = line(&runs);
    assert_eq!(line.last().map(|r| (r.0, r.3)), Some(("Dirt", FULL)));
    assert_eq!(m2.playing(), Some("Dirt"));
}

/// The same with the client's filter (MCL-Q12): Grass comes back where its fade began, after it,
/// and Dirt never plays.
#[test]
fn grass_dirt_grass_brings_grass_back() {
    let mut m = Music::new(RATE, Bugs::Fixed);
    m.request("Grass");
    let _ = m.advance(200, &grass_and_dirt);
    assert_eq!(m.request("Dirt"), Request::Taken);
    let _ = m.advance(15, &grass_and_dirt);
    assert_eq!(m.request("Grass"), Request::Taken);
    let runs = m.advance(75 + 100, &grass_and_dirt);
    let mut want = vec![("Grass", 215, 5, FADE_OUT[1])];
    want.extend(fade_out("Grass", 200).into_iter().skip(2));
    want.extend(fade_in("Grass", 200));
    want.push(("Grass", 290, 10, FULL));
    assert_eq!(line(&runs), join(want));
    assert_eq!(m.playing(), Some("Grass"));
    assert!(runs.iter().all(|r| r.name == "Grass"));
}

/// The track fading in or being opened asked for again while another is pending (MCL-Q12): the
/// original drops the request and the pending track follows; the client drops the pending one.
#[test]
fn the_track_fading_in_asked_for_again_while_another_is_pending() {
    // A fading in, B asked for, then A again.
    let mut m = playing_a_with(45, Bugs::Original);
    assert_eq!(m.request("B"), Request::Taken);
    assert_eq!(m.request("A"), Request::Playing);
    let runs = m.advance(45 + 90 + 20, &tracks);
    let mut want = vec![("A", 45, 5, 63)];
    want.extend(fade_in("A", 0).into_iter().skip(5));
    want.extend(fade_out("A", 90));
    want.extend(fade_in("B", 0).into_iter().take(2));
    assert_eq!(line(&runs), join(want));
    let mut m = playing_a_with(45, Bugs::Fixed);
    assert_eq!(m.request("B"), Request::Taken);
    assert_eq!(m.request("A"), Request::Kept);
    assert_eq!(m.pending(), None);
    let runs = m.advance(45 + 90 + 20, &tracks);
    let mut want = vec![("A", 45, 5, 63)];
    want.extend(fade_in("A", 0).into_iter().skip(5));
    want.push(("A", 90, 110, FULL));
    assert_eq!(line(&runs), join(want));
    assert_eq!((m.playing(), m.place("A")), (Some("A"), None));
    // A track being decoded, A asked for, then the first again: the original opens A after it,
    // the client plays it on.
    let ready = |name: &str| match name {
        "slow" => Track::Frames(400),
        other => tracks(other),
    };
    for (bugs, request, last) in [
        (Bugs::Original, Request::Playing, "A"),
        (Bugs::Fixed, Request::Kept, "slow"),
    ] {
        let mut m = Music::new(RATE, bugs);
        m.request("slow");
        assert!(m.advance(10, &tracks).is_empty());
        assert_eq!(m.request("A"), Request::Taken);
        assert_eq!(m.request("slow"), request, "{bugs:?}");
        let _ = m.advance(90 + 90 + 10, &ready);
        assert_eq!(m.playing(), Some(last), "{bugs:?}");
    }
    assert!(Request::Taken.changed() && Request::Kept.changed());
    assert!(!Request::Playing.changed() && !Request::Pending.changed());
}

#[test]
fn a_missing_track_plays_nothing_and_the_next_one_opens() {
    let mut m = Music::new(RATE, Bugs::Fixed);
    m.request("gone");
    assert!(m.advance(50, &tracks).is_empty());
    assert_eq!(m.playing(), None);
    // Its stream does not play: asked for again, it is taken (and missing again).
    assert_eq!(m.request("gone"), Request::Taken);
    assert!(m.advance(50, &tracks).is_empty());
    m.request("empty");
    assert!(m.advance(50, &tracks).is_empty());
    m.request("A");
    assert_eq!(line(&m.advance(10, &tracks)), [("A", 0, 10, 12)]);
    // From a track playing to a missing one: the fade-out, then nothing.
    let mut m = playing_a(100);
    m.request("gone");
    let runs = m.advance(200, &tracks);
    assert_eq!(line(&runs), join(fade_out("A", 100)));
    assert_eq!(m.playing(), None);
}

#[test]
fn a_track_being_decoded_holds_the_clock() {
    let mut m = Music::new(RATE, Bugs::Fixed);
    m.request("slow");
    assert!(m.advance(50, &tracks).is_empty());
    assert_eq!(m.playing(), Some("slow"));
    assert_eq!(m.request("slow"), Request::Playing);
    // Asked for meanwhile, A waits for slow's fade-in as for any.
    assert_eq!(m.request("A"), Request::Taken);
    let ready = |name: &str| match name {
        "slow" => Track::Frames(400),
        other => tracks(other),
    };
    let runs = m.advance(100, &ready);
    let mut want = fade_in("slow", 0);
    want.push(("slow", 90, 10, 114));
    assert_eq!(line(&runs), join(want));
    assert_eq!(m.place("slow"), Some(90));
}

#[test]
fn a_place_past_a_tracks_end_starts_it_from_the_start() {
    let mut m = playing_a(950);
    m.request("B");
    let _ = m.advance(90 + 100, &tracks);
    assert_eq!(m.place("A"), Some(950));
    m.request("A");
    // A shorter now (another file of the name): from its start.
    let shorter = |name: &str| match name {
        "A" => Track::Frames(900),
        other => tracks(other),
    };
    let runs = m.advance(100, &shorter);
    assert_eq!(runs.get(9).map(|r| (r.from, r.volume)), Some((0, 12)));
}

/// The music stream as the device plays it: the runs queued, played from the front.
#[derive(Default)]
struct Device {
    queue: VecDeque<Run>,
    heard: Vec<Run>,
}

impl Device {
    fn queued(&self) -> u64 {
        self.queue.iter().map(|r| r.frames).sum()
    }

    fn put(&mut self, runs: Vec<Run>) {
        self.queue.extend(runs);
    }

    /// `n` frames played (fewer when the queue runs dry).
    fn play(&mut self, mut n: u64) {
        while n > 0 {
            let Some(mut front) = self.queue.pop_front() else {
                return;
            };
            let take = n.min(front.frames);
            self.heard.push(Run {
                frames: take,
                ..front.clone()
            });
            if take < front.frames {
                front.from += take;
                front.frames -= take;
                self.queue.push_front(front);
            }
            n -= take;
        }
    }
}

/// A second of frames queued ahead (100 at [`RATE`]).
const AHEAD: u64 = 100;

/// The window's turn: the stream topped up to [`AHEAD`].
fn service(q: &mut Queue, s: &mut Device, tracks: &impl Fn(&str) -> Track) {
    let runs = q.top_up(s.queued(), AHEAD, tracks);
    s.put(runs);
}

/// The window's cue: the request, the stream cleared when it changed the music to come.
fn cue(q: &mut Queue, s: &mut Device, name: &str) -> Request {
    let request = q.request(name, s.queued(), &tracks);
    if request.changed() {
        s.queue.clear();
    }
    request
}

/// `frames` played in turns of `turn` frames, the stream topped up before each.
fn run(q: &mut Queue, s: &mut Device, frames: u64, turn: u64) {
    for _ in 0..frames / turn {
        service(q, s, &tracks);
        s.play(turn);
    }
}

/// R2-3 and R2-11: a request lands at the frames heard though a second is queued ahead; what is
/// queued is taken back and queued again from there, so the stream plays as the model would.
#[test]
fn the_queue_plays_what_the_model_hears() {
    let (mut q, mut s) = (Queue::new(RATE, Bugs::Fixed), Device::default());
    assert_eq!(cue(&mut q, &mut s, "A"), Request::Taken);
    run(&mut q, &mut s, 200, 20);
    assert_eq!(s.queued(), AHEAD - 20);
    assert_eq!(cue(&mut q, &mut s, "B"), Request::Taken);
    assert_eq!((q.heard_at(), s.queued()), (200, 0));
    // Half of A's fade heard (its tail and B's start queued), C asked for: B never plays.
    run(&mut q, &mut s, 40, 20);
    assert_eq!(cue(&mut q, &mut s, "B"), Request::Pending);
    assert_eq!(cue(&mut q, &mut s, "C"), Request::Taken);
    run(&mut q, &mut s, 50 + 30, 10);
    // C fading in, B asked for: C's fade-in ends, then it fades out from its frame 90.
    assert_eq!(cue(&mut q, &mut s, "B"), Request::Taken);
    run(&mut q, &mut s, 60 + 90 + 20, 17);
    let mut want = fade_in("A", 0);
    want.push(("A", 90, 110, FULL));
    want.extend(fade_out("A", 200));
    want.extend(fade_in("C", 0));
    want.extend(fade_out("C", 90));
    want.extend(fade_in("B", 0).into_iter().take(2));
    assert_eq!(line(&s.heard), join(want));
    // The model alone, asked at the same frames heard, hears the same.
    let mut m = Music::new(RATE, Bugs::Fixed);
    let mut alone = Vec::new();
    for (frames, name) in [(0, "A"), (200, "B"), (40, "C"), (80, "B")] {
        alone.extend(m.advance(frames, &tracks));
        m.request(name);
    }
    alone.extend(m.advance(170, &tracks));
    assert_eq!(line(&s.heard), line(&alone));
    assert_eq!(q.heard().place("A"), Some(200));
    assert_eq!(q.heard().place("C"), Some(90));
}

/// A dropped request takes nothing back; the stream cleared by the window is queued again from
/// the frames heard, as if it had not been.
#[test]
fn a_cleared_stream_plays_on_as_before() {
    let (mut q, mut s) = (Queue::new(RATE, Bugs::Fixed), Device::default());
    cue(&mut q, &mut s, "A");
    run(&mut q, &mut s, 150, 25);
    let before = s.queued();
    assert_eq!(cue(&mut q, &mut s, "A"), Request::Playing);
    assert_eq!(s.queued(), before);
    s.queue.clear();
    q.cleared();
    run(&mut q, &mut s, 100, 25);
    let mut want = fade_in("A", 0);
    want.push(("A", 90, 160, FULL));
    assert_eq!(line(&s.heard), join(want));
}

/// The client's (MCL-Q12): the track fading in asked for again while another is pending takes
/// back what is queued (its fade-out, the other's fade-in) and plays on.
#[test]
fn a_track_kept_plays_on_in_the_queue() {
    let (mut q, mut s) = (Queue::new(RATE, Bugs::Fixed), Device::default());
    cue(&mut q, &mut s, "A");
    run(&mut q, &mut s, 40, 20);
    assert_eq!(cue(&mut q, &mut s, "B"), Request::Taken);
    run(&mut q, &mut s, 20, 20);
    // A's fade-out queued (past its fade-in's frame 90, below the whole).
    assert!(s.queue.iter().any(|r| r.from >= 90 && r.volume < FULL));
    assert_eq!(cue(&mut q, &mut s, "A"), Request::Kept);
    assert_eq!(s.queued(), 0);
    run(&mut q, &mut s, 200, 20);
    let mut want = fade_in("A", 0);
    want.push(("A", 90, 170, FULL));
    assert_eq!(line(&s.heard), join(want));
}

/// A track still being decoded: nothing is queued, the clock holds, and it fades in once decoded.
#[test]
fn the_queue_waits_for_a_track_being_decoded() {
    let (mut q, mut s) = (Queue::new(RATE, Bugs::Fixed), Device::default());
    cue(&mut q, &mut s, "slow");
    run(&mut q, &mut s, 50, 25);
    assert_eq!((s.queued(), q.heard_at()), (0, 0));
    let ready = |name: &str| match name {
        "slow" => Track::Frames(400),
        other => tracks(other),
    };
    service(&mut q, &mut s, &ready);
    s.play(30);
    service(&mut q, &mut s, &ready);
    assert_eq!(q.heard_at(), 30);
    assert_eq!(line(&s.heard), join(fade_in("slow", 0).into_iter().take(3)));
}
