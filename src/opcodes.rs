#![allow(dead_code)]

pub enum Opcode {
    STOP = 0x00,
    ADD = 0x01,
    PUSH1 = 0x60,
    PUSH32 = 0x7F,
    UNUSED,
}

impl From<u8> for Opcode {
    fn from(opcode: u8) -> Opcode {
        match opcode {
            0x00 => Opcode::STOP,
            0x01 => Opcode::ADD,
            0x60 => Opcode::PUSH1,
            0x7F => Opcode::PUSH32,
            _ => Opcode::UNUSED,
        }
    }
}

pub enum Operation {
    Stop,
    Add,
    Push1(u8),
    Push32([u8; 32]),
}

impl Operation {
    pub fn from_bytecode(bytecode: Vec<u8>) -> Vec<Self> {
        let mut operations = vec![];
        let mut i = 0;

        while i < bytecode.len() {
            let Some(opcode) = bytecode.get(i).copied() else {
                break;
            };
            let op = match Opcode::from(opcode) {
                Opcode::STOP => Operation::Stop,
                Opcode::ADD => Operation::Add,
                Opcode::PUSH1 => {
                    i += 1;
                    let x = bytecode[i];
                    Operation::Push1(x)
                }
                Opcode::PUSH32 => {
                    i += 1;
                    let x = bytecode[i..(i + 32)].try_into().unwrap();
                    i += 31;
                    Operation::Push32(x)
                }
                Opcode::UNUSED => panic!("Unknown opcode {:02X}", opcode),
            };
            operations.push(op);
            i += 1;
        }
        operations
    }
}
