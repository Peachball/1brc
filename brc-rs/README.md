Rust implementation of brc

## Benchmarks

On big gtx
  - `calculate_average_royvanrijn.sh`: 4.88s
  - 1brc-simd (cpp solution)
    - 32 threads: 0.48s
    - 8 threads: 1.32s
    - 1 thread: 8.79s
  - reference cheating rust solution [2]: 0.307s
    - 1 thread: 3.90s
  - my solutions:
    - v3: 6.96s
    - v4: 75.67
    - v5: 37.27
    - v6: 20.8s
    - v7: 19.94
    - v8: 18.9s
    - v9: 20.07s
    - v10 (memchr crate): 16.82s - 17.19s
  - my basic single threaded cpp solution:

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
    - v8 (based off v6 + use mmap): 58s
    - v9 (do stuff based off perf)
      - remove option based on perf report: 58.4s
      - use i16 everywhere: 56.8s
      - change hashing function: 53.5s
      - hardcoded parsing: 50s

## Summary of techniques used by others

Technique, from most influential to least:
  - avoid memory allocations/string/utf8
  - custom hash table + fnv-a hashing
  - multithreading
  - mmap files
  - use perf
    - `perf record` + `perf report` can check which specific instructions were
      run the most
      - determine instruction is taking long
      - can also measure other cpu statistics + which instructions cause them
        (see `perf list` for complete list). In particular, other cool events
        are:
        - branch-misses
        - cache-misses
    - use `perf stat -d` to check useful stats like
      - cpu cache performance
      - branch misses
      * useful guide [1]

Notes
  - mmaping doesn't really help on small laptop -- small laptop has slow cpu +
    slow memory which is main bottleneck. larger computers will effectively
    store entire file in memory


## References

[1]: https://rust-lang.github.io/packed_simd/perf-guide/prof/linux.html
[2]: https://curiouscoding.nl/posts/1brc/
