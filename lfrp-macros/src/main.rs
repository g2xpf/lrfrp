extern crate lfrp_macros;
use lfrp_macros::frp;

frp! {
    mod SimFanController;

    Args {
        fan_init: Bool
    }

    In {
        tmp: f32,
        hmd: f32
    }

    Out {
        di: Float,
        fan: Bool,
    }

    let di = 0.81 * tmp + 0.01 * hmd * (0.99 * tmp - 14.3) + 46.3;
    let fan = di >= th;
    let fan_delayed: Bool <- delay fan_init -< fan;
    let th = 75.0 + if fan_delayed then -0.5 else 0.5;
}

fn main() {}
