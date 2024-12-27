use actix_web::{
    get,
    http::header::ContentType,
    http::header::LOCATION,
    post,
    web::{self, ServiceConfig},
    Error, HttpRequest, HttpResponse, Responder,
};
use cargo_manifest::Manifest;
use serde::Deserialize;
use serde_json;
use serde_yaml;
use shuttle_actix_web::ShuttleActixWeb;
use std::net::{Ipv4Addr, Ipv6Addr};
use toml;

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

// ===================================================
// TASK: /5/manifest - ignoring invalid orders
// ===================================================

#[post("/5/manifest")]
async fn handle_manifest(body: String, req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    // 1) Check Content-Type to determine how to parse.
    let content_type = req
        .headers()
        .get("Content-Type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or_default()
        .to_lowercase();

    // 2) Basic top-level validation:
    //    - For TOML: use cargo_manifest::Manifest
    //    - For JSON / YAML: minimal check that "package.name" is a string
    // 2) Basic top-level validation + rust-version check:
    let top_level_ok = match content_type.as_str() {
        "application/toml" => {
            cargo_manifest::Manifest::from_slice(body.as_bytes()).is_ok()
        }
        "application/json" => serde_json::from_str::<serde_json::Value>(&body)
            .map(|v| is_valid_package_json(&v))
            .unwrap_or(false),
        "application/yaml" => serde_yaml::from_str::<serde_yaml::Value>(&body)
            .map(|v| is_valid_package_yaml(&v))
            .unwrap_or(false),
        _ => {
            return Ok(HttpResponse::UnsupportedMediaType().finish());
        }
    };

    println!("content-type: {:#?}", content_type);
    println!("Body: {}", &body);

    if !top_level_ok {
        return Ok(HttpResponse::BadRequest().body("Invalid manifest"));
    }

    // 3) Parse the same data into our "unified" type, so we can ignore invalid orders.
    let unified_val = parse_as_unified_value(&body, content_type.as_str())
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid manifest"))?;

    // 4) Check rust-version here, after parsing into UnifiedValue
    if !has_valid_rust_version(&unified_val) {
        return Ok(HttpResponse::BadRequest().body("Invalid manifest"));
    }

    // 5) Check "magic keyword" => "Christmas 2024"
    if !contains_magic_keyword(&unified_val) {
        return Ok(HttpResponse::BadRequest().body("Magic keyword not provided"));
    }

    // 6) Extract valid orders ignoring those with invalid quantity
    let valid_orders = extract_valid_orders(&unified_val);

    // 7) If none are valid => 204
    if valid_orders.is_empty() {
        return Ok(HttpResponse::NoContent().finish());
    }

    // 8) Print them line-by-line
    let response_body = orders_to_string(&valid_orders);
    Ok(HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(response_body))
}

// -----------------------------------------------------------------------------
// Our "unified" Value enum + parse logic
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum UnifiedValue {
    Toml(toml::Value),
    Json(serde_json::Value),
    Yaml(serde_yaml::Value),
}

/// Parse the body as either TOML, JSON, or YAML into a UnifiedValue.
fn parse_as_unified_value(body: &str, content_type: &str) -> Result<UnifiedValue, ()> {
    match content_type {
        "application/toml" => {
            let v = toml::from_str::<toml::Value>(body).map_err(|_| ())?;
            Ok(UnifiedValue::Toml(v))
        }
        "application/json" => {
            let v = serde_json::from_str::<serde_json::Value>(body).map_err(|_| ())?;
            Ok(UnifiedValue::Json(v))
        }
        "application/yaml" => {
            let v = serde_yaml::from_str::<serde_yaml::Value>(body).map_err(|_| ())?;
            Ok(UnifiedValue::Yaml(v))
        }
        _ => Err(()), // Should be unreachable because we check beforehand
    }
}

fn has_valid_rust_version(value: &UnifiedValue) -> bool {
    println!("has_valid_rust_version: {:#?}", &value);
    match value {
        UnifiedValue::Toml(toml) => {
            toml.get("package")
                .and_then(|pkg| pkg.get("rust-version"))
                .map_or(true, |rv| rv.as_str().map_or(false, |s| s.parse::<f64>().is_ok()))
        }
        UnifiedValue::Json(json) => {
            json.as_object()
                .and_then(|obj| obj.get("package"))
                .and_then(|pkg| pkg.get("rust-version"))
                .map_or(true, |rv| rv.as_str().map_or(false, |s| s.parse::<f64>().is_ok()))
        }
        UnifiedValue::Yaml(yaml) => {
            yaml.as_mapping()
                .and_then(|obj| obj.get(&serde_yaml::Value::String("package".to_string())))
                .and_then(|pkg| pkg.get(&serde_yaml::Value::String("rust-version".to_string())))
                .map_or(true, |rv| rv.as_str().map_or(false, |s| s.parse::<f64>().is_ok()))
        }
    }
}

// -----------------------------------------------------------------------------
// Minimal "valid cargo package" checks for JSON/YAML
// -----------------------------------------------------------------------------

fn is_valid_package_json(v: &serde_json::Value) -> bool {
    v.get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(|name| name.as_str())
        .is_some()
}

fn is_valid_package_yaml(v: &serde_yaml::Value) -> bool {
    if let Some(pkg) = v.get("package") {
        if let Some(name) = pkg.get("name") {
            return name.as_str().is_some();
        }
    }
    false
}

// -----------------------------------------------------------------------------
// This trait is the key. We must implement it for `UnifiedValue`.
// -----------------------------------------------------------------------------

trait GenericValue {
    /// Attempt to descend into a nested path: e.g. ["package","metadata"]
    /// Return the sub-`UnifiedValue` if found.
    fn get_path(&self, path: &[&str]) -> Option<UnifiedValue>;

    /// Return a string if this node is a string
    fn as_str(&self) -> Option<&str>;

    /// Return a u32 if this node is an integer within range
    fn as_u32(&self) -> Option<u32>;

    /// Return an owned Vec of sub-`UnifiedValue` if this node is an array
    fn as_array(&self) -> Option<Vec<UnifiedValue>>;
}

// -----------------------------------------------------------------------------
// Implement the trait for `UnifiedValue`
// -----------------------------------------------------------------------------

impl GenericValue for UnifiedValue {
    fn get_path(&self, path: &[&str]) -> Option<UnifiedValue> {
        match self {
            UnifiedValue::Toml(t) => {
                let mut current = t;
                for key in path {
                    current = current.get(*key)?; // each step
                }
                // Re-wrap in `UnifiedValue::Toml(...)`
                Some(UnifiedValue::Toml(current.clone()))
            }
            UnifiedValue::Json(j) => {
                let mut current = j;
                for key in path {
                    current = current.get(*key)?;
                }
                Some(UnifiedValue::Json(current.clone()))
            }
            UnifiedValue::Yaml(y) => {
                let mut current = y;
                for key in path {
                    current = match current {
                        serde_yaml::Value::Mapping(map) => {
                            let key_val = serde_yaml::Value::String(key.to_string());
                            map.get(&key_val)?
                        }
                        _ => return None,
                    };
                }
                Some(UnifiedValue::Yaml(current.clone()))
            }
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            UnifiedValue::Toml(t) => t.as_str(),
            UnifiedValue::Json(j) => j.as_str(),
            UnifiedValue::Yaml(y) => y.as_str(),
        }
    }

    fn as_u32(&self) -> Option<u32> {
        match self {
            // TOML: integers are i64
            UnifiedValue::Toml(t) => t.as_integer().and_then(|val| {
                if val >= 0 && val <= i64::from(u32::MAX) {
                    Some(val as u32)
                } else {
                    None
                }
            }),

            // JSON: as_u64()
            UnifiedValue::Json(j) => j.as_u64().and_then(|val| {
                if val <= u64::from(u32::MAX) {
                    Some(val as u32)
                } else {
                    None
                }
            }),

            // YAML: i64
            UnifiedValue::Yaml(y) => match y.as_i64() {
                Some(i) if i >= 0 && i <= i64::from(u32::MAX) => Some(i as u32),
                _ => None,
            },
        }
    }

    fn as_array(&self) -> Option<Vec<UnifiedValue>> {
        match self {
            UnifiedValue::Toml(t) => {
                if let toml::Value::Array(arr) = t {
                    Some(
                        arr.iter()
                            .map(|el| UnifiedValue::Toml(el.clone()))
                            .collect(),
                    )
                } else {
                    None
                }
            }
            UnifiedValue::Json(j) => {
                if let Some(arr) = j.as_array() {
                    Some(
                        arr.iter()
                            .map(|el| UnifiedValue::Json(el.clone()))
                            .collect(),
                    )
                } else {
                    None
                }
            }
            UnifiedValue::Yaml(y) => {
                if let serde_yaml::Value::Sequence(seq) = y {
                    Some(
                        seq.iter()
                            .map(|el| UnifiedValue::Yaml(el.clone()))
                            .collect(),
                    )
                } else {
                    None
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Checking the magic keyword
// -----------------------------------------------------------------------------

fn contains_magic_keyword(value: &UnifiedValue) -> bool {
    if let Some(keywords_array) = (*value)
        .get_path(&["package", "keywords"])
        .and_then(|v| v.as_array())
    {
        keywords_array
            .iter()
            .any(|val| val.as_str() == Some("Christmas 2024"))
    } else {
        false
    }
}

// -----------------------------------------------------------------------------
// Extracting valid orders ignoring invalid fields
// -----------------------------------------------------------------------------

#[derive(Debug)]
struct ValidOrder {
    item: String,
    quantity: u32,
}

fn extract_valid_orders(value: &UnifiedValue) -> Vec<ValidOrder> {
    let orders_arr = match (*value)
        .get_path(&["package", "metadata", "orders"])
        .and_then(|u| u.as_array())
    {
        Some(arr) => arr,
        None => return vec![],
    };

    let mut out = vec![];
    for entry in orders_arr {
        let item = entry
            .get_path(&["item"])
            .and_then(|v| v.as_str().map(|s| s.to_owned()));

        let qty = entry.get_path(&["quantity"]).and_then(|v| v.as_u32());

        // Validate item and quantity before adding the order
        if let (Some(i), Some(q)) = (item, qty) {
            out.push(ValidOrder {
                item: i,
                quantity: q,
            });
        }
    }
    out
}

// -----------------------------------------------------------------------------
// Convert orders to string
// -----------------------------------------------------------------------------

fn orders_to_string(orders: &[ValidOrder]) -> String {
    orders
        .iter()
        .enumerate()
        .map(|(i, o)| {
            if i == 0 {
                format!("{}: {}", o.item, o.quantity)
            } else {
                format!("\n{}: {}", o.item, o.quantity)
            }
        })
        .collect()
}

// -----------------------------------------------------------------------------
// Shuttle + Actix main
// -----------------------------------------------------------------------------

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
