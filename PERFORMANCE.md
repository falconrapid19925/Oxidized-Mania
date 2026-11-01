# Measured gameplay CPU work reduction

This pass follows PR #1. It preserves the existing judgement predicates and
timestamps. It does not establish osu!stable parity or an end-to-end latency gain.

## Changes

- Key releases previously scanned every remaining note until finding a held note
  in the requested lane. A tap-only chart took a full scan on every release.
  Build an immutable index of hold-note positions per lane after mirroring.
  A lane without holds returns immediately. Other lanes scan at most 16 adjacent
  notes first, then seek into their own hold index. This retains the inexpensive
  nearby-hold case and the original first-match order.
- Hit/miss/holding/ghost flags remain live in the original notes. The index stores
  no advancing cursor, so restart and backwards active-index changes need no
  reset. Rebuild it if note order, type, or lane assignments change.
- Key presses keep the existing selection code. Only key-release lookup changes.
- The existing frame profiler performed clock reads and shifted a rolling sample
  vector even with reporting disabled. Normal builds now compile those operations
  out. Opt in with `--features frame-profiler` to collect and print the existing
  section timings every 120 frames. Diagnostic output itself can disturb timing.

Index construction is O(notes), with O(hold notes) additional storage. A tap-only
chart needs no index heap allocation. For other lanes, release lookup costs a
bounded prefix plus O(log(lane holds) + remaining lane holds) in the worst case.
It is not constant-time on hold-heavy charts.

## Validation

```sh
cargo test --locked --manifest-path tests/headless/Cargo.toml
cargo check --locked --tests
cargo check --locked --tests --features frame-profiler
rustc -O tools/bench_note_index.rs -o /tmp/bench-note-index
/tmp/bench-note-index
```

All 11 headless tests passed locally. Four new tests cover empty/tap-only charts,
inclusive boundaries, the prefix/index transition, mirrored lanes, backwards
positions, and state changes. Deterministic comparisons check 2,048 lane queries
against the original full-scan selection. Both full-app type-check configurations
passed with the same 57 pre-existing warnings.

One Linux cloud run, Rust 1.99.1, 50,000-note synthetic fixtures, 512 release queries
per batch, median of nine batches with alternating old/new measurement order:

| Fixture | Original (microseconds/query) | Indexed (microseconds/query) |
| --- | ---: | ---: |
| Tap-only | 17.848 | 0.006 |
| 10% holds, none currently held | 19.676 | 0.667 |
| All holds, none currently held | 18.344 | 5.779 |
| Empty requested lane | 22.026 | 0.006 |
| Held note nearby | 0.004 | 0.006 |

All selected note indices matched exactly before timing. The nearby-hold case
adds roughly a nanosecond in this run; report it alongside the favorable cases.
These use compact synthetic note records, not the full runtime NoteRT layout.
Numbers vary with hardware, layout and cache state. They measure selection only,
excluding index construction, event dispatch, judgement, audio and rendering.
They are not FPS or input-to-photon measurements. No numeric gain is claimed for
compiling out the profiler, because whole-frame timing has not been measured.

The native application was type-checked, not linked or playtested. Long-note,
pause/restart, mirror and replay parity still need native validation before release.

## Next performance work

1. Establish native baselines on representative real charts: normal, dense chords,
   overlapping long notes, heavy SV, and a large song library. Record cold/warm
   loading, CPU and presentation timing, frame-time p50/p95/p99, allocations and
   memory. Existing section totals do not measure complete event-loop frame pacing
   or input-to-photon latency.
2. Profile song-library construction. It currently loads and analyzes each chart
   serially. Evaluate metadata/difficulty caches with correct invalidation before
   parallelizing; the existing ROX cache uses sanitized file stems, which can
   collide between folders and must not be relied on as unique cache identities.
3. Inspect repeated skin lookups, text work and allocations in gameplay. Retain
   only changes that improve representative frame-time distributions.
4. Compare presentation modes and queue depth on target hardware. Keep visual
   latency, tearing and frame pacing explicit; higher FPS alone is not sufficient.
5. Evaluate LTO/PGO only after the algorithmic work, using the same recorded workload
   and judgement/replay checks. No fixed speedup percentage is promised.
