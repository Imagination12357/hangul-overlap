//! 한글 문자끼리 겹칠 수 있는지 판정하고, 겹칠 수 있는 글자를 찾습니다.
//!
//! 여기서 겹친다는 것은 위 글자가 아래 글자를 완전히 가려 보이지 않게 한다는 뜻입니다.
//! 실제 글꼴이나 픽셀을 비교하지 않고, 미리 정의된 자모 관계표와 [`OverlapPolicy`]로 판정합니다.
//!
//! - [`check_overlap`]: 아래 글자와 위 글자의 겹침 여부를 판정합니다.
//! - [`overs_of`]: 주어진 글자 위에 겹칠 수 있는 글자를 구합니다.
//! - [`unders_of`]: 주어진 글자 아래에 놓을 수 있는 글자를 구합니다.
//!
//! # 예시
//!
//! 관계에는 방향이 있으므로 아래 글자와 위 글자의 순서를 구분해야 합니다.
//!
//! ```
//! use hangul_overlap::{check_overlap, OverlapPolicy};
//!
//! let policy = OverlapPolicy::empty();
//! assert_eq!(check_overlap('ㄱ', 'ㄹ', policy), Some(true));
//! assert_eq!(check_overlap('ㄹ', 'ㄱ', policy), Some(false));
//! ```
//!
//! # 지원 입력
//!
//! 각 함수는 단일 [`char`]를 받습니다. 현대 한글 완성형 음절 `가`-`힣`
//! (`U+AC00`-`U+D7A3`)과 한글 호환용 자모 `ㄱ`-`ㅣ` (`U+3131`-`U+3163`)를 지원합니다.
//! 조합용 자모 `ᄀ` (`U+1100`), `ᅡ` (`U+1161`)나 옛한글, 영문, 숫자, 공백 등은
//! 지원하지 않습니다. 입력은 정규화하지 않고 그대로 처리합니다.
//!
//! # 정책
//!
//! 기본 자모 관계는 항상 적용합니다. [`OverlapPolicy::empty()`]는 기본 관계만 적용하고,
//! [`OverlapPolicy::all()`]은 정의된 추가 관계를 모두 허용합니다.
//! 필요한 플래그만 `|`로 조합할 수도 있습니다.
//!
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
    /// 기본 자모 관계에 추가로 허용할 겹침 관계를 선택합니다.
    ///
    /// [`Self::empty()`]와 [`Self::default()`]는 기본 관계만 적용합니다.
    /// 플래그를 켜면 해당 관계를 추가로 허용하며, 기본 관계를 제거하지는 않습니다.
    /// 같은 자모끼리의 겹침은 플래그와 관계없이 허용합니다.
    ///
    /// # 예시
    ///
    /// ```
    /// use hangul_overlap::{check_overlap, OverlapPolicy};
    ///
    /// assert_eq!(check_overlap('ㄱ', 'ㄲ', OverlapPolicy::empty()), Some(false));
    ///
    /// let policy = OverlapPolicy::DOUBLE_CONSONANT | OverlapPolicy::COMPLEX_CONSONANT;
    /// assert_eq!(check_overlap('ㄱ', 'ㄲ', policy), Some(true));
    /// assert_eq!(check_overlap('ㄱ', 'ㄳ', policy), Some(true));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct OverlapPolicy: u8 {
        /// 홑자음에서 쌍자음으로 겹치기를 허용합니다.
        ///
        /// `ㄱ -> ㄲ`, `ㄷ -> ㄸ`, `ㅂ -> ㅃ`, `ㅅ -> ㅆ`, `ㅈ -> ㅉ`를 추가합니다.
        const DOUBLE_CONSONANT = 1 << 0;
        /// 구성 자음에서 겹자음으로 겹치기를 허용합니다.
        ///
        /// 예를 들어 `ㄱ -> ㄳ`, `ㅅ -> ㄳ`, `ㄴ -> ㄵ`, `ㅈ -> ㄵ`를 추가합니다.
        const COMPLEX_CONSONANT = 1 << 1;
        /// 기본 관계에 없는 추가 모음 관계를 허용합니다.
        ///
        /// 예를 들어 `ㅏ -> ㅑ`, `ㅓ -> ㅕ`, `ㅗ -> ㅛ`, `ㅜ -> ㅠ`를 추가합니다.
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

/// `over`로 `under`를 덮을 수 있는지 판정합니다.
///
/// `under`는 아래에 놓을 글자, `over`는 위에 겹칠 글자입니다.
/// 인자 순서를 바꾸면 판정 결과도 달라질 수 있습니다.
/// 완성형 음절은 초성, 중성, 종성으로 나누어 비교합니다.
/// 아래 글자에 받침이 없어도 위 글자에는 받침이 올 수 있으며,
/// 단독 자음은 완성형 음절의 종성과도 비교합니다.
/// `policy`로 기본 관계에 추가할 자모 관계를 선택합니다.
///
/// # 반환값
///
/// - `Some(true)`: 해당 정책에서 겹침 관계가 성립합니다. 같은 지원 문자끼리도 여기에 해당합니다.
/// - `Some(false)`: 두 문자 모두 지원하는 입력이지만 겹침 관계가 성립하지 않습니다.
/// - `None`: 어느 한쪽이라도 지원하지 않는 문자입니다.
///
/// 지원 범위는 [crate 문서의 지원 입력](crate#지원-입력)을 참고하세요.
///
/// # 예시
///
/// ```
/// use hangul_overlap::{check_overlap, OverlapPolicy};
///
/// let policy = OverlapPolicy::empty();
/// assert_eq!(check_overlap('가', '각', policy), Some(true));
/// assert_eq!(check_overlap('각', '가', policy), Some(false));
/// assert_eq!(check_overlap('ㄱ', '악', policy), Some(true));
/// assert_eq!(check_overlap('각', '각', policy), Some(true));
/// assert_eq!(check_overlap('A', '가', policy), None);
/// ```
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

