# axonpdf

A PDF engine written in Rust. Work in progress: nothing here is ready to use yet.

## Development setup

These steps are for macOS and Linux.

### Rust

Install Rust with [rustup](https://rustup.rs), then add the target that the WebAssembly build uses:

```sh
rustup target add wasm32-unknown-emscripten
```

### C++ compiler

The engine includes Yoga, a C++20 library, and compiles it during the build. On macOS, the Xcode
Command Line Tools provide the compiler (`xcode-select --install`). On Linux, install clang or a
recent GCC.

### Emscripten

Emscripten builds the engine and Yoga into one WebAssembly file for browsers and Node. The project
uses Emscripten 7.0.0, installed with emsdk. One copy can serve every checkout, so it can live
anywhere; `.tools/emsdk` inside this repo is ignored by git.

```sh
git clone https://github.com/emscripten-core/emsdk.git .tools/emsdk
cd .tools/emsdk
./emsdk install 7.0.0
./emsdk activate 7.0.0
```

Then, in each new shell, run `source .tools/emsdk/emsdk_env.sh` to put `emcc` on your `PATH`.
`./emsdk activate --permanent` does this for you, but it edits your shell profile.

### Node

Install Node 24 or newer for the JavaScript side of the project.

### Check

```sh
rustc --version
rustup target list --installed   # should include wasm32-unknown-emscripten
emcc --version
node --version
```

## Build and test

```sh
cargo test                                      # native
source .tools/emsdk/emsdk_env.sh
cargo test --target wasm32-unknown-emscripten   # WebAssembly, run in Node
```

The layout tests in `crates/engine/tests/yoga.rs` compare the engine's boxes with those from
`yoga-layout` 3.2.1, Meta's own build of the same Yoga release. The expected boxes are stored in
`crates/engine/tests/fixtures/yoga.json`. After adding or changing a case there, write them again:

```sh
cd parity && npm ci && npm run yoga
```

## License

MIT
