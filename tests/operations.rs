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

#[test]
fn push_once() {
    let the_answer: u8 = 0_u8;

    // Test for PUSH0, PUSH1, ... , PUSH32
    for i in 0..33 {
        let bytes = vec![the_answer; i];
        let value = BigUint::from_bytes_be(&bytes);
        let program = vec![Operation::Push(value)];
        run_program_assert_result(program, the_answer);
    }
}

#[test]
fn push_twice() {
    let the_answer: u8 = 42;

    let program = vec![
        Operation::Push(BigUint::from(1_u8)),
        Operation::Push(BigUint::from(the_answer)),
    ];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push_fill_stack() {
    let stack_top: u8 = 88;

    // Push 1024 times
    let program = vec![Operation::Push(BigUint::from(stack_top)); 1024];
    run_program_assert_result(program, stack_top);
}

#[test]
fn push_stack_overflow() {
    // Push 1025 times
    let program = vec![Operation::Push(BigUint::from(88_u8)); 1025];
    run_program_assert_revert(program);
}

#[test]
fn push_push_add() {
    let (a, b): (u8, u8) = (11, 31);

    let program = vec![
        Operation::Push(BigUint::from(a)),
        Operation::Push(BigUint::from(b)),
        Operation::Add,
    ];
    run_program_assert_result(program, a + b);
}

#[test]
fn add_with_stack_underflow() {
    run_program_assert_revert(vec![Operation::Add]);
}

#[test]
fn push_push_pop() {
    // Push two values to the stack and then pop once
    // The program result should be equal to the first
    // pushed value
    let (a, b): (u8, u8) = (1, 2);

    let program = vec![
        Operation::Push(BigUint::from(a)),
        Operation::Push(BigUint::from(b)),
        Operation::Pop,
    ];
    run_program_assert_result(program, a);
}

#[test]
fn pop_with_stack_underflow() {
    // Pop with an empty stack
    let program = vec![Operation::Pop];
    run_program_assert_revert(program);
}
