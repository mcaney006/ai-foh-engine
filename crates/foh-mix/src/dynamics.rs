use foh_core::audio::lin_from_db;

pub struct Compressor {
    threshold: f32,
    ratio: f32,
    att: f32,
    rel: f32,
    env: f32,
}

impl Compressor {
    pub fn new(sr: u32, threshold_db: f32, ratio: f32, attack_ms: f32, release_ms: f32) -> Self {
        Self {
            threshold: lin_from_db(threshold_db),
            ratio: ratio.max(1.0),
            att: coef(sr, attack_ms),
            rel: coef(sr, release_ms),
            env: 0.0,
        }
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        for x in samples.iter_mut() {
            let a = x.abs();
            if a > self.env {
                self.env = self.att * self.env + (1.0 - self.att) * a;
            } else {
                self.env = self.rel * self.env + (1.0 - self.rel) * a;
            }
            let gain = if self.env > self.threshold && self.threshold > 0.0 {
                let over = self.env / self.threshold;
                over.powf(1.0 / self.ratio - 1.0)
            } else {
                1.0
            };
            *x *= gain;
        }
    }
}

pub struct Gate {
    threshold: f32,
    att: f32,
    rel: f32,
    env: f32,
}

impl Gate {
    pub fn new(sr: u32, threshold_db: f32, attack_ms: f32, release_ms: f32) -> Self {
        Self {
            threshold: lin_from_db(threshold_db),
            att: coef(sr, attack_ms),
            rel: coef(sr, release_ms),
            env: 0.0,
        }
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        for x in samples.iter_mut() {
            let a = x.abs();
            if a > self.env {
                self.env = self.att * self.env + (1.0 - self.att) * a;
            } else {
                self.env = self.rel * self.env + (1.0 - self.rel) * a;
            }
            let open = if self.env >= self.threshold { 1.0 } else { 0.05 };
            *x *= open;
        }
    }
}

fn coef(sr: u32, ms: f32) -> f32 {
    let t = (ms.max(0.1) / 1000.0) * sr as f32;
    (-1.0 / t).exp()
}
