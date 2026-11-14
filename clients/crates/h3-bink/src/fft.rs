//! The sound decoder's FFT (E27 M2 T2): Takuya Ooura's `fft8g.c` (its `cdft` with the sign −1:
//! `makewt`, `bitrv2`, `bitrv2conj`, `cftbsub`, `cft1st`, `cftmdl`) in single precision, as
//! `BINKW32.DLL` has it compiled (0x1000C610 to 0x1000E5F0): each expression computed on the x87
//! at 53 bits ([`X`]) and rounded to binary32 (`.single()`) exactly where the DLL stores it to
//! memory, the values it keeps on the stack not. Where it stores which value was read from the
//! DLL's code (a scratch symbolic run of its instructions, aligned with Ooura's C statement by
//! statement: every store of the transforms of 1024, 2048 and 4096 values, each rounding the same
//! in every iteration; design §8); this file was written from that reading and Ooura's C.
//!
//! **Origin.** Ported from Takuya Ooura's General Purpose FFT Package, `fft8g.c` (Copyright
//! Takuya Ooura, 1996-2001, <http://www.kurims.kyoto-u.ac.jp/~ooura/fft.html>: "You may use,
//! copy, modify and distribute this code for any purpose (include commercial use) and without
//! fee. Please refer to this package when you modify this code."), as compiled into the game's
//! `BINKW32.DLL`; distributed under the GNU General Public License version 2 or later as the rest
//! of the project (ADR-0009; the user's choice, CLT-Q120). Changes: the arithmetic is the DLL's
//! (binary32 arrays, the x87's 53-bit precision and its stores' roundings, in integers), the
//! twiddles' cosines and sines correctly rounded in integers (`trig`, which the DLL's values
//! are), the integers' types and the loops' form Rust's.
//!
//! The arrays' sizes are fixed by the transform's length (`Fft::new`); every index the loops
//! form lies inside them by Ooura's construction, so they index directly.

use alloc::vec;
use alloc::vec::Vec;

use crate::trig;
use crate::x87::X;

/// The binary32 bits of `atan(1)` as the DLL rounds it (`fpatan` 64 bits, 53, then 24).
const QUARTER_PI: u32 = 0x3F49_0FDB;

/// A transform's tables (`ip` and `w`, built at the first call as `cdft` builds them).
#[derive(Debug, Clone)]
pub(crate) struct Fft {
    n: usize,
    ip: Vec<usize>,
    w: Vec<X>,
}

impl Fft {
    /// The tables for `n` values (a power of two from 16 up).
    pub(crate) fn new(n: usize) -> Self {
        let mut ip = vec![0; 2 + n];
        let mut w = vec![X::ZERO; n / 4];
        makewt(n >> 2, &mut ip, &mut w);
        Self { n, ip, w }
    }

    /// The twiddles (`w`, for the checks).
    pub(crate) fn twiddles(&self) -> &[X] {
        &self.w
    }

    /// `cdft(n, -1, a, ip, w)` on `a` (`n` binary32 values: `n / 2` complex ones).
    pub(crate) fn inverse(&mut self, a: &mut [X]) {
        bitrv2conj(self.n, &mut self.ip[2..], a);
        cftbsub(self.n, a, &self.w);
    }
}

/// `makewt`: the twiddles. `delta`'s quarter π rounded as the DLL's (0x1000C6C9) and divided
/// by a power of two (exact); `x` the cosine of each angle at 53 bits, `y` the sine of the
/// angle stored as binary32 (0x1000C722 to 0x1000C72C), both stored as binary32.
fn makewt(nw: usize, ip: &mut [usize], w: &mut [X]) {
    ip[0] = nw;
    ip[1] = 1;
    if nw > 2 {
        let nwh = nw >> 1;
        let delta = X::from_bits32(QUARTER_PI).scaled(-(nwh.trailing_zeros() as i32));
        w[0] = X::int(1);
        w[1] = X::ZERO;
        w[nwh] = trig::cos(X::int(nwh as i32).mul(delta));
        w[nwh + 1] = w[nwh];
        if nwh > 2 {
            let mut j = 2;
            while j < nwh {
                let angle = X::int(j as i32).mul(delta);
                let x = trig::cos(angle);
                let y = trig::sin(angle.single());
                w[j] = x;
                w[j + 1] = y;
                w[nw - j] = y;
                w[nw - j + 1] = x;
                j += 2;
            }
            let mut j = nwh - 2;
            while j >= 2 {
                let x = w[2 * j];
                let y = w[2 * j + 1];
                w[nwh + j] = x;
                w[nwh + j + 1] = y;
                j -= 2;
            }
            bitrv2(nw, &mut ip[2..], w);
        }
    }
}

