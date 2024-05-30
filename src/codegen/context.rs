use std::collections::BTreeMap;

use melior::{ir::BlockRef, Context as MeliorContext};

use crate::program::Program;

#[derive(Debug, Clone)]
pub(crate) struct OperationCtx<'c> {
    /// The MLIR context.
    pub mlir_context: &'c MeliorContext,
    /// The MLIR module.
    // pub mlir_module: &'c MeliorModule<'c>,
    /// The compile session info.
    // pub session: &'c Session,
    /// The program IR.
    pub program: &'c Program,
    /// Reference to the revert block.
    /// This block takes care of reverts.
    pub revert_block: BlockRef<'c, 'c>,
    /// Reference to the jump table block.
    /// This block receives the PC as an argument and jumps to the block corresponding to that PC,
    /// or reverts in case the destination is not a JUMPDEST.
    pub jumptable_block: BlockRef<'c, 'c>,
    /// Blocks to jump to. This are registered dynamically as JUMPDESTs are processed.
    pub jumpdest_blocks: BTreeMap<usize, BlockRef<'c, 'c>>,
}
