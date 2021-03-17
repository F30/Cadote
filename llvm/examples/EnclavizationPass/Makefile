TARGET_CRATE := wrapsteel

LLVM_DIR ?= /usr/local/lib/llvm-11-debug
RUST_DIR ?= $(HOME)/.rustup/toolchains/nightly-2020-10-25-x86_64-unknown-linux-gnu

CLANG := $(LLVM_DIR)/bin/clang
RUSTC := $(RUST_DIR)/bin/rustc
OPT := $(LLVM_DIR)/bin/opt -debug
LLC := $(LLVM_DIR)/bin/llc

# Based on https://medium.com/@squanderingtime/manually-linking-rust-binaries-to-support-out-of-tree-llvm-passes-8776b1d037a4


.PHONY: all clean

all: build/$(TARGET_CRATE)

clean:
	rm -rf build


%.o: %.bc
	$(LLC) --filetype obj -o $@ $<

%_indir.bc: %.bc
	$(OPT) --load-pass-plugin ../../build/lib/libEnclavizationPass.so --passes enclavization-pass -o $@ $<


build/rust-tmp: $(TARGET_CRATE).rs
	rm -rf $@
	$(RUSTC) -C save-temps --emit llvm-ir --out-dir $@ $<

build/$(TARGET_CRATE).bc: build/rust-tmp
	find build/rust-tmp -name '$(TARGET_CRATE).$(TARGET_CRATE).*.bc' ! -name '*.no-opt.bc' -exec cp {} $@ \;

build/rust-helper.bc: build/rust-tmp
	find build/rust-tmp -name '$(TARGET_CRATE).*.bc' ! -name '$(TARGET_CRATE).$(TARGET_CRATE).*.bc' -exec cp {} $@ \;

build/$(TARGET_CRATE): build/$(TARGET_CRATE)_indir.o build/rust-helper_indir.o
	# TODO: Adding all Rust std rlibs in a linking group works, but is not optimal performance-wise
	$(CLANG) -lresolv -lc -lm -ldl -lpthread -o $@ $^ -Wl,--start-group $(shell find $(RUST_DIR)/lib/rustlib -name '*.rlib') -Wl,--end-group
