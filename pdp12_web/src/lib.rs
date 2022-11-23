use core::slice;

use pdp12_emulator::{Memory, MASK_12BIT, PDP12};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Machine {
    machine: pdp12_emulator::PDP12,
}

#[wasm_bindgen]
impl Machine {
    #[wasm_bindgen(constructor)]
    pub fn new(memory: &[u16]) -> Self {
        let mut code = [0u16; 4096];
        // Safety: we write the javascript ourself and shouldn't supply buffers less than 4096 u16s.
        code.copy_from_slice(memory);
        let memory = Memory::with_code(code);
        setLightBit("8_mode", true);
        setLightBit("linc_mode", false);
        log(&format!("First word of memory: {}", memory.read(0)));
        Self {
            machine: PDP12::new(Default::default(), memory),
        }
    }

    #[wasm_bindgen]
    pub fn run_one_frame(&mut self) {
        setLightBit("run", true);
        let time = getNow(true);
        let mut count: u64 = 0;
        while getNow(true) - time < 15.0 {
            self.machine.step();
            count += 1;
        }
        let (state, _) = self.machine.get_state();
        setLightBits("progCount", state.pc);
        log(&format!("ran for {} steps", count));
    }

    #[wasm_bindgen]
    pub fn examine(&mut self, lsw: u16, step: bool) {
        let _ = self.machine.change_state(|mut state, memory| {
            if !step {
                state.mra = lsw & MASK_12BIT;
            } else {
                state.mra = (state.mra + 1) & MASK_12BIT;
            }
            state.lsw = lsw;
            setLightBits("memAddr", state.mra);
            setLightBits("memBuf", memory.read(state.mra));
            state
        });
    }

    #[wasm_bindgen]
    pub fn fill(&mut self, lsw: u16, rsw: u16, step: bool) {
        let _ = self.machine.change_state(|mut state, memory| {
            state.lsw = lsw & MASK_12BIT;
            state.rsw = rsw & MASK_12BIT;
            if !step {
                state.mra = lsw & MASK_12BIT;
            } else {
                state.mra = (state.mra + 1) & MASK_12BIT;
            }
            memory.write(state.mra, state.rsw);
            setLightBits("memAddr", state.mra);
            setLightBits("memBuf", memory.read(state.mra));
            state
        });
    }

    #[wasm_bindgen]
    pub fn key_do(&mut self, lsw: u16) {
        let instr = lsw & MASK_12BIT;
        let _ = self.machine.change_state(|mut state, memory| {
            state.mri = instr;
            state.lsw = instr;
            let state = pdp12_emulator::eight_mode::exec(instr, state, memory);
            setLightBit("link", state.link);
            setLightBits("acc", state.acc);
            setLightBits("instrReg", state.mri);
            state
        });
    }

    #[wasm_bindgen]
    pub fn dump_memory(&self) -> *const u16 {
        self.machine.memory.dump()
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "uiFunctions"])]
    fn setLightBits(s: &str, bits: u16);

    #[wasm_bindgen(js_namespace = ["window", "uiFunctions"])]
    fn setLightBit(s: &str, val: bool);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = ["window", "uiFunctions"])]
    fn getNow(dummy: bool) -> f64;
}

#[wasm_bindgen]
pub fn greet(on: bool) {
    if on {
        setLightBits("acc", 0b0000_010_101_010_101);
    } else {
        setLightBits("acc", 0b0000_101_010_101_010);
    }
    // sleep(Duration::from_millis(20));
    // setLightBits("acc", 0b0000_101_101_101_101);
    // sleep(Duration::from_millis(20));
}
