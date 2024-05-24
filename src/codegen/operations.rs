use melior::{
    dialect::{
        arith,
        llvm::{self, LoadStoreOptions},
    },
    ir::{Attribute, Block, Location, Value},
    Context as MeliorContext,
};
use num_bigint::BigUint;

use super::context::CodegenCtx;
use crate::opcodes::Operation;

pub fn generate_code_for_op(
    context: CodegenCtx,
    block: &Block,
    stack_base: Value,
    op: &Operation,
) -> Result<(), String> {
    match op {
        Operation::Push32(x) => codegen_push(context, block, stack_base, *x),
        _ => todo!(),
    }
}

// TODO: use const generics to generalize for pushN
fn codegen_push(
    codegen_ctx: CodegenCtx,
    block: &Block,
    stack_base: Value,
    value_to_push: [u8; 32],
) -> Result<(), String> {
    let context = &codegen_ctx.mlir_context;
    let location = Location::unknown(context);

    let constant_value = block
        .append_operation(arith::constant(
            context,
            integer_constant(context, value_to_push),
            location,
        ))
        .result(0)
        .unwrap()
        .into();

    block.append_operation(llvm::store(
        context,
        constant_value,
        stack_base,
        location,
        LoadStoreOptions::default(),
    ));

    Ok(())
}

fn integer_constant(context: &MeliorContext, value: [u8; 32]) -> Attribute {
    let str_value = BigUint::from_bytes_be(&value).to_string();
    Attribute::parse(context, &format!("{str_value} : i256")).unwrap()
}
