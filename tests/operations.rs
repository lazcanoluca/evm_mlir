use evm_mlir::{compile_binary, constants::REVERT_EXIT_CODE, opcodes::Operation};
use num_bigint::BigUint;
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
fn dup1_once() {
    let program = vec![
        Operation::Push32(new_32_byte_immediate(10)),
        Operation::Push32(new_32_byte_immediate(32)),
        Operation::DupN(1),
    ];

    run_program_assert_result(program, 32);
}

#[test]
fn dup2_once() {
    let program = vec![
        Operation::Push32(new_32_byte_immediate(4)),
        Operation::Push32(new_32_byte_immediate(5)),
        Operation::DupN(2),
    ];

    run_program_assert_result(program, 4);
}

#[test]
fn push_push_sub() {
    let (a, b) = (11, 31);

    let program = vec![
        Operation::Push32(new_32_byte_immediate(a)),
        Operation::Push32(new_32_byte_immediate(b)),
        Operation::Sub,
    ];
    run_program_assert_result(program, b - a);
}

#[test]
fn dup_with_stack_underflow() {
    let program = vec![Operation::DupN(1)];
    run_program_assert_revert(program);
}

#[test]
fn substraction_wraps_the_result() {
    let (a, b) = (10, 0);

    let program = vec![
        Operation::Push32(new_32_byte_immediate(a)),
        Operation::Push32(new_32_byte_immediate(b)),
        Operation::Sub,
    ];

    let result = ((b as u32).wrapping_sub(a as u32)) as u8;

    run_program_assert_result(program, result);
}

#[test]
fn sub_add_wrapping() {
    let a = [0xFF; 32];
    let b = [10; 32];

    let program = vec![
        Operation::Push32(a),
        Operation::Push32(new_32_byte_immediate(10)),
        Operation::Add,
        Operation::Push32(new_32_byte_immediate(10)),
        Operation::Sub,
    ];

    run_program_assert_result(program, 1);
}

#[test]
fn add_with_stack_underflow() {
    run_program_assert_revert(vec![Operation::Add]);
}
