use gba_core::Key;
use wasm_bindgen::prelude::*;

use crate::{cpu_debug::CpuDebugInfo, debugger::BackgroundsState};

/// Events from controller to GBA thread
pub enum Request {
    ControlEvent(ControlEvent),
    LoadRom(Vec<u8>),
    ScreenData,
    CpuDebugInfo,
    KeyEvent{key: Key, pressed: bool},
    /// Tile display with specified palette, or in 256 color mode
    Tiles{ palette: Option<usize> },
    /// Palette display for all 16 palettes
    Palettes,
    /// Background data and display
    Background { bg: usize },
    Instruction { addr: u32 },
}

pub enum ControlEvent {
    Pause(bool) 
}

pub struct ControlState {
    pub pause: bool,
    pub tick_rate: u32,
}

impl ControlState {
    pub fn new() -> Self {
        Self {
            pause: false,
            tick_rate: 16_780_000, // 16.78 Mhz
        }
    }

    pub fn update(&mut self, event: ControlEvent) {
        match event {
            ControlEvent::Pause(pause) => {
                self.pause = pause
            }
        }
    }
}

/// Stores the colors of a single tile.
type Tile = Vec<[u8; 3]>;

/// Information about a single instruction
#[wasm_bindgen]
#[derive(Clone)]
pub struct InstructionInfo {
    pub value: u32,
    #[wasm_bindgen(skip)]
    pub arm_dis: String,
    #[wasm_bindgen(skip)]
    pub thumb_dis: String,
}

#[wasm_bindgen]
impl InstructionInfo {
    #[wasm_bindgen(getter)]
    pub fn arm_dis(&self) -> String {
        self.arm_dis.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn thumb_dis(&self) -> String {
        self.thumb_dis.clone()
    }
}

/// Responses from GBA thread to controller
pub enum Response {
    ScreenData(Vec<u8>),
    CpuDebugInfo(CpuDebugInfo),
    /// All the current tiles
    TileData(Vec<Tile>),
    /// Colors of each palette; 16 palettes with 16 colors each.
    PaletteData(Vec<Vec<[u8; 3]>>),
    BackgroundData { 
        /// The background this data belongs to
        bg: usize,
        bg_mode: u8,
        data: BackgroundsState,
        display: Vec<[u8; 3]>,
    },
    InstructionInfo {
        addr: u32,
        info: InstructionInfo,
    },
}

