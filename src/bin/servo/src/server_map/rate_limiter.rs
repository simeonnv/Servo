use core::fmt;
use log::debug;
use pingora_limits::rate::Rate;
use std::{net::IpAddr, time::Duration};

pub struct RateLimiter {
    rate_limiter: Rate,
    max_req_sec: isize,
}

impl RateLimiter {
    pub fn new(max_req_sec: isize) -> Self {
        let rate = Rate::new(Duration::from_secs(1));
        Self {
            rate_limiter: rate,
            max_req_sec,
        }
    }

    pub fn rate_limit(&self, addr: &IpAddr) -> bool {
        let current_window_req = self.rate_limiter.observe(addr, 1);
        debug!("ratelimiter: {addr}:{current_window_req}");
        self.max_req_sec < current_window_req
    }
}

impl fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RateLimiter")
            .field("rate_limiter", &"<Rate>")
            .field("max_req_sec", &self.max_req_sec)
            .finish()
    }
}
