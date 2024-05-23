#[allow(dead_code)]
use std::{
    alloc::Layout,
    ffi::{CStr, CString},
    mem::MaybeUninit,
    path::PathBuf,
    ptr::{addr_of_mut, null_mut},
    sync::OnceLock,
};

use llvm_sys::{
    core::{LLVMContextCreate, LLVMContextDispose, LLVMDisposeMessage, LLVMDisposeModule},
    error::LLVMGetErrorMessage,
    target::{
        LLVM_InitializeAllAsmPrinters, LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargetMCs,
        LLVM_InitializeAllTargets,
    },
    target_machine::{
        LLVMCodeGenFileType, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetMachine,
        LLVMDisposeTargetMachine, LLVMGetDefaultTargetTriple, LLVMGetHostCPUFeatures,
        LLVMGetHostCPUName, LLVMGetTargetFromTriple, LLVMRelocMode, LLVMTargetMachineEmitToFile,
        LLVMTargetRef,
    },
    transforms::pass_builder::{
        LLVMCreatePassBuilderOptions, LLVMDisposePassBuilderOptions, LLVMRunPasses,
    },
};
use melior::{
    dialect::{arith, func, llvm, DialectRegistry},
    ir::{
        attribute::{IntegerAttribute, StringAttribute, TypeAttribute},
        operation::OperationBuilder,
        r#type::{FunctionType, IntegerType},
        Block, Identifier, Location, Module as MeliorModule, Region,
    },
    utility::{register_all_dialects, register_all_llvm_translations, register_all_passes},
    Context as MeliorContext,
};
use mlir_sys::mlirTranslateModuleToLLVMIR;

use crate::{
    codegen::run_pass_manager,
    opcodes::{Opcode, Operation},
};

use super::module::MLIRModule;

#[derive(Debug, Eq, PartialEq)]
pub struct Context {
    pub melior_context: MeliorContext,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        let melior_context = initialize_mlir();
        Self { melior_context }
    }

    pub fn compile(&self, _program: Vec<Operation>) -> Result<MLIRModule, String> {
        static INITIALIZED: OnceLock<()> = OnceLock::new();
        INITIALIZED.get_or_init(|| unsafe {
            LLVM_InitializeAllTargets();
            LLVM_InitializeAllTargetInfos();
            LLVM_InitializeAllTargetMCs();
            LLVM_InitializeAllAsmPrinters();
        });

        let target_triple = get_target_triple();

        let context = &self.melior_context;

        let location = Location::unknown(context);

        let uint256 = IntegerType::new(context, 256);

        // Build a single region with a single block with a constant and a return operation
        let func_region = Region::new();
        let func_block = Block::new(&[]);

        let constant_1024 = func_block
            .append_operation(arith::constant(
                context,
                IntegerAttribute::new(uint256.into(), 42).into(),
                location,
            ))
            .result(0)
            .unwrap()
            .into();

        // let result_layout = Layout::from_size_align(64, 8).unwrap();
        // let _op = block
        //     .append_operation(
        //         OperationBuilder::new("llvm.alloca", location)
        //             .add_attributes(&[
        //                 (
        //                     Identifier::new(context, "alignment"),
        //                     IntegerAttribute::new(
        //                         IntegerType::new(context, 64).into(),
        //                         result_layout.align().try_into().unwrap(),
        //                     )
        //                     .into(),
        //                 ),
        //                 (
        //                     Identifier::new(context, "elem_type"),
        //                     TypeAttribute::new(IntegerType::new(context, 256).into())
        //                         .into(),
        //                 ),
        //             ])
        //             .add_operands(&[constant_1024])
        //             .add_results(&[llvm::r#type::pointer(context, 0)])
        //             .build()
        //             .unwrap(),
        //     )
        //     .result(0)
        //     .unwrap();
        func_block.append_operation(func::r#return(&[constant_1024], Location::unknown(context)));
        func_region.append_block(func_block);

        // Build a module with a single function
        let module_region = Region::new();
        let module_block = Block::new(&[]);

        module_block.append_operation(
            func::func(
                context,
                StringAttribute::new(context, "main"),
                TypeAttribute::new(FunctionType::new(context, &[], &[uint256.into()]).into()),
                func_region,
                &[],
                Location::unknown(context),
            )
            .into(),
        );

        module_region.append_block(module_block);

        let data_layout_ret = &get_data_layout_rep()?;

        // build main module
        let op = OperationBuilder::new("builtin.module", location)
            .add_attributes(&[
                (
                    Identifier::new(context, "llvm.target_triple"),
                    StringAttribute::new(context, &target_triple).into(),
                ),
                (
                    Identifier::new(context, "llvm.data_layout"),
                    StringAttribute::new(context, data_layout_ret).into(),
                ),
            ])
            .add_regions([module_region])
            .build()
            .map_err(|_| "failed to build module operation")?;
        assert!(op.verify(), "module operation is not valid");

        let mut melior_module = MeliorModule::from_operation(op).expect("module failed to create");

        // TODO: here we should wire the call to the specific code generation for each opcode

        // let codegen_ctx = CodegenCtx {
        //     mlir_context: context,
        //     session,
        //     mlir_module: &melior_module,
        //     program,
        // };

        // super::codegen::compile_program(codegen_ctx)?;

        assert!(melior_module.as_operation().verify());

        dbg!(melior_module.as_operation().to_string());
        std::fs::write(
            PathBuf::from("generated.mlir"),
            melior_module.as_operation().to_string(),
        )
        .unwrap();

        // TODO: Add proper error handling.
        run_pass_manager(context, &mut melior_module).unwrap();

        // The func to llvm pass has a bug where it sets the data layout string to ""
        // This works around it by setting it again.
        {
            let mut op = melior_module.as_operation_mut();
            op.set_attribute(
                "llvm.data_layout",
                StringAttribute::new(context, data_layout_ret).into(),
            );
        }

        // if session.output_mlir {
        //     std::fs::write(
        //         session.output_file.with_extension("after-pass.mlir"),
        //         melior_module.as_operation().to_string(),
        //     )?;
        // }

        Ok(MLIRModule::new(melior_module))
    }
}

