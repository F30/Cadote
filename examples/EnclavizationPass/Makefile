#### Target Paths ####
enclave_build_dir := build/enclave
app_build_dir := build/app

enclave_edl_file := $(enclave_build_dir)/enclave.edl
enclave_stub_trusted_dir := $(enclave_build_dir)
enclave_stub_untrusted_dir := $(app_build_dir)
enclave_stub_files := $(enclave_stub_trusted_dir)/enclave_t.c $(enclave_stub_trusted_dir)/enclave_t.h $(enclave_stub_untrusted_dir)/enclave_u.c $(enclave_stub_untrusted_dir)/enclave_u.h

enclave_lib_file := $(enclave_build_dir)/libenclave.a
enclave_bin_file := build/enclave.so
signed_enclave_bin_file := build/enclave.signed.so

app_stub_lib_file := $(app_build_dir)/libEnclave_u.a
app_bin_file = build/$(project_name)


#### Rust Settings ####
project_name := wrapsteel
build_mode := debug
#build_mode := release

ifeq ($(build_mode), release)
CARGO_BUILD_FLAGS := $(CARGO_BUILD_FLAGS) --release
endif

# Toolchain based on the one used by Teaclave Rust-SGX SDK 1.1.3
RUSTUP_TOOLCHAIN := custom-llvm-nightly-2020-10-25
CARGO := LD_LIBRARY_PATH=/usr/local/lib/llvm-11-rs-debug/lib cargo +$(RUSTUP_TOOLCHAIN)
#CARGO := LD_LIBRARY_PATH=/usr/local/lib/llvm-11-rs-debug/lib valgrind --tool=none --trace-children=yes cargo +$(RUSTUP_TOOLCHAIN)

TEACLAVE_SDK ?= $(HOME)/ext/incubator-teaclave-sgx-sdk


#### SGX SDK Settings ####
SGX_SDK ?= $(HOME)/.sgxsdk/sgxsdk
SGX_MODE ?= HW
SGX_SIGNING_KEY ?= $(HOME)/sgx-sign/private_key.pem

include $(TEACLAVE_SDK)/buildenv.mk

SGX_COMMON_CFLAGS := -m64
SGX_LIBRARY_PATH := $(SGX_SDK)/lib64
SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x64/sgx_sign
SGX_EDGER8R := $(SGX_SDK)/bin/x64/sgx_edger8r

ifeq ($(SGX_DEBUG), 1)
ifeq ($(SGX_PRERELEASE), 1)
$(error Cannot set SGX_DEBUG and SGX_PRERELEASE at the same time!)
endif
endif

ifeq ($(SGX_DEBUG), 1)
SGX_COMMON_CFLAGS += -O0 -g
else
SGX_COMMON_CFLAGS += -O2
endif

SGX_COMMON_CFLAGS += -fstack-protector


#### Enclave Settings ####
ifneq ($(SGX_MODE), HW)
trts_library_name := sgx_trts_sim
service_library_name := sgx_tservice_sim
else
trts_library_name := sgx_trts
service_library_name := sgx_tservice
endif

crypto_library_name := sgx_tcrypto
keyExchange_library_name := sgx_tkey_exchange
protectedFs_library_name := sgx_tprotected_fs


#### Search Paths ####
CUSTOM_EDL_PATH := $(TEACLAVE_SDK)/edl
CUSTOM_COMMON_PATH := $(TEACLAVE_SDK)/common


#### Support File Paths ####
SUPPORT_DIR ?= ../../support

CADOTE_RUNTIME_DIR := $(SUPPORT_DIR)/cadote_runtime
ENCLAVE_CONFIG_FILE := $(SUPPORT_DIR)/enclave-config.xml
LINKER_SCRIPT_FILE := $(SUPPORT_DIR)/enclave.lds


#### Generic Targets ####
.PHONY: all clean

all: $(signed_enclave_bin_file) $(app_bin_file)

clean:
	rm -rf build
	rm -f Cargo.toml


#### Stub File Build ####
$(enclave_stub_files): $(enclave_edl_file)
	$(SGX_EDGER8R) --trusted $< --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --trusted-dir $(enclave_stub_trusted_dir)
	$(SGX_EDGER8R) --untrusted $< --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --untrusted-dir $(enclave_stub_untrusted_dir)


