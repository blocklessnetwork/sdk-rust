# Blockless-sdk-rust

![](blockless.png)

### How to build

1. Install the rust with rustup, please visit the site 'https://rustup.rs/'.

2. Use follow command for build the project.

```bash
$ cargo build
```

HTTP example

```rust
use blockless_sdk::*;
use json;

fn main() {
    let opts = HttpOptions::new("GET", 30, 10);
    let http = BlocklessHttp::open("https://demo.bls.dev/tokens", &opts);
    let http = http.unwrap();
    let body = http.get_all_body().unwrap();
    let body = String::from_utf8(body).unwrap();
    let tokens = match json::parse(&body).unwrap() {
        json::JsonValue::Object(o) => o,
        _ => panic!("must be object"),
    };
    let tokens = match tokens.get("tokens") {
        Some(json::JsonValue::Array(tokens)) => tokens,
        _ => panic!("must be array"),
    };
    tokens.iter().for_each(|s| {
        println!("{:?}", s.as_str());
    });
}
```

## Install from [crates.io](https://crates.io/crates/blockless-sdk)

```sh
cargo add blockless-sdk
```

## Examples

Examples are in the [`examples`](./examples/) directory.

### [Coingecko Oracle](./examples/coingecko_oracle.rs)

```sh
# Build example
cargo build --release --target wasm32-wasip1 --example coingecko_oracle

# Run example with blockless runtime
echo "bitcoin" | runtime target/wasm32-wasip1/release/examples/coingecko_oracle.wasm --permission https://api.coingecko.com/
```

### [HTTP](./examples/httpbin.rs)

```sh
# Build example
cargo build --release --target wasm32-wasip1 --example httpbin

# Run example with blockless runtime
~/.bls/runtime/bls-runtime target/wasm32-wasip1/release/examples/httpbin.wasm --permission http://httpbin.org/anything
```

### [LLM-MCP](./examples/llm-mcp.rs)

```sh
# Build example
cargo build --release --target wasm32-wasip1 --example llm-mcp

# Run example with blockless runtime and tool servers running
# Make sure you have the tool servers running on ports 3001 and 3002
~/.bls/runtime/bls-runtime target/wasm32-wasip1/release/examples/llm-mcp.wasm
```

## Examples list

| Example | Description | [Browser runtime](https://github.com/blocklessnetwork/b7s-browser) support | [Native runtime](https://github.com/blessnetwork/bls-runtime) support |
| ------- | ----------- | --------------- | --------------- |
| [coingecko_oracle](./examples/coingecko_oracle.rs) | Coingecko Oracle to query price of bitcoin from coingecko | ✅ | ✅ |
| [httpbin](./examples/httpbin.rs) | HTTP to query anything from httpbin | ✅ | ✅ |
| [llm](./examples/llm.rs) | LLM to chat with `Llama-3.1-8B-Instruct-q4f32_1-MLC` and `SmolLM2-1.7B-Instruct-q4f16_1-MLC` models | ✅ | ✅ |
| [llm-mcp](./examples/llm-mcp.rs) | LLM with MCP (Model Control Protocol) demonstrating tool integration using SSE endpoints | ✅ | ✅ |


## Testing

The SDK uses FFI (Foreign Function Interface) calls that are only available in the Blockless WASM runtime environment.
To run tests without host runtime, use the `mock-ffi` feature which provides mock implementations:

```bash
cargo test --all --features mock-ffi
```

This feature enables mock implementations of all FFI functions, allowing you to:
- Test SDK struct creation and configuration
- Test error handling logic
- Verify API contracts without needing the runtime
- Run unit tests in CI/CD pipelines

Note:
- The mocks return predictable test data and don't perform actual network requests or system calls.
- Only one implementation of the FFI functions is allowed to be mocked.
