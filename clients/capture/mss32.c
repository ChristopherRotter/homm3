/* mss32.dll (E27 M2 T2; *provisional*; clients/capture/README.md, "The clips' sound"): a stand-in
   for Miles' MSS32.DLL that SMACKW32.DLL loads when smackprobe.exe runs it, so that the sound the
   DLL decodes is written to files instead of played. Built by setup.sh with mingw-w64 into the
   run directory's mss/ (the game's own MSS32.DLL stays beside Heroes3.exe for the captures); the
   probe loads it by its path before the DLL asks for "MSS32.DLL", which then finds it loaded.
   Our own code: the functions SMACKW32.DLL's SmackSoundUseMSS asks for by name (GetProcAddress,
   0x100079F0), with the arguments it passes, and the version string it checks (string 1, at least
   "3.50F"; the game's MSS32.DLL says 5.0e).

   What the DLL sees: a digital driver whose preference 15 (DIG_USE_WAVEOUT) is 1, so it streams
   through Miles' double buffers (AIL_sample_buffer_ready, AIL_load_sample_buffer: its service
   0x10007D50 copies the decoded ring into them); the game's own driver usually says 0, where the
   DLL writes the same ring into Miles' DirectSound buffer instead (0x100095C0). Both buffers are
   always free: each load is written out at once. The DLL services its tracks from a Miles timer
   (AIL_register_timer); the stand-in calls the registered callbacks only when the probe asks
   (MssPump), so nothing runs on another thread.

   The probe's own exports:
     MssRecord(path)  each sample handle's loads to <path>.<handle>.pcm (the bytes in order) and
                      every call to <path>.log;
     MssPump()        calls the started timers' callbacks once; the number of buffers loaded. */
#include <windows.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_SAMPLES 16
#define MAX_TIMERS 8

typedef void(__stdcall *timer_fn)(DWORD);

typedef struct {
    int used;
    int next; /* the buffer reported free: the other one than the last loaded */
} Sample;

static Sample samples[MAX_SAMPLES];
static FILE *pcm[MAX_SAMPLES];
static timer_fn timers[MAX_TIMERS];
static int started[MAX_TIMERS];
static DWORD loads;
static char record[MAX_PATH];
static FILE *log_file;

static void note(const char *format, ...)
{
    va_list args;
    if (!log_file)
        return;
    va_start(args, format);
    vfprintf(log_file, format, args);
    va_end(args);
    fputc('\n', log_file);
    fflush(log_file);
}

static int index_of(void *s)
{
    int i = (int)((Sample *)s - samples);
    return i >= 0 && i < MAX_SAMPLES ? i : -1;
}

void __stdcall MssRecord(const char *path)
{
    char name[MAX_PATH];
    snprintf(record, sizeof record, "%s", path);
    snprintf(name, sizeof name, "%s.log", path);
    log_file = fopen(name, "w");
}

int __stdcall MssPump(void)
{
    DWORD before = loads;
    int i;
    for (i = 0; i < MAX_TIMERS; i++)
        if (timers[i] && started[i])
            timers[i](0);
    return (int)(loads - before);
}

DWORD __stdcall AIL_minimum_sample_buffer_size(void *driver, long rate, long format)
{
    note("minimum %ld %ld", rate, format);
    return 4096;
}

void *__stdcall AIL_allocate_sample_handle(void *driver)
{
    int i;
    for (i = 0; i < MAX_SAMPLES; i++) {
        if (!samples[i].used) {
            samples[i].used = 1;
            samples[i].next = 0;
            note("allocate %d", i);
            return &samples[i];
        }
    }
    return NULL;
}

void __stdcall AIL_release_sample_handle(void *s)
{
    note("release %d", index_of(s));
    ((Sample *)s)->used = 0;
}

void __stdcall AIL_init_sample(void *s) { note("init %d", index_of(s)); }

void __stdcall AIL_set_sample_type(void *s, long format, DWORD flags)
{
    note("type %d %ld %lu", index_of(s), format, (unsigned long)flags);
}

void __stdcall AIL_set_sample_playback_rate(void *s, long rate)
{
    note("rate %d %ld", index_of(s), rate);
}

void __stdcall AIL_set_sample_user_data(void *s, DWORD index, long value)
{
    note("user %d %lu", index_of(s), (unsigned long)index);
}

void __stdcall AIL_set_sample_volume(void *s, long volume) { note("volume %d %ld", index_of(s), volume); }
void __stdcall AIL_set_sample_pan(void *s, long pan) { note("pan %d %ld", index_of(s), pan); }
void __stdcall AIL_serve(void) {}
long __stdcall AIL_sample_buffer_ready(void *s) { return ((Sample *)s)->next; }

void __stdcall AIL_load_sample_buffer(void *s, DWORD n, const void *buffer, DWORD length)
{
    int i = index_of(s);
    char name[MAX_PATH];
    note("load %d %lu %lu", i, (unsigned long)n, (unsigned long)length);
    if (i < 0)
        return;
    if (!pcm[i] && record[0]) {
        snprintf(name, sizeof name, "%s.%d.pcm", record, i);
        pcm[i] = fopen(name, "wb");
    }
    if (pcm[i] && length) {
        fwrite(buffer, 1, length, pcm[i]);
        fflush(pcm[i]);
    }
    loads++;
    samples[i].next = (int)(n ^ 1);
}

void __stdcall AIL_end_sample(void *s) { note("end %d", index_of(s)); }
DWORD __stdcall AIL_sample_status(void *s) { return 4; /* SMP_PLAYING */ }

long __stdcall AIL_sample_buffer_info(void *s, DWORD *pos0, DWORD *len0, DWORD *pos1, DWORD *len1)
{
    DWORD *out[4] = {pos0, len0, pos1, len1};
    int i;
    for (i = 0; i < 4; i++)
        if (out[i])
            *out[i] = 0;
    return 0;
}

void __stdcall AIL_lock(void) {}
void __stdcall AIL_unlock(void) {}

long __stdcall AIL_register_timer(timer_fn callback)
{
    int i;
    for (i = 0; i < MAX_TIMERS; i++) {
        if (!timers[i]) {
            timers[i] = callback;
            note("timer %d", i);
            return i;
        }
    }
    return -1;
}

void __stdcall AIL_set_timer_frequency(long timer, DWORD hertz)
{
    note("frequency %ld %lu", timer, (unsigned long)hertz);
}

void __stdcall AIL_start_timer(long timer)
{
    if (timer >= 0 && timer < MAX_TIMERS)
        started[timer] = 1;
}

void __stdcall AIL_release_timer_handle(long timer)
{
    if (timer >= 0 && timer < MAX_TIMERS) {
        timers[timer] = NULL;
        started[timer] = 0;
    }
}

void *__stdcall AIL_mem_alloc_lock(DWORD size) { return calloc(1, size); }
void __stdcall AIL_mem_free_lock(void *p) { free(p); }

long __stdcall AIL_get_preference(DWORD number)
{
    note("preference %lu", (unsigned long)number);
    return number == 15 ? 1 : 0;
}
