<script lang="ts">
  // 항목 수정 (FR-4 / BR-5). 큰 금액 확인은 금액을 바꿨을 때 적용 (BR-3.4)
  import Modal from "./Modal.svelte";
  import EntryFields from "./EntryFields.svelte";
  import { toAppError, updateEntry } from "../lib/api";
  import { toManwonText } from "../lib/amount";
  import { confirmLargeAmount, editTarget } from "../lib/dialogs";
  import { buildEntryInput, firstErrorField } from "../lib/entry";
  import { formatShortTime } from "../lib/format";
  import { notifyDataChanged, showError, showToast } from "../lib/stores";
  import type { FieldErrors, Side } from "../lib/types";

  let name = $state("");
  let amountText = $state("");
  let side = $state<Side>("groom");
  let tickets = $state<number | null>(1);
  let memo = $state("");
  let errors = $state<FieldErrors>({});
  let saving = $state(false);
  let fields: ReturnType<typeof EntryFields> | undefined = $state();

  const target = $derived($editTarget);

  $effect(() => {
    const t = target;
    if (!t) return;
    name = t.name;
    amountText = toManwonText(t.amountWon);
    side = t.side;
    tickets = t.tickets;
    memo = t.memo;
    errors = {};
    const timer = setTimeout(() => fields?.focusField("name"), 0);
    return () => clearTimeout(timer);
  });

  function close() {
    editTarget.set(null);
  }

  async function save() {
    const t = $editTarget;
    if (!t || saving) return;
    const { input, errors: localErrors } = buildEntryInput({ name, amountText, side, tickets, memo });
    errors = localErrors;
    if (!input) {
      const f = firstErrorField(localErrors);
      if (f) fields?.focusField(f);
      return;
    }
    saving = true;
    try {
      if (input.amountWon !== t.amountWon && !(await confirmLargeAmount(input.amountWon))) {
        setTimeout(() => fields?.focusField("amount"), 0);
        return;
      }
      await updateEntry(t.id, input);
      close();
      notifyDataChanged();
      showToast("ok", `${input.name} 항목을 수정했습니다`);
    } catch (e) {
      const err = toAppError(e);
      if (err.code === "validation" && err.fieldErrors) {
        errors = err.fieldErrors;
        const f = firstErrorField(err.fieldErrors);
        if (f) fields?.focusField(f);
      } else {
        showError(err, "수정하지 못했습니다");
      }
    } finally {
      saving = false;
    }
  }
</script>

<Modal open={target !== null} title="항목 수정" onclose={close} testid="entry-edit-dialog" wide>
  {#if target}
    <EntryFields
      bind:this={fields}
      bind:name
      bind:amountText
      bind:side
      bind:tickets
      bind:memo
      {errors}
      prefix="entry-edit"
      disabled={saving}
      onsubmit={save}
    />
    <p class="meta">접수 시각 {formatShortTime(target.receivedAt)} · 입력 PC {target.sourcePc}</p>
  {/if}
  {#snippet actions()}
    <button type="button" class="btn" onclick={close} data-testid="entry-edit-dialog-cancel-button">취소</button>
    <button type="button" class="btn primary" disabled={saving} onclick={save} data-testid="entry-edit-dialog-save-button">
      {saving ? "저장 중…" : "저장"}
    </button>
  {/snippet}
</Modal>
