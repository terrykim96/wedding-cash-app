<script lang="ts">
  // 사용 설명(튜토리얼): 7단계 대화상자.
  // 끝까지 보거나 "다음부터 보지 않기"를 체크하면 settings.json 에 기록해 다음 실행부터 자동으로 열지 않는다.
  import Modal from "./Modal.svelte";
  import { markTutorialSeen } from "../lib/api";
  import { TUTORIAL_STEPS } from "../lib/tutorial";
  import { appState, showError, tutorialOpen } from "../lib/stores";

  const total = TUTORIAL_STEPS.length;

  let step = $state(0);
  let dontShowAgain = $state(false);
  let nextButton: HTMLButtonElement | undefined = $state();

  const open = $derived($tutorialOpen);
  const current = $derived(TUTORIAL_STEPS[step]);
  const isLast = $derived(step === total - 1);
  const alreadySeen = $derived($appState?.tutorialSeen ?? false);

  // 열릴 때마다 처음 단계부터 시작하고 [다음] 버튼에 포커스
  $effect(() => {
    if (!open) return;
    step = 0;
    dontShowAgain = false;
    const timer = setTimeout(() => nextButton?.focus(), 0);
    return () => clearTimeout(timer);
  });

  function prev() {
    if (step > 0) step -= 1;
  }

  function next() {
    if (isLast) {
      void close(true);
    } else {
      step += 1;
    }
  }

  async function close(completed: boolean) {
    const remember = (completed || dontShowAgain) && !alreadySeen;
    tutorialOpen.set(false);
    if (!remember) return;
    try {
      await markTutorialSeen();
      appState.update((s) => (s ? { ...s, tutorialSeen: true } : s));
    } catch (e) {
      showError(e, "사용 설명 확인 여부를 저장하지 못했습니다");
    }
  }

  function onWindowKeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === "ArrowRight") {
      e.preventDefault();
      next();
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      prev();
    }
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<Modal {open} title={current.title} wide onclose={() => close(false)} testid="tutorial-dialog">
  <div class="tutorial-body">
    <div class="tutorial-progress">
      <span data-testid="tutorial-dialog-progress">{step + 1} / {total}</span>
      <span class="tutorial-dots" aria-hidden="true">
        {#each TUTORIAL_STEPS as s, i (s.title)}
          <span class="dot" class:active={i === step} class:done={i < step}></span>
        {/each}
      </span>
    </div>

    {#if current.lead}
      <p class="tutorial-lead">{current.lead}</p>
    {/if}

    {#if current.items}
      {#if current.ordered}
        <ol class="tutorial-list">
          {#each current.items as item (item)}<li>{item}</li>{/each}
        </ol>
      {:else}
        <ul class="tutorial-list">
          {#each current.items as item (item)}<li>{item}</li>{/each}
        </ul>
      {/if}
    {/if}

    {#if current.note}
      <p class="tutorial-note" data-testid="tutorial-dialog-note">{current.note}</p>
    {/if}
  </div>

  {#snippet actions()}
    <div class="tutorial-actions">
      {#if alreadySeen}
        <span></span>
      {:else}
        <label class="check-label">
          <input type="checkbox" bind:checked={dontShowAgain} data-testid="tutorial-dialog-dont-show-checkbox" />
          다음부터 자동으로 열지 않기
        </label>
      {/if}
      <div class="tutorial-buttons">
        <button type="button" class="btn" onclick={() => close(false)} data-testid="tutorial-dialog-skip-button">
          {isLast ? "닫기" : "건너뛰기"}
        </button>
        <button type="button" class="btn" disabled={step === 0} onclick={prev} data-testid="tutorial-dialog-prev-button">이전</button>
        <button bind:this={nextButton} type="button" class="btn primary" onclick={next} data-testid="tutorial-dialog-next-button">
          {isLast ? "시작하기" : "다음"}
        </button>
      </div>
    </div>
  {/snippet}
</Modal>
