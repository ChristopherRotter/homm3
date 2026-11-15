/* vsound.h (E27 M2 T5b1, *provisional*; clients/capture/README.md, "The clips' sound clock"): a
   stand-in DirectSound whose buffers play on a given clock, for smackprobe.exe's `wait` (and
   xdd.dll's game runs, T5b2). Our own code, built with mingw-w64.

   SMACKW32.DLL plays a Smacker clip's sound through the DirectSound object Miles lends it
   (`SmackSoundUseMSS` 0x100079F0: Miles' preference 15 is 0, so `SmackSoundUseDirectSound` with
   the driver's +0xA0) and times the clip's frames by that buffer's play cursor (E27-design
   §11). Under Wine the cursor moves in real time; here it moves on the caller's clock (the
   milliseconds `vsound_now` returns): a playing buffer's cursor is where it was at its Play (or
   at the SetCurrentPosition after it) plus the clock's milliseconds since then times the
   format's bytes a second / 1000, rounded down to the format's block, modulo the buffer's size;
   the write cursor is the play cursor (an ideal device: no lead, no granularity). Every call
   that changes what plays is appended to the log (`vsound_log`, a line each: the clock, the
   call, its numbers), and every byte the cursor passes, as the buffer then held it, to the
   played file (`vsound_played`): the sound a listener hears, up to the clock's last reading
   (`vsound_flush`; with no played file, as in xdd.dll's runs, the cursor moves the same and
   nothing is written). Not modelled: the primary buffer (SMACKW32 sets its format only for a
   DirectSound it made itself), notifications, 3D, duplicates. */
#include <dsound.h>

static DWORD (*vsound_now)(void);
static FILE *vsound_log, *vsound_played;
/* T5b2: set, the log leaves out the lock and unlock lines (xdd.dll's game runs: a clip's refills,
   a few a second, compared nowhere). */
static int vsound_brief;

struct vbuffer {
    IDirectSoundBufferVtbl *lpVtbl;
    LONG refs;
    DWORD size, rate, align, flags;
    BYTE *data;
    int playing;
    DWORD since;   /* the clock at the Play, or at a SetCurrentPosition while playing */
    DWORD from;    /* the cursor then (while stopped: where it stands) */
    DWORD heard;   /* the cursor's place as last flushed into the played file */
    LONG volume, pan;
};

/* At most one buffer plays at a time in the start's clips: the played file is that one's. */
#define VBUFFERS 8
static struct vbuffer *vbuffers[VBUFFERS];

static DWORD vcursor(struct vbuffer *b)
{
    ULONGLONG moved;
    if (!b->playing)
        return b->from;
    moved = (ULONGLONG)(vsound_now() - b->since) * b->rate / 1000;
    moved -= moved % b->align;
    return (DWORD)((b->from + moved) % b->size);
}

/* The bytes the cursor passed since the last flush, as the buffer holds them (to the played file
   when there is one: T5b2's game runs keep none, the cursor moving all the same). */
static void vflush(struct vbuffer *b)
{
    DWORD at = vcursor(b);
    if (at == b->heard)
        return;
    if (!vsound_played) {
        /* nothing written */
    } else if (at > b->heard) {
        fwrite(b->data + b->heard, 1, at - b->heard, vsound_played);
    } else {
        fwrite(b->data + b->heard, 1, b->size - b->heard, vsound_played);
        fwrite(b->data, 1, at, vsound_played);
    }
    b->heard = at;
}

static inline void vsound_flush(void)
{
    int i;
    for (i = 0; i < VBUFFERS; i++)
        if (vbuffers[i])
            vflush(vbuffers[i]);
}

static void vnote(const char *what, long a, long b)
{
    if (vsound_brief && (!strcmp(what, "lock") || !strcmp(what, "unlock")))
        return;
    if (vsound_log)
        fprintf(vsound_log, "%lu %s %ld %ld\n", (unsigned long)vsound_now(), what, a, b);
}

