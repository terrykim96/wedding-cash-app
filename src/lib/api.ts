// Rust Tauri 명령 래퍼 (component-methods.md 2절)

import { invoke } from "@tauri-apps/api/core";
import type {
  AppError,
  AppStateView,
  Entry,
  EntryInput,
  EntryQuery,
  HistoryItem,
  MergeReport,
  QueryResult,
  Side,
} from "./types";

/** invoke 실패 값을 AppError 형태로 정리한다 */
export function toAppError(e: unknown): AppError {
  if (e && typeof e === "object" && "code" in e && "message" in e) {
    return e as AppError;
  }
  return { code: "state", message: typeof e === "string" ? e : String(e) };
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw toAppError(e);
  }
}

export const getAppState = () => call<AppStateView>("get_app_state");
export const setupPcSide = (side: Side) => call<AppStateView>("setup_pc_side", { side });
export const setDefaultSide = (side: Side) => call<void>("set_default_side", { side });
export const markTutorialSeen = () => call<void>("mark_tutorial_seen");
export const addEntry = (input: EntryInput) => call<Entry>("add_entry", { input });
export const updateEntry = (id: string, input: EntryInput) => call<Entry>("update_entry", { id, input });
export const deleteEntry = (id: string) => call<void>("delete_entry", { id });
export const restoreEntry = (id: string) => call<Entry>("restore_entry", { id });
export const listRecent = (limit = 10) => call<Entry[]>("list_recent", { limit });
export const queryEntries = (query: EntryQuery) => call<QueryResult>("query_entries", { query });
export const listHistory = (entryId?: string) => call<HistoryItem[]>("list_history", { entryId: entryId ?? null });
export const mergeFiles = (pathA: string, pathB: string, outputPath: string) =>
  call<MergeReport>("merge_files", { pathA, pathB, outputPath });
export const openLedgerFile = (path: string) => call<AppStateView>("open_ledger_file", { path });
export const exportCsv = (path: string) => call<void>("export_csv", { path });
export const exportXlsx = (path: string) => call<void>("export_xlsx", { path });
