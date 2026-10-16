// 입력 폼 값 → EntryInput 변환과 화면 단계 검증 (business-rules.md BR-1, BR-2)
// Rust validation.rs 와 같은 규칙/문구. 최종 판단은 Rust.

import { amountError, parseManwon } from "./amount";
import type { EntryInput, FieldErrors, FieldName, Side } from "./types";

export const NAME_MAX = 30;
export const MEMO_MAX = 200;
export const TICKETS_MIN = 0;
export const TICKETS_MAX = 20;

export interface EntryFormValues {
  name: string;
  amountText: string;
  side: Side;
  tickets: number | null;
  memo: string;
}

/** 유니코드 문자 수 (Rust chars().count() 와 동일) */
export function charCount(s: string): number {
  return [...s].length;
}

/** 식권 수 정규화: 정수가 아니거나 범위를 벗어나면 가장 가까운 값으로 */
export function clampTickets(v: number | null | undefined): number {
  if (v === null || v === undefined || !Number.isFinite(v)) return TICKETS_MIN;
  return Math.min(TICKETS_MAX, Math.max(TICKETS_MIN, Math.trunc(v)));
}

export function buildEntryInput(v: EntryFormValues): { input: EntryInput | null; errors: FieldErrors } {
  const errors: FieldErrors = {};
  const name = v.name.trim();
  const memo = v.memo.trim();

  if (name === "") errors.name = "이름을 입력해 주세요";
  else if (charCount(name) > NAME_MAX) errors.name = `이름은 ${NAME_MAX}자 이하로 입력해 주세요`;

  const amountMsg = amountError(v.amountText);
  if (amountMsg) errors.amount = amountMsg;

  const t = v.tickets;
  if (t === null || !Number.isInteger(t) || t < TICKETS_MIN || t > TICKETS_MAX) {
    errors.tickets = `식권 수는 ${TICKETS_MIN}~${TICKETS_MAX} 사이로 입력해 주세요`;
  }

  if (charCount(memo) > MEMO_MAX) errors.memo = `메모는 ${MEMO_MAX}자 이하로 입력해 주세요`;

  if (Object.keys(errors).length > 0) return { input: null, errors };

  return {
    input: {
      name,
      amountWon: parseManwon(v.amountText) as number,
      side: v.side,
      tickets: t as number,
      memo,
    },
    errors,
  };
}

/** 화면 배치 순서대로 첫 번째 오류 필드 */
export function firstErrorField(errors: FieldErrors): FieldName | null {
  const order: FieldName[] = ["name", "amount", "tickets", "memo"];
  return order.find((f) => errors[f]) ?? null;
}
