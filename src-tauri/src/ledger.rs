//! 접수 장부 로직 (business-logic-model.md 2~8절)
//!
//! 모든 함수는 현재 시각과 PC 이름을 인자로 받는 순수 로직이다.
//! 파일 저장은 commands 계층이 담당한다.

use std::cmp::Ordering;

use crate::model::{
    AppError, AppResult, Entry, EntryInput, EntryQuery, HistoryItem, HistoryKind, HistoryRecord,
    LedgerFile, QueryResult, Side, SideSummary, SortBy, SortDir, Summary, Timestamp,
};
use crate::validation::validate_entry_input;

/// 접수 화면 최근 목록 건수 (FR-3.1)
pub const RECENT_LIMIT: usize = 10;

/// 수정 결과: 실제로 바뀌었는지 여부를 함께 돌려준다 (BR-5.3)
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateOutcome {
    pub entry: Entry,
    pub changed: bool,
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn find_mut<'a>(ledger: &'a mut LedgerFile, id: &str) -> AppResult<&'a mut Entry> {
    ledger
        .entries
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::not_found("항목을 찾을 수 없습니다"))
}

/// 항목 추가 (BR-4)
pub fn add(ledger: &mut LedgerFile, input: &EntryInput, pc: &str, now: Timestamp) -> AppResult<Entry> {
    let v = validate_entry_input(input)?;
    let entry = Entry {
        id: new_id(),
        name: v.name,
        amount_won: v.amount_won,
        side: v.side,
        tickets: v.tickets,
        memo: v.memo,
        received_at: now,
        updated_at: now,
        deleted: false,
        source_pc: pc.to_string(),
    };
    ledger.entries.push(entry.clone());
    Ok(entry)
}

/// 항목 수정 (BR-5)
pub fn update(
    ledger: &mut LedgerFile,
    id: &str,
    input: &EntryInput,
    pc: &str,
    now: Timestamp,
) -> AppResult<UpdateOutcome> {
    let v = validate_entry_input(input)?;
    let entry = find_mut(ledger, id)?;
    if entry.deleted {
        return Err(AppError::state("삭제된 항목입니다"));
    }
    let before = entry.to_input();
    if before == v {
        return Ok(UpdateOutcome { entry: entry.clone(), changed: false });
    }
    entry.apply_input(&v);
    entry.updated_at = now;
    let result = entry.clone();
    ledger.history.push(HistoryRecord {
        id: new_id(),
        entry_id: id.to_string(),
        kind: HistoryKind::Update,
        at: now,
        before,
        after: Some(v),
        pc_name: pc.to_string(),
    });
    Ok(UpdateOutcome { entry: result, changed: true })
}

/// 항목 삭제 - 소프트 삭제 (BR-6.2, 6.3)
pub fn delete(ledger: &mut LedgerFile, id: &str, pc: &str, now: Timestamp) -> AppResult<()> {
    let entry = find_mut(ledger, id)?;
    if entry.deleted {
        return Err(AppError::state("이미 삭제된 항목입니다"));
    }
    entry.deleted = true;
    entry.updated_at = now;
    let before = entry.to_input();
    ledger.history.push(HistoryRecord {
        id: new_id(),
        entry_id: id.to_string(),
        kind: HistoryKind::Delete,
        at: now,
        before,
        after: None,
        pc_name: pc.to_string(),
    });
    Ok(())
}

/// 삭제 항목 복구 (BR-6.4, 6.5)
pub fn restore(ledger: &mut LedgerFile, id: &str, pc: &str, now: Timestamp) -> AppResult<Entry> {
    let entry = find_mut(ledger, id)?;
    if !entry.deleted {
        return Err(AppError::state("삭제된 항목이 아닙니다"));
    }
    entry.deleted = false;
    entry.updated_at = now;
    let snapshot = entry.to_input();
    let result = entry.clone();
    ledger.history.push(HistoryRecord {
        id: new_id(),
        entry_id: id.to_string(),
        kind: HistoryKind::Restore,
        at: now,
        before: snapshot.clone(),
        after: Some(snapshot),
        pc_name: pc.to_string(),
    });
    Ok(result)
}

/// 최근 목록: 삭제 제외, 접수 시각 내림차순 (BR-7)
pub fn recent(ledger: &LedgerFile, limit: usize) -> Vec<Entry> {
    let mut list: Vec<Entry> = ledger.active_entries().cloned().collect();
    list.sort_by(|a, b| b.received_at.cmp(&a.received_at).then_with(|| b.id.cmp(&a.id)));
    list.truncate(limit);
    list
}

