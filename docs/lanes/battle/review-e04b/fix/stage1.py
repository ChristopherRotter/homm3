#!/usr/bin/env python3
"""Exact rational model of the original's caster stage (0x4E5760) and of the engine's old formula.

A store to f32 rounds to 24 significant bits (ties to even); every x87 operation rounds to 53.
"""
from fractions import Fraction as F
import struct, sys

def rnd(x: F, bits: int) -> F:
    if x == 0:
        return F(0)
    assert x > 0
    # exponent e with 2^e <= x < 2^(e+1)
    n, d = x.numerator, x.denominator
    e = n.bit_length() - d.bit_length()
    if F(2) ** e > x:
        e -= 1
    scale = F(2) ** (bits - 1 - e)
    m = x * scale
    q, r = divmod(m.numerator, m.denominator)
    twice = 2 * r
    if twice > m.denominator or (twice == m.denominator and q & 1):
        q += 1
    return F(q) / scale

def f32(x): return rnd(F(x), 24)
def r53(x): return rnd(F(x), 53)
def bits_to_f(bits): return F(struct.unpack('>f', struct.pack('>I', bits))[0])
def f32_of_bp(bp): return f32(F(bp, 10000))

def new_stage1(value, school, allr, bonus=lambda t: 0):
    """school, allr: the rates as exact Fractions of their f32 values."""
    v = f32(value)
    v = f32(r53(v * r53(1 + school)))
    v = f32(r53(r53(1 + allr) * v))
    t = int(v)  # trunc, v >= 0
    b = bonus(t)
    if b >= 0:
        w = f32(r53(v + f32(b)))
    else:
        w = f32(r53(max(v - f32(-b), 0)))
    return t, int(w)

def old_stage1(value, school, allr):
    one = 1 << 40
    s = int(school * one); a = int(allr * one)
    return value * (one + s) * (one + a) // (one * one)

if __name__ == '__main__':
    # r1 F-2: 50 at 16 %
    print('50 @ 1600bp:', new_stage1(50, F(0), f32_of_bp(1600)), old_stage1(50, F(0), f32_of_bp(1600)))
    print('200 @ 2250bp:', new_stage1(200, F(0), f32_of_bp(2250)), old_stage1(200, F(0), f32_of_bp(2250)))
    print('200 @ 0x3e666667:', new_stage1(200, F(0), bits_to_f(0x3e666667)), old_stage1(200, F(0), bits_to_f(0x3e666667)))
    # the existing order test: 50 x 1.5 x 1.15 (1000 + 500 bp as two f32 added and rounded)
    a = f32(f32_of_bp(1000) + f32_of_bp(500))
    print('50 orb sorc:', new_stage1(50, f32_of_bp(5000), a), old_stage1(50, f32_of_bp(5000), a))
    # where does the hero's own single value still matter? adjacent f32 around 0.225, bolt values
    lo, hi = bits_to_f(0x3e666666), bits_to_f(0x3e666667)
    found = []
    for p in range(1, 256):
        value = 10 * p + 10
        x, y = new_stage1(value, F(0), lo)[0], new_stage1(value, F(0), hi)[0]
        if x != y:
            found.append((p, value, x, y, value * 12250 // 10000))
    print('own single matters (power, value, lo, hi, exact):', found[:12], len(found))
