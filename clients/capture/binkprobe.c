/* binkprobe.exe (E17 M5 T4b2b; *provisional*; clients/capture/README.md, "The clips"): runs the
   game's own BINKW32.DLL on its clips under Wine as a black box (the user, 2026-10-28: the
   colours and the timing found by running the DLL, not by reading it). Built by setup.sh with
   mingw-w64; our own code. It calls only the functions Heroes3.exe calls (BinkOpen,
   BinkDoFrame, BinkCopyToBuffer, BinkNextFrame, BinkWait, BinkGetRects, BinkClose), with the
   arguments the executable passes (`video_open` 0x44D5A0, `video_tick` 0x44DAA0), and patches
   only the DLL's own timeGetTime import slot (as xdd.dll patches the executable's), so that its
   clock is ours.

     binkprobe frames <vid> <entry> <type> <out>
         Opens the archive's entry as the game does (a file handle at the entry's offset, flags
         0x8000000), then for every frame BinkDoFrame, BinkCopyToBuffer into a 16-bit buffer of
         the clip's size (pitch twice its width, surface type <type>) and BinkGetRects; <out>
         gets each frame's pixels (little-endian words, row by row) and <out>.rects one line per
         frame: "<frame> <count> x y w h ...".
     binkprobe play <vid> <start> <loop|-> <type> <ms> <out> [<first> [<redraws>]]
         The result window's playback (0x44D830 and 0x44DAA0): the loop opened first (flags
         0x400000), then the start, at the clock's 0; at <first> (default 0) <redraws> times
         (default 0) the dialog's redraw of the clip (0x44D9E0, from dialog_show 0x5FEAE0:
         BinkDoFrame while the clip is at its first frame, then BinkCopyToBuffer); then each
         millisecond from <first> to <ms> the game's tick, as often as the DLL lets it draw (up
         to 64 times a millisecond); the loop taking over when the start's last frame is
         drawn. <out> gets one line per drawn frame:
         "<ms> <clip> <frame> <count> x y w h ..." (clip 0 the start, 1 the loop), the rects
         BinkGetRects gives on the handle the game asks after the frame; <out>.screen the
         buffer as the screen would show it (the frame's rects copied) after each drawn frame.

   The surface type is the value BinkDDSurfaceType gives the game's primary surface (read from
   the running game's 0x694CA0, the runbook). */
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void *(WINAPI *open_fn)(HANDLE, DWORD);
typedef int(WINAPI *one_fn)(void *);
typedef void(WINAPI *next_fn)(void *);
typedef int(WINAPI *copy_fn)(void *, void *, int, int, int, int, DWORD);
typedef int(WINAPI *rects_fn)(void *, DWORD);
typedef void(WINAPI *close_fn)(void *);

static open_fn bink_open;
static one_fn bink_do_frame, bink_wait;
static next_fn bink_next_frame;
static copy_fn bink_copy;
static rects_fn bink_rects;
static close_fn bink_close;

/* The clock the DLL reads. */
static volatile DWORD clock_now;
static DWORD WINAPI clock_read(void)
{
    return clock_now;
}

/* Points BINKW32.DLL's timeGetTime import slot at clock_read. */
static int hook_clock(HMODULE dll)
{
    BYTE *base = (BYTE *)dll;
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
            *slot = (DWORD)clock_read;
            VirtualProtect(slot, sizeof *slot, old, &old);
            return 1;
        }
    }
    return 0;
}

