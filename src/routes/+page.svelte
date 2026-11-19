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

<style>
  .loading-text {
    color: #888;
    font-size: 1.1rem;
    animation: pulse 1.5s infinite;
  }
  @keyframes pulse {
    0% { opacity: 0.5; }
    50% { opacity: 1; }
    100% { opacity: 0.5; }
  }
</style>