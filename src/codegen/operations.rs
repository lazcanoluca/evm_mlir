use melior::{
    dialect::arith,
    ir::{Attribute, Block, Location, Region},
    Context as MeliorContext,
};
use num_bigint::BigUint;

use super::context::CodegenCtx;
use crate::{errors::CodegenError, opcodes::Operation, utils::stack_push};

/// Generates blocks for target [`Operation`].
/// Returns the unterminated last block of the generated code.
pub fn generate_code_for_op<'c>(
    context: CodegenCtx<'c>,
    region: &Region<'c>,
    op: Operation,
) -> Result<Block<'c>, CodegenError> {
    match op {
        Operation::Push32(x) => codegen_push(context, region, x),
        _ => todo!(),
    }
}

// TODO: use const generics to generalize for pushN
fn codegen_push<'c>(
    codegen_ctx: CodegenCtx<'c>,
    _region: &Region<'c>,
    value_to_push: [u8; 32],
) -> Result<Block<'c>, CodegenError> {
    // TODO: handle stack overflow
    let block = Block::new(&[]);
    let context = &codegen_ctx.mlir_context;
    let location = Location::unknown(context);

    let constant_value = block
        .append_operation(arith::constant(
            context,
            integer_constant(context, value_to_push),
            location,
        ))
        .result(0)?
        .into();

    stack_push(context, &block, constant_value)?;

    Ok(block)
}

fn integer_constant(context: &MeliorContext, value: [u8; 32]) -> Attribute {
    let str_value = BigUint::from_bytes_be(&value).to_string();
    // TODO: should we handle this error?
    Attribute::parse(context, &format!("{str_value} : i256")).unwrap()
}
