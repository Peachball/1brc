Rust implementation of brc

## Benchmarks

On small refurbished laptop linux:
  - `calculate_average_royvanrijn.sh`: 29s
  - 1brc-simd (cpp solution) 8 threads: 24s
    - 1 thread: 34s
  - my solutions:
    - v1 + v2: impossible -- requires >12gb of memory
    - v3 (4 threads + memmap): still impossible -- memmap pattern + internally
      used data structures seem to be increasing memory too much
    - v4 (1 thread + normal reader + not stupid way of storing partial sums):
      253s
    - v5 (try manual parsing + custom buffered reader): 121s
    - v6 (custom hashmap): 64s
    - v7 (use i32 for parsing): 65s

## Summary of techniques used by others

Technique:
  - avoid memory allocations/string/utf8
  - custom hash table + fnv-a hashing
  - mmap files
  - multithreading
  - use perf
    - `perf annotate` + `perf report` can check which specific instructions were
      run the most
    - use `perf stat -d` to check useful stats like
      - which instruction is taking long
      - cpu cache performance
      * useful guide [1]

## References

[1]: https://rust-lang.github.io/packed_simd/perf-guide/prof/linux.html
