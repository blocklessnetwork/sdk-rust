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
