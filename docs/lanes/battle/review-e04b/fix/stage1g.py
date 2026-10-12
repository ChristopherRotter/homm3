from fractions import Fraction as F
from stage1 import *
import math
out = []
# bonus rules: the spell's own +50 % (t * 5000 // 10000) and the scaled specialist ceil(per_level*steps*t/10000)
def own50(t): return t * 5000 // 10000
def own100(t): return t
def scaled(steps):
    return lambda t: -((-t * 300 * steps) // 10000)
rules = [('own50', own50), ('own100', own100)] + [(f'spec{st}', scaled(st)) for st in (1, 2, 5, 10, 20, 30)]
for abp in range(0, 5001, 25):
    a = f32_of_bp(abp)
    for sbp in (0, 5000):
        s = f32_of_bp(sbp)
        for p in range(0, 256):
            value = 10 * p + 10
            for name, rule in rules:
                t, w = new_stage1(value, s, a, rule)
                if w != t + rule(t):
                    out.append((sbp, abp, value, name, t, rule(t), w))
for o in out[:20]: print(o)
print(len(out))
