extern crate wasmparser;

use std::env;
use wasmparser::flow_mapper;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} in.wasm.", args[0]);
        return;
    }

    let mut mapper = flow_mapper::new_mapper();

    println!("Analyzing {}...", args[1]);

    let buf: Vec<u8> = mapper.read_wasm(&args[1]).unwrap();
    let nodes = mapper.map(buf);
    println!("{:#?}", nodes);
    mapper.print_tree(nodes);
}
