#![no_std]
#![feature(test)]

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

use std::convert::TryFrom;
use std::env;
use std::hint::black_box;
use std::time::Instant;


static COUNT: i128 = 10000;


#[cfg(feature = "enclavization_bin")]
fn main() {
  let args: Vec<String> = env::args().collect();
  let num: i64 = args[1].parse().unwrap();

  let mut result_sum: i128 = 0;
  let beginstant = Instant::now();
  for _ in 0..COUNT {
    result_sum += factorial(num) as i128;
  }
  eprintln!("EVALUATION DURATION: {}", beginstant.elapsed().as_nanos() / u128::try_from(COUNT).unwrap());

  println!("Result: {}", result_sum / COUNT);
}

#[inline(never)]
fn factorial(mut num: i64) -> i64 {
  let n = black_box(&mut num);
  if *n <= 1 {
    return 1;
  }
  num * factorial_enclaved_(num - 1)
}

#[inline(never)]
fn factorial_enclaved_(num: i64) -> i64 {
  if num <= 1 {
    return 1;
  }
  num * factorial(num - 1)
}

