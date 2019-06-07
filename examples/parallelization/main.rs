extern crate wasmparser;

use std::env;
use wasmparser::parallelize;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} in.wasm.", args[0]);
        return;
    }

    let mut mapper = parallelize::new_mapper();

    println!("Analyzing {}...", args[1]);

    let buf: Vec<u8> = mapper.read_wasm(&args[1]).unwrap();
    let nodes = mapper.map(buf);

    // println!("{:#x?}", nodes);
    // mapper.print_tree(nodes);

    let mut node = &nodes[&5];
    let collapsed_node = node.clone().collapse();
    println!("{:#x?}", collapsed_node);
}
