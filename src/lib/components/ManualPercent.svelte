<script lang="ts">
  import type { LimitKind, ManualView } from "../api/usage";
  import type { Messages } from "../i18n/messages";
  import { manualPercentError } from "../settings/preferences";
  import { formatCountdown, formatPercent, limitTitle } from "../usage/format";

  interface Props {
    messages: Messages;
    manual: readonly ManualView[];
    now: number;
    onsave: (kind: LimitKind, percent: number | null) => Promise<boolean>;
    inactive: boolean;
  }

  let { messages, manual, now, onsave, inactive }: Props = $props();

  const KINDS: readonly LimitKind[] = ["five_hour", "seven_day"];

  let drafts = $state<Record<LimitKind, number | null>>({ five_hour: null, seven_day: null });
  let errors = $state<Record<LimitKind, string>>({ five_hour: "", seven_day: "" });
  let busy = $state(false);

  function current(kind: LimitKind): ManualView | undefined {
    return manual.find((reading) => reading.kind === kind);
  }

  async function submit(kind: LimitKind) {
    const value = drafts[kind];
    const problem = value === null ? messages.manual.invalid : manualPercentError(value, messages);
    if (problem !== null || value === null) {
      errors[kind] = problem ?? messages.manual.invalid;
      return;
    }
    await store(kind, value);
  }

  async function store(kind: LimitKind, percent: number | null) {
    errors[kind] = "";
    busy = true;
    const ok = await onsave(kind, percent);
    busy = false;
    if (ok) {
      drafts[kind] = null;
    } else {
      errors[kind] = messages.manual.failed;
    }
  }
</script>

<section class="manual" class:inactive aria-labelledby="manual-title">
  <div class="heading">
    <h3 class="title" id="manual-title">{messages.manual.title}</h3>
    {#if inactive}
      <span class="badge" title={messages.manual.inactive}>
        <svg class="lock" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
          <rect x="3.5" y="7" width="9" height="6.5" rx="1.5" />
          <path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2" />
        </svg>
        <span class="visually-hidden">{messages.manual.inactive}</span>
      </span>
    {/if}
  </div>
  <fieldset class="rows" disabled={inactive} aria-labelledby="manual-title">
    {#each KINDS as kind (kind)}
      {@const reading = current(kind)}
      <div class="row">
        <label class="label" for={`manual-${kind}`}>{limitTitle(kind, messages)}</label>
        <span class="field">
          <input
            id={`manual-${kind}`}
            type="number"
            min="0"
            max="100"
            step="0.1"
            inputmode="decimal"
            aria-describedby={`manual-${kind}-status`}
            bind:value={drafts[kind]}
          />
          <span aria-hidden="true">%</span>
        </span>
        <button type="button" class="primary" disabled={busy} onclick={() => submit(kind)}>
          {messages.manual.save}
        </button>
        {#if reading !== undefined}
          <button type="button" class="secondary" disabled={busy} onclick={() => store(kind, null)}>
            {messages.manual.clear}
          </button>
        {/if}
        <p class="status" id={`manual-${kind}-status`}>
          {reading === undefined
            ? ""
            : messages.manual.entered(
                formatPercent(reading.percent),
                formatCountdown(now - reading.enteredAt, messages),
              )}
        </p>
        {#if errors[kind] !== ""}
          <p class="error" role="alert">{errors[kind]}</p>
        {/if}
      </div>
    {/each}
  </fieldset>
</section>

<style>
  .manual {
    display: grid;
    gap: var(--space-2);
    padding-top: var(--space-2);
    border-top: 0.0625rem solid var(--color-border);
  }

  .heading {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .title {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
  }

  .badge {
    display: inline-grid;
    place-items: center;
    width: 1.5rem;
    height: 1.5rem;
    color: var(--color-text-muted);
    border: 0.0625rem solid var(--color-border-strong);
    border-radius: var(--radius-pill);
  }

  .lock {
    width: 0.875rem;
    height: 0.875rem;
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

  .rows {
    display: grid;
    gap: var(--space-2);
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }

  .inactive .rows {
    opacity: 0.45;
  }

  .inactive .title {
    color: var(--color-text-muted);
  }

  .status {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  .status:empty {
    display: none;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-2);
    font-size: 0.875rem;
  }

  .label {
    flex: 1 1 0;
    min-width: 5rem;
  }

  .field {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .status,
  .error {
    flex-basis: 100%;
  }

  input {
    width: 4rem;
    padding: 0.125rem var(--space-2);
    font: inherit;
    font-size: 0.875rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
    color: var(--color-text);
    background: var(--color-surface);
    border: 0.0625rem solid var(--color-border-strong);
    border-radius: var(--radius-md);
  }

  button {
    min-height: 1.5rem;
    padding: 0.125rem var(--space-2);
    font: inherit;
    font-size: 0.8125rem;
    border-radius: var(--radius-pill);
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .primary {
    font-weight: 600;
    color: var(--color-on-selected);
    background: var(--color-selected);
    border: 0;
  }

  .secondary {
    color: var(--color-text);
    background: transparent;
    border: 0.0625rem solid var(--color-border-strong);
  }

  .error {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--color-danger);
  }

  @media (forced-colors: active) {
    .primary {
      forced-color-adjust: none;
      color: ButtonText;
      background: ButtonFace;
      border: 0.0625rem solid ButtonText;
    }
  }
</style>
