/* xdd.dll for the reference captures (clients/capture/README.md). Built by setup.sh with
   mingw-w64; our own code. The original's files never change; in its memory only the import
   slots of 2 and 5 and the roll calls of 3.

   1. A forwarder (E17 M1 T8): the GOG Heroes3.exe imports DirectDrawCreate from xdd.dll, its
      DDrawCompat wrapper, which does not run under Wine; this xdd.dll hands the call to Wine's own
      ddraw (xdd.def).
   2. A virtual clock (E17 M2 T1, *provisional*): the executable reads time only through WINMM's
      timeGetTime (its import table: no GetTickCount, QueryPerformanceCounter or timers), so
      redirecting that one import slot of Heroes3.exe controls every animation of the game. With
      H3CLOCK set to a directory (a Windows path, Z:\... under Wine), the clock runs in real time
      until the harness writes a command to <H3CLOCK>\ctl, one line "<seq> <command>":
        <seq> freeze      the clock stops at its current time
        <seq> step <ms>   the stopped clock moves on by <ms> milliseconds
        <seq> run         the clock runs on in real time from where it stands
      A new <seq> applies its command once. Every few milliseconds the DLL writes
      <H3CLOCK>\ack as "<seq> <time> <calls>": the last applied command, the clock's time and how
      many times the game read the clock since that command (a game that keeps reading a stopped
      clock is idle: it has drawn everything up to that time). Without H3CLOCK nothing changes.
      With H3FROZEN set as well, to a number of milliseconds (E27 M1 T2: the shell's recipes),
      the clock stands at that time from the game's first read, before any command, so every
      time of a run (a clip's frames, a screen's opening) is the same in each run.
   3. Forced rolls (E17 M3 T7b-2b-1, *provisional*; the runbook, "Forced rolls"): with H3CLOCK
      set, the calls of the battle's rolls in `sites` (morale, fear, luck, the creatures'
      strike abilities; the wall shots', E17 M4 T2b) go through
      forced_between or forced_rand, which draw as the call would (random_between 0x50B230 or the
      C library's rand 0x617E5C, so the generator's sequence stays the original's) and return
      that draw unless the harness forced the site:
        <seq> roll <site> <value> <count>   the site's next <count> draws return <value>
        <seq> rolls <site> <value>          its other draws return <value> ...
        <seq> rolls <site> natural          ... or the generator's again (as at the start)
      A call is redirected only where its bytes are the call the table names. The ack goes on
      with "<forced> <pending> <sites> <refused>": the draws forced so far, the forced draws
      still waiting for their roll, the calls redirected and (E17 M4 T3a) what was refused: a
      roll command it could not read (a queue past QUEUE values included), a queue that replaced
      values of the site's last one never drawn, and a forced value outside the range its call
      draws from (random_between(a, b): the call's own draw is returned instead).
   4. Logged draws (E17 M3 T7c-3, *provisional*): a site marked `logged` (Chain Lightning's
      rays turning in `ray_aim`) appends each of its draws, as returned, to <H3CLOCK>\draws, one
      line "<site> <value>", so the frame check can draw the same rays; the file starts empty
      with the game. Nothing is forced there unless the harness asks, as at any site. A site
      logged with its range (E09 T34: `logged` 2, the taverns' draws) writes "<site> <value>
      <low> <high>", the bounds of its random_between(low, high).
   5. The clips' clock (E17 M5 T4b2b, *provisional*): BINKW32.DLL reads WINMM's timeGetTime
      itself to pace a clip (`BinkWait`), so its import slot is pointed at the same clock
      (bink_time, which does not count as the game's reads); the ack's eighth field is 1 when
      it is. Without it a clip ran in real time while the game's clock stood still. SMACKW32.DLL
      reads it as well (E27 M1 T2: the start's 3DO logo, the score screen's and the lost
      screen's clips are Smacker ones), so its slot points at bink_time too; the ack's eleventh
      field is 1 when it does. Without H3SOUND its timeSetEvent timers stay in real time (none
      paces a frame the game draws: they serve Smacker's sound, which then never plays; with
      H3SOUND section 7. plays it and points timeSetEvent at a timer never fired). The clips'
      reads of that clock are counted apart, the ack's twelfth field (since the last command): in a
      full-screen Bink clip the game's loop leaves the waiting to BinkWait and reads no clock
      itself, so the harness takes these for the game being idle there.
      BinkGetSummary (E27 M1 T2): the game asks a full-screen clip's summary at its end (did it
      run through), and the DLL divides the bytes it read x 1000 by its read time + 1 (the
      handle's +0x10C and +0x110, 0x100061C4): on a frozen clock that time is 0 ms, and a clip of
      4 MB or more overflows the division (Wine: "division by zero", the game stops). Heroes3.exe's
      import of it goes through summary_hook, which raises the read time to the least that fits
      for the call and puts it back after; the summary's read speed is the only field it
      changes. The ack's thirteenth field is 1 when the import is hooked.
   6. Sounds on the virtual clock (E17 M5 T5b2, *provisional*): with H3SOUND set as well (the
      harness's `sound` runs, which give Wine an audio device and the game an effects volume),
      Heroes3.exe's Miles imports that start, end and ask after a sample go through the hooks
      below, so a sample plays for its length on the virtual clock: AIL_sample_status reports it
      playing (4) from AIL_start_sample until (now - start) x rate reaches its frames x 1000 x its
      loop count (0: until ended), then done (2); AIL_end_sample and AIL_init_sample end it;
      AIL_stop_sample pauses a playing one, stopped (8) as Miles answers, until AIL_resume_sample
      moves its start on by the pause (E17 M5 T8b1, R7-6). A
      sample's frames are its WAV's `fact` count (every entry of the sound archives has one: the
      viewer's decoder gives the same, h3-assets' wav.rs), its rate the `fmt ` chunk's. Miles
      still mixes the real sample (Wine's device is ALSA's null sink, faster than real time), but
      the game reads only the hooks' answers, so `wait_for_sample` 0x59A4C0 and `sample_wait`
      0x599EC0 end where the viewer's timeline does. Each start is appended to <H3CLOCK>\sounds as
      "<time> <channel> <length ms> <loops>" (the channel: the game's sample handle's index in
      the order the hooks first saw it, in any hooked call (its file set, as a rule: E17 M5 T8b1,
      R7-12); the length rounded up); the ack's ninth field counts the hooked
      imports (9 when all are: AIL_serve's too, whose real time resets the game's reads like
      every hooked call, so the harness never takes a game inside Miles for idle), its tenth
      (E17 M5 T8b1, R7-5) the calls the hooks could not follow: a file not read as a RIFF WAVE
      with a `fmt ` rate and a `fact` count, and a handle past the table's SAMPLES (real Miles
      answers for it); the harness fails a sound run when it is not 0. With sound the
      game's other threads read the clock too (its sound service), so only the main thread's
      reads count: they alone say the battle is idle (`machines`' heal read half done
      otherwise).
   7. The clips' sound on the virtual clock (E27 M2 T5b2, *provisional*; E27-design §12): with
      H3SOUND set, a clip's track plays on the virtual clock too, its calls logged to
      <H3CLOCK>\clipsound, one line each "<time> <call> <a> <b>". SMACKW32.DLL times a Smacker
      clip's frames by the play cursor of its DirectSound buffer (the DLL's own, made on the
      DirectSound object Miles opened: §11.1), which Wine moves in real time: Heroes3.exe's
      import of SmackSoundUseMSS goes through
      use_mss_hook, which gives the DLL's SmackSoundUseDirectSound vsound.h's stand-in (its
      cursor on the virtual clock, an ideal device) instead of Miles' object (the game's Miles
      answers preference 15 with 0 and goes there itself), so the DLL never loads Miles'
      functions and makes its sound timer with timeSetEvent, whose import (and timeKillEvent's)
      points at smack_set_event, made and never fired: the DLL's sound service runs from the
      game's calls alone, as in `smackprobe wait`. The log gets vsound.h's lines but lock and
      unlock (create, flags, play, stop, setpos, volume, pan, release) and "use", "timer",
      "timer-killed". BINKW32.DLL streams a Bink clip's sound through Miles' double-buffered
      sample, the functions BinkOpenMiles takes by name (GetProcAddress, 0x1000F970): its
      GetProcAddress import goes through proc_hook, which answers the sample's calls with the
      hooks below: the buffers play on the virtual clock at the sample's bytes a second, one
      after the other from the first load (bink_* below); the real Miles makes, sets and
      releases the handle and gives the least buffer size but never gets a buffer (it plays
      nothing). The log gets "bink-minimum <size> <rate>", "bink-allocate", "bink-volume <v>",
      "bink-load <buffer> <bytes>", "bink-play" (the first load: the sound starts), "bink-dry"
      (a buffer played out with none after it), "bink-stop", "bink-resume" (BinkPause) and
      "bink-release" (BinkClose). The ack's fourteenth field counts the Smacker hooks (3: the
      import and the two timers), its fifteenth Bink's Miles functions answered (11, once
      BinkOpenMiles has run: at the sound's start).
   8. The generator's seed (E24 M4 T1b1, *provisional*; the runbook, "The map's sequences"): the
      game seeds the C library's generator once, with the clock's time as its window opens
      (srand 0x617E4F at 0x4F7C40), and a computer player's turn draws from it (random_between
      0x50B230 and rand 0x617E5C: the main thread's seed, +0x14 of its thread data, 0x61D303).
      On the running clock that time and the draws before a sequence differ from run to run, so
      the harness may seed it again:
        <seq> seed <value>   the main thread's seed becomes <value> at its next clock read
      The ack's sixteenth field counts the seeds set so. The DLL's load runs on the main thread
      (DllMain, the process's first), whose id it keeps for this.
   9. The random map's clock (E14 T1b1, *provisional*; docs/lanes/rmg/ORACLE.md): the random map
      generator seeds the C library's generator with time() in seconds (0x536148), and the C
      library's time() (0x6198E0, its one caller the generator's) reads KERNEL32's GetLocalTime
      and GetSystemTime. With H3TIME set to a number of seconds since 1970-01-01 00:00 UTC (and
      H3CLOCK, as for everything here), Heroes3.exe's imports of both return that time, fixed,
      to every caller: with TZ=UTC (the shell's runs) time() returns H3TIME, the map's seed.
      The ack's seventeenth field counts the two imports hooked (2). */
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef DWORD(WINAPI *time_fn)(void);
typedef int(__fastcall *between_fn)(int, int);
typedef int(__cdecl *rand_fn)(void);
typedef BYTE *(__cdecl *thread_data_fn)(void);

