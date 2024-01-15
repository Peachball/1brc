mod solve_v1;
mod solve_v2;
mod solve_v3;

use solve_v3::solve_v3;
use solve_v2::solve_v2;
use solve_v1::solve_v1;
use std::{fs::File, io::Write};

pub const MEASUREMENTS: &'static str = "../measurements.txt";
pub const OUTPUT_FILE: &'static str = "/tmp/output.txt";

pub fn write_string_to_output(s: &str) {
  let mut output = File::create("/tmp/output.txt").unwrap();
  output.write_all(s.as_bytes()).unwrap();
}

fn main() {
  solve_v3();
}
