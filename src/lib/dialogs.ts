// 전역 다이얼로그 상태 (frontend-components.md F12)

import { writable } from "svelte/store";
import { formatManwon, formatWon, LARGE_AMOUNT_WON } from "./amount";
import type { Entry } from "./types";

export interface ConfirmOptions {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  /** 처음 포커스를 받을 버튼 */
  defaultFocus?: "confirm" | "cancel";
  danger?: boolean;
}

export interface ConfirmRequest extends ConfirmOptions {
  resolve: (ok: boolean) => void;
}

export const confirmState = writable<ConfirmRequest | null>(null);

/** 확인 창을 띄우고 사용자의 선택을 기다린다 */
export function askConfirm(options: ConfirmOptions): Promise<boolean> {
  return new Promise((resolve) => confirmState.set({ ...options, resolve }));
}

/** 큰 금액 확인 (BR-3). 기준 미만이면 바로 true */
export function confirmLargeAmount(won: number): Promise<boolean> {
  if (won < LARGE_AMOUNT_WON) return Promise.resolve(true);
  return askConfirm({
    title: "큰 금액 확인",
    message: `${formatWon(won)}(${formatManwon(won)})이 맞습니까?`,
    confirmText: "예, 맞습니다",
    cancelText: "아니요 (다시 입력)",
    defaultFocus: "cancel",
  });
}

/** 수정 대상 항목 */
export const editTarget = writable<Entry | null>(null);

/** 항목별 이력 보기 대상 */
export const historyTarget = writable<{ entryId: string; name: string } | null>(null);
