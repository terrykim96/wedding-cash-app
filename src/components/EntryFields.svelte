<script lang="ts">
  // 접수/수정 공통 입력 필드 (frontend-components.md EntryForm)
  import HelpTip from "./HelpTip.svelte";
  import { formatWon, parseManwon, QUICK_AMOUNTS_MANWON } from "../lib/amount";
  import { clampTickets, MEMO_MAX, NAME_MAX, TICKETS_MAX, TICKETS_MIN } from "../lib/entry";
  import type { FieldErrors, FieldName, Side } from "../lib/types";
  import { SIDE_LABEL } from "../lib/types";

  interface Props {
    name: string;
    amountText: string;
    side: Side;
    tickets: number | null;
    memo: string;
    errors: FieldErrors;
    /** id / data-testid 접두어 (예: "entry-form") */
    prefix: string;
    disabled?: boolean;
    /** 접수 화면용 큰 레이아웃 */
    large?: boolean;
    /** 항목마다 ? 도움말을 붙일지 (수정 창에서는 끔) */
    help?: boolean;
    onsubmit: () => void;
    onsidechange?: (side: Side) => void;
  }

  let {
    name = $bindable(),
    amountText = $bindable(),
    side = $bindable(),
    tickets = $bindable(),
    memo = $bindable(),
    errors,
    prefix,
    disabled = false,
    large = false,
    help = false,
    onsubmit,
    onsidechange,
  }: Props = $props();

  const SIDES: Side[] = ["groom", "bride"];

  let nameEl: HTMLInputElement | undefined = $state();
  let amountEl: HTMLInputElement | undefined = $state();
  let ticketsEl: HTMLInputElement | undefined = $state();
  let memoEl: HTMLInputElement | undefined = $state();

  const preview = $derived.by(() => {
    const won = parseManwon(amountText);
    return won === null ? "" : formatWon(won);
  });

  export function focusField(field: FieldName): void {
    const el = { name: nameEl, amount: amountEl, tickets: ticketsEl, memo: memoEl }[field];
    el?.focus();
    if (field === "amount" || field === "tickets") el?.select();
  }

  // 한글 조합 중 Enter(isComposing)는 무시한다.
  // Windows WebView2 는 조합 종료 후 Enter keydown 을 한 번 더 보내므로 그때 처리된다.
  function isEnter(e: KeyboardEvent): boolean {
    return e.key === "Enter" && !e.isComposing;
  }

  function onNameKeydown(e: KeyboardEvent) {
    if (isEnter(e)) {
      e.preventDefault();
      focusField("amount");
    }
  }

  function onSubmitKeydown(e: KeyboardEvent) {
    if (isEnter(e)) {
      e.preventDefault();
      onsubmit();
    }
  }

  function chooseSide(s: Side) {
    if (side === s) return;
    side = s;
    onsidechange?.(s);
  }

  function setQuickAmount(manwon: number) {
    amountText = String(manwon);
    amountEl?.focus();
  }

  function stepTickets(delta: number) {
    tickets = clampTickets((tickets ?? 0) + delta);
  }

  // 음수, 소수, 지수 입력 차단
  function onTicketsKeydown(e: KeyboardEvent) {
    if (["-", "+", "e", "E", ".", ","].includes(e.key)) {
      e.preventDefault();
      return;
    }
    onSubmitKeydown(e);
  }

  function onTicketsBlur() {
    tickets = clampTickets(tickets);
  }

  const describedBy = (field: FieldName) => (errors[field] ? `${prefix}-${field}-error` : undefined);
</script>

