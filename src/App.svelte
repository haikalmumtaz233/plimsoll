<script lang="ts">
  import AppHeader from "./lib/components/AppHeader.svelte";
  import { hidePopup, readAppVersion } from "./lib/api/app";
  import { formatVersion } from "./lib/api/version";

  let version = $state<string | undefined>(undefined);

  $effect(() => {
    readAppVersion()
      .then((value) => {
        version = value;
      })
      .catch(() => {
        version = undefined;
      });
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      void hidePopup();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="popup">
  <AppHeader versionLabel={version === undefined ? "" : formatVersion(version)} />
  <p class="status" role="status">Usage tracking is not connected yet.</p>
</main>

<style>
  .popup {
    display: grid;
    gap: var(--space-4);
    padding: var(--space-4);
  }

  .status {
    margin: 0;
    color: var(--color-text-muted);
  }
</style>
