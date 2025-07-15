use blockless_sdk::http::HttpClient;
use blockless_sdk::memory::read_stdin;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
struct CoinPrice {
    id: String,
    price: u64,
    currency: String,
}

fn main() {
    // read coin id from stdin
    let mut buf = [0; 1024];
    let len = read_stdin(&mut buf).unwrap();
    let coin_id = std::str::from_utf8(&buf[..len as usize])
        .unwrap_or_default()
        .trim();

    // perform http request
    let client = HttpClient::new();
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
        coin_id
    );
    let response = client.get(&url).send().unwrap();
    let body = response.bytes().to_vec(); // e.g. {"bitcoin":{"usd":67675}}

    // println!("{}", String::from_utf8(body.clone()).unwrap());

    // parse the json response; extrac usd price
    let json: serde_json::Result<HashMap<String, HashMap<String, f64>>> =
        serde_json::from_slice(&body);
    let Ok(data) = json else {
        eprintln!("Failed to parse JSON");
        return;
    };
    let Some(coin_data) = data.get(coin_id) else {
        eprintln!("Coin not found in response.");
        return;
    };
    let Some(usd_price) = coin_data.get("usd") else {
        eprintln!("USD price not found for {}.", coin_id);
        return;
    };

    let coin_price = CoinPrice {
        id: coin_id.to_string(),
        price: (*usd_price * 1_000_000.0) as u64, // price in 6 decimals
        currency: "usd".to_string(),
    };
    println!("{}", json!(coin_price));
}