/// 합계 계산 (BR-8.7, 8.9)
pub fn summarize<'a>(entries: impl IntoIterator<Item = &'a Entry>) -> Summary {
    let mut s = Summary::default();
    for e in entries {
        let add = |t: &mut SideSummary| {
            t.amount_won += e.amount_won;
            t.count += 1;
            t.tickets += e.tickets;
        };
        add(&mut s.total);
        match e.side {
            Side::Groom => add(&mut s.groom),
            Side::Bride => add(&mut s.bride),
        }
    }
    s
}

/// 동순위 정렬: 접수 시각 오름차순 → id (BR-8.5)
fn tie_break(a: &Entry, b: &Entry) -> Ordering {
    a.received_at.cmp(&b.received_at).then_with(|| a.id.cmp(&b.id))
}

/// 이름 비교: 유니코드 순서 = 한글 음절 가나다순 (BR-8.4)
pub fn compare_names(a: &str, b: &str) -> Ordering {
    a.to_lowercase().cmp(&b.to_lowercase())
}

/// 정산 화면 조회 (BR-8)
pub fn query(ledger: &LedgerFile, q: &EntryQuery) -> QueryResult {
    let active: Vec<&Entry> = ledger.active_entries().collect();
    let total_summary = summarize(active.iter().copied());

    let term = q.search.as_deref().unwrap_or("").trim().to_lowercase();
    let mut filtered: Vec<Entry> = active
        .into_iter()
        .filter(|e| term.is_empty() || e.name.to_lowercase().contains(&term))
        .filter(|e| q.side.is_none_or(|s| e.side == s))
        .cloned()
        .collect();

    filtered.sort_by(|a, b| {
        let primary = match q.sort_by {
            SortBy::Name => compare_names(&a.name, &b.name),
            SortBy::Amount => a.amount_won.cmp(&b.amount_won),
            SortBy::ReceivedAt => a.received_at.cmp(&b.received_at),
        };
        let primary = match q.sort_dir {
            SortDir::Asc => primary,
            SortDir::Desc => primary.reverse(),
        };
        primary.then_with(|| tie_break(a, b))
    });

    let filtered_summary = summarize(filtered.iter());
    QueryResult { entries: filtered, filtered_summary, total_summary }
}

/// 변경 이력 조회 (BR-9): 시각 내림차순, 복구 가능 여부 포함
pub fn history(ledger: &LedgerFile, entry_id: Option<&str>) -> Vec<HistoryItem> {
    let mut records: Vec<&HistoryRecord> = ledger
        .history
        .iter()
        .filter(|h| entry_id.is_none_or(|id| h.entry_id == id))
        .collect();
    records.sort_by(|a, b| b.at.cmp(&a.at).then_with(|| b.id.cmp(&a.id)));

    records
        .into_iter()
        .map(|h| {
            let entry = ledger.entries.iter().find(|e| e.id == h.entry_id);
            let entry_name = entry.map(|e| e.name.clone()).unwrap_or_else(|| h.before.name.clone());
            let can_restore = h.kind == HistoryKind::Delete
                && entry.is_some_and(|e| e.deleted)
                && latest_delete_id(ledger, &h.entry_id) == Some(h.id.as_str());
            HistoryItem { record: h.clone(), entry_name, can_restore }
        })
        .collect()
}

