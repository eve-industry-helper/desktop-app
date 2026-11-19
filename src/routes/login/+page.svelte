<script lang="ts">
    import './login.css';
    import { onMount } from "svelte";
    import {
        config,
        statusMessage,
        isLoggingIn,
        initializeLogin,
        loginStandalone,
        goBack
    } from "./login";

    onMount(initializeLogin);
</script>

<main class="container">
    <div class="content">
        <h1>Authentication Required</h1>
        <div class="settings-card auth-section">
            {#if $config?.mode === "Standalone"}
                <p class="auth-desc">Log in directly via EVE SSO to fetch your character's industry data.</p>
                <button class="eve-login-button" on:click={loginStandalone} disabled={$isLoggingIn}>
                    {$isLoggingIn ? "Waiting for authorization..." : "LOG IN WITH EVE ONLINE"}
                </button>
            {:else if $config?.mode === "Alliance"}
                <p class="auth-desc">Connecting to your Quarkus server for authentication.</p>
                <button class="eve-login-button" disabled>LOGIN VIA QUARKUS (Coming Soon)</button>
            {/if}

            <button class="text-button" on:click={goBack} disabled={$isLoggingIn}>
                ← Go Back
            </button>

            {#if $statusMessage}
                <p class="status {$isLoggingIn ? 'info' : 'error'}">{$statusMessage}</p>
            {/if}
        </div>
    </div>
</main>