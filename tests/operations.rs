use evm_mlir::{compile_binary, opcodes::Operation};
use tempfile::NamedTempFile;

#[test]
fn push32() {
    let output_file = NamedTempFile::new().unwrap().into_temp_path();
    let the_answer = 42;
    let program = vec![Operation::Push32([the_answer; 32])];

    compile_binary(program, &output_file).unwrap();
    let mut res = std::process::Command::new(&output_file).spawn().unwrap();
    let output = res.wait().unwrap();
    assert_eq!(output.code().unwrap(), the_answer.into());
}
