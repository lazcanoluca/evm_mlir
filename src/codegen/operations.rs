use melior::{
    dialect::{arith, cf, func},
    ir::{
        attribute::IntegerAttribute, r#type::IntegerType, Attribute, Block, BlockRef, Location,
        Region,
    },
    Context as MeliorContext,
};
use num_bigint::BigUint;

use super::context::CodegenCtx;
use crate::{
    errors::CodegenError,
    opcodes::Operation,
    utils::{check_stack_has_space_for, stack_push},
};

/// Generates blocks for target [`Operation`].
/// Returns both the starting block, and the unterminated last block of the generated code.
pub fn generate_code_for_op<'c, 'r>(
    context: CodegenCtx<'c>,
    region: &'r Region<'c>,
    op: Operation,
) -> Result<(BlockRef<'c, 'r>, BlockRef<'c, 'r>), CodegenError> {
    match op {
        Operation::Push32(x) => codegen_push(context, region, x),
        _ => todo!(),
    }
}

// TODO: use const generics to generalize for pushN
fn codegen_push<'c, 'r>(
    codegen_ctx: CodegenCtx<'c>,
    region: &'r Region<'c>,
    value_to_push: [u8; 32],
) -> Result<(BlockRef<'c, 'r>, BlockRef<'c, 'r>), CodegenError> {
    // TODO: handle stack overflow
    let start_block = region.append_block(Block::new(&[]));
    let context = &codegen_ctx.mlir_context;
    let location = Location::unknown(context);
    let uint256 = IntegerType::new(context, 256);

    // Check there's enough space in stack
    let flag = check_stack_has_space_for(context, &start_block, 1)?;

    // Create REVERT block
    // TODO: create only one revert block and use it for all revert operations
    let revert_block = region.append_block(Block::new(&[]));

    let exit_code = revert_block
        .append_operation(arith::constant(
            context,
            IntegerAttribute::new(uint256.into(), 1 as i64).into(),
            location,
        ))
        .result(0)?;

    revert_block.append_operation(func::r#return(&[exit_code.into()], location));

    let ok_block = region.append_block(Block::new(&[]));

    start_block.append_operation(cf::cond_br(
        context,
        flag,
        &ok_block,
        &revert_block,
        &[],
        &[],
        location,
    ));

    let constant_value = ok_block
        .append_operation(arith::constant(
            context,
            integer_constant(context, value_to_push),
            location,
        ))
        .result(0)?
        .into();

    stack_push(context, &ok_block, constant_value)?;

    Ok((start_block, ok_block))
}

fn integer_constant(context: &MeliorContext, value: [u8; 32]) -> Attribute {
    let str_value = BigUint::from_bytes_be(&value).to_string();
    // TODO: should we handle this error?
    Attribute::parse(context, &format!("{str_value} : i256")).unwrap()
}
