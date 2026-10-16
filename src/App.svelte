<script lang="ts">
  import { onMount } from "svelte";
  import { getAppState } from "./lib/api";
  import { appState, openHelp, showError, tutorialOpen } from "./lib/stores";
  import AppHeader from "./components/AppHeader.svelte";
  import SetupDialog from "./components/SetupDialog.svelte";
  import ReceptionView from "./components/ReceptionView.svelte";
  import SettlementView from "./components/SettlementView.svelte";
  import HistoryView from "./components/HistoryView.svelte";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import EntryEditDialog from "./components/EntryEditDialog.svelte";
  import EntryHistoryDialog from "./components/EntryHistoryDialog.svelte";
  import TutorialDialog from "./components/TutorialDialog.svelte";
  import Toast from "./components/Toast.svelte";

  type Tab = "reception" | "settlement" | "history";
  const TABS: { id: Tab; label: string }[] = [
    { id: "reception", label: "접수" },
    { id: "settlement", label: "정산" },
    { id: "history", label: "변경 이력" },
  ];

  let tab = $state<Tab>("reception");
  const tabButtons: Record<string, HTMLButtonElement | undefined> = $state({});

  onMount(async () => {
    try {
      appState.set(await getAppState());
    } catch (e) {
      showError(e, "프로그램 상태를 불러오지 못했습니다");
    }
  });

  // 사용 설명은 실행할 때 자동으로 한 번 열린다. 다만 "끝까지 봤거나 다음부터 보지 않기"를 고른 뒤에는 열지 않는다.
  // 최초 PC 구분 설정과 데이터 파일 오류가 있을 때는 설정이 끝난 뒤에 연다.
  let tutorialAutoShown = false;
  $effect(() => {
    const s = $appState;
    if (!s || tutorialAutoShown) return;
    if (s.needsSetup || s.loadError) return;
    tutorialAutoShown = true;
    if (!s.tutorialSeen) tutorialOpen.set(true);
  });

  // 탭을 바꾸면 열려 있던 ? 도움말은 닫는다
  $effect(() => {
    void tab;
    openHelp.set(null);
  });

  // 좌우 화살표로 탭 이동 (WAI-ARIA tabs 패턴)
  function onTabKey(e: KeyboardEvent, index: number) {
    if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
    e.preventDefault();
    const next = (index + (e.key === "ArrowRight" ? 1 : TABS.length - 1)) % TABS.length;
    tab = TABS[next].id;
    tabButtons[TABS[next].id]?.focus();
  }
</script>

<div class="app">
  <AppHeader />

  <div class="tabs" role="tablist" aria-label="화면 선택">
    {#each TABS as t, i (t.id)}
      <button
        bind:this={tabButtons[t.id]}
        type="button"
        role="tab"
        id={`tab-${t.id}`}
        aria-selected={tab === t.id}
        aria-controls={`panel-${t.id}`}
        tabindex={tab === t.id ? 0 : -1}
        class="tab"
        class:active={tab === t.id}
        onclick={() => (tab = t.id)}
        onkeydown={(e) => onTabKey(e, i)}
        data-testid={`app-tab-${t.id}`}
      >
        {t.label}
      </button>
    {/each}
  </div>

  <main class="panels">
    <div id="panel-reception" role="tabpanel" aria-labelledby="tab-reception" hidden={tab !== "reception"}>
      <ReceptionView active={tab === "reception"} />
    </div>
    <div id="panel-settlement" role="tabpanel" aria-labelledby="tab-settlement" hidden={tab !== "settlement"}>
      <SettlementView />
    </div>
    <div id="panel-history" role="tabpanel" aria-labelledby="tab-history" hidden={tab !== "history"}>
      <HistoryView />
    </div>
  </main>
</div>

<SetupDialog />
<TutorialDialog />
<EntryEditDialog />
<EntryHistoryDialog />
<ConfirmDialog />
<Toast />
