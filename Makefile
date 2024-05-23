.PHONY: check-deps
check-deps:
	ifeq (, $(shell which cargo))
		$(error "The cargo command could not be found in your PATH, please install Rust: https://www.rust-lang.org/tools/install")
	endif
	ifndef LLVM_SYS_180_PREFIX
		$(error Could not find a suitable LLVM 18 toolchain, please set LLVM_SYS_180_PREFIX env pointing to the LLVM 18 dir)
	endif
	ifndef MLIR_SYS_180_PREFIX
		$(error Could not find a suitable LLVM 18 toolchain (mlir), please set MLIR_SYS_180_PREFIX env pointing to the LLVM 18 dir)
	endif
	ifndef TABLEGEN_180_PREFIX
		$(error Could not find a suitable LLVM 18 toolchain (tablegen), please set TABLEGEN_180_PREFIX env pointing to the LLVM 18 dir)
	endif
		@echo "[make] LLVM is correctly set at $(MLIR_SYS_180_PREFIX)."

lint:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-features --benches --examples --tests -- -D warnings

fmt:
	cargo fmt --all

test:
	cargo nextest run --workspace --all-features
