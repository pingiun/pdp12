use crate::{memory::Memory, consts::{MASK_12BIT}, eight_mode};

pub struct PDP12 {
    pub memory: Memory,
    generations: Vec<(State, usize)>,
    generation: usize,
    devices: [Option<Box<dyn Device>>; 64],
}

impl PDP12 {
    pub fn new(state: State, memory: Memory) -> Self {
        const INIT: Option<Box<dyn Device>> = None;
        Self {
            memory,
            generations: vec![(state, 0)],
            generation: 0,
            devices: [INIT; 64]
        }
    }

    pub fn get_state(&self) -> (State, &Memory) {
        (self.generations.last().unwrap().0, &self.memory)
    }

    pub fn change_state(&mut self, f: impl FnOnce(State, &mut Memory) -> State) -> Result<(), ()> {
        if self.generation != self.generations.len() - 1 {
            return Err(());
        }
        let newstate = f(self.generations.last().unwrap().0, &mut self.memory);
        self.generations.push((newstate, self.memory.generation()));
        self.generation = self.generations.len() - 1;
        Ok(())
    }

    pub fn step(&mut self) {
        if self.generation == self.generations.len() - 1 {
            let newstate = step(self.generations.last().unwrap().0, &mut self.memory);
            self.generations.push((newstate, self.memory.generation()));
            self.generation = self.generations.len() - 1;
        } else {
            self.generation += 1;
            self.memory.apply(self.generations[self.generation].1);
        }
    }

    pub fn step_back(&mut self) {
        if self.generation > 1 {
            self.memory.unapply(self.generations[self.generation].1);
            self.generation -= 1;
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct State {
    /// PDP-12 accumulator, the lower 12 A form the PDP-12 accumulator
    /// xxxx AAA AAA AAA AAA
    pub acc: u16,
    pub link: bool,
    pub pc: u16,
    // Full instruction, top 3 bits are instruction register
    pub mri: u16,
    // Memory address register
    pub mra: u16,
    // Memory buffer register
    pub mrb: u16,

    // Left switches
    pub lsw: u16,
    // Right switches
    pub rsw: u16,

    pub running: bool,
}

pub fn fetch(state: State, memory: &mut Memory) -> (u16, State) {
    let instr = memory.read(state.pc);
    (
        instr,
        State {
            pc: (state.pc + 1) & MASK_12BIT, // When 13th bit is set (wrapped around) clear back to 0
            mri: instr,
            ..state
        },
    )
}

#[must_use]
pub fn step(state: State, memory: &mut Memory) -> State {
    let (instr, state) = fetch(state, memory);
    eight_mode::exec(instr, state, memory)
}

trait Device<T> {
    const SELECTOR: u8;

    fn iot(&mut self, state: State, memory: &mut Memory) -> State;

    fn get_info(&mut self) -> T;
    fn write_info(&mut self, info: T);
}

struct Keyboard {
    buffer: u8,
}

impl Device<u8> for Keyboard {
    const SELECTOR: u8 = 0b00_000_000;

    fn iot(&mut self, state: State, memory: &mut Memory) -> State {
        todo!()
    }

    fn get_info(&mut self) -> u8 {
        self.buffer
    }

    fn write_info(&mut self, info: u8) {
        self.buffer = info;
    }
}
