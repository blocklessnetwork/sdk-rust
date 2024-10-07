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
cargo build --release --target wasm32-wasi --example coingecko_oracle

# Run example with blockless runtime
echo "bitcoin" | runtime target/wasm32-wasi/release/examples/coingecko_oracle.wasm --permission https://api.coingecko.com/
```

### [HTTP](./examples/httpbin.rs)

```sh
# Build example
cargo build --release --target wasm32-wasi --example httpbin

# Run example with blockless runtime
runtime target/wasm32-wasi/release/examples/httpbin.wasm --permission http://httpbin.org/anything
```
