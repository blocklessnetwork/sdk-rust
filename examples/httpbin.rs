use blockless_sdk::*;

fn main() {
    let opts = HttpOptions::new("GET", 30, 10);
    let http = BlocklessHttp::open("http://httpbin.org/anything", &opts);
    let http = http.unwrap();
    let body = http.get_all_body().unwrap();
    let body = String::from_utf8(body).unwrap();
    let bodies = match json::parse(&body).unwrap() {
        json::JsonValue::Object(o) => o,
        _ => panic!("must be object"),
    };
    let headers = match bodies.get("headers") {
        Some(json::JsonValue::Object(headers)) => headers,
        _ => panic!("must be array"),
    };
    headers.iter().for_each(|s| {
        println!("{} = {}", s.0, s.1);
    });
}
