//! 통합 테스트: 결혼식 당일 운영 흐름 (PC 2대 → 병합 → 정산 → 내보내기)
//!
//! 화면 대신 Tauri 명령이 사용하는 것과 같은 상태 계층(app_state::Inner)을 직접 호출한다.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Duration};
use wedding_gift_app_lib::app_state::Inner;
use wedding_gift_app_lib::model::{EntryInput, EntryQuery, LedgerKind, Side, SortBy, SortDir, Timestamp};
use wedding_gift_app_lib::paths::{BRIDE_FILE, GROOM_FILE, SETTINGS_FILE};
use wedding_gift_app_lib::{export, ledger, merge, storage};

fn t0() -> Timestamp {
    DateTime::parse_from_rfc3339("2026-11-21T11:00:00+09:00").unwrap()
}

fn input(name: &str, won: i64, side: Side, tickets: i64) -> EntryInput {
    EntryInput { name: name.into(), amount_won: won, side, tickets, memo: String::new() }
}

fn add(pc: &mut Inner, name: &str, won: i64, side: Side, tickets: i64, at: Timestamp) -> String {
    let pc_name = pc.pc_name.clone();
    pc.mutate(|l| ledger::add(l, &input(name, won, side, tickets), &pc_name, at).map(|e| (e, true)))
        .unwrap()
        .id
}

fn all_query() -> EntryQuery {
    EntryQuery { search: None, side: None, sort_by: SortBy::ReceivedAt, sort_dir: SortDir::Asc }
}

fn start(dir: &Path, pc: &str) -> Inner {
    Inner::start(dir.to_path_buf(), None, pc.into(), t0())
}

