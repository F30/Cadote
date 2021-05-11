Cadote: Compiler-Aided Development of Trusted Enclaves
======================================================

This is the code for my Masters Thesis. It is quite hacky research-grade code, where I had to go to the limit of what is possible with an LLVM pass. Also, there is quite some C++ code and I'm not much of a C++ programmer. Sorry about that.

For details on the idea and its limitations, read the actual thesis.

Setup
-----
### Path Adjustments
You might have to adjust some (hardcoded) paths to the ones of your local installations. These can be found by `grep`-ing for "ADJUST".

Copyright
---------
The "llvm-patches" directory contains patch files to [LLVM project](https://github.com/llvm/llvm-project) code. LLVM consists of contributions from various authors, licensed under the Apache (Version 2.0) license with LLVM Exceptions and/or the Legacy LLVM license.

Some code in the "llvm-pass" directory is based on or inspired by the [llvm-tutor](https://github.com/banach-space/llvm-tutor) project. llvm-tutor was created by Andrzej Warzyński and other contributors and is licensed under the MIT license.

A header file and library compiled from [rustc-demangle](https://github.com/alexcrichton/rustc-demangle) are contained in the "llvm-pass/vendor" directory, as well as the patch file "llvm-patches/0001-Add-Enclavization-Pass.patch". rustc-demangle was created by Alex Crichton and other contributors and is dual-licensed under the Apache (Version 2.0) and MIT licenses.

Some code in the "rust-enclave" directory is based on or inspired by the [Apache Teaclave Rust-SGX SDK](https://github.com/apache/incubator-teaclave-sgx-sdk) project. The project's code has been licensed to the Apache Software Foundation under contributor license agreements and is available under the Apache (Version 2.0) license.
