<script lang="ts">
  // 정산 화면 전체 내역 (FR-6.2, FR-4.1, 설계 Q4: C)
  import HelpTip from "./HelpTip.svelte";
  import SideBadge from "./SideBadge.svelte";
  import { confirmAndDelete } from "../lib/actions";
  import { formatWon } from "../lib/amount";
  import { editTarget, historyTarget } from "../lib/dialogs";
  import { formatShortTime } from "../lib/format";
  import type { Entry } from "../lib/types";

  let { entries }: { entries: Entry[] } = $props();
</script>

<section class="card table-card" aria-labelledby="entry-table-title" data-testid="entry-table">
  <div class="title-row">
    <h2 id="entry-table-title" class="section-title">전체 내역</h2>
    <HelpTip id="table" />
  </div>
  {#if entries.length === 0}
    <p class="empty">내역이 없습니다</p>
  {:else}
    <div class="table-scroll">
      <table class="table">
        <thead>
          <tr>
            <th scope="col" class="num">번호</th>
            <th scope="col">구분</th>
            <th scope="col">이름</th>
            <th scope="col" class="num">금액</th>
            <th scope="col" class="num">식권</th>
            <th scope="col">메모</th>
            <th scope="col">접수 시각</th>
            <th scope="col">입력 PC</th>
            <th scope="col"><span class="sr-only">작업</span></th>
          </tr>
        </thead>
        <tbody>
          {#each entries as e, i (e.id)}
            <tr data-testid={`entry-table-row-${e.id}`}>
              <td class="num">{i + 1}</td>
              <td><SideBadge side={e.side} /></td>
              <td>{e.name}</td>
              <td class="num">{formatWon(e.amountWon)}</td>
              <td class="num">{e.tickets}</td>
              <td class="memo-cell">{e.memo}</td>
              <td>{formatShortTime(e.receivedAt)}</td>
              <td>{e.sourcePc}</td>
              <td class="row-actions">
                <button type="button" class="btn small" aria-label={`${e.name} 수정`} onclick={() => editTarget.set(e)} data-testid={`entry-table-edit-button-${e.id}`}>수정</button>
                <button type="button" class="btn small danger-outline" aria-label={`${e.name} 삭제`} onclick={() => confirmAndDelete(e)} data-testid={`entry-table-delete-button-${e.id}`}>삭제</button>
                <button
                  type="button"
                  class="btn small"
                  aria-label={`${e.name} 변경 이력`}
                  onclick={() => historyTarget.set({ entryId: e.id, name: e.name })}
                  data-testid={`entry-table-history-button-${e.id}`}
                >
                  이력
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>
