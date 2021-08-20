#![no_std]

#[cfg(feature = "enclavization_lib")]
#[macro_use]
extern crate sgx_tstd as std;

// For now, `macro_use` is required for std, see https://github.com/rust-lang/rust/issues/53977
#[cfg(feature = "enclavization_bin")]
#[macro_use]
extern crate std;

use std::prelude::v1::*;

#[cfg(feature = "enclavization_lib")]
extern crate cadote_trusted_runtime;

#[cfg(feature = "enclavization_bin")]
extern crate cadote_untrusted_runtime;

use std::env;
use std::time::Instant;


#[cfg(feature = "enclavization_bin")]
fn main() {
  let args: Vec<String> = env::args().collect();
  let num: i64 = args[1].parse().unwrap();

  let beginstant = Instant::now();
  let result = factorial(num);
  eprintln!("EVALUATION DURATION: {}", beginstant.elapsed().as_micros());

  println!("Result: {}", result);
}

fn factorial(num: i64) -> i64 {
  if num <= 1 {
    return 1;
  }
  num * factorial_enclaved_(num - 1)
}

fn factorial_enclaved_(num: i64) -> i64 {
  if num <= 1 {
    return 1;
  }
  num * factorial(num - 1)
}

