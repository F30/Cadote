#![no_std]

#[cfg(feature = "enclavization_lib")]
#[macro_use]
extern crate sgx_tstd as std;

// For now, `macro_use` is required for std, see https://github.com/rust-lang/rust/issues/53977
#[cfg(feature = "enclavization_bin")]
#[macro_use]
extern crate std;
#[cfg(feature = "enclavization_bin")]
use std::prelude::v1::*;

#[cfg(feature = "enclavization_bin")]
extern crate cadote_untrusted_runtime;


#[derive(Debug)]
struct User {
  sign_in_count: u64,
  active: bool
}

#[no_mangle]
pub fn foo_enclaved_(param: &i64, lala: &str) -> i64 {
  println!("The number is: {} {}", param, lala);
  99
}

fn bar_enclaved_() -> (f64, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) {
  (1337.0, true, false, true, false, true, false, true, false, true, false, true)
}

fn more_enclaved_() -> User {
  User {
    sign_in_count: 2,
    active: true
  }
}

fn even_more_enclaved_() -> (i64, i64) {
  (23, 42)
}

fn main() {
  let val = 42;
  let lala = "abcdefghi";
  foo_enclaved_(&val, &lala[3..]);
  let x = foo_enclaved_(&23, &lala[4..]);
  println!("foo return value: {}", x);
  let y = bar_enclaved_();
  println!("bar return value: {:?}", y);
  let a = more_enclaved_();
  println!("more return value: {:?}", a);
  let b = even_more_enclaved_();
  println!("even more return value: {:?}", b)
}
