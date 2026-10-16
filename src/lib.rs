mod helpers;
mod data;
use data::{
    ALWAYS,
    DOUBLE_CONSONANT,
    COMPLEX_CONSONANT,
    ASYM_VOWEL,
};
use bitflags::bitflags;
use multi_bimap::HashMultiBimap;
use itertools::{iproduct, Itertools};
use std::sync::LazyLock;
use crate::helpers::{HangulJamo, HashMultiBimapExt, IteratorExt, compose_hangul_jamo, split_hangul};

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
static BIMAPS: [LazyLock<HashMultiBimap<char, char>>; 8] = [
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
fn get_bimap(policy: OverlapPolicy) -> &'static HashMultiBimap<char, char> {
    // Unknown bits do not affect the relations selected by build_bimap.
    let index = (policy & OverlapPolicy::all()).bits() as usize;
    LazyLock::force(&BIMAPS[index])
}

fn build_bimap(policy: OverlapPolicy) -> HashMultiBimap<char, char> {
    ALWAYS
        .iter()
        .copied()
        .chain_if_lazy(policy.contains(OverlapPolicy::DOUBLE_CONSONANT), || DOUBLE_CONSONANT.iter().copied())
        .chain_if_lazy(policy.contains(OverlapPolicy::COMPLEX_CONSONANT), || COMPLEX_CONSONANT.iter().copied())
        .chain_if_lazy(policy.contains(OverlapPolicy::ASYM_VOWEL), || ASYM_VOWEL.iter().copied())
        .collect_bimap()
}

fn check_overlap_jamo(under: HangulJamo, over: HangulJamo, bimap: &HashMultiBimap<char, char>) -> bool {
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
        iproduct!(cho_list, jung_list, jong_list)
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
        iproduct!(cho_list, jung_list, jong_list)
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
