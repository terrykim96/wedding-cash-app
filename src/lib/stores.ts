// 화면 전역 상태 (frontend-components.md 2절)

import { writable } from "svelte/store";
import type { AppError, AppStateView } from "./types";

export const appState = writable<AppStateView | null>(null);

/** 지금 열려 있는 ? 도움말 id (한 번에 하나만 열린다) */
export const openHelp = writable<string | null>(null);

/** 사용 설명(튜토리얼) 대화상자 표시 여부 */
export const tutorialOpen = writable(false);

/** 데이터가 바뀔 때마다 증가 → 목록 화면이 다시 조회한다 */
export const dataVersion = writable(0);

export function notifyDataChanged(): void {
  dataVersion.update((v) => v + 1);
}

export interface ToastMessage {
  id: number;
  kind: "ok" | "error";
  text: string;
}

export const toast = writable<ToastMessage | null>(null);

let toastSeq = 0;
let toastTimer: ReturnType<typeof setTimeout> | undefined;

export function showToast(kind: "ok" | "error", text: string): void {
  toastSeq += 1;
  toast.set({ id: toastSeq, kind, text });
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.set(null), kind === "error" ? 6000 : 2500);
}

export function showError(e: unknown, prefix?: string): void {
  const err = e as Partial<AppError>;
  const message = typeof err?.message === "string" ? err.message : String(e);
  showToast("error", prefix ? `${prefix}: ${message}` : message);
}
