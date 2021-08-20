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

use std::convert::TryFrom;
use std::env;
use std::time::Instant;


static COUNT: i64 = 10000;


#[cfg(feature = "enclavization_bin")]
fn main() {
  let args: Vec<String> = env::args().collect();
  let num1: i64 = args[1].parse().unwrap();
  let num2: i64 = args[2].parse().unwrap();

  let mut result_sum: i64 = 0;
  let beginstant = Instant::now();
  for _ in 0..COUNT {
    result_sum += gcd(num1, num2);
  }
  eprintln!("EVALUATION DURATION: {}", beginstant.elapsed().as_nanos() / u128::try_from(COUNT).unwrap());

  println!("Result: {}", result_sum / COUNT);
}

#[inline(never)]
fn gcd(a: i64, b:i64) -> i64 {
  if b == 0 {
    return a;
  }
  let remainder = a % b;
  gcd_enclaved_(b, remainder)
}

#[inline(never)]
fn gcd_enclaved_(a: i64, b:i64) -> i64 {
  if b == 0 {
    return a;
  }
  let remainder = a % b;
  gcd(b, remainder)
}

