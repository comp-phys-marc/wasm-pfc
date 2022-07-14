## QuantEmu

Note this repository hosts a minimal addition to the WebAssembly binary file decoder necessary to demonstrate (with the help of pyQUBO) that WebAssembly may be lowered to pyQUBO compatible constraint expressions via a method insipred by PFC (Parallelizing Fortran Compiler) and is NOT A COMPLETE IMPLEMENTATION ONLY A DEMO.

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
