import { describe, expect, it } from "vitest";
import { buildEntryInput, clampTickets, firstErrorField } from "./entry";
import { describeHistory, fileName, formatShortTime } from "./format";
import type { HistoryItem } from "./types";

const base = { name: "김민수", amountText: "5.5", side: "groom" as const, tickets: 1, memo: "" };

describe("buildEntryInput (BR-1 화면 단계)", () => {
  it("정상 입력은 원 단위 EntryInput 으로 바뀐다", () => {
    const { input, errors } = buildEntryInput({ ...base, name: "  김민수 ", memo: " 대학 동기 " });
    expect(errors).toEqual({});
    expect(input).toEqual({ name: "김민수", amountWon: 55_000, side: "groom", tickets: 1, memo: "대학 동기" });
  });

  it("식권 수는 0~20 정수만 허용", () => {
    expect(buildEntryInput({ ...base, tickets: 0 }).input).not.toBeNull();
    expect(buildEntryInput({ ...base, tickets: 20 }).input).not.toBeNull();
    for (const t of [-1, 21, 1.5, null]) {
      expect(buildEntryInput({ ...base, tickets: t }).errors.tickets).toBe("식권 수는 0~20 사이로 입력해 주세요");
    }
  });

  it("이름/메모 길이와 금액 오류를 모두 알려준다", () => {
    const { input, errors } = buildEntryInput({ ...base, name: "가".repeat(31), amountText: "", memo: "메".repeat(201) });
    expect(input).toBeNull();
    expect(Object.keys(errors).sort()).toEqual(["amount", "memo", "name"]);
    expect(firstErrorField(errors)).toBe("name");
  });

  it("clampTickets", () => {
    expect(clampTickets(-3)).toBe(0);
    expect(clampTickets(25)).toBe(20);
    expect(clampTickets(null)).toBe(0);
    expect(clampTickets(3.7)).toBe(3);
  });
});

describe("format", () => {
  it("formatShortTime 은 입력 PC 기준 시각을 그대로 보여준다", () => {
    expect(formatShortTime("2026-11-21T11:32:10+09:00")).toBe("11-21 11:32");
  });

  it("fileName 은 Windows/macOS 경로 모두 처리", () => {
    expect(fileName("D:\\USB\\축의금_통합.json")).toBe("축의금_통합.json");
    expect(fileName("/Users/me/Documents/축의금/축의금_신랑측.json")).toBe("축의금_신랑측.json");
  });

  it("describeHistory 는 바뀐 필드만 보여준다", () => {
    const item: HistoryItem = {
      id: "h1",
      entryId: "e1",
      kind: "update",
      at: "2026-11-21T11:40:00+09:00",
      before: { name: "김민슈", amountWon: 50_000, side: "groom", tickets: 1, memo: "" },
      after: { name: "김민수", amountWon: 100_000, side: "groom", tickets: 1, memo: "" },
      pcName: "PC",
      entryName: "김민수",
      canRestore: false,
    };
    expect(describeHistory(item)).toBe("이름: 김민슈 → 김민수, 금액: 50,000원 → 100,000원");
    expect(describeHistory({ ...item, kind: "delete", after: null })).toBe("삭제 (50,000원)");
  });
});
