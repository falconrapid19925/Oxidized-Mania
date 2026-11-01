//! Immutable per-lane hold indexes for key-release lookup. Build after applying mirror mode.
//! Rebuild if note order, lanes, or types change. Hit/hold/ghost state stays live
//! in the caller; no cursor is cached, so restart and rewind need no reset.

#[derive(Debug)]
pub struct HoldNoteIndex {
    holds: [Vec<usize>; 4],
}

impl HoldNoteIndex {
    pub fn new(notes: impl IntoIterator<Item = (usize, bool)>) -> Self {
        let mut index = Self {
            holds: std::array::from_fn(|_| Vec::new()),
        };
        for (i, (lane, is_hold)) in notes.into_iter().enumerate() {
            if lane < 4 && is_hold {
                index.holds[lane].push(i);
            }
        }
        index
    }

    pub fn lane_holds(&self, lane: usize, active_idx: usize) -> &[usize] {
        Self::remaining(&self.holds, lane, active_idx)
    }

    /// A tap-only lane can return immediately. Otherwise preserve the cheap
    /// nearby-hold case before searching just this lane's remaining holds.
    pub fn release_candidates(&self, lane: usize, active_idx: usize, note_count: usize) -> impl Iterator<Item = usize> + '_ {
        let has_holds = self.holds.get(lane).is_some_and(|v| !v.is_empty());
        let end = if has_holds { active_idx.saturating_add(16).min(note_count) } else { active_idx };
        (active_idx..end).chain(
            std::iter::once_with(move || self.lane_holds(lane, end)).flatten().copied()
        )
    }

    fn remaining(lanes: &[Vec<usize>; 4], lane: usize, active_idx: usize) -> &[usize] {
        let Some(indices) = lanes.get(lane) else { return &[] };
        &indices[indices.partition_point(|&i| i < active_idx)..]
    }
}

#[cfg(test)]
mod tests {
    use super::HoldNoteIndex;

    #[test]
    fn preserves_chart_order_and_inclusive_active_boundary() {
        let index = HoldNoteIndex::new([(2, false), (0, true), (2, true), (2, true), (0, false)]);
        assert_eq!(index.lane_holds(2, 2), &[2, 3]);
        assert!(index.lane_holds(0, 2).is_empty());
        assert!(index.lane_holds(0, 5).is_empty());
        // A backward jump must restore candidates without resetting the index.
        assert_eq!(index.lane_holds(0, 0), &[1]);
    }

    #[test]
    fn empty_tap_only_and_unsupported_lanes() {
        let empty = HoldNoteIndex::new([]);
        assert!(empty.lane_holds(0, 0).is_empty());
        let taps = HoldNoteIndex::new((0..100_000).map(|i| (i % 4, false)));
        for lane in 0..4 {
            assert!(taps.lane_holds(lane, 0).is_empty());
        }
        let invalid = HoldNoteIndex::new([(7, true), (0, false)]);
        assert!(invalid.lane_holds(7, 0).is_empty());
        assert!(invalid.lane_holds(0, 0).is_empty());
    }

    #[test]
    fn prefix_boundary_and_mirrored_lanes_match_scan() {
        for mirrored in [false, true] {
            let notes: Vec<_> = (0..40).map(|i| {
                let lane = if mirrored { 3 - i % 4 } else { i % 4 };
                (lane, [0, 15, 16, 17, 32, 39].contains(&i))
            }).collect();
            let index = HoldNoteIndex::new(notes.iter().copied());
            for active in [0, 1, 15, 16, 17, 32, 39, 40, 0] {
                for lane in 0..4 {
                    let held = |i: &usize| notes[*i] == (lane, true);
                    let expected: Vec<_> = (active..notes.len()).filter(held).collect();
                    let actual: Vec<_> = index.release_candidates(lane, active, notes.len()).filter(held).collect();
                    assert_eq!(actual, expected);
                }
            }
        }
    }

    #[test]
    fn selection_matches_full_scan_with_live_states_and_rewinds() {
        // Deterministic mixed chart with multiple held notes in each lane.
        // State flags represent hits, misses, held notes and resume ghosts.
        let mut state = 0x1234_5678_u64;
        let mut random = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut notes: Vec<_> = (0..2048).map(|_| {
            let r = random();
            ((r % 4) as usize, r & 16 != 0, (r >> 8) as u8 & 15)
        }).collect();
        let index = HoldNoteIndex::new(notes.iter().map(|&(lane, hold, _)| (lane, hold)));
        for _ in 0..512 {
            // Change state without rebuilding the immutable index.
            let changed = random() as usize % notes.len();
            notes[changed].2 = random() as u8 & 15;
            let active = random() as usize % (notes.len() + 1);
            for lane in 0..4 {
                let held = |i: &usize| {
                    let (l, hold, flags) = notes[*i];
                    l == lane && hold && flags & 7 == 0 && flags & 8 != 0
                };
                assert_eq!(
                    index.release_candidates(lane, active, notes.len()).find(held),
                    (active..notes.len()).find(held)
                );
            }
        }
    }
}
