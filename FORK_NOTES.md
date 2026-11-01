# First stability and loading pass

This fork remains a 4K mania game. It is not a full osu!stable port, and no
stable gameplay parity or input/audio latency improvements have been established.

## Changes

- Key mapping ignores bindings beyond the four supported lanes. Previously an
  extra named or numeric binding could return index 4 or higher to gameplay
  code which indexes four-element arrays.
- Map conversion finds the active inherited timing point with upper-bound
  binary search rather than a reverse linear scan per note. This changes lookup
  cost from O(notes × timing points) to O(notes × log(timing points)). The
  existing stable sorting, default speed and last-entry-wins behaviour at
  duplicate timestamps are preserved. It also works for unsorted note queries.
- Added a headless test harness compiling the production modules directly,
  and a GitHub Actions job. It needs no BASS binaries, window or GPU.

## Validation

```sh
cargo check --locked --tests
cargo test --locked --manifest-path tests/headless/Cargo.toml
rustc -O tools/bench_chart_timing.rs -o /tmp/bench-chart-timing
/tmp/bench-chart-timing
```

Seven regression tests cover key mapping limits and precedence, empty timelines,
negative times, exact boundaries, duplicate timing points, and comparison against
the original lookup over 5,000 queries.

One Linux cloud run with Rust 1.99.1, 10,000 synthetic timing points and 20,000
note queries, median of seven runs:

| Lookup | Time |
| --- | ---: |
| Original reverse scan | 114.159 ms |
| Upper-bound search | 0.712 ms |

Checksums matched. This is an isolated synthetic lookup benchmark, not measured
end-to-end map loading, FPS, frame pacing or input-to-screen latency. Sparse-SV
maps may see little difference. Results vary by hardware and compiler.

The full application was type-checked, not linked or played. Native BASS/BASS_FX
binaries and a suitable display/audio environment are still required for that.
Existing upstream warnings remain. Next validation should cover real charts,
long-note judgements and replay consistency before changing gameplay timing.

The upstream README declares MIT licensing; a standalone source license file
is absent. Bundled music, skins, fonts and native audio libraries have their own
provenance/terms and are not relicensed by this fork.
