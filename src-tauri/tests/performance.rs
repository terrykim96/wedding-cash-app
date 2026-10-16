//! 성능 테스트: 하객이 몰릴 때 저장/조회가 충분히 빠른지 확인
//!
//! 기본 실행에서는 건너뛴다. 실행: `cargo test --release --test performance -- --ignored --nocapture`

use std::time::Instant;

use chrono::{DateTime, Duration};
use wedding_gift_app_lib::app_state::Inner;
use wedding_gift_app_lib::model::{EntryInput, EntryQuery, Side, SortBy, SortDir};
use wedding_gift_app_lib::{export, ledger, merge};

/// 대형 결혼식 기준 하객 수
const GUESTS: usize = 1_000;

#[test]
#[ignore = "성능 측정용. --ignored 로 실행"]
fn save_and_query_with_1000_guests() {
    let dir = tempfile::tempdir().unwrap();
    let t0 = DateTime::parse_from_rfc3339("2026-11-21T11:00:00+09:00").unwrap();
    let mut pc = Inner::start(dir.path().to_path_buf(), None, "PC".into(), t0);
    pc.setup(Side::Groom, t0).unwrap();

    // 1건 저장마다 파일 전체를 원자적으로 다시 씀 → 마지막 구간이 가장 느림
    let mut worst_ms = 0f64;
    let mut last_100_total = 0f64;
    let start_all = Instant::now();
    for i in 0..GUESTS {
        let input = EntryInput {
            name: format!("하객{i:04}"),
            amount_won: 50_000 + (i as i64 % 4) * 50_000,
            side: if i % 2 == 0 { Side::Groom } else { Side::Bride },
            tickets: 1,
            memo: "회사 동료".into(),
        };
        let at = t0 + Duration::seconds(i as i64);
        let started = Instant::now();
        pc.mutate(|l| ledger::add(l, &input, "PC", at).map(|e| (e, true))).unwrap();
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        worst_ms = worst_ms.max(ms);
        if i >= GUESTS - 100 {
            last_100_total += ms;
        }
    }
    let total_s = start_all.elapsed().as_secs_f64();
    let avg_last_100 = last_100_total / 100.0;

    let q = EntryQuery { search: Some("하객09".into()), side: Some(Side::Bride), sort_by: SortBy::Name, sort_dir: SortDir::Asc };
    let started = Instant::now();
    let r = ledger::query(pc.ledger().unwrap(), &q);
    let query_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert!(!r.entries.is_empty());

    let xlsx = dir.path().join("out.xlsx");
    let started = Instant::now();
    export::write_xlsx(pc.ledger().unwrap(), &xlsx).unwrap();
    let xlsx_ms = started.elapsed().as_secs_f64() * 1000.0;

    let other = dir.path().join("copy.json");
    std::fs::copy(pc.active_path.as_ref().unwrap(), &other).unwrap();
    let started = Instant::now();
    merge::merge_files(pc.active_path.as_ref().unwrap(), &other, &dir.path().join("m.json"), "PC", t0).unwrap();
    let merge_ms = started.elapsed().as_secs_f64() * 1000.0;

    let file_kb = std::fs::metadata(pc.active_path.as_ref().unwrap()).unwrap().len() as f64 / 1024.0;
    println!("PERF guests={GUESTS} file={file_kb:.0}KB total={total_s:.2}s save_avg_last100={avg_last_100:.1}ms save_worst={worst_ms:.1}ms query={query_ms:.1}ms xlsx={xlsx_ms:.1}ms merge={merge_ms:.1}ms");

    // 목표: 1,000명째에도 저장 1건이 체감 지연 없이(200ms 이하) 끝남
    assert!(avg_last_100 < 200.0, "저장이 너무 느림: {avg_last_100:.1}ms");
    assert!(query_ms < 100.0, "조회가 너무 느림: {query_ms:.1}ms");
}
