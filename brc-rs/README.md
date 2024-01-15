Rust implementation of brc

## Benchmarks

On small refurbished laptop linux:
  - `calculate_average_royvanrijn.sh`: 29s
  - 1brc-simd (cpp solution) 8 threads: 24s
    - 1 thread: 34s
  - my solutions:
    - v1 + v2: impossible -- requires >12gb of memory

## Summary of techniques used by others

Technique:
  - use perf to check useful stats like
    - which instruction is taking long
    - cpu cache performance
  - mmap files
  - multithreading
  - custom hash table + fnv-a hashing
