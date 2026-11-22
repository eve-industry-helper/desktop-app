<script lang="ts">
    import { priceSettings } from '#lib/stores/industrySettings';
    import {
        MARKET_HUBS,
        PRICE_BASIS_OPTIONS,
        PRICE_PROVIDERS,
        handleBuyHubChange,
        handleJaniceKeyChange,
        handleMaterialBasisChange,
        handleProductBasisChange,
        handleProviderChange,
        handleSellHubChange
    } from '../settings';
</script>

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
                        on:change={handleProviderChange}
                >
                    {#each PRICE_PROVIDERS as p (p.value)}
                        <option value={p.value} selected={p.value === $priceSettings.provider}>
                            {p.label}
                        </option>
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
                            on:change={handleJaniceKeyChange}
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
                            on:change={handleBuyHubChange}
                    >
                        {#each MARKET_HUBS as hub (hub.id)}
                            <option value={hub.id} selected={hub.id === $priceSettings.buyRegionId}>
                                {hub.name}
                            </option>
                        {/each}
                    </select>
                </label>

                <label class="field-label">
                    <span>Material Cost Basis</span>
                    <select
                            class="text-input"
                            on:change={handleMaterialBasisChange}
                    >
                        {#each PRICE_BASIS_OPTIONS as opt (opt.value)}
                            <option value={opt.value} selected={opt.value === $priceSettings.materialPriceBasis}>
                                {opt.label}
                            </option>
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
                            on:change={handleSellHubChange}
                    >
                        {#each MARKET_HUBS as hub (hub.id)}
                            <option value={hub.id} selected={hub.id === $priceSettings.sellRegionId}>
                                {hub.name}
                            </option>
                        {/each}
                    </select>
                </label>

                <label class="field-label">
                    <span>Product Sale Basis</span>
                    <select
                            class="text-input"
                            on:change={handleProductBasisChange}
                    >
                        {#each PRICE_BASIS_OPTIONS as opt (opt.value)}
                            <option value={opt.value} selected={opt.value === $priceSettings.productPriceBasis}>
                                {opt.label}
                            </option>
                        {/each}
                    </select>
                </label>
            </div>
        </div>
    </div>
</div>