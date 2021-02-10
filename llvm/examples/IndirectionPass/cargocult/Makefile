LLVM_DIR ?= /usr/local/lib/llvm-11-debug
RUST_DIR ?= $(HOME)/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu
PASS_DIR ?= ../../../build/lib

# Direkt calling `$(RUST_DIR)/bin/cargo` will use rustc from the wrong (non-nightly) toolchain
CARGO := cargo +nightly
CLANG := $(LLVM_DIR)/bin/clang
OPT := $(LLVM_DIR)/bin/opt -debug
LLC := $(LLVM_DIR)/bin/llc

# Based on https://medium.com/@squanderingtime/manually-linking-rust-binaries-to-support-out-of-tree-llvm-passes-8776b1d037a4


.PHONY: debug clean target_indir target_obj cargocult_bin

debug: target/debug/deps/cargocult

clean:
	rm -rf target


%.o: %.bc
	$(LLC) --filetype obj -o $@ $<

%_indir.bc: %.bc
	$(OPT) --load-pass-plugin $(PASS_DIR)/libIndirectionPass.so --passes indirection-pass -o $@ $<


target/debug/deps: clean
	$(CARGO) rustc -- -C save-temps --emit llvm-ir
	rm -f $@/*.no-opt.bc
	rm -f $@/*.no-opt.o
	find target/debug/deps/ -name 'cargocult-*' -executable -delete

target_indir: $(patsubst %.bc,%_indir.bc,$(wildcard target/debug/deps/*.bc))

target_obj: $(patsubst %.bc,%.o,$(wildcard target/debug/deps/*_indir.bc))

target/debug/deps/cargocult: target/debug/deps
	# Call helper targets in a new Make instances, forcing prerequisites to be re-evaluated
	make target_indir
	make target_obj
	make cargocult_bin

cargocult_bin: target/debug/deps/*_indir.bc
	# TODO: Adding all Rust std rlibs in a linking group works, but is not optimal performance-wise
	$(CLANG) -lresolv -lc -lm -ldl -lpthread -o target/debug/deps/cargocult $^ -Wl,--start-group target/debug/deps/*.rlib -Wl,--end-group -Wl,--start-group $(shell find $(RUST_DIR)/lib/rustlib -name '*.rlib') -Wl,--end-group
