//! 축의금 관리 프로그램 - Tauri 백엔드

pub mod app_state;
pub mod commands;
pub mod env;
pub mod export;
pub mod ledger;
pub mod merge;
pub mod model;
pub mod paths;
pub mod settings;
pub mod storage;
pub mod validation;

use std::sync::Mutex;

use app_state::{AppState, Inner};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pc_name = env::pc_name();
    let inner = match paths::resolve_data_dir() {
        Ok((dir, notice)) => Inner::start(dir, notice, pc_name, env::now()),
        Err(e) => Inner::start_failed(e.to_string(), pc_name),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(Mutex::new(inner)))
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::setup_pc_side,
            commands::set_default_side,
            commands::mark_tutorial_seen,
            commands::add_entry,
            commands::update_entry,
            commands::delete_entry,
            commands::restore_entry,
            commands::list_recent,
            commands::query_entries,
            commands::list_history,
            commands::merge_files,
            commands::open_ledger_file,
            commands::export_csv,
            commands::export_xlsx,
        ])
        .run(tauri::generate_context!())
        .expect("축의금 관리 프로그램을 실행하지 못했습니다");
}
