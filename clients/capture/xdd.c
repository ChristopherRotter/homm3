/* xdd.dll for the reference captures (clients/capture/README.md). Built by setup.sh with
   mingw-w64; our own code. The original's files never change; in its memory only the import
   slot of 2 and the roll calls of 3.

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
      with "<forced> <pending> <sites>": the draws forced so far, the forced draws still waiting
      for their roll and the calls redirected.
   4. Logged draws (E17 M3 T7c-3, *provisional*): a site marked `logged` (Chain Lightning's
      rays turning in `ray_aim`) appends each of its draws, as returned, to <H3CLOCK>\draws, one
      line "<site> <value>", so the frame check can draw the same rays; the file starts empty
      with the game. Nothing is forced there unless the harness asks, as at any site. */
#include <windows.h>
#include <stdio.h>
#include <string.h>

typedef DWORD(WINAPI *time_fn)(void);
typedef int(__fastcall *between_fn)(int, int);
typedef int(__cdecl *rand_fn)(void);

#define RANDOM_BETWEEN 0x50B230u
#define CRT_RAND 0x617E5Cu

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
    int logged;       /* its draws go to <H3CLOCK>\draws */
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
};
#define SITES (sizeof sites / sizeof *sites)

static volatile LONG forced_draws;

static time_fn real_time;
static CRITICAL_SECTION lock;
static int frozen;
static DWORD stopped_at;   /* the time while frozen */
static DWORD shift;        /* running: the time is real_time() + shift */
static volatile LONG reads; /* the game's reads since the last command */
static char ctl_path[MAX_PATH], ack_path[MAX_PATH], tmp_path[MAX_PATH], draws_path[MAX_PATH];

static DWORD now(void)
{
    return frozen ? stopped_at : real_time() + shift;
}

static DWORD WINAPI virtual_time(void)
{
    DWORD t;
    InterlockedIncrement(&reads);
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
    LeaveCriticalSection(&lock);
}

/* A redirected call's draw `draw`, returning to `back` (the call + 5): forced or as drawn. */
static int forced(DWORD back, int draw)
{
    size_t i;
    EnterCriticalSection(&lock);
    for (i = 0; i < SITES; i++) {
        struct site *s = &sites[i];
        if (s->call + 5 != back)
            continue;
        if (s->taken < s->queued) {
            draw = (int)s->queue[s->taken++];
            forced_draws++;
        } else if (s->count > 0) {
            s->count--;
            draw = (int)s->value;
            forced_draws++;
        } else if (s->fixed) {
            draw = (int)s->otherwise;
            forced_draws++;
        }
        if (s->logged) {
            FILE *f = fopen(draws_path, "a");
            if (f) {
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
    return forced((DWORD)__builtin_return_address(0), draw);
}

static int __cdecl forced_rand(void)
{
    int draw = ((rand_fn)CRT_RAND)();
    return forced((DWORD)__builtin_return_address(0), draw);
}

/* `roll <site> <value> <count>`, `rolls <site> <value>|natural`, or (E17 M4 T2c-2) `roll <site>
   <a>,<b>,...`: the next draws return a, b, ... in order (at most QUEUE; before any `<count>`). */
static void force(const char *command, const char *name, const char *value, long count)
{
    size_t i;
    char *end;
    long v = strtol(value, &end, 10);
    int natural = !strcmp(value, "natural");
    long queue[QUEUE];
    int queued = 0;
    if (!natural && *value != 0 && *end == ',' && !strcmp(command, "roll")) {
        const char *at = value;
        for (;;) {
            if (queued == QUEUE)
                return;
            queue[queued++] = strtol(at, &end, 10);
            if (end == at || (*end != ',' && *end != 0))
                return;
            if (*end == 0)
                break;
            at = end + 1;
        }
    } else if (!natural && (*value == 0 || *end != 0)) {
        return;
    }
    EnterCriticalSection(&lock);
    for (i = 0; i < SITES; i++) {
        struct site *s = &sites[i];
        if (strcmp(s->name, name))
            continue;
        if (queued > 0) {
            memcpy(s->queue, queue, sizeof queue);
            s->queued = queued;
            s->taken = 0;
        } else if (!strcmp(command, "roll") && !natural && count >= 0) {
            s->value = v;
            s->count = count;
        } else if (!strcmp(command, "rolls")) {
            s->fixed = !natural;
            s->otherwise = natural ? 0 : v;
        }
    }
    InterlockedExchange(&reads, 0);
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
            fprintf(f, "%lu %lu %ld %ld %ld %d\n", applied, (unsigned long)t, (long)reads,
                    (long)forced_draws, waiting, redirected);
            fclose(f);
            MoveFileExA(tmp_path, ack_path, MOVEFILE_REPLACE_EXISTING);
        }
        Sleep(2);
    }
    return 0;
}

/* Points Heroes3.exe's timeGetTime import slot at virtual_time. */
static int hook(void)
{
    BYTE *base = (BYTE *)GetModuleHandleA(NULL);
    IMAGE_NT_HEADERS32 *nt = (IMAGE_NT_HEADERS32 *)(base + ((IMAGE_DOS_HEADER *)base)->e_lfanew);
    IMAGE_DATA_DIRECTORY dir = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    IMAGE_IMPORT_DESCRIPTOR *d = (IMAGE_IMPORT_DESCRIPTOR *)(base + dir.VirtualAddress);
    FARPROC target = GetProcAddress(GetModuleHandleA("winmm.dll"), "timeGetTime");
    if (!target)
        return 0;
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
            *slot = (DWORD)virtual_time;
            VirtualProtect(slot, sizeof *slot, old, &old);
            return 1;
        }
    }
    return 0;
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
    {
        FILE *f = fopen(draws_path, "w");
        if (f)
            fclose(f);
    }
    InitializeCriticalSection(&lock);
    if (hook()) {
        redirected = redirect();
        CreateThread(NULL, 0, control, NULL, 0, NULL);
    }
    return TRUE;
}
