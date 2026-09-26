<script lang="ts">
  import type { TokenView } from "../api/usage";
  import type { Messages } from "../i18n/messages";
  import { formatTokens, resetText } from "../usage/format";

  interface Props {
    messages: Messages;
    id: string;
    title: string;
    usage: TokenView;
    now: number;
    emptyText: string;
  }

  let { messages, id, title, usage, now, emptyText }: Props = $props();

  const titleId = $derived(`tokens-${id}`);
  const hasUsage = $derived(usage.windowEnd !== null && usage.tokens > 0);
  const reset = $derived(resetText(usage.windowEnd, now, messages));
</script>

<section class="card" aria-labelledby={titleId}>
  <h2 class="title" id={titleId}>{title}</h2>
  {#if hasUsage}
    <p class="value">{formatTokens(usage.tokens, messages)}</p>
    {#if reset !== null}
      <p class="muted">{reset}</p>
    {/if}
  {:else}
    <p class="muted">{emptyText}</p>
  {/if}
</section>

<style>
  .card {
    display: grid;
    gap: var(--space-1);
  }

  .title {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .value {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    line-height: 1.2;
    font-variant-numeric: tabular-nums;
  }

  .muted {
    margin: 0;
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }
</style>
