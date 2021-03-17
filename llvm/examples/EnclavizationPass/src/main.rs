fn foo_enclaved_(param: &i64, lala: &str) -> i64 {
  println!("The number is: {} {}", param, lala);
  99
}

fn bar_enclaved_() -> (f64, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) {
  (1337.0, true, false, true, false, true, false, true, false, true, false, true)
}

fn main() {
  let val = 42;
  let lala = "abcdefghi";
  foo_enclaved_(&val, &lala[3..]);
  let x = foo_enclaved_(&23, &lala[4..]);
  println!("Return value: {}", x);
  let y = bar_enclaved_();
  println!("Return value: {:?}", y);
}