fn bitrv2(n: usize, ip: &mut [usize], a: &mut [X]) {
    let (mut j, mut j1, mut k, mut k1, mut l, mut m, m2): (
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    );
    ip[0] = 0;
    l = n;
    m = 1;
    while (m << 3) < l {
        l >>= 1;
        j = 0;
        while j < m {
            ip[m + j] = ip[j] + l;
            j += 1;
        }
        m <<= 1;
    }
    m2 = 2 * m;
    if (m << 3) == l {
        k = 0;
        while k < m {
            j = 0;
            while j < k {
                j1 = (2 * j) + ip[k];
                k1 = (2 * k) + ip[j];
                let xr = a[j1];
                let xi = a[j1 + 1];
                let yr = a[k1];
                let yi = a[k1 + 1];
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j1 += m2;
                k1 += 2 * m2;
                let xr = a[j1];
                let xi = a[j1 + 1];
                let yr = a[k1];
                let yi = a[k1 + 1];
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j1 += m2;
                k1 -= m2;
                let xr = a[j1];
                let xi = a[j1 + 1];
                let yr = a[k1];
                let yi = a[k1 + 1];
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j1 += m2;
                k1 += 2 * m2;
                let xr = a[j1];
                let xi = a[j1 + 1];
                let yr = a[k1];
                let yi = a[k1 + 1];
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j += 1;
            }
            j1 = ((2 * k) + m2) + ip[k];
            k1 = j1 + m2;
            let xr = a[j1];
            let xi = a[j1 + 1];
            let yr = a[k1];
            let yi = a[k1 + 1];
            a[j1] = yr.single();
            a[j1 + 1] = yi.single();
            a[k1] = xr.single();
            a[k1 + 1] = xi.single();
            k += 1;
        }
    } else {
        k = 1;
        while k < m {
            j = 0;
            while j < k {
                j1 = (2 * j) + ip[k];
                k1 = (2 * k) + ip[j];
                let xr = a[j1];
                let xi = a[j1 + 1];
                let yr = a[k1];
                let yi = a[k1 + 1];
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j1 += m2;
                k1 += m2;
                let xr = a[j1];
                let xi = a[j1 + 1];
                let yr = a[k1];
                let yi = a[k1 + 1];
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j += 1;
            }
            k += 1;
        }
    }
}

fn bitrv2conj(n: usize, ip: &mut [usize], a: &mut [X]) {
    let (mut j, mut j1, mut k, mut k1, mut l, mut m, m2): (
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    );
    ip[0] = 0;
    l = n;
    m = 1;
    while (m << 3) < l {
        l >>= 1;
        j = 0;
        while j < m {
            ip[m + j] = ip[j] + l;
            j += 1;
        }
        m <<= 1;
    }
    m2 = 2 * m;
    if (m << 3) == l {
        k = 0;
        while k < m {
            j = 0;
            while j < k {
                j1 = (2 * j) + ip[k];
                k1 = (2 * k) + ip[j];
                let xr = a[j1];
                let xi = a[j1 + 1].neg();
                let yr = a[k1];
                let yi = a[k1 + 1].neg();
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j1 += m2;
                k1 += 2 * m2;
                let xr = a[j1];
                let xi = a[j1 + 1].neg();
                let yr = a[k1];
                let yi = a[k1 + 1].neg();
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j1 += m2;
                k1 -= m2;
                let xr = a[j1];
                let xi = a[j1 + 1].neg();
                let yr = a[k1];
                let yi = a[k1 + 1].neg();
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j1 += m2;
                k1 += 2 * m2;
                let xr = a[j1];
                let xi = a[j1 + 1].neg();
                let yr = a[k1];
                let yi = a[k1 + 1].neg();
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j += 1;
            }
            k1 = (2 * k) + ip[k];
            a[k1 + 1] = a[k1 + 1].neg().single();
            j1 = k1 + m2;
            k1 = j1 + m2;
            let xr = a[j1];
            let xi = a[j1 + 1].neg();
            let yr = a[k1];
            let yi = a[k1 + 1].neg();
            a[j1] = yr.single();
            a[j1 + 1] = yi.single();
            a[k1] = xr.single();
            a[k1 + 1] = xi.single();
            k1 += m2;
            a[k1 + 1] = a[k1 + 1].neg().single();
            k += 1;
        }
    } else {
        a[1] = a[1].neg().single();
        a[m2 + 1] = a[m2 + 1].neg().single();
        k = 1;
        while k < m {
            j = 0;
            while j < k {
                j1 = (2 * j) + ip[k];
                k1 = (2 * k) + ip[j];
                let xr = a[j1];
                let xi = a[j1 + 1].neg();
                let yr = a[k1];
                let yi = a[k1 + 1].neg();
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j1 += m2;
                k1 += m2;
                let xr = a[j1];
                let xi = a[j1 + 1].neg();
                let yr = a[k1];
                let yi = a[k1 + 1].neg();
                a[j1] = yr.single();
                a[j1 + 1] = yi.single();
                a[k1] = xr.single();
                a[k1 + 1] = xi.single();
                j += 1;
            }
            k1 = (2 * k) + ip[k];
            a[k1 + 1] = a[k1 + 1].neg().single();
            a[(k1 + m2) + 1] = a[(k1 + m2) + 1].neg().single();
            k += 1;
        }
    }
}

