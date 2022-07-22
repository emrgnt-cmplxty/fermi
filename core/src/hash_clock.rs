//!
//! the hashclock is a blockchain primitive that enables a vdf
//! to be constructed trivially
//!
use gdex_crypto::hash::CryptoHash;
use std::fmt::Debug;
use std::time::{Duration, Instant};
use types::{hash_clock::HashTime, spot::DiemCryptoMessage};

pub const DEFAULT_TICKS_PER_CYCLE: u64 = 1_000;
pub const DEFAULT_HASH_TIME_INIT_MSG: &str = "HashClock";

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

    pub fn tick_for_interval(&mut self, time_in_milis: u64) {
        let start: Instant = Instant::now();
        let wait: Duration = Duration::from_millis(time_in_milis);
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
            DiemCryptoMessage(DEFAULT_HASH_TIME_INIT_MSG.to_string()).hash(),
            DEFAULT_TICKS_PER_CYCLE,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock() {
        let clock_0: &mut HashClock = &mut HashClock::default();
        assert!(
            clock_0.get_ticks_per_cycle() == DEFAULT_TICKS_PER_CYCLE,
            "default clock does not match expectations"
        );
        clock_0.cycle();

        clock_0.update_ticks_per_cycle(2 * DEFAULT_TICKS_PER_CYCLE);
        assert!(
            clock_0.get_ticks_per_cycle() == 2 * DEFAULT_TICKS_PER_CYCLE,
            "clock did not update"
        );
        clock_0.cycle();

        // verify clock has now ticked 3 cycles
        assert!(
            clock_0.get_n_ticks() == 3 * DEFAULT_TICKS_PER_CYCLE,
            "clock has not ticked as expected"
        );

        let clock_1: &mut HashClock = &mut HashClock::new(
            DiemCryptoMessage(DEFAULT_HASH_TIME_INIT_MSG.to_string()).hash(),
            3 * DEFAULT_TICKS_PER_CYCLE,
        );
        // tick new clock 3 cycles
        clock_1.cycle();

        assert!(
            clock_0.get_hash_time() == clock_1.get_hash_time(),
            "clock hashes do not match after equal cycles"
        );

        // reset the clocks
        clock_0.update_hash_time(DiemCryptoMessage(DEFAULT_HASH_TIME_INIT_MSG.to_string()).hash());
        clock_1.update_hash_time(DiemCryptoMessage(DEFAULT_HASH_TIME_INIT_MSG.to_string()).hash());

        assert!(
            clock_0.get_hash_time() == clock_1.get_hash_time(),
            "clock hashes do not match after reset"
        );

        // tick clock for 1 mili
        clock_0.tick_for_interval(1);
        assert!(
            clock_0.get_hash_time() != clock_1.get_hash_time(),
            "assert clocks do not match after ticking clock 0 for 1 mili"
        );
    }
}
