#![no_std]

#[cfg(feature = "enclavization_lib")]
#[macro_use]
#[cfg(feature = "enclavization_lib")]
extern crate sgx_tstd as std;

#[cfg(feature = "enclavization_lib")]
extern crate cadote_trusted_runtime;

// For now, `macro_use` is required for std, see https://github.com/rust-lang/rust/issues/53977
#[cfg(feature = "enclavization_bin")]
#[macro_use]
extern crate std;

#[cfg(feature = "enclavization_bin")]
extern crate cadote_untrusted_runtime;

use std::prelude::v1::*;


#[derive(Debug)]
struct User {
  sign_in_count: u64,
  active: bool
}

#[no_mangle]
pub fn foo_enclaved_(param1: &i64, param2: &str) -> i64 {
  println!("The passed_parameters are: {} {}", param1, param2);
  99
}

fn get_array_enclaved_() -> [i64; 20] {
  [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
}

fn get_short_tuple_enclaved_() -> (i64, i64) {
  (23, 42)
}

fn get_long_tuple_enclaved_() -> (f64, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) {
  (1337.0, true, false, true, false, true, false, true, false, true, false, true)
}

fn get_struct_enclaved_() -> User {
  User {
    sign_in_count: 2,
    active: true
  }
}

fn pass_slice_enclaved_(param: &str) -> &str {
  &param[6..]
}

fn get_long_int_enclaved_() -> i128 {
  i128::MAX - 2000
}

fn get_result_enclaved_(i: i64) -> Result<(), &'static str> {
  if i >= 0 {
    Ok(())
  } else {
    Err("Error message")
  }
}

fn take_string_enclaved_(s: String) {
  println!("Got string: {}", s);
}

#[allow(dead_code)]
fn get_box_enclaved_() -> Box<i64> {
  Box::new(5)
}

#[allow(dead_code)]
fn get_box_array_enclaved_() -> [Box<i64>; 3] {
  [Box::new(1), Box::new(2), Box::new(3)]
}

#[allow(dead_code)]
fn get_string_enclaved_() -> String {
  String::from("admin")
}

#[allow(dead_code)]
unsafe fn get_enclave_addr_enclaved_() -> usize {
  let val: Box<i64> = Box::new(1337);
  let raw_ptr: *const i64 = &*val;
  let addr = raw_ptr as usize;
  // This will (rightfully) not be detected as problematic, because it doesn't have a pointer type
  addr
}

#[allow(dead_code)]
unsafe fn pass_enclave_addr() {
  let addr = get_enclave_addr_enclaved_();
  println!("Got enclave address 0x{:x}, now passing it back as reference (that should fail)", addr);
  let raw_ptr = addr as *const i64;
  let val = raw_ptr.as_ref().unwrap();
  // Trigger the "Passing a pointer to enclave memory from outside" case
  pass_ref_enclaved_(val);
}

#[allow(dead_code)]
fn pass_ref_enclaved_(param: &i64) {
  println!("This should not work: {}", param);
}

fn main() {
  let x = 42;
  let y = "abcdefghi";
  let a = foo_enclaved_(&x, &y[3..]);
  println!("foo() return value: {}", a);

  let b = get_array_enclaved_();
  println!("Array: {:?}", b);
  let c = get_short_tuple_enclaved_();
  println!("Short tuple: {:?}", c);
  let d = get_long_tuple_enclaved_();
  println!("Long tuple: {:?}", d);
  let e = get_struct_enclaved_();
  println!("Struct: {:?}", e);
  let f = pass_slice_enclaved_("Hello world");
  println!("Passed slice: {:?}", f);
  let g = get_long_int_enclaved_();
  println!("Long int: {}", g);
  get_result_enclaved_(1).expect("Did not get OK result");
  println!("Got OK result");
  take_string_enclaved_(String::from("String into enclave"));

  // Uncomment to trigger check cases
  //let z = get_box_enclaved_();
  //let z = get_box_array_enclaved_();
  //let z = get_string_enclaved_();
  //println!("Value: {:?}", z);
  //unsafe {
  //  pass_enclave_addr();
  //}
}
