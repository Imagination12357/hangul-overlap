mod helpers;
mod data;
mod overlap_map;
use data::{
    ALWAYS,
    DOUBLE_CONSONANT,
    COMPLEX_CONSONANT,
    ASYM_VOWEL,
};
use bitflags::bitflags;
use itertools::{iproduct, Itertools};
use std::sync::LazyLock;
use crate::helpers::{HangulJamo, IteratorExt, compose_hangul_jamo, split_hangul};
use crate::overlap_map::OverlapMap;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct OverlapPolicy: u8 {
        const DOUBLE_CONSONANT = 1 << 0;
        const COMPLEX_CONSONANT = 1 << 1;
        const ASYM_VOWEL = 1 << 2;
    }
}

// Generated using AI.
// Each slot is initialized only when its policy combination is first requested.
static BIMAPS: [LazyLock<OverlapMap>; 8] = [
    LazyLock::new(|| build_bimap(OverlapPolicy::from_bits_retain(0))),
    LazyLock::new(|| build_bimap(OverlapPolicy::from_bits_retain(1))),
    LazyLock::new(|| build_bimap(OverlapPolicy::from_bits_retain(2))),
    LazyLock::new(|| build_bimap(OverlapPolicy::from_bits_retain(3))),
    LazyLock::new(|| build_bimap(OverlapPolicy::from_bits_retain(4))),
    LazyLock::new(|| build_bimap(OverlapPolicy::from_bits_retain(5))),
    LazyLock::new(|| build_bimap(OverlapPolicy::from_bits_retain(6))),
    LazyLock::new(|| build_bimap(OverlapPolicy::from_bits_retain(7))),
];

// Generated using AI.
fn get_bimap(policy: OverlapPolicy) -> &'static OverlapMap {
    // Unknown bits do not affect the relations selected by build_bimap.
    let index = (policy & OverlapPolicy::all()).bits() as usize;
    LazyLock::force(&BIMAPS[index])
}

fn build_bimap(policy: OverlapPolicy) -> OverlapMap {
    OverlapMap::from_pairs(
        ALWAYS
            .iter()
            .copied()
            .chain_if_lazy(policy.contains(OverlapPolicy::DOUBLE_CONSONANT), || DOUBLE_CONSONANT.iter().copied())
            .chain_if_lazy(policy.contains(OverlapPolicy::COMPLEX_CONSONANT), || COMPLEX_CONSONANT.iter().copied())
            .chain_if_lazy(policy.contains(OverlapPolicy::ASYM_VOWEL), || ASYM_VOWEL.iter().copied())
    )
}

fn check_overlap_jamo(under: HangulJamo, over: HangulJamo, bimap: &OverlapMap) -> bool {
    under==over || match (under, over) {
        (HangulJamo::Some(under), HangulJamo::Some(over)) => bimap.contains_relation(&under, &over),
        (HangulJamo::None, HangulJamo::None) => true, // Can't reach here, but I wrote this...
        (HangulJamo::Some(_), HangulJamo::None) => false,
        (HangulJamo::None, HangulJamo::Some(_)) => true,
    }
}

pub fn check_overlap(under: char, over: char, policy: OverlapPolicy) -> Option<bool> {
    let bimap = get_bimap(policy);

    let under = split_hangul(under)?;
    let over = split_hangul(over)?;

    Some(
        check_overlap_jamo(under.0, over.0, bimap)
        && check_overlap_jamo(under.1, over.1, bimap)
        && check_overlap_jamo(under.2, over.2, bimap)
        ||(
            (under.0 != HangulJamo::None && under.1 == HangulJamo::None && under.2 == HangulJamo::None) && (
                check_overlap_jamo(under.0, over.2, bimap)
            ) // if under is single consonant
        )
        ||(
            (under.0 == HangulJamo::None && under.1 == HangulJamo::None && under.2 != HangulJamo::None) && (
                check_overlap_jamo(under.2, over.1, bimap)
            ) // if under is single complex consonant
        )
    )
}

