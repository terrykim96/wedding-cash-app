// 금액 입력 도우미 (business-rules.md BR-2)
// 최종 검증은 Rust validation.rs 가 수행한다. 여기서는 입력 변환과 미리보기만 담당한다.

/** 빠른 금액 버튼 (만원) */
export const QUICK_AMOUNTS_MANWON = [5, 10, 15, 20] as const;

/** 큰 금액 확인 기준 (BR-3.1) */
export const LARGE_AMOUNT_WON = 1_000_000;

/** 금액 상한 (BR-1.5) */
export const MAX_AMOUNT_WON = 10_000_000;

export const AMOUNT_FORMAT_ERROR = "만원 단위 숫자로 입력해 주세요 (예: 5, 5.5)";

const MANWON_PATTERN = /^(\d{1,4})(?:\.(\d))?$/;

/**
 * 만원 단위 문자열을 원 단위 정수로 바꾼다. 형식이 틀리면 null.
 * 부동소수점 연산 없이 정수부/소수부를 나눠 계산한다.
 *   "5" → 50000, "5.5" → 55000, "0.5" → 5000
 */
export function parseManwon(text: string): number | null {
  const m = MANWON_PATTERN.exec(text.trim());
  if (!m) return null;
  const whole = Number(m[1]);
  const tenth = m[2] === undefined ? 0 : Number(m[2]);
  return whole * 10_000 + tenth * 1_000;
}

/** 원 단위 정수를 만원 단위 입력 문자열로 바꾼다 (수정 화면 초기값용). 55000 → "5.5" */
export function toManwonText(won: number): string {
  const whole = Math.floor(won / 10_000);
  const tenth = Math.floor((won % 10_000) / 1_000);
  return tenth === 0 ? String(whole) : `${whole}.${tenth}`;
}

/** 55000 → "55,000원" */
export function formatWon(won: number): string {
  return `${won.toLocaleString("ko-KR")}원`;
}

/** 1000000 → "100만원", 55000 → "5.5만원" */
export function formatManwon(won: number): string {
  return `${toManwonText(won)}만원`;
}

/**
 * 화면 단계 금액 검증. 오류 메시지 또는 null.
 * Rust 쪽 규칙(BR-1.3 ~ 1.5)과 같은 문구를 쓴다.
 */
export function amountError(text: string): string | null {
  if (text.trim() === "") return "금액을 입력해 주세요";
  const won = parseManwon(text);
  if (won === null) return AMOUNT_FORMAT_ERROR;
  if (won < 1_000) return "금액을 입력해 주세요";
  if (won > MAX_AMOUNT_WON) return "1,000만원을 넘을 수 없습니다";
  return null;
}
