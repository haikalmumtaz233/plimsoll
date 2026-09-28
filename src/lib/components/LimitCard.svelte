<script lang="ts">
  import type { LimitView, ThresholdsView } from "../api/usage";
  import type { Messages } from "../i18n/messages";
  import { formatPercent, limitTitle, resetText } from "../usage/format";
  import { paceFraction, paceText } from "../usage/pace";

  interface Props {
    messages: Messages;
    limit: LimitView;
    thresholds: ThresholdsView;
    now: number;
  }

  let { messages, limit, thresholds, now }: Props = $props();

  const titleId = $derived(`limit-${limit.kind}`);
  const reset = $derived(resetText(limit.resetsAt, now, messages));
  const pace = $derived(paceFraction(limit.kind, limit.resetsAt, now));
  const paceId = $derived(`pace-${limit.kind}`);
  const paceHint = $derived(pace === null ? null : paceText(limit.percent, pace, messages));
</script>

<section class="card" aria-labelledby={titleId}>
  <div class="row">
    <h2 class="title" id={titleId}>{limitTitle(limit.kind, messages)}</h2>
    {#if reset !== null}
      <p class="muted">{reset}</p>
    {/if}
  </div>
  <p class="value">{formatPercent(limit.percent)}</p>
  <div class="track" title={paceHint ?? undefined}>
    <meter
      class="meter"
      aria-labelledby={titleId}
      aria-describedby={paceHint === null ? undefined : paceId}
      min="0"
      max="100"
      low={thresholds.elevated}
      high={thresholds.high}
      optimum="0"
      value={Math.min(Math.max(limit.percent, 0), 100)}
    >
      {formatPercent(limit.percent)}
    </meter>
    {#if pace !== null}
      <span class="pace" style:left={`${String(pace * 100)}%`} aria-hidden="true"></span>
    {/if}
  </div>
  {#if paceHint !== null}
    <p class="visually-hidden" id={paceId}>{paceHint}</p>
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

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: 0 var(--space-2);
  }

  .track {
    position: relative;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: 0;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  .pace {
    position: absolute;
    top: -0.125rem;
    width: 0.125rem;
    height: 0.75rem;
    margin-left: -0.0625rem;
    border-radius: var(--radius-pill);
    background: var(--color-text);
  }

  .meter {
    display: block;
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

  @media (forced-colors: active) {
    .pace {
      forced-color-adjust: none;
      background: CanvasText;
    }

    .meter,
    .meter::-webkit-meter-bar {
      border: 0.0625rem solid CanvasText;
    }
  }
</style>