/// `ch` 위에 겹칠 수 있는 글자를 구합니다.
///
/// `ch`를 아래에 놓을 글자로 사용하고, `policy`로 추가 자모 관계를 선택합니다.
/// 두 문자를 직접 판정하려면 [`check_overlap`]을, 아래에 놓을 글자를 구하려면
/// [`unders_of`]를 사용합니다.
///
/// # 반환값
///
/// 지원하는 입력이면 후보 목록을 `Some(Vec<char>)`로 반환합니다.
/// 목록은 중복 없이 유니코드 코드 포인트 오름차순으로 정렬됩니다.
/// `ch` 자신도 포함하므로 빈 벡터는 반환하지 않습니다.
/// 지원하지 않는 문자에는 `None`을 반환합니다.
/// 지원 범위는 [crate 문서의 지원 입력](crate#지원-입력)을 참고하세요.
///
/// # 예시
///
/// ```
/// use hangul_overlap::{overs_of, OverlapPolicy};
///
/// let overs = overs_of('각', OverlapPolicy::empty()).unwrap();
/// assert!(overs.contains(&'각'));
/// assert!(overs.contains(&'칵'));
/// assert_eq!(overs_of('A', OverlapPolicy::empty()), None);
/// ```
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

/// `ch` 아래에 놓을 수 있는 글자를 구합니다.
///
/// `ch`를 위에 겹칠 글자로 사용하고, `policy`로 추가 자모 관계를 선택합니다.
/// 두 문자를 직접 판정하려면 [`check_overlap`]을, 위에 겹칠 글자를 구하려면
/// [`overs_of`]를 사용합니다.
///
/// # 반환값
///
/// 지원하는 입력이면 후보 목록을 `Some(Vec<char>)`로 반환합니다.
/// 목록은 중복 없이 유니코드 코드 포인트 오름차순으로 정렬됩니다.
/// `ch` 자신도 포함하므로 빈 벡터는 반환하지 않습니다.
/// 지원하지 않는 문자에는 `None`을 반환합니다.
/// 지원 범위는 [crate 문서의 지원 입력](crate#지원-입력)을 참고하세요.
///
/// # 예시
///
/// ```
/// use hangul_overlap::{unders_of, OverlapPolicy};
///
/// let unders = unders_of('칵', OverlapPolicy::empty()).unwrap();
/// assert!(unders.contains(&'칵'));
/// assert!(unders.contains(&'각'));
///
/// let jamo = unders_of('ㄳ', OverlapPolicy::empty()).unwrap();
/// assert!(jamo.contains(&'ㄳ'));
/// assert_eq!(unders_of('A', OverlapPolicy::empty()), None);
/// ```
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
            .chain([jong]).map(Some).chain([None]).collect(),
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
                    .into_iter().flatten().copied().chain([ch.0.unwrap()]).map(Some)
            )
            .chain_if_lazy(
                ch.1 != None,
                || bimap.get_by_right(&ch.1.unwrap())
                    .into_iter().flatten().copied().chain([ch.1.unwrap()]).map(Some)
            )
            .chain_if_lazy(
                ch.2 != None,
                || bimap.get_by_right(&ch.2.unwrap())
                    .into_iter().flatten().copied().chain([ch.2.unwrap()]).map(Some)
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

    // Generated using AI.
    const OVERLAP_TEST_SYLLABLES: &[char] = &[
        '가', '각', '갂', '개', '객', '갸', '걔', '걗',
        '나', '낙', '까', '깍', '깎', '아', '악', '앆', '앇',
        '야', '묵', '뷁', '쀏', '햯', '힣',
    ];

    // Generated using AI.
    #[test]
    fn test_check_overlap_overs_of() {
        let test_chars: Vec<char> = ('ㄱ'..='ㅣ')
            .chain('꽤'..'꾀')
            .chain(OVERLAP_TEST_SYLLABLES.iter().copied()).collect();

        for policy_bits in 0..8 {
            let policy = OverlapPolicy::from_bits_retain(policy_bits);
            for &under in &test_chars {
                // 판정 함수로 기대 후보를 구합니다.
                let expected_overs: Vec<char> = test_chars.iter().copied()
                    .filter(|&over| check_overlap(under, over, policy).unwrap())
                    .collect();

                // 조회 함수가 기대 후보를 빠뜨리지 않았는지 확인합니다.
                let actual_overs = overs_of(under, policy).unwrap();
                for over in expected_overs {
                    assert!(actual_overs.contains(&over),
                        "overs_of({under}, {policy:?}) is missing {over}");
                }
            }
        }
    }

    // Generated using AI.
    #[test]
    fn test_check_overlap_unders_of() {
        let test_chars: Vec<char> = ('ㄱ'..='ㅣ')
            .chain('꽤'..'꾀')
            .chain(OVERLAP_TEST_SYLLABLES.iter().copied()).collect();

        for policy_bits in 0..8 {
            let policy = OverlapPolicy::from_bits_retain(policy_bits);
            for &over in &test_chars {
                // 판정 함수로 기대 후보를 구합니다.
                let expected_unders: Vec<char> = test_chars.iter().copied()
                    .filter(|&under| check_overlap(under, over, policy).unwrap())
                    .collect();

                // 조회 함수가 기대 후보를 빠뜨리지 않았는지 확인합니다.
                let actual_unders = unders_of(over, policy).unwrap();
                for under in expected_unders {
                    assert!(actual_unders.contains(&under),
                        "unders_of({over}, {policy:?}) is missing {under}");
                }
            }
        }
    }
}
