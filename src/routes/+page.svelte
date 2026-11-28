<script lang="ts">
  import { onMount } from "svelte";
  import { errorMessage, initializeRouter } from "./router";

  onMount(initializeRouter);
</script>

<main class="container">
  <div class="content">
    {#if $errorMessage}
      <p class="status error">{$errorMessage}</p>
    {:else}
      <p class="loading-text">Loading EVE Industry Helper...</p>
    {/if}
  </div>
</main>