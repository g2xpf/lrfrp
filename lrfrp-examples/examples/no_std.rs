#![feature(start, libc, lang_items, rustc_private)]
// std を使用しない
#![no_std]
#![no_main]

extern crate libc;

use core::panic::PanicInfo;

use lrfrp_macros::frp;

// アキュムレータの LRFRP プログラム
frp! {
    mod Accumulator;

    Args {
        acc_init: u32,
    }

    // 入力時変値の宣言
    In {
        input: u32,
    }

    // 出力時変値の宣言
    Out {
        res: u32,
    }

    let cell: u32 <- delay acc_init -< res;
    let res = input + cell;
}

extern "C" {
    pub fn printf(format: *const u8, ...) -> i32;
}

#[no_mangle]
pub extern "C" fn main(_nargs: i32, _args: *const *const u8) -> i32 {
    // アキュムレータの初期化
    let args = Accumulator::Args { acc_init: 0 };
    let mut frp = Accumulator::FRP::new(args);

    // 入力の定義
    let mut input = Accumulator::In { input: 0 };

    // 入力を変えて実行
    for i in 0..=10 {
        input.input = i;

        frp.run(&input);
        let output = frp.sample().unwrap();

        unsafe {
            // アキュムレータによって計算された, 入力の総和を出力
            printf(
                b"(input, sum): (%d,%d)\n\0" as *const u8,
                input.input,
                output.res,
            );
        }
    }

    0
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
