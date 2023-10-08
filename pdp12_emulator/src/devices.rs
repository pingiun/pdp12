use std::ops::{Index, IndexMut};

use downcast_rs::{impl_downcast, Downcast};

use crate::{emulate::State, Memory, KEYBOARD_SELECTOR, MASK_12BIT, TTY_SELECTOR};

pub struct Devices([Option<Box<dyn Device>>; 64]);

impl Default for Devices {
    fn default() -> Self {
        const INIT: Option<Box<dyn Device>> = None;
        Self([INIT; 64])
    }
}

impl Devices {
    pub fn new_with_asr33() -> Self {
        let mut this = Self::default();
        this[KEYBOARD_SELECTOR as usize] = Some(Box::new(Keyboard::new()));
        this[TTY_SELECTOR as usize] = Some(Box::new(Tty::new()));
        this
    }
}

impl Index<usize> for Devices {
    type Output = Option<Box<dyn Device>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Devices {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

pub trait Device: Downcast {
    fn get_selector(&self) -> u8;
    fn iot(&mut self, instr: u16, state: State, memory: &mut Memory) -> State;
}
impl_downcast!(Device);

pub struct Keyboard {
    tti: u8,
    ready: bool,
}

impl Keyboard {
    pub fn new() -> Self {
        Self { tti: 0, ready: false }
    }

    pub fn set_key(&mut self, key: u8) {
        self.tti = key;
        self.ready = true;
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Keyboard {
    fn get_selector(&self) -> u8 {
        KEYBOARD_SELECTOR
    }

    fn iot(&mut self, instr: u16, state: State, memory: &mut Memory) -> State {
        let instr = instr & 0b0000_000_000_000_111;
        let mut state = state;
        if instr & 0b001 > 0 {
            // KSF
            if self.ready {
                state.pc = (state.pc + 1) & MASK_12BIT;
            }
        }
        if instr & 0b010 > 0 {
            // KCC
            state.acc = 0;
            self.ready = false;
        }
        if instr & 0b100 > 0 {
            // KRS
            state.acc = self.tti as u16;
            self.ready = false;
        }
        state
    }
}

pub struct Tty {
    tto: Option<u8>,
    ready: bool,
}

impl Tty {
    pub fn new() -> Self {
        Self { tto: None, ready: false }
    }

    pub fn get_key(&mut self) -> Option<u8> {
        self.ready = true;
        self.tto.take()
    }
}

impl Default for Tty {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Tty {
    fn get_selector(&self) -> u8 {
        TTY_SELECTOR
    }

    fn iot(&mut self, instr: u16, state: State, memory: &mut Memory) -> State {
        let instr = instr & 0b0000_000_000_000_111;
        let mut state = state;
        if instr & 0b001 > 0 {
            // TSF
            if self.ready {
                state.pc = (state.pc + 1) & MASK_12BIT;
            }
        }
        if instr & 0b010 > 0 {
            // TCF
            self.ready = false;
        }
        if instr & 0b100 > 0 {
            // TPC
            self.tto = Some(state.acc as u8);
        }
        state
    }
}
