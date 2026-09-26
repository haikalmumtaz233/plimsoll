<script lang="ts">
  import type { BreakdownView } from "../api/usage";
  import type { Messages } from "../i18n/messages";
  import { formatShare, shareRows, type BreakdownKind } from "../usage/breakdown";
  import { compactTokens, type HistoryRange } from "../usage/history";
  import SegmentedControl from "./SegmentedControl.svelte";

  interface Props {
    breakdown: BreakdownView;
    range: HistoryRange;
    messages: Messages;
  }

  let { breakdown, range, messages }: Props = $props();

  const kinds: readonly { value: BreakdownKind; label: string }[] = $derived([
    { value: "models", label: messages.breakdown.model },
    { value: "projects", label: messages.breakdown.project },
  ]);
  const TRACK_WIDTH = 100;
  const MIN_FILL = 1;

  let kind = $state<BreakdownKind>("models");

  const rows = $derived(shareRows(breakdown[kind], kind, messages));
</script>

<section class="breakdown" aria-labelledby="breakdown-title">
  <div class="heading">
    <h2 class="title" id="breakdown-title">{messages.breakdown.title}</h2>
    <SegmentedControl
      name="breakdown-kind"
      legend={messages.breakdown.groupBy}
      choices={kinds}
      value={kind}
      onchange={(next: BreakdownKind) => {
        kind = next;
      }}
    />
  </div>
  {#if rows.length === 0}
    <p class="muted">{messages.breakdown.noUsage[range]}</p>
  {:else}
    <ul class="rows">
      {#each rows as row (row.key)}
        <li class="row">
          <span class="label" title={row.label}>{row.label}</span>
          <span class="value">
            {messages.breakdown.share(
              formatShare(row.fraction),
              compactTokens(row.tokens, messages.intlLocale),
            )}
          </span>
          <svg
            class="share"
            viewBox="0 0 {TRACK_WIDTH} 4"
            preserveAspectRatio="none"
            aria-hidden="true"
          >
            <rect class="track" width={TRACK_WIDTH} height="4" />
            <rect class="fill" width={Math.max(row.fraction * TRACK_WIDTH, MIN_FILL)} height="4" />
          </svg>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .breakdown {
    display: grid;
    gap: var(--space-2);
  }

  .heading {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .title {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .rows {
    display: grid;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 0.125rem var(--space-2);
    font-size: 0.8125rem;
  }

  .label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .value {
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }

  .share {
    flex-basis: 100%;
    display: block;
    width: 100%;
    height: 0.25rem;
    border-radius: var(--radius-pill);
    overflow: hidden;
  }

  .track {
    fill: var(--color-track);
  }

  .fill {
    fill: var(--color-chart);
  }

  .muted {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  @media (forced-colors: active) {
    .track {
      fill: Canvas;
      stroke: CanvasText;
    }

    .fill {
      fill: CanvasText;
    }
  }
</style>
