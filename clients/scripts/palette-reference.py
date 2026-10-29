#!/usr/bin/env python3
"""The battle stack's palette effects in IEEE doubles (E17 M5 T5b1; `docs/epics/E17-design.md`
§7.26): the reference `h3_render::recolour`'s tests check their digests against.

`draw_battle_stack` 0x43E280 recolours the creature's 16-bit palette, entries 10 to 255, for the
sprite alone: Bloodlust's glow and a clone's colours through `palette16_recolour` 0x522B50, Stone
Gaze's glow through `palette16_stone_glow` 0x522A10 (both RGB to HSV, 0x523190, and back,
0x5232F0), the petrified look through `palette16_petrified` 0x522D50 (integers alone). The x87
runs at 53 bits (`docs/formats/installation.md` §7.10), so each of its operations on these values
is an IEEE double's; each `fstp dword` is a conversion to float32 (ties to even).

Prints, for each effect, the FNV-1a digest of all 65 536 16-bit values recoloured as the
executable does (5-6-5, each value's little-endian bytes in order) and the results for white,
0x1234, 0x8410 and 0x07E0.

    python3 -I clients/scripts/palette-reference.py
"""

import numpy as np

MASKS = (0xF800, 0x07E0, 0x001F)  # red, green, blue (0x69CC64, 0x69CC60, 0x69CC68)
SCALES = tuple(0x7FFFFFFF // m for m in MASKS)
MAGIC = 6755399441055744.0  # 1.5 x 2^52, the integer rounding of 0x5232F0


def f32(x):
    """An `fstp dword`: the double rounded to float32, ties to even."""
    return np.asarray(x, dtype=np.float64).astype(np.float32).astype(np.float64)


def channels(values):
    """Each channel scaled to K x (value & mask), K = 0x7FFFFFFF / mask (unsigned)."""
    return [(values & m).astype(np.uint64) * k for m, k in zip(MASKS, SCALES)]


def pack(rgb):
    """Each channel divided by its K and kept to its mask, or'd together."""
    out = np.zeros(rgb[0].shape, dtype=np.uint64)
    for c, m, k in zip(rgb, MASKS, SCALES):
        out |= (c.astype(np.uint64) // k) & m
    return out.astype(np.uint16)


def rgb_to_hsv(a, b, c):
    """0x523190: a, b, c the scaled red, green, blue (u32)."""
    mx = np.maximum(np.maximum(a, b), c)
    mn = np.minimum(np.minimum(a, b), c)
    vf = f32(mx.astype(np.float64))
    v = f32(vf / 2147483648.0)
    d = f32((mx - mn).astype(np.float64))
    with np.errstate(divide="ignore", invalid="ignore"):
        s = np.where(mx == 0, 0.0, f32(d / vf))
        ra = f32(f32((mx - a).astype(np.float64)) / d)
        rb = f32(f32((mx - b).astype(np.float64)) / d)
        rc = f32(f32((mx - c).astype(np.float64)) / d)
    third, two_thirds = float(np.float32(1 / 3)), float(np.float32(2 / 3))
    h = np.where(
        a == mx,
        f32((rc - rb) / 6.0 + 0.0),
        np.where(b == mx, f32((ra - rc) / 6.0 + third), f32((rb - ra) / 6.0 + two_thirds)),
    )
    h = np.where(h < 0.0, f32(h + 1.0), h)
    h = np.where(mx == mn, 0.0, h)
    return h, s, v


def round_even(x):
    """x + 1.5 x 2^52 at 53 bits, its low word."""
    return ((x + MAGIC) - MAGIC).astype(np.int64).astype(np.uint64) & 0xFFFFFFFF


def hsv_to_rgb(h, s, v, a, b, c):
    """0x5232F0; a sextant above 5 leaves the outputs (a, b, c) as they were."""
    grey = round_even(v * 2147483648.0)
    big_h = f32(h * 6.0)
    f = f32(np.fmod(big_h, 1.0))
    vv = f32(v * 2147483648.0)
    p = round_even(f32((1.0 - s) * vv))
    q = round_even(f32((1.0 - f * s) * vv))
    t = round_even(f32((1.0 - (1.0 - f) * s) * vv))
    vr = round_even(vv)
    i = np.trunc(big_h).astype(np.int64)
    table = {
        0: (vr, t, p),
        1: (q, vr, p),
        2: (p, vr, t),
        3: (p, q, vr),
        4: (t, p, vr),
        5: (vr, p, q),
    }
    out = [a.copy(), b.copy(), c.copy()]
    for n, chans in table.items():
        sel = (i == n) & (s != 0.0)
        for o, ch in zip(out, chans):
            o[sel] = ch[sel]
    zero = s == 0.0
    for o in out:
        o[zero] = grey[zero]
    return out


def scale_toward_one(x, k):
    """A factor above 1 moves x toward 1 (1 - (1 - x) / k), else multiplies it."""
    return f32(1.0 - (1.0 - x) / k) if k > 1.0 else f32(x * k)


def recolour(values, hue, weight, saturation, value):
    """0x522B50 (the arguments float32 values)."""
    a, b, c = channels(values)
    h, s, v = rgb_to_hsv(a, b, c)
    if weight >= 0.0:
        d = f32(hue - h)
        h = f32(d * weight + h)
        far = np.abs(d) > 0.5
        wrapped = np.where(d > 0.0, f32((1.0 - weight) + h), f32(h + weight))
        wrapped = np.where(wrapped >= 1.0, f32(wrapped - 1.0), wrapped)
        h = np.where(far, wrapped, h)
    if saturation >= 0.0:
        s = scale_toward_one(s, saturation)
    if value >= 0.0:
        v = scale_toward_one(v, value)
    return pack(hsv_to_rgb(h, s, v, a, b, c))


def stone_glow(values, glow):
    """0x522A10: the saturation alone."""
    a, b, c = channels(values)
    h, s, v = rgb_to_hsv(a, b, c)
    s = scale_toward_one(s, glow)
    return pack(hsv_to_rgb(h, s, v, a, b, c))


def petrified(values):
    """0x522D50: every channel the largest scaled one."""
    a, b, c = channels(values)
    mx = np.maximum(np.maximum(a, b), c)
    return pack([mx, mx, mx])


def glow(step):
    """f32(step x 0.1) at 53 bits (0x5A12B9)."""
    return float(f32(step * 0.1))


def fnv(values):
    h = 0xCBF29CE484222325
    for byte in values.astype("<u2").tobytes():
        h ^= byte
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def main():
    every = np.arange(65536, dtype=np.uint64)
    samples = [0xFFFF, 0x1234, 0x8410, 0x07E0]
    effects = [("clone", lambda x: recolour(x, float(np.float32(0.67)), 1.0, 2.0, 2.0))]
    for step in range(11):
        g = glow(step)
        one = float(f32(g + 1.0))
        effects.append((f"bloodlust {step}", lambda x, g=g, one=one: recolour(x, 0.0, g, one, one)))
    for step in range(11):
        g = glow(step)
        effects.append((f"stone {step}", lambda x, g=g: stone_glow(x, g)))
    effects.append(("petrified", petrified))
    for name, effect in effects:
        out = effect(every)
        picked = ", ".join(f"0x{int(out[s]):04X}" for s in samples)
        print(f"{name}: 0x{fnv(out):016x} [{picked}]")


if __name__ == "__main__":
    main()