/// 항목의 가장 최근 삭제 이력 id
fn latest_delete_id<'a>(ledger: &'a LedgerFile, entry_id: &str) -> Option<&'a str> {
    ledger
        .history
        .iter()
        .filter(|h| h.entry_id == entry_id && h.kind == HistoryKind::Delete)
        .max_by(|a, b| a.at.cmp(&b.at).then_with(|| a.id.cmp(&b.id)))
        .map(|h| h.id.as_str())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::{LedgerKind, SortBy, SortDir};
    use chrono::{DateTime, Duration};

    pub fn t(s: &str) -> Timestamp {
        DateTime::parse_from_rfc3339(s).unwrap()
    }

    pub fn base() -> Timestamp {
        t("2026-11-21T11:00:00+09:00")
    }

    pub fn input(name: &str, amount: i64, side: Side) -> EntryInput {
        EntryInput { name: name.into(), amount_won: amount, side, tickets: 1, memo: String::new() }
    }

    pub fn ledger() -> LedgerFile {
        LedgerFile::new(LedgerKind::Groom, "PC-1", base())
    }

    fn q(sort_by: SortBy, sort_dir: SortDir) -> EntryQuery {
        EntryQuery { search: None, side: None, sort_by, sort_dir }
    }

    #[test]
    fn add_assigns_metadata_without_history() {
        let mut l = ledger();
        let e = add(&mut l, &input(" 김민수 ", 50_000, Side::Groom), "PC-1", base()).unwrap();
        assert_eq!(e.name, "김민수");
        assert_eq!(e.received_at, base());
        assert_eq!(e.updated_at, base());
        assert!(!e.deleted);
        assert_eq!(e.source_pc, "PC-1");
        assert_eq!(e.id.len(), 36);
        assert_eq!(l.entries.len(), 1);
        assert!(l.history.is_empty());
    }

    #[test]
    fn add_rejects_invalid_input() {
        let mut l = ledger();
        assert!(matches!(
            add(&mut l, &input("", 50_000, Side::Groom), "PC-1", base()),
            Err(AppError::Validation { .. })
        ));
        assert!(l.entries.is_empty());
    }

    #[test]
    fn update_records_history_only_when_changed() {
        let mut l = ledger();
        let e = add(&mut l, &input("김민슈", 50_000, Side::Groom), "PC-1", base()).unwrap();
        let later = base() + Duration::minutes(5);

        let same = update(&mut l, &e.id, &input("김민슈", 50_000, Side::Groom), "PC-2", later).unwrap();
        assert!(!same.changed);
        assert!(l.history.is_empty());
        assert_eq!(l.entries[0].updated_at, base());

        let out = update(&mut l, &e.id, &input("김민수", 100_000, Side::Bride), "PC-2", later).unwrap();
        assert!(out.changed);
        assert_eq!(out.entry.name, "김민수");
        assert_eq!(out.entry.received_at, base());
        assert_eq!(out.entry.updated_at, later);
        assert_eq!(out.entry.source_pc, "PC-1");
        assert_eq!(l.history.len(), 1);
        let h = &l.history[0];
        assert_eq!(h.kind, HistoryKind::Update);
        assert_eq!(h.before.name, "김민슈");
        assert_eq!(h.after.as_ref().unwrap().amount_won, 100_000);
        assert_eq!(h.pc_name, "PC-2");
    }

    #[test]
    fn delete_and_restore_flow() {
        let mut l = ledger();
        let e = add(&mut l, &input("김민수", 50_000, Side::Groom), "PC-1", base()).unwrap();
        let t1 = base() + Duration::minutes(1);
        let t2 = base() + Duration::minutes(2);

        delete(&mut l, &e.id, "PC-1", t1).unwrap();
        assert!(l.entries[0].deleted);
        assert_eq!(l.entries[0].updated_at, t1);
        assert!(matches!(delete(&mut l, &e.id, "PC-1", t1), Err(AppError::State { .. })));
        assert!(matches!(
            update(&mut l, &e.id, &input("김", 1_000, Side::Groom), "PC-1", t1),
            Err(AppError::State { .. })
        ));

        let items = history(&l, None);
        assert_eq!(items.len(), 1);
        assert!(items[0].can_restore);

        let r = restore(&mut l, &e.id, "PC-1", t2).unwrap();
        assert!(!r.deleted);
        assert_eq!(r.updated_at, t2);
        assert!(matches!(restore(&mut l, &e.id, "PC-1", t2), Err(AppError::State { .. })));

        let items = history(&l, Some(&e.id));
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].record.kind, HistoryKind::Restore);
        assert!(items.iter().all(|i| !i.can_restore));
    }

    #[test]
    fn can_restore_only_latest_delete() {
        let mut l = ledger();
        let e = add(&mut l, &input("김민수", 50_000, Side::Groom), "PC-1", base()).unwrap();
        delete(&mut l, &e.id, "PC-1", base() + Duration::minutes(1)).unwrap();
        restore(&mut l, &e.id, "PC-1", base() + Duration::minutes(2)).unwrap();
        delete(&mut l, &e.id, "PC-1", base() + Duration::minutes(3)).unwrap();
        let items = history(&l, None);
        let restorable: Vec<_> = items.iter().filter(|i| i.can_restore).collect();
        assert_eq!(restorable.len(), 1);
        assert_eq!(restorable[0].record.at, base() + Duration::minutes(3));
    }

    #[test]
    fn unknown_id_is_not_found() {
        let mut l = ledger();
        assert!(matches!(delete(&mut l, "nope", "PC", base()), Err(AppError::NotFound { .. })));
        assert!(matches!(restore(&mut l, "nope", "PC", base()), Err(AppError::NotFound { .. })));
    }

    #[test]
    fn recent_returns_latest_ten_active() {
        let mut l = ledger();
        for i in 0..12 {
            add(&mut l, &input(&format!("하객{i}"), 50_000, Side::Groom), "PC", base() + Duration::minutes(i)).unwrap();
        }
        let id11 = l.entries[11].id.clone();
        delete(&mut l, &id11, "PC", base() + Duration::hours(1)).unwrap();
        let r = recent(&l, RECENT_LIMIT);
        assert_eq!(r.len(), 10);
        assert_eq!(r[0].name, "하객10");
        assert_eq!(r[9].name, "하객1");
    }

    fn sample() -> LedgerFile {
        let mut l = ledger();
        add(&mut l, &input("박지훈", 100_000, Side::Groom), "PC", base()).unwrap();
        add(&mut l, &input("김민수", 50_000, Side::Bride), "PC", base() + Duration::minutes(1)).unwrap();
        add(&mut l, &input("이서연", 50_000, Side::Groom), "PC", base() + Duration::minutes(2)).unwrap();
        let mut e = input("최유진", 200_000, Side::Bride);
        e.tickets = 2;
        add(&mut l, &e, "PC", base() + Duration::minutes(3)).unwrap();
        let del = add(&mut l, &input("삭제됨", 300_000, Side::Groom), "PC", base() + Duration::minutes(4)).unwrap();
        delete(&mut l, &del.id, "PC", base() + Duration::minutes(5)).unwrap();
        l
    }

    fn names(r: &QueryResult) -> Vec<&str> {
        r.entries.iter().map(|e| e.name.as_str()).collect()
    }

    #[test]
    fn query_sorts_and_excludes_deleted() {
        let l = sample();
        assert_eq!(names(&query(&l, &q(SortBy::Name, SortDir::Asc))), ["김민수", "박지훈", "이서연", "최유진"]);
        assert_eq!(names(&query(&l, &q(SortBy::ReceivedAt, SortDir::Desc))), ["최유진", "이서연", "김민수", "박지훈"]);
        // 같은 금액(50,000)은 접수 시각 오름차순
        assert_eq!(names(&query(&l, &q(SortBy::Amount, SortDir::Asc))), ["김민수", "이서연", "박지훈", "최유진"]);
        assert_eq!(names(&query(&l, &q(SortBy::Amount, SortDir::Desc))), ["최유진", "박지훈", "김민수", "이서연"]);
    }

    #[test]
    fn query_summaries() {
        let l = sample();
        let r = query(&l, &q(SortBy::Name, SortDir::Asc));
        assert_eq!(r.total_summary.total, SideSummary { amount_won: 400_000, count: 4, tickets: 5 });
        assert_eq!(r.total_summary.groom, SideSummary { amount_won: 150_000, count: 2, tickets: 2 });
        assert_eq!(r.total_summary.bride, SideSummary { amount_won: 250_000, count: 2, tickets: 3 });
        assert_eq!(r.filtered_summary, r.total_summary);
    }

    #[test]
    fn query_search_and_side_filter() {
        let l = sample();
        let mut qq = q(SortBy::Name, SortDir::Asc);
        qq.search = Some("  민수 ".into());
        let r = query(&l, &qq);
        assert_eq!(names(&r), ["김민수"]);
        assert_eq!(r.filtered_summary.total.amount_won, 50_000);
        assert_eq!(r.total_summary.total.amount_won, 400_000);

        let mut qq = q(SortBy::Name, SortDir::Asc);
        qq.side = Some(Side::Groom);
        let r = query(&l, &qq);
        assert_eq!(names(&r), ["박지훈", "이서연"]);
        assert_eq!(r.filtered_summary.total.amount_won, 150_000);

        let mut qq = q(SortBy::Name, SortDir::Asc);
        qq.search = Some("없는이름".into());
        assert!(query(&l, &qq).entries.is_empty());
    }

    #[test]
    fn search_ignores_ascii_case() {
        let mut l = ledger();
        add(&mut l, &input("John Kim", 50_000, Side::Groom), "PC", base()).unwrap();
        let mut qq = q(SortBy::Name, SortDir::Asc);
        qq.search = Some("JOHN".into());
        assert_eq!(query(&l, &qq).entries.len(), 1);
    }
}
