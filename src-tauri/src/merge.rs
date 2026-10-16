//! 두 데이터 파일 병합 (business-rules.md BR-10, business-logic-model.md 9절)

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::model::{AppError, AppResult, Entry, HistoryRecord, LedgerFile, LedgerKind, MergeReport, Timestamp};
use crate::storage;

/// 병합 통계
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeStats {
    pub from_a: i64,
    pub from_b: i64,
    pub merged: i64,
    pub duplicates_resolved: i64,
}

fn active_count(l: &LedgerFile) -> i64 {
    l.active_entries().count() as i64
}

/// 두 장부를 합친 새 장부를 만든다. 원본은 변경하지 않는다.
///
/// - 같은 id는 updatedAt이 늦은 쪽을 사용 (같으면 A 유지)
/// - 이력은 id 기준 합집합, 시각 오름차순
pub fn merge_ledgers(
    a: &LedgerFile,
    b: &LedgerFile,
    name_a: &str,
    name_b: &str,
    pc: &str,
    now: Timestamp,
) -> (LedgerFile, MergeStats) {
    let mut order: Vec<String> = Vec::new();
    let mut map: HashMap<String, Entry> = HashMap::new();
    let mut duplicates = 0i64;

    for e in &a.entries {
        if map.insert(e.id.clone(), e.clone()).is_none() {
            order.push(e.id.clone());
        }
    }
    for e in &b.entries {
        match map.get(&e.id) {
            Some(cur) => {
                duplicates += 1;
                // DateTime 비교는 오프셋을 반영한 실제 시점 기준
                if e.updated_at > cur.updated_at {
                    map.insert(e.id.clone(), e.clone());
                }
            }
            None => {
                order.push(e.id.clone());
                map.insert(e.id.clone(), e.clone());
            }
        }
    }
    let entries: Vec<Entry> = order.into_iter().filter_map(|id| map.remove(&id)).collect();

    let mut seen: HashSet<&str> = HashSet::new();
    let mut history: Vec<HistoryRecord> = Vec::new();
    for h in a.history.iter().chain(b.history.iter()) {
        if seen.insert(h.id.as_str()) {
            history.push(h.clone());
        }
    }
    history.sort_by(|x, y| x.at.cmp(&y.at).then_with(|| x.id.cmp(&y.id)));

    let mut result = LedgerFile::new(LedgerKind::Merged, pc, now);
    result.merged_from = vec![name_a.to_string(), name_b.to_string()];
    result.entries = entries;
    result.history = history;

    let stats = MergeStats {
        from_a: active_count(a),
        from_b: active_count(b),
        merged: active_count(&result),
        duplicates_resolved: duplicates,
    };
    (result, stats)
}

