from fractions import Fraction as F
from stage1 import *
def one_store(value, school, allr):
    v = f32(value)
    w = r53(r53(1 + allr) * r53(v * r53(1 + school)))
    return int(f32(w))
out = []; cnt = 0
for sbp in range(100, 10001, 100):
    s = f32_of_bp(sbp)
    for abp in range(100, 5001, 100):
        a = f32_of_bp(abp)
        k = (10000 + sbp) * (10000 + abp)
        # values making the exact product whole
        from math import gcd
        step = 10**8 // gcd(10**8, k)
        for value in range(step, 20001, step):
            cnt += 1
            n = new_stage1(value, s, a)[0]
            o1 = one_store(value, s, a)
            exact = value * k // 10**8
            if n != o1 or n != exact:
                out.append((sbp, abp, value, n, o1, old_stage1(value, s, a), exact))
for o in out[:30]: print(o)
print(len(out), 'of', cnt)