#### Enclave Build ####
$(enclave_lib_file) $(enclave_edl_file): $(shell find src/ -type f -name '*.rs') Cargo.lib.toml
	ln -s -f Cargo.lib.toml Cargo.toml
	@CARGO_TARGET_DIR=build/cargo_lib_target $(CARGO) rustc $(CARGO_BUILD_FLAGS) --lib --features=enclavization_lib -- -C passes=enclavization-lib-pass -A dead_code -C link-dead-code -C codegen-units=1
	#CARGO_TARGET_DIR=build/cargo_lib_target $(CARGO) rustc $(CARGO_BUILD_FLAGS) --lib --features=enclavization_lib -- -C passes=enclavization-lib-pass -A dead_code -C link-dead-code -C llvm-args=-debug -C codegen-units=1 2>build/output
	rm Cargo.toml
	@echo "Cargo  =>  $(enclave_lib_file)"
	mkdir -p $(enclave_build_dir)
	cp -f build/cargo_lib_target/$(build_mode)/lib$(project_name).a $(enclave_lib_file)
	mv -f enclave.edl $(enclave_edl_file)

$(enclave_build_dir)/enclave_t.o: $(enclave_stub_files)
	@$(CC) $(SGX_COMMON_CFLAGS) $(ENCLAVE_CFLAGS) -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_EDL_PATH) -I$(SGX_SDK)/include \
		-I$(SGX_SDK)/include/tlibc -I$(SGX_SDK)/include/stlport -I$(SGX_SDK)/include/epid \
		-c $(enclave_stub_trusted_dir)/enclave_t.c -o $@
	@echo "CC  <=  $<"

$(enclave_bin_file): $(enclave_build_dir)/enclave_t.o $(enclave_lib_file) $(LINKER_SCRIPT_FILE)
	@$(CXX) $(enclave_build_dir)/enclave_t.o -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles -L$(SGX_LIBRARY_PATH) \
		-Wl,--whole-archive -l$(trts_library_name) -Wl,--no-whole-archive \
		-Wl,--start-group -lsgx_tstdc -l$(service_library_name) -l$(crypto_library_name) -L $(enclave_build_dir) -lenclave -Wl,--end-group \
		-Wl,--version-script=$(LINKER_SCRIPT_FILE) $(ENCLAVE_LDFLAGS) -o $@
	@echo "LINK  =>  $@"

$(signed_enclave_bin_file): $(enclave_bin_file) $(SGX_SIGNING_KEY) $(ENCLAVE_CONFIG_FILE)
	@$(SGX_ENCLAVE_SIGNER) sign -key $(SGX_SIGNING_KEY) -enclave $(enclave_bin_file) -config $(ENCLAVE_CONFIG_FILE) -out $@
	@echo "SIGN  =>  $@"


#### App Build ####
$(app_build_dir)/enclave_u.o: $(enclave_stub_files)
	@$(CC) $(SGX_COMMON_CFLAGS) -fPIC -Wno-attributes -I$(SGX_SDK)/include -I$(CUSTOM_EDL_PATH) -c $(enclave_stub_untrusted_dir)/enclave_u.c -o $@
	@echo "CC  <=  $<"

$(app_stub_lib_file): $(app_build_dir)/enclave_u.o
	$(AR) rcsD $@ $^

$(app_bin_file): $(app_stub_lib_file) $(shell find src/ -type f -name '*.rs') Cargo.bin.toml $(shell find $(CADOTE_RUNTIME_DIR)/src -type f -name '*.rs')
	ln -s -f Cargo.bin.toml Cargo.toml
	@SGX_SDK=$(SGX_SDK) CARGO_TARGET_DIR=build/cargo_bin_target $(CARGO) rustc $(CARGO_BUILD_FLAGS) --features=enclavization_bin -- -C passes=enclavization-bin-pass -C codegen-units=1
	#SGX_SDK=$(SGX_SDK) CARGO_TARGET_DIR=build/cargo_bin_target $(CARGO) rustc $(CARGO_BUILD_FLAGS) --features=enclavization_bin -- -C passes=enclavization-bin-pass -C llvm-args=-debug -C codegen-units=1 2>build/output
	rm Cargo.toml
	@echo "Cargo  =>  $@"
	cp -f build/cargo_bin_target/$(build_mode)/$(project_name) $@
