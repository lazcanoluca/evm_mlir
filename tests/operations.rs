use evm_mlir::{compile_binary, opcodes::Operation};
use tempfile::NamedTempFile;

fn run_program_assert_result(program: Vec<Operation>, expected_result: u8) {
    let output_file = NamedTempFile::new()
        .expect("failed to generate tempfile")
        .into_temp_path();

    compile_binary(program, &output_file).expect("failed to compile program");
    let mut res = std::process::Command::new(&output_file)
        .spawn()
        .expect("spawn process failed");
    let output = res.wait().expect("wait for process failed");

    assert_eq!(output.code().expect("no exit code"), expected_result.into());
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
fn push32_fullstack() {
    let stack_top = 88;
    let mut program = vec![Operation::Push32([0; 32]); 1023];
    program.push(Operation::Push32([stack_top; 32]));

    run_program_assert_result(program, stack_top);
}
