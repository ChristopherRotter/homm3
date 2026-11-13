/* smackprobe.exe (E27 M2 T1; *provisional*; clients/capture/README.md, "The clips"): runs the
   game's own SMACKW32.DLL on its Smacker clips under Wine as a black box, as binkprobe.exe runs
   BINKW32.DLL (E17 M5 T4b2b). Built by setup.sh with mingw-w64; our own code. It calls only
   functions Heroes3.exe calls (SmackOpen, SmackToBuffer, SmackDoFrame, SmackNextFrame,
   SmackClose), with the arguments the executable passes (smacker_open 0x598790: a file handle at
   the entry's offset, the flags | 0x1140, the extra buffer -1; smacker_open_in_dialog 0x598AF0:
   SmackToBuffer at (0, 0) of a buffer, its pitch and height). `frames` opens no sound track (the
   flags' 0xFE000 clear) and gives the DLL no sound driver; `audio` (E27 M2 T2) gives it the
   stand-in Miles (mss32.c, loaded from mss\MSS32.DLL first) as the game gives its driver
   (sound_digital_start 0x5999E3: SmackSoundUseMSS) and opens every track (0xFE000, as
   smacker_open_in_dialog 0x598BAB does with the sound on).

     smackprobe frames <vid> <entry> <type> <out>
         Opens the archive's entry, SmackToBuffer into a buffer of the clip's size (one byte a
         pixel for type 0, two for the 16-bit types 80000000 and C0000000, hexadecimal: the
         flags the game passes, 5-5-5 or 5-6-5), then for every frame (the header's count, and
         the ring frame when the header has one: SmackNextFrame after the last decodes it)
         SmackDoFrame, then writes the buffer and the handle's palette (768 bytes from +0x6C),
         then SmackNextFrame. <out>.info gets one line per frame: "<frame> <new palette>" (the
         handle's +0x374 and +0x68).
     smackprobe audio <vid> <entry> <out>
         SmackSoundUseMSS with the stand-in's driver, SmackOpen with the tracks 0xFE000,
         SmackVolumePan as the game sets it (0x598CBE: the tracks 0xFE000, the volume 3640 times
         the effects' setting, here 9; the pan 0x8000), SmackToBuffer (8 bits), then per frame
         SmackDoFrame, the stand-in's timers called until they load nothing (at most 64 times),
         SmackNextFrame; after the last frame the timers 256 times more (the ring's tail), then
         SmackClose. The stand-in writes each sample handle's loads to <out>.<n>.pcm (the
         track's samples as the DLL decoded them: Miles' volume and pan act on playback, not on
         them) and every call to <out>.log.

   The x87 control word: main sets the game's 0x27F (Heroes3.exe's CRT; mingw-w64 starts at
   0x37F), as binkprobe does, so that any floating point in the DLL rounds as in the game. */
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void *(WINAPI *open_fn)(HANDLE, DWORD, DWORD);
typedef DWORD(WINAPI *one_fn)(void *);
typedef void(WINAPI *to_buffer_fn)(void *, DWORD, DWORD, DWORD, DWORD, void *, DWORD);

static open_fn smack_open;
static one_fn smack_do_frame, smack_next_frame, smack_close;
static to_buffer_fn smack_to_buffer;
typedef BYTE(WINAPI *use_mss_fn)(void *);
typedef void(WINAPI *volume_fn)(void *, DWORD, DWORD, DWORD);
typedef void(WINAPI *record_fn)(const char *);
typedef int(WINAPI *pump_fn)(void);
static use_mss_fn smack_use_mss;
static volume_fn smack_volume_pan;
static record_fn mss_record;
static pump_fn mss_pump;

