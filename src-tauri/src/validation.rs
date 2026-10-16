//! 입력 검증 (business-rules.md BR-1)

use crate::model::{AppError, AppResult, EntryInput, FieldErrors};

pub const NAME_MAX_CHARS: usize = 30;
pub const MEMO_MAX_CHARS: usize = 200;
pub const AMOUNT_MIN_WON: i64 = 1_000;
pub const AMOUNT_MAX_WON: i64 = 10_000_000;
pub const AMOUNT_UNIT_WON: i64 = 1_000;
pub const TICKETS_MIN: i64 = 0;
pub const TICKETS_MAX: i64 = 20;

/// 입력값을 검증하고 정규화(이름/메모 앞뒤 공백 제거)된 값을 돌려준다.
/// 오류가 있으면 모든 항목의 오류를 한 번에 반환한다.
pub fn validate_entry_input(input: &EntryInput) -> AppResult<EntryInput> {
    let mut errors = FieldErrors::new();

    let name = input.name.trim().to_string();
    let name_len = name.chars().count();
    if name_len == 0 {
        errors.insert("name".into(), "이름을 입력해 주세요".into());
    } else if name_len > NAME_MAX_CHARS {
        errors.insert("name".into(), format!("이름은 {NAME_MAX_CHARS}자 이하로 입력해 주세요"));
    }

    let amount = input.amount_won;
    if amount < AMOUNT_MIN_WON {
        errors.insert("amount".into(), "금액을 입력해 주세요".into());
    } else if amount > AMOUNT_MAX_WON {
        errors.insert("amount".into(), "1,000만원을 넘을 수 없습니다".into());
    } else if amount % AMOUNT_UNIT_WON != 0 {
        errors.insert("amount".into(), "금액은 천 원 단위로 입력해 주세요".into());
    }

    if input.tickets < TICKETS_MIN || input.tickets > TICKETS_MAX {
        errors.insert(
            "tickets".into(),
            format!("식권 수는 {TICKETS_MIN}~{TICKETS_MAX} 사이로 입력해 주세요"),
        );
    }

    let memo = input.memo.trim().to_string();
    if memo.chars().count() > MEMO_MAX_CHARS {
        errors.insert("memo".into(), format!("메모는 {MEMO_MAX_CHARS}자 이하로 입력해 주세요"));
    }

    // 구분(side)은 Side 열거형으로 역직렬화되므로 잘못된 값은 명령 진입 시 거부된다 (BR-1.8)

    if !errors.is_empty() {
        return Err(AppError::validation(errors));
    }

    Ok(EntryInput {
        name,
        amount_won: amount,
        side: input.side,
        tickets: input.tickets,
        memo,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Side;

    fn input(name: &str, amount: i64, tickets: i64, memo: &str) -> EntryInput {
        EntryInput {
            name: name.into(),
            amount_won: amount,
            side: Side::Groom,
            tickets,
            memo: memo.into(),
        }
    }

    fn field_errors(result: AppResult<EntryInput>) -> FieldErrors {
        match result {
            Err(AppError::Validation { field_errors, .. }) => field_errors,
            other => panic!("검증 오류가 아님: {other:?}"),
        }
    }

    #[test]
    fn valid_input_is_normalized() {
        let v = validate_entry_input(&input("  김민수 ", 55_000, 1, " 대학 동기 ")).unwrap();
        assert_eq!(v.name, "김민수");
        assert_eq!(v.memo, "대학 동기");
        assert_eq!(v.amount_won, 55_000);
    }

    #[test]
    fn name_boundaries() {
        assert!(field_errors(validate_entry_input(&input("", 50_000, 1, ""))).contains_key("name"));
        assert!(field_errors(validate_entry_input(&input("   ", 50_000, 1, ""))).contains_key("name"));
        assert!(validate_entry_input(&input("김", 50_000, 1, "")).is_ok());
        assert!(validate_entry_input(&input(&"가".repeat(30), 50_000, 1, "")).is_ok());
        let e = field_errors(validate_entry_input(&input(&"가".repeat(31), 50_000, 1, "")));
        assert_eq!(e["name"], "이름은 30자 이하로 입력해 주세요");
    }

    #[test]
    fn amount_boundaries() {
        let err = |a| field_errors(validate_entry_input(&input("김", a, 1, "")))["amount"].clone();
        assert_eq!(err(0), "금액을 입력해 주세요");
        assert_eq!(err(-50_000), "금액을 입력해 주세요");
        assert_eq!(err(999), "금액을 입력해 주세요");
        assert!(validate_entry_input(&input("김", 1_000, 1, "")).is_ok());
        assert_eq!(err(1_500), "금액은 천 원 단위로 입력해 주세요");
        assert!(validate_entry_input(&input("김", 10_000_000, 1, "")).is_ok());
        assert_eq!(err(10_001_000), "1,000만원을 넘을 수 없습니다");
    }

    #[test]
    fn tickets_boundaries() {
        assert!(field_errors(validate_entry_input(&input("김", 50_000, -1, ""))).contains_key("tickets"));
        assert!(validate_entry_input(&input("김", 50_000, 0, "")).is_ok());
        assert!(validate_entry_input(&input("김", 50_000, 20, "")).is_ok());
        let e = field_errors(validate_entry_input(&input("김", 50_000, 21, "")));
        assert_eq!(e["tickets"], "식권 수는 0~20 사이로 입력해 주세요");
    }

    #[test]
    fn memo_boundaries() {
        assert!(validate_entry_input(&input("김", 50_000, 1, &"메".repeat(200))).is_ok());
        assert!(field_errors(validate_entry_input(&input("김", 50_000, 1, &"메".repeat(201)))).contains_key("memo"));
    }

    #[test]
    fn reports_all_errors_at_once() {
        let e = field_errors(validate_entry_input(&input("", 0, 21, &"x".repeat(201))));
        assert_eq!(e.len(), 4);
    }
}
