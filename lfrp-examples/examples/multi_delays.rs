extern crate lfrp_macros;

use lfrp_macros::frp;

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

frp! {
    mod MultiDelays;

    In {
        input: i32,
    }

    Out {
        output: i32,
    }

    let c1: i32 <- delay 2 -< input;
    let c2: i32 <- delay 1 -< c1;
    let c3: i32 <- delay 0 -< c2;
    let output = c3;
}

fn main() {
    let mut frp = MultiDelays::FRP::new();
    let mut input = MultiDelays::In { input: 0 };

    for i in 3.. {
        input.input = i;
        frp.run(&input);
        let output = frp.sample().unwrap();

        println!("{:?}", output);
        thread::sleep(Duration::from_millis(1000));
        print!("{}", ansi_escapes::EraseLines(2));
        io::stdout().flush().unwrap();
    }
}
