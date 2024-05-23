module attributes {llvm.data_layout = "e-m:o-i64:64-i128:128-n32:64-S128", llvm.target_triple = "arm64-apple-darwin23.0.0"} {
  %c1024_i64 = arith.constant 1024 : i64
  %0 = llvm.alloca %c1024_i64 x i256 {alignment = 8 : i64} : (i64) -> !llvm.ptr
}
