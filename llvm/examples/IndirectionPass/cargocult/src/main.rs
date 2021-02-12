use log::warn;

#[no_mangle]
fn foo_wrapped_() {
  println!("Hello Rust!");
}

#[no_mangle]
fn wrapper(func: fn() -> ()) {
  func();
  warn!("... from wrapper()");
}

fn main() {
  env_logger::init();
  foo_wrapped_();
}
