<script lang="ts">
	import Tilemap from "../Tilemap.svelte";
	import BackgroundsCanvas from "./BackgroundsCanvas.svelte";
	import PalettesCanvas from "./PalettesCanvas.svelte";
	import TilesCanvas from "./TilesCanvas.svelte";
    import { runPeriodically, clearRunPeriodically } from "$lib/utils";

	import { gbaStore } from "$lib/gbaStore";
	import { onMount } from "svelte";

    let gba = $gbaStore;
    let ppu_panel: string = "background";

    let palette: number = 0;
    let use_256_colors: boolean = false;

    let background: number = 0;

    //let bg_mode = $gba?.gba.background_mode();
    let bg_mode = 0;

    // Refresh ppu debug data every frame
    function refresh_tiles() {
        if (gba) {
            gba.request_tiles(use_256_colors ? undefined : palette);
        }
    }
    function refresh_palettes() {
        if (gba) {
            gba.request_palettes();
        }
    }
    function refresh_background() {
        if (gba) {
            gba.request_background(background);
        }
    }
    function refresh() {
        if (ppu_panel == "tiles") {
            refresh_tiles();
        } else if (ppu_panel == "palettes") {
            refresh_palettes();
        } else if (ppu_panel == "background") {
            refresh_background();
        }
    }
    onMount(() => {
        let id = runPeriodically(refresh, 60);
        return () => clearRunPeriodically(id);
    });

</script>

<div id="ppu-debugger">
    <label>
        <input type="radio" bind:group={ppu_panel} value={"tiles"}>
        Tiles
    </label>
    <label>
        <input type="radio" bind:group={ppu_panel} value={"palettes"}>
        Palettes
    </label>
    <label>
        <input type="radio" bind:group={ppu_panel} value={"background"}>
        Background
    </label>
    {#if ppu_panel === "tiles"}
        <div>
            <h2>
                Tiles
            </h2>
            <label>
                Palettes
                <input type="number" min=0 max=15 bind:value={palette}>
            </label>
            <label>
                256 Colours
                <input type="checkbox" bind:checked={use_256_colors}>
            </label>
            <TilesCanvas palette={palette} use_256_colors={use_256_colors} />
        </div>
    {:else if ppu_panel === "palettes"}
        <div>
            <h2>
                Palettes
            </h2>
            <PalettesCanvas />
        </div>
    {:else if ppu_panel === "background"}
        <div>
            <h2>Background</h2>
            <label>
                Background
                <input type="number" min=0 max=3 bind:value={background}>
            </label>
            <BackgroundsCanvas background={background} />
        </div>
    {/if}
</div>

