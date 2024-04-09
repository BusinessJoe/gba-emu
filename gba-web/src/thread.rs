use std::sync::mpsc::{Sender, Receiver};

use gba_core::GbaCore;

use wasm_bindgen::prelude::*;
use web_sys::{console, WorkerGlobalScope};

use crate::cpu_debug::CpuDebugInfo;
use crate::to_js_result::{ToJsResult, OptionToJsResult};
use crate::control::{Request, ControlState, Response};

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
                        let info = CpuDebugInfo {
                            pc
                        };
                        self.tx.send(Response::CpuDebugInfo(info)).to_js_result()?;
                    }
                    Request::Tiles { palette } => {
                        let tiles = self.gba.get_tiles(palette);
                        self.tx.send(Response::TileData(tiles));
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
