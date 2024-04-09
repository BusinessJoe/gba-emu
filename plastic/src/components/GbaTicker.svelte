<script lang="ts">
    import initWasm, { Gba, initThreadPool } from '$lib/pkg/gba_web';
	import { gbaStore } from '$lib/gbaStore';
    import { debuggerStore, updateDebuggerData } from '$lib/debuggerStore';
	import { onMount } from 'svelte';

    async function init() {
        console.log('Running init()');
        await initWasm();
        await initThreadPool(1);
        console.log('WASM initialized');
        let gba = new Gba();
        console.log('Gba controller created');
        console.log(gba);
        gbaStore.set(gba);
        debuggerStore.set(gba.debugger_state());
    };

    $: gba = $gbaStore;

    let rid: number;
    function tickGba() {
        if (gba) {
            updateDebuggerData(gba);
        }
        rid = requestAnimationFrame(tickGba);
    }

	onMount(() => {
        init().then(() => {
            rid = requestAnimationFrame(tickGba);
        });
	});
</script>
