//!
//! the hashclock is a blockchain primitive that enables a vdf
//! to be constructed trivially
//!
use gdex_crypto::hash::CryptoHash;
use std::time::{Duration, Instant};
use std::{fmt::Debug};
use types::{hash_clock::HashTime, spot::DiemCryptoMessage};

pub const TICKS_PER_CYCLE: u64 = 1_000;
pub const HASH_TIME_INIT_MSG: &str = "HashClock";

#[derive(Debug)]
pub struct HashClock {
    time: HashTime,
    ticks_per_cycle: u64,
    n_ticks: u64,
}
impl HashClock {
    pub fn new(init_time: HashTime, ticks_per_cycle: u64) -> Self {
        HashClock {
            time: init_time,
            ticks_per_cycle,
            n_ticks: 0,
        }
    }

    pub fn cycle(&mut self) {
        let mut init_tick: u64 = 0;
        while init_tick < self.ticks_per_cycle {
            self.tick();
            init_tick += 1;
        }
    }

    pub fn tick(&mut self) {
        self.time = DiemCryptoMessage(self.time.to_string()).hash();
        self.n_ticks += 1;
    }

    pub fn tick_for_interval(&mut self, time_in_secs: u64) {
        let start: Instant = Instant::now();
        let wait: Duration = Duration::from_secs(time_in_secs);
        loop {
            self.tick();
            if start.elapsed() >= wait {
                break;
            }
        }
    }

    pub fn get_hash_time(&self) -> HashTime {
        self.time
    }

    pub fn get_ticks_per_cycle(&self) -> u64 {
        self.ticks_per_cycle
    }

    pub fn get_n_ticks(&self) -> u64 {
        self.n_ticks
    }

    pub fn update_hash_time(&mut self, new_time: HashTime) {
        self.time = new_time;
    }

    pub fn update_ticks_per_cycle(&mut self, new_ticks_per_cycle: u64) {
        self.ticks_per_cycle = new_ticks_per_cycle;
    }
}

impl Default for HashClock {
    fn default() -> Self {
        Self::new(
            DiemCryptoMessage(HASH_TIME_INIT_MSG.to_string()).hash(),
            TICKS_PER_CYCLE,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock() {
        let clock: &mut HashClock = &mut HashClock::default();
        println!("init clock = {:?}", clock);
        clock.tick_for_interval(1);
        println!("final clock = {:?}", clock);
    }
}
