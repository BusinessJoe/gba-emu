use js_sys::Uint8ClampedArray;
use wasm_bindgen::prelude::*;

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
    pub tiles: TilesState,
    pub palettes: PalettesState,
    pub background: BackgroundsState,
}

#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy)]
pub struct TilesState  {
    pub palette_num: u8,
    pub use_256_colors: bool,
}

#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy)]
pub struct PalettesState {
}

#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy)]
pub struct BackgroundsState {
    pub bg_mode: u8,
    pub bg_num: u8,
}

#[derive(Debug, Default)]
pub struct DebuggerDisplays {
    pub screen: Option<Uint8ClampedArray>,
    pub tiles: Option<Uint8ClampedArray>,
    pub palette: Option<Uint8ClampedArray>,
    pub background: Option<Uint8ClampedArray>,
}
