<script lang="ts">
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import AppHeader from "./lib/components/AppHeader.svelte";
  import CreditsCard from "./lib/components/CreditsCard.svelte";
  import LimitCard from "./lib/components/LimitCard.svelte";
  import SettingsPanel from "./lib/components/SettingsPanel.svelte";
  import TokenCard from "./lib/components/TokenCard.svelte";
  import UsageBreakdown from "./lib/components/UsageBreakdown.svelte";
  import UsageChart from "./lib/components/UsageChart.svelte";
  import { hidePopup, openUsagePage, readAppVersion, startPopupDrag } from "./lib/api/app";
  import {
    loadUsage,
    onUsageAlert,
    onUsageUpdated,
    openLogin,
    refreshNow,
    saveManualPercent,
    savePreferences,
    setAccurateMode,
    setAutostart,
    setCliFallback,
    type LimitKind,
    type PreferencesInput,
    type UsageView,
  } from "./lib/api/usage";
  import { formatVersion } from "./lib/api/version";
  import {
    alertText,
    estimateFor,
    isSyncing,
    statusMessage,
    statusTone,
    updatedText,
  } from "./lib/usage/format";
  import { modelLimitTitle } from "./lib/usage/extras";
  import { type HistoryRange } from "./lib/usage/history";
  import { effectiveRefresh, refreshLabel } from "./lib/usage/refresh";
  import { localeFromTag, messagesFor, type Locale } from "./lib/i18n/messages";

  let version = $state<string | undefined>(undefined);
  let view = $state<UsageView | undefined>(undefined);
  let loadFailed = $state(false);
  let toggleError = $state("");
  let range = $state<HistoryRange>("day");
  let settingsOpen = $state(false);
  let announcement = $state("");
  let clock = $state(Date.now());
  let loginError = $state("");

  const refreshState = $derived(
    view === undefined ? "blocked" : effectiveRefresh(view.refresh, clock),
  );

  $effect(() => {
    const readyAt = view?.refresh.readyAt ?? null;
    if (readyAt === null) {
      return;
    }
    const timer = setTimeout(
      () => {
        clock = Date.now();
      },
      Math.max(0, readyAt - Date.now()),
    );
    return () => {
      clearTimeout(timer);
    };
  });

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

  async function refresh() {
    if (refreshState !== "ready") {
      return;
    }
    try {
      accept(await refreshNow());
    } catch {
      clock = Date.now();
    }
  }

  async function login() {
    loginError = "";
    try {
      accept(await openLogin());
    } catch {
      loginError = messages.login.failed;
    }
  }

  async function changeCliFallback(enabled: boolean): Promise<boolean> {
    try {
      accept(await setCliFallback(enabled));
      return true;
    } catch {
      return false;
    }
  }

  async function changeAutostart(enabled: boolean): Promise<boolean> {
    try {
      accept(await setAutostart(enabled));
      return true;
    } catch {
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
    planLabel={view?.plan ?? null}
    {settingsOpen}
    ontogglesettings={() => {
      settingsOpen = !settingsOpen;
    }}
    onwindowdrag={() => {
      void startPopupDrag();
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
      autostart={view.autostart}
      onautostartchange={changeAutostart}
      officialActive={view.limits.length > 0}
      cliFallback={view.cliFallback}
      onclifallbackchange={changeCliFallback}
    />
    {#if toggleError !== ""}
      <p class="error" role="alert">{toggleError}</p>
    {/if}
  {:else}
    {@const showingLimits = view.limits.length > 0}
    {@const syncing = isSyncing(view.accurateMode, view.status, showingLimits)}
    {@const updated = showingLimits
      ? updatedText(view.officialUpdatedAt, view.generatedAt, messages)
      : null}
    <div class="status-row">
      <p
        class="status {statusTone(view.accurateMode, view.status, showingLimits)}"
        role="status"
        title={syncing ? messages.status.syncing : undefined}
      >
        <span class="dot" aria-hidden="true"></span>
        {statusMessage(view.accurateMode, view.status, showingLimits, messages)}
        {#if updated !== null}
          <span class="age">· {updated}</span>
        {/if}
        {#if syncing}
          <svg class="sync" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
            <path d="M13 8a5 5 0 0 1-8.6 3.5M3 8a5 5 0 0 1 8.6-3.5" />
            <path d="M11.5 1.5v3h-3M4.5 14.5v-3h3" />
          </svg>
          <span class="visually-hidden">{messages.status.syncing}</span>
        {/if}
      </p>
      {#if view.accurateMode}
        {@const label = refreshLabel(refreshState, messages)}
        <button
          type="button"
          class="refresh"
          aria-label={label}
          title={label}
          aria-disabled={refreshState !== "ready"}
          aria-busy={refreshState === "running"}
          onclick={refresh}
        >
          <svg
            class="sync refresh-icon"
            class:spinning={refreshState === "running"}
            viewBox="0 0 16 16"
            aria-hidden="true"
            focusable="false"
          >
            <path d="M13 8a5 5 0 0 1-8.6 3.5M3 8a5 5 0 0 1 8.6-3.5" />
            <path d="M11.5 1.5v3h-3M4.5 14.5v-3h3" />
          </svg>
        </button>
      {/if}
    </div>
    {#if view.login !== "hidden"}
      <div class="login">
        {#if view.login === "running"}
          <p class="login-note" role="status">{messages.login.running}</p>
        {:else if view.login === "renewing"}
          <p class="login-note" role="status">{messages.login.renewing}</p>
        {:else}
          <button type="button" class="login-button" onclick={login}>
            {messages.login.ready}
          </button>
        {/if}
        {#if loginError !== ""}
          <p class="error" role="alert">{loginError}</p>
        {/if}
      </div>
    {/if}
    {#if view.limits.length > 0}
      {#each view.limits as limit (limit.kind)}
        <LimitCard
          {messages}
          {limit}
          thresholds={view.preferences.thresholds}
          now={view.generatedAt}
        />
      {/each}
      {#each view.models as model (model.model)}
        <LimitCard
          {messages}
          limit={{ kind: "seven_day", percent: model.percent, resetsAt: model.resetsAt }}
          thresholds={view.preferences.thresholds}
          now={view.generatedAt}
          title={modelLimitTitle(model.model, messages)}
          key={`model-${model.model}`}
        />
      {/each}
      {#if view.credits !== null}
        <CreditsCard {messages} credits={view.credits} />
      {/if}
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
    <button
      type="button"
      class="usage-link"
      onclick={() => {
        void openUsagePage().catch(() => undefined);
      }}
    >
      {messages.usagePage}
      <svg class="external" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
        <path d="M9 3h4v4M13 3 7 9M11 9.5V13H3V5h3.5" />
      </svg>
    </button>
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

  .status.official,
  .status.local,
  .status.attention {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    width: fit-content;
    padding: 0.125rem var(--space-2);
    font-size: 0.8125rem;
    border: 0.0625rem solid var(--color-border);
    border-radius: var(--radius-pill);
  }

  .status-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .refresh {
    display: inline-grid;
    place-items: center;
    flex-shrink: 0;
    width: 1.75rem;
    height: 1.75rem;
    padding: 0;
    color: var(--color-text);
    background: transparent;
    border: 0.0625rem solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    cursor: pointer;
  }

  .refresh[aria-disabled="true"] {
    color: var(--color-text-muted);
    cursor: default;
  }

  .usage-link {
    display: inline-flex;
    align-items: center;
    justify-self: start;
    gap: var(--space-1);
    min-height: 1.5rem;
    padding: 0;
    font: inherit;
    font-size: 0.8125rem;
    color: var(--color-text);
    text-decoration: underline;
    background: transparent;
    border: 0;
    cursor: pointer;
  }

  .external {
    width: 0.75rem;
    height: 0.75rem;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .login {
    display: grid;
    gap: var(--space-1);
  }

  .login-button {
    justify-self: start;
    min-height: 1.75rem;
    padding: 0 var(--space-4);
    font: inherit;
    font-size: 0.875rem;
    color: var(--color-text);
    background: transparent;
    border: 0.0625rem solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    cursor: pointer;
  }

  .login-note {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  .refresh-icon {
    width: 1rem;
    height: 1rem;
  }

  @media (prefers-reduced-motion: no-preference) {
    .spinning {
      animation: spin 1s linear infinite;
    }
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .dot {
    flex-shrink: 0;
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background: var(--color-text-muted);
  }

  .age {
    font-variant-numeric: tabular-nums;
  }

  .sync {
    width: 0.75rem;
    height: 0.75rem;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  .official .dot {
    background: var(--color-ok);
  }

  .attention .dot {
    background: var(--color-warn);
  }

  @media (forced-colors: active) {
    .dot {
      forced-color-adjust: none;
      background: CanvasText;
    }
  }

  .error {
    margin: 0;
    font-size: 0.875rem;
    color: var(--color-danger);
  }
</style>
