<script lang="ts">
    import './settings.css';
    import SkillsCard from './components/SkillsCard.svelte';
    import StructuresCard from './components/StructuresCard.svelte';
    import PricingCard from './components/PricingCard.svelte';
</script>

<div class="settings-container">
    <div class="settings-header">
        <h2>Industry Settings</h2>
        <span class="subtitle">Configure character skills, facilities, and market pricing for job calculations</span>
    </div>

    <div class="settings-grid">
        <SkillsCard />

        <div class="settings-column">
            <StructuresCard />
            <PricingCard />
        </div>
    </div>
</div>