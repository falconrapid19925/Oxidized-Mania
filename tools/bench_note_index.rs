//! Synthetic key-release selection only: no audio, GPU, event loop, or FPS claims.
//! rustc -O tools/bench_note_index.rs -o /tmp/bench-note-index
//! /tmp/bench-note-index
#[path = "../src/game/note_index.rs"]
mod note_index;

use note_index::HoldNoteIndex;
use std::{hint::black_box, time::Instant};

// Small synthetic fixture, not the runtime NoteRT memory layout.
struct Note {
    lane: usize,
    hold: bool,
    hit: bool,
    missed: bool,
    holding: bool,
    ghost: bool,
}

fn release(notes: &[Note], mut indices: impl Iterator<Item = usize>, lane: usize) -> Option<usize> {
    indices.find(|&i| {
        let n = &notes[i];
        n.holding && n.lane == lane && !n.hit && !n.missed && !n.ghost
    })
}

fn measure(mut operation: impl FnMut()) -> f64 {
    let start = Instant::now();
    operation();
    start.elapsed().as_secs_f64() * 1e6
}

fn main() {
    const COUNT: usize = 50_000;
    const QUERIES: usize = 512;
    println!("Synthetic {COUNT}-note charts; {QUERIES} queries per batch; median of 9; microseconds/batch");
    for (name, hold_every, empty_lane, active_holds) in [
        ("tap-only", 0, false, false),
        ("10% holds, none held", 10, false, false),
        ("all holds, none held", 1, false, false),
        ("empty lane", 10, true, false),
        ("held note nearby", 1, false, true),
    ] {
        let notes: Vec<_> = (0..COUNT).map(|i| {
            let hold = hold_every > 0 && i % hold_every == 0;
            Note { lane: i % if empty_lane { 3 } else { 4 }, hold,
                hit: false, missed: false, holding: active_holds && hold, ghost: false }
        }).collect();
        let build_start = Instant::now();
        let index = HoldNoteIndex::new(notes.iter().map(|n| (n.lane, n.hold)));
        let build_us = build_start.elapsed().as_secs_f64() * 1e6;
        let queries: Vec<_> = (0..QUERIES).map(|q| {
            ((q * 97) % COUNT, if empty_lane { 3 } else if active_holds { 0 } else { q % 4 })
        }).collect();
        for &(start, lane) in &queries {
            assert_eq!(release(&notes, start..notes.len(), lane), release(&notes, index.release_candidates(lane, start, notes.len()), lane));
        }
        let old_batch = || for &(start, lane) in &queries {
            black_box(release(black_box(&notes), black_box(start)..notes.len(), black_box(lane)));
        };
        let new_batch = || for &(start, lane) in &queries {
            black_box(release(black_box(&notes), black_box(&index).release_candidates(black_box(lane), black_box(start), notes.len()), lane));
        };
        old_batch();
        new_batch();
        let (mut old, mut new) = ([0.0; 9], [0.0; 9]);
        for i in 0..9 {
            // Alternate order to reduce systematic warm-cache bias.
            if i % 2 == 0 {
                old[i] = measure(old_batch);
                new[i] = measure(new_batch);
            } else {
                new[i] = measure(new_batch);
                old[i] = measure(old_batch);
            }
        }
        old.sort_by(f64::total_cmp);
        new.sort_by(f64::total_cmp);
        println!("{name}: build {build_us:.1} us; release {:.2} -> {:.2} us/batch ({:.3} -> {:.3} us/query); exact outputs match", old[4], new[4], old[4] / QUERIES as f64, new[4] / QUERIES as f64);
    }
}
