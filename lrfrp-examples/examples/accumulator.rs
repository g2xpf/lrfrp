use lrfrp_macros::frp;

use std::io::{self, Write};
use std::{thread, time::Duration};

frp! {
    mod Accumulator;

    In {
        input: i32,
    }

    Out {
        output: i32,
    }

    Args {
        accumulator_init: i32,
    }

    let output = input + output_delayed;
    let output_delayed: i32 <- delay accumulator_init -< output;
}

fn main() {
    let args = Accumulator::Args {
        accumulator_init: 0,
    };
    let mut frp = Accumulator::FRP::new(args);
    let mut input = Accumulator::In { input: 0 };

    for i in 0.. {
        input.input = i;
        frp.run(&input);
        let output = frp.sample().unwrap();

        println!("{:?}", output);
        thread::sleep(Duration::from_millis(1000));
        print!("{}", ansi_escapes::EraseLines(2));
        io::stdout().flush().unwrap();
    }
}
