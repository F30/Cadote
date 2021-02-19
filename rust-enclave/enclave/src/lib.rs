#![crate_name = "simpleenclave"]
#![crate_type = "staticlib"]
#![no_std]

extern crate sgx_types;
extern crate sgx_tseal;
#[macro_use]
extern crate sgx_tstd as std;

use std::io::{self, Write};
use std::slice;

use sgx_types::*;
use sgx_tseal::SgxSealedData;


#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {
    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    println!("This is a in-Enclave Rust string");

    sgx_status_t::SGX_SUCCESS
}


#[no_mangle]
pub extern "C" fn seal_it(sealed_out: *mut u8, sealed_out_size: u32) -> sgx_status_t {
    let data_clear = "topsecret".as_bytes();
    let add_data: [u8; 0] = [];

    // Copied from what simple seal_data() uses
    let attribute_mask = sgx_attributes_t {
        flags: TSEAL_DEFAULT_FLAGSMASK,
        xfrm: 0,
    };
    let sealed_data_1 = match SgxSealedData::<[u8]>::seal_data_ex(
        SGX_KEYPOLICY_MRSIGNER,
        attribute_mask,
        TSEAL_DEFAULT_MISCMASK,
        &add_data,
        &data_clear
    ) {
        Ok(x) => x,
        Err(status) => return status
    };

    unsafe {
        match sealed_data_1.to_raw_sealed_data_t(
            sealed_out as *mut sgx_sealed_data_t,
            sealed_out_size
        ) {
            Some(_) => (),
            None => return sgx_status_t::SGX_ERROR_UNEXPECTED
        }
    }

    sgx_status_t::SGX_SUCCESS
}


#[no_mangle]
pub extern "C" fn unseal_it(sealed_data: *mut u8, sealed_data_size: u32) -> sgx_status_t {
    let sealed_data_struct = unsafe {
        match SgxSealedData::<[u8]>::from_raw_sealed_data_t(
            sealed_data as *mut sgx_sealed_data_t,
            sealed_data_size
        ) {
            Some(x) => x,
            None => return sgx_status_t::SGX_ERROR_UNEXPECTED
        }
    };

    let unsealed_data = match sealed_data_struct.unseal_data() {
        Ok(x) => x,
        Err(status) => return status
    };

    println!("Sealed and back: {}", std::str::from_utf8(unsealed_data.get_decrypt_txt()).unwrap());

    sgx_status_t::SGX_SUCCESS
}
