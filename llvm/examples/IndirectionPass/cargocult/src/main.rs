use log::warn;

#[no_mangle]
fn foo() {
  env_logger::init();
  println!("Hello Rust!");
}

#[no_mangle]
fn wrapper(func: fn() -> ()) {
  func();
  warn!("... from wrapper()");
}

fn main() {
  foo();
}
