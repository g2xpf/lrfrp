use lrfrp_macros::frp;

frp! {
    mod FanController;

    Args { fan_init: bool, tmp: f32 }
    In { hmd: f32 }
    Out { di: f32, fan: bool }

    fn calc_di(tmp: f32, hmd: f32) -> f32
        = 0.81 * tmp + 0.01 * hmd * (0.99 * tmp - 14.3) + 46.3;

    let di = calc_di(tmp, hmd);
    let fan = di >= th;
    let fan_delayed: bool <- delay fan_init -< fan;
    let th = 75.0 + if fan_delayed then -0.5 else 0.5;
}

fn main() {}
