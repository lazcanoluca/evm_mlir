use melior::{ir::Module as MeliorModule, Context as MeliorContext};

use crate::program::Program;

/// Global codegen context
#[derive(Debug, Clone, Copy)]
pub(crate) struct CodegenCtx<'a> {
    /// The MLIR context.
    pub mlir_context: &'a MeliorContext,
    /// The MLIR module.
    pub mlir_module: &'a MeliorModule<'a>,
    /// The compile session info.
    // pub session: &'a Session,
    /// The program IR.
    pub program: &'a Program,
}
