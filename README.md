Cadote: Compiler-Aided Development of Trusted Enclaves
======================================================

This is the code for [my Masters Thesis](https://www.f30.me/files/masters-thesis.pdf) and our ARES 2022 paper [*Compiler-Aided Development of Trusted Enclaves with Rust*](https://doi.org/10.1145/3538969.3538972).
It is quite hacky research-grade code, where I had to go to the limit of what is possible with an LLVM pass.
Also, there is quite some C++ code and I'm not much of a C++ programmer.
Sorry about that.

For details on the idea and its limitations, refer to [the paper](https://doi.org/10.1145/3538969.3538972) or [the thesis](https://www.f30.me/files/masters-thesis.pdf).
When referencing this work, please cite the paper:

```
@inproceedings{
  author = {Felix Dreissig and Jonas Röckl and Tilo Müller},
  title = {Compiler-Aided Development of Trusted Enclaves with Rust},
  booktitle = {Proceedings of the 17th International Conference on Availability, Reliability and Security},
  series = {ARES 2022},
  publisher = {Association for Computing Machinery},
  date = {2022-08},
  location = {Vienna, Austria},
  doi = {10.1145/3538969.3538972}
}
```

Contents
--------
* "examples" contains sample code using our implementation.
  * "EnclavizationPass" is a test program for all kinds of parameter passing to/from enclaved functions.
  * "IndirectionPass" contains sample code for "llvm-pass" (see below).
  * "maxisign" is an example for creating and validating signatures with a key bound to an enclave.
  * "password_check" is an example for checking passwords against values stored bound to an enclave.
* "llvm-pass" was a first PoC for an LLVM pass wrapping functions.
* "llvm-patches" contains patches (in `git format-patch` format) for the [rust-lang/llvm-project](https://github.com/rust-lang/llvm-project) repository, branch "rustc/11.0-2020-10-12". Large parts of Cadote's implementation live in here.
* "rust-enclave" was a first PoC for an enclave written in Rust.
* "rust-patches" contains patches (in `git format-patch` format) for the [rust-lang/rust](https://github.com/rust-lang/rust) repository, version nightly-2020-10-25 (commit "ffa2e7ae8", used by Teaclave SGX SDK 1.1.3).
* "support" contains helper code for the Cadote build
  * "cadote_\*_runtime" are Cadote's runtime libraries. This is the other major part of the Cadote implemenetation.
  * "build.rs" ensures the linking of untrusted applications to SGX bridges and proxies.
  * "enclave-config.xml" is config for Intel's SGX build tooling.
  * "enclave.lds" is a linker script as recommended by Intel.

### Evaluation
Everything related to the performance evaluation from the thesis is provided on the "eval" branch of this repository.
That includes the code and tools as well as the raw results.

Setup
-----
### System
The code has been primarily developed on Ubuntu 20.04, though I don't see any hard requirement on the specific distribution.
Obviously, an Intel CPU supporting SGX is required.
(Simulator mode might work, but has not been tested.)

### SGX
Intel's SGX tooling for Linux is required.
I used version 2.13.
However, the [installation guide for 2.14](https://download.01.org/intel-sgx/sgx-linux/2.14/docs/Intel_SGX_SW_Installation_Guide_for_Linux.pdf) is much better than for previous versions.

The tooling consists of three parts:

* DCAP driver: I never installed this myself, but the instructions from the Installation Guide seem comprehensive.
  Instructions to build from source are provided in the [driver's REAMDE file](https://github.com/intel/SGXDataCenterAttestationPrimitives/blob/master/driver/linux/README.md)
* Platform Software (PSW): I neither installed that myself. In addition to the installation guide, some documentation on it is provided [in the linux-sgx README](https://github.com/intel/linux-sgx#install-the-intelr-sgx-psw-1).
  Installation should involve adding Intel's APT repository and installing some "libsgx-\*" packages.
* Software Development Kit (SDK): Basically execute the installer as explained [in the linux-sgx README](https://github.com/intel/linux-sgx#install-the-intelr-sgx-sdk-1).
  It will ask you where to install the SGX, which can be any directory ("~/.sgxsdk" in my case).
  For usage, some environment variables have to be set by `source`-ing `<install-path>/environment`.

### LLVM
A custom build of LLVM is required.
For this, you have to get the right version of Rust's LLVM fork and apply Cadote's patches:

1. Clone the [rust-lang/llvm-project](https://github.com/rust-lang/llvm-project) repository.
2. Check out the correct branch for our Rust release, which is "rustc/11.0-2020-10-12".
3. Apply the patches by running `git am <repo-path>/llvm-patches/*`, where `<repo-path>` is the working copy of this very repository.
4. CMake and a basic C++ compiler toolchain (Debian package "build-essential") are required.
5. Create a "build" directory within your working copy of the LLVM repo and change to it.
6. Configure using:
   ```
   cmake -DLLVM_ENABLE_PROJECTS=clang -DLLVM_INSTALL_UTILS=ON -DLLVM_BUILD_LLVM_DYLIB=ON -DBUILD_SHARED_LIBS=ON -DCMAKE_INSTALL_PREFIX=/usr/local/lib/llvm-11-rs-debug -DCMAKE_BUILD_TYPE=Debug -DLLVM_ENABLE_ASSERTIONS=ON -GNinja -DLLVM_PARALLEL_COMPILE_JOBS=6 -DLLVM_PARALLEL_LINK_JOBS=6 ../llvm
   ```
   * `LLVM_ENABLE_PROJECTS=clang` and `LLVM_INSTALL_UTILS` would maybe not be necessary, but it feels more complete with them.
   * `LLVM_BUILD_LLVM_DYLIB` and `BUILD_SHARED_LIBS` are required for Rust.
   * `CMAKE_INSTALL_PREFIX` may be adjusted.
   * `CMAKE_BUILD_TYPE=Debug` allows using `-debug` for more verbose error messages.
   * `LLVM_ENABLE_ASSERTIONS` is required for Cadote's error handling.
   * `-GNinja`, `LLVM_PARALLEL_COMPILE_JOBS` and `LLVM_PARALLEL_LINK_JOBS` may also be adjusted.
7. Build by running `cmake --build .`.
8. Install by running `cmake --build . --target install` as root.

Further build instructions are provided [by LLVM](https://llvm.org/docs/CMake.html).

### Rust
We also need a custom build of the Rust compiler.
That depends on the custom LLVM build, so you have to perform the steps from above before these:

1. Clone the [rust-lang/rust](https://github.com/rust-lang/rust) repository.
2. Check out commit "ffa2e7ae8" (version nightly-2020-10-25).
3. Apply the patches by running `git am <repo-path>/rust-patches/*`, where `<repo-path>` is the working copy of this very repository.
4. Build by running `./x.py build -i library/std`.
5. The common way to use that build is through [rustup](https://rustup.rs). To do that, add a custom toolchain:
   ```
   rustup toolchain link custom-llvm-nightly-2020-10-25 build/x86_64-unknown-linux-gnu/stage1
   ```
   To use it by default, run `rustup default custom-llvm-nightly-2020-10-25`.
6. In order to use the custom Rust build, the custom LLVM build from above needs to be included in the library search path.
   The easiest way probably is to set `LD_LIBRARY_PATH`:
   ```
   LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/usr/local/lib/llvm-11-rs-debug/lib"
   ```
   (Adjust the path if `CMAKE_INSTALL_PREFIX` has been changed above.)

### Teaclave SGX SDK
A working copy of the [apache/incubator-teaclave-sgx-sdk](https://github.com/apache/incubator-teaclave-sgx-sdk) repository must be available somewhere in the file system.
Clone it and check out the "v1.1.3" tag.

### Path Adjustments
You might have to adjust some (hardcoded) paths to the ones of your local installations. These can be found by `grep`-ing for "ADJUST" in this repository.

Copyright
---------
The "llvm-patches" directory contains patch files to [LLVM project](https://github.com/llvm/llvm-project) code. LLVM consists of contributions from various authors, licensed under the Apache (Version 2.0) license with LLVM Exceptions and/or the Legacy LLVM license.

The "rust-patches" directory contains patch files to [Rust](https://github.com/rust-lang/rust) code. Rust consists of contributions from various authors, dual-licensed under the Apache (Version 2.0) and MIT licenses.

Some code in the "llvm-pass" directory is based on or inspired by the [llvm-tutor](https://github.com/banach-space/llvm-tutor) project. llvm-tutor was created by Andrzej Warzyński and other contributors and is licensed under the MIT license.

A header file and library compiled from [rustc-demangle](https://github.com/alexcrichton/rustc-demangle) are contained in the "llvm-pass/vendor" directory, as well as the patch file "llvm-patches/0001-Add-Enclavization-Pass.patch". rustc-demangle was created by Alex Crichton and other contributors and is dual-licensed under the Apache (Version 2.0) and MIT licenses.

Some code in the "rust-enclave" directory is based on or inspired by the [Apache Teaclave Rust-SGX SDK](https://github.com/apache/incubator-teaclave-sgx-sdk) project. The project's code has been licensed to the Apache Software Foundation under contributor license agreements and is available under the Apache (Version 2.0) license.

The file "support/cadote_trusted_runtime/src/io_error.rs" is partly based on a file from the Apache Teaclave Rust-SGX SDK, which in turn appears to have adopted it from the Rust project. Copyright details for both projects see above.
