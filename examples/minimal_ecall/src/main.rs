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

use std::time::Instant;


#[cfg(feature = "enclavization_bin")]
fn main() {
  let beginstant = Instant::now();
  dummy_enclaved_();
  eprintln!("EVALUATION DURATION: {}", beginstant.elapsed().as_micros());
}

// Prohibit from being optimized away, see https://stackoverflow.com/a/42891114
#[no_mangle]
fn dummy_enclaved_() {
  // Enjoy the silence
}
