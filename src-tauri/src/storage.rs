//! 데이터 파일 읽기/쓰기 (business-rules.md BR-13)

use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use crate::model::{AppError, AppResult, LedgerFile, LedgerKind, Timestamp, SCHEMA_VERSION};

/// 같은 폴더의 임시 파일 경로: `<파일명>.tmp`
fn tmp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().map(|n| n.to_os_string()).unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}

fn save_error(e: impl std::fmt::Display) -> AppError {
    AppError::io(format!("저장하지 못했습니다. 저장 위치(USB 등)를 확인해 주세요. ({e})"))
}

/// 원자적 쓰기: 임시 파일에 쓰고 디스크에 반영한 뒤 원래 파일로 교체한다 (BR-13.1)
pub fn write_atomic(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let tmp = tmp_path(path);
    let result = (|| -> std::io::Result<()> {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        // Windows에서도 기존 파일을 교체한다 (MoveFileEx + REPLACE_EXISTING)
        fs::rename(&tmp, path)
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        return Err(save_error(e));
    }
    Ok(())
}

/// 데이터 파일 읽기. 실패해도 파일을 건드리지 않는다 (BR-13.4, 13.5)
pub fn load_ledger(path: &Path) -> AppResult<LedgerFile> {
    let bytes = fs::read(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => AppError::not_found(format!("파일이 없습니다: {}", path.display())),
        _ => AppError::io(format!("파일을 읽지 못했습니다: {e}")),
    })?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| AppError::parse("축의금 데이터 파일이 아니거나 손상된 파일입니다"))?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);

    // 버전을 먼저 확인해 새 버전 파일을 정확히 안내한다
    let raw: serde_json::Value = serde_json::from_str(text)
        .map_err(|_| AppError::parse("축의금 데이터 파일이 아니거나 손상된 파일입니다"))?;
    let version = raw.get("schemaVersion").and_then(|v| v.as_u64());
    match version {
        Some(v) if v > SCHEMA_VERSION as u64 => {
            return Err(AppError::parse("더 새로운 버전에서 만든 파일입니다. 프로그램을 최신 버전으로 바꿔 주세요"));
        }
        Some(_) => {}
        None => return Err(AppError::parse("축의금 데이터 파일이 아니거나 손상된 파일입니다")),
    }
    serde_json::from_value(raw).map_err(|e| AppError::parse(format!("데이터 파일 형식이 올바르지 않습니다: {e}")))
}

/// 데이터 파일 저장 (원자적 쓰기)
pub fn save_ledger(path: &Path, ledger: &LedgerFile) -> AppResult<()> {
    let json = serde_json::to_vec_pretty(ledger).map_err(save_error)?;
    write_atomic(path, &json)
}

/// 파일이 없을 때만 새 장부를 만들고, 있으면 읽는다
pub fn open_or_create_ledger(path: &Path, kind: LedgerKind, pc: &str, now: Timestamp) -> AppResult<LedgerFile> {
    if path.exists() {
        return load_ledger(path);
    }
    let ledger = LedgerFile::new(kind, pc, now);
    save_ledger(path, &ledger)?;
    Ok(ledger)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{self, tests::{base, input}};
    use crate::model::Side;

    fn sample() -> LedgerFile {
        let mut l = LedgerFile::new(LedgerKind::Groom, "PC", base());
        let e = ledger::add(&mut l, &input("김민수", 55_000, Side::Groom), "PC", base()).unwrap();
        ledger::update(&mut l, &e.id, &input("김민수", 60_000, Side::Groom), "PC", base()).unwrap();
        l
    }

    #[test]
    fn round_trip_preserves_data() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("축의금_신랑측.json");
        let l = sample();
        save_ledger(&p, &l).unwrap();
        assert_eq!(load_ledger(&p).unwrap(), l);
        assert!(!tmp_path(&p).exists());
    }

    #[test]
    fn overwrite_replaces_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("data.json");
        let mut l = sample();
        save_ledger(&p, &l).unwrap();
        l.entries.clear();
        save_ledger(&p, &l).unwrap();
        assert!(load_ledger(&p).unwrap().entries.is_empty());
        assert!(!tmp_path(&p).exists());
    }

    #[test]
    fn json_uses_camel_case_fields() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("data.json");
        save_ledger(&p, &sample()).unwrap();
        let text = fs::read_to_string(&p).unwrap();
        for key in ["schemaVersion", "ledgerId", "amountWon", "receivedAt", "sourcePc", "entryId", "pcName"] {
            assert!(text.contains(key), "{key} 없음");
        }
        assert!(text.contains("\"side\": \"groom\""));
        assert!(text.contains("\"kind\": \"update\""));
    }

    #[test]
    fn broken_file_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("data.json");
        fs::write(&p, "{ not json").unwrap();
        assert!(matches!(load_ledger(&p), Err(AppError::Parse { .. })));
        assert!(matches!(
            open_or_create_ledger(&p, LedgerKind::Groom, "PC", base()),
            Err(AppError::Parse { .. })
        ));
        assert_eq!(fs::read_to_string(&p).unwrap(), "{ not json");
    }

    #[test]
    fn newer_schema_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("data.json");
        let mut v = serde_json::to_value(sample()).unwrap();
        v["schemaVersion"] = serde_json::json!(99);
        fs::write(&p, v.to_string()).unwrap();
        let err = load_ledger(&p).unwrap_err();
        assert!(err.to_string().contains("더 새로운 버전"));
    }

    #[test]
    fn bom_is_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("data.json");
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend(serde_json::to_vec(&sample()).unwrap());
        fs::write(&p, bytes).unwrap();
        assert!(load_ledger(&p).is_ok());
    }

    #[test]
    fn missing_file_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(load_ledger(&dir.path().join("x.json")), Err(AppError::NotFound { .. })));
    }

    #[test]
    fn open_or_create_creates_once() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("축의금_신부측.json");
        let created = open_or_create_ledger(&p, LedgerKind::Bride, "PC", base()).unwrap();
        assert_eq!(created.kind, LedgerKind::Bride);
        let opened = open_or_create_ledger(&p, LedgerKind::Groom, "PC", base()).unwrap();
        assert_eq!(opened.ledger_id, created.ledger_id);
        assert_eq!(opened.kind, LedgerKind::Bride);
    }

    #[test]
    fn save_to_missing_folder_fails_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("없는폴더").join("data.json");
        let err = save_ledger(&p, &sample()).unwrap_err();
        assert!(matches!(err, AppError::Io { .. }));
        assert!(err.to_string().contains("저장하지 못했습니다"));
    }
}
