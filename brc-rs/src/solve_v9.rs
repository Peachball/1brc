use std::fs::File;

use memmap2::{Mmap, MmapOptions};

use crate::{
  write_string_to_output, MAX_STATION_NAMES, MAX_STATION_NAME_LEN, MEASUREMENTS,
};

#[derive(Copy, Clone, Debug)]
struct Record {
  total: i32,
  min: i32,
  max: i32,
  num: usize,
}

impl Record {
  fn new() -> Self {
    Self {
      total: 0,
      min: i32::MAX,
      max: i32::MIN,
      num: 0,
    }
  }
}

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

fn parse_temperature(reader: &mut RawBufReader) -> i32 {
  let is_negative = reader.mmap[reader.current_ind] == b'-';
  if is_negative {
    reader.current_ind += 1;
  }

  let (a, b, c, d, increment) = if reader.mmap[reader.current_ind + 1] == b'.' {
    (
      0,
      0,
      reader.mmap[reader.current_ind] - b'0',
      reader.mmap[reader.current_ind + 2] - b'0',
      4,
    )
  } else if reader.mmap[reader.current_ind + 2] == b'.' {
    (
      0,
      reader.mmap[reader.current_ind] - b'0',
      reader.mmap[reader.current_ind + 1] - b'0',
      reader.mmap[reader.current_ind + 3] - b'0',
      5,
    )
  } else {
    (
      reader.mmap[reader.current_ind] - b'0',
      reader.mmap[reader.current_ind + 1] - b'0',
      reader.mmap[reader.current_ind + 2] - b'0',
      reader.mmap[reader.current_ind + 4] - b'0',
      6,
    )
  };
  let temperature_value: i32 =
    1000 * a as i32 + 100 * b as i32 + 10 * c as i32 + d as i32;
  reader.current_ind += increment;

  if is_negative {
    -temperature_value
  } else {
    temperature_value
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
  let mut key = [0u8; 8];
  let l = value.len().min(8);
  key[..l].copy_from_slice(&value[..l]);
  key[0] ^= value.len() as u8;
  (u64::from_le_bytes(key) * FNV_PRIME) as usize
}

struct FixedSizeMap {
  names: [u8; MAP_NAME_SIZE],
  entries: [MapKvPair; MAP_ENTRIES],
  last_name_idx: usize,
}

impl FixedSizeMap {
  fn new() -> Self {
    Self {
      names: [0; MAP_NAME_SIZE],
      entries: [MapKvPair::new(); MAP_ENTRIES],
      last_name_idx: 0,
    }
  }

  fn get_or_insert(&mut self, name: &[u8]) -> usize {
    let hash = fnv_hash(name);
    let mut idx = hash % self.entries.len();
    loop {
      let cur_entry = &mut self.entries[idx];
      if cur_entry.key.start == 0 && cur_entry.key.end == 0 {
        let name_start = self.last_name_idx;
        let name_end = self.last_name_idx + name.len();
        cur_entry.key.start = name_start;
        cur_entry.key.end = name_end;
        for i in 0..name.len() {
          self.names[i + name_start] = name[i];
        }
        self.last_name_idx = name_end;
        return idx;
      }
      let entry = &self.entries[idx];

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
      let entry = &self.entries[idx];

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
      .filter(|e| e.key.start != e.key.end)
      .map(|e| &self.names[e.key.start..e.key.end])
      .collect()
  }
}

/// remove options in hashmap (as_mut() apparently taking 3% of time?)
pub fn solve_v9() {
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
    let value_entry = &mut station_values.entries[value_idx].value;
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
    let min = values.min as f32 / 10.0;
    let avg = values.total as f32 / 10.0 / values.num as f32;
    let max = values.max as f32 / 10.0;
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
