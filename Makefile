all: build

build:
	cargo build --target wasm32-wasi --release --example httpbin
