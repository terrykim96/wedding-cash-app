import { describe, expect, it } from "vitest";
import { HELP } from "./help";
import { TUTORIAL_STEPS } from "./tutorial";

// 화면 컴포넌트 소스를 문자열로 읽어 <HelpTip id="..."> 사용처를 검사한다
const sources = import.meta.glob("../components/*.svelte", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const used = new Set<string>();
for (const text of Object.values(sources)) {
  for (const m of text.matchAll(/<HelpTip\s+id="([^"]+)"/g)) used.add(m[1]);
}

describe("? 도움말 문구", () => {
  it("모든 항목에 제목과 설명이 있다", () => {
    for (const [id, entry] of Object.entries(HELP)) {
      expect(entry.title.trim(), id).not.toBe("");
      expect(entry.body.length, id).toBeGreaterThan(0);
      for (const line of entry.body) expect(line.trim(), id).not.toBe("");
    }
  });

  it("화면에서 쓰는 도움말 id 는 모두 문구가 있다", () => {
    expect(used.size).toBeGreaterThan(0);
    for (const id of used) expect(Object.keys(HELP), `HELP 에 없는 id: ${id}`).toContain(id);
  });

  it("문구를 만들어 두고 화면에 붙이지 않은 항목이 없다", () => {
    for (const id of Object.keys(HELP)) expect([...used], `화면에서 쓰지 않는 id: ${id}`).toContain(id);
  });

  it("같은 id 를 화면에 두 번 붙이지 않는다 (팝오버 id 중복 방지)", () => {
    const all: string[] = [];
    for (const text of Object.values(sources)) {
      for (const m of text.matchAll(/<HelpTip\s+id="([^"]+)"/g)) all.push(m[1]);
    }
    expect(new Set(all).size).toBe(all.length);
  });
});

describe("사용 설명(튜토리얼) 단계", () => {
  it("단계가 있고 제목이 모두 다르다", () => {
    expect(TUTORIAL_STEPS.length).toBeGreaterThanOrEqual(5);
    const titles = TUTORIAL_STEPS.map((s) => s.title);
    expect(new Set(titles).size).toBe(titles.length);
  });

  it("각 단계에 본문(리드/목록/강조)이 하나 이상 있다", () => {
    for (const s of TUTORIAL_STEPS) {
      expect(s.title.trim()).not.toBe("");
      expect(Boolean(s.lead) || (s.items?.length ?? 0) > 0 || Boolean(s.note), s.title).toBe(true);
    }
  });

  it("목록 항목은 단계 안에서 중복되지 않는다 (화면 key 충돌 방지)", () => {
    for (const s of TUTORIAL_STEPS) {
      const items = s.items ?? [];
      expect(new Set(items).size, s.title).toBe(items.length);
    }
  });
});
