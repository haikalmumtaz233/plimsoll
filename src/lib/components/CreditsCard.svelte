<script lang="ts">
  import type { CreditsView } from "../api/usage";
  import type { Messages } from "../i18n/messages";
  import { creditsSummary } from "../usage/extras";
  import { formatPercent } from "../usage/format";

  interface Props {
    messages: Messages;
    credits: CreditsView;
  }

  let { messages, credits }: Props = $props();

  const summary = $derived(creditsSummary(credits, messages));
  const showMeter = $derived(credits.state === "on" && credits.percent !== null);
</script>

<section class="card" aria-labelledby="credits-title">
  <div class="row">
    <h2 class="title" id="credits-title">{messages.credits.title}</h2>
    <p class="muted">{summary.status}</p>
  </div>
  {#if summary.amount !== null}
    <p class="amount">{summary.amount}</p>
  {/if}
  {#if showMeter && credits.percent !== null}
    <meter
      class="meter"
      aria-labelledby="credits-title"
      min="0"
      max="100"
      optimum="0"
      value={Math.min(Math.max(credits.percent, 0), 100)}
    >
      {formatPercent(credits.percent)}
    </meter>
  {/if}
</section>

<style>
  .card {
    display: grid;
    gap: var(--space-1);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: 0 var(--space-2);
  }

  .title {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .muted {
    margin: 0;
    font-size: 0.875rem;
    color: var(--color-text-muted);
  }

  .amount {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
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

  @media (forced-colors: active) {
    .meter,
    .meter::-webkit-meter-bar {
      border: 0.0625rem solid CanvasText;
    }
  }
</style>
