import { get, writable } from 'svelte/store';
import { gbaStore } from './gbaStore';
import type { Gba } from './pkg/debug/gba_web';

interface DebuggerData {
	screen_array: Uint8ClampedArray;
}

// We will always use the same array
let screen_array = new Uint8ClampedArray(240 * 160 * 4);

const initialData: DebuggerData = {
	screen_array
};

export const debuggerStore = writable<DebuggerData>(initialData);

gbaStore.subscribe((gba) => {
	if (gba) {
		console.log('Setting screen array');
		gba.set_screen_array(screen_array);
	}
});

export function updateDebuggerData(gba: Gba) {
	gba.process_responses();
	// Assign to store

	// Request updates
	gba.request_screen_draw();
	gba.request_cpu_debug_info();
}