static int load(void)
{
    HMODULE dll;
    LoadLibraryA("winmm.dll");
    dll = LoadLibraryA("BINKW32.DLL");
    if (!dll) {
        fprintf(stderr, "binkprobe: BINKW32.DLL not found\n");
        return 0;
    }
    bink_open = (open_fn)GetProcAddress(dll, "_BinkOpen@8");
    bink_do_frame = (one_fn)GetProcAddress(dll, "_BinkDoFrame@4");
    bink_wait = (one_fn)GetProcAddress(dll, "_BinkWait@4");
    bink_next_frame = (next_fn)GetProcAddress(dll, "_BinkNextFrame@4");
    bink_copy = (copy_fn)GetProcAddress(dll, "_BinkCopyToBuffer@28");
    bink_rects = (rects_fn)GetProcAddress(dll, "_BinkGetRects@8");
    bink_close = (close_fn)GetProcAddress(dll, "_BinkClose@4");
    if (!bink_open || !bink_do_frame || !bink_wait || !bink_next_frame || !bink_copy ||
        !bink_rects || !bink_close) {
        fprintf(stderr, "binkprobe: an export is missing\n");
        return 0;
    }
    if (!hook_clock(dll)) {
        fprintf(stderr, "binkprobe: the DLL's clock import not found\n");
        return 0;
    }
    return 1;
}

/* A VIDEO.VID: a count, then 44-byte entries (a 40-byte name, an offset). */
static HANDLE archive;
static DWORD *offsets;
static char (*names)[40];
static DWORD count;

static int open_archive(const char *path)
{
    DWORD got, i;
    archive = CreateFileA(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING,
                          FILE_ATTRIBUTE_NORMAL | FILE_FLAG_RANDOM_ACCESS, NULL);
    if (archive == INVALID_HANDLE_VALUE || !ReadFile(archive, &count, 4, &got, NULL) || got != 4)
        return 0;
    offsets = calloc(count, sizeof *offsets);
    names = calloc(count, sizeof *names);
    for (i = 0; i < count; i++) {
        if (!ReadFile(archive, names[i], 40, &got, NULL) || got != 40 ||
            !ReadFile(archive, &offsets[i], 4, &got, NULL) || got != 4)
            return 0;
    }
    return 1;
}

/* BinkOpen on the archive's handle at the entry's offset, as video_open 0x44D5A0 does. */
static void *open_entry(const char *name, DWORD flags)
{
    char want[48];
    DWORD i;
    snprintf(want, sizeof want, "%s.bik", name);
    for (i = 0; i < count; i++) {
        if (lstrcmpiA(names[i], want) && lstrcmpiA(names[i], name))
            continue;
        SetFilePointer(archive, (LONG)offsets[i], NULL, FILE_BEGIN);
        return bink_open(archive, flags | 0x8000000);
    }
    fprintf(stderr, "binkprobe: no entry %s\n", name);
    return NULL;
}

#define FIELD(h, off) (*(DWORD *)((BYTE *)(h) + (off)))

static void write_rects(FILE *f, void *h, int n)
{
    int i;
    fprintf(f, " %d", n);
    for (i = 0; i < n && i < 8; i++)
        fprintf(f, " %lu %lu %lu %lu", (unsigned long)FIELD(h, 0x30 + 16 * i),
                (unsigned long)FIELD(h, 0x34 + 16 * i), (unsigned long)FIELD(h, 0x38 + 16 * i),
                (unsigned long)FIELD(h, 0x3C + 16 * i));
    fprintf(f, "\n");
}

static int frames(const char *vid, const char *entry, DWORD type, const char *out)
{
    void *h;
    DWORD w, ht, n, i;
    WORD *buf;
    FILE *f, *r;
    char path[MAX_PATH];
    if (!open_archive(vid) || !(h = open_entry(entry, 0)))
        return 1;
    w = FIELD(h, 0);
    ht = FIELD(h, 4);
    n = FIELD(h, 8);
    buf = calloc(w * ht, 2);
    snprintf(path, sizeof path, "%s.rects", out);
    f = fopen(out, "wb");
    r = fopen(path, "w");
    if (!f || !r)
        return 1;
    for (i = 1; i <= n; i++) {
        bink_do_frame(h);
        bink_copy(h, buf, (int)(w * 2), (int)ht, 0, 0, type);
        fprintf(r, "%lu", (unsigned long)FIELD(h, 0xC));
        write_rects(r, h, bink_rects(h, type));
        fwrite(buf, 2, w * ht, f);
        if (i < n)
            bink_next_frame(h);
    }
    fclose(f);
    fclose(r);
    printf("%s: %lu x %lu, %lu frames\n", entry, (unsigned long)w, (unsigned long)ht,
           (unsigned long)n);
    bink_close(h);
    return 0;
}

