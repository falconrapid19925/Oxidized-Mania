//! Isolated lookup benchmark, not a gameplay/FPS benchmark.
//! rustc -O tools/bench_chart_timing.rs -o /tmp/bench-chart-timing
//! /tmp/bench-chart-timing
#[path = "../src/chart_timing.rs"]
mod chart_timing;

use std::{hint::black_box, time::Instant};

fn legacy(timeline: &[(f64, f32)], time: f64) -> f32 {
    timeline.iter().rev().find(|(t, _)| *t <= time)
        .map(|(_, speed)| *speed).unwrap_or(1.0)
}

fn measure(timeline: &[(f64, f32)], notes: &[f64], lookup: fn(&[(f64, f32)], f64) -> f32) -> (u128, f64) {
    let start = Instant::now();
    let sum = notes.iter().map(|time| black_box(lookup(black_box(timeline), black_box(*time))) as f64).sum();
    (start.elapsed().as_nanos(), sum)
}

fn main() {
    let timeline: Vec<_> = (0..10_000).map(|i| (i as f64, (i % 7 + 1) as f32 / 2.0)).collect();
    let notes: Vec<_> = (0..20_000).map(|i| i as f64 / 2.0).collect();
    let mut old = Vec::new();
    let mut new = Vec::new();
    for _ in 0..7 {
        let (old_ns, old_sum) = measure(&timeline, &notes, legacy);
        let (new_ns, new_sum) = measure(&timeline, &notes, chart_timing::active_scroll_speed);
        assert_eq!(old_sum, new_sum);
        old.push(old_ns);
        new.push(new_ns);
    }
    old.sort_unstable();
    new.sort_unstable();
    println!("Synthetic: {} timing points / {} notes; median of 7 runs", timeline.len(), notes.len());
    println!("legacy reverse scan: {:.3} ms", old[3] as f64 / 1e6);
    println!("upper-bound lookup: {:.3} ms", new[3] as f64 / 1e6);
    println!("lookup-only speedup: {:.2}x; checksums equal", old[3] as f64 / new[3] as f64);
}
