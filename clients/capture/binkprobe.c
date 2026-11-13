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

     binkprobe tables <vid> <entry> <out>
         Opens the entry (BinkOpen builds the DLL's tables at its first call, 0x10001E70) and
         writes the dequantization tables the inverse DCT multiplies by (E17 M6 T6c1): the
         intra blocks' (0x1001B2F8) and the inter blocks' (0x1001C2F8), each 16 quantizer
         indices of 64 binary32 values in natural order, as little-endian bits (8 KiB).

     binkprobe sound <vid> <entry> <out>
         (E27 M2 T2) The clip's sound as the DLL decodes it: the archive's entry read whole, its
         first audio track opened by BinkAudioDecompressOpen with the arguments BinkOpen passes
         (0x10005195: the track's rate, its channels, 1), then for each frame's packet of that
         track BinkAudioDecompress as BinkDoFrame calls it (0x10005A90): from the packet's
         second word (its first is the decoded size), until the decoded size is given, a block's
         samples past it dropped; <out> gets the samples (16-bit little-endian, the channels
         interleaved). Playback's ring (it drops what does not fit) and an 8-bit track's
         conversion (0x10005430) come after this.
     binkprobe sound-tables <out>
         The sound decoder's tables (E27 M2 T2), by the DLL's own functions: the 256 band scales
         (0x1000F310 on (float)(v * 0.664f), v = 0 to 255, its result stored as binary32, as the
         block decoder 0x1000F62D does), then the twiddles Ooura's cdft (0x1000C610) builds for
         1024, 2048 and 4096 values (makewt 0x1000C6A0: 256, 512 and 1024 binary32), each by a
         cdft on zeros with a fresh table; little-endian bits.
     binkprobe cdft <n> <in> <out>
         The DLL's cdft (0x1000C610) with the sign -1, as the block decoder calls it (0x1000F874),
         on each run of <n> binary32 values of <in> in turn (one table for all, as a decoder
         keeps it); <out> gets the results.

   The surface type is the value BinkDDSurfaceType gives the game's primary surface (read from
   the running game's 0x694CA0, the runbook).

   The x87 control word (E17 M6 T6c1): the DLL's inverse DCT keeps intermediates on the x87
   stack, so its pixels depend on the precision control. The game runs at its CRT's 0x27F
   (53-bit precision, round to nearest: Heroes3.exe's math helpers restore 0x27F, 0x61E646);
   mingw-w64's startup leaves 0x37F (64-bit), so main sets 0x27F before anything else, or the
   hexadecimal value of BINKPROBE_CW when set (to compare). */
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

static HMODULE dll;

static int load(void)
{
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

/* The DLL's dequantization tables, built by BinkOpen (0x10001E70; relative to the DLL's base). */
static int tables(const char *vid, const char *entry, const char *out)
{
    void *h;
    FILE *f;
    if (!open_archive(vid) || !(h = open_entry(entry, 0)))
        return 1;
    if (!(f = fopen(out, "wb")))
        return 1;
    fwrite((BYTE *)dll + 0x1B2F8, 4, 16 * 64, f);
    fwrite((BYTE *)dll + 0x1C2F8, 4, 16 * 64, f);
    fclose(f);
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

/* The entry's bytes, read whole (its size up to the next entry's offset or the archive's end). */
static BYTE *read_entry(const char *name, DWORD *size)
{
    char want[48];
    DWORD i, end, got;
    BYTE *bytes;
    snprintf(want, sizeof want, "%s.bik", name);
    for (i = 0; i < count; i++) {
        if (lstrcmpiA(names[i], want) && lstrcmpiA(names[i], name))
            continue;
        end = i + 1 < count ? offsets[i + 1] : GetFileSize(archive, NULL);
        *size = end - offsets[i];
        bytes = malloc(*size + 8);
        SetFilePointer(archive, (LONG)offsets[i], NULL, FILE_BEGIN);
        if (!bytes || !ReadFile(archive, bytes, *size, &got, NULL) || got != *size)
            return NULL;
        return bytes;
    }
    fprintf(stderr, "binkprobe: no entry %s\n", name);
    return NULL;
}

#define WORD32(p) (*(DWORD *)(p))

typedef void *(WINAPI *sound_open_fn)(DWORD, DWORD, DWORD);
typedef void(WINAPI *sound_fn)(void *, void **, DWORD *, void *, void **);
typedef float(__cdecl *scale_fn)(float);
typedef void(__cdecl *cdft_fn)(int, int, float *, int *, float *);

static int sound(const char *vid, const char *entry, const char *out)
{
    sound_open_fn sound_open = (sound_open_fn)GetProcAddress(dll, "_BinkAudioDecompressOpen@12");
    sound_fn decompress = (sound_fn)GetProcAddress(dll, "_BinkAudioDecompress@20");
    BYTE *clip;
    DWORD size, frames, tracks, word, f, t;
    void *h;
    FILE *o;
    if (!sound_open || !decompress || !open_archive(vid) || !(clip = read_entry(entry, &size)))
        return 1;
    if (!(o = fopen(out, "wb")))
        return 1;
    frames = WORD32(clip + 8);
    tracks = WORD32(clip + 0x28);
    if (tracks == 0) {
        fclose(o);
        printf("%s: no sound\n", entry);
        return 0;
    }
    /* the track's rate and flags: the header, its largest decoded sizes, then this word each */
    word = WORD32(clip + 0x2C + 4 * tracks);
    h = sound_open(word & 0xFFFF, ((word >> 29) & 1) + 1, 1);
    if (!h)
        return 1;
    for (f = 0; f < frames; f++) {
        DWORD at = WORD32(clip + 0x2C + 12 * tracks + 4 * f) & ~1u;
        for (t = 0; t < tracks; t++) {
            DWORD bytes = at + 4 <= size ? WORD32(clip + at) : 0, left;
            BYTE *in;
            at += 4;
            if (t == 0 && bytes >= 4 && at + bytes <= size) {
                left = WORD32(clip + at);
                in = clip + at + 4;
                while (left) {
                    void *samples;
                    DWORD got;
                    decompress(h, &samples, &got, in, (void **)&in);
                    if (got > left)
                        got = left;
                    fwrite(samples, 1, got, o);
                    left -= got;
                }
            }
            at += bytes;
        }
    }
    fclose(o);
    printf("%s: %lu frames, %lu Hz, flags %lx\n", entry, (unsigned long)frames,
           (unsigned long)(word & 0xFFFF), (unsigned long)(word >> 16));
    return 0;
}

/* The decoder's tables by its own functions (relative to the DLL's base 0x10000000). */
static int sound_tables(const char *out)
{
    scale_fn scale = (scale_fn)((BYTE *)dll + 0xF310);
    cdft_fn cdft = (cdft_fn)((BYTE *)dll + 0xC610);
    const DWORD factor = 0x3F29FBE7; /* 0x100183AC: 0.664f */
    FILE *o = fopen(out, "wb");
    int v, n;
    if (!o)
        return 1;
    for (v = 0; v < 256; v++) {
        float c;
        volatile float x, r;
        memcpy(&c, &factor, 4);
        /* fild, fmul by the binary32 factor, fstp: the product of an 8-bit integer and a
           24-bit significand is exact in a double, so the cast is the DLL's one rounding */
        x = (float)((double)v * (double)c);
        r = scale(x);
        fwrite((const void *)&r, 4, 1, o);
    }
    for (n = 1024; n <= 4096; n *= 2) {
        float *a = calloc(n, sizeof *a), *w = calloc(n, sizeof *w);
        int *ip = calloc(2 + n, sizeof *ip);
        if (!a || !w || !ip)
            return 1;
        cdft(n, -1, a, ip, w);
        fwrite(w, 4, n / 4, o);
        free(a);
        free(w);
        free(ip);
    }
    fclose(o);
    return 0;
}

static int run_cdft(int n, const char *in, const char *out)
{
    cdft_fn cdft = (cdft_fn)((BYTE *)dll + 0xC610);
    float *a = calloc(n, sizeof *a), *w = calloc(n, sizeof *w);
    int *ip = calloc(2 + n, sizeof *ip);
    FILE *i = fopen(in, "rb"), *o = fopen(out, "wb");
    if (n < 4 || !a || !w || !ip || !i || !o)
        return 1;
    while (fread(a, 4, n, i) == (size_t)n) {
        cdft(n, -1, a, ip, w);
        fwrite(a, 4, n, o);
    }
    fclose(i);
    fclose(o);
    return 0;
}

int main(int argc, char **argv)
{
    const char *cw = getenv("BINKPROBE_CW");
    unsigned short word = cw ? (unsigned short)strtoul(cw, NULL, 16) : 0x27F;
    __asm__ volatile("fldcw %0" : : "m"(word));
    if (!load())
        return 2;
    if (argc == 6 && !strcmp(argv[1], "frames"))
        return frames(argv[2], argv[3], strtoul(argv[4], NULL, 0), argv[5]);
    if (argc >= 8 && argc <= 10 && !strcmp(argv[1], "play"))
        return play(argv[2], argv[3], argv[4], strtoul(argv[5], NULL, 0),
                    strtoul(argv[6], NULL, 0), argv[7], argc >= 9 ? strtoul(argv[8], NULL, 0) : 0,
                    argc == 10 ? strtoul(argv[9], NULL, 0) : 0);
    if (argc == 5 && !strcmp(argv[1], "tables"))
        return tables(argv[2], argv[3], argv[4]);
    if (argc == 5 && !strcmp(argv[1], "sound"))
        return sound(argv[2], argv[3], argv[4]);
    if (argc == 3 && !strcmp(argv[1], "sound-tables"))
        return sound_tables(argv[2]);
    if (argc == 5 && !strcmp(argv[1], "cdft"))
        return run_cdft((int)strtoul(argv[2], NULL, 0), argv[3], argv[4]);
    fprintf(stderr, "usage: binkprobe frames <vid> <entry> <type> <out>\n"
                    "       binkprobe tables <vid> <entry> <out>\n"
                    "       binkprobe play <vid> <start> <loop|-> <type> <ms> <out> [<first> [<redraws>]]\n"
                    "       binkprobe sound <vid> <entry> <out>\n"
                    "       binkprobe sound-tables <out>\n"
                    "       binkprobe cdft <n> <in> <out>\n");
    return 2;
}
