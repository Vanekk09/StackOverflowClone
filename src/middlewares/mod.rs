use std::time::Instant;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

pub async fn request_timer(
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();

    let method = req.method().clone();
    let path = req.uri().path().to_owned();

    let response = next.run(req).await;

    let duration = start.elapsed();

    info!(
        target: "request_timer",
        "Request completed - Method: {}, Path: {}, Duration: {:.2?}, Status: {}",
        method,
        path,
        duration,
        response.status()
    );

    response
}