static int play(const char *vid, const char *start_name, const char *loop_name, DWORD type,
                DWORD ms, const char *out, DWORD first, DWORD redraws)
{
    void *start = NULL, *loop = NULL;
    DWORD w, ht, t;
    WORD *buf, *screen;
    FILE *f, *s;
    char path[MAX_PATH];
    if (!open_archive(vid))
        return 1;
    clock_now = 0;
    if (strcmp(loop_name, "-") && !(loop = open_entry(loop_name, 0x400000)))
        return 1;
    if (!(start = open_entry(start_name, 0)))
        return 1;
    w = FIELD(start, 0);
    ht = FIELD(start, 4);
    buf = calloc(w * ht, 2);
    screen = calloc(w * ht, 2);
    snprintf(path, sizeof path, "%s.screen", out);
    f = fopen(out, "w");
    s = fopen(path, "wb");
    if (!f || !s)
        return 1;
    clock_now = first;
    for (t = 0; t < redraws; t++) {
        if (FIELD(start, 0xC) == 1)
            bink_do_frame(start);
        bink_copy(start, buf, (int)(w * 2), (int)ht, 0, 0, type);
    }
    for (t = first; t <= ms; t++) {
        int k;
        clock_now = t;
        for (k = 0; k < 64; k++) {
            void *h = start ? start : loop;
            int clip = start ? 0 : 1, n, i;
            DWORD frame;
            if (!h || bink_wait(h))
                break;
            bink_do_frame(h);
            bink_copy(h, buf, (int)(w * 2), (int)ht, 0, 0, type);
            frame = FIELD(h, 0xC);
            if (FIELD(h, 0xC) == FIELD(h, 8) && start && loop) {
                /* the start's last frame: the loop takes over (entry flags +9, +0xA are 0) */
                bink_close(start);
                start = NULL;
                h = loop;
            } else if (FIELD(h, 0xC) == FIELD(h, 8) && !loop) {
                /* a clip without a loop plays on (0x694CD8 is 1): BinkNextFrame wraps */
                bink_next_frame(h);
            } else {
                bink_next_frame(h);
            }
            /* video_show 0x5979D0: the rects of the handle playing now (the loop's once it took
               over) after the frame */
            n = bink_rects(h, type);
            fprintf(f, "%lu %d %lu", (unsigned long)t, clip, (unsigned long)frame);
            write_rects(f, h, n);
            for (i = 0; i < n && i < 8; i++) {
                DWORD x = FIELD(h, 0x30 + 16 * i), y = FIELD(h, 0x34 + 16 * i);
                DWORD rw = FIELD(h, 0x38 + 16 * i), rh = FIELD(h, 0x3C + 16 * i), yy;
                for (yy = y; yy < y + rh && yy < ht; yy++)
                    memcpy(screen + yy * w + x, buf + yy * w + x,
                           2 * (x + rw <= w ? rw : w - x));
            }
            fwrite(screen, 2, w * ht, s);
        }
    }
    fclose(f);
    fclose(s);
    return 0;
}

int main(int argc, char **argv)
{
    if (!load())
        return 2;
    if (argc == 6 && !strcmp(argv[1], "frames"))
        return frames(argv[2], argv[3], strtoul(argv[4], NULL, 0), argv[5]);
    if (argc >= 8 && argc <= 10 && !strcmp(argv[1], "play"))
        return play(argv[2], argv[3], argv[4], strtoul(argv[5], NULL, 0),
                    strtoul(argv[6], NULL, 0), argv[7], argc >= 9 ? strtoul(argv[8], NULL, 0) : 0,
                    argc == 10 ? strtoul(argv[9], NULL, 0) : 0);
    fprintf(stderr, "usage: binkprobe frames <vid> <entry> <type> <out>\n"
                    "       binkprobe play <vid> <start> <loop|-> <type> <ms> <out> [<first> [<redraws>]]\n");
    return 2;
}
