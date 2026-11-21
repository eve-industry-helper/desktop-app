<script lang="ts">
    import './settings.css';
    import { activeCharacterId } from '$lib/stores/global';
    import {
        groupedIndustrySkills,
        computedModifiers,
        isFetchingSkills,
        settingsError,
        lastSyncedCharacter,
        fetchCharacterSkills,
        formatSkillBonus,
        structuresList,
        STRUCTURE_TYPES,
        SECURITY_STATUSES,
        AVAILABLE_SERVICES,
        isEditingStructure,
        structureFitPaste,
        structureForm,
        availableRigs,
        openNewStructure,
        openEditStructure,
        cancelEditStructure,
        applyEftFit,
        toggleService,
        handleSaveStructure,
        handleFileSelected,
        deleteStructure,
        exportStructureToFile,
        priceSettings,
        PRICE_PROVIDERS,
        MARKET_HUBS,
        PRICE_BASIS_OPTIONS,
        updatePriceSettings
    } from './settings';
</script>

<div class="settings-container">
    <div class="settings-header">
        <h2>Industry Settings</h2>
        <span class="subtitle">Configure character skills, facilities, and market pricing for job calculations</span>
    </div>

    <div class="settings-grid">
        <!-- Left Column: Character Skills -->
        <div class="settings-card">
            <div class="card-header">
                <div>
                    <h3>Character Skills</h3>
                    <span class="card-subtitle">
                        {#if $lastSyncedCharacter}
                            Synced with <strong>{$lastSyncedCharacter}</strong>
                        {:else}
                            Fetch skills from ESI to calculate accurate job times, yields, and taxes
                        {/if}
                    </span>
                </div>

                <button
                        type="button"
                        class="primary-btn"
                        on:click={fetchCharacterSkills}
                        disabled={$isFetchingSkills || !$activeCharacterId}
                >
                    {$isFetchingSkills ? 'Fetching...' : 'Sync Skills from ESI'}
                </button>
            </div>

            {#if $settingsError}
                <p class="error-text">{$settingsError}</p>
            {/if}

            <div class="modifiers-summary">
                <span>Mfg Slots: <strong>{$computedModifiers.maxManufacturingSlots}</strong></span>
                <span>Rxn Slots: <strong>{$computedModifiers.maxReactionSlots}</strong></span>
                <span>Sci Slots: <strong>{$computedModifiers.maxScienceSlots}</strong></span>
                <span>Mfg Time: <strong>{($computedModifiers.manufacturingTimeMultiplier * 100).toFixed(1)}%</strong></span>
                <span>Rxn Time: <strong>{($computedModifiers.reactionTimeMultiplier * 100).toFixed(1)}%</strong></span>
            </div>

            <div class="skill-groups">
                {#each $groupedIndustrySkills as section (section.group)}
                    <details class="skill-group-accordion" open={section.group === 'Production'}>
                        <summary class="group-header">
                            <img src="/icon-chevron.svg" alt="" class="group-chevron" />
                            <span class="group-title">{section.group}</span>
                            <span class="group-count">{section.skills.length} skills</span>
                        </summary>

                        <div class="skills-grid">
                            {#each section.skills as skill (skill.id)}
                                <div class="skill-row">
                                    <div class="skill-info">
                                        <span class="skill-name">{skill.name}</span>
                                        <span class="skill-effect">{formatSkillBonus(skill)}</span>
                                    </div>
                                    <div class="skill-pips" title={`Level ${skill.level} / 5`}>
                                        {#each [1, 2, 3, 4, 5] as pip}
                                            <span class="pip" class:filled={pip <= skill.level}></span>
                                        {/each}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </details>
                {/each}
            </div>
        </div>

        <!-- Right Column: Structures + Market & Pricing -->
        <div class="settings-column">
            <!-- Card 1: Structures & Facilities -->
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
                                    type="file"
                                    accept=".json,application/json"
                                    on:change={handleFileSelected}
                                    class="hidden-file-input"
                            />
                        </label>
                        <button type="button" class="primary-btn" on:click={openNewStructure}>
                            + Add Structure
                        </button>
                    </div>
                </div>

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
                                <input type="text" bind:value={$structureForm.name} placeholder="e.g. Jita Azbel" class="text-input" />
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
                                <input type="number" step="0.01" min="0" bind:value={$structureForm.manufacturingCostIndex} class="text-input" />
                            </label>

                            <label class="field-label">
                                <span>Reaction Cost Index (%)</span>
                                <input type="number" step="0.01" min="0" bind:value={$structureForm.reactionCostIndex} class="text-input" />
                            </label>

                            <label class="field-label">
                                <span>Facility Tax (%)</span>
                                <input type="number" step="0.1" min="0" bind:value={$structureForm.facilityTax} class="text-input" />
                            </label>
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
                                    <input type="text" list="available-rigs" bind:value={$structureForm.rigs[0]} placeholder="Rig Slot 1" class="text-input" />
                                    <input type="text" list="available-rigs" bind:value={$structureForm.rigs[1]} placeholder="Rig Slot 2" class="text-input" />
                                    <input type="text" list="available-rigs" bind:value={$structureForm.rigs[2]} placeholder="Rig Slot 3" class="text-input" />
                                </div>
                            </div>
                        {/if}

                        <div class="form-actions">
                            <button type="button" class="secondary-btn" on:click={cancelEditStructure}>
                                Cancel
                            </button>
                            <button type="button" class="primary-btn" on:click={handleSaveStructure} disabled={!$structureForm.name.trim()}>
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
                                <button type="button" class="secondary-btn small" on:click={() => exportStructureToFile(s)}>
                                    Share
                                </button>
                                <button type="button" class="remove-btn small" on:click={() => deleteStructure(s.id)}>
                                    Delete
                                </button>
                            </div>
                        </div>
                    {:else}
                        <p class="empty-hint">No structures configured yet. Add one manually, paste a fit, or import a JSON file.</p>
                    {/each}
                </div>
            </div>

            <!-- Card 2: Market & Pricing -->
            <div class="settings-card">
                <div class="card-header">
                    <div>
                        <h3>Market & Pricing</h3>
                        <span class="card-subtitle">
                            Select how material costs and finished product values are fetched
                        </span>
                    </div>
                </div>

                <div class="pricing-rows">
                    <!-- Line 1: Price Provider -->
                    <div class="pricing-row">
                        <label class="field-label auto-width">
                            <span>Price Provider</span>
                            <select
                                    class="text-input auto-width"
                                    value={$priceSettings.provider}
                                    on:change={(e) => updatePriceSettings({ provider: e.currentTarget.value as any })}
                            >
                                {#each PRICE_PROVIDERS as p (p.value)}
                                    <option value={p.value}>{p.label}</option>
                                {/each}
                            </select>
                        </label>

                        {#if $priceSettings.provider === 'janice'}
                            <label class="field-label">
                                <span>Janice API Key</span>
                                <input
                                        type="password"
                                        class="text-input"
                                        placeholder="Paste your Janice X-ApiKey..."
                                        value={$priceSettings.janiceApiKey}
                                        on:change={(e) => updatePriceSettings({ janiceApiKey: e.currentTarget.value.trim() })}
                                />
                            </label>
                        {/if}
                    </div>

                    <!-- Line 2: Buying Settings -->
                    <div class="pricing-group">
                        <span class="pricing-group-title">Buying Settings</span>
                        <div class="pricing-pair">
                            <label class="field-label">
                                <span>Market Hub</span>
                                <select
                                        class="text-input"
                                        value={$priceSettings.buyRegionId}
                                        on:change={(e) => updatePriceSettings({ buyRegionId: Number(e.currentTarget.value) })}
                                >
                                    {#each MARKET_HUBS as hub (hub.id)}
                                        <option value={hub.id}>{hub.name}</option>
                                    {/each}
                                </select>
                            </label>

                            <label class="field-label">
                                <span>Material Cost Basis</span>
                                <select
                                        class="text-input"
                                        value={$priceSettings.materialPriceBasis}
                                        on:change={(e) => updatePriceSettings({ materialPriceBasis: e.currentTarget.value as any })}
                                >
                                    {#each PRICE_BASIS_OPTIONS as opt (opt.value)}
                                        <option value={opt.value}>{opt.label}</option>
                                    {/each}
                                </select>
                            </label>
                        </div>
                    </div>

                    <!-- Line 3: Selling Settings -->
                    <div class="pricing-group">
                        <span class="pricing-group-title">Selling Settings</span>
                        <div class="pricing-pair">
                            <label class="field-label">
                                <span>Market Hub</span>
                                <select
                                        class="text-input"
                                        value={$priceSettings.sellRegionId}
                                        on:change={(e) => updatePriceSettings({ sellRegionId: Number(e.currentTarget.value) })}
                                >
                                    {#each MARKET_HUBS as hub (hub.id)}
                                        <option value={hub.id}>{hub.name}</option>
                                    {/each}
                                </select>
                            </label>

                            <label class="field-label">
                                <span>Product Sale Basis</span>
                                <select
                                        class="text-input"
                                        value={$priceSettings.productPriceBasis}
                                        on:change={(e) => updatePriceSettings({ productPriceBasis: e.currentTarget.value as any })}
                                >
                                    {#each PRICE_BASIS_OPTIONS as opt (opt.value)}
                                        <option value={opt.value}>{opt.label}</option>
                                    {/each}
                                </select>
                            </label>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>