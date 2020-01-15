use lfrp_macros::frp;

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

frp! {
    mod SimFanController;

    fn calc_di(tmp: f32, hmd: f32) -> f32 = 0.81 * tmp + 0.01 * hmd * (0.99 * tmp - 14.3) + 46.3;

    fn calc_fan(di: f32, th: f32) -> bool = di >= th;

    fn calc_th(fan: bool) -> f32 = 75.0 + if fan then -0.5 else 0.5;

    Args {
        fan_init: bool,
    }

    In {
        tmp: f32,
        hmd: f32
    }

    Out {
        di: f32,
        fan: bool,
    }

    let di = calc_di(tmp, hmd);
    let fan = calc_fan(di, th);
    let fan_delayed: bool <- delay fan_init -< fan;
    let th = calc_th(fan_delayed);
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
        // update parameters
        if input.tmp > 35.0 || input.tmp < 20.0 {
            dt = -dt;
        }
        if input.hmd > 80.0 || input.hmd < 50.0 {
            dh = -dh;
        }

        input.tmp += dt;
        input.hmd += dh;

        // transaction
        frp.run(&input);
        let output = frp.sample().unwrap();

        // print
        println!(
            "tmp={:2.2}, hmd={:2.2}, di={:2.2}, fan: {:-3}",
            input.tmp,
            input.hmd,
            output.di,
            if output.fan { "ON" } else { "OFF" }
        );
        thread::sleep(Duration::from_millis(200));
        print!("{}", ansi_escapes::EraseLines(2));
        io::stdout().flush().unwrap();
    }
}
