module attributes {llvm.data_layout = "e-m:o-i64:64-i128:128-n32:64-S128", llvm.target_triple = "arm64-apple-darwin23.0.0"} {
  %0 = llvm.mlir.constant(1024 : i64) : i64
  %1 = llvm.alloca %0 x i256 {alignment = 8 : i64} : (i64) -> !llvm.ptr
}

