use hangul_overlap::{OverlapPolicy, check_overlap, overs_of, unders_of};
use std::hint::black_box;

// Generated using AI.
// Profiling history
// Environment: Ryzen 5 7600, release build
//
// Commit    Duration    Notes
// 1e8143d   577 ms      Initial implementation
fn main() {
    for code in ('가' as u32)..=('나' as u32) {
        let ch = char::from_u32(code).unwrap();

        for over in overs_of(black_box(ch), OverlapPolicy::empty()).unwrap() {
            assert!(check_overlap(ch, over, OverlapPolicy::empty()).unwrap());
        }

        for under in unders_of(black_box(ch), OverlapPolicy::all()).unwrap() {
            assert!(check_overlap(under, ch, OverlapPolicy::all()).unwrap());
        }
    }
}