<script lang="ts">
  // 헤더: PC 구분, 현재 작업 파일, 저장 위치 안내, 로드 오류 (FR-9.3, NFR-2.2, BR-13.4)
  import HelpTip from "./HelpTip.svelte";
  import { openOtherFile } from "../lib/actions";
  import { fileName } from "../lib/format";
  import { appState, tutorialOpen } from "../lib/stores";
  import { SIDE_LABEL } from "../lib/types";

  const s = $derived($appState);
</script>

<header class="app-header">
  <h1 class="brand">축의금 관리</h1>
  {#if s}
    <div class="header-info">
      {#if s.pcSide}
        <span class={`side-badge ${s.pcSide}`} data-testid="app-header-pc-side">{SIDE_LABEL[s.pcSide]} 접수 PC</span>
      {/if}
      {#if s.activeFilePath}
        <span class="file-name" title={s.activeFilePath} data-testid="app-header-file-name">
          파일: {fileName(s.activeFilePath)}
        </span>
        {#if s.activeFileKind === "merged"}
          <span class="badge merged" data-testid="app-header-merged-badge">통합 파일</span>
        {/if}
        <HelpTip id="datafile" align="right" />
      {/if}
      {#if !s.needsSetup}
        <button type="button" class="btn small tutorial-button" onclick={() => tutorialOpen.set(true)} data-testid="app-header-tutorial-button">
          사용 설명
        </button>
      {/if}
    </div>
  {/if}
</header>

{#if s?.storageNotice}
  <div class="banner info" role="status" data-testid="app-header-notice">{s.storageNotice}</div>
{/if}
{#if s?.loadError}
  <div class="banner error" role="alert" data-testid="app-header-load-error">
    <span>데이터 파일을 열 수 없습니다: {s.loadError}</span>
    <button type="button" class="btn small" onclick={openOtherFile} data-testid="app-header-open-file-button">다른 파일 열기</button>
  </div>
{/if}
