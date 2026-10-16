<script lang="ts">
  // 접수 화면 최근 입력 10건 (FR-3.1, FR-4.1)
  import HelpTip from "./HelpTip.svelte";
  import SideBadge from "./SideBadge.svelte";
  import { listRecent } from "../lib/api";
  import { confirmAndDelete } from "../lib/actions";
  import { formatWon } from "../lib/amount";
  import { editTarget } from "../lib/dialogs";
  import { formatShortTime } from "../lib/format";
  import { appState, dataVersion } from "../lib/stores";
  import type { Entry } from "../lib/types";

  let items = $state<Entry[]>([]);
  const ready = $derived(!!$appState && !$appState.needsSetup && !$appState.loadError);

  $effect(() => {
    void $dataVersion;
    if (!ready) {
      items = [];
      return;
    }
    let cancelled = false;
    listRecent(10)
      .then((list) => {
        if (!cancelled) items = list;
      })
      .catch(() => {
        if (!cancelled) items = [];
      });
    return () => {
      cancelled = true;
    };
  });
</script>

<section class="card recent" aria-labelledby="recent-title" data-testid="recent-list">
  <div class="title-row">
    <h2 id="recent-title" class="section-title">최근 입력 (10건)</h2>
    <HelpTip id="recent" />
  </div>
  {#if items.length === 0}
    <p class="empty">아직 입력한 내역이 없습니다</p>
  {:else}
    <table class="table">
      <thead>
        <tr>
          <th scope="col">시각</th>
          <th scope="col">구분</th>
          <th scope="col">이름</th>
          <th scope="col" class="num">금액</th>
          <th scope="col" class="num">식권</th>
          <th scope="col">메모</th>
          <th scope="col"><span class="sr-only">작업</span></th>
        </tr>
      </thead>
      <tbody>
        {#each items as e (e.id)}
          <tr data-testid={`recent-list-row-${e.id}`}>
            <td>{formatShortTime(e.receivedAt)}</td>
            <td><SideBadge side={e.side} /></td>
            <td>{e.name}</td>
            <td class="num">{formatWon(e.amountWon)}</td>
            <td class="num">{e.tickets}</td>
            <td class="memo-cell">{e.memo}</td>
            <td class="row-actions">
              <button
                type="button"
                class="btn small"
                aria-label={`${e.name} 수정`}
                onclick={() => editTarget.set(e)}
                data-testid={`recent-list-edit-button-${e.id}`}
              >
                수정
              </button>
              <button
                type="button"
                class="btn small danger-outline"
                aria-label={`${e.name} 삭제`}
                onclick={() => confirmAndDelete(e)}
                data-testid={`recent-list-delete-button-${e.id}`}
              >
                삭제
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>
