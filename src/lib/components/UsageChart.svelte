<script lang="ts">
  import type { HistoryView } from "../api/usage";
  import type { Messages } from "../i18n/messages";
  import SegmentedControl from "./SegmentedControl.svelte";
  import { formatTokens } from "../usage/format";
  import {
    DAYS_PER_WEEK_VIEW,
    HOURS_PER_DAY_VIEW,
    compactTokens,
    dailyBars,
    hourlyBars,
    niceCeiling,
    rangeName as nameOfRange,
    totalTokens,
    type Bar,
    type HistoryRange,
  } from "../usage/history";

  interface Props {
    history: HistoryView;
    messages: Messages;
    range?: HistoryRange;
  }

  let { history, messages, range = $bindable("day") }: Props = $props();

  const WIDTH = 320;
  const HEIGHT = 132;
  const LEFT = 34;
  const RIGHT = 2;
  const TOP = 8;
  const BOTTOM = 20;
  const GAP = 2;
  const MAX_BAR = 24;
  const RADIUS = 4;
  const HOUR_LABEL_EVERY = 6;
  const PLOT_WIDTH = WIDTH - LEFT - RIGHT;
  const PLOT_HEIGHT = HEIGHT - TOP - BOTTOM;
  const BASELINE = TOP + PLOT_HEIGHT;
  const ranges: readonly { value: HistoryRange; label: string }[] = $derived([
    { value: "day", label: messages.history.choices.day },
    { value: "week", label: messages.history.choices.week },
  ]);

  let active = $state<number | undefined>(undefined);

  const bars = $derived(
    range === "day"
      ? hourlyBars(history, HOURS_PER_DAY_VIEW, messages.intlLocale)
      : dailyBars(history, DAYS_PER_WEEK_VIEW, messages.intlLocale),
  );
  const ceiling = $derived(niceCeiling(Math.max(0, ...bars.map((bar) => bar.tokens))));
  const ticks = $derived(ceiling === 0 ? [0] : [0, ceiling / 2, ceiling]);
  const slot = $derived(bars.length === 0 ? 0 : PLOT_WIDTH / bars.length);
  const barWidth = $derived(Math.min(MAX_BAR, Math.max(1, slot - GAP)));
  const rangeName = $derived(nameOfRange(range, messages));
  const total = $derived(totalTokens(bars));
  const peak = $derived(
    bars.reduce<Bar | undefined>(
      (best, bar) => (best === undefined || bar.tokens > best.tokens ? bar : best),
      undefined,
    ),
  );
  const activeBar = $derived(active === undefined ? undefined : bars[active]);
  const readout = $derived(
    activeBar === undefined
      ? `${rangeName} · ${formatTokens(total, messages)}`
      : `${activeBar.label} · ${formatTokens(activeBar.tokens, messages)}`,
  );
  const summary = $derived(
    peak === undefined || peak.tokens === 0
      ? messages.history.noUsage(rangeName)
      : messages.history.summary(
          rangeName,
          formatTokens(total, messages),
          formatTokens(peak.tokens, messages),
          peak.label,
        ),
  );

  function yOf(tokens: number): number {
    return ceiling === 0 ? BASELINE : BASELINE - (tokens / ceiling) * PLOT_HEIGHT;
  }

  function barPath(bar: Bar, index: number): string {
    const top = yOf(bar.tokens);
    const height = BASELINE - top;
    if (height <= 0) {
      return "";
    }
    const left = LEFT + index * slot + (slot - barWidth) / 2;
    const right = left + barWidth;
    const radius = Math.min(RADIUS, barWidth / 2, height);
    return [
      `M${String(left)},${String(BASELINE)}`,
      `V${String(top + radius)}`,
      `Q${String(left)},${String(top)} ${String(left + radius)},${String(top)}`,
      `H${String(right - radius)}`,
      `Q${String(right)},${String(top)} ${String(right)},${String(top + radius)}`,
      `V${String(BASELINE)}Z`,
    ].join("");
  }

  function showsLabel(index: number): boolean {
    return range === "week" || index % HOUR_LABEL_EVERY === 0;
  }
</script>

<section class="chart" aria-labelledby="history-title">
  <div class="heading">
    <h2 class="title" id="history-title">{messages.history.title}</h2>
    <SegmentedControl
      name="history-range"
      legend={messages.history.rangeLegend}
      choices={ranges}
      value={range}
      onchange={(next: HistoryRange) => {
        range = next;
      }}
    />
  </div>
  <p class="readout" aria-hidden="true">{readout}</p>
  <svg
    class="plot"
    viewBox="0 0 {WIDTH} {HEIGHT}"
    role="img"
    aria-label={summary}
    onpointerleave={() => (active = undefined)}
  >
    {#each ticks as tick (tick)}
      <line class="grid" x1={LEFT} x2={WIDTH - RIGHT} y1={yOf(tick)} y2={yOf(tick)} />
      <text class="axis" x={LEFT - 4} y={yOf(tick)} text-anchor="end" dominant-baseline="middle">
        {compactTokens(tick, messages.intlLocale)}
      </text>
    {/each}
    {#each bars as bar, index (bar.start)}
      <path class="bar" class:active={active === index} d={barPath(bar, index)} />
      <rect
        class="hit"
        x={LEFT + index * slot}
        y={TOP}
        width={slot}
        height={PLOT_HEIGHT}
        role="presentation"
        onpointerenter={() => (active = index)}
      />
      {#if showsLabel(index)}
        <text class="axis" x={LEFT + index * slot + slot / 2} y={HEIGHT - 4} text-anchor="middle">
          {bar.label}
        </text>
      {/if}
    {/each}
  </svg>
  <details class="data">
    <summary>{messages.history.showData}</summary>
    <table>
      <thead>
        <tr>
          <th scope="col">
            {range === "day" ? messages.history.hourColumn : messages.history.dayColumn}
          </th>
          <th scope="col" class="number">{messages.history.tokensColumn}</th>
        </tr>
      </thead>
      <tbody>
        {#each bars as bar (bar.start)}
          <tr>
            <td>{bar.label}</td>
            <td class="number">{formatTokens(bar.tokens, messages)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </details>
</section>

<style>
  .chart {
    display: grid;
    gap: var(--space-1);
  }

  .heading {
    display: flex;
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

  .readout {
    margin: 0;
    font-size: 0.8125rem;
    font-variant-numeric: tabular-nums;
  }

  .plot {
    display: block;
    width: 100%;
    height: auto;
    overflow: visible;
  }

  .grid {
    stroke: var(--color-border);
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
  }

  .axis {
    fill: var(--color-text-muted);
    font-size: 0.625rem;
    font-variant-numeric: tabular-nums;
  }

  .bar {
    fill: var(--color-chart);
  }

  .bar.active {
    fill: var(--color-chart-active);
  }

  .hit {
    fill: transparent;
    pointer-events: all;
  }

  .data summary {
    width: fit-content;
    font-size: 0.8125rem;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .data table {
    width: 100%;
    margin-top: var(--space-1);
    border-collapse: collapse;
    font-size: 0.8125rem;
    font-variant-numeric: tabular-nums;
  }

  .data th,
  .data td {
    padding: 0.125rem 0;
    border-bottom: 0.0625rem solid var(--color-border);
    text-align: left;
  }

  .data .number {
    text-align: right;
  }

  @media (forced-colors: active) {
    .bar {
      fill: CanvasText;
    }

    .bar.active {
      fill: Highlight;
    }
  }
</style>
