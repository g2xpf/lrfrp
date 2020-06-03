#![feature(start, lang_items)]
#![no_std]
#![no_main]

extern crate libc;

use core::panic::PanicInfo;
use lrfrp_macros::frp;

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern "C" {
    pub fn printf(format: *const u8, ...) -> i32;
}

// FanController の LRFRP プログラム
frp! {
    mod FanController;

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
    let fan_delayed: bool <- delay False -< fan;
    let th = 75.0 + if fan_delayed then -0.5 else 0.5;
}

#[no_mangle]
pub extern "C" fn main(_nargs: i32, _args: *const *const u8) -> i32 {
    let mut frp = FanController::FRP::new();

    let mut input = FanController::In {
        tmp: 30.0,
        hmd: 60.0,
    };

    let (mut dt, mut dh) = (0.5, 1.0);

    let mut output = FanController::Out::default();

    for _ in 0..1_000_000_000 {
        if input.tmp > 35.0 || input.tmp < 20.0 {
            dt = -dt;
        }
        if input.hmd > 80.0 || input.hmd < 50.0 {
            dh = -dh;
        }

        input.tmp += dt;
        input.hmd += dh;

        frp.run(&input);
        output = frp.sample().unwrap().clone();
    }

    unsafe {
        printf(b"output: %d" as *const u8, output.fan as u32);
    }

    0
}
