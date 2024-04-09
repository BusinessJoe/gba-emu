use gba_core::Key;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use js_sys::Uint8ClampedArray;
use wasm_bindgen::prelude::*;

use web_sys::console;

use crate::control::{ControlEvent, Request, Response};
use crate::debugger::{DebuggerDisplays, DebuggerState};
use crate::thread::GbaThread;
use crate::to_js_result::ToJsResult;

#[wasm_bindgen]
/// Gameboy with debugger
pub struct Gba {
    tx: Sender<Request>,
    rx: Receiver<Response>,

    displays: DebuggerDisplays,

    debugger_state: DebuggerState,
}

#[wasm_bindgen]
impl Gba {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Gba {
        console_error_panic_hook::set_once();
        console::log_1(&"Gba controller constructor".into());

        let (to_thread, from_control) = mpsc::channel();
        let (to_control, from_thread) = mpsc::channel();

        let _join_handle = rayon::spawn(move || {
            console::log_1(&"Hello from web worker".into());
            let mut gba_thread = GbaThread::new(to_control, from_control);
            gba_thread.start().unwrap();
        });

        console::log_1(&format!("Join handle: {:?}", _join_handle).into());

        Gba {
            tx: to_thread,
            rx: from_thread,
            displays: DebuggerDisplays::default(),
            debugger_state: DebuggerState::default(),
        }
    }

    /// Load a rom
    pub fn load_rom(&self, rom: Vec<u8>) -> Result<(), JsValue> {
        self.tx.send(Request::LoadRom(rom)).to_js_result()
    }

    pub fn set_key(&mut self, key: Key, pressed: bool) -> Result<(), JsValue> {
        self.tx
            .send(Request::KeyEvent { key, pressed })
            .to_js_result()
    }

    /// Pause the GBA execution
    pub fn set_pause(&self, pause: bool) -> Result<(), JsValue> {
        self.tx
            .send(Request::ControlEvent(ControlEvent::Pause(pause)))
            .to_js_result()
    }

    pub fn set_display(&mut self, name: &str, array: Uint8ClampedArray) -> Result<(), JsValue> {
        match name {
            "screen" => {
                self.displays.screen = Some(array);
            }
            "tiles" => {
                self.displays.tiles = Some(array);
            }
            "palettes" => {
                self.displays.palettes = Some(array);
            }
            _ => return Err("Invalid display name".into())
        }
        console::log_1(&format!("Set display for '{}'", name).into());
        Ok(())
    }

    pub fn request_screen_draw(&self) -> Result<(), JsValue> {
        self.tx.send(Request::ScreenData).to_js_result()
    }

    pub fn request_cpu_debug_info(&self) -> Result<(), JsValue> {
        self.tx.send(Request::CpuDebugInfo).to_js_result()
    }

    pub fn request_tiles(&self, palette: Option<usize>) -> Result<(), JsValue> {
        self.tx
            .send(Request::Tiles { palette })
            .to_js_result()
    }

    pub fn request_palettes(&self) -> Result<(), JsValue> {
        self.tx
            .send(Request::Palettes)
            .to_js_result()
    }

    pub fn process_responses(&mut self) -> Result<(), JsValue> {
        for response in self.rx.try_iter() {
            match response {
                Response::ScreenData(screen_data) => {
                    let js_screen_data: Vec<u8> = screen_data
                        .chunks_exact(3)
                        .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                        .collect();
                    if let Some(screen) = &mut self.displays.screen {
                        screen.copy_from(&js_screen_data);
                    }
                }
                Response::CpuDebugInfo(info) => {}
                Response::TileData(tiles) => {
                    // Hardcoded 64 * 32 tile canvas.
                    // Each tile is 8 * 8 pixels with 4 bytes per pixel.
                    let mut js_screen_data: Vec<u8> = vec![0; 64 * 32 * 8 * 8 * 4];

                    for (meta_row_idx, meta_row) in tiles.chunks(32).enumerate() {
                        for (meta_col_idx, tile) in meta_row.iter().enumerate() {
                            for (sub_row_idx, pixel_row) in tile.chunks(8).enumerate() {
                                let row_idx = meta_row_idx * 8 + sub_row_idx;
                                let pixel_row_flat: Vec<u8> = pixel_row.into_iter().flat_map(|&[r, g, b]| [r, g, b, 255]).collect();
                                let start_pixel: usize = row_idx * 32 * 8 + meta_col_idx * 8;
                                // Each row of pixels is 8 pixel * 4 bytes/pixel = 32 bytes
                                js_screen_data[start_pixel * 4..start_pixel * 4 + 32].copy_from_slice(&pixel_row_flat);
                            }
                        }
                    }


                    if let Some(screen) = &mut self.displays.tiles {
                        screen.copy_from(&js_screen_data);
                    }
                }
                Response::PaletteData(palette_data) => {
                    let js_screen_data: Vec<u8> = palette_data
                        .iter()
                        .flatten()
                        .flat_map(|&[r,g,b]| [r, g, b, 255])
                        .collect();
                    if let Some(screen) = &mut self.displays.palettes {
                        screen.copy_from(&js_screen_data);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn debugger_state(&self) -> DebuggerState {
        self.debugger_state.clone()
    }
}
