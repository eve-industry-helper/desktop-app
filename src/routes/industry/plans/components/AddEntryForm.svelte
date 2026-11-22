<script lang="ts">
    import {
        addEntryMode,
        cancelAddEntry,
        fitInput,
        handleAddEntry,
        industryError,
        isResolving,
        singleItemName,
        singleItemRuns
    } from '../plans';

    export let planName: string;
</script>

<div class="workspace-header">
    <div>
        <h2>Add to {planName}</h2>
        <span class="subtitle">Choose what you want to build</span>
    </div>
</div>

<div class="mode-tabs">
    <button
            type="button"
            class="mode-tab"
            class:active={$addEntryMode === 'fit'}
            on:click={() => addEntryMode.set('fit')}
    >
        Import Fit
    </button>
    <button
            type="button"
            class="mode-tab"
            class:active={$addEntryMode === 'blueprint'}
            on:click={() => addEntryMode.set('blueprint')}
    >
        Single Blueprint
    </button>
    <button
            type="button"
            class="mode-tab"
            class:active={$addEntryMode === 'reaction'}
            on:click={() => addEntryMode.set('reaction')}
    >
        Reaction
    </button>
</div>

<div class="paste-box">
    {#if $addEntryMode === 'fit'}
        <textarea
                bind:value={$fitInput}
                placeholder="[Iteron Mark V, Hauler Fit]&#10;Expanded Cargohold II&#10;10MN Afterburner II&#10;Hobgoblin II x5"
                rows="10"
        ></textarea>
    {:else}
        <div class="single-form">
            <label class="field-label">
                <span>{$addEntryMode === 'reaction' ? 'Reaction Formula or Product Name' : 'Blueprint or Item Name'}</span>
                <input
                        type="text"
                        bind:value={$singleItemName}
                        placeholder={$addEntryMode === 'reaction' ? 'e.g. Carbon Polymers' : 'e.g. Rifter'}
                        class="text-input"
                />
            </label>

            <label class="field-label runs-field">
                <span>Runs</span>
                <input
                        type="number"
                        min="1"
                        bind:value={$singleItemRuns}
                        class="text-input"
                />
            </label>
        </div>
    {/if}

    <div class="paste-actions">
        {#if $industryError}
            <span class="error-text">{$industryError}</span>
        {/if}
        <button type="button" class="secondary-btn" on:click={cancelAddEntry}>
            Cancel
        </button>
        <button
                type="button"
                class="primary-btn"
                on:click={handleAddEntry}
                disabled={$isResolving || ($addEntryMode === 'fit' ? !$fitInput.trim() : !$singleItemName.trim())}
        >
            {$isResolving ? 'Resolving...' : 'Add to Plan'}
        </button>
    </div>
</div>