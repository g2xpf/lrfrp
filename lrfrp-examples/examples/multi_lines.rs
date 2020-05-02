use lrfrp_macros::frp;

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

frp! {
    mod MultiDelays;

    Args {
        a1: f32,
        a2: f32,
    }

    In {
        input: f32,
    }

    Out {
        output: f32,
    }

    let output = 2.0 + -{
        let p1 = input + 1.0;
        let p2 = p1 + {
            let a2 = 3.0;
            input + c1 * a2 + a2
        };
        p2
    };

    let c1 = {
        let p1 = input + 1.0;
        let p2 = input + {
            p1 + a1
        };
        {
            p2 + 1.0
        }
    };
}

fn main() {
    let args = MultiDelays::Args { a1: 10.0, a2: 3.0 };
    let mut frp = MultiDelays::FRP::new(args);
    let mut input = MultiDelays::In { input: 0.0 };

    for i in 0.. {
        input.input = i as f32;
        frp.run(&input);
        let output = frp.sample().unwrap();

        println!("{:?}", output);
        thread::sleep(Duration::from_millis(1000));
        print!("{}", ansi_escapes::EraseLines(2));
        io::stdout().flush().unwrap();
    }
}
