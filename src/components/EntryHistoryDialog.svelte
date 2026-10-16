<script lang="ts">
  // 항목별 변경 이력 (설계 Q4: C)
  import Modal from "./Modal.svelte";
  import HistoryTable from "./HistoryTable.svelte";
  import { listHistory } from "../lib/api";
  import { historyTarget } from "../lib/dialogs";
  import { dataVersion, showError } from "../lib/stores";
  import type { HistoryItem } from "../lib/types";

  let items = $state<HistoryItem[]>([]);
  const target = $derived($historyTarget);

  $effect(() => {
    void $dataVersion;
    const t = target;
    if (!t) {
      items = [];
      return;
    }
    let cancelled = false;
    listHistory(t.entryId)
      .then((list) => {
        if (!cancelled) items = list;
      })
      .catch((e) => showError(e, "이력을 불러오지 못했습니다"));
    return () => {
      cancelled = true;
    };
  });

  function close() {
    historyTarget.set(null);
  }
</script>

<Modal open={target !== null} title={`변경 이력 - ${target?.name ?? ""}`} onclose={close} testid="entry-history-dialog" wide>
  <HistoryTable {items} testid="entry-history-dialog-table" />
  {#snippet actions()}
    <button type="button" class="btn" onclick={close} data-testid="entry-history-dialog-close-button">닫기</button>
  {/snippet}
</Modal>
