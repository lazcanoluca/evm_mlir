#[derive(Debug)]
pub enum Opcode {
    ADD = 0x01,
    PUSH32 = 0x7F,
    POP = 0x50,
    BYTE = 0x1A,
    UNUSED,
}

impl From<u8> for Opcode {
    fn from(opcode: u8) -> Opcode {
        match opcode {
            x if x == Opcode::ADD as u8 => Opcode::ADD,
            x if x == Opcode::PUSH32 as u8 => Opcode::PUSH32,
            x if x == Opcode::POP as u8 => Opcode::POP,
            x if x == Opcode::BYTE as u8 => Opcode::BYTE,
            _ => Opcode::UNUSED,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Add,
    Push32([u8; 32]),
    Pop,
    Byte,
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
                Opcode::ADD => Operation::Add,
                Opcode::PUSH32 => {
                    i += 1;
                    let x = bytecode[i..(i + 32)].try_into().unwrap();
                    i += 31;
                    Operation::Push32(x)
                }
                Opcode::POP => Operation::Pop,
                Opcode::BYTE => Operation::Byte,
                Opcode::UNUSED => panic!("Unknown opcode {:02X}", opcode),
            };
            operations.push(op);
            i += 1;
        }
        operations
    }
}
