/// Find the last timing point at or before a note, in a time-sorted timeline.
/// Upper-bound lookup preserves the legacy last-entry-wins rule for equal times.
pub fn active_scroll_speed(timeline: &[(f64, f32)], time_ms: f64) -> f32 {
    let end = timeline.partition_point(|(time, _)| *time <= time_ms);
    end.checked_sub(1).map(|i| timeline[i].1).unwrap_or(1.0)
}

#[cfg(test)]
mod tests {
    use super::active_scroll_speed;

    fn legacy_lookup(timeline: &[(f64, f32)], time: f64) -> f32 {
        timeline.iter().rev().find(|(t, _)| *t <= time)
            .map(|(_, speed)| *speed).unwrap_or(1.0)
    }

    #[test]
    fn empty_and_before_first_use_default() {
        assert_eq!(active_scroll_speed(&[], 0.0), 1.0);
        assert_eq!(active_scroll_speed(&[(100.0, 2.0)], 99.0), 1.0);
    }

    #[test]
    fn exact_boundaries_and_duplicate_times_match_legacy() {
        let timeline = [(-100.0, 0.5), (0.0, 1.0), (100.0, 2.0), (100.0, 3.0)];
        for time in [-101.0, -100.0, -0.01, 0.0, 99.999, 100.0, 1e9] {
            assert_eq!(active_scroll_speed(&timeline, time), legacy_lookup(&timeline, time));
        }
        assert_eq!(active_scroll_speed(&timeline, 100.0), 3.0);
    }

    #[test]
    fn large_timeline_matches_legacy_for_unsorted_note_queries() {
        let timeline: Vec<_> = (0..4096).map(|i| ((i / 2) as f64, (i % 7 + 1) as f32 / 2.0)).collect();
        for i in (0..5000).rev() {
            let time = i as f64 * 0.5 - 100.0;
            assert_eq!(active_scroll_speed(&timeline, time), legacy_lookup(&timeline, time));
        }
    }
}
