#[derive(Debug)]
pub enum Opcode {
    ADD = 0x01,
    SUB = 0x03,
    PUSH32 = 0x7F,
    DUP1 = 0x80,
    /*DUP2 = 0x81,
    DUP3 = 0x82,
    DUP4 = 0x83,
    DUP5 = 0x84,
    DUP6 = 0x85,
    DUP7 = 0x86,
    DUP8 = 0x87,
    DUP9 = 0x88,
    DUP10 = 0x89,
    DUP11 = 0x8A,
    DUP12 = 0x8B,
    DUP13 = 0x8C,
    DUP14 = 0x8D,
    DUP15 = 0x8E,
    DUP16 = 0x8F,*/
    UNUSED,
}

impl From<u8> for Opcode {
    fn from(opcode: u8) -> Opcode {
        match opcode {
            x if x == Opcode::ADD as u8 => Opcode::ADD,
            x if x == Opcode::SUB as u8 => Opcode::SUB,
            x if x == Opcode::PUSH32 as u8 => Opcode::PUSH32,
            x if x == Opcode::DUP1 as u8 => Opcode::DUP1,
            /*x if x == Opcode::DUP2 as u8 => Opcode::DUP2,
            x if x == Opcode::DUP3 as u8 => Opcode::DUP3,
            x if x == Opcode::DUP4 as u8 => Opcode::DUP4,
            x if x == Opcode::DUP5 as u8 => Opcode::DUP5,
            x if x == Opcode::DUP6 as u8 => Opcode::DUP6,
            x if x == Opcode::DUP7 as u8 => Opcode::DUP7,
            x if x == Opcode::DUP8 as u8 => Opcode::DUP8,
            x if x == Opcode::DUP9 as u8 => Opcode::DUP9,
            x if x == Opcode::DUP10 as u8 => Opcode::DUP10,
            x if x == Opcode::DUP11 as u8 => Opcode::DUP11,
            x if x == Opcode::DUP12 as u8 => Opcode::DUP12,
            x if x == Opcode::DUP13 as u8 => Opcode::DUP13,
            x if x == Opcode::DUP14 as u8 => Opcode::DUP14,
            x if x == Opcode::DUP15 as u8 => Opcode::DUP15,
            x if x == Opcode::DUP16 as u8 => Opcode::DUP16,*/
            _ => Opcode::UNUSED,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Add,
    Sub,
    Push32([u8; 32]),
    DupN(usize),
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
                Opcode::SUB => Operation::Sub,
                Opcode::PUSH32 => {
                    i += 1;
                    let x = bytecode[i..(i + 32)].try_into().unwrap();
                    i += 31;
                    Operation::Push32(x)
                }
                Opcode::DUP1 => Operation::DupN(1),
                Opcode::UNUSED => panic!("Unknown opcode {:02X}", opcode),
            };
            operations.push(op);
            i += 1;
        }
        operations
    }
}
