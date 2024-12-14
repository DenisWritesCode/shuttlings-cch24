use actix_web::{
    get,
    http::{header::LOCATION, Error},
    post,
    web::{self, ServiceConfig},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use serde::Deserialize;
use shuttle_actix_web::ShuttleActixWeb;
use std::net::{Ipv4Addr, Ipv6Addr};
use toml::Value;

/// query Params for Egregrious Encryption
#[derive(Deserialize)]
struct QueryParams {
    from: String,
    key: String,
}

#[derive(Deserialize)]
struct ReverseQueryParams {
    from: String,
    to: String,
}

#[get("/")]
async fn hello_world() -> &'static str {
    "Hello, bird!"
}

#[get("/-1/seek")]
async fn seek() -> impl Responder {
    // Redirect using "302 Found" HTTP Status Code
    HttpResponse::Found()
        .append_header((LOCATION, "https://www.youtube.com/watch?v=9Gc4QTqslN4"))
        .finish()
}

#[get("/2/dest")]
async fn produce_dest(query_params: web::Query<QueryParams>) -> Result<HttpResponse, Error> {
    let from_addr: Ipv4Addr = query_params.from.parse().expect("Invalid Ipv4 Address");
    let key_addr: Ipv4Addr = query_params.key.parse().expect("Invalid Ipv4 Address");

    let from_octets = from_addr.octets();
    let key_octets = key_addr.octets();

    let mut result_octets = [0u8; 4];
    for (i, (&f, &k)) in from_octets.iter().zip(key_octets.iter()).enumerate() {
        result_octets[i] = f.wrapping_add(k);
    }

    // Return a response with the IP in text form
    let result_ip = Ipv4Addr::from(result_octets);
    Ok(HttpResponse::Ok().body(result_ip.to_string()))
}

#[get("/2/key")]
async fn recover_key(query_params: web::Query<ReverseQueryParams>) -> Result<String, Error> {
    // Extract the 'from' & 'to'
    let from_addr: Ipv4Addr = query_params.from.parse().expect("Invalid Ipv4 Address");
    let to_addr: Ipv4Addr = query_params.to.parse().expect("Invalid Ipv4 Address");

    let from_octets = from_addr.octets();
    let to_octets = to_addr.octets();

    let mut result_octets = [0u8; 4];
    for (i, (&f, &t)) in from_octets.iter().zip(to_octets.iter()).enumerate() {
        result_octets[i] = t.wrapping_sub(f);
    }

    let key_addr = Ipv4Addr::from(result_octets);

    // Return the key in standard IPv6 string format
    Ok(key_addr.to_string())
}

#[get("/2/v6/dest")]
async fn produce_dest_v6(query_params: web::Query<QueryParams>) -> Result<HttpResponse, Error> {
    // Parse IPv6 addresses
    let from_addr: Ipv6Addr = query_params.from.parse().expect("Invalid Ipv6 Address");
    let key_addr: Ipv6Addr = query_params.key.parse().expect("Invalid Ipv6 Address");

    let from_octets = from_addr.octets();
    let key_octets = key_addr.octets();

    // XOR each corresponding octet
    let mut result_octets = [0u8; 16];
    for (i, (&f, &k)) in from_octets.iter().zip(key_octets.iter()).enumerate() {
        result_octets[i] = f ^ k;
    }

    let result_ip = Ipv6Addr::from(result_octets);

    // Return a response with the IP in text form
    Ok(HttpResponse::Ok().body(result_ip.to_string()))
}

#[get("/2/v6/key")]
async fn recover_key_v6(query_params: web::Query<ReverseQueryParams>) -> Result<String, Error> {
    // Extract 'from' & 'to'
    let from_addr: Ipv6Addr = query_params.from.parse().expect("Invalid Ipv6 Address");
    let to_addr: Ipv6Addr = query_params.to.parse().expect("Invalid Ipv6 Address");

    let from_octets = from_addr.octets();
    let to_octets = to_addr.octets();

    // XOR to recover the key: key = from XOR to
    let mut key_octets = [0u8; 16];
    for (i, (&f, &t)) in from_octets.iter().zip(to_octets.iter()).enumerate() {
        key_octets[i] = f ^ t;
    }

    let key_addr = Ipv6Addr::from(key_octets);

    // Return the key in standard IPv6 string format
    Ok(key_addr.to_string())
}

#[derive(Deserialize)]
struct Order {
    item: String,
    quantity: u32,
}

#[post("/5/manifest")]
async fn handle_manifest(req: HttpRequest, body: String) -> Result<HttpResponse, Error> {
    let content_type: &str = req.content_type();

    if content_type == "application/toml" {
        let manifest: Value = match toml::from_str(&body) {
            Ok(v) => v,
            Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid TOML")),
        };

        let orders = manifest
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("orders"))
            .and_then(|o| o.as_array());

        let mut valid_orders: Vec<(String, u32)> = Vec::new();

        if let Some(order_array) = orders {
            for order_val in order_array {
                // Each order should have item (String) and quantity (u32)
                let item = order_val.get("item").and_then(|i| i.as_str());
                let quantity = order_val.get("quantity").and_then(|q| q.as_integer());

                if let (Some(item_str), Some(qty)) = (item, quantity) {
                    // quantity must fit into u32
                    if qty >= 0 && qty <= u32::MAX as i64 {
                        valid_orders.push((item_str.to_string(), qty as u32));
                    }
                }
            }
        }

        if valid_orders.is_empty() {
            // No valid orders
            return Ok(HttpResponse::NoContent().finish());
        } else {
            // Return newline-separated list of orders
            let result_str = valid_orders
                .into_iter()
                .map(|(item, qty)| format!("{}: {}", item, qty))
                .collect::<Vec<_>>()
                .join("\n");

            return Ok(HttpResponse::Ok().body(result_str));
        }
    } else {
        // If not "application/toml", you might return an error:
        return Ok(HttpResponse::UnsupportedMediaType().finish());
    }
}
#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_world);
        cfg.service(seek);
        cfg.service(produce_dest);
        cfg.service(recover_key);
        cfg.service(produce_dest_v6);
        cfg.service(recover_key_v6);
        cfg.service(handle_manifest);
    };

    Ok(config.into())
}
