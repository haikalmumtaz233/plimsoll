<script lang="ts">
  import type { Messages } from "../i18n/messages";

  interface Props {
    messages: Messages;
    versionLabel: string;
    settingsOpen: boolean;
    ontogglesettings: () => void;
  }

  let { messages, versionLabel, settingsOpen, ontogglesettings }: Props = $props();

  const toggleLabel = $derived(settingsOpen ? messages.app.back : messages.app.settings);
</script>

<header class="header">
  <div class="identity">
    <h1 class="title">Plimsoll</h1>
    {#if versionLabel !== ""}
      <span class="version">{versionLabel}</span>
    {/if}
  </div>
  <button
    class="toggle"
    type="button"
    aria-expanded={settingsOpen}
    aria-label={toggleLabel}
    title={toggleLabel}
    onclick={ontogglesettings}
  >
    <svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
      {#if settingsOpen}
        <path d="M10 3 5 8l5 5" />
      {:else}
        <path d="M2 4.5h6.5M13.5 4.5H14M2 11.5h1M7.5 11.5H14" />
        <circle cx="11" cy="4.5" r="1.75" />
        <circle cx="5" cy="11.5" r="1.75" />
      {/if}
    </svg>
  </button>
</header>

<style>
  .identity {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }

  .toggle {
    display: inline-grid;
    place-items: center;
    width: 1.75rem;
    height: 1.75rem;
    padding: 0;
    color: var(--color-text);
    background: transparent;
    border: 0.0625rem solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    cursor: pointer;
  }

  .icon {
    width: 1rem;
    height: 1rem;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
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