<div class="entry-fields" class:large>
  <div class="field side-field" role="group" aria-labelledby={`${prefix}-side-label`}>
    <div class="label-row">
      <span id={`${prefix}-side-label`} class="field-label">구분</span>
      {#if help}<HelpTip id="entry-side" />{/if}
    </div>
    <div class="side-toggle">
      {#each SIDES as s (s)}
        <button
          type="button"
          class={`side-button ${s}`}
          class:selected={side === s}
          aria-pressed={side === s}
          {disabled}
          onclick={() => chooseSide(s)}
          data-testid={`${prefix}-side-${s}-button`}
        >
          {#if side === s}<span class="check" aria-hidden="true">✔</span>{/if}{SIDE_LABEL[s]}
        </button>
      {/each}
    </div>
  </div>

  <div class="field">
    <div class="label-row">
      <label for={`${prefix}-name`} class="field-label">이름</label>
      {#if help}<HelpTip id="entry-name" />{/if}
    </div>
    <input
      bind:this={nameEl}
      bind:value={name}
      id={`${prefix}-name`}
      type="text"
      autocomplete="off"
      maxlength={NAME_MAX + 10}
      aria-invalid={errors.name ? "true" : undefined}
      aria-describedby={describedBy("name")}
      {disabled}
      onkeydown={onNameKeydown}
      data-testid={`${prefix}-name-input`}
    />
    {#if errors.name}<p id={`${prefix}-name-error`} class="field-error">{errors.name}</p>{/if}
  </div>

  <div class="field">
    <div class="label-row">
      <label for={`${prefix}-amount`} class="field-label">금액 (만원)</label>
      {#if help}<HelpTip id="entry-amount" />{/if}
    </div>
    <div class="amount-row">
      <input
        bind:this={amountEl}
        bind:value={amountText}
        id={`${prefix}-amount`}
        class="amount-input"
        type="text"
        inputmode="decimal"
        autocomplete="off"
        placeholder="예: 5 또는 5.5"
        aria-invalid={errors.amount ? "true" : undefined}
        aria-describedby={[describedBy("amount"), `${prefix}-amount-preview`].filter(Boolean).join(" ")}
        {disabled}
        onkeydown={onSubmitKeydown}
        data-testid={`${prefix}-amount-input`}
      />
      <span class="unit" aria-hidden="true">만원</span>
      <output id={`${prefix}-amount-preview`} class="amount-preview" aria-live="polite" data-testid={`${prefix}-amount-preview`}>
        {preview}
      </output>
    </div>
    <div class="quick-amounts" role="group" aria-label="빠른 금액">
      {#each QUICK_AMOUNTS_MANWON as m (m)}
        <button
          type="button"
          class="btn quick"
          class:selected={amountText.trim() === String(m)}
          {disabled}
          onclick={() => setQuickAmount(m)}
          data-testid={`${prefix}-quick-amount-${m}-button`}
        >
          {m}만
        </button>
      {/each}
    </div>
    {#if errors.amount}<p id={`${prefix}-amount-error`} class="field-error">{errors.amount}</p>{/if}
  </div>

  <div class="field">
    <div class="label-row">
      <label for={`${prefix}-tickets`} class="field-label">식권 수</label>
      {#if help}<HelpTip id="entry-tickets" />{/if}
    </div>
    <div class="stepper">
      <button
        type="button"
        class="btn step"
        aria-label="식권 1장 빼기"
        disabled={disabled || (tickets ?? 0) <= TICKETS_MIN}
        onclick={() => stepTickets(-1)}
        data-testid={`${prefix}-tickets-minus-button`}
      >
        −
      </button>
      <input
        bind:this={ticketsEl}
        bind:value={tickets}
        id={`${prefix}-tickets`}
        class="tickets-input"
        type="number"
        min={TICKETS_MIN}
        max={TICKETS_MAX}
        step="1"
        inputmode="numeric"
        aria-invalid={errors.tickets ? "true" : undefined}
        aria-describedby={describedBy("tickets")}
        {disabled}
        onkeydown={onTicketsKeydown}
        onblur={onTicketsBlur}
        data-testid={`${prefix}-tickets-input`}
      />
      <button
        type="button"
        class="btn step"
        aria-label="식권 1장 더하기"
        disabled={disabled || (tickets ?? 0) >= TICKETS_MAX}
        onclick={() => stepTickets(1)}
        data-testid={`${prefix}-tickets-plus-button`}
      >
        +
      </button>
    </div>
    {#if errors.tickets}<p id={`${prefix}-tickets-error`} class="field-error">{errors.tickets}</p>{/if}
  </div>

  <div class="field memo-field">
    <div class="label-row">
      <label for={`${prefix}-memo`} class="field-label">메모 (선택)</label>
      {#if help}<HelpTip id="entry-memo" />{/if}
    </div>
    <input
      bind:this={memoEl}
      bind:value={memo}
      id={`${prefix}-memo`}
      type="text"
      autocomplete="off"
      maxlength={MEMO_MAX + 10}
      aria-invalid={errors.memo ? "true" : undefined}
      aria-describedby={describedBy("memo")}
      {disabled}
      onkeydown={onSubmitKeydown}
      data-testid={`${prefix}-memo-input`}
    />
    {#if errors.memo}<p id={`${prefix}-memo-error`} class="field-error">{errors.memo}</p>{/if}
  </div>
</div>
