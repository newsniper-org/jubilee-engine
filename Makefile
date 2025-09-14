RUSTFLAGS = '--cfg getrandom_backend="wasm_js"'

build: src/lib.rs Cargo.toml
	RUSTFLAGS=$(RUSTFLAGS) wasm-pack build --target bundler --out-dir ../pkg