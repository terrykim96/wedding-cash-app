<script lang="ts">
  // 변경 이력 화면: 전체 이력 시간순 (FR-5.4)
  import HelpTip from "./HelpTip.svelte";
  import HistoryTable from "./HistoryTable.svelte";
  import { listHistory } from "../lib/api";
  import { appState, dataVersion } from "../lib/stores";
  import type { HistoryItem } from "../lib/types";

  let items = $state<HistoryItem[]>([]);
  const ready = $derived(!!$appState && !$appState.needsSetup && !$appState.loadError);

  $effect(() => {
    void $dataVersion;
    if (!ready) {
      items = [];
      return;
    }
    let cancelled = false;
    listHistory()
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

<section class="card" aria-labelledby="history-title">
  <div class="title-row">
    <h2 id="history-title" class="section-title">변경 이력</h2>
    <HelpTip id="history" />
  </div>
  <p class="hint">수정·삭제·복구한 기록입니다. 삭제한 항목은 여기서 복구할 수 있습니다.</p>
  <HistoryTable {items} testid="history-view-table" />
</section>
