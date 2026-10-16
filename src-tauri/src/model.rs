//! 도메인 타입 정의 (domain-entities.md)
//!
//! JSON 필드 이름은 camelCase로 직렬화한다.

use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// 현재 데이터 파일 스키마 버전
pub const SCHEMA_VERSION: u32 = 1;

/// 시각 타입: 로컬 오프셋을 포함한 시각 (예: 2026-11-21T11:32:10+09:00)
pub type Timestamp = DateTime<FixedOffset>;

/// 구분: 신랑측 / 신부측
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Groom,
    Bride,
}

impl Side {
    /// 화면/내보내기용 한국어 이름
    pub fn label(self) -> &'static str {
        match self {
            Side::Groom => "신랑측",
            Side::Bride => "신부측",
        }
    }
}

/// 사용자 입력 값 (추가/수정 요청, 이력 스냅샷)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryInput {
    pub name: String,
    pub amount_won: i64,
    pub side: Side,
    pub tickets: i64,
    pub memo: String,
}

/// 접수 항목
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub amount_won: i64,
    pub side: Side,
    pub tickets: i64,
    pub memo: String,
    pub received_at: Timestamp,
    pub updated_at: Timestamp,
    pub deleted: bool,
    pub source_pc: String,
}

impl Entry {
    /// 사용자 입력 필드만 추출
    pub fn to_input(&self) -> EntryInput {
        EntryInput {
            name: self.name.clone(),
            amount_won: self.amount_won,
            side: self.side,
            tickets: self.tickets,
            memo: self.memo.clone(),
        }
    }

    /// 사용자 입력 필드 적용
    pub fn apply_input(&mut self, input: &EntryInput) {
        self.name = input.name.clone();
        self.amount_won = input.amount_won;
        self.side = input.side;
        self.tickets = input.tickets;
        self.memo = input.memo.clone();
    }
}

/// 변경 이력 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HistoryKind {
    Update,
    Delete,
    Restore,
}

/// 변경 이력
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecord {
    pub id: String,
    pub entry_id: String,
    pub kind: HistoryKind,
    pub at: Timestamp,
    pub before: EntryInput,
    pub after: Option<EntryInput>,
    pub pc_name: String,
}

/// 이력 조회 결과 (저장하지 않음)
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    #[serde(flatten)]
    pub record: HistoryRecord,
    pub entry_name: String,
    pub can_restore: bool,
}

/// 데이터 파일 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LedgerKind {
    Groom,
    Bride,
    Merged,
}

impl From<Side> for LedgerKind {
    fn from(side: Side) -> Self {
        match side {
            Side::Groom => LedgerKind::Groom,
            Side::Bride => LedgerKind::Bride,
        }
    }
}

/// 데이터 파일 (장부)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerFile {
    pub schema_version: u32,
    pub ledger_id: String,
    pub kind: LedgerKind,
    pub created_at: Timestamp,
    pub created_by: String,
    #[serde(default)]
    pub merged_from: Vec<String>,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub history: Vec<HistoryRecord>,
}

impl LedgerFile {
    /// 빈 장부 생성
    pub fn new(kind: LedgerKind, created_by: &str, now: Timestamp) -> Self {
        LedgerFile {
            schema_version: SCHEMA_VERSION,
            ledger_id: uuid::Uuid::new_v4().to_string(),
            kind,
            created_at: now,
            created_by: created_by.to_string(),
            merged_from: Vec::new(),
            entries: Vec::new(),
            history: Vec::new(),
        }
    }

    /// 삭제되지 않은 항목
    pub fn active_entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| !e.deleted)
    }
}

/// 설정 파일 (settings.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub schema_version: u32,
    pub pc_side: Option<Side>,
    pub default_side: Side,
    pub active_file: Option<String>,
    /// 사용 설명(튜토리얼)을 끝까지 봤거나 "다음부터 보지 않기"를 고른 경우 true.
    /// 이 필드가 없는 예전 settings.json 도 읽을 수 있도록 기본값을 둔다.
    #[serde(default)]
    pub tutorial_seen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            schema_version: SCHEMA_VERSION,
            pc_side: None,
            default_side: Side::Groom,
            active_file: None,
            tutorial_seen: false,
        }
    }
}

/// 구분별 합계
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SideSummary {
    pub amount_won: i64,
    pub count: i64,
    pub tickets: i64,
}

/// 전체/신랑측/신부측 합계
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub total: SideSummary,
    pub groom: SideSummary,
    pub bride: SideSummary,
}

/// 정렬 기준
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortBy {
    Name,
    Amount,
    ReceivedAt,
}

/// 정렬 방향
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortDir {
    Asc,
    Desc,
}

/// 정산 화면 조회 조건
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub side: Option<Side>,
    pub sort_by: SortBy,
    pub sort_dir: SortDir,
}

/// 정산 화면 조회 결과
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub entries: Vec<Entry>,
    pub filtered_summary: Summary,
    pub total_summary: Summary,
}

/// 병합 결과 리포트
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeReport {
    pub output_path: String,
    pub from_a: i64,
    pub from_b: i64,
    pub merged: i64,
    pub duplicates_resolved: i64,
}

/// 화면에 전달하는 앱 상태
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStateView {
    pub needs_setup: bool,
    pub pc_side: Option<Side>,
    pub pc_name: String,
    pub default_side: Side,
    pub data_dir: String,
    pub active_file_path: Option<String>,
    pub active_file_kind: Option<LedgerKind>,
    pub storage_notice: Option<String>,
    pub load_error: Option<String>,
    pub tutorial_seen: bool,
}

/// 입력 검증 오류: 필드 이름 → 메시지
pub type FieldErrors = BTreeMap<String, String>;

/// 화면에 전달하는 오류
#[derive(Debug, Clone, PartialEq, Eq, Serialize, thiserror::Error)]
#[serde(rename_all = "camelCase", tag = "code")]
pub enum AppError {
    #[error("입력값을 확인해 주세요")]
    Validation {
        message: String,
        #[serde(rename = "fieldErrors")]
        field_errors: FieldErrors,
    },
    #[error("{message}")]
    Io { message: String },
    #[error("{message}")]
    Parse { message: String },
    #[error("{message}")]
    NotFound { message: String },
    #[error("{message}")]
    State { message: String },
}

impl AppError {
    pub fn validation(field_errors: FieldErrors) -> Self {
        AppError::Validation {
            message: "입력값을 확인해 주세요".to_string(),
            field_errors,
        }
    }
    pub fn io(message: impl Into<String>) -> Self {
        AppError::Io { message: message.into() }
    }
    pub fn parse(message: impl Into<String>) -> Self {
        AppError::Parse { message: message.into() }
    }
    pub fn not_found(message: impl Into<String>) -> Self {
        AppError::NotFound { message: message.into() }
    }
    pub fn state(message: impl Into<String>) -> Self {
        AppError::State { message: message.into() }
    }
}

pub type AppResult<T> = Result<T, AppError>;
