use melior::{
    dialect::{
        arith,
        llvm::{self, r#type::pointer, LoadStoreOptions},
    },
    ir::{Attribute, Block, Location},
    Context as MeliorContext,
};
use num_bigint::BigUint;

use super::context::CodegenCtx;
use crate::{constants::STACK_GLOBAL_VAR, opcodes::Operation, utils::llvm_mlir};

pub fn generate_code_for_op(
    context: CodegenCtx,
    block: &Block,
    op: &Operation,
) -> Result<(), String> {
    match op {
        Operation::Push32(x) => codegen_push(context, block, *x),
        _ => todo!(),
    }
}

// TODO: use const generics to generalize for pushN
fn codegen_push(
    codegen_ctx: CodegenCtx,
    block: &Block,
    value_to_push: [u8; 32],
) -> Result<(), String> {
    let context = &codegen_ctx.mlir_context;
    let location = Location::unknown(context);
    let ptr_type = pointer(context, 0);

    let constant_value = block
        .append_operation(arith::constant(
            context,
            integer_constant(context, value_to_push),
            location,
        ))
        .result(0)
        .unwrap()
        .into();

    let stack_baseptr_ptr = block
        .append_operation(llvm_mlir::addressof(
            context,
            STACK_GLOBAL_VAR,
            ptr_type,
            location,
        ))
        .result(0)
        .unwrap();

    let stack_baseptr = block
        .append_operation(llvm::load(
            context,
            stack_baseptr_ptr.into(),
            ptr_type,
            location,
            LoadStoreOptions::default(),
        ))
        .result(0)
        .unwrap();

    block.append_operation(llvm::store(
        context,
        constant_value,
        stack_baseptr.into(),
        location,
        LoadStoreOptions::default(),
    ));

    Ok(())
}

fn integer_constant(context: &MeliorContext, value: [u8; 32]) -> Attribute {
    let str_value = BigUint::from_bytes_be(&value).to_string();
    Attribute::parse(context, &format!("{str_value} : i256")).unwrap()
}
