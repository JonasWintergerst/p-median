mod data;
mod display;
mod generate;
mod greedy;
mod exact;

use std::time::Instant;

use crate::data::Constraints;
use crate::display::show_grid;
use crate::generate::grid_city_random;

fn main() {
    let city = grid_city_random(10, 10, 1.0, 5.0, 42);
    let p = 5;

    // Flip constraints here once — every solver reads the same config.
    let constraints = Constraints {
        contiguity: false,
        workload_limit: None, //Some(1.1),
    };

    // Two solutions to the same instance, one per algorithm; time each solver.
    let t = Instant::now();
    let greedy = greedy::solve(&city, p, &constraints);
    let greedy_time = t.elapsed();

    let t = Instant::now();
    let exact = exact::solve(&city, p, &constraints);
    let exact_time = t.elapsed();

    greedy.print("Greedy");
    exact.print("Exact");

    let boards = [
        ("Greedy", &greedy, greedy_time),
        ("Exact", &exact, exact_time),
    ];
    match show_grid(&city, &boards) {
        Ok(path) => println!("wrote {}", path.display()),
        Err(e) => eprintln!("failed to write grid: {e}"),
    }
}
