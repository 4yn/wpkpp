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
