MAIN_CRATE := wrapsteel

LLVM_DIR ?= /usr/local/lib/llvm-11-debug
RUST_DIR ?= $(HOME)/.rustup/toolchains/nightly-2020-10-25-x86_64-unknown-linux-gnu
PASS_DIR ?= ../../../build/lib
TOOLS_DIR ?= ../../../tools

CARGO := cargo +nightly-2020-10-25-x86_64-unknown-linux-gnu
CLANG := $(LLVM_DIR)/bin/clang
OPT := $(LLVM_DIR)/bin/opt -debug
LLC := $(LLVM_DIR)/bin/llc

# Based on https://medium.com/@squanderingtime/manually-linking-rust-binaries-to-support-out-of-tree-llvm-passes-8776b1d037a4


.PHONY: debug clean

debug: build/$(MAIN_CRATE)

clean:
	rm -rf build


%.o: %.bc
	$(LLC) --filetype obj -o $@ $<

%_indir.bc: %.bc
	$(OPT) --load-pass-plugin ../../build/lib/libEnclavizationPass.so --passes enclavization-pass -o $@ $<


build/target: $(shell find src -type f -name '*.rs')
	rm -rf $@
	CARGO_TARGET_DIR=build/target $(CARGO) rustc -- -C save-temps --emit llvm-ir

build/$(MAIN_CRATE).bc: build/target
	cp $(shell LLVM_DIR=$(LLVM_DIR) $(TOOLS_DIR)/get-bc.sh --main build/target/debug/deps/*.bc) $@

build/_alloc-helper.bc: build/target
	cp $(shell LLVM_DIR=$(LLVM_DIR) $(TOOLS_DIR)/get-bc.sh --alloc build/target/debug/deps/*.bc) $@

build/$(MAIN_CRATE): build/$(MAIN_CRATE)_indir.o build/_alloc-helper_indir.o
	# TODO: Adding all Rust std rlibs in a linking group works, but is not optimal performance-wise
	$(CLANG) -lresolv -lc -lm -ldl -lpthread -o $@ $^ -Wl,--start-group build/target/debug/deps/*.rlib -Wl,--end-group -Wl,--start-group $(shell find $(RUST_DIR)/lib/rustlib -type f -name '*.rlib') -Wl,--end-group
