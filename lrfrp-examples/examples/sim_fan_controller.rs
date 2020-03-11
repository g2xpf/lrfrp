use std::{thread, time::Duration};

use lrfrp_macros::frp;

frp! {
    mod SimFanController;

    Args {
        fan_init: bool,
    }

    In {
        tmp: f32,
        hmd: f32
    }

    Out {
        fan: bool,
    }

    fn calc_di(tmp: f32, hmd: f32) -> f32 = 0.81 * tmp + 0.01 * hmd * (0.99 * tmp - 14.3) + 46.3;

    let di = calc_di(tmp, hmd);
    let fan = di >= th;
    let fan_delayed: bool <- delay fan_init -< fan;
    let th = 75.0 + if fan_delayed then -0.5 else 0.5;
}

fn main() {
    let args = SimFanController::Args { fan_init: false };
    let mut frp = SimFanController::FRP::new(args);

    let mut input = SimFanController::In {
        tmp: 30.0,
        hmd: 60.0,
    };
    let (mut dt, mut dh) = (0.5, 1.0);

    loop {
        if input.tmp > 35.0 || input.tmp < 20.0 {
            dt = -dt;
        }
        if input.hmd > 80.0 || input.hmd < 50.0 {
            dh = -dh;
        }

        input.tmp += dt;
        input.hmd += dh;

        frp.run(&input);
        let output = frp.sample().unwrap();

        println!(
            "tmp={:2.2}, hmd={:2.2}, fan: {:-3}",
            input.tmp,
            input.hmd,
            if output.fan { "ON" } else { "OFF" }
        );

        thread::sleep(Duration::from_millis(33))
    }
}
