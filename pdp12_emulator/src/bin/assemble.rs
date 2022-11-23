use pdp12_emulator::assemble;

fn main() {
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
    let mut last_instr = u16::MAX;
    let mut skipping = false;
    for (addr, instr) in code.iter().enumerate() {
        let addr = addr as u16;
        let instr = *instr;

        if last_instr != instr {
            skipping = false;
        } else if skipping && addr != 0o7777 {
            continue;
        }
        if last_instr == instr && addr != 0o7777 {
            skipping = true;
            println!("...");
            continue;
        }
        println!("{:04o}\t{:04o}", addr, instr);
        last_instr = instr;
    }
}
