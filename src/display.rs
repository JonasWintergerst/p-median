use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use crate::data::{City, Solution};

// Pointy-top hexagon geometry (in px).
const HEX_W: f64 = 46.0;
const HEX_H: f64 = HEX_W * 1.154_700_5; // width * 2/sqrt(3)
const ROW_STEP: f64 = HEX_H * 0.75; // vertical distance between rows

/// Render one board per `(label, solution)` to an HTML file and open it.
///
/// The boards are laid out side by side so the algorithms can be compared.
/// Each district gets its own colour; the station (median) of a district is
/// drawn with a ringed marker. Returns the path of the written file.
pub fn show_grid(city: &City, solutions: &[(&str, &Solution, Duration)]) -> std::io::Result<PathBuf> {
    let html = render_html(city, solutions);

    let path = std::env::temp_dir().join("p_median.html");
    fs::write(&path, html)?;

    if let Err(e) = webbrowser::open(&path.to_string_lossy()) {
        eprintln!("could not open browser ({e}); file written to {}", path.display());
    }

    Ok(path)
}

/// Build the HTML document as a string (no I/O — handy for tests).
///
/// Draws each `(label, solution)` as its own board inside a flex row.
pub fn render_html(city: &City, solutions: &[(&str, &Solution, Duration)]) -> String {
    let width = city.hexes.iter().map(|h| h.location.0).max().unwrap_or(0) + 1;
    let height = city.hexes.iter().map(|h| h.location.1).max().unwrap_or(0) + 1;

    // Container big enough to hold the offset rows (same for every board).
    let board_w = width as f64 * HEX_W + HEX_W / 2.0;
    let board_h = (height.saturating_sub(1)) as f64 * ROW_STEP + HEX_H;

    // Shared palette, sized for the board with the most districts.
    let p_max = solutions.iter().map(|(_, s, _)| s.open_sites.len()).max().unwrap_or(0);
    let mut palette_css = String::new();
    for d in 0..p_max {
        let _ = write!(palette_css, ".d{d}{{background:{};}}", hue(d, p_max));
    }

    let boards: String = solutions
        .iter()
        .map(|(label, sol, dur)| render_board(city, label, sol, *dur))
        .collect();

    format!(
        "<!doctype html>
<html lang=\"en\">
<head>
<meta charset=\"utf-8\">
<title>p-median — {width}×{height}, p={p_max}</title>
<style>
  body {{ font-family: system-ui, sans-serif; background:#111; color:#eee; padding:24px; }}
  h1 {{ font-size:18px; font-weight:600; }}
  h2 {{ font-size:15px; font-weight:600; margin:0 0 12px; }}
  .boards {{ display:flex; gap:48px; flex-wrap:wrap; align-items:flex-start; }}
  .board {{ position:relative; width:{board_w:.0}px; height:{board_h:.0}px; }}
  .hex {{
    position:absolute;
    width:{HEX_W:.0}px; height:{HEX_H:.1}px;
    clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
    display:flex; align-items:center; justify-content:center;
    font-size:11px; color:rgba(0,0,0,.6);
    box-sizing:border-box;
  }}
  .hex.station {{ filter:brightness(1.08); }}
  .dot {{
    position:absolute; width:11px; height:11px; border-radius:50%;
    background:#000; border:2px solid #fff; box-sizing:border-box;
  }}
  .lbl {{ opacity:.55; }}
  ul.legend {{ list-style:none; padding:0; margin:16px 0 0; font-size:13px; }}
  ul.legend li {{ margin:4px 0; display:flex; align-items:center; gap:8px; }}
  .swatch {{
    width:18px; height:18px; display:inline-block;
    clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
  }}
  .obj {{ margin-top:16px; color:#9ad; }}
  .time {{ margin-top:4px; color:#888; font-size:12px; }}
  {palette_css}
</style>
</head>
<body>
  <h1>p-median solution — {width}×{height} grid, p = {p_max}</h1>
  <div class=\"boards\">{boards}</div>
</body>
</html>"
    )
}

/// Render a single labelled board: coloured districts, ringed station markers,
/// a legend, the objective and the solver's wall-clock time.
fn render_board(city: &City, label: &str, solution: &Solution, elapsed: Duration) -> String {
    // open site (hex index) -> district index (its position within open_sites).
    let station_index: HashMap<usize, usize> = solution
        .open_sites
        .iter()
        .enumerate()
        .map(|(d, &s)| (s, d))
        .collect();

    // Absolutely position each hexagon; odd rows are shifted half a hex right.
    let mut cells = String::new();
    for (i, hex) in city.hexes.iter().enumerate() {
        let (x, y) = hex.location;
        let id = hex.id;
        let d = station_index[&solution.assignment[i]];
        let is_station = solution.open_sites[d] == i;

        let left = x as f64 * HEX_W + if y % 2 == 1 { HEX_W / 2.0 } else { 0.0 };
        let top = y as f64 * ROW_STEP;

        let station_class = if is_station { " station" } else { "" };
        let dot = if is_station { "<span class=\"dot\"></span>" } else { "" };
        let _ = write!(
            cells,
            "<div class=\"hex d{d}{station_class}\" style=\"left:{left:.1}px;top:{top:.1}px\" \
             title=\"hex {id} @ ({x},{y}) — district {d} — weight {weight:.2}\">\
             {dot}<span class=\"lbl\">{d}</span></div>",
            weight = hex.weight,
        );
    }

    let mut legend = String::new();
    for (d, &site) in solution.open_sites.iter().enumerate() {
        let loc = city.hexes[site].location;
        let _ = write!(
            legend,
            "<li><span class=\"swatch d{d}\"></span>district {d} — station hex {site} at ({}, {})</li>",
            loc.0, loc.1,
        );
    }

    format!(
        "<section class=\"panel\">\
           <h2>{label}</h2>\
           <div class=\"board\">{cells}</div>\
           <ul class=\"legend\">{legend}</ul>\
           <p class=\"obj\">objective: {obj:.3}</p>\
           <p class=\"time\">total time : {total_ms:.3} ms</p>\
           <p class=\"time\">solver time: {solve_ms:.3} ms</p>\
         </section>",
        obj = solution.objective,
        total_ms = elapsed.as_secs_f64() * 1e3,
        solve_ms = solution.solve_time.as_secs_f64() * 1e3,
    )
}

/// Evenly spaced hue around the colour wheel for district `d` of `p`.
fn hue(d: usize, p: usize) -> String {
    let h = (d as f64) * 360.0 / (p.max(1) as f64);
    format!("hsl({h:.0}, 62%, 58%)")
}