static HRESULT WINAPI vb_query(IDirectSoundBuffer *self, REFIID iid, void **out)
{
    (void)self;
    (void)iid;
    *out = NULL;
    return E_NOINTERFACE;
}
static ULONG WINAPI vb_addref(IDirectSoundBuffer *self)
{
    return (ULONG)++((struct vbuffer *)self)->refs;
}
static ULONG WINAPI vb_release(IDirectSoundBuffer *self)
{
    struct vbuffer *b = (struct vbuffer *)self;
    int i;
    if (--b->refs > 0)
        return (ULONG)b->refs;
    vflush(b);
    vnote("release", 0, 0);
    for (i = 0; i < VBUFFERS; i++)
        if (vbuffers[i] == b)
            vbuffers[i] = NULL;
    free(b->data);
    free(b);
    return 0;
}
static HRESULT WINAPI vb_caps(IDirectSoundBuffer *self, LPDSBCAPS caps)
{
    struct vbuffer *b = (struct vbuffer *)self;
    caps->dwFlags = b->flags;
    caps->dwBufferBytes = b->size;
    return DS_OK;
}
static HRESULT WINAPI vb_position(IDirectSoundBuffer *self, LPDWORD play, LPDWORD write)
{
    struct vbuffer *b = (struct vbuffer *)self;
    vflush(b);
    if (play)
        *play = b->heard;
    if (write)
        *write = b->heard;
    return DS_OK;
}
static HRESULT WINAPI vb_format(IDirectSoundBuffer *self, LPWAVEFORMATEX f, DWORD n, LPDWORD w)
{
    (void)self;
    (void)f;
    (void)n;
    if (w)
        *w = 0;
    return DSERR_INVALIDCALL;
}
static HRESULT WINAPI vb_volume(IDirectSoundBuffer *self, LPLONG v)
{
    *v = ((struct vbuffer *)self)->volume;
    return DS_OK;
}
static HRESULT WINAPI vb_pan(IDirectSoundBuffer *self, LPLONG p)
{
    *p = ((struct vbuffer *)self)->pan;
    return DS_OK;
}
static HRESULT WINAPI vb_frequency(IDirectSoundBuffer *self, LPDWORD f)
{
    (void)self;
    *f = 0;
    return DS_OK;
}
static HRESULT WINAPI vb_status(IDirectSoundBuffer *self, LPDWORD s)
{
    *s = ((struct vbuffer *)self)->playing ? DSBSTATUS_PLAYING | DSBSTATUS_LOOPING : 0;
    return DS_OK;
}
static HRESULT WINAPI vb_init(IDirectSoundBuffer *self, LPDIRECTSOUND ds, LPCDSBUFFERDESC d)
{
    (void)self;
    (void)ds;
    (void)d;
    return DSERR_ALREADYINITIALIZED;
}
static HRESULT WINAPI vb_lock(IDirectSoundBuffer *self, DWORD at, DWORD n, LPVOID *p1,
                              LPDWORD n1, LPVOID *p2, LPDWORD n2, DWORD flags)
{
    struct vbuffer *b = (struct vbuffer *)self;
    vflush(b);
    if (flags || at >= b->size || n > b->size)
        return DSERR_INVALIDPARAM;
    *p1 = b->data + at;
    *n1 = at + n > b->size ? b->size - at : n;
    if (p2)
        *p2 = *n1 < n ? b->data : NULL;
    if (n2)
        *n2 = n - *n1;
    vnote("lock", (long)at, (long)n);
    return DS_OK;
}
static HRESULT WINAPI vb_play(IDirectSoundBuffer *self, DWORD r1, DWORD r2, DWORD flags)
{
    struct vbuffer *b = (struct vbuffer *)self;
    (void)r1;
    (void)r2;
    vflush(b);
    if (!b->playing) {
        b->playing = 1;
        b->since = vsound_now();
    }
    vnote("play", (long)b->from, (long)flags);
    return DS_OK;
}
static HRESULT WINAPI vb_set_position(IDirectSoundBuffer *self, DWORD at)
{
    struct vbuffer *b = (struct vbuffer *)self;
    vflush(b);
    b->from = b->heard = at % b->size;
    b->since = vsound_now();
    vnote("setpos", (long)at, 0);
    return DS_OK;
}
static HRESULT WINAPI vb_set_format(IDirectSoundBuffer *self, LPCWAVEFORMATEX f)
{
    (void)self;
    (void)f;
    return DSERR_INVALIDCALL;
}
static HRESULT WINAPI vb_set_volume(IDirectSoundBuffer *self, LONG v)
{
    ((struct vbuffer *)self)->volume = v;
    vnote("volume", v, 0);
    return DS_OK;
}
static HRESULT WINAPI vb_set_pan(IDirectSoundBuffer *self, LONG p)
{
    ((struct vbuffer *)self)->pan = p;
    vnote("pan", p, 0);
    return DS_OK;
}
static HRESULT WINAPI vb_set_frequency(IDirectSoundBuffer *self, DWORD f)
{
    (void)self;
    vnote("frequency", (long)f, 0);
    return DS_OK;
}
static HRESULT WINAPI vb_stop(IDirectSoundBuffer *self)
{
    struct vbuffer *b = (struct vbuffer *)self;
    vflush(b);
    b->from = b->heard;
    b->playing = 0;
    vnote("stop", (long)b->from, 0);
    return DS_OK;
}
static HRESULT WINAPI vb_unlock(IDirectSoundBuffer *self, LPVOID p1, DWORD n1, LPVOID p2,
                                DWORD n2)
{
    (void)self;
    (void)p1;
    (void)p2;
    vnote("unlock", (long)n1, (long)n2);
    return DS_OK;
}
static HRESULT WINAPI vb_restore(IDirectSoundBuffer *self)
{
    (void)self;
    return DS_OK;
}

