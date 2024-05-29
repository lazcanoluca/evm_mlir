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
fn push_push_pop() {
    // Push two values to the stack and then pop once
    // The program result should be equal to the first
    // pushed value
    let (a, b) = (1, 2);
    let program = vec![
        Operation::Push32(new_32_byte_immediate(a)),
        Operation::Push32(new_32_byte_immediate(b)),
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

#[test]
fn push_push_sar() {
    let (value, shift) = (2, 1);
    let program = vec![
        Operation::Push32(new_32_byte_immediate(value)),
        Operation::Push32(new_32_byte_immediate(shift)),
        Operation::Sar,
    ];
    let expected_result = value >> shift;
    run_program_assert_result(program, expected_result);
}

#[test]
fn sar_with_stack_underflow() {
    let program = vec![Operation::Sar];
    run_program_assert_revert(program);
}

#[test]
fn sar_with_negative_value_preserves_sign() {
    // in this example the the value to be shifted is a 256 bit number
    // where the most significative bit is 1 cand the rest of the bits are 0.
    // i.e,  value = 1000..0000
    //
    // if we shift this value 255 positions to the right, given that
    // the sar operation preserves the sign, the result must be a number
    // in which every bit is 1
    // i.e, result = 1111..1111
    //
    // given that the program results is a u8, the result is then truncated
    // to the less 8 significative bits, i.e  result = 0b11111111.
    //
    // this same example can be visualized in the evm playground in the following link
    // https://www.evm.codes/playground?fork=cancun&unit=Wei&codeType=Mnemonic&code='%2F%2F%20Example%201z32%200x8yyyz8%20255wSAR'~0000000zwPUSHy~~~w%5Cn%01wyz~_

    let mut value: [u8; 32] = [0; 32];
    value[0] = 0b10000000;
    let shift = 255;
    let program = vec![
        Operation::Push32(value),
        Operation::Push32(new_32_byte_immediate(shift)),
        Operation::Sar,
    ];
    let expected_result = 0b11111111;
    run_program_assert_result(program, expected_result);
}
