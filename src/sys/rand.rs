use std::fmt;
use std::ops::Range;
use std::time::{SystemTime, UNIX_EPOCH};

/// Shuffles `collection` in place.
pub fn shuffle<T>(rng: &mut Rng, collection: &mut [T], k: usize) {
    debug_assert!(
        collection.len() > k,
        "Cannot shuffle more items than exist in collection"
    );

    // fisher-yates shuffle
    for j in 0..k {
        let r = rng.u8_in(j as u8..collection.len() as u8) as usize;
        collection.swap(j, r);
    }
}

// ============================================ Rng ================================================

pub struct Rng {
    state: [u64; 4],
    buffer: u64,
    buffer_remaining: u8,
}

impl Rng {
    pub fn init() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Self::from_seed(seed)
    }

    pub fn from_seed(mut seed: u64) -> Self {
        // splitmix64 generator - credits to Sebastiano Vigna (vigna@acm.org)
        let mut next = || {
            seed = seed.wrapping_add(0x9e3779b97f4a7c15);

            let mut z = seed;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);

            z ^ (z >> 31)
        };

        Self {
            state: [next(), next(), next(), next()],
            buffer: 0,
            buffer_remaining: 0,
        }
    }

    pub fn from_state(state: [u64; 4]) -> Self {
        Self {
            state,
            buffer: 0,
            buffer_remaining: 0,
        }
    }

    fn u64(&mut self) -> u64 {
        // xoshiro256++ generator - credits to Sebastiano Vigna (vigna@acm.org)
        const R: u32 = 23;
        const A: u32 = 17;
        const B: u32 = 45;

        let result = self.state[0]
            .wrapping_add(self.state[3])
            .rotate_left(R)
            .wrapping_add(self.state[0]);

        let t = self.state[1] << A;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(B);

        result
    }

    pub fn u8(&mut self) -> u8 {
        if self.buffer_remaining == 0 {
            self.buffer = self.u64();
            self.buffer_remaining = 8;
        }
        let r = self.buffer as u8;
        self.buffer >>= 8;
        self.buffer_remaining -= 1;
        r
    }

    pub fn u8_in(&mut self, range: Range<u8>) -> u8 {
        let s = (range.end - range.start) as u16;

        // Lemire's method for unbiased numbers
        let mut x = self.u8() as u16;
        let mut m = x * s;
        let mut l = m;

        if l < s {
            let t = s.wrapping_neg() % s;
            while l < t {
                x = self.u8() as u16;
                m = x * s;
                l = m;
            }
        }

        range.start + (m >> 8) as u8
    }
}

impl fmt::Display for Rng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "seed: 0x{:x} 0x{:x} 0x{:x} 0x{:x}",
            self.state[0], self.state[1], self.state[2], self.state[3]
        )
    }
}
