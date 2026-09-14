//! Pearson correlation on equal-length mono buffers.
//! Used for kick/bass phase-ish overlap, not as a stereo imaging meter.

pub fn pearson(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    if n < 32 {
        return 0.0;
    }
    let mut ma = 0.0f64;
    let mut mb = 0.0f64;
    for i in 0..n {
        ma += a[i] as f64;
        mb += b[i] as f64;
    }
    ma /= n as f64;
    mb /= n as f64;
    let mut num = 0.0f64;
    let mut da = 0.0f64;
    let mut db = 0.0f64;
    for i in 0..n {
        let xa = a[i] as f64 - ma;
        let xb = b[i] as f64 - mb;
        num += xa * xb;
        da += xa * xa;
        db += xb * xb;
    }
    let den = (da * db).sqrt();
    if den < 1.0e-18 {
        0.0
    } else {
        (num / den) as f32
    }
}
