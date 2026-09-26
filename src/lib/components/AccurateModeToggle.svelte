<script lang="ts">
  import type { Messages } from "../i18n/messages";

  interface Props {
    messages: Messages;
    enabled: boolean;
    onchange: (enabled: boolean) => Promise<boolean>;
  }

  let { messages, enabled, onchange }: Props = $props();

  let busy = $state(false);

  async function handleChange(event: Event & { currentTarget: HTMLInputElement }) {
    const input = event.currentTarget;
    busy = true;
    const changed = await onchange(input.checked);
    if (!changed) {
      input.checked = enabled;
    }
    busy = false;
  }
</script>

<div class="toggle">
  <div class="control">
    <input
      id="accurate-mode"
      type="checkbox"
      checked={enabled}
      disabled={busy}
      aria-describedby="accurate-mode-help"
      onchange={handleChange}
    />
    <label for="accurate-mode">{messages.accurate.label}</label>
  </div>
  <p id="accurate-mode-help" class="help">
    {messages.accurate.help}
  </p>
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
</style>
