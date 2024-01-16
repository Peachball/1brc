mod solve_v1;
mod solve_v2;
mod solve_v3;
mod solve_v4;
mod solve_v5;
mod solve_v6;
mod solve_v7;
mod solve_v8;
mod solve_v9;
mod solve_v10;

use solve_v10::solve_v10;
use solve_v9::solve_v9;
use solve_v8::solve_v8;
use solve_v7::solve_v7;
use solve_v6::solve_v6;
use solve_v5::solve_v5;
use solve_v4::solve_v4;
use solve_v3::solve_v3;
use solve_v2::solve_v2;
use solve_v1::solve_v1;
use std::{fs::File, io::Write};

pub const MEASUREMENTS: &'static str = "../measurements.txt";
pub const OUTPUT_FILE: &'static str = "/tmp/output.txt";
pub const MAX_STATION_NAMES: usize = 10000;
pub const MAX_STATION_NAME_LEN: usize = 100;

pub fn write_string_to_output(s: &str) {
  let mut output = File::create("/tmp/output.txt").unwrap();
  output.write_all(s.as_bytes()).unwrap();
}

fn main() {
  solve_v10();
}
