from fractions import Fraction as F
from stage1 import *
lo, hi = bits_to_f(0x3e666666), bits_to_f(0x3e666667)
found = []
for value in range(1, 60001):
    x, y = new_stage1(value, F(0), lo)[0], new_stage1(value, F(0), hi)[0]
    if x != y:
        found.append((value, x, y, value * 12250 // 10000, old_stage1(value, F(0), lo), old_stage1(value, F(0), hi)))
print('values where the own single matters at 22.5 %:', found[:10], len(found))
# how often does new differ from old for plain bp rates (f32 of bp/10000), bolt values?
diff = []
for bp in range(0, 30001, 50):
    r = f32_of_bp(bp)
    for p in range(1, 256):
        value = 10 * p + 10
        n, o = new_stage1(value, F(0), r)[0], old_stage1(value, F(0), r)
        if n != o:
            diff.append((bp, value, o, n, value * (10000 + bp) // 10000))
print('new != old (bp, value, old, new, exact):', diff[:15], len(diff))
print('all new == exact?', all(d[3] == d[4] for d in diff))
