<script lang="ts">
  // 정산 화면 상단 메뉴: 병합, 다른 파일 열기, 내보내기 (설계 Q6: A / FR-7, FR-8)
  import HelpTip from "./HelpTip.svelte";
  import MergeResultDialog from "./MergeResultDialog.svelte";
  import { exportCsv, exportXlsx, mergeFiles } from "../lib/api";
  import { openOtherFile, pickLedgerFile, pickSavePath } from "../lib/actions";
  import { fileName } from "../lib/format";
  import { appState, showError, showToast } from "../lib/stores";
  import type { MergeReport } from "../lib/types";

  let { disabled }: { disabled: boolean } = $props();

  let busy = $state(false);
  let report = $state<MergeReport | null>(null);

  function today(): string {
    const d = new Date();
    const p = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
  }

  /** 기본 내보내기 파일 이름: 현재 파일 이름 + 날짜 (BR-11.10) */
  function exportName(ext: string): string {
    const base = fileName($appState?.activeFilePath ?? "축의금.json").replace(/\.json$/i, "");
    return `${base}_${today()}.${ext}`;
  }

  async function run(task: () => Promise<void>) {
    if (busy) return;
    busy = true;
    try {
      await task();
    } finally {
      busy = false;
    }
  }

  const merge = () =>
    run(async () => {
      try {
        const a = await pickLedgerFile("병합할 첫 번째 파일 선택 (예: 축의금_신랑측.json)");
        if (!a) return;
        const b = await pickLedgerFile("병합할 두 번째 파일 선택 (예: 축의금_신부측.json)");
        if (!b) return;
        const out = await pickSavePath("통합 파일 저장 위치", "축의금_통합.json", "축의금 데이터 파일", "json");
        if (!out) return;
        report = await mergeFiles(a, b, out);
      } catch (e) {
        showError(e, "병합하지 못했습니다");
      }
    });

  const exportTo = (kind: "csv" | "xlsx") =>
    run(async () => {
      try {
        const path =
          kind === "csv"
            ? await pickSavePath("CSV로 내보내기", exportName("csv"), "CSV 파일", "csv")
            : await pickSavePath("엑셀(XLSX)로 내보내기", exportName("xlsx"), "엑셀 파일", "xlsx");
        if (!path) return;
        if (kind === "csv") await exportCsv(path);
        else await exportXlsx(path);
        showToast("ok", `내보냈습니다: ${fileName(path)}`);
      } catch (e) {
        showError(e, "내보내지 못했습니다");
      }
    });
</script>

<div class="toolbar" role="toolbar" aria-label="파일 메뉴">
  <button type="button" class="btn" disabled={busy || disabled} onclick={merge} data-testid="settlement-toolbar-merge-button">파일 병합</button>
  <HelpTip id="merge" />
  <button type="button" class="btn toolbar-gap" disabled={busy} onclick={() => run(openOtherFile)} data-testid="settlement-toolbar-open-button">다른 파일 열기</button>
  <HelpTip id="open" />
  <span class="toolbar-spacer"></span>
  <button type="button" class="btn" disabled={busy || disabled} onclick={() => exportTo("csv")} data-testid="settlement-toolbar-export-csv-button">CSV 내보내기</button>
  <button type="button" class="btn" disabled={busy || disabled} onclick={() => exportTo("xlsx")} data-testid="settlement-toolbar-export-xlsx-button">엑셀(XLSX) 내보내기</button>
  <HelpTip id="export" align="right" />
</div>

<MergeResultDialog {report} onclose={() => (report = null)} />
