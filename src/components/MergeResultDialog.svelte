<script lang="ts">
  // 병합 결과 + 통합 파일 열기 여부 (FR-8.5, FR-8.6)
  import Modal from "./Modal.svelte";
  import { openFileByPath } from "../lib/actions";
  import { fileName } from "../lib/format";
  import { showError } from "../lib/stores";
  import type { MergeReport } from "../lib/types";

  let { report, onclose }: { report: MergeReport | null; onclose: () => void } = $props();
  let opening = $state(false);

  async function openMerged() {
    if (!report || opening) return;
    opening = true;
    try {
      await openFileByPath(report.outputPath);
      onclose();
    } catch (e) {
      showError(e, "통합 파일을 열지 못했습니다");
    } finally {
      opening = false;
    }
  }
</script>

<Modal open={report !== null} title="병합 완료" {onclose} testid="merge-result-dialog">
  {#if report}
    <p data-testid="merge-result-dialog-summary">
      첫 번째 파일 {report.fromA}건 + 두 번째 파일 {report.fromB}건 → 통합 <strong>{report.merged}건</strong>
      {#if report.duplicatesResolved > 0}(중복 {report.duplicatesResolved}건 정리){/if}
    </p>
    <p class="meta">저장 위치: {fileName(report.outputPath)}</p>
    <p>통합 파일을 열까요? 열면 정산, 수정, 내보내기를 통합 파일 기준으로 할 수 있습니다.</p>
  {/if}
  {#snippet actions()}
    <button type="button" class="btn" onclick={onclose} data-testid="merge-result-dialog-close-button">닫기</button>
    <button type="button" class="btn primary" disabled={opening} onclick={openMerged} data-testid="merge-result-dialog-open-button">
      통합 파일 열기
    </button>
  {/snippet}
</Modal>