pub fn overs_of(ch: char, policy: OverlapPolicy) -> Option<Vec<char>> {
    let bimap = get_bimap(policy);
    let ch = split_hangul(ch)?;

    let cho_list: Vec<HangulJamo> = match ch.0 {
        Some(cho) => bimap.get_by_left(&cho)
            .into_iter().flatten().cloned()
            .chain([cho]).map(Some).collect(),
        None => ('ㄱ'..='ㅎ')
            .map(Some).chain([None]).collect(),
    };

    let jung_list: Vec<HangulJamo> = match ch.1 {
        Some(jung) => bimap.get_by_left(&jung)
            .into_iter().flatten().cloned()
            .chain([jung]).map(Some).collect(),
        None => ('ㅏ'..='ㅣ')
            .map(Some).chain([None]).collect(),
    };

    let jong_list: Vec<HangulJamo> = match ch.2 {
        Some(jong) => bimap.get_by_left(&jong)
            .into_iter().flatten().cloned()
            .chain([jong]).map(Some).collect(),
        None => ('ㄱ'..='ㅎ')
            .map(Some).chain([None]).collect(),
    };

    Some(
        iproduct!(
            cho_list.iter().copied(),
            jung_list.iter().copied(),
            jong_list.iter().copied()
        )
            .map(|(cho, jung, jong)| compose_hangul_jamo(cho, jung, jong))
            .chain_if_lazy(
                ch.0 != None && ch.1 == None && ch.2 == None,
                || {
                    iproduct!(
                        ('ㄱ'..='ㅎ').map(Some),
                        ('ㅏ'..='ㅣ').map(Some)
                    ).map(
                        |(cho, jung)| compose_hangul_jamo(cho, jung, ch.0)
                    )
                }
            )
            .flatten().sorted_unstable().dedup().collect()
    )
}

