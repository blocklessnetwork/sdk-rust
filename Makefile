all: build

build:
	cargo build --target wasm32-wasip1 --release --example httpbin
