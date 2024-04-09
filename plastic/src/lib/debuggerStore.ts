import { writable } from 'svelte/store';
import { gbaStore } from './gbaStore';
import type { Gba } from './pkg/gba_web';

interface DebuggerData {

}

export const DISPLAYS = {
	screen: new Uint8ClampedArray(240 * 160 * 4),
	palettes: new Uint8ClampedArray(16 * 16 * 4),
    tiles: new Uint8ClampedArray(32 * 64 * 8 * 8 * 4),
};

const initialData: DebuggerData = {
};

export const debuggerStore = writable<DebuggerData>(initialData);

gbaStore.subscribe((gba) => {
	if (gba) {
		console.log('Setting screen array');
        gba.set_display("screen", DISPLAYS.screen);
        gba.set_display("tiles", DISPLAYS.tiles);
        gba.set_display("palettes", DISPLAYS.palettes);
	}
});

export function updateDebuggerData(gba: Gba) {
	gba.process_responses();
    debuggerStore.set(gba.debugger_state());
	// Assign to store

	// Request updates that need to happen each frame
    // The responsibility of requesting other updates lies with
    // the corresponding components.
	gba.request_screen_draw();
}
