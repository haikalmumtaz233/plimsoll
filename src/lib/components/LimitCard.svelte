<script lang="ts">
  import type { LimitView } from "../api/usage";
  import { formatPercent, limitTitle, resetText } from "../usage/format";

  interface Props {
    limit: LimitView;
    now: number;
  }

  let { limit, now }: Props = $props();

  const titleId = $derived(`limit-${limit.kind}`);
  const reset = $derived(resetText(limit.resetsAt, now));
</script>

<section class="card" aria-labelledby={titleId}>
  <h2 class="title" id={titleId}>{limitTitle(limit.kind)}</h2>
  <p class="value">{formatPercent(limit.percent)} <span class="unit">used</span></p>
  <meter
    class="meter"
    aria-labelledby={titleId}
    min="0"
    max="100"
    low="50"
    high="80"
    optimum="0"
    value={Math.min(Math.max(limit.percent, 0), 100)}
  >
    {formatPercent(limit.percent)}
  </meter>
  {#if reset !== null}
    <p class="muted">{reset}</p>
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
    font-size: 1.75rem;
    font-weight: 600;
    line-height: 1.2;
    font-variant-numeric: tabular-nums;
  }

  .unit {
    font-size: 0.875rem;
    font-weight: 400;
    color: var(--color-text-muted);
  }

  .meter {
    appearance: none;
    width: 100%;
    height: 0.5rem;
    border: 0;
    border-radius: var(--radius-pill);
    background: var(--color-track);
  }

  .meter::-webkit-meter-bar {
    height: 0.5rem;
    border: 0;
    border-radius: var(--radius-pill);
    background: var(--color-track);
  }

  .meter::-webkit-meter-optimum-value {
    border-radius: var(--radius-pill);
    background: var(--color-ok);
  }

  .meter::-webkit-meter-suboptimum-value {
    border-radius: var(--radius-pill);
    background: var(--color-warn);
  }

  .meter::-webkit-meter-even-less-good-value {
    border-radius: var(--radius-pill);
    background: var(--color-danger);
  }

  .muted {
    margin: 0;
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }
</style>
