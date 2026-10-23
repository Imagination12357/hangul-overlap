use mecab_ko_hangul::{
    decompose, is_choseong, is_jungseong, is_jamo, compose
};


pub(crate) type HangulJamo = Option<char>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct HangulUnit (pub(crate) HangulJamo, pub(crate) HangulJamo, pub(crate) HangulJamo);

impl HangulUnit {
    pub(crate) fn new_from_cho(ch: char) -> HangulUnit {
        HangulUnit(
            HangulJamo::Some(ch),
            HangulJamo::None,
            HangulJamo::None,
        )
    }
    pub(crate) fn new_from_jung(ch: char) -> HangulUnit {
        HangulUnit(
            HangulJamo::None,
            HangulJamo::Some(ch),
            HangulJamo::None,
        )
    }
    pub(crate) fn new_from_jong(ch: char) -> HangulUnit {
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

pub(crate) fn split_hangul(ch: char) -> Option<HangulUnit> {
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

pub(crate) fn compose_hangul_jamo(cho: HangulJamo, jung: HangulJamo, jong: HangulJamo) -> Option<char> {
    if let (Some(cho), Some(jung)) = (cho,jung) {
        compose(cho, jung, jong)
    } else {
        match (cho, jung, jong) {
            (Some(cho), None, None) => Some(cho),
            (None, Some(jung), None) => Some(jung),
            (None, None, Some(jong)) => Some(jong),
            _ => None,
        }
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
