fn foo_wrapped_(param: &i64, lala: &str) {
  println!("The number is: {} {}", param, lala);
}

fn main() {
  let val = 42;
  let lala = "abcdefghi";
  foo_wrapped_(&val, &lala[3..]);
  foo_wrapped_(&23, &lala[4..]);
}
