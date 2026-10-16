//! 실행 중 상태와 상태 변경 흐름 (services.md S1, S2, S8 / business-rules.md BR-12, BR-13)

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::model::{AppError, AppResult, AppStateView, LedgerFile, Settings, Side, Timestamp};
use crate::paths::ledger_file_name;
use crate::{settings, storage};

/// Tauri가 관리하는 전역 상태
pub struct AppState(pub Mutex<Inner>);

/// 실제 상태 값
#[derive(Debug)]
pub struct Inner {
    pub data_dir: PathBuf,
    pub pc_name: String,
    pub settings: Settings,
    pub ledger: Option<LedgerFile>,
    pub active_path: Option<PathBuf>,
    pub load_error: Option<String>,
    pub notices: Vec<String>,
}

impl Inner {
    /// 앱 시작 (S1)
    pub fn start(data_dir: PathBuf, storage_notice: Option<String>, pc_name: String, now: Timestamp) -> Inner {
        let settings = settings::load(&data_dir);
        let mut inner = Inner {
            data_dir,
            pc_name,
            settings,
            ledger: None,
            active_path: None,
            load_error: None,
            notices: storage_notice.into_iter().collect(),
        };
        if let Some(side) = inner.settings.pc_side {
            inner.open_initial(side, now);
        }
        inner
    }

    /// 저장 폴더를 결정하지 못했을 때의 상태
    pub fn start_failed(message: String, pc_name: String) -> Inner {
        Inner {
            data_dir: PathBuf::new(),
            pc_name,
            settings: Settings::default(),
            ledger: None,
            active_path: None,
            load_error: Some(message),
            notices: Vec::new(),
        }
    }

    fn default_path(&self, side: Side) -> PathBuf {
        self.data_dir.join(ledger_file_name(side))
    }

    /// 설정에 기록된 작업 파일(없으면 기본 파일)을 연다 (BR-13.6)
    fn open_initial(&mut self, side: Side, now: Timestamp) {
        let default = self.default_path(side);
        let mut target = self
            .settings
            .active_file
            .as_deref()
            .map(|f| settings::resolve_active_path(&self.data_dir, f))
            .unwrap_or_else(|| default.clone());

        if target != default && !target.exists() {
            self.notices.push(format!(
                "이전에 열었던 파일({})을 찾을 수 없어 기본 파일을 엽니다",
                target.display()
            ));
            target = default.clone();
            self.settings.active_file = Some(ledger_file_name(side).to_string());
            let _ = settings::save(&self.data_dir, &self.settings);
        }

        let result = if target == default {
            storage::open_or_create_ledger(&default, side.into(), &self.pc_name, now)
        } else {
            storage::load_ledger(&target)
        };
        self.active_path = Some(target);
        match result {
            Ok(l) => {
                self.ledger = Some(l);
                self.load_error = None;
            }
            Err(e) => {
                self.ledger = None;
                self.load_error = Some(e.to_string());
            }
        }
    }

    /// 화면에 전달할 상태
    pub fn view(&self) -> AppStateView {
        AppStateView {
            needs_setup: self.settings.pc_side.is_none() && self.load_error.is_none(),
            pc_side: self.settings.pc_side,
            pc_name: self.pc_name.clone(),
            default_side: self.settings.default_side,
            data_dir: self.data_dir.to_string_lossy().into_owned(),
            active_file_path: self.active_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            active_file_kind: self.ledger.as_ref().map(|l| l.kind),
            storage_notice: if self.notices.is_empty() { None } else { Some(self.notices.join("\n")) },
            load_error: self.load_error.clone(),
            tutorial_seen: self.settings.tutorial_seen,
        }
    }

    /// 사용 설명을 봤다고 기록한다. 이미 기록되어 있으면 아무것도 하지 않는다.
    pub fn mark_tutorial_seen(&mut self) -> AppResult<()> {
        if self.settings.tutorial_seen {
            return Ok(());
        }
        if self.data_dir.as_os_str().is_empty() {
            return Err(AppError::state("저장 폴더를 확인할 수 없습니다"));
        }
        let mut next = self.settings.clone();
        next.tutorial_seen = true;
        settings::save(&self.data_dir, &next)?;
        self.settings = next;
        Ok(())
    }

    /// 최초 설정 (S2, BR-12.4)
    pub fn setup(&mut self, side: Side, now: Timestamp) -> AppResult<()> {
        if self.settings.pc_side.is_some() {
            return Err(AppError::state("이미 PC 구분이 설정되어 있습니다"));
        }
        let mut next = self.settings.clone();
        next.pc_side = Some(side);
        next.default_side = side;
        next.active_file = Some(ledger_file_name(side).to_string());
        settings::save(&self.data_dir, &next)?;
        self.settings = next;
        self.open_initial(side, now);
        Ok(())
    }

