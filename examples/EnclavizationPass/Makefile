MAIN_CRATE := wrapsteel

CARGO := cargo +custom-llvm-nightly-2020-10-25


.PHONY: debug clean

debug: build/$(MAIN_CRATE) build/enclavization.edl

clean:
	rm -rf build


build/target: $(shell find src -type f -name '*.rs')
	CARGO_TARGET_DIR=build/target $(CARGO) rustc -- -C passes=enclavization-pass

build/$(MAIN_CRATE): build/target
	cp build/target/debug/$(MAIN_CRATE) $@

build/enclavization.edl: build/target
	mv -f enclavization.edl $@
