<script lang="ts" generics="T extends string">
  interface Choice {
    value: T;
    label: string;
  }

  interface Props {
    name: string;
    legend: string;
    choices: readonly Choice[];
    value: T;
    onchange: (value: T) => void;
  }

  let { name, legend, choices, value, onchange }: Props = $props();
</script>

<fieldset class="segmented">
  <legend class="visually-hidden">{legend}</legend>
  {#each choices as choice (choice.value)}
    <label class="choice">
      <input
        type="radio"
        {name}
        value={choice.value}
        checked={choice.value === value}
        onchange={() => {
          onchange(choice.value);
        }}
      />
      <span>{choice.label}</span>
    </label>
  {/each}
</fieldset>

<style>
  .segmented {
    display: flex;
    gap: 0.125rem;
    margin: 0;
    padding: 0.125rem;
    border: 0.0625rem solid var(--color-border);
    border-radius: var(--radius-pill);
  }

  .choice {
    position: relative;
    display: block;
  }

  .choice input {
    position: absolute;
    inset: 0;
    margin: 0;
    opacity: 0;
    cursor: pointer;
  }

  .choice span {
    display: flex;
    align-items: center;
    min-height: 1.5rem;
    padding: 0 0.625rem;
    white-space: nowrap;
    border-radius: var(--radius-pill);
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  .choice input:checked + span {
    background: var(--color-selected);
    color: var(--color-on-selected);
    font-weight: 600;
  }

  .choice input:focus-visible + span {
    outline: 0.125rem solid var(--color-focus);
    outline-offset: 0.125rem;
  }

  @media (forced-colors: active) {
    .choice input:checked + span {
      forced-color-adjust: none;
      background: Highlight;
      color: HighlightText;
    }
  }
</style>
