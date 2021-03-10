fn foo_wrapped_(param: &i64, lala: &str) -> i64 {
  println!("The number is: {} {}", param, lala);
  99
}

fn main() {
  let val = 42;
  let lala = "abcdefghi";
  foo_wrapped_(&val, &lala[3..]);
  let x = foo_wrapped_(&23, &lala[4..]);
  println!("Return value: {}", x)
}
