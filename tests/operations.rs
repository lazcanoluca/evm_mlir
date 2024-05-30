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

fn new_push_op(bytes: &[u8]) -> Operation {
    Operation::Push(BigUint::from_bytes_be(bytes.into()))
}

#[test]
fn push0_once() {
    let the_answer = 0;
    let program = vec![new_push_op(&[the_answer; 32])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push1_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 1])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push2_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 2])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push3_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 3])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push4_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 4])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push5_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 5])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push6_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 6])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push7_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 7])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push8_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 8])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push9_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 9])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push10_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 10])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push11_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 11])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push12_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 12])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push13_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 13])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push14_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 14])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push15_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 15])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push16_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 16])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push17_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 17])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push18_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 18])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push19_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 19])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push20_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 20])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push21_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 21])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push22_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 22])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push23_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 23])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push24_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 24])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push25_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 25])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push26_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 26])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push27_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 27])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push28_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 28])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push29_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 29])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push30_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 30])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push31_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 31])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push32_once() {
    let the_answer = 42;
    let program = vec![new_push_op(&[the_answer; 32])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push32_twice() {
    let the_answer = 42;
    let program = vec![new_push_op(&[0]), new_push_op(&[the_answer; 32])];
    run_program_assert_result(program, the_answer);
}

#[test]
fn push32_fill_stack() {
    let stack_top = 88;
    // Push 1024 times
    let program = vec![new_push_op(&[stack_top; 32]); 1024];
    run_program_assert_result(program, stack_top);
}

#[test]
fn push32_stack_overflow() {
    // Push 1025 times
    let program = vec![new_push_op(&[88; 32]); 1025];
    run_program_assert_revert(program);
}

#[test]
fn push32_push32_add() {
    let (a, b) = (11, 31);

    let program = vec![
        new_push_op(&new_32_byte_immediate(a)),
        new_push_op(&new_32_byte_immediate(b)),
        Operation::Add,
    ];
    run_program_assert_result(program, a + b);
}

#[test]
fn push2_push1_add() {
    let (a, b) = (11, 31);

    let program = vec![new_push_op(&[0, a]), new_push_op(&[b]), Operation::Add];
    run_program_assert_result(program, a + b);
}

#[test]
fn add_with_stack_underflow() {
    run_program_assert_revert(vec![Operation::Add]);
}

#[test]
fn push32_push32_pop() {
    // Push two values to the stack and then pop once
    // The program result should be equal to the first
    // pushed value
    let (a, b) = (1, 2);
    let program = vec![
        new_push_op(&new_32_byte_immediate(a)),
        new_push_op(&new_32_byte_immediate(b)),
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
