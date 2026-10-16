<script lang="ts">
  // 기능 옆의 ? 버튼. 누르면 설명 팝오버가 열리고, 다시 누르거나 바깥을 누르거나 Esc 를 누르면 닫힌다.
  // 브라우저 기본 popover API 는 오래된 macOS(WebKit)에서 지원되지 않아 직접 구현했다.
  import { HELP, type HelpId } from "../lib/help";
  import { openHelp } from "../lib/stores";

  interface Props {
    id: HelpId;
    /** 팝오버가 펼쳐지는 방향. 화면 오른쪽 끝에 가까운 버튼은 "right" */
    align?: "left" | "right";
  }

  let { id, align = "left" }: Props = $props();

  let wrapper: HTMLSpanElement | undefined = $state();
  let button: HTMLButtonElement | undefined = $state();

  const entry = $derived(HELP[id]);
  const open = $derived($openHelp === id);

  function toggle() {
    openHelp.set(open ? null : id);
  }

  function onWindowPointerDown(e: PointerEvent) {
    if (open && wrapper && !wrapper.contains(e.target as Node)) openHelp.set(null);
  }

  function onWindowKeydown(e: KeyboardEvent) {
    if (open && e.key === "Escape") {
      e.preventDefault();
      openHelp.set(null);
      button?.focus();
    }
  }
</script>

<svelte:window onpointerdown={onWindowPointerDown} onkeydown={onWindowKeydown} />

<span class="help" bind:this={wrapper}>
  <button
    bind:this={button}
    type="button"
    class="help-button"
    aria-label={`도움말: ${entry.title}`}
    aria-expanded={open}
    aria-controls={`help-${id}`}
    onclick={toggle}
    data-testid={`help-${id}-button`}
  >
    ?
  </button>
  {#if open}
    <div id={`help-${id}`} class={`help-popover ${align}`} role="region" aria-label={entry.title} data-testid={`help-${id}-popover`}>
      <p class="help-title">{entry.title}</p>
      {#each entry.body as line (line)}
        <p class="help-text">{line}</p>
      {/each}
    </div>
  {/if}
</span>
