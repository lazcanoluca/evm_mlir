use evm_mlir::{compile, compile_to_object, context::Context, link_binary, opcodes::Operation};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("No path provided").as_str();
    let bytecode = std::fs::read(path).expect("Could not read file");
    let operations = Operation::from_bytecode(bytecode);

    let object_file = compile(operations);

    println!("Linking...");

    link_binary(object_file);

    println!("Done!");
}
