//! In-place radix-2 iterative FFT. Size must be a power of two.

#[derive(Clone, Copy)]
pub struct C {
    pub re: f32,
    pub im: f32,
}

impl C {
    pub fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }
    pub fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }
    pub fn norm(self) -> f32 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

pub fn fft_inplace(buf: &mut [C], inverse: bool) {
    let n = buf.len();
    assert!(n.is_power_of_two() && n >= 2);

    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            buf.swap(i, j);
        }
    }

    let sign = if inverse { 1.0f32 } else { -1.0f32 };
    let mut len = 2usize;
    while len <= n {
        let ang = sign * 2.0 * std::f32::consts::PI / len as f32;
        let wlen = C::new(ang.cos(), ang.sin());
        let mut i = 0usize;
        while i < n {
            let mut w = C::new(1.0, 0.0);
            for k in 0..(len / 2) {
                let u = buf[i + k];
                let v = C::new(
                    buf[i + k + len / 2].re * w.re - buf[i + k + len / 2].im * w.im,
                    buf[i + k + len / 2].re * w.im + buf[i + k + len / 2].im * w.re,
                );
                buf[i + k] = C::new(u.re + v.re, u.im + v.im);
                buf[i + k + len / 2] = C::new(u.re - v.re, u.im - v.im);
                w = C::new(w.re * wlen.re - w.im * wlen.im, w.re * wlen.im + w.im * wlen.re);
            }
            i += len;
        }
        len <<= 1;
    }

    if inverse {
        let s = 1.0 / n as f32;
        for c in buf.iter_mut() {
            c.re *= s;
            c.im *= s;
        }
    }
}

pub fn rfft_mags(frame: &[f32]) -> Vec<f32> {
    let n = frame.len();
    let mut buf: Vec<C> = frame.iter().map(|&x| C::new(x, 0.0)).collect();
    fft_inplace(&mut buf, false);
    let half = n / 2 + 1;
    buf.iter().take(half).map(|c| c.norm() / n as f32).collect()
}
