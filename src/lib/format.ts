// 화면 표시 형식 (business-rules.md BR-14, frontend-components.md 5절)

import { formatWon } from "./amount";
import type { EntryInput, HistoryItem } from "./types";
import { SIDE_LABEL } from "./types";

/**
 * ISO 8601 시각 문자열을 "11-21 11:32" 로 표시한다.
 * 저장된 문자열의 로컬 시각 부분을 그대로 사용한다(입력한 PC 기준 시각).
 */
export function formatShortTime(iso: string): string {
  const m = /^\d{4}-(\d{2})-(\d{2})T(\d{2}):(\d{2})/.exec(iso);
  return m ? `${m[1]}-${m[2]} ${m[3]}:${m[4]}` : iso;
}

/** 파일 경로에서 파일 이름만 */
export function fileName(path: string | null): string {
  if (!path) return "";
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

const FIELD_LABELS: { key: keyof EntryInput; label: string; show: (v: EntryInput) => string }[] = [
  { key: "name", label: "이름", show: (v) => v.name },
  { key: "amountWon", label: "금액", show: (v) => formatWon(v.amountWon) },
  { key: "side", label: "구분", show: (v) => SIDE_LABEL[v.side] },
  { key: "tickets", label: "식권", show: (v) => `${v.tickets}장` },
  { key: "memo", label: "메모", show: (v) => (v.memo === "" ? "(없음)" : v.memo) },
];

/** 이력 변경 내용 (BR-9) */
export function describeHistory(item: HistoryItem): string {
  if (item.kind === "delete") return `삭제 (${formatWon(item.before.amountWon)})`;
  if (item.kind === "restore") return `복구 (${formatWon(item.before.amountWon)})`;
  const after = item.after;
  if (!after) return "수정";
  const changes = FIELD_LABELS.filter((f) => item.before[f.key] !== after[f.key]).map(
    (f) => `${f.label}: ${f.show(item.before)} → ${f.show(after)}`,
  );
  return changes.length > 0 ? changes.join(", ") : "수정";
}

export const HISTORY_KIND_LABEL = {
  update: "수정",
  delete: "삭제",
  restore: "복구",
} as const;
