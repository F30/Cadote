#[no_mangle]
fn foo() {
  println!("Hello Rust!");
}

#[no_mangle]
fn wrapper(func: fn() -> ()) {
  func();
  println!("... from wrapper()");
}

fn main() {
  foo();
}
