<script lang="ts">
  // 접수 화면: 입력 폼 + 최근 10건. 합계는 표시하지 않는다 (FR-3.2)
  import EntryForm from "./EntryForm.svelte";
  import RecentList from "./RecentList.svelte";

  let { active }: { active: boolean } = $props();
  let form: ReturnType<typeof EntryForm> | undefined = $state();

  // 접수 탭으로 돌아오면 이름 칸에 포커스
  $effect(() => {
    if (active) setTimeout(() => form?.focus(), 0);
  });
</script>

<div class="reception">
  <EntryForm bind:this={form} />
  <RecentList />
</div>
