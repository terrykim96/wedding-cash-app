<script lang="ts">
  import Modal from "./Modal.svelte";
  import { confirmState } from "../lib/dialogs";

  let confirmBtn: HTMLButtonElement | undefined = $state();
  let cancelBtn: HTMLButtonElement | undefined = $state();

  const req = $derived($confirmState);

  // 기본 포커스 (BR-3.2: 큰 금액 확인은 "아니요"에 포커스)
  $effect(() => {
    if (!req) return;
    const target = req.defaultFocus === "confirm" ? confirmBtn : cancelBtn;
    const timer = setTimeout(() => target?.focus(), 0);
    return () => clearTimeout(timer);
  });

  function finish(ok: boolean) {
    const r = $confirmState;
    confirmState.set(null);
    r?.resolve(ok);
  }
</script>

<Modal open={req !== null} title={req?.title ?? ""} role="alertdialog" onclose={() => finish(false)} testid="confirm-dialog">
  <p class="confirm-message" data-testid="confirm-dialog-message">{req?.message ?? ""}</p>
  {#snippet actions()}
    <button
      bind:this={cancelBtn}
      type="button"
      class="btn"
      onclick={() => finish(false)}
      data-testid="confirm-dialog-cancel-button"
    >
      {req?.cancelText ?? "취소"}
    </button>
    <button
      bind:this={confirmBtn}
      type="button"
      class="btn primary"
      class:danger={req?.danger}
      onclick={() => finish(true)}
      data-testid="confirm-dialog-confirm-button"
    >
      {req?.confirmText ?? "확인"}
    </button>
  {/snippet}
</Modal>
