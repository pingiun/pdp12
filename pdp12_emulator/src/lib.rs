mod consts;
pub mod eight_mode;
mod emulate;
mod memory;

pub use emulate::PDP12;
pub use memory::Memory;
pub use consts::*;

pub fn assemble<S>(code: S) -> [u16; 4096]
where
    S: AsRef<str>,
{
    let code = code.as_ref();
    let mut pointer: usize = 0;
    let mut memory = [0u16; 4096];
    for line in code.lines() {
        pointer = pointer & MASK_12BIT as usize;
        let mut parts: Vec<&str> = line.split(" ").collect();
        // Assembler directives
        if parts[0].chars().all(|c| c.is_ascii_digit()) {
            // Old style assembly, address then instr: 0000 HLT
            pointer = usize::from_str_radix(parts[0], 8).unwrap();
            parts.remove(0);
        } else {
            // New style assembly, using .directives
            if parts[0].to_lowercase() == ".address" {
                let data = parts[1];
                pointer = usize::from_str_radix(data, 8).unwrap();
                assert!(pointer < 4096);
                continue;
            }
            if parts[0].to_lowercase() == (".data") {
                let data = parts[1];
                let data = u16::from_str_radix(data, 8).unwrap();
                memory[pointer] = data;
                pointer += 1;
                continue;
            }
        }

        if parts[0].chars().all(|c| c.is_ascii_digit()) {
            // Direct data
            let data = parts[1];
            let data = u16::from_str_radix(data, 8).unwrap();
            memory[pointer] = data;
            pointer += 1;
            continue;
        }

        if parts[0].to_uppercase() == "HLT" {
            memory[pointer] = 0b0000_111_100_000_010;
            pointer += 1;
            continue;
        }
        let pos = ["AND", "TAD", "DCA", "JMP", "ISZ", "JMS"]
            .iter()
            .position(|i| i == &parts[0].to_uppercase().as_str());
        let instructions = [
            0b0000_000_000_000_000,
            0b0000_001_000_000_000,
            0b0000_011_000_000_000,
            0b0000_101_000_000_000,
            0b0000_010_000_000_000,
            0b0000_100_000_000_000,
        ];
        if let Some(pos) = pos {
            let mut instr = instructions[pos];
            let addr;
            if parts.len() == 3 && parts[1].to_uppercase() == "I" {
                instr |= 0b0000_000_100_000_000;
                addr = u16::from_str_radix(parts[2], 8).unwrap();
            } else {
                assert!(parts.len() == 2);
                addr = u16::from_str_radix(parts[1], 8).unwrap();
            }
            if addr < 0o200 {
                instr |= addr;
            } else {
                if (addr & MASK_CURRENT_PAGE) != (pointer as u16 & MASK_CURRENT_PAGE) {
                    panic!("Cannot encode instruction '{}', address not in range of current page or 0 page", line);
                }
                instr |= 0b0000_000_010_000_000;
                instr |= addr & !MASK_CURRENT_PAGE;
            }
            memory[pointer] = instr;
            pointer += 1;
            continue;
        }
    }
    memory
}

#[cfg(test)]
mod tests {
    use crate::{
        emulate::{step, State},
        memory::Memory,
    };

    use super::*;

    #[test]
    fn can_assemble() {
        let code = assemble(
            ".address 0
.data 5252
.data 6314
.address 200
JMS 350
DCA 2
HLT
.address 350
.data 0000
TAD 0
AND 1
JMP I 350",
        );

        let mut mem = Memory::with_code(code);
        let mut state = State::default();
        state.pc = 0o200;
        loop {
            state = step(state, &mut mem);
        }
    }
}
