# Bevy-Editor Viewport Game
To compile and make ready, first install these two packages:
```bash
cargo install cargo-make
cargo install wasm-bindgen-cli
```
Then to actually compile, run this:
```bash
# Normally
cargo build-wasm && cargo make wasm-bindgen
# If you have the EditorGame repo next to this one and want to move results directly to there
cargo build-wasm && cargo make wasm-bindgen-and-move
```

