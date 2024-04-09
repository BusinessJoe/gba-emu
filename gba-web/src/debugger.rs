use js_sys::Uint8ClampedArray;
use wasm_bindgen::prelude::*;

// Stores references to JS arrays that are used to put display data into.
#[derive(Debug, Default)]
pub struct DebuggerDisplays {
    pub screen: Option<Uint8ClampedArray>,
    pub tiles: Option<Uint8ClampedArray>,
    pub palettes: Option<Uint8ClampedArray>,
    pub background: Option<Uint8ClampedArray>,
}

// Non-display debugger state which will be stored in a svelte store.
#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy)]
pub struct DebuggerState {
    pub instructions: InstructionsState,
    pub ppu: PpuState,
}

#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy)]
pub struct InstructionsState {

}

#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy)]
pub struct PpuState {
    pub bg_mode: u8,
    pub background_0: BackgroundsState,
    pub background_1: BackgroundsState,
    pub background_2: BackgroundsState,
    pub background_3: BackgroundsState,
}


#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy)]
pub struct BackgroundsState {
    pub priority: u8,
    pub map_base: usize,
    pub mosaic: bool,
    pub use_256_colors: bool,
    pub tile_base: usize,
    pub wraparound: bool,
    pub size_0: usize,
    pub size_1: usize,
    pub offset_0: usize, // maybe the same as wraparound?
    pub offset_1: usize, // maybe the same as wraparound?
}
