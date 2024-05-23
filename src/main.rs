use evm_mlir::codegen::context::{self, compile_to_object};
use std::path::PathBuf;

use crate::opcodes::Operation;

mod opcodes;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("No path provided").as_str();
    let bytecode = std::fs::read(path).expect("Could not read file");
    let operations = Operation::from_bytecode(bytecode);

    let context = context::Context::new();

    println!("Creating MLIR module");

    let result = context.compile(operations).unwrap();

    println!("Compiling with LLVM");

    let object_file = compile_to_object(&result).unwrap();

    println!("Linking...");

    link_binary(object_file);

    println!("Done!");
}

fn link_binary(object_file: PathBuf) {
    let args = vec![
        "-L/usr/local/lib",
        "-L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib",
        object_file.to_str().unwrap(),
        "-o",
        "output",
        "-lSystem",
    ];
    let mut linker = std::process::Command::new("ld");
    let proc = linker.args(args).spawn().unwrap();
    let output = proc.wait_with_output().unwrap();
    assert!(output.status.success());
}
