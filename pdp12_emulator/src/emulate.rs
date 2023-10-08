use crate::{
    consts::MASK_12BIT,
    devices::{Device, Devices, Keyboard, Tty},
    eight_mode,
    memory::Memory,
};

pub struct PDP12 {
    pub memory: Memory,
    generations: Vec<(State, usize)>,
    generation: usize,
    devices: Devices,
}

impl Default for PDP12 {
    fn default() -> Self {
        let mut this = Self::new(Default::default(), Default::default());
        this.register_device(Keyboard::new());
        this.register_device(Tty::new());
        this
    }
}

impl PDP12 {
    pub fn new(state: State, memory: Memory) -> Self {
        Self {
            memory,
            generations: vec![(state, 0)],
            generation: 0,
            devices: Default::default(),
        }
    }

    pub fn register_device<D: Device>(&mut self, device: D) {
        assert!(
            device.get_selector() < 64,
            "Selector must be a 6-bit number"
        );
        let selector = device.get_selector();
        self.devices[selector as usize] = Some(Box::new(device));
    }

    pub fn operate_device<F, D>(&mut self, selector: u8, operate: F) -> Result<(), ()>
    where
        F: FnOnce(&mut D),
        D: Device + 'static,
    {
        let device = self.devices[selector as usize].as_mut().ok_or(())?;
        let device = device.as_mut().downcast_mut::<D>().ok_or(())?;
        operate(device);
        Ok(())
    }

    pub fn get_state(&self) -> (State, &Memory) {
        (self.generations.last().unwrap().0, &self.memory)
    }

    pub fn change_state(&mut self, f: impl FnOnce(State, &mut Memory, &mut Devices) -> State) -> Result<(), ()> {
        if self.generation != self.generations.len() - 1 {
            return Err(());
        }
        let newstate = f(self.generations.last().unwrap().0, &mut self.memory, &mut self.devices);
        self.generations.push((newstate, self.memory.generation()));
        self.generation = self.generations.len() - 1;
        Ok(())
    }

    pub fn step(&mut self) {
        if self.generation == self.generations.len() - 1 {
            let newstate = step(
                self.generations.last().unwrap().0,
                &mut self.memory,
                &mut self.devices,
            );
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
pub fn step(state: State, memory: &mut Memory, devices: &mut Devices) -> State {
    let (instr, state) = fetch(state, memory);
    eight_mode::exec(instr, state, memory, devices)
}
