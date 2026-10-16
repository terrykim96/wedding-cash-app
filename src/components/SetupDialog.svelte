<script lang="ts">
  // 최초 설정: 이 PC의 접수 구분 선택 (BR-12.4, 12.5)
  import Modal from "./Modal.svelte";
  import { setupPcSide } from "../lib/api";
  import { appState, notifyDataChanged, showError } from "../lib/stores";
  import type { Side } from "../lib/types";
  import { SIDE_LABEL } from "../lib/types";

  let busy = $state(false);
  const open = $derived(!!$appState?.needsSetup);
  const SIDES: Side[] = ["groom", "bride"];

  async function choose(side: Side) {
    if (busy) return;
    busy = true;
    try {
      appState.set(await setupPcSide(side));
      notifyDataChanged();
    } catch (e) {
      showError(e, "설정을 저장하지 못했습니다");
    } finally {
      busy = false;
    }
  }
</script>

<Modal {open} title="처음 설정" closable={false} testid="setup-dialog">
  <p class="setup-question">이 PC는 어느 쪽 접수용인가요?</p>
  <div class="setup-choices">
    {#each SIDES as s (s)}
      <button
        type="button"
        class={`side-button big ${s}`}
        disabled={busy}
        onclick={() => choose(s)}
        data-testid={`setup-dialog-${s}-button`}
      >
        {SIDE_LABEL[s]} 접수 PC
      </button>
    {/each}
  </div>
  <p class="hint">
    선택한 구분에 따라 데이터 파일(축의금_{SIDE_LABEL.groom}.json / 축의금_{SIDE_LABEL.bride}.json)이 만들어집니다.
    접수할 때 반대쪽 하객도 선택할 수 있습니다.
    잘못 골랐다면 프로그램 폴더의 settings.json 을 지우고 다시 실행하세요.
  </p>
</Modal>
