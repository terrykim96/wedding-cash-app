<script lang="ts">
  // 변경 이력 표 (BR-9). 전체 이력 화면과 항목별 이력 창에서 함께 사용
  import { confirmAndRestore } from "../lib/actions";
  import { describeHistory, formatShortTime, HISTORY_KIND_LABEL } from "../lib/format";
  import type { HistoryItem } from "../lib/types";

  let { items, testid }: { items: HistoryItem[]; testid: string } = $props();
</script>

{#if items.length === 0}
  <p class="empty" data-testid={`${testid}-empty`}>변경 이력이 없습니다</p>
{:else}
  <table class="table" data-testid={testid}>
    <thead>
      <tr>
        <th scope="col">시각</th>
        <th scope="col">종류</th>
        <th scope="col">이름</th>
        <th scope="col">변경 내용</th>
        <th scope="col">PC</th>
        <th scope="col"><span class="sr-only">작업</span></th>
      </tr>
    </thead>
    <tbody>
      {#each items as h (h.id)}
        <tr>
          <td>{formatShortTime(h.at)}</td>
          <td><span class={`kind-badge ${h.kind}`}>{HISTORY_KIND_LABEL[h.kind]}</span></td>
          <td>{h.entryName}</td>
          <td class="change-cell">{describeHistory(h)}</td>
          <td>{h.pcName}</td>
          <td class="row-actions">
            {#if h.canRestore}
              <button
                type="button"
                class="btn small"
                aria-label={`${h.entryName} 복구`}
                onclick={() => confirmAndRestore(h)}
                data-testid={`${testid}-restore-button-${h.id}`}
              >
                복구
              </button>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
