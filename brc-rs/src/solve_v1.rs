use std::{collections::HashMap, fs};

use ordered_float::NotNan;

use crate::write_string_to_output;

/// Simplest possible solution
pub fn solve_v1()  {
  let mut station_values: HashMap<String, Vec<NotNan<f32>>> = HashMap::new();
  println!("Reading...");
  let file_contents = fs::read_to_string("../measurements.txt").unwrap();
  println!("Calculating...");
  for line in file_contents.split("\n") {
    let components: Vec<_> = line.split(";").collect();
    if components.len() != 2 {
      break;
    }
    station_values
      .entry(components[0].to_string())
      .or_insert(Vec::new())
      .push(components[1].parse().unwrap());
  }
  println!("Summing");
  let mut station_keys: Vec<_> = station_values.keys().collect();
  station_keys.sort();
  let mut result = String::new();
  result.push_str("{");
  for (i, key) in station_keys.into_iter().enumerate() {
    let values = station_values.get(key).unwrap();
    let min = values.iter().min().unwrap();
    let avg = values.iter().sum::<NotNan<f32>>() / values.len() as f32;
    let max = values.iter().max().unwrap();
    if i != 0 {
      result.push_str(", ");
    }
    result.push_str(&format!("{}={:.1}/{:.1}/{:.1}", key, min, avg, max));
  }
  result.push_str("}\n");
  write_string_to_output(&result);
}
