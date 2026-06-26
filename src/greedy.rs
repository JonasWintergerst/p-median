use::std::collections::{HashMap, HashSet, VecDeque};
use std::f64;
use std::time::Instant;

use crate::data::{City, Constraints, Solution};

/// Greedy p-median: repeatedly open the site that most reduces total distance,
/// then assign every hex to its nearest open site.
///
/// Greedy has no separate setup phase, so its `solve_time` is the whole
/// computation (mirrors the exact solver's solve-only timing).
pub fn solve(city: &City, p: usize, constraints: &Constraints) -> Solution {
    assert!(p <= city.len(), "more districts than hexes");

    let t = Instant::now();
    let open_sites = select_stations(city, p);

    // The repair passes work in district-index space: `district[h]` is the
    // index (0..p) into `open_sites` of the site currently serving hex h.
    let mut district = assign(city, &open_sites);

    let adj = build_adjacency(city);
    if constraints.contiguity {
        repair_contiguity(city, &mut district, &adj, &open_sites);
    }
    if let Some(limit) = constraints.workload_limit {
        repair_workload(city, &mut district, &adj, &open_sites, limit);
    }

    // Convert to the shared Solution convention: assignment[h] = open site hex.
    // The objective is computed afterwards so it reflects the repaired map.
    let assignment: Vec<usize> = district.iter().map(|&d| open_sites[d]).collect();
    let objective = (0..city.len())
        .map(|h| city.hexes[h].distance_to(&city.hexes[assignment[h]]))
        .sum();
    let solve_time = t.elapsed();

    Solution { open_sites, assignment, objective, solve_time }
}

pub fn select_stations(city: &City, p: usize) -> Vec<usize> {
    let n = city.len();
    let mut cost = vec![f64::INFINITY; n];
    let mut stations = Vec::with_capacity(p);

    for _ in 0..p {
        let best = (0..n)
            .filter(|c| !stations.contains(c))
            .min_by(|&a, &b| {
                objective_if_added(city, &cost, a)
                    .total_cmp(&objective_if_added(city, &cost, b))
            })
            .expect("no candidate left (is p > n?)");

        stations.push(best);

        for h in 0..n {
            let d = city.hexes[h].distance_to(&city.hexes[best]);
            if d < cost[h] { cost[h] = d; }
        }
    }

    stations
}

pub fn objective_if_added(city: &City, cost: &[f64], c: usize) -> f64 {
    (0..city.len())
        .map(|h| cost[h].min(city.hexes[h].distance_to(&city.hexes[c])))
        .sum()
}

pub fn assign(city: &City, stations: &[usize]) -> Vec<usize> {
    (0..city.len())
        .map(|h| {
            let hex = &city.hexes[h];
            (0..stations.len())
                .min_by(|&a, &b| {
                    hex.distance_to(&city.hexes[stations[a]])
                        .total_cmp(&hex.distance_to(&city.hexes[stations[b]]))
                })
                .unwrap()
        })
        .collect()
}

fn build_adjacency(city: &City) -> Vec<Vec<usize>> {
    let coord: HashMap<(usize, usize), usize> = city.hexes.iter()
        .enumerate()
        .map(|(i, h)| ((h.location()), i))
        .collect();

    const OFFSETS: [(i64, i64); 8] = [
        (-1,-1),(-1,0),(-1,1),
        ( 0,-1),       ( 0,1),
        ( 1,-1),( 1,0),( 1,1),
    ];

    city.hexes.iter().map(|h| {
        OFFSETS.iter().filter_map(|&(dx, dy)| {
            let (x, y) = h.location();
            let (nx, ny) = (x as i64 + dx, y as i64 + dy);
            if nx < 0 || ny < 0 { return None; }
            coord.get(&(nx as usize, ny as usize)).copied()
        }).collect()
    }).collect()
}

fn repair_contiguity(
    city: &City, 
    assignment: &mut Vec<usize>, 
    adj: &Vec<Vec<usize>>,
    stations: &[usize]
) {
    let n = assignment.len();

    for s in 0..stations.len() {
        let start = stations[s];

        let mut reachable: HashSet<usize> = HashSet::new();
        let mut queue: VecDeque<usize> = VecDeque::new();

        reachable.insert(start);
        queue.push_back(start);

        while let Some(i) = queue.pop_front() {
            for &j in &adj[i] {
                if assignment[j] == s && reachable.insert(j) {
                    queue.push_back(j);
                }
            }
        }

        for i in 0..n {
            if assignment[i] == s && !reachable.contains(&i) {
                if let Some(new_s) = adj[i].iter().find(|&&j| assignment[j] != s).map(|&j| assignment[j]) {
                    assignment[i] = new_s;
                }
            }
        }
    }
}


fn repair_workload(city: &City, assignment: &mut Vec<usize>, adj: &[Vec<usize>], stations: &[usize], limit: f64) {
    let (n, p) = (city.hexes.len(), stations.len());

    for _ in 0..2_000 {
        let loads = city.district_loads(assignment, p);
        let mean = loads.iter().sum::<f64>() / p as f64;
        if mean == 0.0 { break; }
        let max_load = loads.iter().cloned().fold(f64::MIN, f64::max);
        if max_load / mean <= limit { break; }

        let over = loads.iter().enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap().0;
        let under = loads.iter().enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .unwrap().0;
        if over == under { break; }

        let mut swapped = false;
        for i in 0..n {
            if assignment[i] != over { continue; }
            if i == stations[over] { continue; }

            let adj_to_under = adj[i].iter().any(|&j| assignment[j] == under);
            if !adj_to_under { continue; }

            if connected_without(assignment, adj, stations[over], i, over) {
                assignment[i] = under;
                swapped = true;
                break;
            }
        }
        if !swapped { break; }
    }
}

fn connected_without(assignment: &mut Vec<usize>, adj: &[Vec<usize>], start: usize, exclued: usize, district: usize) -> bool {
    
    let total = assignment.iter().filter(|&&d| d == district).count();
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::new();
    reachable.insert(start);
    queue.push_back(start);

    while let Some(i) = queue.pop_front() {
        for &j in &adj[i] {
            if j != exclued && assignment[j] == district && reachable.insert(j) {
                queue.push_back(j);
            }
        }
    }
    reachable.len() == total - 1
}