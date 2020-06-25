#![feature(start, lang_items)]
// std を使用しない
#![no_std]
// #![no_main]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

extern crate libc;

// use core::panic::PanicInfo;
use lrfrp_macros::frp;

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

pub fn bench(c: &mut Criterion) {
    let mut frp = FanController::FRP::new();

    let mut input = FanController::In {
        tmp: 30.0,
        hmd: 60.0,
    };

    let (mut dt, mut dh) = (0.5, 1.0);

    c.bench_function("fan_controller", |b| {
        b.iter(|| {
            for _ in 0..black_box(3_000_000) {
                if input.tmp > 35.0 || input.tmp < 20.0 {
                    dt = -dt;
                }
                if input.hmd > 80.0 || input.hmd < 50.0 {
                    dh = -dh;
                }

                input.tmp += dt;
                input.hmd += dh;

                frp.run(&input);
                frp.sample().unwrap();
            }
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
