all: build

build:
	cargo build --target wasm32-wasi --release
	md5sum target/wasm32-wasi/release/exam1.wasm
