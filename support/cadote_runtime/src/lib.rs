extern crate sgx_urts;
extern crate sgx_types;

use sgx_types::*;


static ENCLAVE_FILE_NAME: &'static str = "enclave.signed.so";
static mut ENCLAVE: Option<sgx_urts::SgxEnclave> = None;


fn get_enclave_file_path() -> std::io::Result<std::path::PathBuf> {
  let bin_path = std::env::current_exe()?;
  let bin_dir = bin_path.parent().unwrap();
  let enclave_path = bin_dir.join(ENCLAVE_FILE_NAME);

  Ok(enclave_path)
}

#[no_mangle]
pub fn cadote_init_enclave() {
  let path = get_enclave_file_path().expect("Could not determine enclave file path");

  let mut launch_token: sgx_launch_token_t = [0; 1024];
  let mut launch_token_updated: i32 = 0;
  // Debug support: Set to 1
  let debug = 0;
  let mut misc_attr = sgx_misc_attribute_t{secs_attr: sgx_attributes_t{flags: 0, xfrm: 0}, misc_select: 0};

  match sgx_urts::SgxEnclave::create(path, debug, &mut launch_token, &mut launch_token_updated, &mut misc_attr) {
    Ok(r) => {
      unsafe {
        ENCLAVE = Some(r);
      }
    },
    Err(e) => {
      panic!("Could not initialize enclave: {}", e);
    }
  };
}

#[no_mangle]
pub unsafe fn cadote_get_enclave_id() -> sgx_enclave_id_t {
  match &ENCLAVE {
    Some(e) => e.geteid(),
    None => {
      panic!("Enclave needs to be initialized before usage");
    }
  }
}

#[no_mangle]
pub fn cadote_enclave_error_handler() -> ! {
  panic!("Unrecoverable enclave error");
}
