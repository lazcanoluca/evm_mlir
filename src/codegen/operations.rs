use melior::{
    dialect::{arith, cf},
    ir::{Attribute, Block, BlockRef, Location, Region},
    Context as MeliorContext,
};
use num_bigint::BigUint;

use super::context::CodegenCtx;
use crate::{
    errors::CodegenError,
    opcodes::Operation,
    utils::{
        check_stack_has_at_least, check_stack_has_space_for, revert_block, stack_pop, stack_push,
    },
};

/// Generates blocks for target [`Operation`].
/// Returns both the starting block, and the unterminated last block of the generated code.
pub fn generate_code_for_op<'c, 'r>(
    context: CodegenCtx<'c>,
    region: &'r Region<'c>,
    op: Operation,
) -> Result<(BlockRef<'c, 'r>, BlockRef<'c, 'r>), CodegenError> {
    match op {
        Operation::Push0 => codegen_push(context, region, [0; 32]),
        Operation::Push1(x) => codegen_push(context, region, x),
        Operation::Push2(x) => codegen_push(context, region, x),
        Operation::Push3(x) => codegen_push(context, region, x),
        Operation::Push4(x) => codegen_push(context, region, x),
        Operation::Push5(x) => codegen_push(context, region, x),
        Operation::Push6(x) => codegen_push(context, region, x),
        Operation::Push7(x) => codegen_push(context, region, x),
        Operation::Push8(x) => codegen_push(context, region, x),
        Operation::Push9(x) => codegen_push(context, region, x),
        Operation::Push10(x) => codegen_push(context, region, x),
        Operation::Push11(x) => codegen_push(context, region, x),
        Operation::Push12(x) => codegen_push(context, region, x),
        Operation::Push13(x) => codegen_push(context, region, x),
        Operation::Push14(x) => codegen_push(context, region, x),
        Operation::Push15(x) => codegen_push(context, region, x),
        Operation::Push16(x) => codegen_push(context, region, x),
        Operation::Push17(x) => codegen_push(context, region, x),
        Operation::Push18(x) => codegen_push(context, region, x),
        Operation::Push19(x) => codegen_push(context, region, x),
        Operation::Push20(x) => codegen_push(context, region, x),
        Operation::Push21(x) => codegen_push(context, region, x),
        Operation::Push22(x) => codegen_push(context, region, x),
        Operation::Push23(x) => codegen_push(context, region, x),
        Operation::Push24(x) => codegen_push(context, region, x),
        Operation::Push25(x) => codegen_push(context, region, x),
        Operation::Push26(x) => codegen_push(context, region, x),
        Operation::Push27(x) => codegen_push(context, region, x),
        Operation::Push28(x) => codegen_push(context, region, x),
        Operation::Push29(x) => codegen_push(context, region, x),
        Operation::Push30(x) => codegen_push(context, region, x),
        Operation::Push31(x) => codegen_push(context, region, x),
        Operation::Push32(x) => codegen_push(context, region, x),
        Operation::Add => codegen_add(context, region),
        Operation::Pop => codegen_pop(context, region),
    }
}

fn codegen_push<'c, 'r, const N: usize>(
    codegen_ctx: CodegenCtx<'c>,
    region: &'r Region<'c>,
    value_to_push: [u8; N],
) -> Result<(BlockRef<'c, 'r>, BlockRef<'c, 'r>), CodegenError> {
    let start_block = region.append_block(Block::new(&[]));
    let context = &codegen_ctx.mlir_context;
    let location = Location::unknown(context);

    // Check there's enough space in stack
    let flag = check_stack_has_space_for(context, &start_block, 1)?;

    // Create REVERT block
    let revert_block = region.append_block(revert_block(context)?);

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

fn codegen_add<'c, 'r>(
    codegen_ctx: CodegenCtx<'c>,
    region: &'r Region<'c>,
) -> Result<(BlockRef<'c, 'r>, BlockRef<'c, 'r>), CodegenError> {
    let start_block = region.append_block(Block::new(&[]));
    let context = &codegen_ctx.mlir_context;
    let location = Location::unknown(context);

    // Check there's enough elements in stack
    let flag = check_stack_has_at_least(context, &start_block, 2)?;

    // Create REVERT block
    let revert_block = region.append_block(revert_block(context)?);

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

    let lhs = stack_pop(context, &ok_block)?;
    let rhs = stack_pop(context, &ok_block)?;

    let result = ok_block
        .append_operation(arith::addi(lhs, rhs, location))
        .result(0)?
        .into();

    stack_push(context, &ok_block, result)?;

    Ok((start_block, ok_block))
}

fn codegen_pop<'c, 'r>(
    codegen_ctx: CodegenCtx<'c>,
    region: &'r Region<'c>,
) -> Result<(BlockRef<'c, 'r>, BlockRef<'c, 'r>), CodegenError> {
    let start_block = region.append_block(Block::new(&[]));
    let context = &codegen_ctx.mlir_context;
    let location = Location::unknown(context);

    // Check there's at least 1 element in stack
    let flag = check_stack_has_at_least(context, &start_block, 1)?;

    // Create REVERT block
    let revert_block = region.append_block(revert_block(context)?);

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

    stack_pop(context, &ok_block)?;

    Ok((start_block, ok_block))
}

fn integer_constant<const N: usize>(context: &MeliorContext, value: [u8; N]) -> Attribute {
    let str_value = BigUint::from_bytes_be(&value).to_string();
    // TODO: should we handle this error?
    Attribute::parse(context, &format!("{str_value} : i256")).unwrap()
}
