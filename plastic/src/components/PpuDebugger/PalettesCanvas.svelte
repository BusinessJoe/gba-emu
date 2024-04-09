<script lang="ts">
	import { DISPLAYS } from "$lib/debuggerStore";
	import { gbaStore } from "$lib/gbaStore";
	import { clearRunPeriodically, runPeriodically } from "$lib/utils";
	import { onMount } from "svelte";

    let canvas: HTMLCanvasElement;
    let gba = $gbaStore;

    $: ctx = canvas?.getContext('2d');

    const width = 16;
    const height = 16;

    function refresh() {
        if (ctx) {
            let imageData = new ImageData(DISPLAYS.palettes, 16);
            ctx.putImageData(imageData, 0, 0);
        }
    }

    let id: number;
    onMount(() => {
        id = runPeriodically(refresh, 60);
        return () => clearRunPeriodically(id);
    });

    $: {
        if (ctx && gba) {
            //$gba.gba.draw_palettes(ctx)
        }
    }
</script>

<canvas
    class="palette-canvas"
    bind:this={canvas}
    style="image-rendering: pixelated; 
        --width: {width};
        --height: {height};
        "
    width={width}
    height={height}
/>

<style>
    .palette-canvas {
        width: calc(8px * var(--width));
        height: calc(8px * var(--height));
        padding: 0.5em;
    }
</style>
