<script lang="ts">
  // 정산 화면 (FR-6): 툴바 + 합계 + 검색/필터/정렬 + 전체 내역
  import SettlementToolbar from "./SettlementToolbar.svelte";
  import SummaryPanel from "./SummaryPanel.svelte";
  import FilterBar from "./FilterBar.svelte";
  import EntryTable from "./EntryTable.svelte";
  import { queryEntries } from "../lib/api";
  import { appState, dataVersion, showError } from "../lib/stores";
  import type { EntryQuery, QueryResult, Side, SortBy, SortDir } from "../lib/types";

  let search = $state("");
  let sideFilter = $state<"all" | Side>("all");
  let sortBy = $state<SortBy>("receivedAt");
  let sortDir = $state<SortDir>("desc");
  let debouncedSearch = $state("");
  let result = $state<QueryResult | null>(null);

  const ready = $derived(!!$appState && !$appState.needsSetup && !$appState.loadError);
  const hasCondition = $derived(debouncedSearch.trim() !== "" || sideFilter !== "all");

  // 검색어 입력은 200ms 디바운스
  $effect(() => {
    const s = search;
    const timer = setTimeout(() => (debouncedSearch = s), 200);
    return () => clearTimeout(timer);
  });

  let seq = 0;
  $effect(() => {
    void $dataVersion;
    const query: EntryQuery = {
      search: debouncedSearch,
      side: sideFilter === "all" ? undefined : sideFilter,
      sortBy,
      sortDir,
    };
    if (!ready) {
      result = null;
      return;
    }
    const mySeq = ++seq;
    queryEntries(query)
      .then((r) => {
        if (mySeq === seq) result = r;
      })
      .catch((e) => {
        if (mySeq === seq) showError(e, "정산 내역을 불러오지 못했습니다");
      });
  });
</script>

<div class="settlement">
  <SettlementToolbar disabled={!ready} />
  {#if result}
    <SummaryPanel total={result.totalSummary} filtered={hasCondition ? result.filteredSummary : null} />
  {/if}
  <FilterBar bind:search bind:sideFilter bind:sortBy bind:sortDir />
  <EntryTable entries={result?.entries ?? []} />
</div>
