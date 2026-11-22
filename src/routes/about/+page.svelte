<script lang="ts">
    import "./about.css"
    import { moduleSubNav } from '#lib/stores/global';

    $: moduleSubNav.set([]);
</script>

<div class="about-container">
    <h2>About EVE Industry Helper</h2>
    <p class="version">Version 0.1.0</p>

    <div class="legal-disclaimer">
        <h3>Legal Disclaimer</h3>
        <p>
            EVE Online and the EVE logo are the registered trademarks of CCP hf. All rights are reserved worldwide. All other trademarks are the property of their respective owners. EVE Online, the EVE logo, EVE and all associated logos and designs are the intellectual property of CCP hf. All artwork, screenshots, characters, vehicles, storylines, world facts or other recognizable features of the intellectual property relating to these trademarks are likewise the intellectual property of CCP hf.
        </p>
        <p>
            CCP is in no way responsible for the content on or functioning of this application, nor can it be liable for any damage arising from the use of this application.
        </p>
    </div>
</div>