#define RANDOM_BETWEEN 0x50B230u
#define CRT_RAND 0x617E5Cu
#define CRT_THREAD_DATA 0x61D303u /* 8.: the C library's per-thread data, its seed at +0x14 */

/* The most values a `roll <site> a,b,...` queues. */
#define QUEUE 8

/* The rolls' calls (docs/exe/map.toml): each a call of `target` returning the draw. */
struct site {
    const char *name;
    DWORD call;       /* the call instruction */
    DWORD target;     /* what it calls */
    int redirected;
    long count;       /* forced draws left */
    long value;       /* what they return */
    int fixed;        /* the site's other draws return `otherwise` */
    long otherwise;
    int logged;       /* its draws go to <H3CLOCK>\draws (2: with their range, E09 T34) */
    long queue[QUEUE]; /* E17 M4 T2c-2: the next draws' values, in order (`roll <site> a,b,..`) */
    int queued;       /* how many */
    int taken;        /* how many of them were drawn */
};

static struct site sites[] = {
    /* good_morale_roll 0x464920: random_between(1, 24) <= morale gives another turn */
    {"good_morale", 0x4649D3, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* bad_morale_check 0x464B40: random_between(1, 12) <= -morale freezes the stack (a computer
       side's second draw, 0x464BF0, may let it off: not redirected, hot seat has none) */
    {"bad_morale", 0x464BC7, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* fear_check 0x464D40: rand() mod 10 = 0 frightens the stack */
    {"fear", 0x464DF1, CRT_RAND, 0, 0, 0, 0, 0},
    /* melee_strike 0x441610: random_between(1, 24) <= luck, a lucky strike */
    {"luck", 0x441837, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* stack_shot 0x43F900: the same for a shot */
    {"shot_luck", 0x43F955, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* striker_effects 0x440BC0: per Mighty Gorgon random_between(1, 100) <= 10 kills one */
    {"death_stare", 0x440EE0, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* the Thunderbird's bolt: random_between(1, 100) <= 20 */
    {"thunderbird", 0x44119D, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* the Rust Dragon's acid damage: random_between(1, 100) <= 20 (its amount drawn next) */
    {"acid", 0x4414B7, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* strike_spell_choice 0x440500: the Wyvern Monarch's Poison, random_between(1, 100) <= 30 */
    {"poison", 0x440839, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* the same for Stone Gaze, the Medusas' and Basilisks' (76, 77, 106, 107): <= 20 (E17 M5
       T5b1) */
    {"stone_gaze", 0x440780, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* damage_stage1_with_luck 0x443840: the Dread Knight's (67) Death Blow, random_between(1,
       100) <= 20 outside an estimate (E17 M6 T5d2a) */
    {"death_blow", 0x4439B9, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* The wall shots' rolls (E17 M4 T2b; design §10.2): wall_shot_once 0x445EC0's damage,
       random_between(1, 100) against the BALLIST.TXT record's chances for 0, 1, 2 (in that
       order: the draw falls into the first whose running sum reaches it) */
    {"wall_damage", 0x445ED6, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* its hit: random_between(1, 100) <= the target kind's chance hits the aimed target */
    {"wall_hit", 0x445F25, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* a miss between two other walls equally near the aimed one: <= 50 takes the later */
    {"wall_miss_tie", 0x445F6C, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* wall_shot 0x445D30's next shot once its target fell: <= 50 takes the later of two */
    {"wall_retarget_tie", 0x445E44, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* catapult_turn 0x473C00's automatic aim: random_between(1, k), the k-th weakest wall */
    {"catapult_tie", 0x473DA0, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* earthquake 0x5A7C80's target per hit (E17 M4 T2c-2): random_between(1, k), the k-th
       standing target with hit points left */
    {"quake_target", 0x5A7DD8, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* ray_aim 0x5A5260: Chain Lightning's ray turns by random_between(10, 80) hundredths of a
       radian (or -80 to -10) at each aim while its target is far (E17 M3 T7c-3); logged */
    {"ray_aim", 0x5A53FC, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 1},
    /* E09 T23 (batch TWN-3; survey docs/rules/surveys/E09-growth.md §2 to §5): the adventure
       map's weekly and monthly draws. new_day_week_events 0x4C8780: random_between(1, 4) = 1
       makes the coming week a creature's (not drawn when the month's fourth week ends) */
    {"week_chance", 0x4C87CE, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* its creature: rand() mod the eligible creatures' count, the (r + 1)-th from the top */
    {"week_creature", 0x4C883E, CRT_RAND, 0, 0, 0, 0, 0},
    /* new_month 0x4C8F40: random_between(1, 10), 6 to 9 a creature's month, 10 the Plague,
       else normal (the week of the Imp's month is the Imp's whatever it draws) */
    {"month_kind", 0x4C8F5E, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* a creature's month's creature: month_creatures 0x63E678[random_between(0, 11)] */
    {"month_creature", 0x4C8FA5, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* E24 M4 T1a (design §5): the week's name, random_between(0, 14) into week_creature at every
       week's start (a creature's week overwrites it with its creature), ArrayTxt's records 43 to
       57; a normal month's, random_between(0, 9) into month_creature, records 32 to 41 */
    {"week_name", 0x4C87A6, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    {"month_name", 0x4C8FCC, RANDOM_BETWEEN, 0, 0, 0, 0, 0},
    /* logged: a neutral town's garrison week, random_between(0, 100) below 80 with a Fort,
       Citadel or Castle standing, else below 40, in the towns' order */
    {"garrison_chance_fort", 0x4C8E09, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 1},
    {"garrison_chance", 0x4C8E1C, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 1},
    /* neutral_town_garrison_week 0x4BF570: three random_between(0, n + 1) summed, the level */
    {"garrison_sum_1", 0x4BF5E5, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 1},
    {"garrison_sum_2", 0x4BF5F0, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 1},
    {"garrison_sum_3", 0x4BF5FB, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 1},
    /* its base creature's upgrade: random_between(1, 100) at most 5 */
    {"garrison_upgrade", 0x4BF74D, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 1},
    /* portal_of_summoning_week 0x5BD750: the owner's flagged dwelling, random_between(0, n - 1),
       then one of its creatures, rand() mod their number */
    {"portal_dwelling", 0x5BD825, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 1},
    {"portal_creature", 0x5BD86E, CRT_RAND, 0, 0, 0, 0, 0, 1},
    /* E09 T34 (batch TWN-4; survey docs/rules/surveys/E09-tavern.md §2.2, §4), logged with their
       range: tavern_hero_draw 0x4BB5E0's class, random_between(1, the class weights' total),
       and its type, random_between(1, the drawn class's free types); hero_leave_game
       0x4D9EC0's tavern slot after a retreat or a surrender, random_between(0, 1) */
    {"tavern_class", 0x4BB7AA, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 2},
    {"tavern_type", 0x4BB7CF, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 2},
    {"leave_slot", 0x4DA1C6, RANDOM_BETWEEN, 0, 0, 0, 0, 0, 2},
};
#define SITES (sizeof sites / sizeof *sites)

static volatile LONG forced_draws;
static volatile LONG refused; /* E17 M4 T3a: the roll commands and forced values refused */

static time_fn real_time;
static CRITICAL_SECTION lock;
static int frozen;
static DWORD stopped_at;   /* the time while frozen */
static DWORD shift;        /* running: the time is real_time() + shift */
static volatile LONG reads; /* the game's reads since the last command */
/* The clips' reads (bink_time) since the last command (E27 M1 T2: the ack's twelfth field). */
static volatile LONG clip_reads;
static char ctl_path[MAX_PATH], ack_path[MAX_PATH], tmp_path[MAX_PATH], draws_path[MAX_PATH];

static DWORD now(void)
{
    return frozen ? stopped_at : real_time() + shift;
}

static DWORD main_thread;  /* 6.: with sound only the main thread's reads count */
static DWORD game_thread;  /* 8.: the main thread, whatever the sound */
static volatile LONG seed_pending, seeds_set;
static int date_hooks; /* 9.: the ack's seventeenth field, the date imports hooked */
static DWORD seed_value;

static DWORD WINAPI virtual_time(void)
{
    DWORD t;
    if (seed_pending && GetCurrentThreadId() == game_thread) {
        /* 8.: the game's own thread data, so its own generator. */
        *(DWORD *)(((thread_data_fn)CRT_THREAD_DATA)() + 0x14) = seed_value;
        InterlockedExchange(&seed_pending, 0);
        InterlockedIncrement(&seeds_set);
    }
    if (!main_thread || GetCurrentThreadId() == main_thread)
        InterlockedIncrement(&reads);
    EnterCriticalSection(&lock);
    t = now();
    LeaveCriticalSection(&lock);
    return t;
}

/* The clips' clock (5.): the same time, not counted as the game's reads. */
static DWORD WINAPI bink_time(void)
{
    DWORD t;
    InterlockedIncrement(&clip_reads);
    EnterCriticalSection(&lock);
    t = now();
    LeaveCriticalSection(&lock);
    return t;
}

static void apply(const char *command, long ms)
{
    EnterCriticalSection(&lock);
    if (!strcmp(command, "freeze")) {
        stopped_at = now();
        frozen = 1;
    } else if (!strcmp(command, "step") && ms >= 0) {
        stopped_at = now() + (DWORD)ms;
        frozen = 1;
    } else if (!strcmp(command, "run")) {
        shift = now() - real_time();
        frozen = 0;
    }
    InterlockedExchange(&reads, 0);
    InterlockedExchange(&clip_reads, 0);
    LeaveCriticalSection(&lock);
}

/* A forced value `v` for a call drawing from `low` to `high` (`ranged`; rand() is not): v, or
   the call's own `draw` with the value refused (E17 M4 T3a). */
static int within(long v, int draw, int ranged, int low, int high)
{
    if (ranged && (v < low || v > high)) {
        refused++;
        return draw;
    }
    forced_draws++;
    return (int)v;
}

/* A redirected call's draw `draw` (from `low` to `high` when `ranged`), returning to `back` (the
   call + 5): forced or as drawn. */
static int forced(DWORD back, int draw, int ranged, int low, int high)
{
    size_t i;
    EnterCriticalSection(&lock);
    for (i = 0; i < SITES; i++) {
        struct site *s = &sites[i];
        if (s->call + 5 != back)
            continue;
        if (s->taken < s->queued) {
            draw = within(s->queue[s->taken++], draw, ranged, low, high);
        } else if (s->count > 0) {
            s->count--;
            draw = within(s->value, draw, ranged, low, high);
        } else if (s->fixed) {
            draw = within(s->otherwise, draw, ranged, low, high);
        }
        if (s->logged) {
            FILE *f = fopen(draws_path, "a");
            if (f) {
                if (s->logged == 2 && ranged)
                    fprintf(f, "%s %d %d %d\n", s->name, draw, low, high);
                else
                    fprintf(f, "%s %d\n", s->name, draw);
                fclose(f);
            }
        }
        break;
    }
    LeaveCriticalSection(&lock);
    return draw;
}

static int __fastcall forced_between(int a, int b)
{
    int draw = ((between_fn)RANDOM_BETWEEN)(a, b);
    return forced((DWORD)__builtin_return_address(0), draw, 1, a < b ? a : b, a < b ? b : a);
}

static int __cdecl forced_rand(void)
{
    int draw = ((rand_fn)CRT_RAND)();
    return forced((DWORD)__builtin_return_address(0), draw, 0, 0, 0);
}

/* `roll <site> <value> <count>`, `rolls <site> <value>|natural`, or (E17 M4 T2c-2) `roll <site>
   <a>,<b>,...`: the next draws return a, b, ... in order (at most QUEUE; before any `<count>`).
   A command it cannot read, a site it does not know, a count below 1 and a queue over values of
   the last one not drawn yet are refused (E17 M4 T3a): counted in `refused`, nothing set (the
   new queue set). */
static void force(const char *command, const char *name, const char *value, long count)
{
    size_t i;
    char *end;
    long v = strtol(value, &end, 10);
    int natural = !strcmp(value, "natural");
    long queue[QUEUE];
    int queued = 0, matched = 0;
    if (!natural && *value != 0 && *end == ',' && !strcmp(command, "roll")) {
        const char *at = value;
        for (;;) {
            if (queued == QUEUE) {
                InterlockedIncrement(&refused);
                return;
            }
            queue[queued++] = strtol(at, &end, 10);
            if (end == at || (*end != ',' && *end != 0)) {
                InterlockedIncrement(&refused);
                return;
            }
            if (*end == 0)
                break;
            at = end + 1;
        }
    } else if ((!natural && (*value == 0 || *end != 0))
               || (!strcmp(command, "roll") && (natural || count < 1))) {
        InterlockedIncrement(&refused);
        return;
    }
    EnterCriticalSection(&lock);
    for (i = 0; i < SITES; i++) {
        struct site *s = &sites[i];
        if (strcmp(s->name, name))
            continue;
        matched = 1;
        if (queued > 0) {
            if (s->taken < s->queued)
                refused++;
            memcpy(s->queue, queue, sizeof queue);
            s->queued = queued;
            s->taken = 0;
        } else if (!strcmp(command, "roll")) {
            s->value = v;
            s->count = count;
        } else if (!strcmp(command, "rolls")) {
            s->fixed = !natural;
            s->otherwise = natural ? 0 : v;
        }
    }
    if (!matched)
        refused++;
    InterlockedExchange(&reads, 0);
    InterlockedExchange(&clip_reads, 0);
    LeaveCriticalSection(&lock);
}

/* The forced draws still waiting for their roll. */
static long pending(void)
{
    size_t i;
    long n = 0;
    for (i = 0; i < SITES; i++)
        n += sites[i].count + sites[i].queued - sites[i].taken;
    return n;
}

/* Points each site's call at forced_between or forced_rand; returns how many it redirected. */
static int redirect(void)
{
    size_t i;
    int n = 0;
    for (i = 0; i < SITES; i++) {
        struct site *s = &sites[i];
        BYTE *at = (BYTE *)s->call;
        DWORD rel, old;
        DWORD hook = s->target == CRT_RAND ? (DWORD)forced_rand : (DWORD)forced_between;
        memcpy(&rel, at + 1, sizeof rel);
        if (at[0] != 0xE8 || s->call + 5 + rel != s->target)
            continue;
        if (!VirtualProtect(at, 5, PAGE_EXECUTE_READWRITE, &old))
            continue;
        rel = hook - (s->call + 5);
        memcpy(at + 1, &rel, sizeof rel);
        VirtualProtect(at, 5, old, &old);
        s->redirected = 1;
        n++;
    }
    FlushInstructionCache(GetCurrentProcess(), NULL, 0);
    return n;
}

static int redirected;
static int bink_clock; /* 5.: BINKW32.DLL's clock redirected */
static int smack_clock; /* 5.: SMACKW32.DLL's (E27 M1 T2) */
/* 5.: BinkGetSummary through summary_hook (E27 M1 T2). */
typedef void(WINAPI *summary_fn)(void *, void *);
static summary_fn real_summary;
static int summary_hooked;

static void WINAPI summary_hook(void *bink, void *summary)
{
    DWORD *field = (DWORD *)((BYTE *)bink + 0x110), held = *field;
    DWORD bytes = *(DWORD *)((BYTE *)bink + 0x10C), least = bytes / 4294967u + 1;
    if (held < least)
        *field = least;
    real_summary(bink, summary);
    *field = held;
}

/* 6. Sounds on the virtual clock. */
typedef void(WINAPI *ail0_fn)(void);
typedef int(WINAPI *ail1_fn)(void *);
typedef int(WINAPI *ail2_fn)(void *, int);
typedef int(WINAPI *ail3_fn)(void *, const void *, int);
#define SAMPLES 32
static struct sample {
    void *handle;
    DWORD start;          /* the virtual time it started */
    unsigned long frames; /* per loop */
    unsigned long rate;
    int loops;            /* 0: until ended */
    int playing;
    int paused;           /* E17 M5 T8b1, R7-6: stopped (AIL_stop_sample) while it played */
    DWORD paused_at;      /* the virtual time it was */
} samples[SAMPLES];
/* E17 M5 T8b1, R7-5: the calls the hooks could not follow (the ack's tenth field). */
static volatile LONG unfollowed;
static ail1_fn real_start, real_end, real_init, real_status, real_stop, real_resume;
static ail2_fn real_loops;
static ail0_fn real_serve;
static ail3_fn real_file;
static int sound_hooks; /* the imports hooked */
static char sounds_path[MAX_PATH];

static unsigned long le32(const BYTE *p)
{
    return p[0] | (unsigned long)p[1] << 8 | (unsigned long)p[2] << 16 | (unsigned long)p[3] << 24;
}

/* A Miles call under way or just made: the game's reads start again from 0, so the harness
   does not take the game for idle while Miles works (its calls take real time with a device:
   `machines`' heal showed a millisecond late). */
static void busy(void)
{
    InterlockedExchange(&reads, 0);
}

/* The table's entry of `handle`, made on first sight (none when the table is full: counted in
   `unfollowed`, E17 M5 T8b1, R7-5). */
static struct sample *sample_of(void *handle)
{
    int i;
    for (i = 0; i < SAMPLES; i++)
        if (samples[i].handle == handle)
            return &samples[i];
    for (i = 0; i < SAMPLES; i++)
        if (!samples[i].handle) {
            samples[i].handle = handle;
            samples[i].loops = 1;
            return &samples[i];
        }
    InterlockedIncrement(&unfollowed);
    return NULL;
}

/* The bytes readable from `p` on (E17 M5 T8b1, R7-5): to the end of its committed region, as
   AIL_set_sample_file's file comes without its length. */
static unsigned long readable(const BYTE *p)
{
    MEMORY_BASIC_INFORMATION m;
    if (!p || !VirtualQuery(p, &m, sizeof m) || m.State != MEM_COMMIT
        || (m.Protect & (PAGE_NOACCESS | PAGE_GUARD)))
        return 0;
    return (unsigned long)((const BYTE *)m.BaseAddress + m.RegionSize - p);
}

/* Whether the sample is still playing at the virtual time `t`. */
static int sounding(const struct sample *s, DWORD t)
{
    unsigned long long played, length;
    if (!s->playing)
        return 0;
    if (s->loops == 0)
        return 1;
    played = (unsigned long long)(DWORD)(t - s->start) * s->rate;
    length = (unsigned long long)s->frames * 1000u * (unsigned)s->loops;
    return played < length;
}

static int WINAPI hook_file(void *handle, const void *file, int block)
{
    busy();
    const BYTE *p = (const BYTE *)file;
    struct sample *s;
    int r = real_file(handle, file, block);
    busy();
    EnterCriticalSection(&lock);
    s = sample_of(handle);
    if (s) {
        /* E17 M5 T8b1, R7-5: the length set first, so that a file not read keeps none of the
           sample before; the chunks walked as h3-assets' wav.rs walks them (the first `fmt ` and
           the first `fact`; inside the RIFF size, the whole file when it is 0), and inside the
           memory readable here, the file's length unknown; a file without both counted in
           `unfollowed` (its length 0). */
        unsigned long have = readable(p), end, at = 12;
        int fmt = 0, fact = 0, counted = 0;
        s->frames = 0;
        s->rate = 0;
        if (have >= 12 && !memcmp(p, "RIFF", 4) && !memcmp(p + 8, "WAVE", 4)) {
            end = le32(p + 4);
            end = end == 0 || end > have - 8 ? have : end + 8;
            while (at + 8 <= end && !(fmt && fact)) {
                unsigned long n = le32(p + at + 4);
                if (!fmt && !memcmp(p + at, "fmt ", 4)) {
                    fmt = 1;
                    if (n >= 16 && at + 24 <= end)
                        s->rate = le32(p + at + 12);
                } else if (!fact && !memcmp(p + at, "fact", 4)) {
                    fact = 1;
                    if (n >= 4 && at + 12 <= end) {
                        s->frames = le32(p + at + 8);
                        counted = 1;
                    }
                }
                if (n > end - at - 8)
                    break;
                at += 8 + n + (n & 1);
            }
        }
        if (!s->rate || !counted) {
            s->frames = 0;
            InterlockedIncrement(&unfollowed);
        }
        s->playing = 0;
        s->paused = 0;
    }
    LeaveCriticalSection(&lock);
    return r;
}

static int WINAPI hook_loops(void *handle, int loops)
{
    struct sample *s;
    int r;
    busy();
    EnterCriticalSection(&lock);
    s = sample_of(handle);
    if (s)
        s->loops = loops;
    LeaveCriticalSection(&lock);
    r = real_loops(handle, loops);
    busy();
    return r;
}

static int WINAPI hook_start(void *handle)
{
    busy();
    struct sample *s;
    EnterCriticalSection(&lock);
    s = sample_of(handle);
    if (s) {
        FILE *f = fopen(sounds_path, "a");
        s->start = now();
        s->playing = 1;
        s->paused = 0;
        if (f) {
            unsigned long ms = s->rate ? (s->frames * 1000u + s->rate - 1) / s->rate : 0;
            fprintf(f, "%lu %d %lu %d\n", (unsigned long)s->start, (int)(s - samples), ms,
                    s->loops);
            fclose(f);
        }
    }
    LeaveCriticalSection(&lock);
    {
        int r = real_start(handle);
        busy();
        return r;
    }
}

static int WINAPI hook_end(void *handle)
{
    busy();
    struct sample *s;
    EnterCriticalSection(&lock);
    s = sample_of(handle);
    if (s) {
        s->playing = 0;
        s->paused = 0;
    }
    LeaveCriticalSection(&lock);
    {
        int r = real_end(handle);
        busy();
        return r;
    }
}

static int WINAPI hook_init(void *handle)
{
    busy();
    struct sample *s;
    EnterCriticalSection(&lock);
    s = sample_of(handle);
    if (s) {
        s->playing = 0;
        s->paused = 0;
        s->loops = 1;
    }
    LeaveCriticalSection(&lock);
    {
        int r = real_init(handle);
        busy();
        return r;
    }
}

/* Paused (the game's focus lost, 0x599C40): the time it stood still is added to its start.
   Only a sample still playing pauses (E17 M5 T8b1, R7-6, *provisional*: as Miles' stop leaves
   a done one done). */
static int WINAPI hook_stop(void *handle)
{
    busy();
    struct sample *s;
    EnterCriticalSection(&lock);
    s = sample_of(handle);
    if (s && !s->paused && sounding(s, now())) {
        s->paused = 1;
        s->paused_at = now();
    }
    LeaveCriticalSection(&lock);
    {
        int r = real_stop(handle);
        busy();
        return r;
    }
}

static int WINAPI hook_resume(void *handle)
{
    busy();
    struct sample *s;
    EnterCriticalSection(&lock);
    s = sample_of(handle);
    if (s && s->paused) {
        s->start += now() - s->paused_at;
        s->paused = 0;
    }
    LeaveCriticalSection(&lock);
    {
        int r = real_resume(handle);
        busy();
        return r;
    }
}

static int WINAPI hook_status(void *handle)
{
    struct sample *s;
    int r = -1;
    EnterCriticalSection(&lock);
    for (s = samples; s < samples + SAMPLES; s++)
        if (s->handle == handle && (s->playing || s->frames)) {
            /* Paused: stopped (8), as Miles answers (E17 M5 T8b1, R7-6). */
            r = s->paused ? 8 : sounding(s, now()) ? 4 : 2;
            break;
        }
    LeaveCriticalSection(&lock);
    return r >= 0 ? r : real_status(handle);
}

/* Miles' service of its mixer (play_sample's tail, 0x59A3A3): real time, no sample changed. */
static void WINAPI hook_serve(void)
{
    busy();
    real_serve();
    busy();
}

/* Points Heroes3.exe's Miles import `name` at `fn`; returns the import's own address. */
static FARPROC hook_ail(const char *name, FARPROC fn)
{
    BYTE *base = (BYTE *)GetModuleHandleA(NULL);
    IMAGE_NT_HEADERS32 *nt = (IMAGE_NT_HEADERS32 *)(base + ((IMAGE_DOS_HEADER *)base)->e_lfanew);
    IMAGE_DATA_DIRECTORY dir = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    IMAGE_IMPORT_DESCRIPTOR *d = (IMAGE_IMPORT_DESCRIPTOR *)(base + dir.VirtualAddress);
    HMODULE mss = GetModuleHandleA("mss32.dll");
    FARPROC target = mss ? GetProcAddress(mss, name) : NULL;
    if (!target)
        return NULL;
    for (; d->Name; d++) {
        DWORD *slot = (DWORD *)(base + d->FirstThunk);
        if (lstrcmpiA((const char *)(base + d->Name), "mss32.dll"))
            continue;
        for (; *slot; slot++) {
            DWORD old;
            if (*slot != (DWORD)target)
                continue;
            if (!VirtualProtect(slot, sizeof *slot, PAGE_READWRITE, &old))
                return NULL;
            *slot = (DWORD)fn;
            VirtualProtect(slot, sizeof *slot, old, &old);
            sound_hooks++;
            return target;
        }
    }
    return NULL;
}

static void hook_sounds(void)
{
    real_file = (ail3_fn)hook_ail("_AIL_set_sample_file@12", (FARPROC)hook_file);
    real_loops = (ail2_fn)hook_ail("_AIL_set_sample_loop_count@8", (FARPROC)hook_loops);
    real_start = (ail1_fn)hook_ail("_AIL_start_sample@4", (FARPROC)hook_start);
    real_end = (ail1_fn)hook_ail("_AIL_end_sample@4", (FARPROC)hook_end);
    real_init = (ail1_fn)hook_ail("_AIL_init_sample@4", (FARPROC)hook_init);
    real_stop = (ail1_fn)hook_ail("_AIL_stop_sample@4", (FARPROC)hook_stop);
    real_resume = (ail1_fn)hook_ail("_AIL_resume_sample@4", (FARPROC)hook_resume);
    real_status = (ail1_fn)hook_ail("_AIL_sample_status@4", (FARPROC)hook_status);
    real_serve = (ail0_fn)hook_ail("_AIL_serve@0", (FARPROC)hook_serve);
}

/* 7. The clips' sound on the virtual clock (E27 M2 T5b2). */
#include "vsound.h"

typedef BYTE(WINAPI *use_ds_fn)(void *);
static use_ds_fn smack_use_ds; /* SMACKW32.DLL's SmackSoundUseDirectSound */
static int smack_sound;        /* the ack's fourteenth field: the Smacker hooks in place */
static volatile LONG bink_sound; /* its fifteenth: Bink's Miles functions answered */
static char clip_sound_path[MAX_PATH];
static CRITICAL_SECTION clip_lock; /* the log and Bink's samples: Bink's own thread calls too */

/* The clips' sound clock: the virtual time, not counted as anyone's reads. */
static DWORD clip_now(void)
{
    DWORD t;
    EnterCriticalSection(&lock);
    t = now();
    LeaveCriticalSection(&lock);
    return t;
}

static int WINAPI use_mss_hook(void *driver)
{
    (void)driver;
    vnote("use", 0, 0);
    return smack_use_ds((void *)&vsound);
}

static UINT WINAPI smack_set_event(UINT delay, UINT resolution, LPTIMECALLBACK f, DWORD_PTR user,
                                   UINT kind)
{
    (void)f;
    (void)user;
    vnote("timer", (long)delay, (long)(resolution | kind << 16));
    return 1;
}

static UINT WINAPI smack_kill_event(UINT id)
{
    vnote("timer-killed", (long)id, 0);
    return TIMERR_NOERROR;
}

/* Bink's sample: two buffers, played one after the other from the first load at the sample's
   bytes a second; a buffer is free once played out (`bink_ready` gives the one after the last
   loaded while it is free). Paused by BinkPause (AIL_stop_sample) until AIL_resume_sample. */
#define BINK_SAMPLES 4
static struct bsample {
    void *handle;
    unsigned long bps;      /* bytes a second: the rate times the format's bytes a frame */
    long rate, format;
    DWORD length[2];
    int loaded[2];          /* loaded, not yet played out */
    int fill;               /* the buffer to load next */
    int current;            /* the buffer playing, -1 none */
    DWORD since;            /* when the run of buffers began to play */
    unsigned long long done; /* the bytes of that run's buffers played out */
    int paused;
    DWORD paused_at;
    int heard;              /* it has played */
} bsamples[BINK_SAMPLES];

typedef DWORD(WINAPI *mins_fn)(void *, long, long);
typedef void *(WINAPI *alloc_fn)(void *);
typedef void(WINAPI *v1_fn)(void *);
typedef void(WINAPI *v2_fn)(void *, long);
typedef void(WINAPI *v3_fn)(void *, long, DWORD);
typedef long(WINAPI *ready_fn)(void *);
typedef void(WINAPI *load_fn)(void *, DWORD, const void *, DWORD);
static mins_fn real_minimum;
static alloc_fn real_allocate;
static v1_fn real_binit, real_release, real_bstop, real_bresume;
static v2_fn real_rate, real_volume;
static v3_fn real_type;
static ready_fn real_ready;
static load_fn real_load;

static void bnote(const char *what, long a, long b)
{
    char name[32];
    snprintf(name, sizeof name, "bink-%s", what);
    vnote(name, a, b);
}

static struct bsample *bsample_of(void *handle)
{
    int i;
    for (i = 0; i < BINK_SAMPLES; i++)
        if (bsamples[i].handle == handle)
            return &bsamples[i];
    return NULL;
}

static void bink_bps(struct bsample *s)
{
    /* Miles' DIG_F_*: bit 0 16-bit, bit 1 stereo. */
    s->bps = (unsigned long)s->rate * (s->format & 1 ? 2u : 1u) * (s->format & 2 ? 2u : 1u);
}

/* The buffers played out by `t`. */
static void bink_advance(struct bsample *s, DWORD t)
{
    while (s->current >= 0 && !s->paused && s->bps) {
        int c = s->current;
        unsigned long long end = s->done + s->length[c];
        if ((unsigned long long)(DWORD)(t - s->since) * s->bps < end * 1000u)
            return;
        s->loaded[c] = 0;
        if (s->loaded[c ^ 1]) {
            s->current = c ^ 1;
            s->done = end;
        } else {
            /* Dry: the next load starts a new run at its own time. */
            s->current = -1;
            bnote("dry", c, 0);
        }
    }
}

static DWORD WINAPI bink_minimum(void *driver, long rate, long format)
{
    DWORD n = real_minimum(driver, rate, format);
    EnterCriticalSection(&clip_lock);
    bnote("minimum", (long)n, rate);
    LeaveCriticalSection(&clip_lock);
    return n;
}

static void *WINAPI bink_allocate(void *driver)
{
    void *h = real_allocate(driver);
    int i;
    EnterCriticalSection(&clip_lock);
    for (i = 0; h && i < BINK_SAMPLES; i++)
        if (!bsamples[i].handle) {
            memset(&bsamples[i], 0, sizeof bsamples[i]);
            bsamples[i].handle = h;
            bsamples[i].current = -1;
            break;
        }
    if (h && i == BINK_SAMPLES)
        InterlockedIncrement(&unfollowed);
    bnote("allocate", i, 0);
    LeaveCriticalSection(&clip_lock);
    return h;
}

static void WINAPI bink_init(void *h)
{
    struct bsample *s;
    real_binit(h);
    EnterCriticalSection(&clip_lock);
    if ((s = bsample_of(h))) {
        void *keep = s->handle;
        memset(s, 0, sizeof *s);
        s->handle = keep;
        s->current = -1;
    }
    LeaveCriticalSection(&clip_lock);
}

static void WINAPI bink_type(void *h, long format, DWORD flags)
{
    struct bsample *s;
    real_type(h, format, flags);
    EnterCriticalSection(&clip_lock);
    if ((s = bsample_of(h))) {
        s->format = format;
        bink_bps(s);
    }
    LeaveCriticalSection(&clip_lock);
}

static void WINAPI bink_rate(void *h, long rate)
{
    struct bsample *s;
    real_rate(h, rate);
    EnterCriticalSection(&clip_lock);
    if ((s = bsample_of(h))) {
        s->rate = rate;
        bink_bps(s);
    }
    LeaveCriticalSection(&clip_lock);
}

static void WINAPI bink_volume(void *h, long volume)
{
    real_volume(h, volume);
    EnterCriticalSection(&clip_lock);
    bnote("volume", volume, 0);
    LeaveCriticalSection(&clip_lock);
}

static long WINAPI bink_ready(void *h)
{
    struct bsample *s;
    long n = -1;
    EnterCriticalSection(&clip_lock);
    if ((s = bsample_of(h))) {
        bink_advance(s, clip_now());
        n = s->loaded[s->fill] ? -1 : s->fill;
    }
    LeaveCriticalSection(&clip_lock);
    return n;
}

static void WINAPI bink_load(void *h, DWORD n, const void *buffer, DWORD length)
{
    struct bsample *s;
    DWORD t = clip_now();
    (void)buffer;
    EnterCriticalSection(&clip_lock);
    if ((s = bsample_of(h)) && n < 2) {
        bink_advance(s, t);
        s->loaded[n] = 1;
        s->length[n] = length;
        s->fill = (int)(n ^ 1);
        bnote("load", (long)n, (long)length);
        if (s->current < 0) {
            s->current = (int)n;
            s->since = t;
            s->done = 0;
            if (!s->heard) {
                s->heard = 1;
                bnote("play", (long)n, (long)s->bps);
            }
        }
    } else {
        InterlockedIncrement(&unfollowed);
    }
    LeaveCriticalSection(&clip_lock);
}

static void WINAPI bink_stop(void *h)
{
    struct bsample *s;
    DWORD t = clip_now();
    EnterCriticalSection(&clip_lock);
    if ((s = bsample_of(h)) && !s->paused) {
        bink_advance(s, t);
        s->paused = 1;
        s->paused_at = t;
    }
    bnote("stop", 0, 0);
    LeaveCriticalSection(&clip_lock);
}

static void WINAPI bink_resume(void *h)
{
    struct bsample *s;
    DWORD t = clip_now();
    EnterCriticalSection(&clip_lock);
    if ((s = bsample_of(h)) && s->paused) {
        s->since += t - s->paused_at;
        s->paused = 0;
    }
    bnote("resume", 0, 0);
    LeaveCriticalSection(&clip_lock);
}

static void WINAPI bink_release(void *h)
{
    struct bsample *s;
    EnterCriticalSection(&clip_lock);
    if ((s = bsample_of(h)))
        s->handle = NULL;
    bnote("release", 0, 0);
    LeaveCriticalSection(&clip_lock);
    real_release(h);
}

/* The Miles functions BinkOpenMiles takes by name that the hooks answer, and where each keeps the
   real one. */
static const struct {
    const char *name;
    FARPROC hook;
    FARPROC *real;
} bink_calls[] = {
    {"_AIL_minimum_sample_buffer_size@12", (FARPROC)bink_minimum, (FARPROC *)&real_minimum},
    {"_AIL_allocate_sample_handle@4", (FARPROC)bink_allocate, (FARPROC *)&real_allocate},
    {"_AIL_init_sample@4", (FARPROC)bink_init, (FARPROC *)&real_binit},
    {"_AIL_set_sample_type@12", (FARPROC)bink_type, (FARPROC *)&real_type},
    {"_AIL_set_sample_playback_rate@8", (FARPROC)bink_rate, (FARPROC *)&real_rate},
    {"_AIL_set_sample_volume@8", (FARPROC)bink_volume, (FARPROC *)&real_volume},
    {"_AIL_stop_sample@4", (FARPROC)bink_stop, (FARPROC *)&real_bstop},
    {"_AIL_resume_sample@4", (FARPROC)bink_resume, (FARPROC *)&real_bresume},
    {"_AIL_sample_buffer_ready@4", (FARPROC)bink_ready, (FARPROC *)&real_ready},
    {"_AIL_load_sample_buffer@16", (FARPROC)bink_load, (FARPROC *)&real_load},
    {"_AIL_release_sample_handle@4", (FARPROC)bink_release, (FARPROC *)&real_release},
};

typedef FARPROC(WINAPI *proc_fn)(HMODULE, LPCSTR);
static proc_fn real_proc;

static FARPROC WINAPI proc_hook(HMODULE module, LPCSTR name)
{
    FARPROC real = real_proc(module, name);
    unsigned i;
    if (!real || !HIWORD((DWORD)name) || module != GetModuleHandleA("mss32.dll"))
        return real;
    for (i = 0; i < sizeof bink_calls / sizeof bink_calls[0]; i++)
        if (!strcmp(name, bink_calls[i].name)) {
            *bink_calls[i].real = real;
            InterlockedIncrement(&bink_sound);
            return bink_calls[i].hook;
        }
    return real;
}

/* Points a module's import of `name` from `dll` (found by the address it holds: SMACKW32's and
   BINKW32's tables keep no names once bound) at `fn`; the import's own address, or NULL. */
static FARPROC hook_import(HMODULE module, const char *dll, const char *name, FARPROC fn)
{
    BYTE *base = (BYTE *)module;
    IMAGE_NT_HEADERS32 *nt;
    IMAGE_DATA_DIRECTORY dir;
    IMAGE_IMPORT_DESCRIPTOR *d;
    HMODULE from = GetModuleHandleA(dll);
    FARPROC target = from ? GetProcAddress(from, name) : NULL;
    if (!base || !target)
        return NULL;
    nt = (IMAGE_NT_HEADERS32 *)(base + ((IMAGE_DOS_HEADER *)base)->e_lfanew);
    dir = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    d = (IMAGE_IMPORT_DESCRIPTOR *)(base + dir.VirtualAddress);
    for (; d->Name; d++) {
        DWORD *slot = (DWORD *)(base + d->FirstThunk);
        if (lstrcmpiA((const char *)(base + d->Name), dll))
            continue;
        for (; *slot; slot++) {
            DWORD old;
            if (*slot != (DWORD)target)
                continue;
            if (!VirtualProtect(slot, sizeof *slot, PAGE_READWRITE, &old))
                return NULL;
            *slot = (DWORD)fn;
            VirtualProtect(slot, sizeof *slot, old, &old);
            return target;
        }
    }
    return NULL;
}

static void hook_clip_sounds(void)
{
    HMODULE smack = GetModuleHandleA("SMACKW32.DLL");
    InitializeCriticalSection(&clip_lock);
    vsound_now = clip_now;
    vsound_brief = 1;
    vsound_log = fopen(clip_sound_path, "w");
    if (vsound_log)
        setvbuf(vsound_log, NULL, _IONBF, 0);
    smack_use_ds = smack ? (use_ds_fn)GetProcAddress(smack, "_SmackSoundUseDirectSound@4") : NULL;
    if (smack_use_ds && hook_import(GetModuleHandleA(NULL), "smackw32.dll", "_SmackSoundUseMSS@4",
                                    (FARPROC)use_mss_hook))
        smack_sound++;
    if (hook_import(smack, "winmm.dll", "timeSetEvent", (FARPROC)smack_set_event))
        smack_sound++;
    if (hook_import(smack, "winmm.dll", "timeKillEvent", (FARPROC)smack_kill_event))
        smack_sound++;
    real_proc = (proc_fn)hook_import(GetModuleHandleA("BINKW32.DLL"), "kernel32.dll",
                                     "GetProcAddress", (FARPROC)proc_hook);
}

/* Polls the command file and writes the acknowledgement, every 2 ms. */
static DWORD WINAPI control(LPVOID unused)
{
    unsigned long applied = 0;
    (void)unused;
    for (;;) {
        char line[128] = {0}, command[32] = {0}, name[32] = {0}, value[32] = {0};
        unsigned long seq = 0;
        long ms = -1, count = 1, waiting;
        DWORD t;
        FILE *f = fopen(ctl_path, "r");
        if (f) {
            if (fgets(line, sizeof line, f)
                && sscanf(line, "%lu %31s", &seq, command) == 2 && seq != applied) {
                if (!strcmp(command, "roll") || !strcmp(command, "rolls")) {
                    if (sscanf(line, "%lu %31s %31s %31s %ld", &seq, command, name, value, &count) >= 4)
                        force(command, name, value, count);
                } else if (!strcmp(command, "seed")) {
                    unsigned long v;
                    if (sscanf(line, "%lu %31s %lu", &seq, command, &v) == 3) {
                        EnterCriticalSection(&lock);
                        seed_value = (DWORD)v;
                        InterlockedExchange(&seed_pending, 1);
                        LeaveCriticalSection(&lock);
                    } else {
                        refused++;
                    }
                } else {
                    sscanf(line, "%lu %31s %ld", &seq, command, &ms);
                    apply(command, ms);
                }
                applied = seq;
            }
            fclose(f);
        }
        EnterCriticalSection(&lock);
        t = now();
        waiting = pending();
        LeaveCriticalSection(&lock);
        f = fopen(tmp_path, "w");
        if (f) {
            fprintf(f, "%lu %lu %ld %ld %ld %d %ld %d %d %ld %d %ld %d %d %ld %ld %d\n", applied,
                    (unsigned long)t, (long)reads, (long)forced_draws, waiting, redirected,
                    (long)refused, bink_clock, sound_hooks, (long)unfollowed, smack_clock,
                    (long)clip_reads, summary_hooked, smack_sound, (long)bink_sound,
                    (long)seeds_set, date_hooks);
            fclose(f);
            MoveFileExA(tmp_path, ack_path, MOVEFILE_REPLACE_EXISTING);
        }
        Sleep(2);
    }
    return 0;
}

/* Points a module's timeGetTime import slot at `clock` (Heroes3.exe's at virtual_time;
   BINKW32.DLL's, 5., at bink_time). */
static int hook(HMODULE module, time_fn clock)
{
    BYTE *base = (BYTE *)module;
    IMAGE_NT_HEADERS32 *nt;
    IMAGE_DATA_DIRECTORY dir;
    IMAGE_IMPORT_DESCRIPTOR *d;
    FARPROC target = GetProcAddress(GetModuleHandleA("winmm.dll"), "timeGetTime");
    /* A module not loaded is tested before its headers are read (E17 M5 T8b1, R7-11). */
    if (!base || !target)
        return 0;
    nt = (IMAGE_NT_HEADERS32 *)(base + ((IMAGE_DOS_HEADER *)base)->e_lfanew);
    dir = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    d = (IMAGE_IMPORT_DESCRIPTOR *)(base + dir.VirtualAddress);
    for (; d->Name; d++) {
        DWORD *slot = (DWORD *)(base + d->FirstThunk);
        if (lstrcmpiA((const char *)(base + d->Name), "winmm.dll"))
            continue;
        for (; *slot; slot++) {
            DWORD old;
            if (*slot != (DWORD)target)
                continue;
            if (!VirtualProtect(slot, sizeof *slot, PAGE_READWRITE, &old))
                return 0;
            real_time = (time_fn)target;
            *slot = (DWORD)clock;
            VirtualProtect(slot, sizeof *slot, old, &old);
            return 1;
        }
    }
    return 0;
}

/* Points a module's import of `name` from `dll` (by name) at `replacement`; the original
   function, or NULL when the module imports no such name (5., BinkGetSummary). */
static FARPROC hook_by_name(HMODULE module, const char *dll, const char *name,
                            FARPROC replacement)
{
    BYTE *base = (BYTE *)module;
    IMAGE_NT_HEADERS32 *nt;
    IMAGE_DATA_DIRECTORY dir;
    IMAGE_IMPORT_DESCRIPTOR *d;
    if (!base)
        return NULL;
    nt = (IMAGE_NT_HEADERS32 *)(base + ((IMAGE_DOS_HEADER *)base)->e_lfanew);
    dir = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    d = (IMAGE_IMPORT_DESCRIPTOR *)(base + dir.VirtualAddress);
    for (; d->Name; d++) {
        DWORD *slot = (DWORD *)(base + d->FirstThunk);
        DWORD *names = (DWORD *)(base + (d->OriginalFirstThunk ? d->OriginalFirstThunk
                                                                : d->FirstThunk));
        if (lstrcmpiA((const char *)(base + d->Name), dll))
            continue;
        for (; *names; names++, slot++) {
            DWORD old;
            FARPROC original;
            if (*names & 0x80000000u
                || strcmp((const char *)(base + *names + 2), name))
                continue;
            if (!VirtualProtect(slot, sizeof *slot, PAGE_READWRITE, &old))
                return NULL;
            original = (FARPROC)*slot;
            *slot = (DWORD)replacement;
            VirtualProtect(slot, sizeof *slot, old, &old);
            return original;
        }
    }
    return NULL;
}

/* 9.: the fixed date both imports return. */
static SYSTEMTIME fixed_date;

static void WINAPI fixed_local_time(SYSTEMTIME *t)
{
    *t = fixed_date;
}

static void WINAPI fixed_system_time(SYSTEMTIME *t)
{
    *t = fixed_date;
}

/* `s` seconds since 1970-01-01 00:00 UTC as a SYSTEMTIME (the proleptic Gregorian calendar's
   date from the day count, H. Hinnant's days_from_civil inverted). */
static void date_of(unsigned long s, SYSTEMTIME *t)
{
    long days = (long)(s / 86400), secs = (long)(s % 86400);
    long z = days + 719468, era = z / 146097, doe = z - era * 146097;
    long yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    long doy = doe - (365 * yoe + yoe / 4 - yoe / 100), mp = (5 * doy + 2) / 153;
    long m = mp < 10 ? mp + 3 : mp - 9;
    t->wYear = (WORD)(yoe + era * 400 + (m <= 2));
    t->wMonth = (WORD)m;
    t->wDay = (WORD)(doy - (153 * mp + 2) / 5 + 1);
    t->wDayOfWeek = (WORD)((days + 4) % 7); /* 1970-01-01 was a Thursday */
    t->wHour = (WORD)(secs / 3600);
    t->wMinute = (WORD)(secs / 60 % 60);
    t->wSecond = (WORD)(secs % 60);
    t->wMilliseconds = 0;
}

BOOL WINAPI DllMain(HINSTANCE instance, DWORD reason, LPVOID reserved)
{
    char dir[MAX_PATH - 8];
    DWORD n;
    (void)instance;
    (void)reserved;
    if (reason != DLL_PROCESS_ATTACH)
        return TRUE;
    n = GetEnvironmentVariableA("H3CLOCK", dir, sizeof dir);
    if (n == 0 || n >= sizeof dir)
        return TRUE;
    snprintf(ctl_path, sizeof ctl_path, "%s\\ctl", dir);
    snprintf(ack_path, sizeof ack_path, "%s\\ack", dir);
    snprintf(tmp_path, sizeof tmp_path, "%s\\ack.tmp", dir);
    snprintf(draws_path, sizeof draws_path, "%s\\draws", dir);
    snprintf(sounds_path, sizeof sounds_path, "%s\\sounds", dir);
    snprintf(clip_sound_path, sizeof clip_sound_path, "%s\\clipsound", dir);
    {
        FILE *f = fopen(draws_path, "w");
        if (f)
            fclose(f);
        f = fopen(sounds_path, "w");
        if (f)
            fclose(f);
    }
    InitializeCriticalSection(&lock);
    game_thread = GetCurrentThreadId(); /* 8.: DllMain runs on the main thread */
    {
        char at[16];
        DWORD m = GetEnvironmentVariableA("H3TIME", at, sizeof at);
        if (m > 0 && m < sizeof at) {
            HMODULE exe = GetModuleHandleA(NULL);
            date_of(strtoul(at, NULL, 10), &fixed_date);
            date_hooks = (hook_by_name(exe, "kernel32.dll", "GetLocalTime",
                                       (FARPROC)fixed_local_time) != NULL)
                         + (hook_by_name(exe, "kernel32.dll", "GetSystemTime",
                                         (FARPROC)fixed_system_time) != NULL);
        }
    }
    {
        char at[16];
        DWORD m = GetEnvironmentVariableA("H3FROZEN", at, sizeof at);
        if (m > 0 && m < sizeof at) {
            stopped_at = (DWORD)strtoul(at, NULL, 10);
            frozen = 1;
        }
    }
    if (hook(GetModuleHandleA(NULL), virtual_time)) {
        redirected = redirect();
        /* Heroes3.exe imports BINKW32.DLL, so it is mapped and bound before this runs. */
        bink_clock = hook(GetModuleHandleA("BINKW32.DLL"), bink_time);
        smack_clock = hook(GetModuleHandleA("SMACKW32.DLL"), bink_time);
        real_summary = (summary_fn)hook_by_name(GetModuleHandleA(NULL), "binkw32.dll",
                                                "_BinkGetSummary@8", (FARPROC)summary_hook);
        summary_hooked = real_summary != NULL;
        if (GetEnvironmentVariableA("H3SOUND", NULL, 0)) {
            hook_sounds();
            hook_clip_sounds(); /* 7. */
            /* DllMain runs on the process's first thread, the game's main one. */
            main_thread = GetCurrentThreadId();
        }
        CreateThread(NULL, 0, control, NULL, 0, NULL);
    }
    return TRUE;
}
