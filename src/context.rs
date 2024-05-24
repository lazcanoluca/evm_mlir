#![allow(dead_code)]
use llvm_sys::{
    core::{
        LLVMContextCreate, LLVMContextDispose, LLVMDisposeMessage, LLVMDisposeModule,
        LLVMPrintModuleToFile,
    },
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
    dialect::{
        arith, cf, func,
        llvm::{self, r#type::pointer, AllocaOptions, LoadStoreOptions},
        DialectRegistry,
    },
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
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    path::PathBuf,
    ptr::{addr_of_mut, null_mut},
    sync::OnceLock,
};

use crate::{
    codegen::{context::CodegenCtx, operations::generate_code_for_op, run_pass_manager},
    constants::{MAX_STACK_ELEMENTS, STACK_GLOBAL_VAR},
    module::MLIRModule,
    opcodes::Operation,
    utils::{llvm_mlir, load_from_stack},
};

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

    pub fn compile(&self, program: Vec<Operation>) -> Result<MLIRModule, String> {
        static INITIALIZED: OnceLock<()> = OnceLock::new();
        INITIALIZED.get_or_init(|| unsafe {
            LLVM_InitializeAllTargets();
            LLVM_InitializeAllTargetInfos();
            LLVM_InitializeAllTargetMCs();
            LLVM_InitializeAllAsmPrinters();
        });

        let target_triple = get_target_triple();

        let context = &self.melior_context;

        // Build a module with a single function
        let module_region = Region::new();
        let module_block = Block::new(&[]);

        module_region.append_block(module_block);

        let data_layout_ret = &get_data_layout_rep()?;

        // build main module
        let op = OperationBuilder::new("builtin.module", Location::unknown(context))
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

        let codegen_ctx = CodegenCtx {
            mlir_context: &self.melior_context,
            mlir_module: &melior_module,
            program: &program,
        };

        compile_program(codegen_ctx);

        assert!(melior_module.as_operation().verify());

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

        // Output MLIR
        std::fs::write("after-pass.mlir", melior_module.as_operation().to_string()).unwrap();

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
    register_all_passes();
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
            LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
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
// TODO: pass options to the function
pub fn compile_to_object(module: &MLIRModule<'_>) -> Result<PathBuf, String> {
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
            LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        let opts = LLVMCreatePassBuilderOptions();
        let opt = 0;
        let passes = CString::new(format!("default<O{opt}>")).unwrap();
        let error = LLVMRunPasses(llvm_module as *mut _, passes.as_ptr(), machine, opts);
        if !error.is_null() {
            let msg = LLVMGetErrorMessage(error);
            let msg = CStr::from_ptr(msg);
            Err(msg.to_string_lossy().into_owned())?;
        }

        LLVMDisposePassBuilderOptions(opts);

        // Output the LLVM IR
        let filename = CString::new(
            target_file
                .with_extension("ll")
                .as_os_str()
                .to_string_lossy()
                .as_bytes(),
        )
        .unwrap();
        if LLVMPrintModuleToFile(llvm_module, filename.as_ptr(), error_buffer) != 0 {
            let error = CStr::from_ptr(*error_buffer);
            let err = error.to_string_lossy().to_string();
            LLVMDisposeMessage(*error_buffer);
            Err(err)?;
        } else if !(*error_buffer).is_null() {
            LLVMDisposeMessage(*error_buffer);
            error_buffer = addr_of_mut!(null);
        }

        // Output the object file
        let filename = CString::new(target_file.as_os_str().to_string_lossy().as_bytes()).unwrap();
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
            LLVMDisposeMessage(*error_buffer);
            Err(err)?;
        } else if !(*error_buffer).is_null() {
            LLVMDisposeMessage(*error_buffer);
        }

        // Output the assembly
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
            LLVMCodeGenFileType::LLVMAssemblyFile,
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

fn compile_program(codegen_ctx: CodegenCtx) {
    let context = codegen_ctx.mlir_context;
    let module = codegen_ctx.mlir_module;
    let operations = codegen_ctx.program;

    let location = Location::unknown(context);

    let ptr_type = pointer(context, 0);
    let uint256 = IntegerType::new(context, 256);

    let body = module.body();
    let res = body.append_operation(llvm_mlir::global(
        context,
        STACK_GLOBAL_VAR,
        ptr_type,
        location,
    ));
    assert!(res.verify());

    // Build a region for the main function
    let main_region = Region::new();

    // Setup the stack, memory, etc.
    let setup_block = generate_stack_setup_block(context);

    let mut last_block = setup_block;

    // Generate code for the program
    for op in operations {
        let block = Block::new(&[]);

        generate_code_for_op(codegen_ctx, &block, op).unwrap();

        last_block.append_operation(cf::br(&block, &[], location));
        main_region.append_block(last_block);
        last_block = block;
    }

    let return_block = Block::new(&[]);
    last_block.append_operation(cf::br(&return_block, &[], location));
    main_region.append_block(last_block);

    // Setup return operation
    // This returns the last element of the stack
    let return_value = load_from_stack(context, &return_block);
    return_block.append_operation(func::r#return(&[return_value.into()], location));

    // Append the return operation
    main_region.append_block(return_block);

    let main_func = func::func(
        context,
        StringAttribute::new(context, "main"),
        TypeAttribute::new(FunctionType::new(context, &[], &[uint256.into()]).into()),
        main_region,
        &[],
        location,
    );

    module.body().append_operation(main_func);
}

fn generate_stack_setup_block(context: &MeliorContext) -> Block {
    let block = Block::new(&[]);
    let uint256 = IntegerType::new(context, 256);
    let location = Location::unknown(context);
    let ptr_type = pointer(context, 0);
    let stack_size = block
        .append_operation(arith::constant(
            context,
            IntegerAttribute::new(uint256.into(), MAX_STACK_ELEMENTS).into(),
            location,
        ))
        .result(0)
        .unwrap()
        .into();

    let stack_baseptr = block
        .append_operation(llvm::alloca(
            context,
            stack_size,
            ptr_type,
            location,
            AllocaOptions::new().elem_type(Some(TypeAttribute::new(uint256.into()))),
        ))
        .result(0)
        .unwrap();

    let stack_baseptr_ptr = block
        .append_operation(llvm_mlir::addressof(
            context,
            STACK_GLOBAL_VAR,
            ptr_type,
            location,
        ))
        .result(0)
        .unwrap();

    let res = block.append_operation(llvm::store(
        context,
        stack_baseptr.into(),
        stack_baseptr_ptr.into(),
        location,
        LoadStoreOptions::default(),
    ));

    assert!(res.verify());
    block
}
