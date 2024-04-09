import { writable } from 'svelte/store';
import { Gba } from '$lib/pkg/gba_web';

export const gbaStore = writable<Gba | undefined>(undefined);
export const rom = writable<Uint8Array | undefined>(undefined);

export const reset = () => {
	/*
	gba.update((old) => {
		if (old) {
			old.gba.free();
			old.cpu.free();
		}

		const emu = new GbaCore();

		let rom_data = get(rom);
		if (!rom_data) {
			emu.load_test_rom();
		} else {
			emu.load_rom(rom_data);
		}
		emu.skip_bios();

		return {
			gba: emu,
			cpu: emu.inspect_cpu(),
			ppu: emu.inspect_ppu()
		};
	});
    */
};

export const tick = (numTicks: number) => {
	/*
	gba.update((details) => {
		if (document.hidden) {
			return details;
		}

		if (!details) {
			return details;
		}

		details.gba.tick_multiple(numTicks);

		details.cpu.free();
		details.ppu.free();

		return {
			gba: details.gba,
			cpu: details.gba.inspect_cpu(),
			ppu: details.gba.inspect_ppu()
		};
	});
*/
};

