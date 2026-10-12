from fractions import Fraction as F
from stage1 import *
def one_store(value, school, allr):
    v = f32(value)
    w = r53(r53(1 + allr) * r53(v * r53(1 + school)))
    return int(f32(w))
out = []
import itertools
for sbp in range(100, 10001, 100):
    s = f32_of_bp(sbp)
    for abp in (500, 1000, 1500, 2250, 300, 3300, 7700):
        a = f32_of_bp(abp)
        for value in range(1, 400):
            n = new_stage1(value, s, a)[0]
            o1 = one_store(value, s, a)
            if n != o1:
                exact = value * (10000 + sbp) * (10000 + abp) // 10**8
                out.append((sbp, abp, value, n, o1, old_stage1(value, s, a), exact))
for o in out[:25]: print(o)
print(len(out))
