import { describe, expect, it } from "vitest";
import {
  AMOUNT_FORMAT_ERROR,
  amountError,
  formatManwon,
  formatWon,
  parseManwon,
  toManwonText,
} from "./amount";

describe("parseManwon (BR-2)", () => {
  it("만원 단위 예시를 원 단위로 바꾼다 (BR-2.4)", () => {
    expect(parseManwon("5")).toBe(50_000);
    expect(parseManwon("5.5")).toBe(55_000);
    expect(parseManwon("10")).toBe(100_000);
    expect(parseManwon("0.5")).toBe(5_000);
    expect(parseManwon("1000")).toBe(10_000_000);
    expect(parseManwon(" 15 ")).toBe(150_000);
  });

  it("부동소수점 오차가 없다", () => {
    expect(parseManwon("0.3")).toBe(3_000);
    expect(parseManwon("1.1")).toBe(11_000);
    expect(parseManwon("9999.9")).toBe(99_999_000);
  });

  it("형식이 틀리면 null (BR-2.5)", () => {
    for (const bad of ["", " ", "5.55", "5만", "-1", "1,000", "abc", "5.", ".5", "10000", "1e3", "５"]) {
      expect(parseManwon(bad), bad).toBeNull();
    }
  });
});

describe("amountError", () => {
  it("정상 금액은 null", () => {
    expect(amountError("5")).toBeNull();
    expect(amountError("0.1")).toBeNull();
    expect(amountError("1000")).toBeNull();
  });

  it("빈 값, 0, 형식 오류, 상한 초과", () => {
    expect(amountError("")).toBe("금액을 입력해 주세요");
    expect(amountError("0")).toBe("금액을 입력해 주세요");
    expect(amountError("0.0")).toBe("금액을 입력해 주세요");
    expect(amountError("5.55")).toBe(AMOUNT_FORMAT_ERROR);
    expect(amountError("1000.1")).toBe("1,000만원을 넘을 수 없습니다");
  });
});

describe("표시 형식", () => {
  it("formatWon", () => {
    expect(formatWon(55_000)).toBe("55,000원");
    expect(formatWon(10_000_000)).toBe("10,000,000원");
    expect(formatWon(0)).toBe("0원");
  });

  it("toManwonText / formatManwon 은 parseManwon 과 왕복된다", () => {
    for (const won of [1_000, 5_000, 50_000, 55_000, 100_000, 1_000_000, 10_000_000]) {
      expect(parseManwon(toManwonText(won))).toBe(won);
    }
    expect(formatManwon(1_000_000)).toBe("100만원");
    expect(formatManwon(55_000)).toBe("5.5만원");
  });
});
