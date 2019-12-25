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

    let output = 2 + -{
        let p1 = input + 1;
        let p2 = p1 + input;
        p2
    };
}

fn main() {
    let mut frp = MultiDelays::FRP::new();
    let mut input = MultiDelays::In { input: 0 };

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
