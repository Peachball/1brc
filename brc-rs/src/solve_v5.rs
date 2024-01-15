use std::{
  collections::HashMap,
  fs::{self, File},
  io::{BufRead, BufReader, Read},
};

use ordered_float::NotNan;

use crate::{write_string_to_output, MEASUREMENTS};

struct Record {
  total: f32,
  min: f32,
  max: f32,
  num: usize,
}

const BUFSIZE: usize = 1024 * 4;
struct RawBufReader {
  buffer: [u8; BUFSIZE],
  file: File,
  current_ind: usize,
  last_read_size: usize,
  is_end: bool,
}

impl RawBufReader {
  fn new(file: File) -> Self {
    let mut result = Self {
      file,
      buffer: [0; BUFSIZE],
      current_ind: 0,
      last_read_size: 0,
      is_end: false,
    };
    result.refresh_buffer();
    result
  }

  fn has_next_line(&self) -> bool {
    !self.is_end
  }

  fn refresh_buffer(&mut self) {
    self.last_read_size = self.file.read(&mut self.buffer).unwrap();
    self.is_end = self.last_read_size == 0;
    self.current_ind = 0;
  }

  fn next_char(&mut self) -> u8 {
    // check if should read
    let result = self.buffer[self.current_ind];
    self.current_ind += 1;
    if self.current_ind >= self.last_read_size {
      self.refresh_buffer();
    }
    result
  }
}

fn parse_temperature(reader: &mut RawBufReader) -> f32 {
  let mut temperature_value: f64 = 0.0;
  let mut is_negative = false;
  let mut c = reader.next_char();
  while c != b'\n' {
    if c == b'-' {
      is_negative = true;
    } else if c != b'.' {
      temperature_value = (c - b'0') as f64 + 10.0 * temperature_value;
    }
    c = reader.next_char();
  }
  temperature_value /= 10.0;

  if is_negative {
    -temperature_value as f32
  } else {
    temperature_value as f32
  }
}

/// 1. custom bufreader removing allocations
/// 2. custom float parsing
pub fn solve_v5() {
  let mut station_values: HashMap<Vec<u8>, Record> = HashMap::new();
  let file = File::open(MEASUREMENTS).unwrap();
  let mut bufreader = RawBufReader::new(file);

  println!("Reading");

  while bufreader.has_next_line() {
    let mut c = bufreader.next_char();
    let mut station_name_buffer: [u8; 100] = [0; 100];
    let mut station_name_buffer_ind = 0;
    while c != b';' {
      station_name_buffer[station_name_buffer_ind] = c;
      station_name_buffer_ind += 1;
      c = bufreader.next_char();
    }

    let value = parse_temperature(&mut bufreader);

    let station_name = &station_name_buffer[..station_name_buffer_ind];
    let value_entry =
      station_values
        .entry(station_name.to_vec())
        .or_insert(Record {
          total: 0.0,
          min: f32::INFINITY,
          max: f32::NEG_INFINITY,
          num: 0,
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
    result.push_str(&format!(
      "{}={:.1}/{:.1}/{:.1}",
      std::str::from_utf8(key).unwrap(),
      min,
      avg,
      max
    ));
  }
  result.push_str("}\n");
  write_string_to_output(&result);
}