#[test]
fn wedding_day_two_pcs_merge_and_export() {
    let groom_dir = tempfile::tempdir().unwrap();
    let bride_dir = tempfile::tempdir().unwrap();

    // 1. 각 PC 최초 설정
    let mut groom = start(groom_dir.path(), "GROOM-PC");
    let mut bride = start(bride_dir.path(), "BRIDE-PC");
    assert!(groom.view().needs_setup);
    groom.setup(Side::Groom, t0()).unwrap();
    bride.setup(Side::Bride, t0()).unwrap();
    assert!(groom_dir.path().join(GROOM_FILE).exists());
    assert!(bride_dir.path().join(BRIDE_FILE).exists());
    assert!(groom_dir.path().join(SETTINGS_FILE).exists());

    // 2. 접수: 신랑측 PC에서 신부측 하객도 한 명 받음
    let g1 = add(&mut groom, "김민슈", 50_000, Side::Groom, 1, t0());
    add(&mut groom, "박지훈", 100_000, Side::Groom, 2, t0() + Duration::minutes(1));
    let g3 = add(&mut groom, "이서연", 55_000, Side::Groom, 1, t0() + Duration::minutes(2));
    add(&mut groom, "최유진", 30_000, Side::Bride, 1, t0() + Duration::minutes(3));
    add(&mut bride, "강하늘", 100_000, Side::Bride, 2, t0() + Duration::minutes(1));
    add(&mut bride, "윤서준", 1_500_000, Side::Bride, 4, t0() + Duration::minutes(2));

    // 3. 오타 수정, 잘못 입력 삭제
    let pc = groom.pc_name.clone();
    groom
        .mutate(|l| ledger::update(l, &g1, &input("김민수", 50_000, Side::Groom, 1), &pc, t0() + Duration::minutes(10)).map(|o| (o.entry, o.changed)))
        .unwrap();
    groom.mutate(|l| ledger::delete(l, &g3, &pc, t0() + Duration::minutes(11)).map(|()| ((), true))).unwrap();

    // 4. 프로그램을 다시 켜도 데이터 유지
    let groom = start(groom_dir.path(), "GROOM-PC");
    assert_eq!(groom.ledger().unwrap().active_entries().count(), 3);
    assert_eq!(groom.ledger().unwrap().history.len(), 2);

    // 5. 신부측 파일을 USB로 신랑측 PC 폴더에 복사 → 병합
    let copied = groom_dir.path().join(BRIDE_FILE);
    fs::copy(bride_dir.path().join(BRIDE_FILE), &copied).unwrap();
    let merged_path = groom_dir.path().join("축의금_통합.json");
    let report = merge::merge_files(&groom_dir.path().join(GROOM_FILE), &copied, &merged_path, "GROOM-PC", t0() + Duration::hours(4)).unwrap();
    assert_eq!((report.from_a, report.from_b, report.merged, report.duplicates_resolved), (3, 2, 5, 0));

    // 6. 같은 파일을 실수로 다시 병합해도 금액이 늘지 않음
    let again = groom_dir.path().join("축의금_통합2.json");
    let report2 = merge::merge_files(&merged_path, &copied, &again, "GROOM-PC", t0() + Duration::hours(4)).unwrap();
    assert_eq!(report2.merged, 5);
    assert_eq!(report2.duplicates_resolved, 2);

    // 7. 통합 파일 열기 → 정산
    let mut groom = start(groom_dir.path(), "GROOM-PC");
    groom.open_file(&merged_path).unwrap();
    assert_eq!(groom.view().active_file_kind, Some(LedgerKind::Merged));
    let r = ledger::query(groom.ledger().unwrap(), &all_query());
    assert_eq!(r.total_summary.total.amount_won, 50_000 + 100_000 + 30_000 + 100_000 + 1_500_000);
    assert_eq!(r.total_summary.groom.amount_won, 150_000);
    assert_eq!(r.total_summary.bride.amount_won, 1_630_000);
    assert_eq!(r.total_summary.total.count, 5);
    assert_eq!(r.total_summary.total.tickets, 1 + 2 + 1 + 2 + 4);
    assert!(r.entries.iter().all(|e| e.name != "이서연"));
    assert!(r.entries.iter().any(|e| e.name == "김민수"));

    // 8. 통합 파일에서 삭제 → 복구 → 이력 확인
    let target = r.entries.iter().find(|e| e.name == "최유진").unwrap().id.clone();
    let pc = groom.pc_name.clone();
    groom.mutate(|l| ledger::delete(l, &target, &pc, t0() + Duration::hours(5)).map(|()| ((), true))).unwrap();
    let hist = ledger::history(groom.ledger().unwrap(), None);
    assert!(hist[0].can_restore);
    groom.mutate(|l| ledger::restore(l, &target, &pc, t0() + Duration::hours(5) + Duration::minutes(1)).map(|e| (e, true))).unwrap();
    assert_eq!(ledger::query(groom.ledger().unwrap(), &all_query()).entries.len(), 5);

    // 9. 다시 켜면 통합 파일이 열린 상태로 시작
    let groom = start(groom_dir.path(), "GROOM-PC");
    assert_eq!(groom.view().active_file_kind, Some(LedgerKind::Merged));

    // 10. 내보내기
    let csv = groom_dir.path().join("결과.csv");
    let xlsx = groom_dir.path().join("결과.xlsx");
    export::write_csv(groom.ledger().unwrap(), &csv).unwrap();
    export::write_xlsx(groom.ledger().unwrap(), &xlsx).unwrap();
    let text = fs::read_to_string(&csv).unwrap();
    let lines: Vec<&str> = text.trim_start_matches('\u{feff}').trim_end().split("\r\n").collect();
    assert_eq!(lines.len(), 6);
    assert!(lines[1].starts_with("1,신랑측,김민수,50000"));
    assert!(lines[3].starts_with("3,신부측,강하늘,100000"));
    assert!(fs::metadata(&xlsx).unwrap().len() > 1000);

    // 원본 파일은 그대로
    assert_eq!(storage::load_ledger(&copied).unwrap().entries.len(), 2);
}

#[test]
fn moved_folder_keeps_working() {
    // 폴더째 다른 PC(다른 경로)로 옮겨도 settings.json 의 파일 이름만으로 열린다
    let a = tempfile::tempdir().unwrap();
    let mut pc = start(a.path(), "PC-A");
    pc.setup(Side::Bride, t0()).unwrap();
    add(&mut pc, "김민수", 50_000, Side::Bride, 1, t0());

    let b = tempfile::tempdir().unwrap();
    for name in [BRIDE_FILE, SETTINGS_FILE] {
        fs::copy(a.path().join(name), b.path().join(name)).unwrap();
    }
    drop(a);
    let moved = start(b.path(), "PC-B");
    assert!(!moved.view().needs_setup);
    assert_eq!(moved.ledger().unwrap().entries.len(), 1);
    assert_eq!(moved.view().active_file_path.unwrap(), b.path().join(BRIDE_FILE).to_string_lossy());
}