    /// 기본 구분 저장 (BR-12.7)
    pub fn set_default_side(&mut self, side: Side) -> AppResult<()> {
        if self.settings.default_side == side {
            return Ok(());
        }
        let mut next = self.settings.clone();
        next.default_side = side;
        settings::save(&self.data_dir, &next)?;
        self.settings = next;
        Ok(())
    }

    /// 읽기 전용 장부 접근
    pub fn ledger(&self) -> AppResult<&LedgerFile> {
        if let Some(e) = &self.load_error {
            return Err(AppError::state(format!("데이터 파일을 열 수 없습니다: {e}")));
        }
        self.ledger.as_ref().ok_or_else(|| AppError::state("PC 구분을 먼저 설정해 주세요"))
    }

    /// 변경 공통 처리: 사본에 적용 → 저장 성공 시에만 교체 (BR-13.2)
    ///
    /// `f`는 (결과, 실제 변경 여부)를 돌려준다. 변경이 없으면 저장하지 않는다.
    pub fn mutate<T>(&mut self, f: impl FnOnce(&mut LedgerFile) -> AppResult<(T, bool)>) -> AppResult<T> {
        let mut draft = self.ledger()?.clone();
        let (result, changed) = f(&mut draft)?;
        if changed {
            let path = self.active_path.as_ref().ok_or_else(|| AppError::state("작업 파일이 없습니다"))?;
            storage::save_ledger(path, &draft)?;
            self.ledger = Some(draft);
        }
        Ok(result)
    }

    /// 다른 파일 열기: 읽기에 성공했을 때만 교체 (S8, BR-13.7)
    pub fn open_file(&mut self, path: &Path) -> AppResult<()> {
        let ledger = storage::load_ledger(path)?;
        self.ledger = Some(ledger);
        self.active_path = Some(path.to_path_buf());
        self.load_error = None;
        let mut next = self.settings.clone();
        next.active_file = Some(settings::to_active_file(&self.data_dir, path));
        if settings::save(&self.data_dir, &next).is_ok() {
            self.settings = next;
        } else {
            self.notices.push("설정을 저장하지 못해 다음 실행 때는 이전 파일이 열립니다".to_string());
        }
        Ok(())
    }

    /// 경로가 현재 작업 파일인지
    pub fn is_active_path(&self, path: &Path) -> bool {
        let Some(active) = &self.active_path else { return false };
        match (active.canonicalize(), path.canonicalize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => active == path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{self, tests::{base, input}};
    use crate::model::LedgerKind;
    use crate::paths::{BRIDE_FILE, GROOM_FILE};

    fn start(dir: &Path) -> Inner {
        Inner::start(dir.to_path_buf(), None, "PC-1".into(), base())
    }

    fn add(inner: &mut Inner, name: &str) {
        let pc = inner.pc_name.clone();
        inner
            .mutate(|l| ledger::add(l, &input(name, 50_000, Side::Groom), &pc, base()).map(|e| (e, true)))
            .unwrap();
    }

    #[test]
    fn first_run_needs_setup() {
        let dir = tempfile::tempdir().unwrap();
        let inner = start(dir.path());
        let v = inner.view();
        assert!(v.needs_setup);
        assert!(v.active_file_path.is_none());
        assert!(matches!(inner.ledger(), Err(AppError::State { .. })));
    }

    #[test]
    fn setup_creates_file_and_data_persists_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let mut inner = start(dir.path());
        inner.setup(Side::Bride, base()).unwrap();
        assert!(dir.path().join(BRIDE_FILE).exists());
        let v = inner.view();
        assert!(!v.needs_setup);
        assert_eq!(v.pc_side, Some(Side::Bride));
        assert_eq!(v.default_side, Side::Bride);
        assert_eq!(v.active_file_kind, Some(LedgerKind::Bride));
        assert!(inner.setup(Side::Groom, base()).is_err());

        add(&mut inner, "김민수");
        inner.set_default_side(Side::Groom).unwrap();

        let restarted = start(dir.path());
        assert_eq!(restarted.ledger().unwrap().entries.len(), 1);
        assert_eq!(restarted.view().default_side, Side::Groom);
    }

    #[test]
    fn tutorial_flag_persists_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let mut inner = start(dir.path());
        inner.setup(Side::Groom, base()).unwrap();
        assert!(!inner.view().tutorial_seen);

        inner.mark_tutorial_seen().unwrap();
        assert!(inner.view().tutorial_seen);
        inner.mark_tutorial_seen().unwrap(); // 두 번째 호출도 오류 없음

        // 다시 켜도 유지, 다른 설정은 그대로
        let restarted = start(dir.path());
        assert!(restarted.view().tutorial_seen);
        assert_eq!(restarted.view().pc_side, Some(Side::Groom));
        assert_eq!(restarted.view().default_side, Side::Groom);
    }