fn cftbsub(n: usize, a: &mut [X], w: &[X]) {
    let (mut j, mut j1, mut j2, mut j3, mut j4, mut j5, mut j6, mut j7, mut l): (
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    );
    l = 2;
    if n > 16 {
        cft1st(n, a, w);
        l = 16;
        while (l << 3) < n {
            cftmdl(n, l, a, w);
            l <<= 3;
        }
    }
    if (l << 2) < n {
        let wn4r = w[2];
        j = 0;
        while j < l {
            j1 = j + l;
            j2 = j1 + l;
            j3 = j2 + l;
            j4 = j3 + l;
            j5 = j4 + l;
            j6 = j5 + l;
            j7 = j6 + l;
            let x0r = a[j].add(a[j1]);
            let x0i = a[j + 1].neg().sub(a[j1 + 1]);
            let x1r = a[j].sub(a[j1]);
            let x1i = a[j + 1].neg().add(a[j1 + 1]);
            let x2r = a[j2].add(a[j3]);
            let x2i = a[j2 + 1].add(a[j3 + 1]);
            let x3r = a[j2].sub(a[j3]);
            let x3i = a[j2 + 1].sub(a[j3 + 1]);
            let y0r = x0r.add(x2r);
            let y0i = x0i.single().sub(x2i.single());
            let y2r = x0r.sub(x2r);
            let y2i = x0i.single().add(x2i.single());
            let y1r = x1r.single().sub(x3i.single());
            let y1i = x1i.single().sub(x3r.single());
            let y3r = x1r.single().add(x3i.single());
            let y3i = x1i.single().add(x3r.single());
            let x0r = a[j4].add(a[j5]);
            let x0i = a[j4 + 1].add(a[j5 + 1]);
            let x1r = a[j4].sub(a[j5]);
            let x1i = a[j4 + 1].sub(a[j5 + 1]);
            let x2r = a[j6].add(a[j7]);
            let x2i = a[j6 + 1].add(a[j7 + 1]);
            let x3r = a[j6].sub(a[j7]);
            let x3i = a[j6 + 1].sub(a[j7 + 1]);
            let y4r = x0r.add(x2r);
            let y4i = x0i.single().add(x2i.single());
            let y6r = x0r.sub(x2r);
            let y6i = x0i.single().sub(x2i.single());
            let x0r = x1r.single().sub(x3i.single());
            let x0i = x1i.single().add(x3r.single());
            let x2r = x1r.single().add(x3i.single());
            let x2i = x1i.single().sub(x3r.single());
            let y5r = wn4r.mul(x0r.sub(x0i));
            let y5i = wn4r.mul(x0r.add(x0i));
            let y7r = wn4r.mul(x2r.single().sub(x2i.single()));
            let y7i = wn4r.mul(x2r.single().add(x2i.single()));
            a[j1] = y1r.add(y5r.single()).single();
            a[j1 + 1] = y1i.single().sub(y5i.single()).single();
            a[j5] = y1r.sub(y5r.single()).single();
            a[j5 + 1] = y1i.single().add(y5i.single()).single();
            a[j3] = y3r.single().sub(y7i.single()).single();
            a[j3 + 1] = y3i.single().sub(y7r.single()).single();
            a[j7] = y3r.single().add(y7i.single()).single();
            a[j7 + 1] = y3i.single().add(y7r.single()).single();
            a[j] = y0r.single().add(y4r.single()).single();
            a[j + 1] = y0i.single().sub(y4i.single()).single();
            a[j4] = y0r.single().sub(y4r.single()).single();
            a[j4 + 1] = y0i.single().add(y4i.single()).single();
            a[j2] = y2r.sub(y6i.single()).single();
            a[j2 + 1] = y2i.single().sub(y6r.single()).single();
            a[j6] = y2r.add(y6i.single()).single();
            a[j6 + 1] = y2i.single().add(y6r.single()).single();
            j += 2;
        }
    } else if (l << 2) == n {
        j = 0;
        while j < l {
            j1 = j + l;
            j2 = j1 + l;
            j3 = j2 + l;
            let x0r = a[j].add(a[j1]);
            let x0i = a[j + 1].neg().sub(a[j1 + 1]);
            let x1r = a[j].sub(a[j1]);
            let x1i = a[j + 1].neg().add(a[j1 + 1]);
            let x2r = a[j2].add(a[j3]);
            let x2i = a[j2 + 1].add(a[j3 + 1]);
            let x3r = a[j2].sub(a[j3]);
            let x3i = a[j2 + 1].sub(a[j3 + 1]);
            a[j] = x0r.add(x2r).single();
            a[j + 1] = x0i.single().sub(x2i.single()).single();
            a[j2] = x0r.sub(x2r).single();
            a[j2 + 1] = x0i.single().add(x2i.single()).single();
            a[j1] = x1r.single().sub(x3i.single()).single();
            a[j1 + 1] = x1i.single().sub(x3r.single()).single();
            a[j3] = x1r.single().add(x3i.single()).single();
            a[j3 + 1] = x1i.single().add(x3r.single()).single();
            j += 2;
        }
    } else {
        j = 0;
        while j < l {
            j1 = j + l;
            let x0r = a[j].sub(a[j1]);
            let x0i = a[j + 1].neg().add(a[j1 + 1]);
            a[j] = a[j].add(a[j1]).single();
            a[j + 1] = a[j + 1].neg().sub(a[j1 + 1]).single();
            a[j1] = x0r.single();
            a[j1 + 1] = x0i.single().single();
            j += 2;
        }
    }
}

