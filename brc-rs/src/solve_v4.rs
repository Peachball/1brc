use std::{collections::HashMap, fs::{self, File}, io::{BufReader, BufRead}};

use ordered_float::NotNan;

use crate::{write_string_to_output, MEASUREMENTS};

struct Record {
  total: f32,
  min: f32,
  max: f32,
  num: usize,
}

/// Simplest solution but optimizes for memory
pub fn solve_v4()  {
  let mut station_values: HashMap<String, Record> = HashMap::new();
  let file = File::open(MEASUREMENTS).unwrap();
  let bufreader = BufReader::new(file);
  println!("Reading");
  for line in bufreader.lines() {
    let line = line.unwrap();
    let components: Vec<_> = line.split(";").collect();
    if components.len() != 2 {
      break;
    }
    let value = components[1].parse::<f32>().unwrap();
    let value_entry = station_values
      .entry(components[0].to_string())
      .or_insert(Record {
        total: 0.0,
        min: f32::INFINITY,
        max: f32::NEG_INFINITY,
        num: 0
      });
    value_entry.total += value;
    value_entry.min = value_entry.min.min(value);
    value_entry.max = value_entry.max.max(value);
    value_entry.num += 1;
  }
  println!("Summing");
  let mut station_keys: Vec<_> = station_values.keys().collect();
  station_keys.sort();
  let mut result = String::new();
  result.push_str("{");
  for (i, key) in station_keys.into_iter().enumerate() {
    let values = station_values.get(key).unwrap();
    let min = values.min;
    let avg = values.total / values.num as f32;
    let max = values.max;
    if i != 0 {
      result.push_str(", ");
    }
    result.push_str(&format!("{}={:.1}/{:.1}/{:.1}", key, min, avg, max));
  }
  result.push_str("}\n");
  write_string_to_output(&result);
}
