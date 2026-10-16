//! CSV / XLSX 내보내기 (business-rules.md BR-11)

use std::path::Path;

use rust_xlsxwriter::{Format, FormatAlign, Workbook};

use crate::ledger::{compare_names, summarize};
use crate::model::{AppError, AppResult, Entry, LedgerFile, Side, SideSummary};
use crate::storage::write_atomic;

pub const HEADERS: [&str; 8] = ["번호", "구분", "이름", "금액", "식권 수", "메모", "접수 시각", "입력 PC"];

/// 내보내기 대상: 삭제 제외, 구분(신랑측→신부측) → 이름 가나다순 → 접수 시각 (BR-11.1, 11.2)
pub fn export_rows(ledger: &LedgerFile) -> Vec<Entry> {
    let mut rows: Vec<Entry> = ledger.active_entries().cloned().collect();
    rows.sort_by(|a, b| {
        a.side
            .cmp(&b.side)
            .then_with(|| compare_names(&a.name, &b.name))
            .then_with(|| a.received_at.cmp(&b.received_at))
            .then_with(|| a.id.cmp(&b.id))
    });
    rows
}

/// 엑셀이 수식으로 해석하지 않도록 처리 (BR-11.7)
pub fn sanitize_text(value: &str) -> String {
    match value.chars().next() {
        Some('=' | '+' | '-' | '@') => format!("'{value}"),
        _ => value.to_string(),
    }
}

fn format_time(e: &Entry) -> String {
    e.received_at.format("%Y-%m-%d %H:%M").to_string()
}

/// CSV 필드 이스케이프 (RFC 4180)
fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// CSV 문자열 생성: UTF-8 BOM, CRLF (BR-11.6)
pub fn build_csv(rows: &[Entry]) -> String {
    let mut out = String::from("\u{feff}");
    out.push_str(&HEADERS.join(","));
    out.push_str("\r\n");
    for (i, e) in rows.iter().enumerate() {
        let fields = [
            (i + 1).to_string(),
            e.side.label().to_string(),
            csv_field(&sanitize_text(&e.name)),
            e.amount_won.to_string(),
            e.tickets.to_string(),
            csv_field(&sanitize_text(&e.memo)),
            format_time(e),
            csv_field(&sanitize_text(&e.source_pc)),
        ];
        out.push_str(&fields.join(","));
        out.push_str("\r\n");
    }
    out
}

pub fn write_csv(ledger: &LedgerFile, path: &Path) -> AppResult<()> {
    write_atomic(path, build_csv(&export_rows(ledger)).as_bytes())
}

fn xlsx_error(e: impl std::fmt::Display) -> AppError {
    AppError::io(format!("엑셀 파일을 만들지 못했습니다: {e}"))
}

/// XLSX 바이트 생성: "내역" 시트 + "요약" 시트 (BR-11.8, 11.9)
pub fn build_xlsx(rows: &[Entry]) -> AppResult<Vec<u8>> {
    let mut wb = Workbook::new();
    let header = Format::new().set_bold().set_align(FormatAlign::Center).set_background_color("#F2F2F2");
    let money = Format::new().set_num_format("#,##0");

    {
        let ws = wb.add_worksheet();
        ws.set_name("내역").map_err(xlsx_error)?;
        for (c, h) in HEADERS.iter().enumerate() {
            ws.write_string_with_format(0, c as u16, *h, &header).map_err(xlsx_error)?;
        }
        for (i, e) in rows.iter().enumerate() {
            let r = (i + 1) as u32;
            ws.write_number(r, 0, (i + 1) as f64).map_err(xlsx_error)?;
            ws.write_string(r, 1, e.side.label()).map_err(xlsx_error)?;
            ws.write_string(r, 2, sanitize_text(&e.name)).map_err(xlsx_error)?;
            ws.write_number_with_format(r, 3, e.amount_won as f64, &money).map_err(xlsx_error)?;
            ws.write_number(r, 4, e.tickets as f64).map_err(xlsx_error)?;
            ws.write_string(r, 5, sanitize_text(&e.memo)).map_err(xlsx_error)?;
            ws.write_string(r, 6, format_time(e)).map_err(xlsx_error)?;
            ws.write_string(r, 7, sanitize_text(&e.source_pc)).map_err(xlsx_error)?;
        }
        ws.set_freeze_panes(1, 0).map_err(xlsx_error)?;
        for (c, w) in [6.0, 8.0, 14.0, 12.0, 8.0, 30.0, 18.0, 16.0].iter().enumerate() {
            ws.set_column_width(c as u16, *w).map_err(xlsx_error)?;
        }
    }

    {
        let s = summarize(rows.iter());
        let ws = wb.add_worksheet();
        ws.set_name("요약").map_err(xlsx_error)?;
        for (c, h) in ["구분", "건수", "금액", "식권 수"].iter().enumerate() {
            ws.write_string_with_format(0, c as u16, *h, &header).map_err(xlsx_error)?;
        }
        let lines: [(&str, SideSummary); 3] = [
            (Side::Groom.label(), s.groom),
            (Side::Bride.label(), s.bride),
            ("합계", s.total),
        ];
        for (i, (label, v)) in lines.iter().enumerate() {
            let r = (i + 1) as u32;
            ws.write_string(r, 0, *label).map_err(xlsx_error)?;
            ws.write_number(r, 1, v.count as f64).map_err(xlsx_error)?;
            ws.write_number_with_format(r, 2, v.amount_won as f64, &money).map_err(xlsx_error)?;
            ws.write_number(r, 3, v.tickets as f64).map_err(xlsx_error)?;
        }
        for (c, w) in [10.0, 8.0, 16.0, 10.0].iter().enumerate() {
            ws.set_column_width(c as u16, *w).map_err(xlsx_error)?;
        }
    }

    wb.save_to_buffer().map_err(xlsx_error)
}

