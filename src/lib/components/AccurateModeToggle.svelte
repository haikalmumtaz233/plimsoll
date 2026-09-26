<script lang="ts">
  import type { Messages } from "../i18n/messages";

  interface Props {
    messages: Messages;
    enabled: boolean;
    pollMinutes: number;
    onchange: (enabled: boolean) => Promise<boolean>;
  }

  let { messages, enabled, pollMinutes, onchange }: Props = $props();

  let busy = $state(false);
  let confirming = $state(false);
  let checkbox = $state<HTMLInputElement | undefined>(undefined);
  let heading = $state<HTMLHeadingElement | undefined>(undefined);

  $effect(() => {
    if (confirming) {
      heading?.focus();
    }
  });

  async function apply(next: boolean) {
    busy = true;
    const changed = await onchange(next);
    if (!changed && checkbox !== undefined) {
      checkbox.checked = enabled;
    }
    busy = false;
  }

  async function handleChange(event: Event & { currentTarget: HTMLInputElement }) {
    const input = event.currentTarget;
    if (input.checked && !enabled) {
      input.checked = false;
      confirming = true;
      return;
    }
    await apply(input.checked);
  }

  async function confirm() {
    confirming = false;
    await apply(true);
    checkbox?.focus();
  }

  function cancel() {
    confirming = false;
    checkbox?.focus();
  }

  function handleConfirmKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.stopPropagation();
      cancel();
    }
  }
</script>

<div class="toggle">
  <div class="control">
    <input
      id="accurate-mode"
      type="checkbox"
      checked={enabled}
      disabled={busy || confirming}
      aria-describedby="accurate-mode-help"
      bind:this={checkbox}
      onchange={handleChange}
    />
    <label for="accurate-mode">{messages.accurate.label}</label>
  </div>
  <p id="accurate-mode-help" class="help">
    {messages.accurate.help}
  </p>
  {#if confirming}
    <div
      class="confirm"
      role="alertdialog"
      aria-labelledby="accurate-confirm-title"
      aria-describedby="accurate-confirm-risks"
      tabindex="-1"
      onkeydown={handleConfirmKeydown}
    >
      <h3 class="confirm-title" id="accurate-confirm-title" tabindex="-1" bind:this={heading}>
        {messages.accurate.confirmTitle}
      </h3>
      <ul class="risks" id="accurate-confirm-risks">
        <li>{messages.accurate.risks.token}</li>
        <li>{messages.accurate.risks.storage}</li>
        <li>{messages.accurate.risks.endpoint}</li>
        <li>{messages.accurate.risks.unofficial}</li>
        <li>{messages.accurate.risks.requests(pollMinutes)}</li>
      </ul>
      <div class="actions">
        <button type="button" class="primary" onclick={confirm}>
          {messages.accurate.confirm}
        </button>
        <button type="button" class="secondary" onclick={cancel}>
          {messages.accurate.cancel}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .toggle {
    display: grid;
    gap: var(--space-1);
    padding-top: var(--space-2);
    border-top: 0.0625rem solid var(--color-border);
  }

  .control {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-weight: 600;
  }

  input {
    width: 1.125rem;
    height: 1.125rem;
    margin: 0;
    accent-color: var(--color-focus);
  }

  .help {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  .confirm {
    display: grid;
    gap: var(--space-2);
    margin-top: var(--space-2);
    padding: var(--space-2);
    border: 0.0625rem solid var(--color-border-strong);
    border-radius: var(--radius-md);
  }

  .confirm-title {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
  }

  .confirm-title:focus {
    outline: none;
  }

  .risks {
    display: grid;
    gap: var(--space-1);
    margin: 0;
    padding-left: 1.125rem;
    font-size: 0.8125rem;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  button {
    min-height: 1.5rem;
    padding: 0.25rem var(--space-4);
    font: inherit;
    font-size: 0.875rem;
    border-radius: var(--radius-pill);
    cursor: pointer;
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

  @media (forced-colors: active) {
    .primary {
      forced-color-adjust: none;
      color: ButtonText;
      background: ButtonFace;
      border: 0.0625rem solid ButtonText;
    }
  }
</style>
