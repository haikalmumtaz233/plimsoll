<script lang="ts">
  import type { BreakdownsView, HistoryView } from "../api/usage";
  import type { Messages } from "../i18n/messages";
  import type { HistoryRange } from "../usage/history";
  import { focusOnMount } from "../window/focus";
  import UsageBreakdown from "./UsageBreakdown.svelte";
  import UsageChart from "./UsageChart.svelte";

  interface Props {
    messages: Messages;
    history: HistoryView;
    breakdown: BreakdownsView;
    range?: HistoryRange;
  }

  let { messages, history, breakdown, range = $bindable("day") }: Props = $props();
</script>

<section class="history" aria-labelledby="history-page-title">
  <h2 class="heading" id="history-page-title" tabindex="-1" {@attach focusOnMount}>
    {messages.app.history}
  </h2>
  <UsageChart {history} {messages} bind:range />
  <UsageBreakdown breakdown={breakdown[range]} {range} {messages} />
</section>

<style>
  .history {
    display: grid;
    gap: var(--space-4);
  }

  .heading {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .heading:focus {
    outline: none;
  }
</style>