static int load(void)
{
    /* The stand-in first: SMACKW32.DLL's LoadLibraryA("MSS32.DLL") then finds it loaded. */
    HMODULE mss = LoadLibraryA("mss\\MSS32.DLL"), dll;
    if (mss) {
        mss_record = (record_fn)GetProcAddress(mss, "MssRecord");
        mss_pump = (pump_fn)GetProcAddress(mss, "MssPump");
    }
    dll = LoadLibraryA("SMACKW32.DLL");
    if (!dll) {
        fprintf(stderr, "smackprobe: SMACKW32.DLL not found\n");
        return 0;
    }
    smack_open = (open_fn)GetProcAddress(dll, "_SmackOpen@12");
    smack_do_frame = (one_fn)GetProcAddress(dll, "_SmackDoFrame@4");
    smack_next_frame = (one_fn)GetProcAddress(dll, "_SmackNextFrame@4");
    smack_close = (one_fn)GetProcAddress(dll, "_SmackClose@4");
    smack_to_buffer = (to_buffer_fn)GetProcAddress(dll, "_SmackToBuffer@28");
    smack_use_mss = (use_mss_fn)GetProcAddress(dll, "_SmackSoundUseMSS@4");
    smack_volume_pan = (volume_fn)GetProcAddress(dll, "_SmackVolumePan@16");
    if (!smack_open || !smack_do_frame || !smack_next_frame || !smack_close ||
        !smack_to_buffer || !smack_use_mss || !smack_volume_pan) {
        fprintf(stderr, "smackprobe: an export is missing\n");
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

/* SmackOpen on the archive's handle at the entry's offset, as smacker_open 0x598790 does (the
   name compared ignoring case, with or without its ".smk"). */
static void *open_entry(const char *name, DWORD tracks)
{
    char want[48];
    DWORD i;
    snprintf(want, sizeof want, "%s.smk", name);
    for (i = 0; i < count; i++) {
        if (lstrcmpiA(names[i], want) && lstrcmpiA(names[i], name))
            continue;
        SetFilePointer(archive, (LONG)offsets[i], NULL, FILE_BEGIN);
        return smack_open(archive, tracks | 0x1140, 0xFFFFFFFF);
    }
    fprintf(stderr, "smackprobe: no entry %s\n", name);
    return NULL;
}

#define FIELD(h, off) (*(DWORD *)((BYTE *)(h) + (off)))

static int frames(const char *vid, const char *entry, DWORD type, const char *out)
{
    void *h;
    DWORD w, ht, n, i, pitch;
    BYTE *buf;
    FILE *f, *info;
    char path[MAX_PATH];
    if (!open_archive(vid) || !(h = open_entry(entry, 0))) {
        fprintf(stderr, "smackprobe: %s not opened\n", entry);
        return 1;
    }
    w = FIELD(h, 4);
    ht = FIELD(h, 8);
    n = FIELD(h, 0xC) + (FIELD(h, 0x14) & 1);
    pitch = type ? 2 * w : w;
    buf = calloc(pitch, ht);
    snprintf(path, sizeof path, "%s.info", out);
    f = fopen(out, "wb");
    info = fopen(path, "w");
    if (!buf || !f || !info)
        return 1;
    smack_to_buffer(h, 0, 0, pitch, ht, buf, type);
    for (i = 0; i < n; i++) {
        smack_do_frame(h);
        fwrite(buf, 1, pitch * ht, f);
        fwrite((BYTE *)h + 0x6C, 1, 768, f);
        fprintf(info, "%lu %lu\n", (unsigned long)FIELD(h, 0x374), (unsigned long)FIELD(h, 0x68));
        smack_next_frame(h);
    }
    fclose(f);
    fclose(info);
    printf("%s: %lu x %lu, %lu frames\n", entry, (unsigned long)w, (unsigned long)ht,
           (unsigned long)n);
    smack_close(h);
    return 0;
}

/* The driver SmackSoundUseMSS is given: the stand-in never reads it, and the DLL reads only its
   +0xA0 (Miles' DirectSound object), on the DirectSound path the stand-in's preference avoids. */
static BYTE driver[0x400];

static int audio(const char *vid, const char *entry, const char *out)
{
    void *h;
    DWORD w, ht, n, i;
    BYTE *buf;
    int k;
    if (!mss_record || !mss_pump) {
        fprintf(stderr, "smackprobe: no stand-in mss\\MSS32.DLL (setup.sh builds it)\n");
        return 1;
    }
    mss_record(out);
    if (!smack_use_mss(driver)) {
        fprintf(stderr, "smackprobe: SmackSoundUseMSS refused the stand-in\n");
        return 1;
    }
    if (!open_archive(vid) || !(h = open_entry(entry, 0xFE000))) {
        fprintf(stderr, "smackprobe: %s not opened\n", entry);
        return 1;
    }
    smack_volume_pan(h, 0xFE000, 3640 * 9, 0x8000);
    w = FIELD(h, 4);
    ht = FIELD(h, 8);
    n = FIELD(h, 0xC);
    buf = calloc(w, ht);
    if (!buf)
        return 1;
    smack_to_buffer(h, 0, 0, w, ht, buf, 0);
    for (i = 0; i < n; i++) {
        smack_do_frame(h);
        for (k = 0; k < 64 && mss_pump() > 0; k++)
            ;
        if (i + 1 < n)
            smack_next_frame(h);
    }
    for (k = 0; k < 256; k++)
        mss_pump();
    printf("%s: %lu frames\n", entry, (unsigned long)n);
    smack_close(h);
    return 0;
}

int main(int argc, char **argv)
{
    unsigned short cw = 0x27F;
    __asm__ volatile("fldcw %0" : : "m"(cw));
    if (!load())
        return 1;
    if (argc == 6 && !strcmp(argv[1], "frames"))
        return frames(argv[2], argv[3], strtoul(argv[4], NULL, 16), argv[5]);
    if (argc == 5 && !strcmp(argv[1], "audio"))
        return audio(argv[2], argv[3], argv[4]);
    fprintf(stderr, "usage: smackprobe frames <vid> <entry> <type> <out>\n"
                    "       smackprobe audio <vid> <entry> <out>\n");
    return 2;
}
