use std::time::Instant;

use good_lp::{constraint, default_solver, variable, variables, Expression, Solution as LpSolution, SolverModel};

use crate::data::{City, Constraints, Solution};

pub fn solve(city: &City, p: usize, constraints: &Constraints) -> Solution {
    let cost = city.matrix();
    let weights = city.weights();
    // `weights` will feed a weighted objective and `constraints` will gate the
    // optional constraints; both are plumbed through but not yet enforced.
    let _ = (&weights, constraints);

    let n = cost.len();
    let m = cost[0].len();

    let mut vars = variables!();

    // Diagonal x[j][j] is the binary "facility j is open" decision; the
    // off-diagonal assignment fractions stay continuous in [0, 1] (with the
    // open sites fixed by the integers, the assignment LP is already integral).
    let x: Vec<Vec<_>> = (0..n)
        .map(|i| {
            (0..m)
                .map(|j| {
                    if i == j {
                        vars.add(variable().binary())
                    } else {
                        vars.add(variable().min(0.0).max(1.0))
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let objective: Expression = (0..n)
        .flat_map(|i| (0..m).map(move |j| (i, j)))
        .map(|(i, j)| cost[i][j] * x[i][j])
        .sum();

    let mut model = vars.minimise(&objective).using(default_solver);

    for i in 0..n {
        let assigned: Expression = (0..m).map(|j| x[i][j]).sum();
        model = model.with(constraint!(assigned == 1));
    }

    for i in 0..n {
        for j in 0..m {
            model = model.with(constraint!(x[i][j] <= x[j][j]))
        }
    }

    let opened: Expression = (0..m).map(|j| x[j][j]).sum();
    model = model.with(constraint!(opened == p as f64));

    // Time only the solve itself — the model construction above is "setup".
    let t = Instant::now();
    let sol = model.solve().expect("solver failed");
    let solve_time = t.elapsed();

    let open_sites: Vec<usize> = (0..m).filter(|&j| sol.value(x[j][j]) > 0.5).collect();

    let assignment: Vec<usize> = (0..n)
        .map(|i| {
            (0..m)
                .max_by(|&a, &b| sol.value(x[i][a]).partial_cmp(&sol.value(x[i][b])).unwrap())
                .unwrap()
        })
        .collect();
 
    let objective = sol.eval(&objective);

    for j in 0..m {
        let v = sol.value(x[j][j]);
        if v.abs() > 1e-6 {
            println!("x[{j}{j}] = {v}");
        }
    }

    Solution { open_sites, assignment, objective, solve_time }
}