<script lang="ts">
  // 접수 입력 폼 (FR-1, FR-2 / BR-1 ~ BR-4)
  import EntryFields from "./EntryFields.svelte";
  import { addEntry, setDefaultSide, toAppError } from "../lib/api";
  import { formatWon } from "../lib/amount";
  import { confirmLargeAmount } from "../lib/dialogs";
  import { buildEntryInput, firstErrorField } from "../lib/entry";
  import { appState, notifyDataChanged, showError, showToast } from "../lib/stores";
  import type { FieldErrors, Side } from "../lib/types";

  let name = $state("");
  let amountText = $state("");
  let side = $state<Side>("groom");
  let tickets = $state<number | null>(1);
  let memo = $state("");
  let errors = $state<FieldErrors>({});
  let saving = $state(false);
  let fields: ReturnType<typeof EntryFields> | undefined = $state();

  const ready = $derived(!!$appState && !$appState.needsSetup && !$appState.loadError);

  // 설정이 준비되면 저장된 기본 구분으로 시작 (FR-2.2)
  let sideInitialized = false;
  $effect(() => {
    const s = $appState;
    if (s && !s.needsSetup && !sideInitialized) {
      side = s.defaultSide;
      sideInitialized = true;
      setTimeout(() => fields?.focusField("name"), 0);
    }
  });

  export function focus(): void {
    fields?.focusField("name");
  }

  function reset() {
    name = "";
    amountText = "";
    tickets = 1;
    memo = "";
    errors = {};
  }

  async function changeSide(s: Side) {
    try {
      await setDefaultSide(s);
      appState.update((v) => (v ? { ...v, defaultSide: s } : v));
    } catch (e) {
      showError(e, "기본 구분을 저장하지 못했습니다");
    }
  }

  async function submit() {
    if (saving || !ready) return;
    const { input, errors: localErrors } = buildEntryInput({ name, amountText, side, tickets, memo });
    errors = localErrors;
    if (!input) {
      const f = firstErrorField(localErrors);
      if (f) fields?.focusField(f);
      return;
    }

    saving = true;
    try {
      // 큰 금액 확인 (BR-3)
      if (!(await confirmLargeAmount(input.amountWon))) {
        setTimeout(() => fields?.focusField("amount"), 0);
        return;
      }
      const saved = await addEntry(input);
      reset();
      notifyDataChanged();
      showToast("ok", `${saved.name} ${formatWon(saved.amountWon)} 저장했습니다`);
      setTimeout(() => fields?.focusField("name"), 0);
    } catch (e) {
      const err = toAppError(e);
      if (err.code === "validation" && err.fieldErrors) {
        errors = err.fieldErrors;
        const f = firstErrorField(err.fieldErrors);
        if (f) fields?.focusField(f);
      } else {
        // 저장 실패 시 입력값은 그대로 둔다 (BR-4.5)
        showError(err, "저장하지 못했습니다");
      }
    } finally {
      saving = false;
    }
  }
</script>

<form
  class="entry-form card"
  aria-label="축의금 접수"
  onsubmit={(e) => {
    e.preventDefault();
    submit();
  }}
  data-testid="entry-form"
>
  <EntryFields
    bind:this={fields}
    bind:name
    bind:amountText
    bind:side
    bind:tickets
    bind:memo
    {errors}
    prefix="entry-form"
    large
    help
    disabled={!ready}
    onsubmit={submit}
    onsidechange={changeSide}
  />
  <div class="form-actions">
    <button type="submit" class="btn primary save-button" disabled={saving || !ready} data-testid="entry-form-save-button">
      {saving ? "저장 중…" : "저장 (Enter)"}
    </button>
  </div>
</form>
