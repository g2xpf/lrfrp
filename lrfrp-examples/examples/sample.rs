use lrfrp_macros::frp;

frp! {
  mod SimpleFanController;

  In { tmp : f32 }
  Out { fan : bool }
  Args { fan_init : bool }

  let fan = tmp >= th;
  let th = 30.0 + if fan_delayed then -1.0 else 1.0;
  let fan_delayed: bool <- delay fan_init -< fan;
}

fn main() {
    let args = SimpleFanController::Args { fan_init: false };
    let frp = SimpleFanController::FRP::new(args);
}
