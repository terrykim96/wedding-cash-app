//! 저장 위치와 파일 이름 (business-rules.md BR-12.1 ~ 12.3)

use std::path::{Path, PathBuf};

use crate::model::{AppError, AppResult, Side};

pub const GROOM_FILE: &str = "축의금_신랑측.json";
pub const BRIDE_FILE: &str = "축의금_신부측.json";
pub const MERGED_DEFAULT_FILE: &str = "축의금_통합.json";
pub const SETTINGS_FILE: &str = "settings.json";
pub const MAC_FALLBACK_FOLDER: &str = "축의금";

pub const MAC_FALLBACK_NOTICE: &str = "앱 폴더에 저장할 수 없어 문서/축의금 폴더에 저장합니다";

/// PC 구분별 기본 데이터 파일 이름
pub fn ledger_file_name(side: Side) -> &'static str {
    match side {
        Side::Groom => GROOM_FILE,
        Side::Bride => BRIDE_FILE,
    }
}

/// 실행 파일 위치로부터 데이터 폴더 후보를 계산한다.
///
/// - macOS: `.../이름.app/Contents/MacOS/실행파일` → `.app`이 있는 폴더
/// - 그 외: 실행 파일이 있는 폴더
pub fn app_folder_for_exe(exe: &Path, is_mac: bool) -> Option<PathBuf> {
    if is_mac {
        if let Some(bundle) = exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app")) {
            return bundle.parent().map(Path::to_path_buf);
        }
    }
    exe.parent().map(Path::to_path_buf)
}

/// macOS App Translocation(임시 읽기 전용 위치)에서 실행 중인지
pub fn is_translocated(path: &Path) -> bool {
    path.to_string_lossy().contains("/AppTranslocation/")
}

/// 폴더에 파일을 만들 수 있는지 실제로 확인한다
pub fn is_writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".write-test-{}", uuid::Uuid::new_v4()));
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// 데이터 폴더 결정 로직 (테스트 가능한 형태)
pub fn choose_data_dir(
    exe: &Path,
    is_mac: bool,
    home: Option<&Path>,
    writable: impl Fn(&Path) -> bool,
) -> AppResult<(PathBuf, Option<String>)> {
    let app_dir = app_folder_for_exe(exe, is_mac)
        .ok_or_else(|| AppError::io("실행 파일 위치를 확인할 수 없습니다"))?;

    if !is_mac {
        return Ok((app_dir, None));
    }
    if !is_translocated(&app_dir) && writable(&app_dir) {
        return Ok((app_dir, None));
    }
    let home = home.ok_or_else(|| AppError::io("사용자 폴더를 찾을 수 없습니다"))?;
    let fallback = home.join("Documents").join(MAC_FALLBACK_FOLDER);
    Ok((fallback, Some(MAC_FALLBACK_NOTICE.to_string())))
}

/// 현재 실행 환경의 데이터 폴더와 안내 문구. 폴더가 없으면 만든다.
pub fn resolve_data_dir() -> AppResult<(PathBuf, Option<String>)> {
    let exe = std::env::current_exe().map_err(|e| AppError::io(format!("실행 파일 위치를 확인할 수 없습니다: {e}")))?;
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let (dir, notice) = choose_data_dir(&exe, cfg!(target_os = "macos"), home.as_deref(), is_writable)?;
    std::fs::create_dir_all(&dir).map_err(|e| AppError::io(format!("저장 폴더를 만들지 못했습니다: {e}")))?;
    Ok((dir, notice))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn windows_uses_exe_folder() {
        let (dir, notice) = choose_data_dir(Path::new(r"D:\USB\축의금\wedding-gift-app.exe"), false, None, |_| false).unwrap();
        assert_eq!(dir, PathBuf::from(r"D:\USB\축의금"));
        assert!(notice.is_none());
    }

    #[test]
    fn mac_uses_folder_containing_app() {
        let exe = Path::new("/Volumes/USB/축의금/축의금 관리.app/Contents/MacOS/wedding-gift-app");
        let (dir, notice) = choose_data_dir(exe, true, Some(Path::new("/Users/me")), |_| true).unwrap();
        assert_eq!(dir, PathBuf::from("/Volumes/USB/축의금"));
        assert!(notice.is_none());
    }

    #[test]
    fn mac_falls_back_when_translocated() {
        let exe = Path::new("/private/var/folders/x/AppTranslocation/ABC/d/축의금 관리.app/Contents/MacOS/wedding-gift-app");
        let (dir, notice) = choose_data_dir(exe, true, Some(Path::new("/Users/me")), |_| true).unwrap();
        assert_eq!(dir, PathBuf::from("/Users/me/Documents/축의금"));
        assert_eq!(notice.as_deref(), Some(MAC_FALLBACK_NOTICE));
    }

    #[test]
    fn mac_falls_back_when_not_writable() {
        let exe = Path::new("/Applications/축의금 관리.app/Contents/MacOS/wedding-gift-app");
        let (dir, notice) = choose_data_dir(exe, true, Some(Path::new("/Users/me")), |_| false).unwrap();
        assert_eq!(dir, PathBuf::from("/Users/me/Documents/축의금"));
        assert!(notice.is_some());
    }

    #[test]
    fn writable_probe_leaves_no_file() {
        let dir = tempfile::tempdir().unwrap();
        assert!(is_writable(dir.path()));
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        assert!(!is_writable(&dir.path().join("없는폴더")));
    }

    #[test]
    fn file_names() {
        assert_eq!(ledger_file_name(Side::Groom), "축의금_신랑측.json");
        assert_eq!(ledger_file_name(Side::Bride), "축의금_신부측.json");
    }
}
