from fractions import Fraction as F
from stage1 import *
def one_store(value, school, allr):
    v = f32(value)
    w = r53(r53(1 + allr) * r53(v * r53(1 + school)))
    return int(f32(w))
def no_store(value, school, allr):
    v = F(value)
    return int(r53(r53(1 + allr) * r53(v * r53(1 + school))))
out = []
for sbp in (3000, 2500, 7000, 1000):
    for abp in (500, 1000, 1500, 1600, 2250, 300):
        s, a = f32_of_bp(sbp), f32_of_bp(abp)
        for p in range(0, 256):
            value = 10 * p + 10
            n = new_stage1(value, s, a)[0]
            o1, o0, old = one_store(value, s, a), no_store(value, s, a), old_stage1(value, s, a)
            exact = value * (10000 + sbp) * (10000 + abp) // 10**8
            if n != o1:
                out.append((sbp, abp, value, n, o1, o0, old, exact))
print('two stores != one store: (school bp, all bp, value, new, one-store, no-store, old, exact)')
for o in out[:25]: print(o)
print(len(out))
