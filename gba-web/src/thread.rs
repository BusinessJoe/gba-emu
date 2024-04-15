use std::sync::mpsc::{Receiver, Sender};

use gba_core::GbaCore;

use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::control::{ControlState, Request, Response};
use crate::cpu_debug::CpuDebugInfo;
use crate::debugger::BackgroundsState;
use crate::to_js_result::ToJsResult;

pub struct GbaThread {
    gba: GbaCore,

    tx: Sender<Response>,
    rx: Receiver<Request>,

    control_state: ControlState,
}

impl GbaThread {
    pub fn new(tx: Sender<Response>, rx: Receiver<Request>) -> Self {
        // Emulator instance
        let gba = GbaCore::default();
        console::log_1(&"Constructed a Gba".into());

        Self {
            gba,
            tx,
            rx,
            control_state: ControlState::new(),
        }
    }

    pub fn start(&mut self) -> Result<(), JsValue> {
        // let worker = worker_global_scope()?;
        // let performance = worker
        // .performance()
        // .to_js_result("performance should be available")?;

        let ticks = 100000;

        self.gba.load_test_rom();
        self.gba.skip_bios();

        loop {
            for event in self.rx.try_iter() {
                match event {
                    Request::ControlEvent(event) => {
                        self.control_state.update(event);
                    }
                    Request::LoadRom(rom) => {
                        self.gba = GbaCore::default();
                        self.gba.load_rom(&rom);
                        self.gba.skip_bios();
                    }
                    Request::KeyEvent { key, pressed } => {
                        self.gba.set_key(key, pressed);
                    }
                    Request::ScreenData => {
                        let data = self.gba.screen();
                        self.tx.send(Response::ScreenData(data)).to_js_result()?;
                    }
                    Request::CpuDebugInfo => {
                        let pc = self.gba.pc();
                        let info = CpuDebugInfo { pc };
                        self.tx.send(Response::CpuDebugInfo(info)).to_js_result()?;
                    }
                    Request::Tiles { palette } => {
                        let tiles = self.gba.get_tiles(palette);
                        self.tx.send(Response::TileData(tiles));
                    }
                    Request::Palettes => {
                        let palettes = self.gba.get_palettes();
                        self.tx.send(Response::PaletteData(palettes));
                    }
                    Request::Background { bg } => {
                        // Don't justify bad parameters with a response >:(
                        if bg >= 4 {
                            continue;
                        }

                        let bg_info = self.gba.background_info(bg as u32);

                        let bg_mode = self.gba.background_mode();

                        let size = match bg_info.screen_size {
                            0 => (128, 128),
                            1 => (256, 256),
                            2 => (512, 512),
                            3 => (1024, 1024),
                            _ => unreachable!("Screen size is always from 0 to 3"),
                        };

                        let mut display: Vec<[u8; 3]> = vec![[0, 0, 0]; size.0 * size.1];
                        for y in 0..size.0 {
                            for x in 0..size.1 {
                                display[x * size.1 + y] = self.gba.get_background_pixel(
                                    x.try_into().unwrap(),
                                    y.try_into().unwrap(),
                                    bg,
                                )
                            }
                        }

                        let response = Response::BackgroundData {
                            bg,
                            bg_mode,
                            data: BackgroundsState {
                                priority: bg_info.priority,
                                mosaic: bg_info.mosaic,
                                use_256_colors: bg_info.use_256_colors,
                                map_base: bg_info.screen_base_block, // TODO: add address offset
                                tile_base: bg_info.character_base_block, // TODO: same as above
                                wraparound: false,
                                size_0: size.0,
                                size_1: size.1,
                                offset_0: 0,
                                offset_1: 0,
                            },
                            display,
                        };
                        self.tx.send(response);
                    }
                }
            }

            if self.control_state.pause {
                continue;
            }

            self.gba.tick_multiple(ticks);
        }
    }
}

/*
fn worker_global_scope() -> Result<WorkerGlobalScope, JsValue> {
    let global = js_sys::global();
    // how to properly detect this in wasm_bindgen?
    if js_sys::eval("typeof WorkerGlobalScope !== 'undefined'")?.as_bool().unwrap() {
        Ok(global.dyn_into::<WorkerGlobalScope>()?)
    } else {
        Err("Not in worker".into())
    }
}
*/
