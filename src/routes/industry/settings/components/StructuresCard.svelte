<script lang="ts">
    import {deleteStructure, structuresList} from '$lib/stores/industrySettings';
    import {
        applyEftFit,
        assetLocations,
        AVAILABLE_SERVICES,
        availableRigs,
        cancelEditStructure,
        copyStructureJson,
        handleFileSelected,
        handleSaveStructure,
        isEditingStructure,
        isLoadingAssetLocations,
        loadAssetLocations,
        openEditStructure,
        openNewStructure,
        saveStructureJsonFile,
        SECURITY_STATUSES,
        shareNotice,
        STRUCTURE_TYPES,
        structureFitPaste,
        structureForm,
        toggleService
    } from '../settings';
    import {activeCharacterId} from '$lib/stores/global';

    let openShareMenuId: string | null = null;

    function toggleShareMenu(id: string) {
        openShareMenuId = openShareMenuId === id ? null : id;
    }

    function closeShareMenu() {
        openShareMenuId = null;
    }
</script>

<svelte:window
        on:click={closeShareMenu}
        on:keydown={(e) => {
        if (e.key === 'Escape') closeShareMenu();
    }}
/>

<div class="settings-card">
    <div class="card-header">
        <div>
            <h3>Structures & Facilities</h3>
            <span class="card-subtitle">
                Configure stations and Upwell structures for job cost, tax, and rig bonuses
            </span>
        </div>

        <div class="header-actions">
            <label class="secondary-btn file-btn">
                Import JSON
                <input
                        accept=".json,application/json"
                        class="hidden-file-input"
                        on:change={handleFileSelected}
                        type="file"
                />
            </label>
            <button class="primary-btn" on:click={openNewStructure} type="button">
                + Add Structure
            </button>
        </div>
    </div>

    {#if $shareNotice}
        <div class="share-notice">{$shareNotice}</div>
    {/if}

    {#if $isEditingStructure}
        <div class="structure-form">
            <div class="eft-import-box">
                <label class="field-label">
                    <span>Quick Fill from EFT Structure Fit (optional)</span>
                    <textarea
                            bind:value={$structureFitPaste}
                            rows="4"
                            placeholder="[Azbel, Corp Indy]&#10;Standup Manufacturing Plant I&#10;Standup L-Set Equipment Manufacturing Efficiency I"
                    ></textarea>
                </label>
                <button
                        type="button"
                        class="secondary-btn"
                        on:click={applyEftFit}
                        disabled={!$structureFitPaste.trim()}
                >
                    Parse Fit
                </button>
            </div>

            <div class="form-grid">
                <label class="field-label">
                    <span>Structure Name</span>
                    <input type="text" bind:value={$structureForm.name} placeholder="e.g. Jita Azbel"
                           class="text-input"/>
                </label>

                <label class="field-label">
                    <span>Structure Type</span>
                    <select bind:value={$structureForm.structureType} class="text-input">
                        {#each STRUCTURE_TYPES as st (st.value)}
                            <option value={st.value}>{st.label}</option>
                        {/each}
                    </select>
                </label>

                <label class="field-label">
                    <span>Security Status</span>
                    <select bind:value={$structureForm.securityStatus} class="text-input">
                        {#each SECURITY_STATUSES as sec (sec.value)}
                            <option value={sec.value}>{sec.label}</option>
                        {/each}
                    </select>
                </label>

                <label class="field-label">
                    <span>Mfg Cost Index (%)</span>
                    <input type="number" step="0.01" min="0" bind:value={$structureForm.manufacturingCostIndex}
                           class="text-input"/>
                </label>

                <label class="field-label">
                    <span>Reaction Cost Index (%)</span>
                    <input type="number" step="0.01" min="0" bind:value={$structureForm.reactionCostIndex}
                           class="text-input"/>
                </label>

                <label class="field-label">
                    <span>Facility Tax (%)</span>
                    <input type="number" step="0.1" min="0" bind:value={$structureForm.facilityTax} class="text-input"/>
                </label>
            </div>

            <div class="form-section">
                <span class="field-title">Linked EVE Location (for Local Stock Deduction)</span>
                <div class="location-picker-row">
                    <select bind:value={$structureForm.locationId} class="text-input">
                        <option value={null}>None (do not deduct local stock)</option>
                        {#if $structureForm.locationId != null && !$assetLocations.some(l => l.locationId === $structureForm.locationId)}
                            <option value={$structureForm.locationId}>
                                Linked Location ID: {$structureForm.locationId}
                            </option>
                        {/if}
                        {#each $assetLocations as loc (loc.locationId)}
                            <option value={loc.locationId}>
                                {loc.name} ({loc.itemCount} stacks)
                            </option>
                        {/each}
                    </select>
                    <button
                            type="button"
                            class="secondary-btn"
                            on:click={() => loadAssetLocations(true)}
                            disabled={$isLoadingAssetLocations || !$activeCharacterId}
                    >
                        {$isLoadingAssetLocations ? 'Loading...' : 'Refresh Assets'}
                    </button>
                </div>
            </div>

            <div class="form-section">
                <span class="field-title">Services Offered</span>
                <div class="services-row">
                    {#each AVAILABLE_SERVICES as srv (srv)}
                        <label class="checkbox-pill">
                            <input
                                    type="checkbox"
                                    checked={$structureForm.services.includes(srv)}
                                    on:change={() => toggleService(srv)}
                            />
                            <span>{srv}</span>
                        </label>
                    {/each}
                </div>
            </div>

            {#if $structureForm.structureType !== 'npcstation'}
                <div class="form-section">
                    <span class="field-title">Rigs (up to 3 — filtered for selected structure)</span>
                    <datalist id="available-rigs">
                        {#each $availableRigs as rig (rig)}
                            <option value={rig}></option>
                        {/each}
                    </datalist>
                    <div class="rigs-grid">
                        <input type="text" list="available-rigs" bind:value={$structureForm.rigs[0]}
                               placeholder="Rig Slot 1" class="text-input"/>
                        <input type="text" list="available-rigs" bind:value={$structureForm.rigs[1]}
                               placeholder="Rig Slot 2" class="text-input"/>
                        <input type="text" list="available-rigs" bind:value={$structureForm.rigs[2]}
                               placeholder="Rig Slot 3" class="text-input"/>
                    </div>
                </div>
            {/if}

            <div class="form-actions">
                <button type="button" class="secondary-btn" on:click={cancelEditStructure}>
                    Cancel
                </button>
                <button type="button" class="primary-btn" on:click={handleSaveStructure}
                        disabled={!$structureForm.name.trim()}>
                    Save Structure
                </button>
            </div>
        </div>
    {/if}

    <div class="structures-list">
        {#each $structuresList as s (s.id)}
            <div class="structure-item">
                <div class="structure-main">
                    <div class="structure-title-row">
                        <span class="structure-name">{s.name}</span>
                        <span class="badge">{s.structureType.toUpperCase()}</span>
                        <span class="badge sec">{s.securityStatus.toUpperCase()}</span>
                    </div>
                    <div class="structure-meta">
                        Mfg Index: <strong>{s.manufacturingCostIndex}%</strong> ·
                        Rxn Index: <strong>{s.reactionCostIndex}%</strong> ·
                        Tax: <strong>{s.facilityTax}%</strong>
                    </div>
                    <div class="structure-meta">
                        {#if s.locationId}
                            Stock Link: <strong>ID {s.locationId}</strong>
                        {:else}
                            Stock Link: Not linked
                        {/if}
                    </div>
                    {#if s.services.length > 0}
                        <div class="structure-meta">Services: {s.services.join(', ')}</div>
                    {/if}
                    {#if s.rigs.length > 0}
                        <div class="structure-meta">Rigs: {s.rigs.join(' · ')}</div>
                    {/if}
                </div>

                <div class="structure-actions">
                    <button type="button" class="secondary-btn small" on:click={() => openEditStructure(s)}>
                        Edit
                    </button>
                    <div class="share-menu-wrapper" on:click|stopPropagation>
                        <button
                                type="button"
                                class="secondary-btn small"
                                aria-haspopup="menu"
                                aria-expanded={openShareMenuId === s.id}
                                on:click={() => toggleShareMenu(s.id)}
                        >
                            Share ▾
                        </button>

                        {#if openShareMenuId === s.id}
                            <div class="share-dropdown" role="menu">
                                <button
                                        type="button"
                                        class="share-menu-item"
                                        role="menuitem"
                                        on:click={() => {
                    closeShareMenu();
                    copyStructureJson(s);
                }}
                                >
                                    Copy JSON to Clipboard
                                </button>
                                <button
                                        type="button"
                                        class="share-menu-item"
                                        role="menuitem"
                                        on:click={() => {
                    closeShareMenu();
                    saveStructureJsonFile(s);
                }}
                                >
                                    Save JSON to Downloads
                                </button>
                            </div>
                        {/if}
                    </div>
                    <button type="button" class="remove-btn small" on:click={() => deleteStructure(s.id)}>
                        Delete
                    </button>
                </div>
            </div>
        {:else}
            <p class="empty-hint">No structures configured yet. Add one manually, paste a fit, or import a JSON
                file.</p>
        {/each}
    </div>
</div>