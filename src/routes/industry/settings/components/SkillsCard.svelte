<script lang="ts">
    import { activeCharacterId } from '#lib/stores/global';
    import {
        computedModifiers,
        fetchCharacterSkills,
        groupedIndustrySkills,
        isFetchingSkills,
        lastSyncedCharacter,
        settingsError
    } from '#lib/stores/industrySettings';
    import { formatSkillBonus } from '../settings';
</script>

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