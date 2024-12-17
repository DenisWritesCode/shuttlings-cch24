use actix_web::{
    get,
    http::header::LOCATION,
    post,
    web::{self, ServiceConfig},
    Error, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use cargo_manifest::Manifest;
use serde::Deserialize;
use shuttle_actix_web::ShuttleActixWeb;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
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

#[derive(Debug, Deserialize)]
struct Order {
    item: String,
    quantity: u32,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    orders: Vec<Order>,
}

fn check_for_keyword() {}



fn match_manifest(manifest: Manifest) -> bool {

    println!("Manifest: {:#?}", manifest);

    // match manifest_result {
    //     Ok(manifest) => {
    //         // Access the package section
    //         if let Some(package) = manifest.package {
    //             // Access the metadata section within the package
    //             if let Some(metadata_value) = package.metadata {
    //                 // Serialize metadata_value back to TOML string
    //                 let metadata_toml = toml::to_string(&metadata_value).map_err(|_| actix_web::error::ErrorBadRequest("Invalid metadata 1"))?;

    //                 // Deserialize the TOML string into the Metadata struct
    //                 let metadata: Metadata = toml::from_str(&metadata_toml)
    //                     .map_err(|_| actix_web::error::ErrorBadRequest("Invalid metadata 2"))?;

    //                 let mut valid_orders: Vec<(String, u32)> = Vec::new();

    //                 for order in metadata.orders {
    //                     // Validate each order
    //                     // Since 'quantity' is already u32, no need to check its range
    //                     valid_orders.push((order.item, order.quantity));
    //                 }

    //                 println!("{:#?}", valid_orders);

    //                 if valid_orders.is_empty() {
    //                     // No valid orders found
    //                     println!("\n---------------------\nNo valid orders found\n--------------------\n");
    //                     return Ok(HttpResponse::NoContent().finish());
    //                 } else {
    //                     // Create a newline-separated list of orders
    //                     let result_str = valid_orders
    //                         .into_iter()
    //                         .map(|(item, qty)| format!("{}: {}", item, qty))
    //                         .collect::<Vec<_>>()
    //                         .join("\n");

    //                     return Ok(HttpResponse::Ok().body(result_str));
    //                 }
    //             } else {
    //                 // Metadata section is missing
    //                 return Err(actix_web::error::ErrorBadRequest(
    //                     "Invalid manifest: Missing metadata",
    //                 ));
    //             }
    //         } else {
    //             // Package section is missing
    //             return Err(actix_web::error::ErrorBadRequest(
    //                 "Invalid manifest: Missing package",
    //             ));
    //         }
    //     }
    //     Err(_) => {
    //         // Parsing failed; respond with 400 Bad Request
    //         println!("\n---------------------\nBad request\n--------------------\n");
    //         return Err(actix_web::error::ErrorBadRequest("Invalid manifest"));
    //     }
    // }
    
    // TODO: implement matching logic
    true
}

#[post("/5/manifest")]
async fn handle_manifest(req: HttpRequest, body: String) -> Result<HttpResponse, Error> {
    // Extract the Content-Type header
    let content_type: &str = req
        .headers()
        .get("Content-Type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("");

    match content_type {
        "application/toml" => {
            return Ok(HttpResponse::Ok().finish());
        }
        "application/json" => {
            return Ok(HttpResponse::Ok().finish());
        }
        "application/yaml" => {
            return Ok(HttpResponse::Ok().finish());
        }
        _ => {
            // Unsupported Content-Type; respond with 415 Unsupported Media Type
        return Err(actix_web::error::ErrorUnsupportedMediaType(
            "Unsupported Media Type",
        ));
            // return Ok(HttpResponse::UnsupportedMediaType().finish());
        }
    }

    // // Ensure the Content-Type is application/toml
    // if content_type == "application/toml" {
    //     // Extract the contents into a TOML.
    //     // Extract the contents into a TOML Table
    //     let body_toml: Result<toml::Value, _> = toml::from_str(&body);

    //     println!("Body TOML:{:#?}", body_toml);

    //     // Parse the manifest using cargo_manifest::Manifest
    //     let manifest_result = Manifest::from_slice(body.as_bytes());

    //     println!("Body TOML: {:#?}", body_toml);
    //     println!("Manifest Result: {:#?}", manifest_result);

    //     return Ok(HttpResponse::Ok().finish());

        
    // } else if content_type == "application/json" {
    //     // Extract the contents into a JSON object
    //     let body_toml: Result<toml::Value, _> = toml::from_str(&body);

    //     // Parse the manifest using cargo_manifest::Manifest
    //     let manifest_result = Manifest::from_slice(body.as_bytes());

    //     println!("Body TOML: {:#?}", body_toml);
    //     println!("Manifest Result: {:#?}", manifest_result);

    //     return Ok(HttpResponse::Ok().finish());
    // } else if content_type == "application/yaml" {
    //     // Extract the contents into a YAML object
    //     let body_toml: Result<toml::Value, _> = toml::from_str(&body);

    //     // Parse the manifest using cargo_manifest::Manifest
    //     let manifest_result = Manifest::from_slice(body.as_bytes());

    //     println!("Body TOML: {:#?}", body_toml);
    //     println!("Manifest Result: {:#?}", manifest_result);

    //     return Ok(HttpResponse::Ok().finish());
    // }
    
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
        cfg.service(handle_manifest); // Uncommented and added this line
        // Remove or comment out the manual registration below
        // cfg.service((actix_web::resource::Resource("/5/manifest"), web::post().to(handle_manifest)));
    };

    Ok(config.into())
}

