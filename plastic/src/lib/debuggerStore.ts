import { writable } from 'svelte/store';
import { gbaStore } from './gbaStore';
import type { Gba } from './pkg/gba_web';

// Shared arrays between wasm and js.
// Our wasm module will write to these.
export const DISPLAYS = {
	screen: new Uint8ClampedArray(240 * 160 * 4),
	palettes: new Uint8ClampedArray(16 * 16 * 4),
	tiles: new Uint8ClampedArray(32 * 64 * 8 * 8 * 4)
};

interface BackgroundData {
	priority: number;
	map_base: number;
	tile_base: number;
    mosaic: boolean;
    use_256_colors: boolean;
    size_0: number;
    size_1: number;
}

const initialBackground: BackgroundData = {
	priority: 0,
	map_base: 0,
	tile_base: 0,
    mosaic: false,
    use_256_colors: false,
    size_0: 0,
    size_1: 0,
};

interface DebuggerState {
    instructions: {
    },
    ppu: {
        bg_mode: number,
        background_0: BackgroundData,
        background_1: BackgroundData,
        background_2: BackgroundData,
        background_3: BackgroundData,
    }
}

const initialData: DebuggerState = {
    instructions: {

    },
    ppu: {
        bg_mode: 0,
        background_0: initialBackground,
        background_1: initialBackground,
        background_2: initialBackground,
        background_3: initialBackground,
    }
};

export const debuggerStore = writable<DebuggerState>(initialData);

gbaStore.subscribe((gba) => {
	if (gba) {
		console.log('Setting screen array');
		gba.set_display('screen', DISPLAYS.screen);
		gba.set_display('tiles', DISPLAYS.tiles);
		gba.set_display('palettes', DISPLAYS.palettes);
	}
});

export function updateDebuggerData(gba: Gba) {
	gba.process_responses();
    const state = gba.debugger_state();
	debuggerStore.set(state);

	// Request updates that need to happen each frame
	// The responsibility of requesting other updates lies with
	// the corresponding components.
	gba.request_screen_draw();
}
