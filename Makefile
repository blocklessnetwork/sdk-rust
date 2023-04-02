all: build

ALL_FILES = $(shell find src -name "*.rs")

build: target/exam1.wasm

target/exam1.wasm: $(ALL_FILES)
	cargo build --target wasm32-wasi --release
	mv target/wasm32-wasi/release/exam1.wasm target
	md5sum target/exam1.wasm
