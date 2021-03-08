fn foo_wrapped_(param: &i64, lala: &str) {
  println!("The number is: {} {}", param, lala);
}

#[no_mangle]
fn wrapper(func: fn(&i64, &str) -> (), param: &i64, lala: &str) {
  func(param, lala);
  println!("... from wrapper()");
}

fn main() {
  let val = 42;
  let lala = "abcdefghi";
  foo_wrapped_(&val, &lala[3..]);
}
