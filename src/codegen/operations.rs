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
        Operation::Push32(x) => codegen_push(context, region, x),
        Operation::Add => codegen_add(context, region),
        Operation::Pop => codegen_pop(context, region),
        Operation::Byte => codegen_byte(context, region),
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

fn codegen_byte<'c, 'r>(
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

    let offset = stack_pop(context, &ok_block)?;
    let value = stack_pop(context, &ok_block)?;

    // the idea is to use left and right shifts in order to extract the
    // desired byte from the value, removing the rest of the bytes
    //
    // for example, if we want to extract the 0xFF byte in the following value
    // (for simplicity the value has fewer bytes than it has in reality)
    //
    // value = 0xAABBCCDDFFAABBCC
    //                   ^^
    //              desired byte
    //
    // we can shift the value to the left to remove the left-side bytes that
    // we don't care about
    //
    // value = 0xAABBCCDDFFAABBCC -> 0xFFAABBCC00000000
    //                   ^^            ^^
    // and then shift it to the right to remove the right-side bytes
    //
    // value = 0xFFAABBCC00000000 -> 0x00000000000000FF
    //           ^^                                  ^^

    // define the relevant constants
    const BITS_PER_BYTE: u8 = 8;
    const MAX_SHIFT: u8 = 31;
    let mut bits_per_byte: [u8; 32] = [0; 32];
    bits_per_byte[31] = BITS_PER_BYTE;

    let mut max_shift: [u8; 32] = [0; 32];
    max_shift[31] = MAX_SHIFT * BITS_PER_BYTE;

    let constant_bits_per_byte = ok_block
        .append_operation(arith::constant(
            context,
            integer_constant(context, bits_per_byte),
            location,
        ))
        .result(0)?
        .into();

    let constant_max_shift = ok_block
        .append_operation(arith::constant(
            context,
            integer_constant(context, max_shift),
            location,
        ))
        .result(0)?
        .into();

    // compute how many bits the value must be shifted to the left
    // shift_left = offset * bits_per_byte = offset * 8
    let shift_left = ok_block
        .append_operation(arith::muli(offset, constant_bits_per_byte, location))
        .result(0)?
        .into();

    let shifted_left_value = ok_block
        .append_operation(arith::shli(value, shift_left, location))
        .result(0)?
        .into();

    let result = ok_block
        .append_operation(arith::shrui(
            shifted_left_value,
            constant_max_shift,
            location,
        ))
        .result(0)?
        .into();

    stack_push(context, &ok_block, result)?;
    Ok((start_block, ok_block))
}

fn integer_constant(context: &MeliorContext, value: [u8; 32]) -> Attribute {
    let str_value = BigUint::from_bytes_be(&value).to_string();
    // TODO: should we handle this error?
    Attribute::parse(context, &format!("{str_value} : i256")).unwrap()
}
