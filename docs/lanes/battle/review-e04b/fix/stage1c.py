from fractions import Fraction as F
from stage1 import *
c05 = f32(F(5, 100)); one = F(1)
cases = []
for skill_bp in (500, 1000, 1500):
    s = f32_of_bp(skill_bp)
    for level in range(1, 75):
        own = f32(r53(r53(r53(F(level) * c05) + one) * s))
        exact_bp = F(skill_bp) * (20 + level) / 20      # 0.05 per level
        bp = int(exact_bp + F(1, 2))                     # rounded basis points (a guess at the seam's bp)
        plain = f32_of_bp(bp)
        if own != plain:
            # values where they differ
            vals = [v for v in range(1, 5001) if new_stage1(v, F(0), own)[0] != new_stage1(v, F(0), plain)[0]]
            cases.append((skill_bp, level, bp, float(own), float(plain), vals[:5], len(vals)))
for c in cases[:40]:
    print(c)
print(len(cases))
