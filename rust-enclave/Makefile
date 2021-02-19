#### Rust Settings ####
# Toolchain currently used by Teaclave Rust-SGX SDK 1.1.3
RUSTUP_TOOLCHAIN := nightly-2020-10-25-x86_64-unknown-linux-gnu
CARGO := cargo +$(RUSTUP_TOOLCHAIN)
CARGO_BUILD_FLAGS ?= --release

TEACLAVE_SDK ?= $(HOME)/ext/incubator-teaclave-sgx-sdk

app_project_name := enclave_app
enclave_project_name := simpleenclave


#### SGX SDK Settings ####
SGX_SDK ?= /opt/sgxsdk
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


#### Target Paths ####
bin_dir := ./bin
lib_dir := ./lib

enclave_edl_files := enclave/enclave_t.c enclave/enclave_t.h app/enclave_u.c app/enclave_u.h
app_enclave_lib_file := $(lib_dir)/libEnclave_u.a
app_bin_file := $(bin_dir)/app

enclave_lib_file := lib/libenclave.a
enclave_bin_file := enclave/enclave.so
signed_enclave_bin_file := $(bin_dir)/enclave.signed.so


.PHONY: all clean
all: $(app_bin_file) $(signed_enclave_bin_file)


#### EDL Build ####
$(enclave_edl_files): enclave/enclave.edl
	$(SGX_EDGER8R) --trusted enclave/enclave.edl --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --trusted-dir enclave
	$(SGX_EDGER8R) --untrusted enclave/enclave.edl --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --untrusted-dir app


#### App Build ####
app/enclave_u.o: $(enclave_edl_files)
	@$(CC) $(SGX_COMMON_CFLAGS) -fPIC -Wno-attributes -I./app -I$(SGX_SDK)/include -I$(CUSTOM_EDL_PATH) -c app/enclave_u.c -o $@
	@echo "CC  <=  $<"

$(app_enclave_lib_file): app/enclave_u.o
	mkdir -p $(lib_dir)
	$(AR) rcsD $@ $^

$(app_bin_file): $(app_enclave_lib_file) $(shell find app/ -type f -name '*.rs') app/Cargo.toml
	@cd app && SGX_SDK=$(SGX_SDK) $(CARGO) build $(CARGO_BUILD_FLAGS)
	@echo "Cargo  =>  $@"
	mkdir -p $(bin_dir)
	cp -f app/target/release/$(app_project_name) $(app_bin_file)


#### Enclave Build ####
enclave/enclave_t.o: $(enclave_edl_files)
	@$(CC) $(SGX_COMMON_CFLAGS) $(ENCLAVE_CFLAGS) -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_EDL_PATH) -I$(SGX_SDK)/include \
		-I$(SGX_SDK)/include/tlibc -I$(SGX_SDK)/include/stlport -I$(SGX_SDK)/include/epid -I./enclave -I./include \
		-c enclave/enclave_t.c -o $@
	@echo "CC  <=  $<"

$(enclave_lib_file): $(shell find enclave/ -type f -name '*.rs') app/Cargo.toml
	@cd enclave && $(CARGO) build $(CARGO_BUILD_FLAGS)
	@echo "Cargo  =>  $@"
	cp enclave/target/release/lib$(enclave_project_name).a $(enclave_lib_file)

$(enclave_bin_file): $(enclave_lib_file) enclave/enclave_t.o
	@$(CXX) enclave/enclave_t.o -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles -L$(SGX_LIBRARY_PATH) \
		-Wl,--whole-archive -l$(trts_library_name) -Wl,--no-whole-archive \
		-Wl,--start-group -lsgx_tstdc -l$(service_library_name) -l$(crypto_library_name) -L $(lib_dir) -lenclave -Wl,--end-group \
		-Wl,--version-script=enclave/enclave.lds $(ENCLAVE_LDFLAGS) -o $@
	@echo "LINK  =>  $@"

$(signed_enclave_bin_file): $(enclave_bin_file) $(SGX_SIGNING_KEY)
	mkdir -p $(bin_dir)
	@$(SGX_ENCLAVE_SIGNER) sign -key $(SGX_SIGNING_KEY) -enclave $(enclave_bin_file) -config enclave/enclave-config.xml -out $@
	@echo "SIGN  =>  $@"


#### Cleaning ####
clean:
	rm -f $(app_bin_file) $(enclave_bin_file) $(signed_enclave_bin_file) enclave/*_t.* app/*_u.* $(lib_dir)/*.a
	cd app && cargo clean
	cd enclave && cargo clean
