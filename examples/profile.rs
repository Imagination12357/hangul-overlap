use hangul_overlap::{OverlapPolicy, check_overlap, overs_of, unders_of};
use std::hint::black_box;

// Generated using AI.
// Profiling history
// Environment: Ryzen 5 7600, release build
//
// Commit    Duration    Cycles    Notes
// 1e8143d   577 ms      1         Initial implementation
// 59845bb   494 ms      100       LazyLock bimap cache
// 2011751   404 ms      100       Avoid cloning in iproduct!
// 1b83642   347 ms      100       Use vec for inner bimap
// 07827be   309 ms      100       multi_bimap -> FxHashMap-backed overlap map
fn main() {
    for _ in 0..100{
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
}