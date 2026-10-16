//! Tauri 명령 (component-methods.md 2절, services.md)
//!
//! 프론트엔드에서는 camelCase 인자 이름으로 호출한다 (예: `pathA`, `entryId`).

use std::path::PathBuf;
use std::sync::MutexGuard;

use tauri::State;

use crate::app_state::{AppState, Inner};
use crate::model::{
    AppError, AppResult, AppStateView, Entry, EntryInput, EntryQuery, HistoryItem, MergeReport, QueryResult, Side,
};
use crate::{env, export, ledger, merge};

fn lock<'a>(state: &'a State<'_, AppState>) -> AppResult<MutexGuard<'a, Inner>> {
    state.0.lock().map_err(|_| AppError::state("프로그램 상태를 읽을 수 없습니다. 다시 실행해 주세요"))
}

fn to_path(path: &str) -> AppResult<PathBuf> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::state("파일 경로가 비어 있습니다"));
    }
    Ok(PathBuf::from(trimmed))
}

#[tauri::command]
pub fn get_app_state(state: State<'_, AppState>) -> AppResult<AppStateView> {
    Ok(lock(&state)?.view())
}

#[tauri::command]
pub fn setup_pc_side(state: State<'_, AppState>, side: Side) -> AppResult<AppStateView> {
    let mut s = lock(&state)?;
    s.setup(side, env::now())?;
    Ok(s.view())
}

#[tauri::command]
pub fn set_default_side(state: State<'_, AppState>, side: Side) -> AppResult<()> {
    lock(&state)?.set_default_side(side)
}

#[tauri::command]
pub fn mark_tutorial_seen(state: State<'_, AppState>) -> AppResult<()> {
    lock(&state)?.mark_tutorial_seen()
}

#[tauri::command]
pub fn add_entry(state: State<'_, AppState>, input: EntryInput) -> AppResult<Entry> {
    let mut s = lock(&state)?;
    let pc = s.pc_name.clone();
    s.mutate(|l| ledger::add(l, &input, &pc, env::now()).map(|e| (e, true)))
}

#[tauri::command]
pub fn update_entry(state: State<'_, AppState>, id: String, input: EntryInput) -> AppResult<Entry> {
    let mut s = lock(&state)?;
    let pc = s.pc_name.clone();
    s.mutate(|l| ledger::update(l, &id, &input, &pc, env::now()).map(|o| (o.entry, o.changed)))
}

#[tauri::command]
pub fn delete_entry(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let mut s = lock(&state)?;
    let pc = s.pc_name.clone();
    s.mutate(|l| ledger::delete(l, &id, &pc, env::now()).map(|()| ((), true)))
}

#[tauri::command]
pub fn restore_entry(state: State<'_, AppState>, id: String) -> AppResult<Entry> {
    let mut s = lock(&state)?;
    let pc = s.pc_name.clone();
    s.mutate(|l| ledger::restore(l, &id, &pc, env::now()).map(|e| (e, true)))
}

#[tauri::command]
pub fn list_recent(state: State<'_, AppState>, limit: Option<usize>) -> AppResult<Vec<Entry>> {
    let s = lock(&state)?;
    let limit = limit.unwrap_or(ledger::RECENT_LIMIT).clamp(1, 100);
    Ok(ledger::recent(s.ledger()?, limit))
}

#[tauri::command]
pub fn query_entries(state: State<'_, AppState>, query: EntryQuery) -> AppResult<QueryResult> {
    let s = lock(&state)?;
    Ok(ledger::query(s.ledger()?, &query))
}

#[tauri::command]
pub fn list_history(state: State<'_, AppState>, entry_id: Option<String>) -> AppResult<Vec<HistoryItem>> {
    let s = lock(&state)?;
    Ok(ledger::history(s.ledger()?, entry_id.as_deref()))
}

#[tauri::command]
pub fn merge_files(
    state: State<'_, AppState>,
    path_a: String,
    path_b: String,
    output_path: String,
) -> AppResult<MergeReport> {
    let s = lock(&state)?;
    let (a, b, out) = (to_path(&path_a)?, to_path(&path_b)?, to_path(&output_path)?);
    // 열려 있는 파일을 덮어쓰면 화면 상태와 파일 내용이 어긋나므로 거부
    if s.is_active_path(&out) {
        return Err(AppError::state("지금 열려 있는 파일에는 저장할 수 없습니다. 다른 이름을 지정해 주세요"));
    }
    merge::merge_files(&a, &b, &out, &s.pc_name, env::now())
}

#[tauri::command]
pub fn open_ledger_file(state: State<'_, AppState>, path: String) -> AppResult<AppStateView> {
    let mut s = lock(&state)?;
    s.open_file(&to_path(&path)?)?;
    Ok(s.view())
}

#[tauri::command]
pub fn export_csv(state: State<'_, AppState>, path: String) -> AppResult<()> {
    let s = lock(&state)?;
    export::write_csv(s.ledger()?, &to_path(&path)?)
}

#[tauri::command]
pub fn export_xlsx(state: State<'_, AppState>, path: String) -> AppResult<()> {
    let s = lock(&state)?;
    export::write_xlsx(s.ledger()?, &to_path(&path)?)
}
