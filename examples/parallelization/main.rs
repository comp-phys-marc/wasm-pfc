extern crate wasmparser;

use std::env;
use wasmparser::flow_mapper;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} in.wasm.", args[0]);
        return;
    }

    flow_mapper::new_mapper().map(&args[1]);
}
