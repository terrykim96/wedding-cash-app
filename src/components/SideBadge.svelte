<script lang="ts">
  import type { Side } from "../lib/types";
  import { SIDE_LABEL } from "../lib/types";

  let { side }: { side: Side } = $props();
</script>

<!-- 색상 + 텍스트로 함께 구분 (NFR-4.5) -->
<span class={`side-badge ${side}`}>{SIDE_LABEL[side]}</span>
