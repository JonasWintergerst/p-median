
use std::time::Duration;

#[derive(Debug)]
pub struct City {
    pub hexes: Vec<Hex>,
}

#[derive(Debug, Clone)]
pub struct Hex {
    pub id: usize,
    pub location: (usize, usize),
    pub weight: f64
}

/// A p-median solution in the shared shape returned by every algorithm.
///
/// * `open_sites` — the hex indices chosen as stations (the open medians).
/// * `assignment` — `assignment[h]` is the open site hex `h` is served by.
/// * `objective`  — total assignment cost (sum of distances).
/// * `solve_time` — wall-clock time the solver itself spent (no setup).
#[derive(Debug)]
pub struct Solution {
    pub open_sites: Vec<usize>,
    pub assignment: Vec<usize>,
    pub objective: f64,
    pub solve_time: Duration,
}

impl Solution {
    /// Print the solution in the shared `open sites / assignment / cost` format.
    pub fn print(&self, label: &str) {
        println!("== {label} ==");
        println!("Open sites : {:?}", self.open_sites);
        println!("Assignment : {:?}", self.assignment);
        println!("Total cost : {}", self.objective);
        println!("Solver time: {:.3} ms", self.solve_time.as_secs_f64() * 1e3);
    }
}

/// Toggles for the optional constraints, shared by every solver so they can be
/// flipped in one place (`main`) and take effect across all of them at once.
#[derive(Debug, Clone, Default)]
pub struct Constraints {
    /// Reassign stranded hexes so each district stays contiguous.
    pub contiguity: bool,
    /// Balance district workloads until `max_load / mean_load <= limit`.
    /// `None` disables the pass.
    pub workload_limit: Option<f64>,
}

impl City {
    pub fn district_loads(&self, assignment: &[usize], p: usize) -> Vec<f64> {
        let mut loads = vec![0.0; p];
        for (h, &d) in assignment.iter().enumerate() {
            loads[d] += self.hexes[h].weight;
        }
        loads
    }

    pub fn matrix(&self) -> Vec<Vec<f64>> {

        let n = self.len();
        let mut matrix: Vec<Vec<f64>> = vec![vec![0.0_f64; n]; n];

        for i in 0..n {
            for j in 0..n {
                let hex_i = &self.hexes[i];
                let hex_j = &self.hexes[j];
                matrix[i][j] = hex_i.distance_to(&hex_j);
            }
        }

        matrix
    }

    pub fn weights(&self) -> Vec<f64> {
        let mut weights: Vec<f64> = Vec::new();
        for h in &self.hexes {
            weights.push(h.weight());
        } 
        weights
    }

    pub fn new(hexes: Vec<Hex>) -> Self {
        City { hexes }
    }

    pub fn is_empty(&self) -> bool {
        self.hexes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.hexes.len()
    }
}

impl Hex {
    pub fn new(id: usize, location: (usize, usize)) -> Self {
        Hex { id, location, weight: 1.0 }
    }

    pub fn location(&self) -> (usize, usize) {
        self.location
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn distance_to(&self, other: &Hex) -> f64 {
        let dx = self.location.0 as f64 - other.location.0 as f64;
        let dy = self.location.1 as f64 - other.location.1 as f64;
        (dx * dx + dy * dy).sqrt()
    }
}




