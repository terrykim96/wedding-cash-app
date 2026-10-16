<script lang="ts">
  // 이름 검색 / 구분 필터 / 정렬 (FR-6.3 ~ 6.5)
  import HelpTip from "./HelpTip.svelte";
  import type { Side, SortBy, SortDir } from "../lib/types";

  interface Props {
    search: string;
    sideFilter: "all" | Side;
    sortBy: SortBy;
    sortDir: SortDir;
  }
  let { search = $bindable(), sideFilter = $bindable(), sortBy = $bindable(), sortDir = $bindable() }: Props = $props();

  const SIDE_OPTIONS: { value: "all" | Side; label: string }[] = [
    { value: "all", label: "전체" },
    { value: "groom", label: "신랑측" },
    { value: "bride", label: "신부측" },
  ];
</script>

<div class="filter-bar">
  <div class="field inline">
    <label for="filter-search" class="field-label">이름 검색</label>
    <input id="filter-search" type="search" bind:value={search} placeholder="예: 민수" autocomplete="off" data-testid="filter-bar-search-input" />
  </div>

  <fieldset class="field inline radio-group">
    <legend class="field-label">구분</legend>
    {#each SIDE_OPTIONS as o (o.value)}
      <label class="radio">
        <input type="radio" name="filter-side" value={o.value} bind:group={sideFilter} data-testid={`filter-bar-side-${o.value}-radio`} />
        {o.label}
      </label>
    {/each}
  </fieldset>

  <div class="field inline">
    <label for="filter-sort" class="field-label">정렬</label>
    <select id="filter-sort" bind:value={sortBy} data-testid="filter-bar-sort-select">
      <option value="receivedAt">입력순</option>
      <option value="name">이름순</option>
      <option value="amount">금액순</option>
    </select>
    <button
      type="button"
      class="btn small"
      aria-label={sortDir === "asc" ? "오름차순 (누르면 내림차순)" : "내림차순 (누르면 오름차순)"}
      onclick={() => (sortDir = sortDir === "asc" ? "desc" : "asc")}
      data-testid="filter-bar-sort-dir-button"
    >
      {sortDir === "asc" ? "▲ 오름차순" : "▼ 내림차순"}
    </button>
  </div>

  <HelpTip id="filter" align="right" />
</div>