pub fn unders_of(ch: char, policy: OverlapPolicy) -> Option<Vec<char>> {
    let bimap = get_bimap(policy);
    let ch = split_hangul(ch)?;

    let cho_list: Vec<HangulJamo> = match ch.0 {
        Some(cho) => bimap.get_by_right(&cho)
            .into_iter().flatten().cloned()
            .chain([cho]).map(Some).collect(),
        None => Vec::new(),
    };

    let jung_list: Vec<HangulJamo> = match ch.1 {
        Some(jung) => bimap.get_by_right(&jung)
            .into_iter().flatten().cloned()
            .chain([jung]).map(Some).collect(),
        None => Vec::new(),
    };

    let jong_list: Vec<HangulJamo> = match ch.2 {
        Some(jong) => bimap.get_by_right(&jong)
            .into_iter().flatten().cloned()
            .chain([jong]).map(Some).collect(),
        None => [None].into(),
    };

    Some(
        iproduct!(
            cho_list.iter().copied(),
            jung_list.iter().copied(),
            jong_list.iter().copied()
        )
            .map(|(cho, jung, jong)| compose_hangul_jamo(cho, jung, jong))
            .chain_if_lazy(
                ch.0 != None,
                || bimap.get_by_right(&ch.0.unwrap())
                    .into_iter().flatten().copied().map(Some)
            )
            .chain_if_lazy(
                ch.1 != None,
                || bimap.get_by_right(&ch.1.unwrap())
                    .into_iter().flatten().copied().map(Some)
            )
            .chain_if_lazy(
                ch.2 != None,
                || bimap.get_by_right(&ch.2.unwrap())
                    .into_iter().flatten().copied().map(Some)
            )
            .flatten().sorted_unstable().dedup().collect()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Generated using AI.
    #[test]
    fn test_cached_relations_match_policy_tables() {
        let jamo: Vec<char> = ('ㄱ'..='ㅣ').chain(['A']).collect();

        for bits in 0..8 {
            let policy = OverlapPolicy::from_bits_retain(bits);
            let mut expected = ALWAYS.to_vec();
            if bits & 1 != 0 {
                expected.extend_from_slice(DOUBLE_CONSONANT);
            }
            if bits & 2 != 0 {
                expected.extend_from_slice(COMPLEX_CONSONANT);
            }
            if bits & 4 != 0 {
                expected.extend_from_slice(ASYM_VOWEL);
            }

            let map = get_bimap(policy);
            for &key in &jamo {
                let rights: Vec<char> = expected.iter()
                    .filter(|(left, _)| *left == key)
                    .map(|(_, right)| *right).collect();
                let lefts: Vec<char> = expected.iter()
                    .filter(|(_, right)| *right == key)
                    .map(|(left, _)| *left).collect();

                assert_eq!(map.get_by_left(&key),
                    (!rights.is_empty()).then_some(rights.as_slice()),
                    "left lookup: {policy:?}, {key}");
                assert_eq!(map.get_by_right(&key),
                    (!lefts.is_empty()).then_some(lefts.as_slice()),
                    "right lookup: {policy:?}, {key}");

                for &right in &jamo {
                    assert_eq!(map.contains_relation(&key, &right),
                        expected.contains(&(key, right)),
                        "relation: {policy:?}, {key}, {right}");
                }
            }
        }
    }

    // Generated using AI.
    #[test]
    fn test_cached_bimaps_match_builds() {
        for bits in 0..8 {
            let policy = OverlapPolicy::from_bits_retain(bits);
            assert_eq!(get_bimap(policy), &build_bimap(policy));
            assert!(std::ptr::eq(get_bimap(policy), get_bimap(policy)));
        }
    }

    // Generated using AI.
    #[test]
    fn test_cached_bimaps_ignore_unknown_bits() {
        for bits in 0..=u8::MAX {
            let policy = OverlapPolicy::from_bits_retain(bits);
            let known_policy = policy & OverlapPolicy::all();
            assert!(std::ptr::eq(get_bimap(policy), get_bimap(known_policy)));
        }
    }

    #[test]
    fn test_check_overlap_always() {
        // Should be Some(true)
        assert!(check_overlap('ㄱ', 'ㄱ', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('ㄱ', 'ㄹ', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('ㅏ', 'ㅐ', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('ㄱ', '가', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('ㄱ', '각', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('ㄱ', '악', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('ㄱ', '라', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('ㄱ', '알', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('ㅏ', '가', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('ㅏ', '개', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('각', '각', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('각', '칵', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('각', '갘', OverlapPolicy::empty()).unwrap());
        assert!(check_overlap('각', '객', OverlapPolicy::empty()).unwrap());

        // Should be Some(false)
        assert!(!check_overlap('각', 'ㄱ', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('가', '나', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('각', '가', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('까', '가', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('가', '까', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('가', '까', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('ㄱ', 'ㄳ', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('ㄱ', '앇', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('악', '앇', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('ㅏ', 'ㅑ', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('ㅏ', '야', OverlapPolicy::empty()).unwrap());
        assert!(!check_overlap('아', '야', OverlapPolicy::empty()).unwrap());

        // Should be None
        assert!(check_overlap('A', 'ㄱ', OverlapPolicy::empty()).is_none());
        assert!(check_overlap('각', 'A', OverlapPolicy::empty()).is_none());
    }

    #[test]
    fn test_check_overlap_dc() {
        assert!(check_overlap('ㄱ', 'ㄲ', OverlapPolicy::DOUBLE_CONSONANT).unwrap());
        assert!(check_overlap('ㄱ', '까', OverlapPolicy::DOUBLE_CONSONANT).unwrap());
        assert!(check_overlap('ㄱ', '앆', OverlapPolicy::DOUBLE_CONSONANT).unwrap());
        assert!(check_overlap('가', '까', OverlapPolicy::DOUBLE_CONSONANT).unwrap());
        assert!(check_overlap('악', '앆', OverlapPolicy::DOUBLE_CONSONANT).unwrap());
        assert!(check_overlap('각', '깎', OverlapPolicy::DOUBLE_CONSONANT).unwrap());
        assert!(check_overlap('각', '깍', OverlapPolicy::DOUBLE_CONSONANT).unwrap());
        assert!(check_overlap('각', '갂', OverlapPolicy::DOUBLE_CONSONANT).unwrap());
    }

    #[test]
    fn test_check_overlap_cc() {
        assert!(check_overlap('ㄱ', 'ㄳ', OverlapPolicy::COMPLEX_CONSONANT).unwrap());
        assert!(check_overlap('ㄱ', '앇', OverlapPolicy::COMPLEX_CONSONANT).unwrap());
        assert!(check_overlap('악', '앇', OverlapPolicy::COMPLEX_CONSONANT).unwrap());
    }

    #[test]
    fn test_check_overlap_asym() {
        assert!(check_overlap('ㅏ', 'ㅑ', OverlapPolicy::ASYM_VOWEL).unwrap());
        assert!(check_overlap('ㅏ', '야', OverlapPolicy::ASYM_VOWEL).unwrap());
        assert!(check_overlap('아', '야', OverlapPolicy::ASYM_VOWEL).unwrap());
    }

    #[test]
    fn test_check_overlap_all() {
        assert!(check_overlap('악', '햯', OverlapPolicy::all()).unwrap());
        assert!(check_overlap('각', '걗', OverlapPolicy::all()).unwrap());
        assert!(check_overlap('묵', '뷁', OverlapPolicy::all()).unwrap());
    }

    #[test]
    #[ignore]
    fn manual_test_overs_of() {
        println!("{:?}",overs_of('각', OverlapPolicy::empty()).unwrap());
    }

    #[test]
    #[ignore]
    fn manual_test_unders_of() {
        println!("{:?}",unders_of('쀏', OverlapPolicy::all()).unwrap());
    }

    #[test]
    fn test_overs_of_check_overlap() {
        let policy = OverlapPolicy::empty();
        for ch in '꽤'..'꾀'{
            for over in overs_of(ch, policy).unwrap() {
                assert!(check_overlap(ch, over, policy).unwrap())
            }
        }
    }

    #[test]
    fn test_unders_of_check_overlap() {
        let policy = OverlapPolicy::all();
        for ch in '꽤'..'꾀'{
            for under in unders_of(ch, policy).unwrap() {
                assert!(check_overlap(under, ch, policy).unwrap())
            }
        }
    }
}
