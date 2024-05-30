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
    let value = BigUint::from(5_u8);

    // For PUSH0
    let program = vec![Operation::Push(BigUint::ZERO)];
    run_program_assert_result(program, 0);

    // For PUSH1, ... , PUSH32
    for i in 0..32 {
        let shifted_value: BigUint = value.clone() << (i * 8);
        let program = vec![Operation::Push(shifted_value.clone())];
        let expected_result: u8 = (shifted_value % 256_u32).try_into().unwrap();
        run_program_assert_result(program, expected_result);
    }
}

#[test]
fn push_twice() {
    let the_answer = BigUint::from(42_u8);
    let program = vec![
        Operation::Push(BigUint::from(1_u8)),
        Operation::Push(the_answer.clone()),
    ];
    run_program_assert_result(program, the_answer.try_into().unwrap());
}

#[test]
fn push_fill_stack() {
    let stack_top = BigUint::from(88_u8);
    // Push 1024 times
    let program = vec![Operation::Push(stack_top.clone()); 1024];
    run_program_assert_result(program, stack_top.try_into().unwrap());
}

#[test]
fn push32_stack_overflow() {
    // Push 1025 times
    let program = vec![Operation::Push(BigUint::from(88_u8)); 1025];
    run_program_assert_revert(program);
}

#[test]
fn push_push_add() {
    let (a, b) = (BigUint::from(11_u8), BigUint::from(31_u8));

    let program = vec![
        Operation::Push(a.clone()),
        Operation::Push(b.clone()),
        Operation::Add,
    ];
    run_program_assert_result(program, (a + b).try_into().unwrap());
}

#[test]
fn dup1_once() {
    let program = vec![
        Operation::Push(BigUint::from(10_u8)),
        Operation::Push(BigUint::from(31_u8)),
        Operation::Dup(1),
        Operation::Pop,
    ];

    run_program_assert_result(program, 31);
}

#[test]
fn dup2_once() {
    let program = vec![
        Operation::Push(BigUint::from(4_u8)),
        Operation::Push(BigUint::from(5_u8)),
        Operation::Push(BigUint::from(6_u8)),
        Operation::Dup(2),
    ];

    run_program_assert_result(program, 5);
}

#[test]
fn dup_combined() {
    let program = vec![
        Operation::Push(BigUint::from(4_u8)),
        Operation::Push(BigUint::from(5_u8)),
        Operation::Push(BigUint::from(6_u8)),
        Operation::Dup(2),
        Operation::Dup(1),
        Operation::Dup(5),
        Operation::Dup(3),
        Operation::Dup(4),
        Operation::Dup(7),
        Operation::Dup(6),
        Operation::Dup(8),
        Operation::Dup(9),
        Operation::Dup(12),
        Operation::Dup(11),
        Operation::Dup(10),
        Operation::Dup(13),
        Operation::Dup(15),
        Operation::Dup(14),
        Operation::Dup(16),
    ];

    run_program_assert_result(program, 6);
}

#[test]
fn dup_with_stack_underflow() {
    let program = vec![Operation::Dup(1)];
    run_program_assert_revert(program);
}

#[test]
fn push_push_sub() {
    let (a, b) = (BigUint::from(11_u8), BigUint::from(31_u8));

    let program = vec![
        Operation::Push(a.clone()),
        Operation::Push(b.clone()),
        Operation::Sub,
    ];
    run_program_assert_result(program, 20);
}

#[test]
fn substraction_wraps_the_result() {
    let (a, b) = (BigUint::from(10_u8), BigUint::from(0_u8));

    let program = vec![
        Operation::Push(a.clone()),
        Operation::Push(b.clone()),
        Operation::Sub,
    ];

    run_program_assert_result(program, 246);
}

#[test]
fn add_with_stack_underflow() {
    run_program_assert_revert(vec![Operation::Add]);
}
