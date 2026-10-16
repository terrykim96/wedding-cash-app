<script lang="ts">
  // 공통 모달: 네이티브 <dialog> 사용 (포커스 가두기, Esc 처리, 포커스 복귀)
  import type { Snippet } from "svelte";

  interface Props {
    open: boolean;
    title: string;
    testid: string;
    role?: "dialog" | "alertdialog";
    /** false 이면 Esc 로 닫을 수 없음 */
    closable?: boolean;
    wide?: boolean;
    onclose?: () => void;
    children: Snippet;
    actions?: Snippet;
  }

  let { open, title, testid, role = "dialog", closable = true, wide = false, onclose, children, actions }: Props = $props();

  let el: HTMLDialogElement | undefined = $state();
  let returnFocus: HTMLElement | null = null;
  const titleId = $derived(`${testid}-title`);

  $effect(() => {
    if (!el) return;
    if (open && !el.open) {
      returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      el.showModal();
    } else if (!open && el.open) {
      el.close();
      const target = returnFocus;
      returnFocus = null;
      if (target && document.contains(target)) target.focus();
    }
  });

  function onCancel(e: Event) {
    e.preventDefault();
    if (closable) onclose?.();
  }
</script>

<dialog bind:this={el} class="modal" class:wide {role} aria-labelledby={titleId} aria-modal="true" oncancel={onCancel} data-testid={testid}>
  <h2 id={titleId} class="modal-title">{title}</h2>
  <div class="modal-body">
    {@render children()}
  </div>
  {#if actions}
    <div class="modal-actions">
      {@render actions()}
    </div>
  {/if}
</dialog>
