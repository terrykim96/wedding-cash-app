// 여러 화면에서 함께 쓰는 동작 (삭제, 복구, 파일 열기)

import { open, save } from "@tauri-apps/plugin-dialog";
import { get } from "svelte/store";
import { deleteEntry, openLedgerFile, restoreEntry } from "./api";
import { formatWon } from "./amount";
import { askConfirm } from "./dialogs";
import { appState, notifyDataChanged, showError, showToast } from "./stores";
import type { Entry, HistoryItem } from "./types";

const JSON_FILTER = [{ name: "축의금 데이터 파일", extensions: ["json"] }];

/** 경로 구분자에 맞춰 폴더와 파일 이름을 잇는다 */
export function joinPath(dir: string, name: string): string {
  if (!dir) return name;
  const sep = dir.includes("\\") ? "\\" : "/";
  return dir.endsWith(sep) ? `${dir}${name}` : `${dir}${sep}${name}`;
}

function dataDir(): string {
  return get(appState)?.dataDir ?? "";
}

/** 데이터 파일(.json) 열기 대화상자. 취소하면 null */
export async function pickLedgerFile(title: string): Promise<string | null> {
  const picked = await open({ title, multiple: false, directory: false, filters: JSON_FILTER, defaultPath: dataDir() || undefined });
  return typeof picked === "string" ? picked : null;
}

/** 저장 대화상자. 취소하면 null */
export async function pickSavePath(title: string, defaultName: string, filterName: string, ext: string): Promise<string | null> {
  const picked = await save({
    title,
    defaultPath: joinPath(dataDir(), defaultName),
    filters: [{ name: filterName, extensions: [ext] }],
  });
  return picked ?? null;
}

/** 다른 데이터 파일 열기 (S8) */
export async function openOtherFile(): Promise<void> {
  try {
    const path = await pickLedgerFile("열 데이터 파일 선택");
    if (!path) return;
    await openFileByPath(path);
  } catch (e) {
    showError(e, "파일을 열지 못했습니다");
  }
}

export async function openFileByPath(path: string): Promise<void> {
  const view = await openLedgerFile(path);
  appState.set(view);
  notifyDataChanged();
  showToast("ok", "파일을 열었습니다");
}

/** 삭제 확인 후 삭제 (BR-6.1) */
export async function confirmAndDelete(entry: Entry): Promise<void> {
  const ok = await askConfirm({
    title: "항목 삭제",
    message: `${entry.name} / ${formatWon(entry.amountWon)} 항목을 삭제할까요? 삭제한 항목은 변경 이력 화면에서 복구할 수 있습니다.`,
    confirmText: "삭제",
    cancelText: "취소",
    defaultFocus: "cancel",
    danger: true,
  });
  if (!ok) return;
  try {
    await deleteEntry(entry.id);
    notifyDataChanged();
    showToast("ok", `${entry.name} 항목을 삭제했습니다`);
  } catch (e) {
    showError(e, "삭제하지 못했습니다");
  }
}

/** 복구 확인 후 복구 (BR-6.4) */
export async function confirmAndRestore(item: HistoryItem): Promise<void> {
  const ok = await askConfirm({
    title: "항목 복구",
    message: `${item.entryName} / ${formatWon(item.before.amountWon)} 항목을 복구할까요?`,
    confirmText: "복구",
    cancelText: "취소",
    defaultFocus: "confirm",
  });
  if (!ok) return;
  try {
    await restoreEntry(item.entryId);
    notifyDataChanged();
    showToast("ok", `${item.entryName} 항목을 복구했습니다`);
  } catch (e) {
    showError(e, "복구하지 못했습니다");
  }
}
