#[derive(Debug)]
pub enum Opcode {
    ADD = 0x01,
    PUSH0 = 0x5F,
    PUSH1 = 0x60,
    PUSH2 = 0x61,
    PUSH3 = 0x62,
    PUSH4 = 0x63,
    PUSH5 = 0x64,
    PUSH6 = 0x65,
    PUSH7 = 0x66,
    PUSH8 = 0x67,
    PUSH9 = 0x68,
    PUSH10 = 0x69,
    PUSH11 = 0x6A,
    PUSH12 = 0x6B,
    PUSH13 = 0x6C,
    PUSH14 = 0x6D,
    PUSH15 = 0x6E,
    PUSH16 = 0x6F,
    PUSH17 = 0x70,
    PUSH18 = 0x71,
    PUSH19 = 0x72,
    PUSH20 = 0x73,
    PUSH21 = 0x74,
    PUSH22 = 0x75,
    PUSH23 = 0x76,
    PUSH24 = 0x77,
    PUSH25 = 0x78,
    PUSH26 = 0x79,
    PUSH27 = 0x7A,
    PUSH28 = 0x7B,
    PUSH29 = 0x7C,
    PUSH30 = 0x7D,
    PUSH31 = 0x7E,
    PUSH32 = 0x7F,
    POP = 0x50,
    UNUSED,
}

