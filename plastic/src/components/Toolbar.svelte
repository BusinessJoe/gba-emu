<script lang="ts">
    import { frameTimes } from '$lib/frameTimeStore';
    import { gbaStore, rom, reset, tick } from '$lib/gbaStore';

    export let clockSpeed: number = 8000000;

    let files: FileList;
    $: gba = $gbaStore;
    $: averageFrameTime = $frameTimes.buffer.reduce((acc, x) => acc + x, 0) / $frameTimes.buffer.length;

    const resume = () => {
        /*
        gba?.gba.enable_debugger(false);
        gba?.gba.set_stopped(false);
        tick(1);
        gba?.gba.enable_debugger(true);
        */
    }

    const step = () => {
        /*
        gba?.gba.enable_debugger(false);
        gba?.gba.set_stopped(false);
        tick(1);
        gba?.gba.set_stopped(true);
        gba?.gba.enable_debugger(true);
        gba = gba;
        */
    }

    const handleReset = () => {
        /*
        let arm_breakpoints = Array.from(gba?.gba.arm_breakpoints() ?? []);
        let thumb_breakpoints = Array.from(gba?.gba.thumb_breakpoints() ?? []);
        reset();
        for (const bp of arm_breakpoints) {
            gba?.gba.add_arm_breakpoint(bp);
        }
        for (const bp of thumb_breakpoints) {
            gba?.gba.add_thumb_breakpoint(bp);
        }
        */
    }

    const printPcHistory = () => {
        /*
        const pcs = gba?.gba.pc_history();
        if (!pcs) return;

        const hexs: string[] = Array.from(pcs).map((val) => val.toString(16));
        console.log(hexs);
        */
    }

    $: if (files && files[0]) {
        files[0].arrayBuffer().then((array) => {
            let bytes = new Uint8Array(array);
            if (gba) {
                gba.load_rom(bytes);
            }
            $rom = bytes;
            reset();
        })
    }
</script>

<div id="toolbar" >
    <input type="file" bind:files/>
    <button on:click={handleReset}>Reset</button>
    <button on:click={resume}>Resume</button>
    <button on:click={step}>Step</button>
    <label>
        Clock speed (hz):
        <input type="number" bind:value={clockSpeed} />
    </label>
    <span>Average millis/frame: {averageFrameTime.toFixed(2)}</span>
    <!--<span>PC: 0x{gba?.cpu.pc().toString(16)}</span>-->
    <!--<span>Thumb: {gba?.gba.thumb_state()}</span>-->
    <button on:click={printPcHistory}>PC history</button>
</div>

<style>
    #toolbar {
        width: 100%;
        height: 100px;
        background-color: #295379;
        color: white;
    }
</style>
