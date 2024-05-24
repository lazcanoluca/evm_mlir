use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    path::{Path, PathBuf},
    ptr::{addr_of_mut, null_mut},
    sync::OnceLock,
};

use errors::CodegenError;
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
use mlir_sys::mlirTranslateModuleToLLVMIR;
use module::MLIRModule;
use opcodes::Operation;

use crate::context::Context;

pub mod codegen;
pub mod constants;
pub mod context;
pub mod errors;
pub mod module;
pub mod opcodes;
pub mod utils;

pub fn compile(program: Vec<Operation>) -> Result<PathBuf, CodegenError> {
    static INITIALIZED: OnceLock<()> = OnceLock::new();
    INITIALIZED.get_or_init(|| unsafe {
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmPrinters();
    });
    let context = Context::new();
    let mlir_module = context.compile(&program)?;
    compile_to_object(&mlir_module)
}

/// Converts a module to an object.
/// The object will be written to the specified target path.
/// TODO: error handling
///
/// Returns the path to the object.
// TODO: pass options to the function
pub fn compile_to_object(module: &MLIRModule<'_>) -> Result<PathBuf, CodegenError> {
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
            return Err(CodegenError::LLVMCompileError(err));
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
            return Err(CodegenError::LLVMCompileError(
                msg.to_string_lossy().into_owned(),
            ));
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
            return Err(CodegenError::LLVMCompileError(err));
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
            return Err(CodegenError::LLVMCompileError(err));
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
            return Err(CodegenError::LLVMCompileError(err));
        } else if !(*error_buffer).is_null() {
            LLVMDisposeMessage(*error_buffer);
        }

        LLVMDisposeTargetMachine(machine);
        LLVMDisposeModule(llvm_module);
        LLVMContextDispose(llvm_context);

        Ok(target_file)
    }
}

/// Links object file to produce an executable binary
pub fn link_binary(object_file: impl AsRef<Path>, output_file: impl AsRef<Path>) {
    let args = vec![
        "-L/usr/local/lib",
        "-L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib",
        object_file.as_ref().to_str().unwrap(),
        "-o",
        output_file.as_ref().to_str().unwrap(),
        "-lSystem",
    ];
    let mut linker = std::process::Command::new("ld");
    let proc = linker.args(args).spawn().unwrap();
    let output = proc.wait_with_output().unwrap();
    assert!(output.status.success());
}

pub fn compile_binary(
    program: Vec<Operation>,
    output_file: impl AsRef<Path>,
) -> Result<(), CodegenError> {
    let object_file = compile(program)?;
    link_binary(object_file, &output_file);
    Ok(())
}
