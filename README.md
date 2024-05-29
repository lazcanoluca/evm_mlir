# evm_mlir

An EVM bytecode compiler written with MLIR.

## Getting Started

### Dependencies

- Linux or macOS (aarch64 included) only for now
- LLVM 18 with MLIR: On debian you can use [apt.llvm.org](https://apt.llvm.org/), on macOS you can use brew
- Rust
- Git

### Setup

> This step applies to all operating systems.

Run the following make target to install the dependencies (**both Linux and macOS**):

```bash
make deps
```

#### Linux

Since Linux distributions change widely, you need to install LLVM 18 via your package manager, compile it or check if the current release has a Linux binary.

If you are on Debian/Ubuntu, check out the repository https://apt.llvm.org/
Then you can install with:

```bash
sudo apt-get install llvm-18 llvm-18-dev llvm-18-runtime clang-18 clang-tools-18 lld-18 libpolly-18-dev libmlir-18-dev mlir-18-tools
```

If you decide to build from source, here are some indications:

<details><summary>Install LLVM from source instructions</summary>

```bash
# Go to https://github.com/llvm/llvm-project/releases
# Download the latest LLVM 18 release:
# The blob to download is called llvm-project-18.x.x.src.tar.xz

# For example
wget https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.4/llvm-project-18.1.4.src.tar.xz
tar xf llvm-project-18.1.4.src.tar.xz

cd llvm-project-18.1.4.src.tar
mkdir build
cd build

# The following cmake command configures the build to be installed to /opt/llvm-18
cmake -G Ninja ../llvm \
   -DLLVM_ENABLE_PROJECTS="mlir;clang;clang-tools-extra;lld;polly" \
   -DLLVM_BUILD_EXAMPLES=OFF \
   -DLLVM_TARGETS_TO_BUILD="Native" \
   -DCMAKE_INSTALL_PREFIX=/opt/llvm-18 \
   -DCMAKE_BUILD_TYPE=RelWithDebInfo \
   -DLLVM_PARALLEL_LINK_JOBS=4 \
   -DLLVM_ENABLE_BINDINGS=OFF \
   -DCMAKE_C_COMPILER=clang -DCMAKE_CXX_COMPILER=clang++ -DLLVM_ENABLE_LLD=ON \
   -DLLVM_ENABLE_ASSERTIONS=OFF

ninja install
```

</details>

Setup a environment variable called `MLIR_SYS_180_PREFIX`, `LLVM_SYS_180_PREFIX` and `TABLEGEN_180_PREFIX` pointing to the llvm directory:

```bash
# For Debian/Ubuntu using the repository, the path will be /usr/lib/llvm-18
export MLIR_SYS_180_PREFIX=/usr/lib/llvm-18
export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18
export TABLEGEN_180_PREFIX=/usr/lib/llvm-18
```

Run the deps target to install the other dependencies.

```bash
make deps
```

#### MacOS

The makefile `deps` target (which you should have ran before) installs LLVM 18 with brew for you, afterwards you need to execute the `env-macos.sh` script to setup the environment.

```bash
source scripts/env-macos.sh
```

### Running

To run the compiler, call `cargo run` while passing it a file with the EVM bytecode to compile.
There are some example files under `programs/`, for example:

```bash
cargo run programs/push32.bytecode
```