/// 경로 비교용 정규화. 파일이 아직 없으면(출력 경로) 부모 폴더를 정규화해 붙인다.
fn normalize(path: &Path) -> std::path::PathBuf {
    if let Ok(p) = path.canonicalize() {
        return p;
    }
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => parent
            .canonicalize()
            .map(|p| p.join(name))
            .unwrap_or_else(|_| path.to_path_buf()),
        _ => path.to_path_buf(),
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// 파일 A, B를 읽어 병합한 결과를 out에 저장한다 (BR-10.1 ~ 10.3)
pub fn merge_files(path_a: &Path, path_b: &Path, out: &Path, pc: &str, now: Timestamp) -> AppResult<MergeReport> {
    let (na, nb, no) = (normalize(path_a), normalize(path_b), normalize(out));
    if na == nb {
        return Err(AppError::state("서로 다른 두 파일을 선택해 주세요"));
    }
    if no == na || no == nb {
        return Err(AppError::state("원본 파일과 다른 이름으로 저장해 주세요"));
    }

    let a = storage::load_ledger(path_a)
        .map_err(|e| AppError::parse(format!("첫 번째 파일({})을 읽지 못했습니다: {e}", file_name(path_a))))?;
    let b = storage::load_ledger(path_b)
        .map_err(|e| AppError::parse(format!("두 번째 파일({})을 읽지 못했습니다: {e}", file_name(path_b))))?;

    let (merged, stats) = merge_ledgers(&a, &b, &file_name(path_a), &file_name(path_b), pc, now);
    storage::save_ledger(out, &merged)?;

    Ok(MergeReport {
        output_path: out.to_string_lossy().into_owned(),
        from_a: stats.from_a,
        from_b: stats.from_b,
        merged: stats.merged,
        duplicates_resolved: stats.duplicates_resolved,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{self, tests::{base, input, t}};
    use crate::model::Side;
    use chrono::Duration;

    fn groom_ledger(n: i64) -> LedgerFile {
        let mut l = LedgerFile::new(LedgerKind::Groom, "PC-G", base());
        for i in 0..n {
            ledger::add(&mut l, &input(&format!("신랑하객{i}"), 50_000, Side::Groom), "PC-G", base() + Duration::minutes(i)).unwrap();
        }
        l
    }

    fn bride_ledger(n: i64) -> LedgerFile {
        let mut l = LedgerFile::new(LedgerKind::Bride, "PC-B", base());
        for i in 0..n {
            ledger::add(&mut l, &input(&format!("신부하객{i}"), 100_000, Side::Bride), "PC-B", base() + Duration::minutes(i)).unwrap();
        }
        l
    }

    fn now() -> Timestamp {
        t("2026-11-21T15:00:00+09:00")
    }

    #[test]
    fn scenario1_disjoint_files() {
        let (m, s) = merge_ledgers(&groom_ledger(30), &bride_ledger(25), "a.json", "b.json", "PC-M", now());
        assert_eq!(s, MergeStats { from_a: 30, from_b: 25, merged: 55, duplicates_resolved: 0 });
        assert_eq!(m.kind, LedgerKind::Merged);
        assert_eq!(m.merged_from, ["a.json", "b.json"]);
        assert_eq!(ledger::summarize(m.active_entries()).total.amount_won, 30 * 50_000 + 25 * 100_000);
    }

    #[test]
    fn scenario2_remerge_does_not_double_count() {
        let g = groom_ledger(30);
        let (m1, _) = merge_ledgers(&g, &bride_ledger(25), "a", "b", "PC-M", now());
        let (m2, s) = merge_ledgers(&m1, &g, "m", "a", "PC-M", now());
        assert_eq!(s.merged, 55);
        assert_eq!(s.duplicates_resolved, 30);
        assert_eq!(ledger::summarize(m2.active_entries()).total.amount_won, ledger::summarize(m1.active_entries()).total.amount_won);
    }

    #[test]
    fn scenario3_newer_update_wins_in_either_order() {
        let g = groom_ledger(1);
        let (mut m, _) = merge_ledgers(&g, &bride_ledger(1), "a", "b", "PC-M", now());
        let id = g.entries[0].id.clone();
        ledger::update(&mut m, &id, &input("고친이름", 70_000, Side::Groom), "PC-M", base() + Duration::minutes(50)).unwrap();

        for (x, y) in [(&m, &g), (&g, &m)] {
            let (r, _) = merge_ledgers(x, y, "x", "y", "PC-M", now());
            let e = r.entries.iter().find(|e| e.id == id).unwrap();
            assert_eq!(e.name, "고친이름");
            assert_eq!(e.amount_won, 70_000);
        }
    }

    #[test]
    fn scenario4_deleted_entry_does_not_come_back() {
        let g = groom_ledger(2);
        let (mut m, _) = merge_ledgers(&g, &bride_ledger(0), "a", "b", "PC-M", now());
        let id = g.entries[0].id.clone();
        ledger::delete(&mut m, &id, "PC-M", base() + Duration::hours(1)).unwrap();

        let (r, s) = merge_ledgers(&g, &m, "a", "m", "PC-M", now());
        assert!(r.entries.iter().find(|e| e.id == id).unwrap().deleted);
        assert_eq!(s.merged, 1);
        assert_eq!(r.history.len(), 1);
    }

    #[test]
    fn compares_instants_not_strings() {
        let g = groom_ledger(1);
        let mut other = g.clone();
        // +00:00 03:30 = +09:00 12:30 → 11:00(+09:00)보다 늦음. 문자열로는 "03" < "11"
        other.entries[0].updated_at = t("2026-11-21T03:30:00+00:00");
        other.entries[0].name = "UTC수정".into();
        let (r, _) = merge_ledgers(&g, &other, "a", "b", "PC", now());
        assert_eq!(r.entries[0].name, "UTC수정");
    }

    #[test]
    fn equal_timestamps_keep_a() {
        let g = groom_ledger(1);
        let mut other = g.clone();
        other.entries[0].name = "B쪽".into();
        let (r, _) = merge_ledgers(&g, &other, "a", "b", "PC", now());
        assert_eq!(r.entries[0].name, "신랑하객0");
    }

    #[test]
    fn history_is_unioned_and_sorted() {
        let mut g = groom_ledger(2);
        let id0 = g.entries[0].id.clone();
        let id1 = g.entries[1].id.clone();
        let mut b = g.clone();
        ledger::update(&mut g, &id0, &input("A수정", 50_000, Side::Groom), "PC", base() + Duration::minutes(20)).unwrap();
        ledger::delete(&mut b, &id1, "PC", base() + Duration::minutes(10)).unwrap();
        let (r, _) = merge_ledgers(&g, &b, "a", "b", "PC", now());
        assert_eq!(r.history.len(), 2);
        assert!(r.history[0].at < r.history[1].at);
        let (r2, _) = merge_ledgers(&r, &g, "r", "a", "PC", now());
        assert_eq!(r2.history.len(), 2);
    }

    #[test]
    fn merge_files_rejects_same_paths_and_keeps_sources() {
        let dir = tempfile::tempdir().unwrap();
        let pa = dir.path().join("축의금_신랑측.json");
        let pb = dir.path().join("축의금_신부측.json");
        let out = dir.path().join("축의금_통합.json");
        storage::save_ledger(&pa, &groom_ledger(3)).unwrap();
        storage::save_ledger(&pb, &bride_ledger(2)).unwrap();
        let before_a = std::fs::read(&pa).unwrap();

        // 시나리오 5: 같은 파일
        assert!(matches!(merge_files(&pa, &pa, &out, "PC", now()), Err(AppError::State { .. })));
        assert!(matches!(merge_files(&pa, &pb, &pa, "PC", now()), Err(AppError::State { .. })));

        let report = merge_files(&pa, &pb, &out, "PC", now()).unwrap();
        assert_eq!(report.merged, 5);
        assert_eq!(report.from_a, 3);
        assert_eq!(report.from_b, 2);
        assert_eq!(std::fs::read(&pa).unwrap(), before_a);
        let loaded = storage::load_ledger(&out).unwrap();
        assert_eq!(loaded.entries.len(), 5);
        assert_eq!(loaded.merged_from, ["축의금_신랑측.json", "축의금_신부측.json"]);
    }

    #[test]
    fn merge_files_reports_unreadable_file() {
        let dir = tempfile::tempdir().unwrap();
        let pa = dir.path().join("a.json");
        let pb = dir.path().join("b.json");
        std::fs::write(&pa, "깨진 파일").unwrap();
        storage::save_ledger(&pb, &bride_ledger(1)).unwrap();
        let err = merge_files(&pa, &pb, &dir.path().join("out.json"), "PC", now()).unwrap_err();
        assert!(err.to_string().contains("a.json"));
    }
}