/// Initialize an MLIR context.
pub fn initialize_mlir() -> MeliorContext {
    let context = MeliorContext::new();
    context.append_dialect_registry(&{
        let registry = DialectRegistry::new();
        register_all_dialects(&registry);
        registry
    });
    context.load_all_available_dialects();
    // register_all_passes();
    register_all_llvm_translations(&context);
    context
}

pub fn get_target_triple() -> String {
    let target_triple = unsafe {
        let value = LLVMGetDefaultTargetTriple();
        CStr::from_ptr(value).to_string_lossy().into_owned()
    };
    target_triple
}

pub fn get_data_layout_rep() -> Result<String, String> {
    unsafe {
        let mut null = null_mut();
        let error_buffer = addr_of_mut!(null);

        let target_triple = LLVMGetDefaultTargetTriple();

        {
            let target_triple = unsafe {
                let value = LLVMGetDefaultTargetTriple();
                CStr::from_ptr(value).to_string_lossy().into_owned()
            };
        }
        let target_cpu = LLVMGetHostCPUName();
        let target_cpu_features = LLVMGetHostCPUFeatures();

        let mut target: MaybeUninit<LLVMTargetRef> = MaybeUninit::uninit();

        if LLVMGetTargetFromTriple(target_triple, target.as_mut_ptr(), error_buffer) != 0 {
            let error = CStr::from_ptr(*error_buffer);
            let err = error.to_string_lossy().to_string();
            dbg!(err.clone());
            LLVMDisposeMessage(*error_buffer);
            Err(err)?;
        }
        if !(*error_buffer).is_null() {
            LLVMDisposeMessage(*error_buffer);
        }

        let target = target.assume_init();

        let machine = LLVMCreateTargetMachine(
            target,
            target_triple.cast(),
            target_cpu.cast(),
            target_cpu_features.cast(),
            // match session.optlevel {
            //     OptLevel::None => LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            //     OptLevel::Less => LLVMCodeGenOptLevel::LLVMCodeGenLevelLess,
            //     OptLevel::Default => LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            //     OptLevel::Aggressive => LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
            // },
            LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            // if session.library {
            //     LLVMRelocMode::LLVMRelocDynamicNoPic
            // } else {
            //     LLVMRelocMode::LLVMRelocDefault
            // },
            LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        let data_layout = llvm_sys::target_machine::LLVMCreateTargetDataLayout(machine);
        let data_layout_str =
            CStr::from_ptr(llvm_sys::target::LLVMCopyStringRepOfTargetData(data_layout));
        Ok(data_layout_str.to_string_lossy().into_owned())
    }
}

/// Converts a module to an object.
/// The object will be written to the specified target path.
/// TODO: error handling
///
/// Returns the path to the object.
pub fn compile_to_object(
    // session: &Session,
    module: &MLIRModule<'_>,
) -> Result<PathBuf, String> {
    // TODO: put a proper target_file here
    let target_file = PathBuf::from("output.o");
    // let target_file = session.output_file.with_extension("o");

    // TODO: Rework so you can specify target and host features, etc.
    // Right now it compiles for the native cpu feature set and arch

    unsafe {
        let llvm_context = LLVMContextCreate();

        let op = module.melior_module.as_operation().to_raw();

        let llvm_module = mlirTranslateModuleToLLVMIR(op, llvm_context as *mut _) as *mut _;

        let mut null = null_mut();
        let mut error_buffer = addr_of_mut!(null);

        let target_triple = LLVMGetDefaultTargetTriple();

        let target_cpu = LLVMGetHostCPUName();

        let target_cpu_features = LLVMGetHostCPUFeatures();

        let mut target: MaybeUninit<LLVMTargetRef> = MaybeUninit::uninit();

        if LLVMGetTargetFromTriple(target_triple, target.as_mut_ptr(), error_buffer) != 0 {
            let error = CStr::from_ptr(*error_buffer);
            let err = error.to_string_lossy().to_string();
            LLVMDisposeMessage(*error_buffer);
            Err(err)?;
        } else if !(*error_buffer).is_null() {
            LLVMDisposeMessage(*error_buffer);
            error_buffer = addr_of_mut!(null);
        }

        let target = target.assume_init();

        let machine = LLVMCreateTargetMachine(
            target,
            target_triple.cast(),
            target_cpu.cast(),
            target_cpu_features.cast(),
            // match session.optlevel {
            //     OptLevel::None => LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            //     OptLevel::Less => LLVMCodeGenOptLevel::LLVMCodeGenLevelLess,
            //     OptLevel::Default => LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            //     OptLevel::Aggressive => LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
            // },
            LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            // if session.library {
            //     LLVMRelocMode::LLVMRelocDynamicNoPic
            // } else {
            //     LLVMRelocMode::LLVMRelocDefault
            // },
            LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        let opts = LLVMCreatePassBuilderOptions();
        // let opt = match session.optlevel {
        //     OptLevel::None => 0,
        //     OptLevel::Less => 1,
        //     OptLevel::Default => 2,
        //     OptLevel::Aggressive => 3,
        // };
        let opt = 0;
        let passes = CString::new(format!("default<O{opt}>")).unwrap();
        let error = LLVMRunPasses(llvm_module as *mut _, passes.as_ptr(), machine, opts);
        if !error.is_null() {
            let msg = LLVMGetErrorMessage(error);
            let msg = CStr::from_ptr(msg);
            Err(msg.to_string_lossy().into_owned())?;
        }

        LLVMDisposePassBuilderOptions(opts);

        // if session.output_ll {
        //     let filename = CString::new(
        //         target_file
        //             .with_extension("ll")
        //             .as_os_str()
        //             .to_string_lossy()
        //             .as_bytes(),
        //     )
        //     .unwrap();
        //     if LLVMPrintModuleToFile(llvm_module, filename.as_ptr(), error_buffer) != 0 {
        //         let error = CStr::from_ptr(*error_buffer);
        //         let err = error.to_string_lossy().to_string();
        //         tracing::error!("error outputing ll file: {}", err);
        //         LLVMDisposeMessage(*error_buffer);
        //         Err(CodegenError::LLVMCompileError(err))?;
        //     } else if !(*error_buffer).is_null() {
        //         LLVMDisposeMessage(*error_buffer);
        //         error_buffer = addr_of_mut!(null);
        //     }
        // }

        let filename = CString::new(target_file.as_os_str().to_string_lossy().as_bytes()).unwrap();
        // tracing::debug!("filename to llvm: {:?}", filename);
        let ok = LLVMTargetMachineEmitToFile(
            machine,
            llvm_module,
            filename.as_ptr().cast_mut(),
            LLVMCodeGenFileType::LLVMObjectFile, // object (binary) or assembly (textual)
            error_buffer,
        );

        if ok != 0 {
            let error = CStr::from_ptr(*error_buffer);
            let err = error.to_string_lossy().to_string();
            // tracing::error!("error emitting to file: {:?}", err);
            LLVMDisposeMessage(*error_buffer);
            Err(err)?;
        } else if !(*error_buffer).is_null() {
            LLVMDisposeMessage(*error_buffer);
        }

        let filename = CString::new(
            target_file
                .with_extension("asm")
                .as_os_str()
                .to_string_lossy()
                .as_bytes(),
        )
        .unwrap();
        let ok = LLVMTargetMachineEmitToFile(
            machine,
            llvm_module,
            filename.as_ptr().cast_mut(),
            LLVMCodeGenFileType::LLVMAssemblyFile, // object (binary) or assembly (textual)
            error_buffer,
        );

        if ok != 0 {
            let error = CStr::from_ptr(*error_buffer);
            let err = error.to_string_lossy().to_string();
            LLVMDisposeMessage(*error_buffer);
            Err(err)?;
        } else if !(*error_buffer).is_null() {
            LLVMDisposeMessage(*error_buffer);
        }

        LLVMDisposeTargetMachine(machine);
        LLVMDisposeModule(llvm_module);
        LLVMContextDispose(llvm_context);

        Ok(target_file)
    }
}

fn codegen_push1(context: &Context) -> Result<(), String> {
    let location = Location::unknown(&context.melior_context);

    let module_region = Region::new();
    module_region.append_block(Block::new(&[]));

    let op = OperationBuilder::new("builtin.push1", location)
        .add_regions([module_region])
        .build()
        .map_err(|_| "failed to build push1 operation")?;
    assert!(op.verify(), "push1 operation is not valid");

    Ok(())
}
