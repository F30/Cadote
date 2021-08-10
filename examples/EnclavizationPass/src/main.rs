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
  println!("The passed parameters are: {} {}", param1, param2);
  99
}

fn get_short_tuple_enclaved_() -> (i64, i64) {
  (23, 42)
}

fn get_long_tuple_enclaved_() -> (f64, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) {
  (1337.0, true, false, true, false, true, false, true, false, true, false, true)
}

fn take_tuple_ref_enclaved_(param: &(i32, bool)) {
  println!("The passed-by-reference tuple is: {:?}", param);
}

fn get_struct_enclaved_() -> User {
  User {
    sign_in_count: 2,
    active: true
  }
}

fn get_array_enclaved_() -> [i64; 20] {
  [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
}

fn take_array_enclaved_(param: [i64; 5]) {
  println!("The passed array is: {:?}", param);
}

fn take_array_ref_enclaved_(param: &[i64; 5]) {
  println!("The passed-by-reference array is: {:?}", param);
}

fn modify_array_ref_enclaved_(param: &mut[i64; 3]) {
  param[1] = 23;
}

fn pass_slice_enclaved_(param: &str) -> &str {
  &param[6..]
}

fn modify_slice_array_enclaved_(param: [&mut [i64]; 2]) {
  param[1][0] = 1337;
}

fn pass_slice_slice_enclaved_(param: &mut [&[i64]]) {
  let subslice = &param[0][..2];
  param[1] = subslice;
}

fn get_long_int_enclaved_() -> i128 {
  i128::MAX - 2000
}

fn get_result_enclaved_(i: i64) -> Result<(), i64> {
  if i >= 0 {
    Ok(())
  } else {
    Err(-1)
  }
}

fn reverse_get_array() -> [i64; 20] {
  [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
}

fn reverse_get_struct() -> User {
  User {
    sign_in_count: 14,
    active: false
  }
}

fn reverse_take_tuple_ref(param: &(i64, i64, i64)) {
  println!("The passed-by-reference tuple is: {:?}", param);
}

fn reverse_modify_array_ref(param: &mut[i64; 3]) {
  param[2] = 33;
}

fn reverse_modify_slice_array_ref(param: &mut [&mut [i64]; 1]) {
  param[0][1] = -100;
}

fn reverse_pass_slice(param: &str) -> &str {
  &param[0..3]
}

fn do_reverse_calls_enclaved_(param: &mut [i64]) {
  let a = reverse_get_array();
  println!("Reverse array: {:?}", a);
  let b = reverse_get_struct();
  println!("Reverse struct: {:?}", b);
  let c = (23, 42, 100);
  reverse_take_tuple_ref(&c);
  let mut d = [1, 2, 3];
  reverse_modify_array_ref(&mut d);
  println!("Array after modification: {:?}", d);
  let mut e = [param];
  reverse_modify_slice_array_ref(&mut e);
  let f = reverse_pass_slice("ABCDEFGH");
  println!("Reverse-passed slice: {}", f);
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn reverse_pass_ref_enclaved_() {
  let i = 1337;
  let j = 1338;
  reverse_give_ref([&i, &j]);
}

#[allow(dead_code)]
fn reverse_give_ref(param: [&i64; 2]) {
  println!("This should not work: {:?}", param);
}

#[allow(dead_code)]
unsafe fn reverse_pass_enclave_addr_enclaved_() {
  let val = 200;
  let raw_ptr: *const i64 = &val;
  let addr = raw_ptr as usize;
  println!("This should not work: {}", reverse_get_enclave_ref(addr));
}

#[allow(dead_code)]
unsafe fn reverse_get_enclave_ref<'a>(addr: usize) -> &'a i64 {
  println!("Got enclave address 0x{:x}, now passing it back as reference (that should fail)", addr);
  let raw_ptr = addr as *const i64;
  // Trigger the "Passing a pointer to enclave memory from outside" case
  raw_ptr.as_ref().unwrap()
}


fn main() {
  let x = 42;
  let y = "abcdefghi";
  let a = foo_enclaved_(&x, &y[3..]);
  println!("foo() return value: {}", a);

  let b = get_short_tuple_enclaved_();
  println!("Short tuple: {:?}", b);
  let c = get_long_tuple_enclaved_();
  println!("Long tuple: {:?}", c);
  let d = (-5, false);
  take_tuple_ref_enclaved_(&d);
  let e = get_struct_enclaved_();
  println!("Struct: {:?}", e);

  let f = get_array_enclaved_();
  println!("Array: {:?}", f);
  let g = [100, 99, 98, 97, 96];
  take_array_enclaved_(g);
  take_array_ref_enclaved_(&g);
  let mut f = [10, 20, 30];
  modify_array_ref_enclaved_(&mut f);
  println!("Array after modification: {:?}", f);

  let g = pass_slice_enclaved_("Hello world");
  println!("Passed slice: {:?}", g);
  let mut h = [42, 42, 42];
  let mut i = [23, 23];
  modify_slice_array_enclaved_([&mut h[..], &mut i[..]]);
  println!("(Un)modified slices: {:?}, {:?}", h, i);
  let j = [13, 37, 0];
  let k = [47, 11];
  let mut l = [&j[..], &k[..]];
  pass_slice_slice_enclaved_(&mut l[..]);
  println!("Passed slice slice: {:?}", l);

  let m = get_long_int_enclaved_();
  println!("Long int: {}", m);
  get_result_enclaved_(1).expect("Did not get OK result");
  println!("Got OK result");

  let mut n = [0, 0, 0];
  do_reverse_calls_enclaved_(&mut n);
  println!("Modified slice after back and forth to and from enclave: {:?}", n);

  // This does not work because String is implemented as Vec<u8> and we cannot copy that
  //take_string_enclaved_(String::from("String into enclave"));

  // Uncomment to trigger check cases
  //let z = get_box_enclaved_();
  //let z = get_box_array_enclaved_();
  //let z = get_string_enclaved_();
  //println!("Value: {:?}", z);
  //unsafe {
  //  pass_enclave_addr();
  //}
  //reverse_pass_ref_enclaved_();
  //unsafe {
  //  reverse_pass_enclave_addr_enclaved_();
  //}
}
