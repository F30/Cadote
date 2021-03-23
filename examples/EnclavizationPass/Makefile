MAIN_CRATE := wrapsteel

CARGO := cargo +custom-llvm-nightly-2020-10-25


.PHONY: debug clean

debug: build/$(MAIN_CRATE) build/enclavization.edl

clean:
	rm -rf build


build/target: $(shell find src -type f -name '*.rs')
	CARGO_TARGET_DIR=build/target $(CARGO) rustc -- -C passes=enclavization-pass
	#CARGO_TARGET_DIR=build/target $(CARGO) rustc -- -C passes=enclavization-pass -C llvm-args=-debug -C codegen-units=1 2>build/output

build/$(MAIN_CRATE): build/target
	cp build/target/debug/$(MAIN_CRATE) $@

build/enclavization.edl: build/target
	mv -f enclavization.edl $@
