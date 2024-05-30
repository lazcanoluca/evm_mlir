use melior::{
    dialect::{arith, cf},
    ir::{Block, BlockRef, Location, Region},
};

use super::context::CodegenCtx;
use crate::{
    errors::CodegenError,
    opcodes::Operation,
    utils::{
        check_denominator_is_zero, check_stack_has_at_least, check_stack_has_space_for,
        integer_constant, revert_block, stack_pop, stack_push, u256_bytes_from_u32,
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
        Operation::Push32(x) => codegen_push(context, region, x),
        Operation::Add => codegen_add(context, region),
        Operation::Div => codegen_div(context, region),
    }
}

// TODO: use const generics to generalize for pushN
fn codegen_push<'c, 'r>(
    codegen_ctx: CodegenCtx<'c>,
    region: &'r Region<'c>,
    value_to_push: [u8; 32],
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

fn codegen_div<'c, 'r>(
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

    let num = stack_pop(context, &ok_block)?;
    let den = stack_pop(context, &ok_block)?;

    let den_is_zero = check_denominator_is_zero(context, &ok_block, &den)?;
    let den_zero_bloq = region.append_block(Block::new(&[]));
    let den_not_zero_bloq = region.append_block(Block::new(&[]));
    let return_block = region.append_block(Block::new(&[]));

    // Denominator is zero path
    let constant_value = den_zero_bloq
        .append_operation(arith::constant(
            context,
            integer_constant(context, u256_bytes_from_u32(0u32)),
            location,
        ))
        .result(0)?
        .into();

    stack_push(context, &den_zero_bloq, constant_value)?;

    den_zero_bloq.append_operation(cf::br(&return_block, &[], location));

    // Denominator is not zero path
    let result = den_not_zero_bloq
        .append_operation(arith::divui(num, den, location))
        .result(0)?
        .into();

    stack_push(context, &den_not_zero_bloq, result)?;

    den_not_zero_bloq.append_operation(cf::br(&return_block, &[], location));

    ok_block.append_operation(cf::cond_br(
        context,
        den_is_zero,
        &den_zero_bloq,
        &den_not_zero_bloq,
        &[],
        &[],
        location,
    ));

    Ok((start_block, return_block))
}
