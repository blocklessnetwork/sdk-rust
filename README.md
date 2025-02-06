# Blockless SDK for Rust

![](blockless.png)

### How to Build the Project

1. Install Rust using rustup by visiting the website [https://rustup.rs/](https://rustup.rs/).

2. To build the project, use the following command:

```bash
$ cargo build
```

### HTTP Example

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
        _ => panic!("Expected an object"),
    };
    let tokens = match tokens.get("tokens") {
        Some(json::JsonValue::Array(tokens)) => tokens,
        _ => panic!("Expected an array"),
    };
    tokens.iter().for_each(|s| {
        println!("{:?}", s.as_str());
    });
}
```

## Install via [crates.io](https://crates.io/crates/blockless-sdk)

```sh
cargo add blockless-sdk
```

## Examples

Examples can be found in the [`examples`](./examples/) directory.

### [Coingecko Oracle](./examples/coingecko_oracle.rs)

```sh
# Build the example
cargo build --release --target wasm32-wasip1 --example coingecko_oracle

# Run the example with the Blockless runtime
echo "bitcoin" | runtime target/wasm32-wasip1/release/examples/coingecko_oracle.wasm --permission https://api.coingecko.com/
```

### [HTTP](./examples/httpbin.rs)

```sh
# Build the example
cargo build --release --target wasm32-wasip1 --example httpbin

# Run the example with the Blockless runtime
~/.bls/runtime/bls-runtime target/wasm32-wasip1/release/examples/httpbin.wasm --permission http://httpbin.org/anything
```

## Examples List

| Example | Description | [Browser Runtime Support](https://github.com/blocklessnetwork/b7s-browser) | [Native Runtime Support](https://github.com/blessnetwork/bls-runtime) |
| ------- | ----------- | ---------------------------------------------------- | -------------------------------- |
| [coingecko_oracle](./examples/coingecko_oracle.rs) | Coingecko Oracle to query Bitcoin price from Coingecko | ✅ | ✅ |
| [httpbin](./examples/httpbin.rs) | HTTP to query anything from httpbin | ✅ | ✅ |
| [llm](./examples/llm.rs) | LLM to chat with `Llama-3.1-8B-Instruct-q4f32_1-MLC` and `SmolLM2-1.7B-Instruct-q4f16_1-MLC` models | ✅ | ❌ |