<script lang="ts">
  interface Props {
    versionLabel: string;
  }

  let { versionLabel }: Props = $props();
</script>

<header class="header">
  <h1 class="title">Plimsoll</h1>
  {#if versionLabel !== ""}
    <span class="version">{versionLabel}</span>
  {/if}
</header>

<style>
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
