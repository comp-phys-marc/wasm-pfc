# The WebAssembly binary file decoder in Rust

[![Build Status](https://travis-ci.org/yurydelendik/wasmparser.rs.svg?branch=master)](https://travis-ci.org/yurydelendik/wasmparser.rs)
[![crates.io link](https://img.shields.io/crates/v/wasmparser.svg)](https://crates.io/crates/wasmparser)

The decoder library provides lightwight and fast decoding/parsing of WebAssembly binary files.

The other goal is minimal memory footprint. For this reason, there is no AST or IR of WebAssembly data.

See also its sibling at https://github.com/wasdk/wasmparser


## Documentation

The documentation and examples can be found at the https://docs.rs/wasmparser/


## Example

```rust
use wasmparser::WasmDecoder;
use wasmparser::Parser;
use wasmparser::ParserState;

fn get_name(bytes: &[u8]) -> &str {
  str::from_utf8(bytes).ok().unwrap()
}

fn main() {
  let ref buf: Vec<u8> = read_wasm_bytes();
  let mut parser = Parser::new(buf);
  loop {
    let state = parser.read();
    match *state {
        ParserState::BeginWasm { .. } => {
            println!("====== Module");
        }
        ParserState::ExportSectionEntry { field, ref kind, .. } => {
            println!("  Export {} {:?}", get_name(field), kind);
        }
        ParserState::ImportSectionEntry { module, field, .. } => {
            println!("  Import {}::{}", get_name(module), get_name(field))
        }
        ParserState::EndWasm => break,
        _ => ( /* println!(" Other {:?}", state) */ )
    }
  }
}
```


## Fuzzing

To fuzz test wasmparser.rs, switch to a nightly Rust compiler and install [cargo-fuzz]:

```
cargo install cargo-fuzz
```

Then, from the root of the repository, run:

```
cargo fuzz run parse
```

If you want to use files as seeds for the fuzzer, add them to `fuzz/corpus/parse/` and restart cargo-fuzz.

[cargo-fuzz]: https://github.com/rust-fuzz/cargo-fuzz

## QuantEmu

To test the PFC implementation:

```
cargo run --example parallelize  ./tests/parallelization/math.wasm
```

To create or update a WASM file, write the .wat by hand, and run wat2wasm. For example:

```
wat2wasm tests/parallelization/math.wat -o tests/parallelization/math.wasm
```

To enable verbose stack trace debugging, export this variable before running the program:

```
export RUST_BACKTRACE=1
```

wat2wasm is a part of the open source WebAssembly Binary Toolkit [wabt](https://github.com/WebAssembly/wabt).

## License

Copyright 2019 Marcus Edwards

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at:

```
http://www.apache.org/licenses/LICENSE-2.0
```

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