fn cft1st(n: usize, a: &mut [X], w: &[X]) {
    let (mut j, mut k1): (usize, usize);
    let wn4r = w[2];
    let x0r = a[0].add(a[2]);
    let x0i = a[1].add(a[3]);
    let x1r = a[0].sub(a[2]);
    let x1i = a[1].sub(a[3]);
    let x2r = a[4].add(a[6]);
    let x2i = a[5].add(a[7]);
    let x3r = a[4].sub(a[6]);
    let x3i = a[5].sub(a[7]);
    let y0r = x0r.add(x2r);
    let y0i = x0i.single().add(x2i.single());
    let y2r = x0r.sub(x2r);
    let y2i = x0i.single().sub(x2i.single());
    let y1r = x1r.single().sub(x3i.single());
    let y1i = x1i.single().add(x3r.single());
    let y3r = x1r.single().add(x3i.single());
    let y3i = x1i.single().sub(x3r.single());
    let x0r = a[8].add(a[10]);
    let x0i = a[9].add(a[11]);
    let x1r = a[8].sub(a[10]);
    let x1i = a[9].sub(a[11]);
    let x2r = a[12].add(a[14]);
    let x2i = a[13].add(a[15]);
    let x3r = a[12].sub(a[14]);
    let x3i = a[13].sub(a[15]);
    let y4r = x0r.add(x2r);
    let y4i = x0i.single().add(x2i.single());
    let y6r = x0r.sub(x2r);
    let y6i = x0i.single().sub(x2i.single());
    let x0r = x1r.single().sub(x3i.single());
    let x0i = x1i.single().add(x3r.single());
    let x2r = x1r.single().add(x3i.single());
    let x2i = x1i.single().sub(x3r.single());
    let y5r = wn4r.mul(x0r.sub(x0i));
    let y5i = wn4r.mul(x0r.add(x0i));
    let y7r = wn4r.mul(x2r.single().sub(x2i.single()));
    let y7i = wn4r.mul(x2r.single().add(x2i.single()));
    a[2] = y1r.add(y5r.single()).single();
    a[3] = y1i.single().add(y5i.single()).single();
    a[10] = y1r.sub(y5r.single()).single();
    a[11] = y1i.single().sub(y5i.single()).single();
    a[6] = y3r.single().sub(y7i.single()).single();
    a[7] = y3i.single().add(y7r.single()).single();
    a[14] = y3r.single().add(y7i.single()).single();
    a[15] = y3i.single().sub(y7r.single()).single();
    a[0] = y0r.single().add(y4r.single()).single();
    a[1] = y0i.single().add(y4i.single()).single();
    a[8] = y0r.single().sub(y4r.single()).single();
    a[9] = y0i.single().sub(y4i.single()).single();
    a[4] = y2r.sub(y6i.single()).single();
    a[5] = y2i.single().add(y6r.single()).single();
    a[12] = y2r.add(y6i.single()).single();
    a[13] = y2i.single().sub(y6r.single()).single();
    if n > 16 {
        let wk1r = w[4];
        let wk1i = w[5];
        let x0r = a[16].add(a[18]);
        let x0i = a[17].add(a[19]);
        let x1r = a[16].sub(a[18]);
        let x1i = a[17].sub(a[19]);
        let x2r = a[20].add(a[22]);
        let x2i = a[21].add(a[23]);
        let x3r = a[20].sub(a[22]);
        let x3i = a[21].sub(a[23]);
        let y0r = x0r.add(x2r);
        let y0i = x0i.single().add(x2i.single());
        let y2r = x0r.sub(x2r);
        let y2i = x0i.single().sub(x2i.single());
        let y1r = x1r.single().sub(x3i.single());
        let y1i = x1i.single().add(x3r.single());
        let y3r = x1r.single().add(x3i.single());
        let y3i = x1i.single().sub(x3r.single());
        let x0r = a[24].add(a[26]);
        let x0i = a[25].add(a[27]);
        let x1r = a[24].sub(a[26]);
        let x1i = a[25].sub(a[27]);
        let x2r = a[28].add(a[30]);
        let x2i = a[29].add(a[31]);
        let x3r = a[28].sub(a[30]);
        let x3i = a[29].sub(a[31]);
        let y4r = x0r.add(x2r);
        let y4i = x0i.single().add(x2i.single());
        let y6r = x0r.sub(x2r);
        let y6i = x0i.single().sub(x2i.single());
        let x0r = x1r.single().sub(x3i.single());
        let x0i = x1i.single().add(x3r.single());
        let x2r = x1r.single().add(x3i.single());
        let x2i = x3r.single().sub(x1i.single());
        let y5r = wk1i.mul(x0r).sub(wk1r.mul(x0i));
        let y5i = wk1i.mul(x0i).add(wk1r.mul(x0r));
        let y7r = wk1r.mul(x2r.single()).add(wk1i.mul(x2i.single()));
        let y7i = wk1r.mul(x2i.single()).sub(wk1i.mul(x2r.single()));
        let x0r = wk1r.mul(y1r).sub(wk1i.mul(y1i));
        let x0i = wk1r.mul(y1i).add(wk1i.mul(y1r));
        a[18] = x0r.single().add(y5r.single()).single();
        a[19] = x0i.add(y5i.single()).single();
        a[26] = y5i.single().sub(x0i).single();
        a[27] = x0r.single().sub(y5r.single()).single();
        let x0r = wk1i.mul(y3r.single()).sub(wk1r.mul(y3i.single()));
        let x0i = wk1i.mul(y3i.single()).add(wk1r.mul(y3r.single()));
        a[22] = x0r.single().sub(y7r.single()).single();
        a[23] = x0i.add(y7i.single()).single();
        a[30] = y7i.single().sub(x0i).single();
        a[31] = x0r.single().add(y7r.single()).single();
        a[16] = y0r.single().add(y4r.single()).single();
        a[17] = y0i.single().add(y4i.single()).single();
        a[24] = y4i.single().sub(y0i.single()).single();
        a[25] = y0r.single().sub(y4r.single()).single();
        let x0r = y2r.single().sub(y6i.single());
        let x0i = y2i.single().add(y6r.single());
        a[20] = wn4r.mul(x0r.sub(x0i)).single();
        a[21] = wn4r.mul(x0i.add(x0r)).single();
        let x0r = y6r.single().sub(y2i.single());
        let x0i = y2r.single().add(y6i.single());
        a[28] = wn4r.mul(x0r.sub(x0i)).single();
        a[29] = wn4r.mul(x0i.add(x0r)).single();
        k1 = 4;
        j = 32;
        while j < n {
            k1 += 4;
            let wk1r = w[k1];
            let wk1i = w[k1 + 1];
            let wk2r = w[k1 + 2];
            let wk2i = w[k1 + 3];
            let wtmp = X::int(2).mul(wk2i);
            let wk3r = wk1r.sub(wtmp.mul(wk1i));
            let wk3i = wtmp.mul(wk1r).sub(wk1i);
            let wk4r = X::int(1).sub(wtmp.mul(wk2i));
            let wk4i = wtmp.mul(wk2r);
            let wtmp = X::int(2).mul(wk4i);
            let wk5r = wk3r.single().sub(wtmp.mul(wk1i));
            let wk5i = wtmp.mul(wk1r).sub(wk3i.single());
            let wk6r = wk2r.sub(wtmp.mul(wk2i));
            let wk6i = wtmp.mul(wk2r).sub(wk2i);
            let wk7r = wk1r.sub(wtmp.mul(wk3i.single()));
            let wk7i = wtmp.mul(wk3r.single()).sub(wk1i);
            let x0r = a[j].add(a[j + 2]);
            let x0i = a[j + 1].add(a[j + 3]);
            let x1r = a[j].sub(a[j + 2]);
            let x1i = a[j + 1].sub(a[j + 3]);
            let x2r = a[j + 4].add(a[j + 6]);
            let x2i = a[j + 5].add(a[j + 7]);
            let x3r = a[j + 4].sub(a[j + 6]);
            let x3i = a[j + 5].sub(a[j + 7]);
            let y0r = x0r.add(x2r);
            let y0i = x0i.single().add(x2i.single());
            let y2r = x0r.sub(x2r);
            let y2i = x0i.single().sub(x2i.single());
            let y1r = x1r.single().sub(x3i.single());
            let y1i = x1i.single().add(x3r.single());
            let y3r = x1r.single().add(x3i.single());
            let y3i = x1i.single().sub(x3r.single());
            let x0r = a[j + 8].add(a[j + 10]);
            let x0i = a[j + 9].add(a[j + 11]);
            let x1r = a[j + 8].sub(a[j + 10]);
            let x1i = a[j + 9].sub(a[j + 11]);
            let x2r = a[j + 12].add(a[j + 14]);
            let x2i = a[j + 13].add(a[j + 15]);
            let x3r = a[j + 12].sub(a[j + 14]);
            let x3i = a[j + 13].sub(a[j + 15]);
            let y4r = x0r.add(x2r);
            let y4i = x0i.single().add(x2i.single());
            let y6r = x0r.sub(x2r);
            let y6i = x0i.single().sub(x2i.single());
            let x0r = x1r.single().sub(x3i.single());
            let x0i = x1i.single().add(x3r.single());
            let x2r = x1r.single().add(x3i.single());
            let x2i = x1i.single().sub(x3r.single());
            let y5r = wn4r.mul(x0r.sub(x0i));
            let y5i = wn4r.mul(x0r.add(x0i));
            let y7r = wn4r.mul(x2r.single().sub(x2i.single()));
            let y7i = wn4r.mul(x2r.single().add(x2i.single()));
            let x0r = y1r.single().add(y5r.single());
            let x0i = y1i.single().add(y5i.single());
            a[j + 2] = wk1r.mul(x0r).sub(wk1i.mul(x0i)).single();
            a[j + 3] = wk1r.mul(x0i).add(wk1i.mul(x0r)).single();
            let x0r = y1r.single().sub(y5r.single());
            let x0i = y1i.single().sub(y5i.single());
            a[j + 10] = wk5r.single().mul(x0r).sub(wk5i.single().mul(x0i)).single();
            a[j + 11] = wk5r.single().mul(x0i).add(wk5i.single().mul(x0r)).single();
            let x0r = y3r.single().sub(y7i.single());
            let x0i = y3i.single().add(y7r.single());
            a[j + 6] = wk3r.single().mul(x0r).sub(wk3i.single().mul(x0i)).single();
            a[j + 7] = wk3r.single().mul(x0i).add(wk3i.single().mul(x0r)).single();
            let x0r = y3r.single().add(y7i.single());
            let x0i = y3i.single().sub(y7r.single());
            a[j + 14] = wk7r.single().mul(x0r).sub(wk7i.single().mul(x0i)).single();
            a[j + 15] = wk7r.single().mul(x0i).add(wk7i.single().mul(x0r)).single();
            a[j] = y0r.single().add(y4r.single()).single();
            a[j + 1] = y0i.single().add(y4i.single()).single();
            let x0r = y0r.single().sub(y4r.single());
            let x0i = y0i.single().sub(y4i.single());
            a[j + 8] = wk4r.single().mul(x0r).sub(wk4i.single().mul(x0i)).single();
            a[j + 9] = wk4r.single().mul(x0i).add(wk4i.single().mul(x0r)).single();
            let x0r = y2r.single().sub(y6i.single());
            let x0i = y2i.single().add(y6r.single());
            a[j + 4] = wk2r.mul(x0r).sub(wk2i.mul(x0i)).single();
            a[j + 5] = wk2r.mul(x0i).add(wk2i.mul(x0r)).single();
            let x0r = y2r.single().add(y6i.single());
            let x0i = y2i.single().sub(y6r.single());
            a[j + 12] = wk6r.single().mul(x0r).sub(wk6i.single().mul(x0i)).single();
            a[j + 13] = wk6r.single().mul(x0i).add(wk6i.single().mul(x0r)).single();
            j += 16;
        }
    }
}

