<script lang="ts">
  import type { Messages } from "../i18n/messages";

  interface Props {
    messages: Messages;
    versionLabel: string;
    settingsOpen: boolean;
    ontogglesettings: () => void;
  }

  let { messages, versionLabel, settingsOpen, ontogglesettings }: Props = $props();
</script>

<header class="header">
  <div class="identity">
    <h1 class="title">Plimsoll</h1>
    {#if versionLabel !== ""}
      <span class="version">{versionLabel}</span>
    {/if}
  </div>
  <button class="toggle" type="button" aria-expanded={settingsOpen} onclick={ontogglesettings}>
    {settingsOpen ? messages.app.back : messages.app.settings}
  </button>
</header>

<style>
  .identity {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }

  .toggle {
    padding: 0.125rem var(--space-2);
    font: inherit;
    font-size: 0.875rem;
    color: var(--color-text);
    background: transparent;
    border: 0.0625rem solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    cursor: pointer;
  }

  .header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2);
    padding-bottom: var(--space-2);
    border-bottom: 0.0625rem solid var(--color-border);
  }

  .title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .version {
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }
</style>
