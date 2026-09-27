<script lang="ts">
  import { untrack } from "svelte";
  import type {
    LanguageChoice,
    LimitKind,
    ManualView,
    PreferencesInput,
    PreferencesView,
  } from "../api/usage";
  import type { Messages } from "../i18n/messages";
  import { intervalLabel, thresholdError } from "../settings/preferences";
  import AccurateModeToggle from "./AccurateModeToggle.svelte";
  import ManualPercent from "./ManualPercent.svelte";

  interface Props {
    messages: Messages;
    preferences: PreferencesView;
    accurateMode: boolean;
    onsave: (preferences: PreferencesInput) => Promise<boolean>;
    onaccuratechange: (enabled: boolean) => Promise<boolean>;
    manual: readonly ManualView[];
    now: number;
    onmanualsave: (kind: LimitKind, percent: number | null) => Promise<boolean>;
    autostart: boolean;
    onautostartchange: (enabled: boolean) => Promise<boolean>;
  }

  let {
    messages,
    preferences,
    accurateMode,
    onsave,
    onaccuratechange,
    manual,
    now,
    onmanualsave,
    autostart,
    onautostartchange,
  }: Props = $props();

  let startupBusy = $state(false);
  let startupError = $state("");

  async function handleAutostart(event: Event & { currentTarget: HTMLInputElement }) {
    const input = event.currentTarget;
    startupBusy = true;
    startupError = "";
    const changed = await onautostartchange(input.checked);
    if (!changed) {
      input.checked = autostart;
      startupError = messages.settings.autostartFailed;
    }
    startupBusy = false;
  }

  const LANGUAGES: readonly LanguageChoice[] = ["system", "en", "id"];

  const initial = untrack(() => preferences);

  let elevated = $state(initial.thresholds.elevated);
  let high = $state(initial.thresholds.high);
  let critical = $state(initial.thresholds.critical);
  let pollMinutes = $state(initial.pollMinutes);
  let language = $state(initial.language);
  let error = $state("");
  let saved = $state(false);
  let busy = $state(false);
  let heading = $state<HTMLHeadingElement | undefined>(undefined);

  $effect(() => {
    heading?.focus();
  });

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    saved = false;
    const problem = thresholdError({ elevated, high, critical }, messages);
    if (problem !== null) {
      error = problem;
      return;
    }
    error = "";
    busy = true;
    const ok = await onsave({ elevated, high, critical, pollMinutes, language });
    busy = false;
    if (ok) {
      saved = true;
    } else {
      error = messages.settings.saveFailed;
    }
  }
</script>

<section class="settings" aria-labelledby="settings-title">
  <h2 class="heading" id="settings-title" tabindex="-1" bind:this={heading}>
    {messages.settings.title}
  </h2>
  <form class="form" onsubmit={handleSubmit} novalidate>
    <fieldset class="levels" aria-describedby="levels-help">
      <legend class="legend">{messages.settings.alertLevels}</legend>
      <p id="levels-help" class="help">{messages.settings.alertHelp}</p>
      <label class="level">
        <span>{messages.settings.warningAt}</span>
        <span class="field">
          <input type="number" min="1" max="100" step="1" required bind:value={elevated} />
          <span aria-hidden="true">%</span>
        </span>
      </label>
      <label class="level">
        <span>{messages.settings.highAt}</span>
        <span class="field">
          <input type="number" min="1" max="100" step="1" required bind:value={high} />
          <span aria-hidden="true">%</span>
        </span>
      </label>
      <label class="level">
        <span>{messages.settings.criticalAt}</span>
        <span class="field">
          <input type="number" min="1" max="100" step="1" required bind:value={critical} />
          <span aria-hidden="true">%</span>
        </span>
      </label>
    </fieldset>
    <div class="interval">
      <label class="legend" for="poll-interval">{messages.settings.refresh}</label>
      <select id="poll-interval" aria-describedby="interval-help" bind:value={pollMinutes}>
        {#each preferences.pollChoices as minutes (minutes)}
          <option value={minutes}>{intervalLabel(minutes, messages)}</option>
        {/each}
      </select>
      <p id="interval-help" class="help">{messages.settings.refreshHelp}</p>
    </div>
    <div class="interval">
      <label class="legend" for="language">{messages.settings.language}</label>
      <select id="language" bind:value={language}>
        {#each LANGUAGES as choice (choice)}
          <option value={choice} lang={choice === "system" ? undefined : choice}>
            {messages.settings.languageNames[choice]}
          </option>
        {/each}
      </select>
    </div>
    {#if error !== ""}
      <p class="error" role="alert">{error}</p>
    {/if}
    <div class="actions">
      <button type="submit" disabled={busy}>{messages.settings.save}</button>
      <p class="saved" role="status">{saved ? messages.settings.saved : ""}</p>
    </div>
  </form>
  <div class="startup">
    <input
      id="autostart"
      type="checkbox"
      checked={autostart}
      disabled={startupBusy}
      onchange={handleAutostart}
    />
    <label for="autostart">{messages.settings.autostart}</label>
  </div>
  {#if startupError !== ""}
    <p class="error" role="alert">{startupError}</p>
  {/if}
  <ManualPercent {messages} {manual} {now} onsave={onmanualsave} />
  <AccurateModeToggle
    {messages}
    enabled={accurateMode}
    pollMinutes={preferences.pollMinutes}
    onchange={onaccuratechange}
  />
</section>

<style>
  .startup {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding-top: var(--space-2);
    border-top: 0.0625rem solid var(--color-border);
    font-size: 0.875rem;
  }

  .startup input {
    width: 1.125rem;
    height: 1.125rem;
    margin: 0;
    accent-color: var(--color-focus);
  }

  .settings {
    display: grid;
    gap: var(--space-4);
  }

  .heading {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .heading:focus {
    outline: none;
  }

  .form {
    display: grid;
    gap: var(--space-4);
  }

  .levels {
    display: grid;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    border: 0;
  }

  .legend {
    padding: 0;
    font-size: 0.875rem;
    font-weight: 600;
  }

  .help {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  .level {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    font-size: 0.875rem;
  }

  .field {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  input,
  select {
    font: inherit;
    font-size: 0.875rem;
    color: var(--color-text);
    background: var(--color-surface);
    border: 0.0625rem solid var(--color-border-strong);
    border-radius: var(--radius-md);
  }

  input {
    width: 4rem;
    padding: 0.125rem var(--space-2);
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .interval {
    display: grid;
    gap: var(--space-1);
  }

  select {
    padding: 0.25rem var(--space-2);
  }

  .error {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--color-danger);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  button {
    padding: 0.25rem var(--space-4);
    font: inherit;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-on-selected);
    background: var(--color-selected);
    border: 0;
    border-radius: var(--radius-pill);
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .saved {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  @media (forced-colors: active) {
    button {
      forced-color-adjust: none;
      color: ButtonText;
      background: ButtonFace;
      border: 0.0625rem solid ButtonText;
    }

    button:disabled {
      color: GrayText;
      border-color: GrayText;
    }
  }
</style>
