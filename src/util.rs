use std::time;

pub struct ResetableTimer {
    time: Option<time::Instant>,
}

impl ResetableTimer {
    pub fn new() -> Self {
        Self {
            time: Some(time::Instant::now()),
        }
    }

    pub fn seconds_since(&mut self) -> f64 {
        let current = time::Instant::now();
        let since = current.duration_since(self.time.unwrap());
        self.time.replace(current);

        since.as_secs_f64()
    }
}

pub fn mod_inv(a: u64, module: u64) -> u64 {
    // https://rosettacode.org/wiki/Modular_inverse#Rust
    let a = a as i64;
    let module = module as i64;

    let mut mn = (module, a);
    let mut xy = (0, 1);

    while mn.1 != 0 {
        xy = (xy.1, xy.0 - (mn.0 / mn.1) * xy.1);
        mn = (mn.1, mn.0 % mn.1);
    }

    while xy.0 < 0 {
        xy.0 += module;
    }
    xy.0 as u64
}