pub fn write_xlsx(ledger: &LedgerFile, path: &Path) -> AppResult<()> {
    let bytes = build_xlsx(&export_rows(ledger))?;
    write_atomic(path, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{self, tests::{base, input, ledger as new_ledger}};
    use chrono::Duration;

    fn sample() -> LedgerFile {
        let mut l = new_ledger();
        ledger::add(&mut l, &input("최유진", 50_000, Side::Bride), "PC-B", base()).unwrap();
        ledger::add(&mut l, &input("박지훈", 100_000, Side::Groom), "PC-G", base() + Duration::minutes(1)).unwrap();
        ledger::add(&mut l, &input("김민수", 50_000, Side::Groom), "PC-G", base() + Duration::minutes(2)).unwrap();
        ledger::add(&mut l, &input("강하늘", 30_000, Side::Bride), "PC-B", base() + Duration::minutes(3)).unwrap();
        let mut memo = input("이서연", 70_000, Side::Groom);
        memo.memo = "회사, \"팀장\"".into();
        ledger::add(&mut l, &memo, "PC-G", base() + Duration::minutes(4)).unwrap();
        let del = ledger::add(&mut l, &input("삭제됨", 10_000, Side::Groom), "PC-G", base()).unwrap();
        ledger::delete(&mut l, &del.id, "PC-G", base() + Duration::minutes(5)).unwrap();
        l
    }

    #[test]
    fn rows_sorted_by_side_then_name_and_exclude_deleted() {
        let names: Vec<String> = export_rows(&sample()).into_iter().map(|e| e.name).collect();
        assert_eq!(names, ["김민수", "박지훈", "이서연", "강하늘", "최유진"]);
    }

    #[test]
    fn csv_format() {
        let csv = build_csv(&export_rows(&sample()));
        assert!(csv.starts_with('\u{feff}'));
        let lines: Vec<&str> = csv.trim_start_matches('\u{feff}').split("\r\n").collect();
        assert_eq!(lines[0], "번호,구분,이름,금액,식권 수,메모,접수 시각,입력 PC");
        assert_eq!(lines[1], "1,신랑측,김민수,50000,1,,2026-11-21 11:02,PC-G");
        assert_eq!(lines[3], "3,신랑측,이서연,70000,1,\"회사, \"\"팀장\"\"\",2026-11-21 11:04,PC-G");
        assert_eq!(lines[4], "4,신부측,강하늘,30000,1,,2026-11-21 11:03,PC-B");
        assert_eq!(lines.len(), 7); // 헤더 + 5행 + 마지막 빈 문자열
    }

    #[test]
    fn formula_like_text_is_escaped() {
        assert_eq!(sanitize_text("=SUM(A1)"), "'=SUM(A1)");
        assert_eq!(sanitize_text("-친구"), "'-친구");
        assert_eq!(sanitize_text("+1"), "'+1");
        assert_eq!(sanitize_text("@x"), "'@x");
        assert_eq!(sanitize_text("김민수"), "김민수");
        assert_eq!(sanitize_text(""), "");
    }

    #[test]
    fn xlsx_is_created() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("축의금.xlsx");
        write_xlsx(&sample(), &p).unwrap();
        let bytes = std::fs::read(&p).unwrap();
        assert!(bytes.len() > 1000);
        assert_eq!(&bytes[..2], b"PK"); // xlsx = zip
    }

    #[test]
    fn csv_file_is_written() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("축의금.csv");
        write_csv(&sample(), &p).unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(text.contains("박지훈"));
        assert!(!text.contains("삭제됨"));
    }
}
