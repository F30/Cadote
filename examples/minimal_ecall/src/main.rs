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

use std::hint::black_box;
use std::time::Instant;


static COUNT: u128 = 10000;


#[cfg(feature = "enclavization_bin")]
fn main() {
  let mut x = 0;
  let beginstant = Instant::now();
  for _ in 0..COUNT {
    dummy_enclaved_(&mut x);
  }
  eprintln!("EVALUATION DURATION: {}", beginstant.elapsed().as_nanos() / COUNT);
}

#[inline(never)]
fn dummy_enclaved_(x: &mut i64) {
  // Prohibit the function from being optimized away
  black_box(x);
}
