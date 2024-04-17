<script lang="ts">
	import type { LineData } from "$lib/debugger";
	import { gbaStore } from "$lib/gbaStore";
	import type { InstructionInfo } from "$lib/pkg/gba_web";
	import { clearRunPeriodically, runPeriodically } from "$lib/utils";
	import { onMount } from "svelte";

    export let line: LineData;
    export let lineHeight: number;
    export let instructionSize: number;
    //export let isExecuting: boolean;
    //export let isPc: boolean;
    export let isBreakpoint: boolean;
    export let toggleBreakpoint: (address: number) => void;

    let info: InstructionInfo | undefined = undefined;

    function refreshInstructionInfo() {
        if ($gbaStore) {
            $gbaStore.request_instruction_info(line.address);
        }
        info = $gbaStore?.instruction_info(line.address);
    }

    $: disassembly = (instructionSize == 4 ? info?.arm_dis : info?.thumb_dis) || ":(";
    //$: cls = `disassembly ${isExecuting ? 'executing' : ''} ${isPc ? 'pc' : ''}`;
    $: disassembly_class = 'disassembly';

    let value: number | undefined = undefined;
    $: if (instructionSize == 4) {
        value = info?.value;
    } else {
        if (info?.value !== undefined) {
            value = info.value & 0xffff;
        } else {
            value = undefined
        }
    }

    onMount(() => {
        let rid = runPeriodically(refreshInstructionInfo, 60);
        return () => clearRunPeriodically(rid);
    });
</script>

<tr style='height: {lineHeight}px'>
    <th 
        on:click={() => toggleBreakpoint(line.address)}
        class="
            {isBreakpoint ? 'breakpoint' : ''}
        "
    >
        {line.address.toString(16)}
    </th>
    <td class={disassembly_class}>{disassembly}</td>
    <td>{value?.toString(16).padStart(instructionSize * 2, '0')}</td>
</tr>

<style>
    .disassembly {
        min-width: 20em;
    }

    .breakpoint {
        background-color: red;
        color: white;
    }

    .pc {
        background-color: gray;
        color: white;
    }

    .executing {
        background-color: red;
        color: white;
    }
</style>
