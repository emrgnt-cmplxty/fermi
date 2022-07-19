use std::{fmt::Debug};

use diem_crypto::{
    hash::{CryptoHash, HashValue},
};
use types::{
    spot::{DiemCryptoMessage},
};

#[derive(Debug)]
pub struct HashClock {
    time: HashValue,
    n_ticks: u64
} 
impl HashClock {
    pub fn new() -> Self {
        HashClock {
            time: DiemCryptoMessage(String::from("HashClock")).hash(),
            n_ticks: 0
        }
    }

    pub fn tick(&mut self, ticks: u64) {
        let init_tick: u64 = self.n_ticks;
        while self.n_ticks < init_tick + ticks {
            self.time = DiemCryptoMessage(self.time.to_string()).hash();
            self.n_ticks += 1;
        }
    }

    pub fn get_time(&self) -> HashValue {
        self.time
    }

    pub fn get_n_ticks(&self) -> u64 {
        self.n_ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clock() {
        let mut clock: HashClock = HashClock::new();
        println!("Clock={:?}", clock);
        clock.tick(100);
        println!("Clock={:?}", clock);
    }
}