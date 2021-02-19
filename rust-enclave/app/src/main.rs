extern crate sgx_types;
extern crate sgx_urts;

use std::env;
use std::io;
use std::path;

use sgx_types::*;
use sgx_urts::SgxEnclave;


static ENCLAVE_FILE_NAME: &'static str = "enclave.signed.so";


extern {
    fn say_something(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
                     some_string: *const u8, len: usize) -> sgx_status_t;
    fn seal_it(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
               sealed_out: *mut u8, sealed_out_size: u32) -> sgx_status_t;
    fn unseal_it(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
                 sealed_data: *mut u8, sealed_data_size: u32) -> sgx_status_t;
}


fn get_enclave_file_path() -> io::Result<path::PathBuf> {
    let bin_path = env::current_exe()?;
    let bin_dir = bin_path.parent().unwrap();
    let enclave_path = bin_dir.join(ENCLAVE_FILE_NAME);
    Ok(enclave_path)
}


fn init_enclave<P: AsRef<path::Path>>(enclave_path: P) -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // Call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    SgxEnclave::create(enclave_path,
                       debug,
                       &mut launch_token,
                       &mut launch_token_updated,
                       &mut misc_attr)
}


fn main() {
    let enclave_path = get_enclave_file_path().expect("Could not determine enclave file path");

    let enclave = match init_enclave(enclave_path) {
        Ok(r) => {
            println!("[+] Init Enclave successful: {}", r.geteid());
            r
        },
        Err(x) => {
            println!("[-] Init Enclave failed: {}", x.as_str());
            return;
        },
    };

    let input_string = String::from("This is a normal world string passed into Enclave!\n");
    let mut retval = sgx_status_t::SGX_SUCCESS;

    unsafe {
        match say_something(
            enclave.geteid(),
            &mut retval,
            input_string.as_ptr() as * const u8,
            input_string.len()
        ) {
            sgx_status_t::SGX_SUCCESS => {},
            x => {
                println!("[-] ECALL failed: {}", x.as_str());
                return;
            }
        }
    }

    let mut sealed_data: [u8; 1024] = [0; 1024];

    unsafe {
        match seal_it(
            enclave.geteid(),
            &mut retval,
            &mut sealed_data as *mut _ as *mut u8,
            sealed_data.len() as u32
        ) {
            sgx_status_t::SGX_SUCCESS => (),
            x => {
                println!("[-] ECALL failed: {}", x.as_str());
                return;
            }
        }
    }

    unsafe {
        match unseal_it(
            enclave.geteid(),
            &mut retval,
            &mut sealed_data as *mut _ as *mut u8,
            sealed_data.len() as u32
        ) {
            sgx_status_t::SGX_SUCCESS => (),
            x => {
                println!("[-] ECALL failed: {}", x.as_str());
                return;
            }
        }
    }

    enclave.destroy();
}
