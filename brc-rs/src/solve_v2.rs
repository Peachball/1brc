use std::{collections::HashMap, fs::File, io::Read, sync::Arc, thread};

use ordered_float::NotNan;

use crate::{write_string_to_output, MEASUREMENTS};

/// Split file up into number of processes + use basic multithreading to solve
pub fn solve_v2() {
  let mut station_values: HashMap<String, Vec<NotNan<f32>>> = HashMap::new();
  println!("Reading...");
  let mut file = File::open(MEASUREMENTS).unwrap();
  let mut buffer = Vec::new();
  file.read_to_end(&mut buffer);

  println!("Calculating...");
  let num_processes = thread::available_parallelism().unwrap();

  // compute segment splits
  let mut segment_splits: Vec<usize> = vec![0];
  let segment_size = buffer.len() / num_processes;
  let mut threads = Vec::new();
  for i in 0..usize::from(num_processes) - 1 {
    let mut estimated_start = segment_size * (i + 1);
    while buffer[estimated_start] != b'\n' {
      estimated_start += 1;
    }
    segment_splits.push(estimated_start);
  }
  segment_splits.push(buffer.len() - 1);

  let buffer_ptr = Arc::new(buffer);
  for i in 0..usize::from(num_processes) {
    let thread_buffer_ptr = buffer_ptr.clone();
    let start_ind = segment_splits[i] + if i == 0 { 0 } else { 1 };
    let end_ind = segment_splits[i + 1];
    println!(
      "start segment: {}",
      std::str::from_utf8(&thread_buffer_ptr[start_ind..start_ind + 10]).unwrap()
    );
    threads.push(thread::spawn(move || {
      let mut counts: HashMap<String, Vec<NotNan<f32>>> = HashMap::new();
      let file_chunk = std::str::from_utf8(&thread_buffer_ptr[start_ind..end_ind]).unwrap();
      for line in file_chunk.lines() {
        let pieces: Vec<_> = line.split(";").collect();
        if pieces.len() != 2 {
          println!("weird parsing: {}", line);
        }
        counts
          .entry(pieces[0].to_string())
          .or_insert(Vec::new())
          .push(pieces[1].parse().unwrap());
      }
      counts
    }));
  }

  let mut station_values = HashMap::new();
  for t in threads {
    let partial_result = t.join().unwrap();
    for (key, mut value) in partial_result.into_iter() {
      station_values
        .entry(key)
        .or_insert(Vec::new())
        .append(&mut value);
    }
  }

  println!("threads: {}", num_processes);

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