fn cftmdl(n: usize, l: usize, a: &mut [X], w: &[X]) {
    let (mut j, mut j1, mut j2, mut j3, mut j4, mut j5, mut j6, mut j7, mut k, mut k1, m): (
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    );
    m = l << 3;
    let wn4r = w[2];
    j = 0;
    while j < l {
        j1 = j + l;
        j2 = j1 + l;
        j3 = j2 + l;
        j4 = j3 + l;
        j5 = j4 + l;
        j6 = j5 + l;
        j7 = j6 + l;
        let x0r = a[j].add(a[j1]);
        let x0i = a[j + 1].add(a[j1 + 1]);
        let x1r = a[j].sub(a[j1]);
        let x1i = a[j + 1].sub(a[j1 + 1]);
        let x2r = a[j2].add(a[j3]);
        let x2i = a[j2 + 1].add(a[j3 + 1]);
        let x3r = a[j2].sub(a[j3]);
        let x3i = a[j2 + 1].sub(a[j3 + 1]);
        let y0r = x0r.add(x2r);
        let y0i = x0i.single().add(x2i.single());
        let y2r = x0r.sub(x2r);
        let y2i = x0i.single().sub(x2i.single());
        let y1r = x1r.single().sub(x3i.single());
        let y1i = x1i.single().add(x3r.single());
        let y3r = x1r.single().add(x3i.single());
        let y3i = x1i.single().sub(x3r.single());
        let x0r = a[j4].add(a[j5]);
        let x0i = a[j4 + 1].add(a[j5 + 1]);
        let x1r = a[j4].sub(a[j5]);
        let x1i = a[j4 + 1].sub(a[j5 + 1]);
        let x2r = a[j6].add(a[j7]);
        let x2i = a[j6 + 1].add(a[j7 + 1]);
        let x3r = a[j6].sub(a[j7]);
        let x3i = a[j6 + 1].sub(a[j7 + 1]);
        let y4r = x0r.add(x2r);
        let y4i = x0i.single().add(x2i.single());
        let y6r = x0r.sub(x2r);
        let y6i = x0i.single().sub(x2i.single());
        let x0r = x1r.single().sub(x3i.single());
        let x0i = x1i.single().add(x3r.single());
        let x2r = x1r.single().add(x3i.single());
        let x2i = x1i.single().sub(x3r.single());
        let y5r = wn4r.mul(x0r.sub(x0i));
        let y5i = wn4r.mul(x0r.add(x0i));
        let y7r = wn4r.mul(x2r.single().sub(x2i.single()));
        let y7i = wn4r.mul(x2r.single().add(x2i.single()));
        a[j1] = y1r.add(y5r.single()).single();
        a[j1 + 1] = y1i.single().add(y5i.single()).single();
        a[j5] = y1r.sub(y5r.single()).single();
        a[j5 + 1] = y1i.single().sub(y5i.single()).single();
        a[j3] = y3r.single().sub(y7i.single()).single();
        a[j3 + 1] = y3i.single().add(y7r.single()).single();
        a[j7] = y3r.single().add(y7i.single()).single();
        a[j7 + 1] = y3i.single().sub(y7r.single()).single();
        a[j] = y0r.single().add(y4r.single()).single();
        a[j + 1] = y0i.single().add(y4i.single()).single();
        a[j4] = y0r.single().sub(y4r.single()).single();
        a[j4 + 1] = y0i.single().sub(y4i.single()).single();
        a[j2] = y2r.sub(y6i.single()).single();
        a[j2 + 1] = y2i.single().add(y6r.single()).single();
        a[j6] = y2r.add(y6i.single()).single();
        a[j6 + 1] = y2i.single().sub(y6r.single()).single();
        j += 2;
    }
    if m < n {
        let wk1r = w[4];
        let wk1i = w[5];
        j = m;
        while j < (l + m) {
            j1 = j + l;
            j2 = j1 + l;
            j3 = j2 + l;
            j4 = j3 + l;
            j5 = j4 + l;
            j6 = j5 + l;
            j7 = j6 + l;
            let x0r = a[j].add(a[j1]);
            let x0i = a[j + 1].add(a[j1 + 1]);
            let x1r = a[j].sub(a[j1]);
            let x1i = a[j + 1].sub(a[j1 + 1]);
            let x2r = a[j2].add(a[j3]);
            let x2i = a[j2 + 1].add(a[j3 + 1]);
            let x3r = a[j2].sub(a[j3]);
            let x3i = a[j2 + 1].sub(a[j3 + 1]);
            let y0r = x0r.add(x2r);
            let y0i = x0i.single().add(x2i.single());
            let y2r = x0r.sub(x2r);
            let y2i = x0i.single().sub(x2i.single());
            let y1r = x1r.single().sub(x3i.single());
            let y1i = x1i.single().add(x3r.single());
            let y3r = x1r.single().add(x3i.single());
            let y3i = x1i.single().sub(x3r.single());
            let x0r = a[j4].add(a[j5]);
            let x0i = a[j4 + 1].add(a[j5 + 1]);
            let x1r = a[j4].sub(a[j5]);
            let x1i = a[j4 + 1].sub(a[j5 + 1]);
            let x2r = a[j6].add(a[j7]);
            let x2i = a[j6 + 1].add(a[j7 + 1]);
            let x3r = a[j6].sub(a[j7]);
            let x3i = a[j6 + 1].sub(a[j7 + 1]);
            let y4r = x0r.add(x2r);
            let y4i = x0i.single().add(x2i.single());
            let y6r = x0r.sub(x2r);
            let y6i = x0i.single().sub(x2i.single());
            let x0r = x1r.single().sub(x3i.single());
            let x0i = x1i.single().add(x3r.single());
            let x2r = x1r.single().add(x3i.single());
            let x2i = x3r.single().sub(x1i.single());
            let y5r = wk1i.mul(x0r).sub(wk1r.mul(x0i));
            let y5i = wk1i.mul(x0i).add(wk1r.mul(x0r));
            let y7r = wk1r.mul(x2r.single()).add(wk1i.mul(x2i.single()));
            let y7i = wk1r.mul(x2i.single()).sub(wk1i.mul(x2r.single()));
            let x0r = wk1r.mul(y1r).sub(wk1i.mul(y1i));
            let x0i = wk1r.mul(y1i).add(wk1i.mul(y1r));
            a[j1] = x0r.single().add(y5r.single()).single();
            a[j1 + 1] = x0i.add(y5i.single()).single();
            a[j5] = y5i.single().sub(x0i).single();
            a[j5 + 1] = x0r.single().sub(y5r.single()).single();
            let x0r = wk1i.mul(y3r.single()).sub(wk1r.mul(y3i.single()));
            let x0i = wk1i.mul(y3i.single()).add(wk1r.mul(y3r.single()));
            a[j3] = x0r.sub(y7r.single()).single();
            a[j3 + 1] = x0i.add(y7i.single()).single();
            a[j7] = y7i.single().sub(x0i).single();
            a[j7 + 1] = x0r.add(y7r.single()).single();
            a[j] = y0r.single().add(y4r.single()).single();
            a[j + 1] = y0i.single().add(y4i.single()).single();
            a[j4] = y4i.single().sub(y0i.single()).single();
            a[j4 + 1] = y0r.single().sub(y4r.single()).single();
            let x0r = y2r.sub(y6i.single());
            let x0i = y2i.add(y6r.single());
            a[j2] = wn4r.mul(x0r.sub(x0i)).single();
            a[j2 + 1] = wn4r.mul(x0i.add(x0r)).single();
            let x0r = y6r.single().sub(y2i);
            let x0i = y2r.add(y6i.single());
            a[j6] = wn4r.mul(x0r.single().sub(x0i)).single();
            a[j6 + 1] = wn4r.mul(x0i.add(x0r.single())).single();
            j += 2;
        }
        k1 = 4;
        k = 2 * m;
        while k < n {
            k1 += 4;
            let wk1r = w[k1];
            let wk1i = w[k1 + 1];
            let wk2r = w[k1 + 2];
            let wk2i = w[k1 + 3];
            let wtmp = X::int(2).mul(wk2i);
            let wk3r = wk1r.sub(wtmp.mul(wk1i));
            let wk3i = wtmp.mul(wk1r).sub(wk1i);
            let wk4r = X::int(1).sub(wtmp.mul(wk2i));
            let wk4i = wtmp.mul(wk2r);
            let wtmp = X::int(2).mul(wk4i);
            let wk5r = wk3r.single().sub(wtmp.mul(wk1i));
            let wk5i = wtmp.mul(wk1r).sub(wk3i.single());
            let wk6r = wk2r.sub(wtmp.mul(wk2i));
            let wk6i = wtmp.mul(wk2r).sub(wk2i);
            let wk7r = wk1r.sub(wtmp.mul(wk3i.single()));
            let wk7i = wtmp.mul(wk3r.single()).sub(wk1i);
            j = k;
            while j < (l + k) {
                j1 = j + l;
                j2 = j1 + l;
                j3 = j2 + l;
                j4 = j3 + l;
                j5 = j4 + l;
                j6 = j5 + l;
                j7 = j6 + l;
                let x0r = a[j].add(a[j1]);
                let x0i = a[j + 1].add(a[j1 + 1]);
                let x1r = a[j].sub(a[j1]);
                let x1i = a[j + 1].sub(a[j1 + 1]);
                let x2r = a[j2].add(a[j3]);
                let x2i = a[j2 + 1].add(a[j3 + 1]);
                let x3r = a[j2].sub(a[j3]);
                let x3i = a[j2 + 1].sub(a[j3 + 1]);
                let y0r = x0r.add(x2r);
                let y0i = x0i.single().add(x2i.single());
                let y2r = x0r.sub(x2r);
                let y2i = x0i.single().sub(x2i.single());
                let y1r = x1r.single().sub(x3i.single());
                let y1i = x1i.single().add(x3r.single());
                let y3r = x1r.single().add(x3i.single());
                let y3i = x1i.single().sub(x3r.single());
                let x0r = a[j4].add(a[j5]);
                let x0i = a[j4 + 1].add(a[j5 + 1]);
                let x1r = a[j4].sub(a[j5]);
                let x1i = a[j4 + 1].sub(a[j5 + 1]);
                let x2r = a[j6].add(a[j7]);
                let x2i = a[j6 + 1].add(a[j7 + 1]);
                let x3r = a[j6].sub(a[j7]);
                let x3i = a[j6 + 1].sub(a[j7 + 1]);
                let y4r = x0r.add(x2r);
                let y4i = x0i.single().add(x2i.single());
                let y6r = x0r.sub(x2r);
                let y6i = x0i.single().sub(x2i.single());
                let x0r = x1r.single().sub(x3i.single());
                let x0i = x1i.single().add(x3r.single());
                let x2r = x1r.single().add(x3i.single());
                let x2i = x1i.single().sub(x3r.single());
                let y5r = wn4r.mul(x0r.sub(x0i));
                let y5i = wn4r.mul(x0r.add(x0i));
                let y7r = wn4r.mul(x2r.single().sub(x2i.single()));
                let y7i = wn4r.mul(x2r.single().add(x2i.single()));
                let x0r = y1r.add(y5r.single());
                let x0i = y1i.single().add(y5i.single());
                a[j1] = wk1r.mul(x0r).sub(wk1i.mul(x0i)).single();
                a[j1 + 1] = wk1r.mul(x0i).add(wk1i.mul(x0r)).single();
                let x0r = y1r.sub(y5r.single());
                let x0i = y1i.single().sub(y5i.single());
                a[j5] = wk5r.single().mul(x0r).sub(wk5i.single().mul(x0i)).single();
                a[j5 + 1] = wk5r.single().mul(x0i).add(wk5i.single().mul(x0r)).single();
                let x0r = y3r.single().sub(y7i.single());
                let x0i = y3i.single().add(y7r.single());
                a[j3] = wk3r.single().mul(x0r).sub(wk3i.single().mul(x0i)).single();
                a[j3 + 1] = wk3r.single().mul(x0i).add(wk3i.single().mul(x0r)).single();
                let x0r = y3r.single().add(y7i.single());
                let x0i = y3i.single().sub(y7r.single());
                a[j7] = wk7r.single().mul(x0r).sub(wk7i.single().mul(x0i)).single();
                a[j7 + 1] = wk7r.single().mul(x0i).add(wk7i.single().mul(x0r)).single();
                a[j] = y0r.single().add(y4r.single()).single();
                a[j + 1] = y0i.single().add(y4i.single()).single();
                let x0r = y0r.single().sub(y4r.single());
                let x0i = y0i.single().sub(y4i.single());
                a[j4] = wk4r.single().mul(x0r).sub(wk4i.single().mul(x0i)).single();
                a[j4 + 1] = wk4r.single().mul(x0i).add(wk4i.single().mul(x0r)).single();
                let x0r = y2r.sub(y6i.single());
                let x0i = y2i.single().add(y6r.single());
                a[j2] = wk2r.mul(x0r).sub(wk2i.mul(x0i)).single();
                a[j2 + 1] = wk2r.mul(x0i).add(wk2i.mul(x0r)).single();
                let x0r = y2r.add(y6i.single());
                let x0i = y2i.single().sub(y6r.single());
                a[j6] = wk6r.single().mul(x0r).sub(wk6i.single().mul(x0i)).single();
                a[j6 + 1] = wk6r.single().mul(x0i).add(wk6i.single().mul(x0r)).single();
                j += 2;
            }
            k += m;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The values' binary32 bits.
    fn bits(a: &[X]) -> Vec<u32> {
        a.iter().map(|x| x.to_bits32().unwrap()).collect()
    }

    #[test]
    fn a_constant_spectrum_is_an_impulse_and_back() {
        // Every size from 8 to 4096 (each of cftbsub's tails, bitrv2's and bitrv2conj's two
        // paths, makewt's): an impulse at 0 transforms to every value 1, exactly.
        for n in [8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096] {
            let mut fft = Fft::new(n);
            let mut a = vec![X::ZERO; n];
            a[0] = X::int(1);
            fft.inverse(&mut a);
            for (k, v) in bits(&a).iter().enumerate() {
                assert_eq!(*v, if k % 2 == 0 { 0x3F80_0000 } else { 0 }, "n {n} at {k}");
            }
        }
    }

    #[test]
    fn the_first_stage_of_sixteen_values_skips_its_twiddled_half() {
        // cft1st of 16 values (cftbsub never calls it for so few): the stage without its
        // second half, here of all ones.
        let fft = Fft::new(64);
        let mut a = vec![X::int(1); 16];
        cft1st(16, &mut a, fft.twiddles());
        let mut expected = vec![0; 16];
        expected[0] = 0x4100_0000;
        expected[1] = 0x4100_0000;
        assert_eq!(bits(&a), expected);
        // cftmdl over the whole array (cftbsub calls it only for less): its first half alone.
        let mut a = vec![X::ZERO; 128];
        a[0] = X::int(1);
        cftmdl(128, 16, &mut a, fft.twiddles());
        assert_eq!(bits(&a[..2]), [0x3F80_0000, 0]);
    }
}
