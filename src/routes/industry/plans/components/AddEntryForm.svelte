<script lang="ts">
    import {
        addEntryMode,
        cancelAddEntry,
        fitInput,
        handleAddEntry,
        industryError,
        isResolving,
        singleItemName,
        singleItemRuns,
        blueprintSuggestions,
        clearBlueprintSuggestions,
        pickBlueprintSuggestion,
        searchBlueprints
    } from '../plans';

    let highlightedIndex = -1;

    $: if ($addEntryMode) {
        highlightedIndex = -1;
        clearBlueprintSuggestions();
    }

    function handleNameInput(event: Event) {
        const value = (event.currentTarget as HTMLInputElement).value;
        highlightedIndex = -1;
        searchBlueprints(value, $addEntryMode);
    }

    function handleNameKeydown(event: KeyboardEvent) {
        const list = $blueprintSuggestions;
        if (list.length === 0) return;

        if (event.key === 'ArrowDown') {
            event.preventDefault();
            highlightedIndex = (highlightedIndex + 1) % list.length;
        } else if (event.key === 'ArrowUp') {
            event.preventDefault();
            highlightedIndex = (highlightedIndex - 1 + list.length) % list.length;
        } else if (event.key === 'Enter' && highlightedIndex >= 0) {
            event.preventDefault();
            pickBlueprintSuggestion(list[highlightedIndex]);
            highlightedIndex = -1;
        } else if (event.key === 'Escape') {
            clearBlueprintSuggestions();
            highlightedIndex = -1;
        }
    }

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
            <div class="field-label">
                <label for="single-item-input">
                    {$addEntryMode === 'reaction' ? 'Reaction Formula or Product Name' : 'Blueprint or Item Name'}
                </label>
                <div class="autocomplete-wrap">
                    <input
                            id="single-item-input"
                            type="text"
                            class="text-input"
                            autocomplete="off"
                            bind:value={$singleItemName}
                            on:input={handleNameInput}
                            on:keydown={handleNameKeydown}
                            on:blur={() => setTimeout(clearBlueprintSuggestions, 120)}
                            placeholder={$addEntryMode === 'reaction' ? 'e.g. Fermionic Condensates...' : 'e.g. Nighthawk Blueprint...'}
                    />

                    {#if $blueprintSuggestions.length > 0}
                        <div class="autocomplete-dropdown" role="listbox">
                            {#each $blueprintSuggestions as item, idx (item.blueprintTypeId)}
                                <button
                                        type="button"
                                        class="autocomplete-option"
                                        class:highlighted={idx === highlightedIndex}
                                        role="option"
                                        aria-selected={idx === highlightedIndex}
                                        on:mousedown|preventDefault={() => {
                                            pickBlueprintSuggestion(item);
                                            highlightedIndex = -1;
                                        }}
                                >
                                    <img
                                            src={`https://images.evetech.net/types/${item.blueprintTypeId}/bp?size=32`}
                                            alt=""
                                            class="material-icon"
                                    />
                                    <span class="autocomplete-text">
                                        <span class="autocomplete-title">{item.blueprintName}</span>
                                        <span class="autocomplete-sub">Produces: {item.productName}</span>
                                    </span>
                                </button>
                            {/each}
                        </div>
                    {/if}
                </div>
            </div>

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