<script lang="ts">
	import { DISPLAYS } from "$lib/debuggerStore";
	import { clearRunPeriodically, runPeriodically } from "$lib/utils";
	import { onMount } from "svelte";

    export let palette: number;
    export let use_256_colors: boolean;

    let tiles_canvas: HTMLCanvasElement;

    $: ctx = tiles_canvas?.getContext('2d');

    // Used to cancel runPeriodically()
    let id: number;
    const width = 32 * 8;
    const height = 32 * 8;

    function refresh() {
        if (ctx) {
            let imageData = new ImageData(DISPLAYS.tiles, 32 * 8);
            ctx.putImageData(imageData, 0, 0);
        }
    }
    onMount(() => {
        id = runPeriodically(refresh, 60);
        return () => clearRunPeriodically(id);
    });
</script>

<canvas
    class="tiles-canvas"
    bind:this={tiles_canvas}
    style="image-rendering: pixelated; 
        --width: {width};
        --height: {height};
        "
    width={width}
    height={height}
/>

<style>
    .tiles-canvas {
        width: calc(2px * var(--width));
        height: calc(2px * var(--height));
        padding: 0.5em;
    }
</style>
