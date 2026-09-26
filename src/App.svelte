<script lang="ts">
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import AppHeader from "./lib/components/AppHeader.svelte";
  import LimitCard from "./lib/components/LimitCard.svelte";
  import SettingsPanel from "./lib/components/SettingsPanel.svelte";
  import TokenCard from "./lib/components/TokenCard.svelte";
  import UsageBreakdown from "./lib/components/UsageBreakdown.svelte";
  import UsageChart from "./lib/components/UsageChart.svelte";
  import { hidePopup, readAppVersion } from "./lib/api/app";
  import {
    loadUsage,
    onUsageAlert,
    onUsageUpdated,
    saveManualPercent,
    savePreferences,
    setAccurateMode,
    type LimitKind,
    type PreferencesInput,
    type UsageView,
  } from "./lib/api/usage";
  import { formatVersion } from "./lib/api/version";
  import { alertText, estimateFor, statusMessage } from "./lib/usage/format";
  import { type HistoryRange } from "./lib/usage/history";
  import { localeFromTag, messagesFor, type Locale } from "./lib/i18n/messages";

  let version = $state<string | undefined>(undefined);
  let view = $state<UsageView | undefined>(undefined);
  let loadFailed = $state(false);
  let toggleError = $state("");
  let range = $state<HistoryRange>("day");
  let settingsOpen = $state(false);
  let announcement = $state("");

  const locale: Locale = $derived(
    view?.preferences.resolvedLanguage ?? localeFromTag(navigator.language),
  );
  const messages = $derived(messagesFor(locale));

  $effect(() => {
    document.documentElement.lang = locale;
  });

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

  $effect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;
    onUsageAlert((alert) => {
      announcement = alertText(alert);
    })
      .then((stop) => {
        if (disposed) {
          stop();
        } else {
          unlisten = stop;
        }
      })
      .catch(() => {
        announcement = "";
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
      toggleError = messages.app.accurateModeFailed;
      return false;
    }
  }

  async function changeManualPercent(kind: LimitKind, percent: number | null): Promise<boolean> {
    try {
      accept(await saveManualPercent(kind, percent));
      return true;
    } catch {
      return false;
    }
  }

  async function changePreferences(preferences: PreferencesInput): Promise<boolean> {
    try {
      accept(await savePreferences(preferences));
      return true;
    } catch {
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
  <AppHeader
    {messages}
    versionLabel={version === undefined ? "" : formatVersion(version)}
    {settingsOpen}
    ontogglesettings={() => {
      settingsOpen = !settingsOpen;
    }}
  />
  <div class="announcement" class:active={announcement !== ""} role="alert">
    {#if announcement !== ""}
      <p class="announcement-text">{announcement}</p>
      <button
        type="button"
        class="dismiss"
        onclick={() => {
          announcement = "";
        }}
      >
        {messages.app.dismiss}
      </button>
    {/if}
  </div>
  {#if view === undefined}
    <p class="status" role="status">
      {loadFailed ? messages.app.unavailable : messages.app.loading}
    </p>
  {:else if settingsOpen}
    <SettingsPanel
      {messages}
      preferences={view.preferences}
      accurateMode={view.accurateMode}
      onsave={changePreferences}
      onaccuratechange={changeAccurateMode}
      manual={view.manual}
      now={view.generatedAt}
      onmanualsave={changeManualPercent}
    />
    {#if toggleError !== ""}
      <p class="error" role="alert">{toggleError}</p>
    {/if}
  {:else}
    <p class="status" role="status">
      {statusMessage(view.accurateMode, view.status, view.limits.length > 0, messages)}
    </p>
    {#if view.limits.length > 0}
      {#each view.limits as limit (limit.kind)}
        <LimitCard
          {messages}
          {limit}
          thresholds={view.preferences.thresholds}
          now={view.generatedAt}
        />
      {/each}
    {:else}
      <TokenCard
        id="five-hour"
        title={messages.windows.fiveHour}
        usage={view.fiveHour}
        now={view.generatedAt}
        emptyText={messages.windows.noUsageFiveHour}
        estimate={estimateFor(view.estimates, "five_hour")}
        {messages}
      />
      <TokenCard
        id="weekly"
        title={messages.windows.week}
        usage={view.weekly}
        now={view.generatedAt}
        emptyText={messages.windows.noUsageWeek}
        estimate={estimateFor(view.estimates, "seven_day")}
        {messages}
      />
    {/if}
    <UsageChart history={view.history} {messages} bind:range />
    <UsageBreakdown breakdown={view.breakdown[range]} {range} {messages} />
  {/if}
</main>

<style>
  .announcement {
    position: absolute;
  }

  .announcement.active {
    position: static;
    display: flex;
    align-items: start;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-2);
    border: 0.0625rem solid var(--color-danger);
    border-radius: var(--radius-md);
  }

  .announcement-text {
    margin: 0;
    font-size: 0.875rem;
  }

  .dismiss {
    flex-shrink: 0;
    min-height: 1.5rem;
    padding: 0 var(--space-2);
    font: inherit;
    font-size: 0.8125rem;
    color: var(--color-text);
    background: transparent;
    border: 0.0625rem solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    cursor: pointer;
  }

  .popup {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
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
