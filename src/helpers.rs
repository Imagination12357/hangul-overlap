use std::hash::Hash;
use multi_bimap::HashMultiBimap;
use mecab_ko_hangul::{
    is_jamo,
    decompose,
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

pub enum HangulUnit {
    Syllable {
        cho: char,
        jung: char,
        jong: Option<char>,
    },
    Jamo(char),
}

pub fn split_hangul(ch: char) -> Option<HangulUnit> {
    if let Some((cho, jung, jong)) = decompose(ch) {
        Some(HangulUnit::Syllable{cho, jung, jong})
    } else if is_jamo(ch) {
        Some(HangulUnit::Jamo(ch))
    } else {
        None
    }
}