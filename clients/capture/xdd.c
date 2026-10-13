/* xdd.dll for the reference captures (clients/capture/README.md). Built by setup.sh with
   mingw-w64; our own code, no byte of the original changes.

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
      clock is idle: it has drawn everything up to that time). Without H3CLOCK nothing changes. */
#include <windows.h>
#include <stdio.h>

typedef DWORD(WINAPI *time_fn)(void);

static time_fn real_time;
static CRITICAL_SECTION lock;
static int frozen;
static DWORD stopped_at;   /* the time while frozen */
static DWORD shift;        /* running: the time is real_time() + shift */
static volatile LONG reads; /* the game's reads since the last command */
static char ctl_path[MAX_PATH], ack_path[MAX_PATH], tmp_path[MAX_PATH];

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

/* Polls the command file and writes the acknowledgement, every 2 ms. */
static DWORD WINAPI control(LPVOID unused)
{
    unsigned long applied = 0;
    (void)unused;
    for (;;) {
        char line[128] = {0}, command[32] = {0};
        unsigned long seq = 0;
        long ms = -1;
        DWORD t;
        FILE *f = fopen(ctl_path, "r");
        if (f) {
            if (fgets(line, sizeof line, f)
                && sscanf(line, "%lu %31s %ld", &seq, command, &ms) >= 2 && seq != applied) {
                apply(command, ms);
                applied = seq;
            }
            fclose(f);
        }
        EnterCriticalSection(&lock);
        t = now();
        LeaveCriticalSection(&lock);
        f = fopen(tmp_path, "w");
        if (f) {
            fprintf(f, "%lu %lu %ld\n", applied, (unsigned long)t, (long)reads);
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
    InitializeCriticalSection(&lock);
    if (hook())
        CreateThread(NULL, 0, control, NULL, 0, NULL);
    return TRUE;
}