    #[test]
    fn tutorial_flag_keeps_state_when_save_fails() {
        let dir = tempfile::tempdir().unwrap();
        let mut inner = start(dir.path());
        inner.setup(Side::Bride, base()).unwrap();
        inner.data_dir = dir.path().join("없는폴더");
        assert!(matches!(inner.mark_tutorial_seen(), Err(AppError::Io { .. })));
        assert!(!inner.view().tutorial_seen);

        let mut failed = Inner::start_failed("폴더 오류".into(), "PC".into());
        assert!(matches!(failed.mark_tutorial_seen(), Err(AppError::State { .. })));
    }

    #[test]
    fn save_failure_keeps_previous_state() {
        let dir = tempfile::tempdir().unwrap();
        let mut inner = start(dir.path());
        inner.setup(Side::Groom, base()).unwrap();
        add(&mut inner, "김민수");
        inner.active_path = Some(dir.path().join("없는폴더").join("x.json"));
        let pc = inner.pc_name.clone();
        let r = inner.mutate(|l| ledger::add(l, &input("박지훈", 50_000, Side::Groom), &pc, base()).map(|e| (e, true)));
        assert!(matches!(r, Err(AppError::Io { .. })));
        assert_eq!(inner.ledger().unwrap().entries.len(), 1);
    }

    #[test]
    fn unchanged_update_does_not_save() {
        let dir = tempfile::tempdir().unwrap();
        let mut inner = start(dir.path());
        inner.setup(Side::Groom, base()).unwrap();
        add(&mut inner, "김민수");
        let path = dir.path().join(GROOM_FILE);
        let before = std::fs::metadata(&path).unwrap().modified().unwrap();
        let id = inner.ledger().unwrap().entries[0].id.clone();
        let pc = inner.pc_name.clone();
        inner
            .mutate(|l| ledger::update(l, &id, &input("김민수", 50_000, Side::Groom), &pc, base()).map(|o| (o.entry, o.changed)))
            .unwrap();
        assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), before);
        assert!(inner.ledger().unwrap().history.is_empty());
    }

    #[test]
    fn broken_data_file_is_reported_and_kept() {
        let dir = tempfile::tempdir().unwrap();
        let mut inner = start(dir.path());
        inner.setup(Side::Groom, base()).unwrap();
        std::fs::write(dir.path().join(GROOM_FILE), "깨짐").unwrap();

        let mut restarted = start(dir.path());
        assert!(restarted.view().load_error.is_some());
        assert!(!restarted.view().needs_setup);
        assert!(restarted.ledger().is_err());
        let pc = restarted.pc_name.clone();
        assert!(restarted
            .mutate(|l| ledger::add(l, &input("김", 50_000, Side::Groom), &pc, base()).map(|e| (e, true)))
            .is_err());
        assert_eq!(std::fs::read_to_string(dir.path().join(GROOM_FILE)).unwrap(), "깨짐");
    }

    #[test]
    fn open_other_file_and_fallback_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let mut inner = start(dir.path());
        inner.setup(Side::Groom, base()).unwrap();

        let merged = dir.path().join("축의금_통합.json");
        let mut l = LedgerFile::new(LedgerKind::Merged, "PC", base());
        ledger::add(&mut l, &input("통합하객", 50_000, Side::Bride), "PC", base()).unwrap();
        storage::save_ledger(&merged, &l).unwrap();

        assert!(inner.open_file(&dir.path().join("없음.json")).is_err());
        assert_eq!(inner.view().active_file_kind, Some(LedgerKind::Groom));

        inner.open_file(&merged).unwrap();
        assert_eq!(inner.view().active_file_kind, Some(LedgerKind::Merged));
        assert_eq!(inner.settings.active_file.as_deref(), Some("축의금_통합.json"));
        assert!(inner.is_active_path(&merged));

        let restarted = start(dir.path());
        assert_eq!(restarted.view().active_file_kind, Some(LedgerKind::Merged));

        std::fs::remove_file(&merged).unwrap();
        let restarted = start(dir.path());
        assert_eq!(restarted.view().active_file_kind, Some(LedgerKind::Groom));
        assert!(restarted.view().storage_notice.unwrap().contains("찾을 수 없어"));
    }
}
