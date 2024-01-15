use std::{
  collections::HashMap,
  fs::{self, File},
  io::{BufRead, BufReader, Read},
};

use memmap2::{Mmap, MmapOptions};
use ordered_float::NotNan;

use crate::{
  write_string_to_output, MAX_STATION_NAMES, MAX_STATION_NAME_LEN, MEASUREMENTS,
};

#[derive(Copy, Clone)]
struct Record {
  total: f32,
  min: f32,
  max: f32,
  num: usize,
}

impl Record {
  fn new() -> Self {
    Self {
      total: 0.0,
      min: f32::INFINITY,
      max: f32::NEG_INFINITY,
      num: 0,
    }
  }
}

const BUFSIZE: usize = 1024 * 4;
struct RawBufReader {
  current_ind: usize,
  mmap: Mmap,
}

impl RawBufReader {
  fn new(file: File) -> Self {
     Self {
      mmap: unsafe { MmapOptions::new().map(&file).unwrap() },
      current_ind: 0,
    }
  }

  fn has_next_line(&self) -> bool {
    self.mmap.len() > self.current_ind
  }

  #[inline]
  fn next_char(&mut self) -> u8 {
    // check if should read
    let result = self.mmap[self.current_ind];
    self.current_ind += 1;
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

#[derive(Clone, Copy)]
struct MapStrRef {
  start: usize,
  end: usize,
}

#[derive(Clone, Copy)]
struct MapKvPair {
  key: MapStrRef,
  value: Record,
}

impl MapKvPair {
  fn new() -> Self {
    Self {
      key: MapStrRef { start: 0, end: 0 },
      value: Record::new(),
    }
  }
}

const MAP_LOAD_FACTOR: f64 = 2.0;
const MAP_NAME_SIZE: usize = MAX_STATION_NAMES * MAX_STATION_NAME_LEN;
const MAP_ENTRIES: usize =
  (MAX_STATION_NAMES as f64 * MAP_LOAD_FACTOR) as usize;

/// fnv a hash
fn fnv_hash(value: &[u8]) -> usize {
  const FNV_PRIME: u64 = 1099511628211;
  const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
  let mut result = FNV_OFFSET_BASIS;

  for b in value {
    result = result ^ *b as u64;
    result *= FNV_PRIME;
  }

  result as usize
}

struct FixedSizeMap {
  names: [u8; MAP_NAME_SIZE],
  entries: [Option<MapKvPair>; MAP_ENTRIES],
  last_name_idx: usize,
}

impl FixedSizeMap {
  fn new() -> Self {
    Self {
      names: [0; MAP_NAME_SIZE],
      entries: [None; MAP_ENTRIES],
      last_name_idx: 0,
    }
  }

  fn get_or_insert(&mut self, name: &[u8]) -> usize {
    let hash = fnv_hash(name);
    let mut idx = hash % self.entries.len();
    loop {
      if self.entries[idx].is_none() {
        let name_start = self.last_name_idx;
        let name_end = self.last_name_idx + name.len();
        let key = MapStrRef {
          start: name_start,
          end: name_end,
        };
        for i in 0..name.len() {
          self.names[i + name_start] = name[i];
        }
        self.last_name_idx = name_end;
        let kvpair = MapKvPair {
          key,
          value: Record::new(),
        };
        self.entries[idx] = Some(kvpair);
        return idx;
      }
      let mut entry = self.entries[idx].unwrap();

      // linear probing
      if &self.names[entry.key.start..entry.key.end] != name {
        idx = (idx + 1) % self.entries.len();
        continue;
      }

      return idx;
    }
  }

  fn get(&self, name: &[u8]) -> Record {
    let hash = fnv_hash(name);
    let mut idx = hash % self.entries.len();
    loop {
      let entry = &self.entries[idx].unwrap();

      // linear probing
      if &self.names[entry.key.start..entry.key.end] != name {
        idx = (idx + 1) % self.entries.len();
        continue;
      }

      return entry.value;
    }
  }

  fn keys(&self) -> Vec<&[u8]> {
    self
      .entries
      .iter()
      .filter_map(|e| e.as_ref())
      .map(|e| &self.names[e.key.start..e.key.end])
      .collect()
  }
}

/// use mmap
pub fn solve_v8() {
  let mut station_values = FixedSizeMap::new();
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
    let value_idx = station_values.get_or_insert(station_name);
    let value_entry =
      &mut station_values.entries[value_idx].as_mut().unwrap().value;
    value_entry.total += value;
    value_entry.min = value_entry.min.min(value);
    value_entry.max = value_entry.max.max(value);
    value_entry.num += 1;
  }
  println!("Summing");
  let mut station_keys: Vec<_> = station_values.keys();
  station_keys.sort();
  let mut result = String::new();
  result.push_str("{");
  for (i, key) in station_keys.iter().enumerate() {
    let values = station_values.get(key);
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
