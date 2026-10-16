<script lang="ts">
  import { toast } from "../lib/stores";

  const t = $derived($toast);
</script>

<!-- 화면 낭독기에도 알림이 전달되도록 항상 존재하는 live region -->
<div class="toast-region" aria-live="polite" aria-atomic="true">
  {#if t}
    {#key t.id}
      <div class={`toast ${t.kind}`} role={t.kind === "error" ? "alert" : "status"} data-testid="toast">
        <span>{t.text}</span>
        <button type="button" class="toast-close" aria-label="알림 닫기" onclick={() => toast.set(null)} data-testid="toast-close-button">×</button>
      </div>
    {/key}
  {/if}
</div>
