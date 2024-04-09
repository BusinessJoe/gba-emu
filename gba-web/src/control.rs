use gba_core::Key;

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
    Background { bg: usize }
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
    },
}

