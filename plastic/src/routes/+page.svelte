<script lang="ts">
	import { gbaStore } from '$lib/gbaStore';
	import { handleKey } from '$lib/keys';
	import Debugger from '../components/Debugger/Debugger.svelte';
	import PpuDebugger from '../components/PpuDebugger/PpuDebugger.svelte';
	import EmuInfo from '../components/EmuInfo.svelte';
	import GbaTicker from '../components/GbaTicker.svelte';
	import Screen from '../components/Screen.svelte';
	import Toolbar from '../components/Toolbar.svelte';

	let clockSpeed: number;
	let leftPanel: string = 'instructions';

	$: gba = $gbaStore;

	function selectLeftPanel(event: any) {
		leftPanel = event.currentTarget?.value;
	}
</script>

<GbaTicker />
<Toolbar bind:clockSpeed />
<div id="main">
	<div>
		<div id="left-panel-select">
			<label>
				<input
					checked={leftPanel === 'instructions'}
					on:change={selectLeftPanel}
					type="radio"
					name="panel"
					value="instructions"
				/>
				Instructions
			</label>
			<label>
				<input
					checked={leftPanel === 'ppu'}
					on:change={selectLeftPanel}
					type="radio"
					name="panel"
					value="ppu"
				/>
				PPU
			</label>
		</div>
		{#if leftPanel === 'instructions'}
			<Debugger />
		{:else if leftPanel === 'ppu'}
			<PpuDebugger />
		{/if}
	</div>
	<div class="column">
		<div>
			<EmuInfo />
		</div>
		<div class="row">
			<div
				id="screen-wrapper"
                role="input"
				on:keydown={(evt) => handleKey(gba, evt, true)}
				on:keyup={(evt) => handleKey(gba, evt, false)}
				tabindex={0}
			>
				<Screen />
			</div>
		</div>
	</div>
</div>

<style>
	:global(body) {
		background-color: #0e2a3a;
		margin: 0;
		height: 100vh;
		display: flex;
		flex-direction: column;
		font-family: monospace;
		color: white;
	}

	#main {
		flex-grow: 1;
		display: flex;
		position: relative;
		flex-direction: row;
		align-items: stretch;
		/* Prevents overflow */
		min-height: 0;
	}

	#screen-wrapper {
		flex-grow: 1;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	#left-panel-select {
		color: white;
	}

	#left-panel-select {
		max-height: 100%;
	}

	.column {
		display: flex;
		flex-direction: column;
		width: 100%;
	}

	.row {
		display: flex;
		flex-direction: row;
	}
</style>
