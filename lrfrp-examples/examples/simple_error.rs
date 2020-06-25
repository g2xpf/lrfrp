use lrfrp_macros::frp;

frp! {
  mod Example;

  In { foo: i32 }
  Out { bar : i32 }

  let hoge = 1 + foo;
}

fn main() {}
