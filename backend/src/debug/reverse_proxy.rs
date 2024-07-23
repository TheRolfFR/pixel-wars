use actix_web::{web, HttpRequest, HttpResponse, Responder};
use reqwest::Client;

#[allow(dead_code)]
pub async fn proxy_handler(req: HttpRequest, body: web::Bytes) -> impl Responder {
    let client = Client::new();
    let target_url = "http://localhost:5173";

    // Build the URL to forward the request to
    let uri = req.uri().to_string();
    let url = format!("{}{}", target_url, uri);

    // Create a new request
    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes()).unwrap();
    let mut proxy_req = client.request(method, &url);

    // Forward headers
    for (key, value) in req.headers().iter() {
        proxy_req = proxy_req.header(key.to_string(), value.to_str().unwrap());
    }

    // Forward the request body
    let proxy_resp = proxy_req.body(body).send().await;

    match proxy_resp {
        Ok(res) => {
            // Create a new response
            let status_code = res.status().as_u16().try_into().unwrap();
            let mut client_resp = HttpResponse::build(status_code);

            // Forward headers
            for (key, value) in res.headers().iter() {
                client_resp.insert_header((key.to_string().as_bytes(), value.to_str().unwrap()));
            }

            // Forward the response body
            let bytes = res.bytes().await.unwrap_or_default();
            client_resp.body(bytes)
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Failed to forward request: {}", err)),
    }
}
