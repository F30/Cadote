#[no_mangle]
fn foo_wrapped_() {
  println!("Hello Rust!");
}

#[no_mangle]
fn wrapper(func: fn() -> ()) {
  func();
  println!("... from wrapper()");
}

fn main() {
  foo_wrapped_();
}
