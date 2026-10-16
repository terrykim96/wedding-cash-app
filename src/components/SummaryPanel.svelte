<script lang="ts">
  // 합계 (FR-6.1, FR-6.6)
  import HelpTip from "./HelpTip.svelte";
  import { formatWon } from "../lib/amount";
  import type { SideSummary, Summary } from "../lib/types";

  let { total, filtered }: { total: Summary; filtered: Summary | null } = $props();

  const cards: { key: keyof Summary; label: string }[] = [
    { key: "total", label: "전체" },
    { key: "groom", label: "신랑측" },
    { key: "bride", label: "신부측" },
  ];

  const line = (s: SideSummary) => `${s.count}건 · 식권 ${s.tickets}장`;
</script>

<div class="summary-wrap">
  <div class="title-row">
    <h2 id="summary-title" class="section-title">합계</h2>
    <HelpTip id="summary" />
  </div>

  <section class="summary" aria-labelledby="summary-title" data-testid="summary-panel">
    {#each cards as c (c.key)}
      <div class={`summary-card ${c.key}`} data-testid={`summary-panel-${c.key}`}>
        <h3 class="summary-label">{c.label}</h3>
        <p class="summary-amount">{formatWon(total[c.key].amountWon)}</p>
        <p class="summary-meta">{line(total[c.key])}</p>
      </div>
    {/each}
  </section>

  {#if filtered}
    <p class="filtered-summary" role="status" data-testid="summary-panel-filtered">
      조건 결과: {filtered.total.count}건, 금액 {formatWon(filtered.total.amountWon)}, 식권 {filtered.total.tickets}장
    </p>
  {/if}
</div>
