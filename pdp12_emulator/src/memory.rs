use crate::consts::{MASK_MSDIGIT, MASK_CURRENT_PAGE, MASK_12BIT};

#[derive(Clone, Copy, Debug)]
pub struct MemoryChange {
    addr: u16,
    was: u16,
    now: u16,
}

#[derive(Debug)]
pub struct Memory {
    current: [u16; 4096],
    operations: Vec<MemoryChange>,
}

impl Memory {
    pub fn with_code(code: [u16; 4096]) -> Self {
        Self {
            current: code,
            operations: vec![],
        }
    }

    pub(crate) fn generation(&self) -> usize {
        self.operations.len()
    }

    pub fn read(&self, addr: u16) -> u16 {
        self.current[(addr & MASK_12BIT) as usize] // always safe because of mask
    }

    pub fn write(&mut self, addr: u16, value: u16) {
        let was = self.read(addr);
        self.operations.push(MemoryChange {
            addr,
            was,
            now: value,
        });
        self.current[(addr & MASK_12BIT) as usize] = value;
    }

    pub(crate) fn apply(&mut self, generation: usize) {
        if generation == 0 {
            panic!("Cannot apply generation 0");
        }
        let op = self.operations[generation - 1];
        self.write(op.addr, op.now);
    }

    pub(crate) fn unapply(&mut self, generation: usize) {
        if generation == 0 {
            panic!("Cannot unapply generation 0");
        }
        let op = self.operations[generation - 1];
        self.write(op.addr, op.was);
    }

    pub fn dump(&self) -> *const u16 {
        self.current.as_ptr()
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            current: [0; 4096],
            operations: vec![],
        }
    }
}

fn get_addr(instr: u16, pc: u16) -> u16 {
    let page_address_bits = instr & 0b0000_000_001_111_111;
    let page = if instr & 0b0000_000_010_000_000 == 0 {
        // Page 0
        0
    } else {
        // Current page
        pc & MASK_CURRENT_PAGE
    };
    page | page_address_bits
}

pub fn decode_addr(instr: u16, pc: u16, memory: &mut Memory) -> u16 {
    if (instr & MASK_MSDIGIT) >> 9 >= 6 {
        // No operand address in IOT or any group 1 or group 2 operations
        return 0;
    }
    if instr & 0b0000_000_100_000_000 == 0 {
        // Direct addressing
        get_addr(instr, pc)
    } else {
        // Indirect addressing
        let pointer_addr = get_addr(instr, pc);
        memory.read(pointer_addr)
    }
}
