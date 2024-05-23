use crate::opcodes::Opcode;

mod opcodes;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("No path provided").as_str();
    let code = std::fs::read(path).expect("Could not read file");
    let mut stack: Vec<u128> = Vec::new();
    let mut pc = 0;
    loop {
        let Some(opcode) = code.get(pc).copied() else {
            break;
        };
        println!("PC: {:04X} Opcode: {:02X}", pc, opcode);
        match Opcode::from(opcode) {
            Opcode::STOP => break,
            Opcode::ADD => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                println!("Adding {a} and {b}, result: {}", a + b);
                stack.push(a + b);
            }
            Opcode::PUSH1 => {
                pc += 1;
                let x = code[pc];
                println!("Pushing {x} to stack");
                stack.push(x as u128);
            }
            Opcode::UNUSED => panic!("Unknown opcode {:02X}", opcode),
        }
        pc += 1;
    }
    println!("Stack:");
    println!("{:?}", stack);
}
