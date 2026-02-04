use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rng([u64; 4]);

impl Rng {
    pub fn init() -> Self {
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);

        let h0 = hasher.finish();
        h0.hash(&mut hasher);

        let h1 = hasher.finish();
        h1.hash(&mut hasher);

        let h2 = hasher.finish();
        h2.hash(&mut hasher);

        let h3 = hasher.finish();

        Self([h3, h2, h1, h0])
    }

    pub fn from_seed(seed: [u64; 4]) -> Self {
        Self(seed)
    }

    pub fn u64(&mut self) -> u64 {
        // SAFETY Iterator will never return None
        self.next().expect("RNG is infallible")
    }

    /// Returns random value in range `0..n`.
    pub fn next_bounded(&mut self, s: u64) -> u64 {
        // Implements Lemire's method for unbiased random number generation

        let mut x = self.u64();
        let mut m = (x as u128) * (s as u128);
        let mut l = m as u64;

        if l < s {
            let t = s.wrapping_neg() % s;
            while l < t {
                x = self.u64();
                m = (x as u128) * (s as u128);
                l = m as u64;
            }
        }

        (m >> 64) as u64
    }
}

impl fmt::Display for Rng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "seed: 0x{:x} 0x{:x} 0x{:x} 0x{:x}",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

impl Iterator for Rng {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        // xoshiro256++
        let result = self.0[0]
            .wrapping_add(self.0[3])
            .rotate_left(23)
            .wrapping_add(self.0[0]);

        let t = self.0[1] << 17;

        self.0[2] ^= self.0[0];
        self.0[3] ^= self.0[1];
        self.0[1] ^= self.0[2];
        self.0[0] ^= self.0[3];

        self.0[2] ^= t;

        self.0[3] = self.0[3].rotate_left(45);

        Some(result)
    }
}
