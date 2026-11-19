<script lang="ts">
    import './setup.css';
    import { onMount } from "svelte";
    import { config, statusMessage, initializeSetup, saveConfig } from "./setup";

    onMount(initializeSetup);
</script>

<main class="container">
    <div class="content">
        <h1>EVE Industry Setup</h1>
        <div class="settings-card">
            <div class="form-group">
                <label for="mode-select"><strong>Application Mode:</strong></label>
                <!-- Svelte's bind:value is smart enough to automatically update the store! -->
                <select id="mode-select" bind:value={$config.mode}>
                    <option value="Standalone">Standalone (Local DB & ESI)</option>
                    <option value="Alliance">Alliance (Server Sync)</option>
                </select>
            </div>

            {#if $config.mode === "Alliance"}
                <div class="form-group">
                    <label for="api-url"><strong>Server API URL:</strong></label>
                    <input id="api-url" type="text" bind:value={$config.api_url} placeholder="https://api.yourcorp.com" />
                </div>
            {/if}

            <button class="primary-button" on:click={saveConfig}>Save Configuration</button>

            {#if $statusMessage}
                <p class="status">{$statusMessage}</p>
            {/if}
        </div>
    </div>
</main>