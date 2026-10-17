//! Generated using AI.
//! Immutable, bidirectional char relations backed by Fx maps and ordered vectors.
//! Built from relation pairs; queries borrow slices. Duplicate pairs are preserved.

use rustc_hash::FxHashMap;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OverlapMap {
    left_to_right: FxHashMap<char, Vec<char>>,
    right_to_left: FxHashMap<char, Vec<char>>,
}

impl OverlapMap {
    pub(crate) fn from_pairs(pairs: impl IntoIterator<Item = (char, char)>) -> Self {
        let mut map = Self {
            left_to_right: FxHashMap::default(),
            right_to_left: FxHashMap::default(),
        };

        for (left, right) in pairs {
            map.left_to_right.entry(left).or_default().push(right);
            map.right_to_left.entry(right).or_default().push(left);
        }

        map
    }

    pub(crate) fn get_by_left(&self, left: &char) -> Option<&[char]> {
        self.left_to_right.get(left).map(Vec::as_slice)
    }

    pub(crate) fn get_by_right(&self, right: &char) -> Option<&[char]> {
        self.right_to_left.get(right).map(Vec::as_slice)
    }

    pub(crate) fn contains_relation(&self, left: &char, right: &char) -> bool {
        self.get_by_left(left)
            .is_some_and(|rights| rights.contains(right))
    }
}

// Generated using AI.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_relations_in_both_directions() {
        let map = OverlapMap::from_pairs([
            ('ㄱ', 'ㅂ'),
            ('ㄴ', 'ㅂ'),
            ('ㄱ', 'ㄹ'),
            ('ㄱ', 'ㅂ'),
        ]);

        assert_eq!(map.get_by_left(&'ㄱ'), Some(&['ㅂ', 'ㄹ', 'ㅂ'][..]));
        assert_eq!(map.get_by_right(&'ㅂ'), Some(&['ㄱ', 'ㄴ', 'ㄱ'][..]));
        assert_eq!(map.get_by_left(&'ㄴ'), Some(&['ㅂ'][..]));
        assert_eq!(map.get_by_right(&'ㄹ'), Some(&['ㄱ'][..]));
        assert!(map.contains_relation(&'ㄱ', &'ㅂ'));
        assert!(!map.contains_relation(&'ㄴ', &'ㄹ'));
        assert!(!map.contains_relation(&'ㅂ', &'ㄱ'));
        assert_eq!(map.get_by_left(&'ㅂ'), None);
        assert_eq!(map.get_by_right(&'ㄱ'), None);
    }

    #[test]
    fn empty_map_has_no_relations() {
        let map = OverlapMap::from_pairs([]);

        assert_eq!(map.get_by_left(&'ㄱ'), None);
        assert_eq!(map.get_by_right(&'ㄹ'), None);
        assert!(!map.contains_relation(&'ㄱ', &'ㄹ'));
    }
}
