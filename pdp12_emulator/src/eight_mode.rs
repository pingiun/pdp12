use crate::{consts::{MASK_MSDIGIT, MASK_12BIT}, emulate::State, memory::{Memory, decode_addr}};

#[must_use]
pub fn exec(instr: u16, state: State, memory: &mut Memory) -> State {
    let op_addr = decode_addr(instr, state.pc, memory);
    let msdigit = instr & MASK_MSDIGIT;
    if msdigit == 0b0000_000_000_000_000 {
        and(op_addr, state, &memory)
    } else if msdigit == 0b0000_001_000_000_000 {
        tad(op_addr, state, &memory)
    } else if msdigit == 0b0000_011_000_000_000 {
        dca(op_addr, state, memory)
    } else if msdigit == 0b0000_101_000_000_000 {
        jmp(op_addr, state)
    } else if msdigit == 0b0000_010_000_000_000 {
        isz(op_addr, state, memory)
    } else if msdigit == 0b0000_100_000_000_000 {
        jms(op_addr, state, memory)
    } else if msdigit == 0b0000_111_000_000_000 {
        op(instr, state)
    } else if msdigit == 0b0000_110_000_000_000 {
        iot(instr, state)
    } else {
        unreachable!()
    }
}

pub fn and(op_addr: u16, state: State, memory: &Memory) -> State {
    State {
        acc: state.acc & memory.read(op_addr),
        ..state
    }
}

pub fn tad(op_addr: u16, state: State, memory: &Memory) -> State {
    let lhs = state.acc
        | (if state.link {
            0b0001_000_000_000_000
        } else {
            0
        });
    let res = lhs + memory.read(op_addr);
    State {
        acc: res & MASK_12BIT,
        link: res & 0b0001_000_000_000_000 > 0,
        ..state
    }
}

pub fn dca(op_addr: u16, state: State, memory: &mut Memory) -> State {
    memory.write(op_addr, state.acc);
    State { acc: 0, ..state }
}

pub fn jmp(op_addr: u16, state: State) -> State {
    State {
        pc: op_addr,
        ..state
    }
}

pub fn isz(op_addr: u16, state: State, memory: &mut Memory) -> State {
    let newval = (memory.read(op_addr) + 1) & MASK_12BIT;
    memory.write(op_addr, newval);
    if newval == 0 {
        State {
            pc: state.pc + 1,
            ..state
        }
    } else {
        State { ..state }
    }
}

pub fn jms(op_addr: u16, state: State, memory: &mut Memory) -> State {
    memory.write(op_addr, state.pc);
    State {
        pc: (op_addr + 1) & MASK_12BIT,
        ..state
    }
}

pub fn op(instr: u16, state: State) -> State {
    if instr & 0b0000_000_100_000_000 == 0 {
        group1_op(instr, state)
    } else {
        group2_op(instr, state)
    }
}

pub fn group1_op(instr: u16, state: State) -> State {
    let mut state = state;
    if instr & 0b0000_000_010_000_000 > 0 {
        state.acc = 0;
    }
    if instr & 0b0000_000_001_000_000 > 0 {
        state.link = false;
    }
    if instr & 0b0000_000_000_100_000 > 0 {
        state.acc = !state.acc;
    }
    if instr & 0b0000_000_000_010_000 > 0 {
        state.link = !state.link;
    }
    if instr & 0b0000_000_000_000_001 > 0 {
        state.acc = (state.acc + 1) & MASK_12BIT;
    }
    if instr & 0b0000_000_000_001_000 > 0 {
        ror(&mut state);
        if instr & 0b0000_000_000_000_010 > 0 {
            ror(&mut state);
        }
    }
    // TODO: The Laborartory Computer Handbook tells us that a ror and rol in the same
    // instruction should not be possible. What does the real PDP-12 in this case?
    if instr & 0b0000_000_000_000_100 > 0 {
        rol(&mut state);
        if instr & 0b0000_000_000_000_010 > 0 {
            rol(&mut state);
        }
    }
    state
}

fn ror(state: &mut State) {
    let has_one = state.acc.trailing_zeros() == 0;
    state.acc = (state.acc >> 1)
        | if state.link {
            0b0000_100_000_000_000
        } else {
            0b0000_000_000_000_000
        };
    state.link = has_one;
}

fn rol(state: &mut State) {
    // If only 4 leading zeros, this means that the highest bit of the 12 bit number is 1
    let has_one = state.acc.leading_zeros() == 4;
    state.acc = ((state.acc << 1) & MASK_12BIT)
        | if state.link {
            0b0000_000_000_000_001
        } else {
            0b0000_000_000_000_000
        };
    state.link = has_one;
}

pub fn group2_op(instr: u16, state: State) -> State {
    let mut state = state;

    let mut skip = false;
    if instr & 0b0000_000_001_000_000 > 0 {
        skip |= state.acc & 0b0000_100_000_000_000 > 0;
    }
    if instr & 0b0000_000_000_100_000 > 0 {
        skip |= state.acc == 0;
    }
    if instr & 0b0000_000_000_010_000 > 0 {
        skip |= state.link;
    }
    if instr & 0b0000_000_000_001_000 > 0 {
        skip = !skip;
    }
    if instr & 0b0000_000_010_000_000 > 0 {
        // CLA
        state.acc = 0;
    }
    if instr & 0b0000_000_000_000_100 > 0 {
        // OSR
        state.acc |= state.rsw;
    }
    if instr & 0b0000_000_000_000_010 > 0 {
        // HLT
        state.running = false;
    }
    if skip {
        state.pc = (state.pc + 1) & MASK_12BIT;
    }

    state
}

pub fn iot(instr: u16, state: State) -> State {
    if instr & 0b0000_000_111_111_000 == 0b0000_000_000_011_000 {
        // Keyboard input (TTI)

    }
    state
}
