//! 설정 파일 (settings.json) 읽기/쓰기 (business-rules.md BR-12)

use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{AppResult, Settings};
use crate::paths::SETTINGS_FILE;
use crate::storage::write_atomic;

pub fn settings_path(dir: &Path) -> PathBuf {
    dir.join(SETTINGS_FILE)
}

/// 설정 읽기. 없거나 깨졌으면 기본값(최초 설정 필요)을 돌려준다 (BR-12.6)
pub fn load(dir: &Path) -> Settings {
    let Ok(text) = fs::read_to_string(settings_path(dir)) else {
        return Settings::default();
    };
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    serde_json::from_str(text).unwrap_or_default()
}

/// 설정 저장 (원자적 쓰기)
pub fn save(dir: &Path, settings: &Settings) -> AppResult<()> {
    let json = serde_json::to_vec_pretty(settings).expect("설정 직렬화는 실패하지 않음");
    write_atomic(&settings_path(dir), &json)
}

/// 설정의 activeFile 값을 실제 경로로 바꾼다. 파일 이름만 있으면 데이터 폴더 기준.
pub fn resolve_active_path(dir: &Path, active_file: &str) -> PathBuf {
    let p = Path::new(active_file);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        dir.join(p)
    }
}

/// 경로를 설정 저장용 값으로 바꾼다. 데이터 폴더 안의 파일이면 파일 이름만 저장한다.
pub fn to_active_file(dir: &Path, path: &Path) -> String {
    let same_dir = match (path.parent(), dir.canonicalize()) {
        (Some(parent), Ok(d)) => parent.canonicalize().map(|p| p == d).unwrap_or(false),
        _ => false,
    };
    match (same_dir, path.file_name()) {
        (true, Some(name)) => name.to_string_lossy().into_owned(),
        _ => path.to_string_lossy().into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Side;

    #[test]
    fn missing_or_broken_settings_need_setup() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()).pc_side, None);
        fs::write(settings_path(dir.path()), "깨진 설정").unwrap();
        assert_eq!(load(dir.path()), Settings::default());
    }

    #[test]
    fn old_settings_without_tutorial_field_still_load() {
        // 사용 설명 기능이 생기기 전에 만들어진 settings.json
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            settings_path(dir.path()),
            r#"{"schemaVersion":1,"pcSide":"groom","defaultSide":"bride","activeFile":"축의금_신랑측.json"}"#,
        )
        .unwrap();
        let s = load(dir.path());
        assert_eq!(s.pc_side, Some(Side::Groom));
        assert_eq!(s.default_side, Side::Bride);
        assert_eq!(s.active_file.as_deref(), Some("축의금_신랑측.json"));
        assert!(!s.tutorial_seen);
    }

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let s = Settings {
            pc_side: Some(Side::Bride),
            default_side: Side::Groom,
            active_file: Some("축의금_신부측.json".into()),
            ..Settings::default()
        };
        save(dir.path(), &s).unwrap();
        assert_eq!(load(dir.path()), s);
        let text = fs::read_to_string(settings_path(dir.path())).unwrap();
        assert!(text.contains("\"pcSide\": \"bride\""));
    }

    #[test]
    fn active_file_is_relative_inside_data_dir() {
        let dir = tempfile::tempdir().unwrap();
        let inside = dir.path().join("축의금_통합.json");
        fs::write(&inside, "{}").unwrap();
        assert_eq!(to_active_file(dir.path(), &inside), "축의금_통합.json");
        assert_eq!(resolve_active_path(dir.path(), "축의금_통합.json"), inside);

        let other = tempfile::tempdir().unwrap();
        let outside = other.path().join("usb.json");
        fs::write(&outside, "{}").unwrap();
        let stored = to_active_file(dir.path(), &outside);
        assert!(Path::new(&stored).is_absolute());
        assert_eq!(resolve_active_path(dir.path(), &stored), outside);
    }
}