impl From<u8> for Opcode {
    fn from(opcode: u8) -> Opcode {
        match opcode {
            x if x == Opcode::ADD as u8 => Opcode::ADD,
            x if x == Opcode::PUSH0 as u8 => Opcode::PUSH0,
            x if x == Opcode::PUSH1 as u8 => Opcode::PUSH1,
            x if x == Opcode::PUSH2 as u8 => Opcode::PUSH2,
            x if x == Opcode::PUSH3 as u8 => Opcode::PUSH3,
            x if x == Opcode::PUSH4 as u8 => Opcode::PUSH4,
            x if x == Opcode::PUSH5 as u8 => Opcode::PUSH5,
            x if x == Opcode::PUSH6 as u8 => Opcode::PUSH6,
            x if x == Opcode::PUSH7 as u8 => Opcode::PUSH7,
            x if x == Opcode::PUSH8 as u8 => Opcode::PUSH8,
            x if x == Opcode::PUSH9 as u8 => Opcode::PUSH9,
            x if x == Opcode::PUSH10 as u8 => Opcode::PUSH10,
            x if x == Opcode::PUSH11 as u8 => Opcode::PUSH11,
            x if x == Opcode::PUSH12 as u8 => Opcode::PUSH12,
            x if x == Opcode::PUSH13 as u8 => Opcode::PUSH13,
            x if x == Opcode::PUSH14 as u8 => Opcode::PUSH14,
            x if x == Opcode::PUSH15 as u8 => Opcode::PUSH15,
            x if x == Opcode::PUSH16 as u8 => Opcode::PUSH16,
            x if x == Opcode::PUSH17 as u8 => Opcode::PUSH17,
            x if x == Opcode::PUSH18 as u8 => Opcode::PUSH18,
            x if x == Opcode::PUSH19 as u8 => Opcode::PUSH19,
            x if x == Opcode::PUSH20 as u8 => Opcode::PUSH20,
            x if x == Opcode::PUSH21 as u8 => Opcode::PUSH21,
            x if x == Opcode::PUSH22 as u8 => Opcode::PUSH22,
            x if x == Opcode::PUSH23 as u8 => Opcode::PUSH23,
            x if x == Opcode::PUSH24 as u8 => Opcode::PUSH24,
            x if x == Opcode::PUSH25 as u8 => Opcode::PUSH25,
            x if x == Opcode::PUSH26 as u8 => Opcode::PUSH26,
            x if x == Opcode::PUSH27 as u8 => Opcode::PUSH27,
            x if x == Opcode::PUSH28 as u8 => Opcode::PUSH28,
            x if x == Opcode::PUSH29 as u8 => Opcode::PUSH29,
            x if x == Opcode::PUSH30 as u8 => Opcode::PUSH30,
            x if x == Opcode::PUSH31 as u8 => Opcode::PUSH31,
            x if x == Opcode::PUSH32 as u8 => Opcode::PUSH32,
            x if x == Opcode::PUSH32 as u8 => Opcode::PUSH32,
            x if x == Opcode::POP as u8 => Opcode::POP,
            _ => Opcode::UNUSED,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Add,
    Push0,
    Push1([u8; 1]),
    Push2([u8; 2]),
    Push3([u8; 3]),
    Push4([u8; 4]),
    Push5([u8; 5]),
    Push6([u8; 6]),
    Push7([u8; 7]),
    Push8([u8; 8]),
    Push9([u8; 9]),
    Push10([u8; 10]),
    Push11([u8; 11]),
    Push12([u8; 12]),
    Push13([u8; 13]),
    Push14([u8; 14]),
    Push15([u8; 15]),
    Push16([u8; 16]),
    Push17([u8; 17]),
    Push18([u8; 18]),
    Push19([u8; 19]),
    Push20([u8; 20]),
    Push21([u8; 21]),
    Push22([u8; 22]),
    Push23([u8; 23]),
    Push24([u8; 24]),
    Push25([u8; 25]),
    Push26([u8; 26]),
    Push27([u8; 27]),
    Push28([u8; 28]),
    Push29([u8; 29]),
    Push30([u8; 30]),
    Push31([u8; 31]),
    Push32([u8; 32]),
    Pop,
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
                Opcode::PUSH0 => {
                    i += 32;
                    Operation::Push0
                }
                Opcode::PUSH1 => {
                    i += 1;
                    let x = bytecode[i..(i + 1)].try_into().unwrap();
                    i += 31;
                    Operation::Push1(x)
                }
                Opcode::PUSH2 => {
                    i += 1;
                    let x = bytecode[i..(i + 2)].try_into().unwrap();
                    i += 1;
                    Operation::Push2(x)
                }
                Opcode::PUSH3 => {
                    i += 1;
                    let x = bytecode[i..(i + 3)].try_into().unwrap();
                    i += 2;
                    Operation::Push3(x)
                }
                Opcode::PUSH4 => {
                    i += 1;
                    let x = bytecode[i..(i + 4)].try_into().unwrap();
                    i += 3;
                    Operation::Push4(x)
                }
                Opcode::PUSH5 => {
                    i += 1;
                    let x = bytecode[i..(i + 5)].try_into().unwrap();
                    i += 4;
                    Operation::Push5(x)
                }
                Opcode::PUSH6 => {
                    i += 1;
                    let x = bytecode[i..(i + 6)].try_into().unwrap();
                    i += 5;
                    Operation::Push6(x)
                }
                Opcode::PUSH7 => {
                    i += 1;
                    let x = bytecode[i..(i + 7)].try_into().unwrap();
                    i += 6;
                    Operation::Push7(x)
                }
                Opcode::PUSH8 => {
                    i += 1;
                    let x = bytecode[i..(i + 8)].try_into().unwrap();
                    i += 7;
                    Operation::Push8(x)
                }
                Opcode::PUSH9 => {
                    i += 1;
                    let x = bytecode[i..(i + 9)].try_into().unwrap();
                    i += 8;
                    Operation::Push9(x)
                }
                Opcode::PUSH10 => {
                    i += 1;
                    let x = bytecode[i..(i + 10)].try_into().unwrap();
                    i += 9;
                    Operation::Push10(x)
                }
                Opcode::PUSH11 => {
                    i += 1;
                    let x = bytecode[i..(i + 11)].try_into().unwrap();
                    i += 10;
                    Operation::Push11(x)
                }
                Opcode::PUSH12 => {
                    i += 1;
                    let x = bytecode[i..(i + 12)].try_into().unwrap();
                    i += 11;
                    Operation::Push12(x)
                }
                Opcode::PUSH13 => {
                    i += 1;
                    let x = bytecode[i..(i + 13)].try_into().unwrap();
                    i += 12;
                    Operation::Push13(x)
                }
                Opcode::PUSH14 => {
                    i += 1;
                    let x = bytecode[i..(i + 14)].try_into().unwrap();
                    i += 13;
                    Operation::Push14(x)
                }
                Opcode::PUSH15 => {
                    i += 1;
                    let x = bytecode[i..(i + 15)].try_into().unwrap();
                    i += 14;
                    Operation::Push15(x)
                }
                Opcode::PUSH16 => {
                    i += 1;
                    let x = bytecode[i..(i + 16)].try_into().unwrap();
                    i += 15;
                    Operation::Push16(x)
                }
                Opcode::PUSH17 => {
                    i += 1;
                    let x = bytecode[i..(i + 17)].try_into().unwrap();
                    i += 16;
                    Operation::Push17(x)
                }
                Opcode::PUSH18 => {
                    i += 1;
                    let x = bytecode[i..(i + 18)].try_into().unwrap();
                    i += 17;
                    Operation::Push18(x)
                }
                Opcode::PUSH19 => {
                    i += 1;
                    let x = bytecode[i..(i + 19)].try_into().unwrap();
                    i += 18;
                    Operation::Push19(x)
                }
                Opcode::PUSH20 => {
                    i += 1;
                    let x = bytecode[i..(i + 20)].try_into().unwrap();
                    i += 19;
                    Operation::Push20(x)
                }
                Opcode::PUSH21 => {
                    i += 1;
                    let x = bytecode[i..(i + 21)].try_into().unwrap();
                    i += 20;
                    Operation::Push21(x)
                }
                Opcode::PUSH22 => {
                    i += 1;
                    let x = bytecode[i..(i + 32)].try_into().unwrap();
                    i += 21;
                    Operation::Push22(x)
                }
                Opcode::PUSH23 => {
                    i += 1;
                    let x = bytecode[i..(i + 32)].try_into().unwrap();
                    i += 22;
                    Operation::Push23(x)
                }
                Opcode::PUSH24 => {
                    i += 1;
                    let x = bytecode[i..(i + 32)].try_into().unwrap();
                    i += 23;
                    Operation::Push24(x)
                }
                Opcode::PUSH25 => {
                    i += 1;
                    let x = bytecode[i..(i + 32)].try_into().unwrap();
                    i += 24;
                    Operation::Push25(x)
                }
                Opcode::PUSH26 => {
                    i += 1;
                    let x = bytecode[i..(i + 26)].try_into().unwrap();
                    i += 25;
                    Operation::Push26(x)
                }
                Opcode::PUSH27 => {
                    i += 1;
                    let x = bytecode[i..(i + 27)].try_into().unwrap();
                    i += 26;
                    Operation::Push27(x)
                }
                Opcode::PUSH28 => {
                    i += 1;
                    let x = bytecode[i..(i + 28)].try_into().unwrap();
                    i += 27;
                    Operation::Push28(x)
                }
                Opcode::PUSH29 => {
                    i += 1;
                    let x = bytecode[i..(i + 29)].try_into().unwrap();
                    i += 28;
                    Operation::Push29(x)
                }
                Opcode::PUSH30 => {
                    i += 1;
                    let x = bytecode[i..(i + 30)].try_into().unwrap();
                    i += 29;
                    Operation::Push30(x)
                }
                Opcode::PUSH31 => {
                    i += 1;
                    let x = bytecode[i..(i + 31)].try_into().unwrap();
                    i += 30;
                    Operation::Push31(x)
                }
                Opcode::PUSH32 => {
                    i += 1;
                    let x = bytecode[i..(i + 32)].try_into().unwrap();
                    i += 31;
                    Operation::Push32(x)
                }
                Opcode::POP => Operation::Pop,
                Opcode::UNUSED => panic!("Unknown opcode {:02X}", opcode),
            };
            operations.push(op);
            i += 1;
        }
        operations
    }
}