static IDirectSoundBufferVtbl vbuffer_calls = {
    vb_query, vb_addref, vb_release, vb_caps, vb_position, vb_format, vb_volume,
    vb_pan, vb_frequency, vb_status, vb_init, vb_lock, vb_play, vb_set_position,
    vb_set_format, vb_set_volume, vb_set_pan, vb_set_frequency, vb_stop, vb_unlock,
    vb_restore,
};

/* The DirectSound object: only CreateSoundBuffer makes anything. */
static HRESULT WINAPI vs_query(IDirectSound *self, REFIID iid, void **out)
{
    (void)self;
    (void)iid;
    *out = NULL;
    return E_NOINTERFACE;
}
static ULONG WINAPI vs_addref(IDirectSound *self)
{
    (void)self;
    return 2;
}
static ULONG WINAPI vs_release(IDirectSound *self)
{
    (void)self;
    return 1;
}
static HRESULT WINAPI vs_create(IDirectSound *self, LPCDSBUFFERDESC d, LPDIRECTSOUNDBUFFER *out,
                                LPUNKNOWN outer)
{
    struct vbuffer *b;
    int i;
    (void)self;
    (void)outer;
    *out = NULL;
    if (!d || !d->lpwfxFormat || !d->dwBufferBytes || !d->lpwfxFormat->nBlockAlign)
        return DSERR_INVALIDPARAM;
    for (i = 0; i < VBUFFERS && vbuffers[i]; i++)
        ;
    if (i == VBUFFERS || !(b = calloc(1, sizeof *b)) || !(b->data = calloc(1, d->dwBufferBytes)))
        return DSERR_OUTOFMEMORY;
    b->lpVtbl = &vbuffer_calls;
    b->refs = 1;
    b->size = d->dwBufferBytes;
    b->rate = d->lpwfxFormat->nAvgBytesPerSec;
    b->align = d->lpwfxFormat->nBlockAlign;
    b->flags = d->dwFlags;
    vbuffers[i] = b;
    vnote("create", (long)b->size, (long)b->rate);
    vnote("flags", (long)b->flags, (long)b->align);
    *out = (LPDIRECTSOUNDBUFFER)b;
    return DS_OK;
}
static HRESULT WINAPI vs_caps(IDirectSound *self, LPDSCAPS c)
{
    (void)self;
    (void)c;
    return DSERR_UNSUPPORTED;
}
static HRESULT WINAPI vs_duplicate(IDirectSound *self, LPDIRECTSOUNDBUFFER a,
                                   LPLPDIRECTSOUNDBUFFER b)
{
    (void)self;
    (void)a;
    *b = NULL;
    return DSERR_UNSUPPORTED;
}
static HRESULT WINAPI vs_level(IDirectSound *self, HWND w, DWORD l)
{
    (void)self;
    (void)w;
    (void)l;
    return DS_OK;
}
static HRESULT WINAPI vs_compact(IDirectSound *self)
{
    (void)self;
    return DS_OK;
}
static HRESULT WINAPI vs_speakers(IDirectSound *self, LPDWORD s)
{
    (void)self;
    *s = DSSPEAKER_STEREO;
    return DS_OK;
}
static HRESULT WINAPI vs_set_speakers(IDirectSound *self, DWORD s)
{
    (void)self;
    (void)s;
    return DS_OK;
}
static HRESULT WINAPI vs_init(IDirectSound *self, LPCGUID g)
{
    (void)self;
    (void)g;
    return DSERR_ALREADYINITIALIZED;
}

static IDirectSoundVtbl vsound_calls = {
    vs_query, vs_addref, vs_release, vs_create, vs_caps, vs_duplicate, vs_level,
    vs_compact, vs_speakers, vs_set_speakers, vs_init,
};
static struct {
    IDirectSoundVtbl *lpVtbl;
} vsound = {&vsound_calls};
