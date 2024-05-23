pub enum Opcode {
    STOP = 0x00,
    ADD = 0x01,
    PUSH1 = 0x60,
    UNUSED,
}

impl From<u8> for Opcode {
    fn from(opcode: u8) -> Opcode {
        match opcode {
            0x00 => Opcode::STOP,
            0x01 => Opcode::ADD,
            0x60 => Opcode::PUSH1,
            _ => Opcode::UNUSED,
        }
    }
}


pub enum Operation {
    Stop,
    // Add {}
    Push1 { value: u8 },
}
