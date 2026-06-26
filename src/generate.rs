use crate::data::{City, Hex};

/// Build a rectangular `width` x `height` grid city.
///
/// Hexes are laid out row by row, every cell gets a hex with weight 1.0.
/// Ids are assigned in row-major order: `id = y * width + x`.
pub fn grid_city(width: usize, height: usize) -> City {
    let mut hexes = Vec::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            let id = y * width + x;
            hexes.push(Hex::new(id, (x, y)));
        }
    }

    City::new(hexes)
}

/// Convenience: a 10x10 city.
pub fn grid_city_10x10() -> City {
    grid_city(10, 10)
}

/// Build a grid city where each hex's weight is decided by `weight_fn`,
/// called with the cell coordinates `(x, y)`.
pub fn grid_city_weighted(
    width: usize,
    height: usize,
    mut weight_fn: impl FnMut(usize, usize) -> f64,
) -> City {
    let mut hexes = Vec::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            let id = y * width + x;
            let mut hex = Hex::new(id, (x, y));
            hex.weight = weight_fn(x, y);
            hexes.push(hex);
        }
    }

    City::new(hexes)
}

/// Build a grid city with pseudo-random weights drawn uniformly from
/// `[min_weight, max_weight)`, reproducible from `seed`.
///
/// Uses a small built-in PRNG so test data stays deterministic without
/// pulling in an external crate.
pub fn grid_city_random(
    width: usize,
    height: usize,
    min_weight: f64,
    max_weight: f64,
    seed: u64,
) -> City {
    let mut rng = SplitMix64::new(seed);
    let span = max_weight - min_weight;
    grid_city_weighted(width, height, |_, _| min_weight + rng.next_f64() * span)
}

/// Minimal SplitMix64 PRNG — deterministic and dependency-free.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A float in `[0, 1)`.
    fn next_f64(&mut self) -> f64 {
        // Use the top 53 bits for a uniform double.
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}
