use evm_mlir::{compile_binary, constants::REVERT_EXIT_CODE, opcodes::Operation};
use tempfile::NamedTempFile;

fn run_program_assert_result(program: Vec<Operation>, expected_result: u8) {
    let output_file = NamedTempFile::new()
        .expect("failed to generate tempfile")
        .into_temp_path();

    compile_binary(program, &output_file).expect("failed to compile program");

    assert!(output_file.exists(), "output file does not exist");

    let mut res = std::process::Command::new(&output_file)
        .spawn()
        .expect("spawn process failed");
    let output = res.wait().expect("wait for process failed");

    assert_eq!(output.code().expect("no exit code"), expected_result.into());
}

fn run_program_assert_revert(program: Vec<Operation>) {
    // TODO: design a way to check for stack overflow
    run_program_assert_result(program, REVERT_EXIT_CODE);
}

fn new_32_byte_immediate(value: u8) -> [u8; 32] {
    let mut arr = [0; 32];
    arr[31] = value;
    arr
}

fn new_32_byte_by_lshift(byte_value: u8, byte_lshift: u8) -> [u8; 32] {
    assert!(byte_lshift < 32);
    let mut arr = [0; 32];
    let idx = (31 - byte_lshift);
    arr[idx as usize] = byte_value;
    arr
}

#[test]
fn push32_once() {
    let the_answer = 42;
    let program = vec![Operation::Push32([the_answer; 32])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push32_twice() {
    let the_answer = 42;
    let program = vec![
        Operation::Push32([0; 32]),
        Operation::Push32([the_answer; 32]),
    ];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push32_fill_stack() {
    let stack_top = 88;
    // Push 1024 times
    let program = vec![Operation::Push32([stack_top; 32]); 1024];

    run_program_assert_result(program, stack_top);
}

#[test]
fn push32_stack_overflow() {
    // Push 1025 times
    let program = vec![Operation::Push32([88; 32]); 1025];

    run_program_assert_revert(program);
}

#[test]
fn push_push_add() {
    let (a, b) = (11, 31);

    let program = vec![
        Operation::Push32(new_32_byte_immediate(a)),
        Operation::Push32(new_32_byte_immediate(b)),
        Operation::Add,
    ];
    run_program_assert_result(program, a + b);
}

#[test]
fn add_with_stack_underflow() {
    run_program_assert_revert(vec![Operation::Add]);
}

#[test]
fn div_without_remainder() {
    let (a, b) = (20, 5);

    let expected_result = 4;

    let program = vec![
        Operation::Push32(new_32_byte_immediate(b)),
        Operation::Push32(new_32_byte_immediate(a)),
        Operation::Div,
    ];

    run_program_assert_result(program, expected_result);
}

#[test]
fn div_signed_division() {
    // a = [1, 0, 0, 0, .... , 0, 0, 0, 0] == 1 << 255
    let a = new_32_byte_by_lshift(0x80, 31);
    // b = [0, 0, 1, 0, .... , 0, 0, 0, 0] == 1 << 253
    let b = new_32_byte_by_lshift(0x20, 31);
    //r = a / b = [0, 0, 0, 0, ....., 0, 1, 0, 0] = 4 in decimal
    //If we take the lowest byte
    //r = [0, 0, 0, 0, 0, 1, 0, 0] = 4 in decimal
    let expected_result: u8 = 4;

    let program = vec![
        Operation::Push32(b), //
        Operation::Push32(a), //
        Operation::Div,       //
    ];
    run_program_assert_result(program, expected_result);
}

#[test]
fn div_with_remainder() {
    let (a, b) = (21, 5);

    let expected_result = 4;

    let program = vec![
        Operation::Push32(new_32_byte_immediate(b)),
        Operation::Push32(new_32_byte_immediate(a)),
        Operation::Div,
    ];
    run_program_assert_result(program, expected_result);
}

#[ignore]
#[test]
fn div_with_zero_denominator() {
    let (a, b) = (5, 0);

    let expected_result = 0;

    let program = vec![
        Operation::Push32(new_32_byte_immediate(b)),
        Operation::Push32(new_32_byte_immediate(a)),
        Operation::Div,
    ];
    run_program_assert_result(program, expected_result);
}

#[test]
fn div_with_zero_numerator() {
    let (a, b) = (0, 10);

    let expected_result = 0;

    let program = vec![
        Operation::Push32(new_32_byte_immediate(b)),
        Operation::Push32(new_32_byte_immediate(a)),
        Operation::Div,
    ];
    run_program_assert_result(program, expected_result);
}

#[test]
fn div_with_stack_underflow() {
    run_program_assert_revert(vec![Operation::Div]);
}
