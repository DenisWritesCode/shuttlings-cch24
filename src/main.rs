use actix_web::{get, http::header::LOCATION, web::ServiceConfig, HttpResponse, Responder};
use shuttle_actix_web::ShuttleActixWeb;

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

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_world);
        cfg.service(seek);
    };

    Ok(config.into())
}
