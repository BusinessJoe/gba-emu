<script lang="ts">
	import { gbaStore } from "$lib/gbaStore";
    import { debuggerStore } from "$lib/debuggerStore";

    export let background: number;

    let canvas: HTMLCanvasElement;
    let gba = $gbaStore;

    $: ctx = canvas?.getContext('2d');
    $: background_info = [
        $debuggerStore.ppu.background_0,
        $debuggerStore.ppu.background_1,
        $debuggerStore.ppu.background_2,
        $debuggerStore.ppu.background_3
    ][background];

    const width = 256;
    const height = 256;

</script>

<div>
    BG Mode: {$debuggerStore.ppu.bg_mode}
</div>

<div>
    Priority: {background_info.priority}<br>
    Character base block: {background_info.tile_base}<br>
    Mosaic: {background_info.mosaic}<br>
    Use 256 colors: {background_info.use_256_colors}<br>
    Screen base block: {background_info.map_base}<br>
    Wraparound: {background_info.wraparound}<br>
    Screen size: {background_info.size_0}x{background_info.size_1}
</div>

<canvas
    class="background-canvas"
    bind:this={canvas}
    style="image-rendering: pixelated; 
        --width: {width};
        --height: {height};
        "
    width={width}
    height={height}
/>

<style>
    .background-canvas {
        width: calc(2px * var(--width));
        height: calc(2px * var(--height));
        padding: 0.5em;
    }
</style>
