# hangul-overlap

한글 문자끼리 겹칠 수 있는지 판정하고, 겹칠 수 있는 글자를 찾는 Rust 라이브러리입니다.

`ㄱ`을 `ㄹ`로 덮을 수 있는지 확인하거나, `각` 위에 겹칠 수 있는 글자를 찾을 수 있습니다. 자모 관계표와 선택한 정책을 사용해 판정합니다.

```rust
use hangul_overlap::{check_overlap, OverlapPolicy};

let policy = OverlapPolicy::empty();

assert_eq!(check_overlap('ㄱ', 'ㄹ', policy), Some(true));
assert_eq!(check_overlap('ㄹ', 'ㄱ', policy), Some(false));
assert_eq!(check_overlap('가', '나', policy), Some(false));
```

## 설치

프로젝트의 `Cargo.toml`에 다음 의존성을 추가합니다.

```toml
[dependencies]
hangul-overlap = { git = "https://github.com/Imagination12357/hangul-overlap.git" }
```

코드에서는 `use hangul_overlap::...`으로 가져옵니다.

## 겹침 관계

`under`는 아래에 놓을 글자, `over`는 그 위에 겹칠 글자입니다. 판정은 인자 순서에 의존합니다.

이 프로젝트에서 `겹치다`라는 용어는 그 아래의 대상을 완전히 가려 보이지 않게 만든다는 의미로 사용됩니다.

```text
check_overlap(under, over, policy)
              아래   위
```

기본 관계에는 `ㄱ -> ㄹ`, `ㅁ -> ㅂ`, `ㅏ -> ㅐ` 등이 있습니다. 완성형 음절은 초성, 중성, 종성으로 나누어 비교합니다. 아래 글자에 받침이 없어도 위 글자에는 받침이 올 수 있습니다. 단독 자음은 완성형 음절의 종성과도 비교합니다.

```rust
use hangul_overlap::{check_overlap, OverlapPolicy};

let policy = OverlapPolicy::empty();

assert_eq!(check_overlap('각', '칵', policy), Some(true));
assert_eq!(check_overlap('가', '각', policy), Some(true));
assert_eq!(check_overlap('각', '가', policy), Some(false));
assert_eq!(check_overlap('ㄱ', '악', policy), Some(true));
```

판정 기준은 라이브러리의 [src/data.rs](src/data.rs)입니다. 실제 글꼴을 렌더링하거나 픽셀을 비교하지 않습니다.

[src/data.rs](src/data.rs)는 [reference.xlsx](reference.xlsx)에 기반하여 작성되었습니다. 두 파일 모두 수동 작성했으므로 오류를 발견하신다면 제보 부탁드립니다.

## 사용법

### 두 문자의 관계 판정

```rust
pub fn check_overlap(under: char, over: char, policy: OverlapPolicy) -> Option<bool>
```

| 반환값 | 의미 |
| --- | --- |
| `Some(true)` | 해당 정책에서 겹침 관계가 성립함 |
| `Some(false)` | 두 문자 모두 지원하는 입력이지만 관계가 성립하지 않음 |
| `None` | 어느 한쪽이라도 지원하지 않는 문자임 |

```rust
use hangul_overlap::{check_overlap, OverlapPolicy};

let policy = OverlapPolicy::empty();

assert_eq!(check_overlap('각', '각', policy), Some(true));
assert_eq!(check_overlap('가', '나', policy), Some(false));
assert_eq!(check_overlap('A', '가', policy), None);
```

### 위에 겹칠 문자 후보 조회

```rust
pub fn overs_of(ch: char, policy: OverlapPolicy) -> Option<Vec<char>>
```

`ch` 위에 겹칠 수 있는 글자를 구합니다.

```rust
use hangul_overlap::{overs_of, OverlapPolicy};

let overs = overs_of('각', OverlapPolicy::empty()).unwrap();
assert!(overs.contains(&'칵'));
```

### 아래에 놓일 문자 후보 조회

```rust
pub fn unders_of(ch: char, policy: OverlapPolicy) -> Option<Vec<char>>
```

`ch` 아래에 놓을 수 있는 글자를 구합니다.

```rust
use hangul_overlap::{unders_of, OverlapPolicy};

let unders = unders_of('칵', OverlapPolicy::empty()).unwrap();
assert!(unders.contains(&'각'));
```

두 함수의 결과는 중복 없이 유니코드 코드 포인트 순으로 정렬됩니다. 후보가 없으면 빈 벡터를, 지원하지 않는 문자를 입력하면 `None`을 반환합니다.

## OverlapPolicy

기본 관계는 항상 적용합니다. 플래그를 켜면 해당 관계도 추가로 허용합니다.

| 정책 | 의미 | 예시 |
| --- | --- | --- |
| `OverlapPolicy::empty()` | 기본 관계만 적용 | `ㄱ -> ㄹ`, `ㅏ -> ㅐ` |
| `DOUBLE_CONSONANT` | 쌍자음으로 겹치기 허용 | `ㄱ -> ㄲ`, `ㅅ -> ㅆ` |
| `COMPLEX_CONSONANT` | 겹자음으로 겹치기 허용 | `ㄱ -> ㄳ`, `ㅅ -> ㄳ` |
| `ASYM_VOWEL` | 추가 모음 관계 허용 | `ㅏ -> ㅑ`, `ㅗ -> ㅛ` |

```rust
use hangul_overlap::{check_overlap, OverlapPolicy};

assert_eq!(
    check_overlap('ㄱ', 'ㄲ', OverlapPolicy::empty()),
    Some(false)
);
assert_eq!(
    check_overlap('ㄱ', 'ㄲ', OverlapPolicy::DOUBLE_CONSONANT),
    Some(true)
);

// 필요한 정책을 | 연산자로 조합합니다.
let policy = OverlapPolicy::DOUBLE_CONSONANT | OverlapPolicy::COMPLEX_CONSONANT;
assert_eq!(check_overlap('ㄱ', 'ㄲ', policy), Some(true));
assert_eq!(check_overlap('ㄱ', 'ㄳ', policy), Some(true));

// 정의된 플래그를 모두 활성화합니다.
let all = OverlapPolicy::all();
assert_eq!(check_overlap('ㅏ', 'ㅑ', all), Some(true));
```

각 플래그가 허용하는 관계는 [src/data.rs](src/data.rs)에 정의되어 있으며, [reference.xlsx](reference.xlsx)에서도 확인할 수 있습니다.

## 지원 입력

| 입력 종류 | 지원 범위 |
| --- | --- |
| 현대 한글 완성형 음절 | `가`-`힣` (`U+AC00`-`U+D7A3`) |
| 한글 호환용 자모 | `ㄱ`-`ㅣ` (`U+3131`-`U+3163`) |

자모를 입력할 때는 `ㄱ`, `ㅏ`, `ㄳ`와 같은 호환용 자모를 사용합니다. 조합용 자모 `ᄀ` (`U+1100`)나 `ᅡ` (`U+1161`), 옛한글, 영문, 숫자, 공백 등을 입력하면 `None`을 반환합니다. 입력은 정규화하지 않고 그대로 처리합니다.

## 개발 및 테스트

저장소 루트에서 테스트를 실행합니다.

```sh
cargo test --locked
```

[프로파일 예제](examples/profile.rs)로 반복 호출 성능을 측정할 수 있습니다.

```sh
cargo run --release --locked --example profile
```

예제에서는 정해진 범위의 각 문자에 대해 후보를 조회하고 겹침 관계를 검증하는 것을 여러 번 반복합니다.

## 라이선스

이 프로젝트는 [MIT 라이선스](LICENSE)에 따라 배포됩니다.
