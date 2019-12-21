extern crate lfrp_macros;

use lfrp_macros::frp;

frp! {
    mod SimFanController;

    In {
        a: Float<Int>,
        b: (T, )
    }

    Out {
        a: Float<Int>,
        b: Int<[(_)]>
    }

    Args {
        a: Float<_>,
        b: Int
    }
}
