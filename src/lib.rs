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
use crate::helpers::{HashMultiBimapExt, IteratorExt, HangulUnit, split_hangul};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct OverlapPolicy: u8 {
        const DOUBLE_CONSONANT = 1 << 0;
        const COMPLEX_CONSONANT = 1 << 1;
        const ASYM_VOWEL = 1 << 2;
    }
}

pub fn get_bimap(policy: OverlapPolicy) -> HashMultiBimap<char, char> {
    ALWAYS
        .iter()
        .copied()
        .chain_if(policy.contains(OverlapPolicy::DOUBLE_CONSONANT), DOUBLE_CONSONANT.iter().copied())
        .chain_if(policy.contains(OverlapPolicy::COMPLEX_CONSONANT), COMPLEX_CONSONANT.iter().copied())
        .chain_if(policy.contains(OverlapPolicy::ASYM_VOWEL), ASYM_VOWEL.iter().copied())
        .collect_bimap()
}

fn check_overlap_jamo(under: char, over: char, bimap: &HashMultiBimap<char, char>) -> bool {
    under==over || bimap.contains_relation(&under, &over)
}

fn check_overlap_jamo_opt_handled(under: Option<char>, over: Option<char>, bimap: &HashMultiBimap<char, char>) -> bool {
    let Some(under) = under else {return true;};
    let Some(over) = over else {return false;}; // under is Some and over is None. Which means over can't overlap under.

    check_overlap_jamo(under, over, bimap)
}

pub fn check_overlap(under: char, over: char, policy: OverlapPolicy) -> Option<bool> {
    let bimap = get_bimap(policy);

    let under = split_hangul(under)?;
    let over = split_hangul(over)?;

    Some(match (under, over) {
        (
            HangulUnit::Jamo(underjamo),
            HangulUnit::Jamo(overjamo)
        ) => check_overlap_jamo(underjamo, overjamo, &bimap),

        (
            HangulUnit::Jamo(underjamo),
            HangulUnit::Syllable{cho: overcho, jung: overjung, jong: overjong}
        ) => {
            check_overlap_jamo(underjamo, overcho, &bimap)
            || check_overlap_jamo(underjamo, overjung, &bimap)
            || overjong.is_some_and(|overjong| check_overlap_jamo(underjamo, overjong, &bimap))
        },

        (
            HangulUnit::Syllable{..},
            HangulUnit::Jamo(_)
        ) => false, // under is full syllable, and over is jamo, which can't overlap under always.

        (
            HangulUnit::Syllable { cho: undercho, jung: underjung, jong: underjong },
            HangulUnit::Syllable { cho: overcho , jung: overjung , jong: overjong }
        ) => {
            check_overlap_jamo(undercho, overcho, &bimap)
            && check_overlap_jamo(underjung, overjung, &bimap)
            && check_overlap_jamo_opt_handled(underjong, overjong, &bimap)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}