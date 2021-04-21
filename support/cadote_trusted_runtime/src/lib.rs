#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

use sgx_trts::trts;


#[no_mangle]
pub fn cadote_check_arg_ptr(ptr: *const u8, size: usize) {
  //println!("Cadote runtime: Checking arg pointer {:?}, size {}", ptr, size);
  if trts::rsgx_raw_is_within_enclave(ptr, size) {
    panic!("Passing a pointer to enclave memory from outside, this is very nasty!");
  }
}

#[no_mangle]
pub fn cadote_check_return_ptr(ptr: *const u8, size: usize) {
  //println!("Cadote runtime: Checking return pointer {:?}, size {}", ptr, size);
  if trts::rsgx_raw_is_within_enclave(ptr, size) {
    panic!("Passing an allocation from within the enclave to the outside world");
  }
}
