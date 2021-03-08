LLVM_DIR ?= /usr/local/lib/llvm-11-debug
RUST_DIR ?= $(HOME)/.rustup/toolchains/nightly-2020-10-25-x86_64-unknown-linux-gnu

CLANG := $(LLVM_DIR)/bin/clang
RUSTC := $(RUST_DIR)/bin/rustc
OPT := $(LLVM_DIR)/bin/opt -debug
LLC := $(LLVM_DIR)/bin/llc


.PHONY: all clean clean_rs wrapsteel_indir wrapsteel_obj

all: wraps wrapsteel

clean: clean_rs
	rm -f *.bc *.o
	rm -f wraps

clean_rs:
	rm -rf wrapsteel-tmp
	rm -f wrapsteel


%.o: %.bc
	$(LLC) --filetype obj -o $@ $<

%_indir.bc: %.bc
	$(OPT) --load-pass-plugin ../../build/lib/libIndirectionPass.so --passes indirection-pass -o $@ $<


wraps.bc: wraps.c
	$(CLANG) -c -O0 -emit-llvm -o $@ $<

wraps: wraps_indir.o
	$(CLANG) -o $@ $<


# Rust build based on https://medium.com/@squanderingtime/manually-linking-rust-binaries-to-support-out-of-tree-llvm-passes-8776b1d037a4

wrapsteel-tmp: wrapsteel.rs
	rm -rf wrapsteel-tmp
	$(RUSTC) -C save-temps --emit llvm-ir --out-dir $@ $<
	rm -f $@/*.no-opt.bc

wrapsteel_indir: $(patsubst %.bc,%_indir.bc,$(wildcard wrapsteel-tmp/*.bc))

wrapsteel_obj: $(patsubst %.bc,%.o,$(wildcard wrapsteel-tmp/*_indir.bc))

wrapsteel: clean_rs wrapsteel-tmp
	# Call helper targets in a new Make instances, forcing prerequisites to be re-evaluated
	make wrapsteel_indir
	make wrapsteel_obj
	make wrapsteel_bin

wrapsteel_bin: wrapsteel-tmp/*_indir.o
	# TODO: Adding all Rust std rlibs in a linking group works, but is not optimal performance-wise
	$(CLANG) -lresolv -lc -lm -ldl -lpthread -o wrapsteel $^ -Wl,--start-group $(shell find $(RUST_DIR)/lib/rustlib -name '*.rlib') -Wl,--end-group
