<script lang="ts">
  import type { Messages } from "../i18n/messages";
  import { dragHandle } from "../window/drag";
  import type { Page, Subpage } from "../window/page";

  interface Props {
    messages: Messages;
    versionLabel: string;
    planLabel: string | null;
    page: Page;
    ontogglepage: (target: Subpage) => void;
    onwindowdrag: () => void;
  }

  let { messages, versionLabel, planLabel, page, ontogglepage, onwindowdrag }: Props = $props();

  const historyOpen = $derived(page === "history");
  const settingsOpen = $derived(page === "settings");
  const historyLabel = $derived(historyOpen ? messages.app.back : messages.app.history);
  const settingsLabel = $derived(settingsOpen ? messages.app.back : messages.app.settings);
</script>

<header class="header" {@attach dragHandle(onwindowdrag)}>
  <div class="identity">
    <h1 class="title">Plimsoll</h1>
    {#if versionLabel !== ""}
      <span class="version">{versionLabel}</span>
    {/if}
    {#if planLabel !== null}
      <span class="plan" title={messages.app.plan(planLabel)}>
        <span aria-hidden="true">{planLabel}</span>
        <span class="visually-hidden">{messages.app.plan(planLabel)}</span>
      </span>
    {/if}
  </div>
  <div class="actions">
    <button
      class="toggle"
      type="button"
      aria-expanded={historyOpen}
      aria-label={historyLabel}
      title={historyLabel}
      onclick={() => {
        ontogglepage("history");
      }}
    >
      <svg class="icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
        {#if historyOpen}
          <path d="M10 3 5 8l5 5" />
        {:else}
          <path d="M2.5 13.5h11M4.5 11V8M8 11V3.5M11.5 11V6" />
        {/if}
      </svg>
    </button>
    <button
      class="toggle"
      type="button"
      aria-expanded={settingsOpen}
      aria-label={settingsLabel}
      title={settingsLabel}
      onclick={() => {
        ontogglepage("settings");
      }}
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
  </div>
</header>

<style>
  .identity {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }

  .actions {
    display: flex;
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
    user-select: none;
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

  .plan {
    padding: 0 var(--space-2);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text);
    border: 0.0625rem solid var(--color-border-strong);
    border-radius: var(--radius-pill);
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
