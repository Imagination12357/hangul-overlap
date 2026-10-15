use std::hash::Hash;
use multi_bimap::HashMultiBimap;
use mecab_ko_hangul::{
    decompose, is_choseong, is_jungseong, is_jamo
};

// Generated using AI.
pub trait IteratorExt: Iterator + Sized {
    fn chain_if<I>(
        self,
        condition: bool,
        iter: I,
    ) -> impl Iterator<Item = Self::Item>
    where
        I: IntoIterator<Item = Self::Item>,
    {
        self.chain(
            condition
                .then_some(iter)
                .into_iter()
                .flatten()
        )
    }

    fn collect_bimap<L, R>(self) -> HashMultiBimap<L, R>
    where
        Self: Iterator<Item = (L, R)>,
        L: Eq + Hash + Clone,
        R: Eq + Hash + Clone,
    {
        let mut map = HashMultiBimap::new();

        for (left, right) in self {
            map.insert(left, right);
        }

        map
    }
}

impl<I: Iterator> IteratorExt for I {}

pub trait HashMultiBimapExt<L, R> {
    fn contains_relation(&self, left: &L, right: &R) -> bool;
}

impl<L, R> HashMultiBimapExt<L, R> for HashMultiBimap<L, R>
where
    L: Eq + Hash + Clone,
    R: Eq + Hash + Clone,
{
    fn contains_relation(&self, left: &L, right: &R) -> bool {
        self.get_by_left(left)
            .is_some_and(|rights| rights.contains(right))
    }
}

pub type HangulJamo = Option<char>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HangulUnit (pub HangulJamo, pub HangulJamo, pub HangulJamo);

impl HangulUnit {
    pub fn new_from_cho(ch: char) -> HangulUnit {
        HangulUnit(
            HangulJamo::Some(ch),
            HangulJamo::None,
            HangulJamo::None,
        )
    }
    pub fn new_from_jung(ch: char) -> HangulUnit {
        HangulUnit(
            HangulJamo::None,
            HangulJamo::Some(ch),
            HangulJamo::None,
        )
    }
    pub fn new_from_jong(ch: char) -> HangulUnit {
        HangulUnit(
            HangulJamo::None,
            HangulJamo::None,
            HangulJamo::Some(ch),
        )
    }
}

fn is_cc(c: char) -> bool {
    is_jamo(c) && !is_choseong(c) && !is_jungseong(c)
}

pub fn split_hangul(ch: char) -> Option<HangulUnit> {
    if let Some((cho, jung, jong)) = decompose(ch) {
        Some(HangulUnit(
            HangulJamo::Some(cho),
            HangulJamo::Some(jung),
            jong,
        ))
    } else if is_choseong(ch) {
        Some(HangulUnit::new_from_cho(ch))
    } else if is_jungseong(ch) {
        Some(HangulUnit::new_from_jung(ch))
    } else if is_cc(ch) {
        Some(HangulUnit::new_from_jong(ch))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_is_cc() {
        assert!(is_cc('ㄳ'));
        assert!(is_cc('ㄵ'));
        assert!(is_cc('ㄶ'));
        assert!(is_cc('ㄺ'));
        assert!(is_cc('ㄻ'));
        assert!(is_cc('ㄼ'));
        assert!(is_cc('ㄽ'));
        assert!(is_cc('ㄾ'));
        assert!(is_cc('ㄿ'));
        assert!(is_cc('ㅀ'));
        assert!(is_cc('ㅄ'));

        assert!(!is_cc('ㄱ'));
        assert!(!is_cc('가'));
        assert!(!is_cc('ㅏ'));
        assert!(!is_cc('ㄲ'));
        assert!(!is_cc('까'));
        assert!(!is_cc('악'));
        assert!(!is_cc('앆'));
        assert!(!is_cc('앇'));
    }
}