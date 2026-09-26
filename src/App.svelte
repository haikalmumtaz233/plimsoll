<script lang="ts">
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import AccurateModeToggle from "./lib/components/AccurateModeToggle.svelte";
  import AppHeader from "./lib/components/AppHeader.svelte";
  import LimitCard from "./lib/components/LimitCard.svelte";
  import TokenCard from "./lib/components/TokenCard.svelte";
  import { hidePopup, readAppVersion } from "./lib/api/app";
  import { loadUsage, onUsageUpdated, setAccurateMode, type UsageView } from "./lib/api/usage";
  import { formatVersion } from "./lib/api/version";
  import { statusMessage } from "./lib/usage/format";

  let version = $state<string | undefined>(undefined);
  let view = $state<UsageView | undefined>(undefined);
  let loadFailed = $state(false);
  let toggleError = $state("");

  function accept(next: UsageView) {
    if (view === undefined || next.generatedAt >= view.generatedAt) {
      view = next;
    }
  }

  $effect(() => {
    readAppVersion()
      .then((value) => {
        version = value;
      })
      .catch(() => {
        version = undefined;
      });
  });

  $effect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;
    onUsageUpdated(accept)
      .then((stop) => {
        if (disposed) {
          stop();
        } else {
          unlisten = stop;
        }
      })
      .catch(() => {
        loadFailed = true;
      });
    loadUsage()
      .then(accept)
      .catch(() => {
        loadFailed = true;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  });

  async function changeAccurateMode(enabled: boolean): Promise<boolean> {
    toggleError = "";
    try {
      accept(await setAccurateMode(enabled));
      return true;
    } catch {
      toggleError = "Could not change accurate mode. Try again.";
      return false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      void hidePopup();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="popup">
  <AppHeader versionLabel={version === undefined ? "" : formatVersion(version)} />
  {#if view === undefined}
    <p class="status" role="status">
      {loadFailed ? "Usage is unavailable right now." : "Loading usage…"}
    </p>
  {:else}
    <p class="status" role="status">
      {statusMessage(view.accurateMode, view.status, view.limits.length > 0)}
    </p>
    {#if view.limits.length > 0}
      {#each view.limits as limit (limit.kind)}
        <LimitCard {limit} now={view.generatedAt} />
      {/each}
    {:else}
      <TokenCard
        id="five-hour"
        title="5-hour window"
        usage={view.fiveHour}
        now={view.generatedAt}
        emptyText="No usage in the last 5 hours."
      />
      <TokenCard
        id="weekly"
        title="This week"
        usage={view.weekly}
        now={view.generatedAt}
        emptyText="No usage this week yet."
      />
    {/if}
    <AccurateModeToggle enabled={view.accurateMode} onchange={changeAccurateMode} />
    {#if toggleError !== ""}
      <p class="error" role="alert">{toggleError}</p>
    {/if}
  {/if}
</main>

<style>
  .popup {
    display: grid;
    gap: var(--space-4);
    padding: var(--space-4);
  }

  .status {
    margin: 0;
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }

  .error {
    margin: 0;
    font-size: 0.875rem;
    color: var(--color-danger);
  }
</style>
